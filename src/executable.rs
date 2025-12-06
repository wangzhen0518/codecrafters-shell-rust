use std::{env, path::PathBuf, process};

use is_executable::IsExecutable;

use crate::command::{Args, Execute};

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

impl Execute for Executable {
    fn execute(&self) {
        process::Command::new(&self.name)
            .args(&self.args)
            .spawn()
            .expect("Executable failed to start.")
            .wait()
            .expect("Executable failed to execute.");
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
