use std::{process, thread};
use std::sync::mpsc;
use std::thread::JoinHandle;

use crate::core::Guess;

pub fn run(my_series: Vec<u8>, my_superzahl: u8) {
   let max_parallel = thread::available_parallelism().unwrap().get();
   log::info!("START games with {} parallel threads.", max_parallel-1);

   let (tx, rx) = mpsc::channel();

   let guess = Guess::new(my_series, my_superzahl, tx);
   validate(&guess);

   let mut handles = Vec::new();

   // Spawn max. available threads minus main thread.
   for _ in 1..max_parallel {
      let guess = guess.clone();

      let handle = thread::spawn(move || {
         // Run games until player wins (or a different thread solved the task).
         // Blocking until player wins:
         guess.run_games_until_win();
      });

      handles.push(handle)
   }

   // Explicitly drop guess.tx instance before calling receive_and_wait(),
   // otherwise the channel never gets closed which would lead to endless
   // receiver loop there.
   drop(guess.tx);

   receive_and_wait(rx, handles);
}

fn receive_and_wait(receiver: mpsc::Receiver<String>, joinhandles: Vec<JoinHandle<()>>) {
   // Wait for signals.
   for received in receiver {
      let msg = received;
      // Emit every received message of the channel
      log::info!("{msg}");
   }
   log::info!("mpsc channel closed.");

   for handle in joinhandles {
      handle.join().unwrap();
   }
}

fn validate(guess: &Guess) {
   match guess.validate() {
      Ok(()) => {
         log::info!("Your guess: {:?} -- Superzahl: {}", guess.my_series, guess.my_superzahl);
      }

      Err(messages) => {
         log::warn!("Please try again. Your guess was: {:?} -- Superzahl: {}",
            guess.my_series, guess.my_superzahl);

         for message in messages {
            log::error!("{}", message);
         }

         process::exit(1);
      }
   }
}
