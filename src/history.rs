use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
    sync::{Mutex, atomic::AtomicUsize},
};

use lazy_static::lazy_static;

use crate::{RL, Result};

lazy_static! {
    pub static ref CURRENT_SESSION_HISTORY: Mutex<Vec<String>> = Mutex::new(Vec::new());
    pub static ref LAST_APPEND_INDEX: AtomicUsize = AtomicUsize::default();
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
    let hists_to_save = if is_append {
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
        hist_iter.fold(
            hist.map_or(String::new(), |s| s.to_string() + "\n"),
            |acc, hist| acc + hist + "\n",
        )
    } else {
        rl.history()
            .iter()
            .fold(String::new(), |acc, hist| acc + hist + "\n")
    };
    fp.write_all(hists_to_save.as_bytes())?;
    Ok(())
}
