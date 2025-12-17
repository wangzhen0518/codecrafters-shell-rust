// #![allow(dead_code)]

use std::{sync::Mutex, thread};

use lazy_static::lazy_static;
use rustyline::{CompletionType, Config, EditMode, Editor, history::FileHistory};

use crate::{
    command::Execute,
    helper::ShellHelper,
    parser::{CommandExecution, parse_tokens},
    tokenize::tokenize,
};

mod builtin;
mod command;
mod completer;
mod executable;
mod helper;
mod parser;
mod redirect;
mod tokenize;
mod utils;
mod validator;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

static PROMPT: &str = "$ ";
static HISTORY_FILE: &str = ".history";

lazy_static! {
    pub static ref RL: Mutex<Editor<ShellHelper, FileHistory>> = {
        let helper = ShellHelper::new();
        let config = Config::builder()
            .history_ignore_space(true)
            .auto_add_history(true)
            .edit_mode(EditMode::Emacs)
            .completion_type(CompletionType::List)
            .build();
        let mut rl = Editor::with_config(config).expect("Failed to build Editor");
        rl.set_helper(Some(helper));
        let _ = rl.load_history(HISTORY_FILE);
        Mutex::new(rl)
    };
}

fn main() {
    utils::config_logger();

    tracing::debug!("{:?}", std::env::var("HISTFILE"));

    loop {
        let line = RL.lock().unwrap().readline(PROMPT);
        match line {
            Ok(line) => {
                let tokens = tokenize(&line);
                match parse_tokens(&tokens) {
                    Ok(command_exec_vec) => {
                        for CommandExecution {
                            command,
                            reader,
                            output_writer,
                            error_writer,
                            use_pipe,
                        } in command_exec_vec
                        {
                            //? 对于 pipe 采用并行运行是否是正确的做法？
                            if use_pipe {
                                command.execute(reader, output_writer, error_writer);
                            } else {
                                // 不需要单独 join，因为最后一个 pipe 命令是阻塞执行的，所以在不被取消的情况下，会一直等待前面的命令全部执行完才终止
                                thread::spawn(move || {
                                    command.execute(reader, output_writer, error_writer)
                                });
                            }
                        }
                    }
                    Err(err) => eprintln!("{}", err),
                }
            }
            Err(err) => {
                eprintln!("{}", err);
                break;
            }
        }
    }

    let _ = RL.lock().unwrap().append_history(HISTORY_FILE);
}
