use std::{collections::HashSet, env, fmt::Display, io::Write, path::PathBuf};

use lazy_static::lazy_static;

use crate::{
    command::{Execute, Parse, ParseCommandError},
    executable::{find_in_path, Executable},
    redirect::Writer,
    Result,
};

lazy_static! {
    pub static ref BUILTIN_COMMANDS: HashSet<&'static str> =
        HashSet::from(["echo", "type", "pwd", "cd", "exit"]);
}

pub type ExitCode = i32;

#[derive(Debug, PartialEq, Eq)]
pub enum BuiltinCommand {
    Echo(String),
    Type(Vec<Type>),
    Pwd,
    Cd(String),
    Exit(ExitCode),
}

impl Parse for BuiltinCommand {
    fn parse(command: &str, args: &[String]) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        let builtin_command = match command {
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
                    return Err(
                        ParseCommandError::LessArgs(command.to_string(), args.to_vec(), 1).into(),
                    );
                }

                BuiltinCommand::Type(args.iter().map(|arg| Type::parse(arg)).collect())
            }
            "pwd" => {
                if !args.is_empty() {
                    return Err(
                        ParseCommandError::MoreArgs(command.to_string(), args.to_vec(), 0).into(),
                    );
                }

                BuiltinCommand::Pwd
            }
            "cd" => {
                if args.len() > 1 {
                    return Err(
                        ParseCommandError::MoreArgs(command.to_string(), args.to_vec(), 1).into(),
                    );
                }

                let target_dir = if !args.is_empty() {
                    args[0].to_string()
                } else {
                    "~".to_string()
                };
                BuiltinCommand::Cd(target_dir)
            }
            "exit" => {
                if args.len() > 1 {
                    return Err(
                        ParseCommandError::MoreArgs(command.to_string(), args.to_vec(), 1).into(),
                    );
                }

                let exit_code = if args.is_empty() { 0 } else { args[0].parse()? };
                BuiltinCommand::Exit(exit_code)
            }
            _ => unreachable!(),
        };
        Ok(builtin_command)
    }
}

impl Execute for BuiltinCommand {
    fn execute(&self, mut output_writer: Writer, mut error_writer: Writer) -> ExitCode {
        match self {
            BuiltinCommand::Echo(content) => {
                -(writeln!(output_writer, "{}", content).is_err() as ExitCode)
            }
            BuiltinCommand::Type(types) => {
                for ty in types {
                    if writeln!(output_writer, "{}", ty).is_err() {
                        return -1;
                    }
                }
                0
            }
            BuiltinCommand::Pwd => {
                if let Ok(pwd) = env::current_dir() {
                    -(writeln!(output_writer, "{}", pwd.display()).is_err() as ExitCode)
                } else {
                    let _ = writeln!(error_writer, "invalid directory");
                    -1
                }
            }
            BuiltinCommand::Cd(target_dir) => {
                let mut paths: Vec<String> = PathBuf::from(target_dir)
                    .components()
                    .map(|p| p.as_os_str().to_string_lossy().to_string())
                    .collect();
                //TODO 是否需要检查 paths.is_empty()
                if paths[0] == "~" {
                    paths[0] = env::home_dir()
                        .map_or("".to_string(), |path| path.to_string_lossy().to_string());
                }
                let target_dir: PathBuf = paths.iter().collect();
                if env::set_current_dir(&target_dir).is_err() {
                    let _ = writeln!(
                        error_writer,
                        "cd: {}: No such file or directory",
                        target_dir.display()
                    );
                    -1
                } else {
                    0
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
                write!(f, "{} is {}", name, path.display())
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
            BuiltinCommand::parse("echo", &[]).unwrap(),
            BuiltinCommand::Echo("\n".to_string())
        );
        assert_eq!(
            BuiltinCommand::parse(
                "echo",
                &["abc".to_string(), "".to_string(), "123".to_string()]
            )
            .unwrap(),
            BuiltinCommand::Echo("abc  123".to_string())
        );
    }

    #[test]
    fn test_parse_type_error() {
        assert_eq!(
            BuiltinCommand::parse("type", &[])
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
                "type",
                &[
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
            BuiltinCommand::parse("exit", &[]).unwrap(),
            BuiltinCommand::Exit(0)
        );
        assert_eq!(
            BuiltinCommand::parse("exit", &["123".to_string()]).unwrap(),
            BuiltinCommand::Exit(123)
        );
    }
}
