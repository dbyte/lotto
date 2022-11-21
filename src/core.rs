use std::{thread, time};
use std::ops::RangeInclusive;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use rand::Rng;

const MAX_SERIES_LENGTH: usize = 6;
const SERIES_NUMBER_RANGE: RangeInclusive<u8> = 1..=49;

pub static HAS_WON: AtomicBool = AtomicBool::new(false);

struct Outcome {
   single_game: Vec<u8>,
   num_tries: usize,
   last_poll: time::Instant,
}

impl Outcome {
   fn new(max_pulls: usize) -> Self {
      Outcome {
         single_game: vec![0; max_pulls],
         num_tries: 0,
         last_poll: time::Instant::now(),
      }
   }

   fn extract_single_game_series(&self) -> &[u8] {
      &self.single_game[..&self.single_game.len() - 1]
   }

   fn extract_single_game_superzahl(&self) -> &u8 {
      &self.single_game[self.single_game.len() - 1]
   }

   fn publish(&mut self, sender: &mpsc::Sender<String>) {
      let now = time::Instant::now();
      let diff = now - self.last_poll;

      if diff > Duration::from_secs(3) {
         sender.send(format!("{:?} running: {} iterations",
                             thread::current().id(),
                             self.num_tries)).unwrap();
         self.last_poll = now;
      }
   }
}

#[derive(Clone)]
pub struct Guess {
   // This struct is expected to be immutable.
   pub my_series: Vec<u8>,
   pub my_superzahl: u8,
   pub sender: mpsc::Sender<String>,
}

impl Guess {
   pub fn new(series: Vec<u8>, superzahl: u8, sender: mpsc::Sender<String>) -> Self {
      Guess {
         my_series: series, // Example: vec![1, 45, 38, 5, 23, 19]
         my_superzahl: superzahl,
         sender,
      }
   }

   pub fn validate(&self) -> Result<(), Vec<String>> {
      let mut messages = Vec::<String>::new();

      if self.my_series.len() == 0 {
         messages.push("Your guess series has no numbers, which is not allowed.".to_string());
      }

      if self.my_series.len() > MAX_SERIES_LENGTH {
         messages.push(format!(
            "Your guess series has {} numbers, which is not allowed. Maximum allowed: {}.",
            self.my_series.len(), MAX_SERIES_LENGTH));
      }

      if self.my_series.iter().any(|item| !SERIES_NUMBER_RANGE.contains(&item)) {
         messages.push(format!(
            "Each number of your guess series must be in a range from {} to {}.",
            SERIES_NUMBER_RANGE.start(), SERIES_NUMBER_RANGE.end()));
      }

      if !SERIES_NUMBER_RANGE.contains(&self.my_superzahl) {
         messages.push(format!(
            "Your Superzahl must be in a range from {} to {}.",
            SERIES_NUMBER_RANGE.start(), SERIES_NUMBER_RANGE.end()));
      }

      if self.my_series.contains(&self.my_superzahl) {
         messages.push(format!(
            "Your Superzahl {} must not be contained in your guess series.", self.my_superzahl));
      }

      if !messages.is_empty() {
         return Err(messages);
      }

      Ok(())
   }

   pub fn run_games_until_win(&self) -> usize {
      let mut outcome = Outcome::new(self.max_pulls());

      loop {
         if self.has_finished() { break; }
         self.run_single_game(&mut outcome);

         outcome.num_tries += 1;
         outcome.publish(&self.sender);

         if self.my_series_contains_all_of(outcome.extract_single_game_series()) {
            if self.my_superzahl != *outcome.extract_single_game_superzahl() {
               // The series is matching but Superzahl is not.
               continue;
            }

            // Player wins!
            self.on_win(&outcome);
         }
      }

      outcome.num_tries
   }

   fn on_win(&self, outcome: &Outcome) {
      // Usually called just one time per guess and only for the thread
      // which solved the game. However, it's not guaranteed - especially
      // if there are less than 5 numbers in the guessed series. In other words,
      // multiple threads may solve the guess at the same time.
      self.sender.send("~".repeat(60)).unwrap();

      self.sender.send(format!(
         "🏖 You won! 🍀 {:?} -- Superzahl: {}",
         outcome.extract_single_game_series(),
         outcome.extract_single_game_superzahl())).unwrap();

      self.sender.send(format!(
         "🏖 {:?} pulled your guess after {} games.",
         thread::current().id(), outcome.num_tries)).unwrap();

      self.sender.send("~".repeat(60)).unwrap();

      HAS_WON.store(true, Ordering::SeqCst);
   }

   fn has_finished(&self) -> bool {
      if !HAS_WON.load(Ordering::SeqCst) {
         return false;
      }
      true
   }

   fn run_single_game(&self, result: &mut Outcome) {
      result.single_game.fill_with(Default::default);

      for i in 0..self.max_pulls() {
         let pulled_number = self.pull_single_number(result);
         result.single_game[i] = pulled_number;
      }
   }

   fn pull_single_number(&self, result: &mut Outcome) -> u8 {
      let pulled_number: u8 = rand::thread_rng().gen_range(SERIES_NUMBER_RANGE);

      if !result.single_game.contains(&pulled_number) {
         return pulled_number;
      } else {
         self.pull_single_number(result)
      }
   }

   fn max_pulls(&self) -> usize {
      // + 1 designates the Superzahl.
      self.my_series.len() + 1
   }

   fn my_series_contains_all_of(&self, slice: &[u8]) -> bool {
      slice.iter().all(|item| self.my_series.contains(&item))
   }
}
