use std::{
    collections::HashSet,
    io::{stdin, BufRead},
};

use lazy_static::lazy_static;

use crate::Result;

lazy_static! {
    static ref SPECIAL_CHARS: HashSet<char> = HashSet::from_iter(['\'', '"', '\\']);
}

enum ReadState {
    Finish,      // 当前 token 已结束
    Continue,    // 当前 token 未结束，当前 buffer 还有剩余内容
    NeedNewLine, // 当前 token 未结束，但当前 buffer 无剩余内容，需要读取下一行
}

fn read_native(buf: &[char], start_pos: usize) -> (ReadState, String, usize) {
    let mut token_start_pos = start_pos;
    while token_start_pos < buf.len() && buf[token_start_pos].is_whitespace() {
        token_start_pos += 1;
    }

    let mut token_end_pos = token_start_pos;
    while token_end_pos < buf.len()
        && !buf[token_end_pos].is_whitespace()
        && !SPECIAL_CHARS.contains(&buf[token_end_pos])
    {
        token_end_pos += 1;
    }

    let token = buf[token_start_pos..token_end_pos].iter().collect();

    let (read_state, num) =
        if token_end_pos < buf.len() && SPECIAL_CHARS.contains(&buf[token_end_pos]) {
            (ReadState::Continue, token_end_pos - start_pos)
        } else {
            let mut end_pos = token_end_pos;
            while end_pos < buf.len() && buf[end_pos].is_whitespace() {
                end_pos += 1;
            }

            (ReadState::Finish, end_pos - start_pos)
        };

    (read_state, token, num)
}

fn read_single_quote(buf: &[char], start_pos: usize) -> (ReadState, String, usize) {
    if start_pos + 1 == buf.len() {
        return (ReadState::NeedNewLine, String::new(), 0);
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
            ReadState::Finish
        } else {
            ReadState::Continue
        };
        (read_state, num)
    } else {
        (ReadState::NeedNewLine, end_pos - start_pos)
    };

    (read_state, token, num)
}

fn read_double_quote(buf: &[char], start_pos: usize) -> (ReadState, String, usize) {
    if start_pos + 1 == buf.len() {
        return (ReadState::NeedNewLine, String::new(), 0);
    }

    let mut end_pos = start_pos + 1;
    while end_pos < buf.len() && buf[end_pos] != '"' {
        end_pos += 1;
    }

    let token = buf[start_pos + 1..end_pos].iter().collect();
    let (read_state, num) = if end_pos < buf.len() && buf[end_pos] == '"' {
        let num = end_pos - start_pos + 1; // +1 是跳过最后的 '"'
        let read_state = if end_pos + 1 >= buf.len() || buf[end_pos + 1].is_whitespace() {
            // 已经到 buffer 末尾，或者 '"' 的下一个字符是空白字符，那么当前 token 已结束
            ReadState::Finish
        } else {
            ReadState::Continue
        };
        (read_state, num)
    } else {
        (ReadState::NeedNewLine, end_pos - start_pos)
    };

    (read_state, token, num)
}

fn parse_input_from_reader<R: BufRead>(reader: &mut R) -> Result<Vec<String>> {
    let mut input = String::new();
    reader.read_line(&mut input)?;
    let mut buf: Vec<char> = input.chars().collect();

    let mut current_pos = 0;
    let mut new_token = String::new();
    let mut cmd_vec: Vec<String> = vec![];

    while current_pos < buf.len() {
        let c = buf[current_pos];

        let (read_state, part_token, num) = if c == '\'' {
            read_single_quote(&buf, current_pos)
        } else if c == '"' {
            read_double_quote(&buf, current_pos)
        } else {
            read_native(&buf, current_pos)
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
            ReadState::Finish => {
                if !new_token.is_empty() {
                    cmd_vec.push(new_token.clone());
                    new_token.clear();
                }
            }
            ReadState::Continue => {}
            ReadState::NeedNewLine => {
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

pub fn parse_input() -> Result<(String, Vec<String>)> {
    let cmd_vec = parse_input_from_reader(&mut stdin().lock())?;
    if !cmd_vec.is_empty() {
        Ok((cmd_vec[0].clone(), cmd_vec[1..].to_vec()))
    } else {
        Ok(("".to_string(), cmd_vec))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn test_parse(input: &str, target: &[&str]) {
        assert_eq!(
            parse_input_from_reader(&mut Cursor::new(input.as_bytes())).unwrap(),
            target.to_vec()
        );
    }

    #[test]
    fn test_parse_native() {
        test_parse("echo shell   hello", &["echo", "shell", "hello"]);
    }

    #[test]
    fn test_parse_single_quote() {
        test_parse("echo 'shell   hello'", &["echo", "shell   hello"]);
        test_parse("echo 'shell''hello'", &["echo", "shellhello"]);
        test_parse("echo shell''hello", &["echo", "shellhello"]);
        test_parse(
            "cat '/tmp/file name' '/tmp/file name with spaces'",
            &["cat", "/tmp/file name", "/tmp/file name with spaces"],
        );
    }

    #[test]
    fn test_parse_native_double_quote() {
        test_parse("echo \"shell   hello\"", &["echo", "shell   hello"]);
        test_parse("echo \"shell\"\"hello\"", &["echo", "shellhello"]);
        test_parse("echo shell\"\"hello", &["echo", "shellhello"]);
        test_parse("echo \"shell's test\"", &["echo", "shell's test"]);
        test_parse(
            "cat \"/tmp/file name\" \"/tmp/'file name' with spaces\"",
            &["cat", "/tmp/file name", "/tmp/'file name' with spaces"],
        );
    }
}
