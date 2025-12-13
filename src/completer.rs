use lazy_static::lazy_static;
use radix_trie::{Trie, TrieCommon};
use rustyline::completion::Completer;

macro_rules! add_suffix {
    // 处理数组
    ([$($key: expr), *], $suffix: expr) => {
        [$((concat!($key, $suffix), ())), *]
    };
    // 处理切片引用
    (&[$($key: expr), *], $suffix: expr) => {
        &[$((concat!($key, $suffix), ())),*]
    };
}

lazy_static! {
    static ref SUPPORT_COMMANDS: Trie<&'static str, ()> =
        Trie::from_iter(add_suffix!(["echo", "type", "exit", "pwd", "cd"], " "));
}

pub struct ShellCompleter;

impl Completer for ShellCompleter {
    type Candidate = String;

    fn complete(
        &self, // FIXME should be `&mut self`
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        //TODO 支持 command, args 区分，支持不同类型的补全
        if let Some(sub_trie) = SUPPORT_COMMANDS.get_raw_descendant(line) {
            Ok((0, sub_trie.keys().map(|key| key.to_string()).collect()))
        } else {
            Ok((pos, Vec::with_capacity(0)))
        }
    }
}
