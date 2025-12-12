// #![allow(dead_code)]

use rustyline::{CompletionType, Config, EditMode, Editor};

use crate::{
    command::Execute,
    parser::{CommandExecution, parse_tokens},
    reader::ShellHelper,
    tokenize::tokenize,
};

mod builtin;
mod command;
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
    let _ = rl.load_history("history.txt");

    loop {
        match rl.readline(PROMPT) {
            Ok(line) => {
                let tokens = tokenize(&line);
                match parse_tokens(&tokens) {
                    Ok(command_exec_vec) => {
                        for CommandExecution {
                            command,
                            output_writer,
                            error_writer,
                        } in command_exec_vec
                        {
                            command.execute(output_writer, error_writer);
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
}
