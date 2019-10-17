use std::env;
use std::process;

use carvers::Config;

fn main() {
    println!(
        r"
   _____          _______      ________   _____   _____ 
  / ____|   /\   |  __ \ \    / /  ____| |  __ \ / ____|
 | |       /  \  | |__) \ \  / /| |__    | |__) | (___  
 | |      / /\ \ |  _  / \ \/ / |  __|   |  _  / \___ \ 
 | |____ / ____ \| | \ \  \  /  | |____ _| | \ \ ____) |
  \_____/_/    \_\_|  \_\  \/   |______(_)_|  \_\_____/ 
"
    );

    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    println!("{:?}", config);
}
