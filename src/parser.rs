use std::{collections::HashSet, fs, io};

use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    Result,
    command::{Command, Parse},
    redirect::{Reader, Writer},
};

lazy_static! {
    static ref COMMAND_END_TOKENS: HashSet<&'static str> =
        HashSet::from(["&", "&&", "|", "||", ";"]);
}

#[derive(Debug)]
pub struct CommandExecution {
    pub command: Command,
    pub reader: Reader,
    pub output_writer: Writer,
    pub error_writer: Writer,
    pub use_pipe: bool,
}

impl CommandExecution {
    pub fn new(
        command: Command,
        reader: Reader,
        output_writer: Writer,
        error_writer: Writer,
        use_pipe: bool,
    ) -> Self {
        Self {
            command,
            reader,
            output_writer,
            error_writer,
            use_pipe,
        }
    }
}

impl Default for CommandExecution {
    fn default() -> Self {
        Self {
            command: Command::Empty,
            reader: Reader::Stdin,
            output_writer: io::stdout().into(),
            error_writer: io::stderr().into(),
            use_pipe: true,
        }
    }
}

enum RedirectIO {
    Stdout,
    Stderr,
}

fn extract_redirect(s: &str) -> Option<(&str, &str, &str)> {
    let re: Regex = Regex::new(r"^([12]?)(>|>>)(?:&([12]))?$").unwrap();
    if let Some(caps) = re.captures(s) {
        let start = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let redirect = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let target = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        Some((start, redirect, target))
    } else {
        None
    }
}

fn parse_redirect(
    tokens: &[String],
    start_pos: usize,
) -> Result<Option<(RedirectIO, Writer, usize)>> {
    //TODO 支持输入重定向
    if let Some((origin, redirect, new)) = extract_redirect(&tokens[start_pos]) {
        let mut num = 1;
        let redirect_io = match origin {
            "1" | "" => RedirectIO::Stdout,
            "2" => RedirectIO::Stderr,
            _ => unreachable!(),
        };
        let writer = match new {
            // TODO 重定向到 output 或者 error 的 writer，而不是 stdout, stderr
            // TODO 比如 2>&1 | tee，这里 stdout 也重定向了，那么直接将 stderr 重定向到 io::stdout() 是错误的
            "1" => io::stdout().into(),
            "2" => io::stderr().into(),
            "" => {
                if start_pos + 1 == tokens.len() {
                    return Err("syntax error".into());
                } else {
                    num += 1;
                    fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .append(redirect == ">>")
                        .open(&tokens[start_pos + 1])?
                        .into()
                }
            }
            _ => unreachable!(),
        };
        Ok(Some((redirect_io, writer, num)))
    } else {
        Ok(None)
    }
}

pub fn parse_tokens(tokens: &[String]) -> Result<Vec<CommandExecution>> {
    let mut idx = 0;
    let mut command_exec_vec = vec![];
    let mut current_cmd_args: Vec<String> = vec![];

    let mut reader = None;
    let mut next_reader = None;
    let mut output_writer = None;
    let mut error_writer = None;

    while idx < tokens.len() {
        if let Some((redirect_io, writer, num)) = parse_redirect(tokens, idx)? {
            match redirect_io {
                RedirectIO::Stdout => output_writer = Some(writer),
                RedirectIO::Stderr => error_writer = Some(writer),
            }
            idx += num;
        } else if COMMAND_END_TOKENS.contains(tokens[idx].as_str()) {
            let use_pipe = match tokens[idx].as_str() {
                "&" => todo!(),
                "|" => {
                    let (pipe_reader, pipe_writer) = io::pipe()?;
                    next_reader = Some(Reader::PipeReader(pipe_reader));
                    output_writer = Some(Writer::PipeWriter(pipe_writer));
                    false
                }
                //TODO 处理 exit code
                "&&" => todo!(),
                "||" => todo!(),
                ";" => true,
                _ => unreachable!(),
            };
            command_exec_vec.push(CommandExecution::new(
                Command::parse(&current_cmd_args[0], &current_cmd_args[1..])?,
                reader.take().unwrap_or(Reader::Stdin),
                output_writer.take().unwrap_or(io::stdout().into()),
                error_writer.take().unwrap_or(io::stderr().into()),
                use_pipe,
            ));

            reader = next_reader.take();
            current_cmd_args.clear();
            idx += 1;
        } else {
            current_cmd_args.push(tokens[idx].to_string());
            idx += 1;
        }
    }

    if !current_cmd_args.is_empty() {
        command_exec_vec.push(CommandExecution::new(
            Command::parse(&current_cmd_args[0], &current_cmd_args[1..])?,
            reader.unwrap_or(Reader::Stdin),
            output_writer.unwrap_or(io::stdout().into()),
            error_writer.unwrap_or(io::stderr().into()),
            true,
        ));
    }

    Ok(command_exec_vec)
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{builtin::BuiltinCommand, utils::vec_str_to_vec_string};

    use super::*;

    #[test]
    fn test_extract_redirect() {
        assert_eq!(extract_redirect("1>&2"), Some(("1", ">", "2")));
        assert_eq!(extract_redirect("1>"), Some(("1", ">", "")));
        assert_eq!(extract_redirect(">&2"), Some(("", ">", "2")));
        assert_eq!(extract_redirect(">"), Some(("", ">", "")));
        assert_eq!(extract_redirect("2>>&1"), Some(("2", ">>", "1")));
        assert_eq!(extract_redirect("2>>&"), None);
        assert_eq!(extract_redirect(">>&"), None);
        assert_eq!(extract_redirect("1>&"), None);
    }

    // #[test]
    // fn test_parse_tokens() {
    //     let mut command_exec_vec =
    //         parse_tokens(&get_parsed_tokens("echo hello > output.txt")).unwrap();
    //     assert_eq!(command_exec_vec.len(), 1);
    //     let command_exec = command_exec_vec.remove(0);
    //     assert_eq!(
    //         command_exec.command,
    //         Command::BuiltinCommand(BuiltinCommand::Echo("hello".to_string()))
    //     );
    // }
}
