use lazy_static::lazy_static;
use rustyline::completion::Completer;
use trie_rs::Trie;
// use radix_trie::Trie;

lazy_static! {
    static ref SUPPORT_COMMANDS: Trie<u8> = Trie::from_iter(["echo ", "type ", "exit ", "pwd ", "cd "]);
}
// static  SUPPORT_COMMANDS = Trie::from(value);

pub struct ShellCompleter;

impl Completer for ShellCompleter {
    type Candidate = String;

    fn complete(
        &self, // FIXME should be `&mut self`
        line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        //TODO 支持 command, args 区分，支持不同类型的补全
        Ok((0, SUPPORT_COMMANDS.predictive_search(line).collect()))
    }
}
