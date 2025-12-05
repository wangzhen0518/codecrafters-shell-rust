use std::{collections::HashSet, fmt::Display};

use lazy_static::lazy_static;

use crate::Result;

pub enum Command {
    Empty,
    Echo(String),
    Type(Type),
    Exit(i32),
    Unknown(UnknownCommand),
}

impl Command {
    pub fn parse(s: &str) -> Result<Self> {
        let cmd_vec: Vec<String> = s.trim().split(' ').map(|s| s.to_string()).collect();
        let (command, args) = (cmd_vec[0].to_string(), cmd_vec[1..].to_vec());

        let command = match command.as_str() {
            "" => Command::Empty,
            "echo" => {
                let content = if !args.is_empty() {
                    args.join(" ").to_string()
                } else {
                    "\n".to_string()
                };
                Command::Echo(content)
            }
            "type" => {
                // TODO 能否统一 check arg num 过程？
                if args.is_empty() {
                    return Err(ParseCommandError::LessArgs(command, args, 1).into());
                }

                Command::Type(Type::parse(args[0].as_str()))
            }
            "exit" => {
                if args.len() > 1 {
                    return Err(ParseCommandError::MoreArgs(command, args, 1).into());
                }

                let exit_code: i32 = if args.is_empty() { 0 } else { args[0].parse()? };
                Command::Exit(exit_code)
            }
            _ => Command::Unknown(UnknownCommand { command, args }),
        };
        Ok(command)
    }
}

#[derive(Debug)]
pub enum Type {
    BuiltinCommand(String),
    UnrecognizedCommad(String),
}

lazy_static! {
    static ref BUILTIN_COMMANDS: HashSet<&'static str> = HashSet::from(["echo", "type", "exit"]);
}

impl Type {
    fn parse(cmd: &str) -> Type {
        let cmd = cmd.to_string();
        if BUILTIN_COMMANDS.contains(cmd.as_str()) {
            Type::BuiltinCommand(cmd)
        } else {
            Type::UnrecognizedCommad(cmd)
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::BuiltinCommand(cmd) => write!(f, "{} is a shell builtin", cmd),
            Type::UnrecognizedCommad(cmd) => write!(f, "{}: not found", cmd),
        }
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

type Args = Vec<String>;

#[derive(Debug)]
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
