use std::{borrow::Cow, collections::HashSet, sync::RwLock};

use lazy_static::lazy_static;
use radix_trie::{Trie, TrieCommon};
use rustyline::{Changeset, completion::Completer, line_buffer::LineBuffer};

use crate::{
    builtin::BUILTIN_COMMANDS,
    executable::{PATH_ENV, PATHS, load_env_path, load_paths},
    utils::get_executables_from_dir,
};

lazy_static! {
    static ref SUPPORT_COMMANDS: RwLock<Trie<String, ()>> = {
        let mut commands: HashSet<String> =
            BUILTIN_COMMANDS.iter().map(|cmd| cmd.to_string()).collect();

        for dir in PATHS.read().unwrap_or_else(|err| err.into_inner()).iter() {
            for exec in get_executables_from_dir(dir) {
                if let Some(exec) = exec.file_name().and_then(|basename| basename.to_str()) {
                    commands.insert(exec.to_string());
                }
            }
        }

        RwLock::new(Trie::from_iter(commands.into_iter().map(|cmd| (cmd, ()))))
    };
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
        // 检查 PATH 是否有更新
        let new_env_path = load_env_path();
        let old_env_path = PATH_ENV.read().unwrap_or_else(|err| err.into_inner());
        if new_env_path.as_str() != old_env_path.as_str() {
            *PATH_ENV.write().unwrap() = new_env_path;

            let new_paths = HashSet::from_iter(load_paths());
            let old_paths = PATHS.read().unwrap_or_else(|err| err.into_inner());
            let added_paths = new_paths.difference(&old_paths);

            let mut writer = SUPPORT_COMMANDS.write().unwrap();
            for dir in added_paths {
                for exec in get_executables_from_dir(dir) {
                    if let Some(exec) = exec.file_name().and_then(|basename| basename.to_str()) {
                        writer.insert(exec.to_string(), ());
                    }
                }
            }
        }

        //TODO 支持 command, args 区分，支持不同类型的补全
        if let Some(sub_trie) = SUPPORT_COMMANDS.read().unwrap().get_raw_descendant(line) {
            // dbg!(&sub_trie);

            // candidates 无需排序，trie 中取出来之后就是按字典序排好序的
            let candidates: Vec<String> = sub_trie.keys().map(|key| key.to_string()).collect();
            // dbg!(&candidates);

            Ok((0, candidates))
        } else {
            Ok((pos, Vec::with_capacity(0)))
        }
    }

    fn update(&self, line: &mut LineBuffer, start: usize, elected: &str, cl: &mut Changeset) {
        let end = line.pos();
        let elected = if let Some(sub_trie) = SUPPORT_COMMANDS.read().unwrap().subtrie(elected)
            && sub_trie.is_leaf()
        {
            Cow::Owned(elected.to_string() + " ")
        } else {
            Cow::Borrowed(elected)
        };
        line.replace(start..end, &elected, cl);
    }
}
