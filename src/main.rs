use std::env;

use lotto::runner::Runner;

fn main() {
   env::set_var("RUST_LOG", "DEBUG");
   env_logger::init();

   let _args: Vec<String> = env::args().collect();

   let my_series: Vec<u8> = vec![1, 45, 37, 22, 19, 36];
   let my_superzahl: u8 = 13;

   Runner::new().run(my_series, my_superzahl);
}
