use std::{collections::HashSet, env, io::Write, path::PathBuf};

use lazy_static::lazy_static;
use rustyline::history::History;

use crate::{
    RL, Result,
    command::{Execute, Parse, ParseCommandError},
    redirect::{Reader, Writer},
};

mod history;
mod type_;

use type_::Type;

lazy_static! {
    pub static ref BUILTIN_COMMANDS: HashSet<&'static str> =
        HashSet::from(["echo", "type", "history", "pwd", "cd", "exit"]);
}

pub type ExitCode = i32;

#[derive(Debug, PartialEq, Eq)]
pub enum BuiltinCommand {
    Echo(String),
    Type(Type),
    History(i64),
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
            "type" => BuiltinCommand::Type(Type::parse(command, args)?),
            "history" => {
                // TODO 能否统一 check arg num 过程？
                if args.len() > 1 {
                    return Err(
                        ParseCommandError::MoreArgs(command.to_string(), args.to_vec(), 1).into(),
                    );
                }

                let n = if args.is_empty() {
                    -1
                } else {
                    args[0].parse()?
                };

                BuiltinCommand::History(n)
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
    fn execute(
        &self,
        reader: Reader,
        mut output_writer: Writer,
        mut error_writer: Writer,
    ) -> ExitCode {
        match self {
            BuiltinCommand::Echo(content) => {
                -(writeln!(output_writer, "{}", content).is_err() as ExitCode)
            }
            BuiltinCommand::Type(ty) => ty.execute(reader, output_writer, error_writer),
            BuiltinCommand::History(n) => {
                if let Ok(rl) = RL.lock() {
                    let history = rl.history();
                    let num = history.len();
                    let length = (num as f64).log10() as usize + 1;
                    let n = if *n == -1 { 0 } else { num - *n as usize };
                    for (idx, record) in history.iter().enumerate().skip(n) {
                        if writeln!(output_writer, "   {:length$}  {}", idx + 1, record).is_err() {
                            return -1;
                        }
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

#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf};

    use crate::{
        executable::Executable,
        utils::{set_env_path, vec_str_to_vec_string},
    };

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
                &vec_str_to_vec_string::<Vec<_>>(&["abc", "", "123"])
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
