#![allow(dead_code)]

use std::io::{self, Write};

use crate::{command::Execute, parse_input::CommandExecution};

mod auto_completion;
mod builtin;
mod command;
mod executable;
mod parse_input;
mod redirect;
mod utils;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() {
    utils::config_logger();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        match parse_input::parse_input() {
            Ok(command_exec_vec) => {
                for CommandExecution {
                    command,
                    output_writer,
                    error_writer,
                } in command_exec_vec
                {
                    command.execute(output_writer, error_writer);
                }
            }
            Err(err) => eprintln!("{}", err),
        }
        io::stdout().flush().unwrap();
    }
}
