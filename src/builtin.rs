use std::{collections::HashSet, fmt::Display};

use lazy_static::lazy_static;

use crate::{
    command::{Execute, ParseCommandError},
    executable::{find_in_path, Executable},
    Result,
};

lazy_static! {
    pub static ref BUILTIN_COMMANDS: HashSet<&'static str> =
        HashSet::from(["echo", "type", "exit"]);
}

#[derive(Debug, PartialEq, Eq)]
pub enum BuiltinCommand {
    Echo(String),
    Type(Vec<Type>),
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

                BuiltinCommand::Type(args.iter().map(|arg| Type::parse(arg)).collect())
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
            BuiltinCommand::Type(types) => {
                for ty in types {
                    println!("{}", ty)
                }
            }
            BuiltinCommand::Exit(exit_code) => std::process::exit(*exit_code),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    BuiltinCommand(String),
    Executable(Executable),
    UnrecognizedCommand(String),
}

impl Type {
    fn parse(command: &str) -> Type {
        let cmd = command.to_string();
        if BUILTIN_COMMANDS.contains(cmd.as_str()) {
            Type::BuiltinCommand(cmd)
        } else if let Some(path) = find_in_path(&cmd) {
            Type::Executable(Executable::new(cmd, path, vec![]))
        } else {
            Type::UnrecognizedCommand(cmd)
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::BuiltinCommand(cmd) => write!(f, "{} is a shell builtin", cmd),
            #[allow(unused_variables)]
            Type::Executable(Executable { name, path, args }) => {
                write!(f, "{} is {}", name, path.to_string_lossy())
            }
            Type::UnrecognizedCommand(cmd) => write!(f, "{}: not found", cmd),
        }
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf};

    use crate::utils::set_env_path;

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
    fn test_parse_type() {
        set_env_path();
        assert_eq!(
            BuiltinCommand::parse(
                "type".to_string(),
                vec![
                    "echo".to_string(),
                    "type".to_string(),
                    "exit".to_string(),
                    "ls".to_string(),
                    "invalid_command".to_string()
                ]
            )
            .unwrap(),
            BuiltinCommand::Type(vec![
                Type::BuiltinCommand("echo".to_string()),
                Type::BuiltinCommand("type".to_string()),
                Type::BuiltinCommand("exit".to_string()),
                Type::Executable(Executable::new(
                    "ls".to_string(),
                    PathBuf::from("/usr/bin/ls"),
                    vec![],
                )),
                Type::UnrecognizedCommand("invalid_command".to_string()),
            ])
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
