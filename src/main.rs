use std::env;

use lotto::runner::Runner;

fn main() {
   env::set_var("RUST_LOG", "DEBUG");
   env_logger::init();
   Runner::default().run();
}
