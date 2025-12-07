use std::{collections::HashSet, fs, io};

use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    command::{Command, Parse},
    redirect::Writer,
    Result,
};

lazy_static! {
    static ref SPECIAL_CHARS: HashSet<char> = HashSet::from(['\'', '"', '\\']);
    static ref TOKEN_END_CHARS: HashSet<char> = HashSet::from(['&', '|', ';']);
    static ref COMMAND_END_TOKENS: HashSet<&'static str> =
        HashSet::from(["&", "&&", "|", "||", ";"]);
}

enum ReadStatus {
    Finish,      // 当前 token 已结束
    Continue,    // 当前 token 未结束，当前 buffer 还有剩余内容
    NeedNewLine, // 当前 token 未结束，但当前 buffer 无剩余内容，需要读取下一行
}

fn read_native(buf: &[char], start_pos: usize) -> (ReadStatus, String, usize) {
    let mut token_start_pos = start_pos;
    while token_start_pos < buf.len() && buf[token_start_pos].is_whitespace() {
        token_start_pos += 1;
    }

    let mut token_end_pos = token_start_pos;
    while token_end_pos < buf.len()
        && !buf[token_end_pos].is_whitespace()
        && !SPECIAL_CHARS.contains(&buf[token_end_pos])
        && !TOKEN_END_CHARS.contains(&buf[token_end_pos])
    {
        token_end_pos += 1;
    }

    let token = buf[token_start_pos..token_end_pos].iter().collect();

    let (read_state, num) =
        if token_end_pos < buf.len() && SPECIAL_CHARS.contains(&buf[token_end_pos]) {
            (ReadStatus::Continue, token_end_pos - start_pos)
        } else {
            let mut end_pos = token_end_pos;
            while end_pos < buf.len() && buf[end_pos].is_whitespace() {
                end_pos += 1;
            }

            (ReadStatus::Finish, end_pos - start_pos)
        };

    (read_state, token, num)
}

fn read_single_quote(buf: &[char], start_pos: usize) -> (ReadStatus, String, usize) {
    if start_pos + 1 == buf.len() {
        return (ReadStatus::NeedNewLine, String::new(), 0);
    }

    let mut end_pos = start_pos + 1;
    while end_pos < buf.len() && buf[end_pos] != '\'' {
        end_pos += 1;
    }

    let token = buf[start_pos + 1..end_pos].iter().collect();
    let (read_state, num) = if end_pos < buf.len() && buf[end_pos] == '\'' {
        let num = end_pos - start_pos + 1; // +1 是跳过最后的 '\''
        let read_state = if end_pos + 1 >= buf.len() || buf[end_pos + 1].is_whitespace() {
            // 已经到 buffer 末尾，或者 '\'' 的下一个字符是空白字符，那么当前 token 已结束
            ReadStatus::Finish
        } else {
            ReadStatus::Continue
        };
        (read_state, num)
    } else {
        (ReadStatus::NeedNewLine, end_pos - start_pos)
    };

    (read_state, token, num)
}

fn read_double_quote(buf: &[char], start_pos: usize) -> (ReadStatus, String, usize) {
    if start_pos + 1 == buf.len() {
        return (ReadStatus::NeedNewLine, String::new(), 0);
    }

    let mut token = String::new();
    let mut end_pos = start_pos + 1;
    while end_pos < buf.len() && buf[end_pos] != '"' {
        if buf[end_pos] == '\\' {
            let (read_state, part_token, num) = read_backslash(buf, end_pos, true);
            token.push_str(&part_token);
            end_pos += num;
            if let ReadStatus::NeedNewLine = read_state {
                return (read_state, token, end_pos);
            }
        } else {
            token.push(buf[end_pos]);
            end_pos += 1;
        }
    }

    let (read_state, num) = if end_pos < buf.len() && buf[end_pos] == '"' {
        let num = end_pos - start_pos + 1; // +1 是跳过最后的 '"'
        let read_state = if end_pos + 1 >= buf.len() || buf[end_pos + 1].is_whitespace() {
            // 已经到 buffer 末尾，或者 '"' 的下一个字符是空白字符，那么当前 token 已结束
            ReadStatus::Finish
        } else {
            ReadStatus::Continue
        };
        (read_state, num)
    } else {
        (ReadStatus::NeedNewLine, end_pos - start_pos)
    };

    (read_state, token, num)
}

