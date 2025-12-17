use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
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
                if load_history(file).is_err() {
                    writeln!(error_writer, "Failed to read {}.", file.display()).ok();
                    -1
                } else {
                    0
                }
            }
            History::Write(file) => {
                if save_history(file, false).is_err() {
                    writeln!(error_writer, "Failed to write {}.", file.display()).ok();
                    -1
                } else {
                    0
                }
            }
            History::Append(file) => {
                if save_history(file, true).is_err() {
                    writeln!(error_writer, "Failed to append {}.", file.display()).ok();
                    -1
                } else {
                    0
                }
            }
        }
    }
}

pub fn load_history<P: AsRef<Path>>(file: P) -> Result<()> {
    let fp = BufReader::new(File::open(file)?);
    let mut rl = RL.lock().expect("Failed to require history");
    for line in fp.lines() {
        rl.add_history_entry(line?).ok();
    }
    Ok(())
}

pub fn save_history<P: AsRef<Path>>(file: P, is_append: bool) -> Result<()> {
    let mut fp = OpenOptions::new()
        .read(true)
        .write(true)
        .append(is_append)
        .create(true)
        .open(file)?;
    let rl = RL.lock().expect("Failed to require history");
    let hists_to_write = if is_append {
        let mut hist_iter = rl.history().iter();
        let mut fp_iter = BufReader::new(&fp).lines();

        let mut hist = hist_iter.next();
        let mut line = fp_iter.next();

        while hist.is_some() && line.is_some() {
            if *hist.unwrap() != line.unwrap()? {
                break;
            }
            hist = hist_iter.next();
            line = fp_iter.next();
        }
        hist_iter.fold(String::new(), |acc, hist| acc + hist + "\n")
    } else {
        rl.history()
            .iter()
            .fold(String::new(), |acc, hist| acc + hist + "\n")
    };
    fp.write_all(hists_to_write.as_bytes())?;
    Ok(())
}
