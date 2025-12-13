use std::{collections::HashSet, env, path::PathBuf, process, sync::RwLock};

use is_executable::IsExecutable;
use lazy_static::lazy_static;

use crate::{
    builtin::ExitCode,
    command::{Args, Execute, Parse},
    redirect::Writer,
};

lazy_static! {
    pub static ref PATH_ENV: RwLock<String> = RwLock::new(load_env_path());
    pub static ref PATHS: RwLock<HashSet<PathBuf>> = RwLock::new(HashSet::from_iter(load_paths()));
}

pub fn load_env_path() -> String {
    env::var("PATH").expect("Invalid $PATH")
}

#[allow(unused)]
pub fn load_paths() -> Vec<PathBuf> {
    //TODO 是否可以用 HashSet，还是应该用 Vec?
    env::split_paths(&load_env_path())
        .filter(|path| path.is_dir())
        .collect()
}

pub fn find_in_path(executable: &str) -> Option<PathBuf> {
    let executable = PathBuf::from(executable);
    if executable.exists() && executable.is_executable() {
        return Some(executable);
    }

    for dir in env::split_paths(&load_env_path()) {
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
    fn parse(command: &str, args: &[String]) -> crate::Result<Self>
    where
        Self: std::marker::Sized,
    {
        if let Some(exec_path) = find_in_path(command) {
            Ok(Executable::new(
                command.to_string(),
                exec_path,
                args.to_vec(),
            ))
        } else {
            Err("Cannot find executable".into())
        }
    }
}

impl Execute for Executable {
    fn execute(&self, output_writer: Writer, error_writer: Writer) -> ExitCode {
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
