use std::{thread, time};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;

use log;

use super::game::Guess;
use super::rules;

pub struct Runner {
   num_played_games_until_win: usize,
   start_time: time::Instant,
   end_time: time::Instant,
   receiver: Option<Receiver<String>>,
}

impl Default for Runner {
   fn default() -> Self {
      let now = time::Instant::now();
      Self {
         num_played_games_until_win: 0,
         start_time: now,
         end_time: now,
         receiver: None,
      }
   }
}

impl Runner {
   pub fn run(&mut self) -> Result<(), rules::InvalidGuessError> {
      // Parse & validate user's guess. Return early on invalid guess.
      let (series, superzahl) = rules::UserInput::create().parse()?;

      // Create a channel for n:1 thread communication
      let (sender, receiver) = mpsc::channel();
      self.receiver = Some(receiver);

      // Create the main guess game
      let origin_guess = Guess::new(series, superzahl, sender);

      let num_threads = 32; // thread::available_parallelism().unwrap().get();
      // Drop one for the main thread.
      log::debug!("START games with {} parallel worker threads.", num_threads-1);

      // Create a vector for thread completion handling
      let mut joinhandles = vec![];

      // Start timer
      self.start_time = time::Instant::now();

      // Spawn max. available threads minus main thread.
      for _ in 1..num_threads {
         let guess = origin_guess.clone();

         let handle = thread::spawn(move || {
            // Run games until player wins (or a different thread solved the task).
            // fn runs until player has won in this or in other threads:
            guess.run_games_until_win()
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

      Ok(())
   }

   fn receive_messages(&mut self) {
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
         let thread_id = &handle.thread().id();

         // Note: join() is blocking
         let num_games_per_thread = &handle.join().unwrap();
         self.num_played_games_until_win += num_games_per_thread;

         log::debug!("{:?} closed. Played {} games.", thread_id, num_games_per_thread);
      }

      self.end_time = time::Instant::now();
      log::debug!("All worker threads deallocated.");
   }

   fn duration_seconds(&self) -> usize {
      (self.end_time - self.start_time).as_secs() as usize
   }

   fn games_per_second(&self) -> usize {
      self.num_played_games_until_win
         .checked_div(self.duration_seconds())
         .unwrap_or_default()
   }

   fn print_summary(&self) {
      log::info!("{}", "~".repeat(60));
      log::info!("🤘 Summary: Played {} games until win.", self.num_played_games_until_win);
      log::debug!("Duration: about {:?} seconds.", self.duration_seconds());
      log::debug!("Games per second: {:?}.", self.games_per_second());
      log::info!("{}", "~".repeat(60));
   }
}
