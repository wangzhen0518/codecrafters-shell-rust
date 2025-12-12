use lazy_static::lazy_static;
use rustyline::{Completer, Helper, Highlighter, Hinter, Validator};

// use radix_trie::Trie;
use trie_rs::Trie;

lazy_static! {
    static ref SUPPORT_COMMANDS: Trie<u8> = Trie::from_iter(["echo", "type", "exit", "pwd", "cd"]);
}
// static  SUPPORT_COMMANDS = Trie::from(value);

#[derive(Helper, Completer, Hinter, Highlighter, Validator)]
pub struct ShellHelper {}

impl ShellHelper {
    pub fn new() -> Self {
        Self {}
    }
}