fn escape_special_char(c: char) -> String {
    match c {
        '\\' => "\\".to_string(),
        ' ' => " ".to_string(),
        '"' => "\"".to_string(),
        _ => String::from_iter(&['\\', c]),
    }
}

fn read_backslash(
    buf: &[char],
    start_pos: usize,
    in_double_quote: bool,
) -> (ReadStatus, String, usize) {
    if start_pos + 1 == buf.len() {
        return (ReadStatus::NeedNewLine, String::new(), 0);
    }

    let token = if !in_double_quote {
        String::from(buf[start_pos + 1])
    } else {
        escape_special_char(buf[start_pos + 1])
    };

    let read_state = if start_pos + 2 == buf.len() || buf[start_pos + 2].is_whitespace() {
        ReadStatus::Finish
    } else {
        ReadStatus::Continue
    };

    (read_state, token, 2)
}

fn parse_string<R: io::BufRead>(reader: &mut R) -> Result<Vec<String>> {
    let mut input = String::new();
    reader.read_line(&mut input)?;
    let mut buf: Vec<char> = input.trim().chars().collect();

    let mut current_pos = 0;
    let mut new_token = String::new();
    let mut cmd_vec: Vec<String> = vec![];

    while current_pos < buf.len() {
        let c = buf[current_pos];

        let (read_state, part_token, num) = match c {
            '\'' => read_single_quote(&buf, current_pos),
            '"' => read_double_quote(&buf, current_pos),
            '\\' => read_backslash(&buf, current_pos, false),
            //TODO 1. '|' 需要考虑等待下一行的情况
            //TODO 2. 增加对 "&&" 和 "||" 的支持
            '&' | ';' | '|' => (ReadStatus::Finish, c.to_string(), 1),
            _ => read_native(&buf, current_pos),
        };

        if !part_token.is_empty() {
            if new_token.is_empty() {
                new_token = part_token;
            } else {
                new_token.push_str(&part_token);
            }
        }
        current_pos += num;
        match read_state {
            ReadStatus::Finish => {
                if !new_token.is_empty() {
                    cmd_vec.push(new_token.clone());
                    new_token.clear();
                }
            }
            ReadStatus::Continue => {}
            ReadStatus::NeedNewLine => {
                input.clear();
                reader.read_line(&mut input)?;
                buf = input.chars().collect();
                current_pos = 0;
                if buf[current_pos].is_whitespace() && !new_token.is_empty() {
                    cmd_vec.push(new_token.clone());
                    new_token.clear();
                }
            }
        }
    }

    if !new_token.is_empty() {
        cmd_vec.push(new_token.clone());
        new_token.clear();
    }

    Ok(cmd_vec)
}

#[derive(Debug)]
pub struct CommandExecution {
    pub command: Command,
    pub output_writer: Writer,
    pub error_writer: Writer,
}

impl CommandExecution {
    pub fn new(command: Command, output_writer: Writer, error_writer: Writer) -> Self {
        Self {
            command,
            output_writer,
            error_writer,
        }
    }
}

