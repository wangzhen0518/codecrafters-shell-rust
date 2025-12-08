use lazy_static::lazy_static;
use trie_rs::Trie;

lazy_static! {
    static ref SUPPORT_COMMANDS: Trie<u8> = Trie::from_iter(["echo", "type", "exit", "pwd", "cd"]);
}
