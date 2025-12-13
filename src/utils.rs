use std::{
    fs,
    path::{Path, PathBuf},
};

use is_executable::IsExecutable;

pub fn config_logger() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .pretty()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");
}

#[allow(unused)]
pub fn set_env_path() {
    unsafe { std::env::set_var("PATH", "/usr/bin:/usr/local/bin:$PATH") };
}

#[allow(unused)]
pub fn vec_str_to_vec_string<B: FromIterator<String>>(s_vec: &[&str]) -> B {
    s_vec.iter().map(|s| s.to_string()).collect()
}

pub fn get_executables_from_dir(dir: &Path) -> Vec<PathBuf> {
    let mut execs = vec![];
    if let Ok(dir_entries) = fs::read_dir(dir) {
        for entry in dir_entries.flatten() {
            let file = entry.path();
            if file.is_file() && file.is_executable() {
                execs.push(file);
            }
        }
        execs
    } else {
        Vec::with_capacity(0)
    }
}
