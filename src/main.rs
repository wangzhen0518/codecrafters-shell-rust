// #![allow(dead_code)]

use std::thread;

use rustyline::{CompletionType, Config, EditMode, Editor};

use crate::{
    command::Execute,
    parser::{CommandExecution, parse_tokens},
    reader::ShellHelper,
    tokenize::tokenize,
};

mod builtin;
mod command;
mod completer;
mod executable;
mod parser;
mod reader;
mod redirect;
mod tokenize;
mod utils;
mod validator;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

static PROMPT: &str = "$ ";
static HISTORY_FILE: &str = ".history";

fn main() {
    utils::config_logger();

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

    loop {
        match rl.readline(PROMPT) {
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

    let _ = rl.append_history(HISTORY_FILE);
}
