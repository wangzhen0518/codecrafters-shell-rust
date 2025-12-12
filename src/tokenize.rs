use std::collections::HashSet;

use lazy_static::lazy_static;

lazy_static! {
    static ref SPECIAL_CHARS: HashSet<char> = HashSet::from(['\'', '"', '\\']);
    static ref TOKEN_END_CHARS: HashSet<char> = HashSet::from(['&', '|', ';']);
    static ref COMMAND_END_TOKENS: HashSet<&'static str> =
        HashSet::from(["&", "&&", "|", "||", ";"]);
}

pub fn tokenize(input: &str) -> Vec<String> {
    debug_assert_ne!(input.chars().next_back(), Some('\\'));

    let buffer: Vec<char> = input.trim().chars().collect();

    let mut current_pos = 0;
    let mut new_token = String::new();
    let mut cmd_vec: Vec<String> = vec![];

    while current_pos < buffer.len() {
        let c = buffer[current_pos];

        let (read_state, part_token, num) = match c {
            '\'' => parse_single_quote(&buffer, current_pos),
            '"' => parse_double_quote(&buffer, current_pos),
            '\\' => parse_backslash(&buffer, current_pos, false),
            //TODO 1. '|' 需要考虑等待下一行的情况
            //TODO 2. 增加对 "&&" 和 "||" 的支持
            '&' | ';' | '|' => (ReadStatus::Finish, c.to_string(), 1),
            _ => parse_native(&buffer, current_pos),
        };

        if !part_token.is_empty() {
            if new_token.is_empty() {
                new_token = part_token;
            } else {
                new_token.push_str(&part_token);
            }
        }

        if let ReadStatus::Finish = read_state
            && !new_token.is_empty()
        {
            cmd_vec.push(new_token.clone());
            new_token.clear();
        }

        current_pos += num;
    }

    if !new_token.is_empty() {
        cmd_vec.push(new_token.clone());
        new_token.clear();
    }

    cmd_vec
}

enum ReadStatus {
    Finish,   // 当前 token 已结束
    Continue, // 当前 token 未结束
}

//TODO 考虑将 buffer 类型修改为 &str

fn parse_native(buffer: &[char], start_pos: usize) -> (ReadStatus, String, usize) {
    let mut token_start_pos = start_pos;
    while token_start_pos < buffer.len() && buffer[token_start_pos].is_whitespace() {
        token_start_pos += 1;
    }

    let mut token_end_pos = token_start_pos;
    while token_end_pos < buffer.len()
        && !buffer[token_end_pos].is_whitespace()
        && !SPECIAL_CHARS.contains(&buffer[token_end_pos])
        && !TOKEN_END_CHARS.contains(&buffer[token_end_pos])
    {
        token_end_pos += 1;
    }

    let token = buffer[token_start_pos..token_end_pos].iter().collect();

    let (read_state, num) =
        if token_end_pos < buffer.len() && SPECIAL_CHARS.contains(&buffer[token_end_pos]) {
            (ReadStatus::Continue, token_end_pos - start_pos)
        } else {
            let mut end_pos = token_end_pos;
            while end_pos < buffer.len() && buffer[end_pos].is_whitespace() {
                end_pos += 1;
            }

            (ReadStatus::Finish, end_pos - start_pos)
        };

    (read_state, token, num)
}

fn parse_single_quote(buffer: &[char], start_pos: usize) -> (ReadStatus, String, usize) {
    debug_assert_ne!(start_pos + 1, buffer.len());

    let mut end_pos = start_pos + 1;
    while end_pos < buffer.len() && buffer[end_pos] != '\'' {
        end_pos += 1;
    }
    // debug_assert_eq!(buffer[end_pos], '\'');

    let token = buffer[start_pos + 1..end_pos].iter().collect();
    if end_pos < buffer.len() {
        // ' 匹配上
        let num = end_pos - start_pos + 1; // +1 是跳过最后的 '
        if end_pos + 1 >= buffer.len() || buffer[end_pos + 1].is_whitespace() {
            // 已经到 buffer 末尾，或者 ' 的下一个字符是空白字符，那么当前 token 已结束
            (ReadStatus::Finish, token, num)
        } else {
            (ReadStatus::Continue, token, num)
        }
    } else {
        // ' 未匹配上，但 buffer 已结束
        (ReadStatus::Continue, token, end_pos - start_pos)
    }
}

fn parse_double_quote(buffer: &[char], start_pos: usize) -> (ReadStatus, String, usize) {
    debug_assert_ne!(start_pos + 1, buffer.len());

    let mut token = String::new();
    let mut end_pos = start_pos + 1;
    while end_pos < buffer.len() && buffer[end_pos] != '"' {
        if buffer[end_pos] == '\\' {
            let (_, part_token, num) = parse_backslash(buffer, end_pos, true);
            token.push_str(&part_token);
            end_pos += num;
        } else {
            token.push(buffer[end_pos]);
            end_pos += 1;
        }
    }
    // debug_assert_eq!(buffer[end_pos], '"');

    if end_pos < buffer.len() {
        // " 匹配上
        let num = end_pos - start_pos + 1; // +1 是跳过最后的 "
        if end_pos + 1 >= buffer.len() || buffer[end_pos + 1].is_whitespace() {
            // 已经到 buffer 末尾，或者 " 的下一个字符是空白字符，那么当前 token 已结束
            (ReadStatus::Finish, token, num)
        } else {
            (ReadStatus::Continue, token, num)
        }
    } else {
        // " 未匹配上，但 buffer 已结束
        (ReadStatus::Continue, token, end_pos - start_pos)
    }
}

