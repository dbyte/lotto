use std::{process, thread};
use std::sync::mpsc;
use std::thread::JoinHandle;

use log;

use crate::core::Guess;

pub fn run(my_series: Vec<u8>, my_superzahl: u8) {
   let max_parallel = thread::available_parallelism().unwrap().get();
   log::debug!("START games with {} parallel threads.", max_parallel-1);

   let (sender, receiver) = mpsc::channel();

   let guess = Guess::new(my_series, my_superzahl, sender);
   validate(&guess);

   let mut handles = Vec::new();

   // Spawn max. available threads minus main thread.
   for _ in 1..max_parallel {
      let guess = guess.clone();

      let handle = thread::spawn(move || {
         // Run games until player wins (or a different thread solved the task).
         // fn runs until player has won in this or in other threads:
         return guess.run_games_until_win();
      });

      handles.push(handle)
   }

   // Important: Explicitly drop guess.sender instance before calling receive_and_wait(),
   // otherwise the async channel never gets closed which would lead to an endless
   // receiver loop there.
   drop(guess.sender);

   receive_messages(receiver);
   let overall_num_games = collect_results(handles);

   log::info!("{}", "~".repeat(60));
   log::info!("🤘 Summary: Played {} games until win.", overall_num_games);
   log::info!("{}", "~".repeat(60));
}

fn receive_messages(receiver: mpsc::Receiver<String>) {
   // Wait for downstream messages of the async mpsc channel.
   for received in receiver {
      let msg = received;
      // Emit every received message of the channel, sent by any thread.
      log::info!("{msg}");
   }
   log::debug!("mpsc channel closed. Waiting for worker threads to tear down.");
}

fn collect_results(handles: Vec<JoinHandle<usize>>) -> usize {
   let mut overall_num_games = 0;

   for handle in handles {
      let thread_id = handle.thread().id();
      // Note: join() is blocking
      let num_games = handle.join().unwrap();
      log::debug!("{:?} closed. Played {} games.", thread_id, num_games);
      overall_num_games += num_games
   }

   log::debug!("All worker threads deallocated.");
   overall_num_games
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
