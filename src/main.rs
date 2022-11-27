use std::env;

use lotto::runner::Runner;

fn main() {
   env::set_var("RUST_LOG", "DEBUG");
   env_logger::init();

   loop {
      if Runner::default().run().is_ok() { break; }
   }
}
