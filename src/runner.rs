use std::{process, thread};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;

use log;

use crate::core::Guess;

pub struct Runner {
   num_played_games_until_win: usize,
   receiver: Option<Receiver<String>>,
}

impl Runner {
   pub fn new() -> Self {
      Runner {
         num_played_games_until_win: 0,
         receiver: None,
      }
   }

   pub fn run(&mut self, my_series: Vec<u8>, my_superzahl: u8) {
      let max_parallel = thread::available_parallelism().unwrap().get();
      log::debug!("START games with {} parallel threads.", max_parallel-1);

      // Create a channel for n:1 thread communication
      let (sender, receiver) = mpsc::channel();
      self.receiver = Some(receiver);

      // Create the main guess game
      let origin_guess = Guess::new(my_series, my_superzahl, sender);
      self.validate(&origin_guess);

      // Create a vector for thread completion waiting
      let mut joinhandles = vec![];

      // Spawn max. available threads minus main thread.
      for _ in 1..max_parallel {
         let guess = origin_guess.clone();

         let handle = thread::spawn(move || {
            // Run games until player wins (or a different thread solved the task).
            // fn runs until player has won in this or in other threads:
            return guess.run_games_until_win();
         });

         joinhandles.push(handle)
      }

      // Important: Explicitly drop origin_guess.sender instance before calling
      // receive_and_wait(), otherwise the async channel never gets closed which would
      // lead to an endless receiver loop there.
      drop(origin_guess.sender);

      // Signal OS that it may schedule other threads on the CPU instead of this
      // main thread. Nearly doubles game performance (at least on macOS arm64).
      thread::yield_now();

      // Stay tuned for worker thread messages
      self.receive_messages();

      self.collect_results(joinhandles);
      self.print_summary();
   }

   fn validate(&self, origin_guess: &Guess) {
      match origin_guess.validate() {
         Ok(()) => {
            log::info!("Your guess: {:?} -- Superzahl: {}",
               origin_guess.my_series, origin_guess.my_superzahl);
         }

         Err(messages) => {
            log::warn!("Please try again. Your guess was: {:?} -- Superzahl: {}",
            origin_guess.my_series, origin_guess.my_superzahl);

            for message in messages {
               log::error!("{}", message);
            }

            process::exit(1);
         }
      }
   }

   fn receive_messages(&self) {
      // Blocks as long as there is at least 1 active sender.
      // Guard
      if self.receiver.is_none() {
         panic!("Channel-receiver not initialized. Sent Messages from worker \
         threads can't be evaluated.");
      }

      // Wait for downstream messages of the async mpsc channel.
      let receiver = self.receiver.as_ref().unwrap();
      for received in receiver {
         let msg = received;
         // Emit every received message of the channel, sent by any thread.
         log::info!("{}", msg);
      }
      log::debug!("mpsc channel closed. Waiting for worker threads to tear down.");
   }

   fn collect_results(&mut self, handles: Vec<JoinHandle<usize>>) {
      for handle in handles {
         let thread_id = handle.thread().id();

         // Note: join() is blocking
         let num_games_per_thread = handle.join().unwrap();
         log::debug!("{:?} closed. Played {} games.", thread_id, num_games_per_thread);

         self.num_played_games_until_win += num_games_per_thread;
      }

      log::debug!("All worker threads deallocated.");
   }

   fn print_summary(&self) {
      log::info!("{}", "~".repeat(60));
      log::info!("🤘 Summary: Played {} games until win.", self.num_played_games_until_win);
      log::info!("{}", "~".repeat(60));
   }
}
