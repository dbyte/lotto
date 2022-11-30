use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::{stdin, stdout, Write};
use std::ops::RangeInclusive;

pub const MAX_SERIES_LENGTH: usize = 6;
pub const SERIES_NUMBER_RANGE: RangeInclusive<u8> = 1..=49;

#[derive(Debug)]
pub struct InvalidGuessError;

impl Display for InvalidGuessError {
   fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
      write!(formatter, "Bad guess.")
   }
}

impl std::error::Error for InvalidGuessError {}

pub struct UserInput {
   series: String,
   superzahl: String,
}

impl UserInput {
   fn new(series: String, superzahl: String) -> Self {
      Self {
         series, // Example: "1, 45, 38, 5, 23, 19"
         superzahl, // Example: "13"
      }
   }

   pub fn create() -> Self {
      // 1. User provides the series guess
      print!("Enter your guess series (max. {} numbers between {} and {}, separated by commas): ",
             MAX_SERIES_LENGTH,
             SERIES_NUMBER_RANGE.start(),
             SERIES_NUMBER_RANGE.end());

      stdout().flush().unwrap();
      let mut input_guess_series = String::new();
      stdin().read_line(&mut input_guess_series).unwrap();

      // 2. User provides the Superzahl guess
      print!("Enter your Superzahl between {} and {}: ",
             SERIES_NUMBER_RANGE.start(),
             SERIES_NUMBER_RANGE.end());

      stdout().flush().unwrap();
      let mut input_superzahl = String::new();
      stdin().read_line(&mut input_superzahl).unwrap();

      Self::new(input_guess_series, input_superzahl)
   }

   pub fn parse(&self) -> Result<(Vec<u8>, u8), InvalidGuessError> {
      let parsed_series: Vec<u8> = self.series
         .trim_matches(|c: char| c == ',' || c.is_whitespace())
         .split(',')
         .map(|s| s.matches(char::is_numeric)
            .fold("".to_string(), |acc: String, nxt: &str| acc + nxt)
            .parse().unwrap_or_default()
         )
         .collect();

      let parsed_superzahl: u8 = self.superzahl
         .matches(char::is_numeric)
         .fold("".to_string(), |acc: String, nxt: &str| acc + nxt)
         .parse().unwrap_or_default();

      log::info!("Your guess: {:?} -- Superzahl: {}", parsed_series, parsed_superzahl);

      match Self::validate(&parsed_series, &parsed_superzahl) {
         Ok(()) => Ok((parsed_series, parsed_superzahl)),

         Err(messages) => {
            for message in &messages {
               log::error!("{}", message);
            }
            log::warn!("Please try again.");
            Err(InvalidGuessError)
         }
      }
   }

   fn validate(series: &[u8], superzahl: &u8) -> Result<(), Vec<String>> {
      let mut messages = Vec::<String>::new();

      if series.is_empty() {
         messages.push("Your guess series has no numbers, which is not allowed.".to_string());
      }

      if series.len() > MAX_SERIES_LENGTH {
         messages.push(format!(
            "Your guess series has {} numbers, which is not allowed. Maximum allowed: {}.",
            series.len(), MAX_SERIES_LENGTH));
      }

      if series.iter().any(|item| !SERIES_NUMBER_RANGE.contains(item)) {
         messages.push(format!(
            "Each number of your guess series must be in a range from {} to {}.",
            SERIES_NUMBER_RANGE.start(), SERIES_NUMBER_RANGE.end()));
      }

      if !SERIES_NUMBER_RANGE.contains(superzahl) {
         messages.push(format!(
            "Your Superzahl must be in a range from {} to {}.",
            SERIES_NUMBER_RANGE.start(), SERIES_NUMBER_RANGE.end()));
      }

      if (1..series.len()).any(|i| series[i..].contains(&series[i - 1])) {
         messages.push("Each number of your series must be unique.".to_string());
      }

      if series.contains(superzahl) {
         messages.push(format!(
            "Your Superzahl {} must not be contained in your guess series.", superzahl));
      }

      if !messages.is_empty() {
         return Err(messages);
      }

      Ok(())
   }
}