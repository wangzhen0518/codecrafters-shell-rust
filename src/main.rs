#![allow(dead_code)]

use std::io::{self, Write};

use crate::command::{Command, Execute, Parse};

mod builtin;
mod command;
mod executable;
mod parse_input;
mod utils;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

fn execute_command(command: &str, args: &[&str]) {
    match Command::parse(command, args) {
        Ok(command) => {
            command.execute(io::stdout(), io::stderr());
        }
        Err(err) => eprintln!(
            "Failed to parse command: \"{}\", args: \"{:?}\", Error: {}",
            command, args, err
        ),
    }
}

fn main() {
    utils::config_logger();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        match parse_input::parse_input() {
            Ok((command, args)) => execute_command(
                &command,
                &args.iter().map(|arg| arg.as_str()).collect::<Vec<&str>>(),
            ),
            Err(err) => eprintln!("{}", err),
        }
        io::stdout().flush().unwrap();
    }
}
