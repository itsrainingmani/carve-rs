use std::env;
use std::process;

use seamstress::Config;

fn main() {
    println!("Welcome to Seamstress");

    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    println!("{:?}", config);

    if let Err(e) = seamstress::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}
