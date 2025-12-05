use std::{collections::HashSet, fmt::Display};

use lazy_static::lazy_static;

use crate::{
    command::{Execute, ParseCommandError},
    Result,
};

lazy_static! {
    pub static ref BUILTIN_COMMANDS: HashSet<&'static str> =
        HashSet::from(["echo", "type", "exit"]);
}

#[derive(Debug, PartialEq, Eq)]
pub enum BuiltinCommand {
    Echo(String),
    Type(Type),
    Exit(i32),
}

impl BuiltinCommand {
    pub fn parse(command: String, args: Vec<String>) -> Result<BuiltinCommand> {
        let builtin_command = match command.as_str() {
            "echo" => {
                let content = if !args.is_empty() {
                    args.join(" ").to_string()
                } else {
                    "\n".to_string()
                };
                BuiltinCommand::Echo(content)
            }
            "type" => {
                // TODO 能否统一 check arg num 过程？
                if args.is_empty() {
                    return Err(ParseCommandError::LessArgs(command, args, 1).into());
                }

                BuiltinCommand::Type(Type::parse(args[0].as_str()))
            }
            "exit" => {
                if args.len() > 1 {
                    return Err(ParseCommandError::MoreArgs(command, args, 1).into());
                }

                let exit_code: i32 = if args.is_empty() { 0 } else { args[0].parse()? };
                BuiltinCommand::Exit(exit_code)
            }
            _ => unreachable!(),
        };
        Ok(builtin_command)
    }
}

impl Execute for BuiltinCommand {
    fn execute(&self) {
        match self {
            BuiltinCommand::Echo(content) => println!("{}", content),
            BuiltinCommand::Type(ty) => println!("{}", ty),
            BuiltinCommand::Exit(exit_code) => std::process::exit(*exit_code),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    BuiltinCommand(String),
    UnrecognizedCommand(String),
}

impl Type {
    fn parse(cmd: &str) -> Type {
        let cmd = cmd.to_string();
        if BUILTIN_COMMANDS.contains(cmd.as_str()) {
            Type::BuiltinCommand(cmd)
        } else {
            Type::UnrecognizedCommand(cmd)
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::BuiltinCommand(cmd) => write!(f, "{} is a shell builtin", cmd),
            Type::UnrecognizedCommand(cmd) => write!(f, "{}: not found", cmd),
        }
    }
}

#[allow(unused)]
mod tests {
    use crate::command::{Command, ParseCommandError};

    use super::*;

    #[test]
    fn test_parse_echo() {
        assert_eq!(
            BuiltinCommand::parse("echo".to_string(), vec![]).unwrap(),
            BuiltinCommand::Echo("\n".to_string())
        );
        assert_eq!(
            BuiltinCommand::parse(
                "echo".to_string(),
                vec!["abc".to_string(), "".to_string(), "123".to_string()]
            )
            .unwrap(),
            BuiltinCommand::Echo("abc  123".to_string())
        );
    }

    #[test]
    fn test_parse_type_error() {
        assert_eq!(
            BuiltinCommand::parse("type".to_string(), vec![])
                .unwrap_err()
                .downcast::<ParseCommandError>()
                .unwrap(),
            ParseCommandError::LessArgs("type".to_string(), vec![], 1).into()
        );
    }

    #[test]
    fn test_parse_type_builtin() {
        assert_eq!(
            BuiltinCommand::parse("type".to_string(), vec!["echo".to_string()]).unwrap(),
            BuiltinCommand::Type(Type::BuiltinCommand("echo".to_string()))
        );
    }

    #[test]
    fn test_parse_type_unrecognized() {
        assert_eq!(
            BuiltinCommand::parse("type".to_string(), vec!["invalid_command".to_string()]).unwrap(),
            BuiltinCommand::Type(Type::UnrecognizedCommand("invalid_command".to_string()))
        );
    }

    #[test]
    fn test_parse_exit() {
        assert_eq!(
            BuiltinCommand::parse("exit".to_string(), vec![]).unwrap(),
            BuiltinCommand::Exit(0)
        );
        assert_eq!(
            BuiltinCommand::parse("exit".to_string(), vec!["123".to_string()]).unwrap(),
            BuiltinCommand::Exit(123)
        );
    }
}