fn parse_backslash(
    buffer: &[char],
    start_pos: usize,
    in_double_quote: bool,
) -> (ReadStatus, String, usize) {
    debug_assert_ne!(start_pos + 1, buffer.len());

    let escape_char = buffer[start_pos + 1];
    let token = if escape_char == '\n' {
        String::new()
    } else if in_double_quote {
        match escape_char {
            '\\' => "\\".to_string(),
            '"' => "\"".to_string(),
            _ => String::from_iter(['\\', escape_char]),
        }
    } else {
        escape_char.to_string()
    };

    let read_state = if start_pos + 2 == buffer.len()
        || (buffer[start_pos + 2].is_whitespace() && !in_double_quote)
    {
        //token 转译完到达 buffer 末尾，或者 后续是空白字符且不在双引号内
        ReadStatus::Finish
    } else {
        ReadStatus::Continue
    };

    (read_state, token, 2)
}
#[allow(unused)]
#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{builtin::BuiltinCommand, utils::vec_str_to_vec_string};

    use super::*;

    #[test]
    fn test_parse_native() {
        assert_eq!(
            tokenize("echo shell   hello"),
            vec_str_to_vec_string::<Vec<_>>(&["echo", "shell", "hello"])
        );
    }

    #[test]
    fn test_parse_single_quote() {
        assert_eq!(
            tokenize("echo 'shell   hello'"),
            vec_str_to_vec_string::<Vec<_>>(&["echo", "shell   hello"])
        );
        assert_eq!(
            tokenize("echo 'shell''hello'"),
            vec_str_to_vec_string::<Vec<_>>(&["echo", "shellhello"])
        );
        assert_eq!(
            tokenize("echo shell''hello"),
            vec_str_to_vec_string::<Vec<_>>(&["echo", "shellhello"])
        );
        assert_eq!(
            tokenize("cat '/tmp/file name' '/tmp/file name with spaces'"),
            vec_str_to_vec_string::<Vec<_>>(&[
                "cat",
                "/tmp/file name",
                "/tmp/file name with spaces"
            ])
        );
    }

    #[test]
    fn test_parse_native_double_quote() {
        assert_eq!(
            tokenize("echo \"shell   hello\""),
            vec_str_to_vec_string::<Vec<_>>(&["echo", "shell   hello"])
        );
        assert_eq!(
            tokenize("echo \"shell\"\"hello\""),
            vec_str_to_vec_string::<Vec<_>>(&["echo", "shellhello"])
        );
        assert_eq!(
            tokenize("echo shell\"\"hello"),
            vec_str_to_vec_string::<Vec<_>>(&["echo", "shellhello"])
        );
        assert_eq!(
            tokenize("echo \"shell's test\""),
            vec_str_to_vec_string::<Vec<_>>(&["echo", "shell's test"])
        );
        assert_eq!(
            tokenize("cat \"/tmp/file name\" \"/tmp/'file name' with spaces\""),
            vec_str_to_vec_string::<Vec<_>>(&[
                "cat",
                "/tmp/file name",
                "/tmp/'file name' with spaces"
            ])
        );
    }

    #[test]
    fn test_parse_backslash() {
        assert_eq!(
            tokenize("echo world\\ \\ \\ \\ \\ \\ script"),
            vec_str_to_vec_string::<Vec<_>>(&["echo", "world      script"])
        );
        assert_eq!(
            tokenize("echo before\\ after"),
            vec_str_to_vec_string::<Vec<_>>(&["echo", "before after"])
        );
        assert_eq!(
            tokenize("echo test\\nexample"),
            vec_str_to_vec_string::<Vec<_>>(&["echo", "testnexample"])
        );
        assert_eq!(
            tokenize("echo hello\\\\world"),
            vec_str_to_vec_string::<Vec<_>>(&["echo", "hello\\world"])
        );
        assert_eq!(
            tokenize("echo \\'hello\\'"),
            vec_str_to_vec_string::<Vec<_>>(&["echo", "'hello'"])
        );
        assert_eq!(
            tokenize("echo \\'\\\"hello world\\\"\\'"),
            vec_str_to_vec_string::<Vec<_>>(&["echo", "'\"hello", "world\"'"])
        );
        assert_eq!(
            tokenize("echo \"/tmp/pig/f\\n56\" \"/tmp/pig/f\\90\" \"/tmp/pig/f'\\'83\""),
            vec_str_to_vec_string::<Vec<_>>(&[
                "echo",
                "/tmp/pig/f\\n56",
                "/tmp/pig/f\\90",
                "/tmp/pig/f'\\'83",
            ])
        );
        assert_eq!(
            tokenize("cat \"/tmp/file\\\\name\" \"/tmp/file\\ name\""),
            vec_str_to_vec_string::<Vec<_>>(&["cat", "/tmp/file\\name", "/tmp/file\\ name"])
        );
    }
}
