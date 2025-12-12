use lazy_static::lazy_static;
use rustyline::completion::Completer;
use trie_rs::Trie;
// use radix_trie::Trie;

macro_rules! add_suffix {
    // 处理数组
    ([$($str:expr),*], $suffix:expr) => {
        [$(concat!($str, $suffix)),*]
    };
    // 处理切片引用
    (&[$($str:expr),*], $suffix:expr) => {
        &[$(concat!($str, $suffix)),*]
    };
}

lazy_static! {
    // static ref SUPPORT_COMMANDS: Trie<u8> =
    //     Trie::from_iter(["echo ", "type ", "exit ", "pwd ", "cd "]);
    static ref SUPPORT_COMMANDS: Trie<u8> =
        Trie::from_iter(add_suffix!(["echo", "type", "exit", "pwd", "cd"], " "));
}
// static  SUPPORT_COMMANDS = Trie::from(value);

// const fn add_space<const N: usize>(cmds: [&'static str; N]) -> [&'static str; N] {
//     cmds.map(|cmd| cmd)
// }

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
