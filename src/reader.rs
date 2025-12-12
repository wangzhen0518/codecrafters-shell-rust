use rustyline::{Completer, Helper, Highlighter, Hinter, Validator, hint::HistoryHinter};

use crate::{completer::ShellCompleter, validator::ShellValidator};

#[derive(Helper, Completer, Hinter, Highlighter, Validator)]
pub struct ShellHelper {
    // 不能直接使用库中的括号匹配，需要忽略单双引号之间的括号
    #[rustyline(Validator)]
    validator: ShellValidator,

    #[rustyline(Completer)]
    completer: ShellCompleter,

    #[rustyline(Hinter)]
    hinter: HistoryHinter,
}

impl ShellHelper {
    pub fn new() -> Self {
        Self {
            validator: ShellValidator,
            completer: ShellCompleter,
            hinter: HistoryHinter::new(),
        }
    }
}
