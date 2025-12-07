use std::{fmt::Display, io::Write, process};

use crate::{
    builtin::{BuiltinCommand, ExitCode, BUILTIN_COMMANDS},
    executable::Executable,
    Result,
};

pub trait Execute {
    fn execute<O, E>(&self, output_writer: O, error_writer: E) -> ExitCode
    where
        O: Write,
        E: Write,
        process::Stdio: From<O> + From<E>;
}

pub trait Parse {
    fn parse(command: &str, args: &[&str]) -> Result<Self>
    where
        Self: std::marker::Sized;
}

pub type Args = Vec<String>;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Empty,
    BuiltinCommand(BuiltinCommand),
    Executable(Executable),
    Unknown(UnknownCommand),
}

impl Parse for Command {
    fn parse(command: &str, args: &[&str]) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        let command = if command.is_empty() {
            Command::Empty
        } else if BUILTIN_COMMANDS.contains(command) {
            Command::BuiltinCommand(BuiltinCommand::parse(command, args)?)
        } else if let Ok(exec) = Executable::parse(command, args) {
            Command::Executable(exec)
        } else {
            Command::Unknown(UnknownCommand::new(
                command.to_string(),
                args.iter().map(|arg| arg.to_string()).collect(),
            ))
        };
        Ok(command)
    }
}

impl Execute for Command {
    fn execute<O, E>(&self, output_writer: O, mut error_writer: E) -> ExitCode
    where
        O: Write,
        E: Write,
        process::Stdio: From<O> + From<E>,
    {
        match self {
            Command::Empty => 0,
            Command::BuiltinCommand(builtin_command) => {
                builtin_command.execute(output_writer, error_writer)
            }
            Command::Executable(exec) => exec.execute(output_writer, error_writer),
            Command::Unknown(unknown) => {
                let _ = writeln!(error_writer, "{}: command not found", unknown.command);
                0
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct UnknownCommand {
    pub command: String,
    pub args: Args,
}

impl UnknownCommand {
    pub fn new(command: String, args: Args) -> Self {
        Self { command, args }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseCommandError {
    LessArgs(String, Args, usize),
    MoreArgs(String, Args, usize),
}

impl Display for ParseCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseCommandError::LessArgs(cmd, _args, _tgt_num) => {
                write!(f, "{}: not enough arguments", cmd)
            }
            ParseCommandError::MoreArgs(cmd, _args, _tgt_num) => {
                write!(f, "{}: too many arguments", cmd)
            }
        }
    }
}

impl std::error::Error for ParseCommandError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        assert!(matches!(Command::parse("", &[]), Ok(Command::Empty)));
    }

    #[test]
    fn test_parse_unknown() {
        assert_eq!(
            Command::parse("invalid_command", &["invalid", "args"]).unwrap(),
            Command::Unknown(UnknownCommand::new(
                "invalid_command".to_string(),
                vec!["invalid".to_string(), "args".to_string()]
            ))
        );
    }

    #[test]
    fn test_parse_exit() {
        assert_eq!(
            Command::parse("exit", &[]).unwrap(),
            Command::BuiltinCommand(BuiltinCommand::Exit(0))
        );
        assert_eq!(
            Command::parse("exit", &["123"]).unwrap(),
            Command::BuiltinCommand(BuiltinCommand::Exit(123))
        );
    }
}
