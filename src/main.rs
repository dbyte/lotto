use std::env;

use lotto::runner;

fn main() {
   env::set_var("RUST_LOG", "INFO");
   env_logger::init();

   let _args: Vec<String> = env::args().collect();

   let my_series: Vec<u8> = vec![1, 45, 30, 39, 22, 10];
   let my_superzahl: u8 = 13;

   runner::run(my_series, my_superzahl);
}
