#![allow(dead_code)]

use std::{
    io::{self, Write},
    process::exit,
};

use crate::command::Command;

mod command;
mod utils;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

fn execute_command(input: &str) {
    match Command::parse(input) {
        Ok(command) => match command {
            Command::Exit(exit_code) => exit(exit_code),
            Command::Unknown(unknown_command) => {
                println!("{}: command not found", unknown_command.command)
            }
        },
        Err(err) => println!("Failed to parse input: \"{}\", Error: {}", input.trim(), err),
    }
}

fn main() {
    utils::config_logger();

    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let mut input = String::new();
    loop {
        match io::stdin().read_line(&mut input) {
            Ok(_) => execute_command(&input),
            Err(err) => tracing::error!("{}", err),
        }
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
    }
}
