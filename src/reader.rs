use lazy_static::lazy_static;
use rustyline::{Completer, Helper, Highlighter, Hinter, Validator};

// use radix_trie::Trie;
use trie_rs::Trie;

use crate::validator::ShellValidator;

lazy_static! {
    static ref SUPPORT_COMMANDS: Trie<u8> = Trie::from_iter(["echo", "type", "exit", "pwd", "cd"]);
}
// static  SUPPORT_COMMANDS = Trie::from(value);

#[derive(Helper, Completer, Hinter, Highlighter, Validator)]
pub struct ShellHelper {
    // 不能直接使用库中的括号匹配，需要忽略单双引号之间的括号
    #[rustyline(Validator)]
    validator: ShellValidator,
}

impl ShellHelper {
    pub fn new() -> Self {
        Self {
            validator: ShellValidator,
        }
    }
}
