use std::ops::RangeInclusive;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;

use rand::Rng;

const MAX_SERIES_LENGTH: usize = 6;
const SERIES_NUMBER_RANGE: RangeInclusive<u8> = 1..=49;

pub static HAS_WON: AtomicBool = AtomicBool::new(false);

struct Outcome {
   single_game: Vec<u8>,
   num_tries: usize,
}

impl Outcome {
   fn new(max_pulls: usize) -> Self {
      Outcome {
         single_game: vec![0; max_pulls],
         num_tries: 0,
      }
   }
}

#[derive(Clone)]
pub struct Guess {
   // This struct is expected to be immutable.
   pub my_series: Vec<u8>,
   pub my_superzahl: u8,
   pub tx: mpsc::Sender<String>,
}

impl Guess {
   pub fn new(series: Vec<u8>, superzahl: u8, tx: mpsc::Sender<String>) -> Self {
      Guess {
         my_series: series, // Example: vec![1, 45, 38, 5, 23, 19]
         my_superzahl: superzahl,
         tx,
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

   pub fn run_games_until_win(&self) {
      let mut outcome = Outcome::new(self.max_pulls());
      let mut pulled_series: &[u8];
      let mut pulled_superzahl: &u8;
      let mut i: usize = 0;

      loop {
         if self.is_finished(&outcome) { break; }

         self.run_single_game(&mut outcome);
         outcome.num_tries += 1;
         i += 1;

         pulled_series = &outcome.single_game[..self.max_pulls() - 1];

         if i % 20_000_000 == 0 {
            self.tx.send(outcome.num_tries.to_string()).unwrap();
            i = 0;
         }

         if self.my_series_contains_all_of(pulled_series) {
            pulled_superzahl = outcome.single_game.last().unwrap();

            if self.my_superzahl != *pulled_superzahl {
               // The series is matching but Superzahl is not.
               continue;
            }

            log::info!("🏖 You won! 🍀 {:?} -- Superzahl: {}",
               pulled_series, pulled_superzahl);

            log::info!("{:?} pulled your guess after {} games.",
               thread::current().id(), outcome.num_tries);

            HAS_WON.store(true, Ordering::SeqCst);
         }
      }
   }

   fn is_finished(&self, outcome: &Outcome) -> bool {
      if !HAS_WON.load(Ordering::SeqCst) {
         return false;
      }

      self.tx.send(format!(
         "{:?} played {} games.",
         thread::current().id(),
         outcome.num_tries))
         .unwrap();

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
