use std::{env, io::Write, path::PathBuf, process};

use is_executable::IsExecutable;

use crate::command::{Args, Execute, Parse};

pub fn load_path_var() -> String {
    env::var("PATH").expect("Invalid $PATH")
}

pub fn load_env_path() -> Vec<PathBuf> {
    env::split_paths(&load_path_var())
        .filter(|path| path.is_dir())
        .collect()
}

pub fn find_in_path(executable: &str) -> Option<PathBuf> {
    let executable = PathBuf::from(executable);
    if executable.exists() && executable.is_executable() {
        return Some(executable);
    }

    for dir in env::split_paths(&load_path_var()) {
        let candidate = dir.join(&executable);
        if candidate.exists() && candidate.is_executable() {
            return Some(candidate);
        }
    }
    None
}

#[derive(Debug, PartialEq, Eq)]
pub struct Executable {
    pub name: String,
    pub path: PathBuf,
    pub args: Args,
}

impl Executable {
    pub fn new(name: String, path: PathBuf, args: Args) -> Self {
        Self { name, path, args }
    }
}

impl Parse for Executable {
    fn parse(command: &str, args: &[&str]) -> crate::Result<Self>
    where
        Self: std::marker::Sized,
    {
        if let Some(exec_path) = find_in_path(command) {
            Ok(Executable::new(
                command.to_string(),
                exec_path,
                args.iter().map(|arg| arg.to_string()).collect(),
            ))
        } else {
            Err("Cannot find executable".into())
        }
    }
}

impl Execute for Executable {
    fn execute<O, E>(&self, output_writer: O, error_writer: E) -> crate::builtin::ExitCode
    where
        O: Write,
        E: Write,
        process::Stdio: From<O> + From<E>,
    {
        if let Ok(mut child) = process::Command::new(&self.name)
            .args(&self.args)
            .stdout(process::Stdio::from(output_writer))
            .stderr(process::Stdio::from(error_writer))
            .spawn()
        {
            if let Ok(exit_status) = child.wait() {
                exit_status.code().unwrap_or(-1)
            } else {
                -1
            }
        } else {
            -1
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::set_env_path;

    use super::*;

    #[test]
    fn test_find_in_path() {
        set_env_path();
        assert_eq!(find_in_path("ls"), Some(PathBuf::from("/usr/bin/ls")));
    }
}
