#![allow(dead_code)]

use std::io::{self, Write};

use crate::command::{Command, Execute, Parse};

mod builtin;
mod command;
mod executable;
mod utils;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

fn parse_input() -> Result<(String, Vec<String>)> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let mut in_single_quote = false;
    let mut new_token = String::new();
    let mut cmd_vec: Vec<String> = vec![];
    for c in input.chars() {
        if c == '\'' {
            in_single_quote = !in_single_quote;
        } else if c.is_whitespace() && !in_single_quote {
            if !new_token.is_empty() {
                cmd_vec.push(new_token.clone());
                new_token.clear();
            }
        } else {
            new_token.push(c);
        }
    }
    if !cmd_vec.is_empty() {
        Ok((cmd_vec[0].clone(), cmd_vec[1..].to_vec()))
    } else {
        Ok(("".to_string(), cmd_vec))
    }
}

fn execute_command(command: &str, args: &[&str]) {
    match Command::parse(command, args) {
        Ok(command) => command.execute(),
        Err(err) => tracing::error!(
            "Failed to parse command: \"{}\", args: \"{:?}\", Error: {}",
            command,
            args,
            err
        ),
    }
}

fn main() {
    utils::config_logger();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        match parse_input() {
            Ok((command, args)) => execute_command(
                &command,
                &args.iter().map(|arg| arg.as_str()).collect::<Vec<&str>>(),
            ),
            Err(err) => tracing::error!(err),
        }
        io::stdout().flush().unwrap();
    }
}
