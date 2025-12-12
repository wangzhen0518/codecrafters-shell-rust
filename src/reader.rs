use lazy_static::lazy_static;
use rustyline::{
    Completer, Helper, Highlighter, Hinter,
    validate::{ValidationContext, ValidationResult, Validator},
};

// use radix_trie::Trie;
use trie_rs::Trie;

lazy_static! {
    static ref SUPPORT_COMMANDS: Trie<u8> = Trie::from_iter(["echo", "type", "exit", "pwd", "cd"]);
}
// static  SUPPORT_COMMANDS = Trie::from(value);

#[derive(Helper, Completer, Hinter, Highlighter)]
pub struct ShellHelper {
    // 不能直接使用库中的括号匹配，需要忽略单双引号之间的括号
}

impl ShellHelper {
    pub fn new() -> Self {
        Self {
            // validator: MatchingBracketValidator::new(),
        }
    }
}

impl Validator for ShellHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();
        let validation_res = if input.ends_with('\\') {
            ValidationResult::Incomplete
        } else {
            validate_brackets_and_quote(input)
        };
        Ok(validation_res)
    }
}

fn validate_brackets_and_quote(input: &str) -> ValidationResult {
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    let mut stack = vec![];
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' if !in_single_quote => {
                chars.next();
            }
            '\'' => {
                // 只有当前既不在 ' 中，也不在 " 中，遇到 ' 才会开始进入 ' 中
                in_single_quote = !in_single_quote && !in_double_quote;
            }
            '"' => {
                // 只有当前既不在 ' 中，也不在 " 中，遇到 ' 才会开始进入 " 中
                in_double_quote = !in_single_quote && !in_double_quote;
            }
            '(' | '[' | '{' if !in_single_quote && !in_double_quote => stack.push(c),
            ')' | ']' | '}' if !in_single_quote && !in_double_quote => match (stack.pop(), c) {
                (Some('('), ')') | (Some('['), ']') | (Some('{'), '}') => {}
                (Some(wanted), _) => {
                    return ValidationResult::Invalid(Some(format!(
                        "Mismatched brackets: {wanted:?} is not properly closed"
                    )));
                }
                (None, c) => {
                    return ValidationResult::Invalid(Some(format!(
                        "Mismatched brackets: {c:?} is unpaired"
                    )));
                }
            },
            _ => {}
        }
    }

    if stack.is_empty() && !in_single_quote && !in_double_quote {
        ValidationResult::Valid(None)
    } else {
        ValidationResult::Incomplete
    }
}
