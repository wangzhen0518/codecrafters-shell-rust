use std::path::Path;

use rustyline::{
    Config,
    history::{DefaultHistory, FileHistory, History, SearchDirection, SearchResult},
};

pub struct ShellHistory {
    inner: FileHistory,
}

impl ShellHistory {
    pub fn with_config(config: &Config) -> ShellHistory {
        ShellHistory {
            inner: DefaultHistory::with_config(config),
        }
    }
}

impl History for ShellHistory {
    fn get(
        &self,
        index: usize,
        dir: rustyline::history::SearchDirection,
    ) -> rustyline::Result<Option<rustyline::history::SearchResult<'_>>> {
        self.inner.get(index, dir)
    }

    fn add(&mut self, line: &str) -> rustyline::Result<bool> {
        self.inner.add(line)
    }

    fn add_owned(&mut self, line: String) -> rustyline::Result<bool> {
        self.inner.add_owned(line)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn set_max_len(&mut self, len: usize) -> rustyline::Result<()> {
        self.inner.set_max_len(len)
    }

    fn ignore_dups(&mut self, yes: bool) -> rustyline::Result<()> {
        self.inner.ignore_dups(yes)
    }

    fn ignore_space(&mut self, yes: bool) {
        self.inner.ignore_space(yes);
    }

    fn save(&mut self, path: &Path) -> rustyline::Result<()> {
        self.inner.save(path)
    }

    fn append(&mut self, path: &Path) -> rustyline::Result<()> {
        self.inner.append(path)
    }

    fn load(&mut self, path: &Path) -> rustyline::Result<()> {
        self.inner.load(path)
    }

    fn clear(&mut self) -> rustyline::Result<()> {
        self.inner.clear()
    }

    fn search(
        &self,
        term: &str,
        start: usize,
        dir: SearchDirection,
    ) -> rustyline::Result<Option<SearchResult<'_>>> {
        self.inner.search(term, start, dir)
    }

    fn starts_with(
        &self,
        term: &str,
        start: usize,
        dir: SearchDirection,
    ) -> rustyline::Result<Option<SearchResult<'_>>> {
        self.inner.starts_with(term, start, dir)
    }
}
