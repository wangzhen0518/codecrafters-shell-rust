#![allow(dead_code)]

use std::io::{self, Write};

use crate::command::{Command, Execute, Parse};

mod builtin;
mod command;
mod executable;
mod utils;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

fn parse_input_from_reader<R: io::BufRead>(reader: &mut R) -> Result<(String, Vec<String>)> {
    let mut input = String::new();
    reader.read_line(&mut input)?;
    // io::stdin().read_line(&mut input)?;

    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut new_token = String::new();
    let mut cmd_vec: Vec<String> = vec![];

    for c in input.chars() {
        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
        } else if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
        } else if c.is_whitespace() && !in_single_quote && !in_double_quote {
            if !new_token.is_empty() {
                cmd_vec.push(new_token.clone());
                new_token.clear();
            }
        } else {
            new_token.push(c);
        }
    }
    if !new_token.is_empty() {
        cmd_vec.push(new_token.clone());
        new_token.clear();
    }

    if !cmd_vec.is_empty() {
        Ok((cmd_vec[0].clone(), cmd_vec[1..].to_vec()))
    } else {
        Ok(("".to_string(), cmd_vec))
    }
}

fn parse_input() -> Result<(String, Vec<String>)> {
    parse_input_from_reader(&mut io::stdin().lock())
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

#[cfg(test)]
mod tests {

    use super::*;

    fn test_parse(input: &str, target: &[&str]) {
        assert_eq!(
            parse_input_from_reader(&mut io::Cursor::new(input.as_bytes())).unwrap(),
            (
                target[0].to_string(),
                target[1..].iter().map(|s| s.to_string()).collect()
            )
        );
    }

    #[test]
    fn test_parse_native() {
        test_parse("echo shell   hello", &["echo", "shell", "hello"]);
    }

    #[test]
    fn test_parse_single_quote() {
        test_parse("echo 'shell   hello'", &["echo", "shell   hello"]);
        test_parse("echo 'shell''hello'", &["echo", "shellhello"]);
        test_parse("echo shell''hello", &["echo", "shellhello"]);
        test_parse(
            "cat '/tmp/file name' '/tmp/file name with spaces'",
            &["cat", "/tmp/file name", "/tmp/file name with spaces"],
        );
    }

    #[test]
    fn test_parse_native_double_quote() {
        test_parse("echo \"shell   hello\"", &["echo", "shell   hello"]);
        test_parse("echo \"shell\"\"hello\"", &["echo", "shellhello"]);
        test_parse("echo shell\"\"hello", &["echo", "shellhello"]);
        test_parse("echo \"shell's test\"", &["echo", "shell's test"]);
        test_parse(
            "cat \"/tmp/file name\" \"/tmp/'file name' with spaces\"",
            &["cat", "/tmp/file name", "/tmp/'file name' with spaces"],
        );
    }
}
