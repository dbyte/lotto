use std::{thread, time};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use rand::Rng;

use super::rules::{MAX_SERIES_LENGTH, SERIES_NUMBER_RANGE};

pub static HAS_WON: AtomicBool = AtomicBool::new(false);

struct Outcome {
   single_game: [u8; MAX_SERIES_LENGTH + 1],
   num_tries: usize,
   last_poll: time::Instant,
}

impl Outcome {
   fn new() -> Self {
      Self {
         single_game: [0; MAX_SERIES_LENGTH + 1],
         num_tries: 0,
         last_poll: time::Instant::now(),
      }
   }

   fn extract_single_game_series(&self) -> &[u8] {
      &self.single_game[..self.single_game.len() - 1]
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
   pub my_series: [u8; MAX_SERIES_LENGTH],
   pub my_superzahl: u8,
   pub sender: mpsc::Sender<String>,
}

impl Guess {
   pub fn new(series: [u8; MAX_SERIES_LENGTH],
              superzahl: u8,
              sender: mpsc::Sender<String>) -> Self {
      Self {
         my_series: series, // Example: [1, 45, 38, 5, 23, 19]
         my_superzahl: superzahl,
         sender,
      }
   }

   pub fn run_games_until_win(&self) -> usize {
      let mut outcome = Outcome::new();

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
         let pulled_number = Self::pull_single_number(result);
         result.single_game[i] = pulled_number;
      }
   }

   fn pull_single_number(result: &mut Outcome) -> u8 {
      let pulled_number: u8 = rand::thread_rng().gen_range(SERIES_NUMBER_RANGE);

      if !result.single_game.contains(&pulled_number) {
         pulled_number
      } else {
         Self::pull_single_number(result)
      }
   }

   fn max_pulls(&self) -> usize {
      // + 1 designates the Superzahl.
      self.my_series.len() + 1
   }

   fn my_series_contains_all_of(&self, slice: &[u8]) -> bool {
      slice.iter().all(|item| self.my_series.contains(item))
   }
}
