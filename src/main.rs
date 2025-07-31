#![allow(dead_code)]

use std::io::{self, Write};

mod utils;

fn main() {
    utils::config_logger();

    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let mut input = String::new();
    loop {
        match io::stdin().read_line(&mut input) {
            Ok(_) => println!("{}: command not found", input.trim()),
            Err(err) => tracing::error!("{}", err),
        }
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
    }
}
