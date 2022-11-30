use std::env;

use crate::core::runner::Runner;

mod core;

fn main() {
   env::set_var("RUST_LOG", "DEBUG");
   env_logger::init();

   loop {
      if Runner::default().run().is_ok() { break; }
   }
}
