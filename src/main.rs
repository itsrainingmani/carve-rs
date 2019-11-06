use std::env;
use std::process;

use carvers::Config;

fn main() {
    println!(
        r"
   _____          _______      ________            _____   _____ 
  / ____|   /\   |  __ \ \    / /  ____|          |  __ \ / ____|
 | |       /  \  | |__) \ \  / /| |__     ______  | |__) | (___  
 | |      / /\ \ |  _  / \ \/ / |  __|   |______| |  _  / \___ \ 
 | |____ / ____ \| | \ \  \  /  | |____           | | \ \ ____) |
  \_____/_/    \_\_|  \_\  \/   |______|          |_|  \_\_____/ 
"
    );

    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    println!("{:?}", config);

    if let Err(e) = carvers::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}
