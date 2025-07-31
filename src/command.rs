use std::fmt::Display;

use crate::Result;

pub enum Command {
    Unknown(UnknownCommand),
    Exit(i32),
}

impl Command {
    pub fn parse(s: &str) -> Result<Self> {
        let cmd_vec: Vec<&str> = s.trim().split(' ').collect();
        if cmd_vec.is_empty() {
            return Err(ParseCommandError::NoCommand(s.to_string()).into());
        }

        let command = match cmd_vec[0] {
            "exit" => {
                if cmd_vec.len() > 2 {
                    return Err(ParseCommandError::MoreArgs(
                        cmd_vec[0].to_string(),
                        Args::from_str_slice(&cmd_vec[1..]),
                        1,
                    )
                    .into());
                }

                let exit_code: i32 = if cmd_vec.len() == 1 {
                    0
                } else {
                    cmd_vec[1].parse()?
                };
                Command::Exit(exit_code)
            }
            _ => Command::Unknown(UnknownCommand {
                command: cmd_vec[0].to_string(),
                args: Args::from_str_slice(&cmd_vec[1..]),
            }),
        };
        Ok(command)
    }
}

#[derive(Debug)]
pub struct UnknownCommand {
    pub command: String,
    pub args: Args,
}

impl UnknownCommand {
    pub fn new(command: String, args: Args) -> Self {
        Self { command, args }
    }
}

#[derive(Debug)]
pub struct Args {
    inner: Vec<String>,
}

impl Args {
    pub fn new(inner: Vec<String>) -> Self {
        Self { inner }
    }

    pub fn from_str_slice(slice: &[&str]) -> Self {
        Self {
            inner: slice.iter().map(|s| s.trim().to_string()).collect(),
        }
    }
}

pub struct ExitCommand {
    pub exit_code: u8,
}

impl ExitCommand {
    pub fn new(exit_code: u8) -> Self {
        Self { exit_code }
    }
}

#[derive(Debug)]
pub enum ParseCommandError {
    NoCommand(String),
    LessArgs(String, Args, usize),
    MoreArgs(String, Args, usize),
}

impl Display for ParseCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) //TODO 完善
    }
}

impl std::error::Error for ParseCommandError {}
