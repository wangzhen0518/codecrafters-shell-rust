use std::fmt::Display;

use crate::{
    builtin::{BuiltinCommand, BUILTIN_COMMANDS},
    Result,
};

pub trait Execute {
    fn execute(&self);
}

// pub trait Parse {
//     fn parse(command: &str, args: &[&str]) -> Result<Command>;
// }

pub type Args = Vec<String>;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    // TODO 修改逻辑 判断是否是 builtin -> 是 -> 解析 builtin 并执行
    // TODO 否 -> PATH 中寻找可执行文件 -> 找到，执行可执行文件
    // TODO                             -> 未找到，报错无效命令
    Empty,
    BuiltinCommand(BuiltinCommand),
    Unknown(UnknownCommand),
}

impl Command {
    pub fn parse(s: &str) -> Result<Self> {
        let cmd_vec: Vec<String> = s.trim().split(' ').map(|s| s.to_string()).collect();
        let (command, args) = (cmd_vec[0].to_string(), cmd_vec[1..].to_vec());
        let command = if command.is_empty() {
            Command::Empty
        } else if BUILTIN_COMMANDS.contains(command.as_str()) {
            Command::BuiltinCommand(BuiltinCommand::parse(command, args)?)
        } else {
            Command::Unknown(UnknownCommand::new(command, args))
            // unimplemented!() //TODO
        };
        Ok(command)
    }
}

impl Execute for Command {
    fn execute(&self) {
        match self {
            Command::Empty => {}
            Command::BuiltinCommand(builtin_command) => builtin_command.execute(),
            Command::Unknown(unknown_command) => {
                println!("{}: command not found", unknown_command.command)
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
    use crate::builtin::Type;

    use super::*;

    #[test]
    fn test_parse_empty() {
        assert!(matches!(Command::parse(""), Ok(Command::Empty)));
    }

    #[test]
    fn test_parse_unknown() {
        assert_eq!(
            Command::parse("invalid_command invalid args").unwrap(),
            Command::Unknown(UnknownCommand::new(
                "invalid_command".to_string(),
                vec!["invalid".to_string(), "args".to_string()]
            ))
        );
    }

    #[test]
    fn test_parse_echo() {
        assert_eq!(
            Command::parse("echo").unwrap(),
            Command::BuiltinCommand(BuiltinCommand::Echo("\n".to_string()))
        );
        assert_eq!(
            Command::parse("echo abc  123").unwrap(),
            Command::BuiltinCommand(BuiltinCommand::Echo("abc  123".to_string()))
        );
    }

    #[test]
    fn test_parse_type_error() {
        assert_eq!(
            Command::parse("type")
                .unwrap_err()
                .downcast::<ParseCommandError>()
                .unwrap(),
            ParseCommandError::LessArgs("type".to_string(), vec![], 1).into()
        );
    }

    #[test]
    fn test_parse_type() {
        assert_eq!(
            Command::parse("type echo type exit invalid_command").unwrap(),
            Command::BuiltinCommand(BuiltinCommand::Type(vec![
                Type::BuiltinCommand("echo".to_string()),
                Type::BuiltinCommand("type".to_string()),
                Type::BuiltinCommand("exit".to_string()),
                Type::UnrecognizedCommand("invalid_command".to_string()),
            ]))
        );
    }

    #[test]
    fn test_parse_exit() {
        assert_eq!(
            Command::parse("exit").unwrap(),
            Command::BuiltinCommand(BuiltinCommand::Exit(0))
        );
        assert_eq!(
            Command::parse("exit 123").unwrap(),
            Command::BuiltinCommand(BuiltinCommand::Exit(123))
        );
    }
}