impl Default for CommandExecution {
    fn default() -> Self {
        Self {
            command: Command::Empty,
            output_writer: io::stdout().into(),
            error_writer: io::stderr().into(),
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
    if let Some((origin, redirect, new)) = extract_redirect(&tokens[start_pos]) {
        let mut num = 1;
        let redirect_io = match origin {
            "1" | "" => RedirectIO::Stdout,
            "2" => RedirectIO::Stderr,
            _ => unreachable!(),
        };
        let writer = match new {
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

fn parse_tokens(tokens: &[String]) -> Result<Vec<CommandExecution>> {
    let mut idx = 0;
    let mut command_exec_vec = vec![];
    let mut current_cmd_args: Vec<String> = vec![];
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
            // TODO
            // match tokens[idx] {
            //     "&"=>{output_writer = process::Stdio::null()}
            // }
            command_exec_vec.push(CommandExecution::new(
                Command::parse(&current_cmd_args[0], &current_cmd_args[1..])?,
                output_writer.take().unwrap_or(io::stdout().into()),
                error_writer.take().unwrap_or(io::stderr().into()),
            ));
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
            output_writer.unwrap_or(io::stdout().into()),
            error_writer.unwrap_or(io::stderr().into()),
        ));
    }

    Ok(command_exec_vec)
}

pub fn parse_input() -> Result<Vec<CommandExecution>> {
    let tokens = parse_string(&mut io::stdin().lock())?;
    parse_tokens(&tokens)
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{builtin::BuiltinCommand, utils::vec_str_to_vec_string};

    use super::*;

    fn get_parsed_tokens(input: &str) -> Vec<String> {
        parse_string(&mut Cursor::new(input.as_bytes())).unwrap()
    }

    #[test]
    fn test_parse_native() {
        assert_eq!(
            get_parsed_tokens("echo shell   hello"),
            vec_str_to_vec_string(&["echo", "shell", "hello"])
        );
    }

    #[test]
    fn test_parse_single_quote() {
        assert_eq!(
            get_parsed_tokens("echo 'shell   hello'"),
            vec_str_to_vec_string(&["echo", "shell   hello"])
        );
        assert_eq!(
            get_parsed_tokens("echo 'shell''hello'"),
            vec_str_to_vec_string(&["echo", "shellhello"])
        );
        assert_eq!(
            get_parsed_tokens("echo shell''hello"),
            vec_str_to_vec_string(&["echo", "shellhello"])
        );
        assert_eq!(
            get_parsed_tokens("cat '/tmp/file name' '/tmp/file name with spaces'"),
            vec_str_to_vec_string(&["cat", "/tmp/file name", "/tmp/file name with spaces"])
        );
    }

    #[test]
    fn test_parse_native_double_quote() {
        assert_eq!(
            get_parsed_tokens("echo \"shell   hello\""),
            vec_str_to_vec_string(&["echo", "shell   hello"])
        );
        assert_eq!(
            get_parsed_tokens("echo \"shell\"\"hello\""),
            vec_str_to_vec_string(&["echo", "shellhello"])
        );
        assert_eq!(
            get_parsed_tokens("echo shell\"\"hello"),
            vec_str_to_vec_string(&["echo", "shellhello"])
        );
        assert_eq!(
            get_parsed_tokens("echo \"shell's test\""),
            vec_str_to_vec_string(&["echo", "shell's test"])
        );
        assert_eq!(
            get_parsed_tokens("cat \"/tmp/file name\" \"/tmp/'file name' with spaces\""),
            vec_str_to_vec_string(&["cat", "/tmp/file name", "/tmp/'file name' with spaces"])
        );
    }

    #[test]
    fn test_parse_backslash() {
        assert_eq!(
            get_parsed_tokens("echo world\\ \\ \\ \\ \\ \\ script"),
            vec_str_to_vec_string(&["echo", "world      script"])
        );
        assert_eq!(
            get_parsed_tokens("echo before\\ after"),
            vec_str_to_vec_string(&["echo", "before after"])
        );
        assert_eq!(
            get_parsed_tokens("echo test\\nexample"),
            vec_str_to_vec_string(&["echo", "testnexample"])
        );
        assert_eq!(
            get_parsed_tokens("echo hello\\\\world"),
            vec_str_to_vec_string(&["echo", "hello\\world"])
        );
        assert_eq!(
            get_parsed_tokens("echo \\'hello\\'"),
            vec_str_to_vec_string(&["echo", "'hello'"])
        );
        assert_eq!(
            get_parsed_tokens("echo \\'\\\"hello world\\\"\\'"),
            vec_str_to_vec_string(&["echo", "'\"hello", "world\"'"])
        );
        assert_eq!(
            get_parsed_tokens("echo \"/tmp/pig/f\\n56\" \"/tmp/pig/f\\90\" \"/tmp/pig/f'\\'83\""),
            vec_str_to_vec_string(&[
                "echo",
                "/tmp/pig/f\\n56",
                "/tmp/pig/f\\90",
                "/tmp/pig/f'\\'83",
            ])
        );
        assert_eq!(
            get_parsed_tokens("cat \"/tmp/file\\\\name\" \"/tmp/file\\ name\""),
            vec_str_to_vec_string(&["cat", "/tmp/file\\name", "/tmp/file name"])
        );
    }

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
