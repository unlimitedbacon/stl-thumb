extern crate stl_thumb;

use std::process;
use stl_thumb::Config;

fn main() {
    let config = Config::new();

    println!("STL File: {}", config.stl_filename);
    println!("Thumbnail File: {}", config.img_filename);

    if let Err(e) = stl_thumb::run(&config) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
