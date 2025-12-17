use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    str::FromStr,
};

use rustyline::history::History as _;

use crate::{
    RL, Result,
    builtin::ExitCode,
    command::{Execute, Parse, ParseCommandError},
    map_err_to_exit_code,
    redirect::{Reader, Writer},
};

// -r, -a -w
#[derive(Debug, PartialEq, Eq)]
pub enum History {
    Show(Option<usize>),
    Read(PathBuf),
    Write(PathBuf),
    Append(PathBuf),
}

impl Parse for History {
    fn parse(command: &str, args: &[String]) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        if args.len() <= 1 {
            let show_num = if args.is_empty() {
                None
            } else {
                Some(args[0].parse()?)
            };
            Ok(History::Show(show_num))
        } else if args.len() == 2 {
            let file = PathBuf::from_str(&args[1])?;
            let cmd = match args[0].as_str() {
                "-r" => {
                    if !file.is_file() {
                        return Err(format!(
                            "File {} does not exits or is not a file.",
                            file.display()
                        )
                        .into());
                    }
                    History::Read(file)
                }
                "-w" => History::Write(file),
                "-a" => History::Append(file),
                arg => {
                    return Err(format!("unknown parameter: {}", arg).into());
                }
            };
            Ok(cmd)
        } else {
            Err(ParseCommandError::MoreArgs(command.to_string(), args.to_vec(), 2).into())
        }
    }
}

impl Execute for History {
    fn execute(
        &self,
        _reader: Reader,
        mut output_writer: Writer,
        mut error_writer: Writer,
    ) -> ExitCode {
        match self {
            History::Show(show_num) => {
                let rl = map_err_to_exit_code!(RL.lock());
                let history = rl.history();
                let num = history.len();
                let length = (num as f64).log10() as usize + 1;
                let skip_num = show_num.map_or(0, |n| num - n);
                for (idx, record) in history.iter().enumerate().skip(skip_num) {
                    map_err_to_exit_code!(writeln!(
                        output_writer,
                        "   {:length$}  {}",
                        idx + 1,
                        record
                    ));
                }
                0
            }
            History::Read(file) => {
                if let Ok(fp) = File::open(file) {
                    let fp = BufReader::new(fp);
                    let mut rl = map_err_to_exit_code!(RL.lock());
                    for line in fp.lines() {
                        let line = map_err_to_exit_code!(line);
                        map_err_to_exit_code!(rl.add_history_entry(line));
                    }
                    0
                } else {
                    map_err_to_exit_code!(writeln!(
                        error_writer,
                        "Failed to read {}.",
                        file.display()
                    ));
                    -1
                }
            }
            History::Write(file) => {
                if let Ok(mut fp) = File::create(file) {
                    let rl = map_err_to_exit_code!(RL.lock());
                    let all_history = rl
                        .history()
                        .iter()
                        .fold(String::new(), |acc, hist| acc + hist + "\n");
                    map_err_to_exit_code!(fp.write_all(all_history.as_bytes()));
                    0
                } else {
                    map_err_to_exit_code!(writeln!(
                        error_writer,
                        "Failed to write {}.",
                        file.display()
                    ));
                    -1
                }
            }
            History::Append(file) => {
                if let Ok(mut fp) = OpenOptions::new().append(true).open(file) {
                    let rl = map_err_to_exit_code!(RL.lock());
                    let all_history = rl
                        .history()
                        .iter()
                        .fold(String::new(), |acc, hist| acc + hist + "\n");
                    map_err_to_exit_code!(fp.write_all(all_history.as_bytes()));
                    0
                } else {
                    map_err_to_exit_code!(writeln!(
                        error_writer,
                        "Failed to append {}.",
                        file.display()
                    ));
                    -1
                }
            }
        }
    }
}
