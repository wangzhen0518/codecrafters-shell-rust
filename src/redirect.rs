use std::{fs, io, process};

use crate::Result;

#[derive(Debug)]
pub enum Writer {
    Stdout(io::Stdout),
    Stderr(io::Stderr),
    PipeWriter(io::PipeWriter),
    File(fs::File),
    Null,
}

impl Writer {
    pub fn try_clone(&self) -> Result<Writer> {
        let writer = match self {
            Writer::Stdout(_) => Writer::Stdout(io::stdout()),
            Writer::Stderr(_) => Writer::Stderr(io::stderr()),
            Writer::PipeWriter(pipe_writer) => Writer::PipeWriter(pipe_writer.try_clone()?),
            Writer::File(file) => Writer::File(file.try_clone()?),
            Writer::Null => Writer::Null,
        };
        Ok(writer)
    }
}

impl io::Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Writer::Stdout(stdout) => stdout.write(buf),
            Writer::Stderr(stderr) => stderr.write(buf),
            Writer::PipeWriter(pipe_writer) => pipe_writer.write(buf),
            Writer::File(file) => file.write(buf),
            Writer::Null => Ok(buf.len()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Writer::Stdout(stdout) => stdout.flush(),
            Writer::Stderr(stderr) => stderr.flush(),
            Writer::PipeWriter(pipe_writer) => pipe_writer.flush(),
            Writer::File(file) => file.flush(),
            Writer::Null => Ok(()),
        }
    }
}

impl From<Writer> for process::Stdio {
    fn from(value: Writer) -> Self {
        match value {
            Writer::Stdout(stdout) => process::Stdio::from(stdout),
            Writer::Stderr(stderr) => process::Stdio::from(stderr),
            Writer::PipeWriter(pipe_writer) => process::Stdio::from(pipe_writer),
            Writer::File(file) => process::Stdio::from(file),
            Writer::Null => process::Stdio::null(),
        }
    }
}

impl From<io::Stdout> for Writer {
    fn from(value: io::Stdout) -> Self {
        Writer::Stdout(value)
    }
}

impl From<io::Stderr> for Writer {
    fn from(value: io::Stderr) -> Self {
        Writer::Stderr(value)
    }
}

impl From<io::PipeWriter> for Writer {
    fn from(value: io::PipeWriter) -> Self {
        Writer::PipeWriter(value)
    }
}

impl From<fs::File> for Writer {
    fn from(value: fs::File) -> Self {
        Writer::File(value)
    }
}

#[derive(Debug)]
pub enum Reader {
    Stdin,
    PipeReader(io::PipeReader),
    File(fs::File),
}

impl Reader {
    pub fn try_clone(&self) -> Result<Reader> {
        let reader = match self {
            Reader::Stdin => Reader::Stdin,
            Reader::PipeReader(pipe_reader) => Reader::PipeReader(pipe_reader.try_clone()?),
            Reader::File(file) => Reader::File(file.try_clone()?),
        };
        Ok(reader)
    }
}

impl io::Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Reader::Stdin => Ok(0),
            Reader::PipeReader(pipe_reader) => pipe_reader.read(buf),
            Reader::File(file) => file.read(buf),
        }
    }
}

impl From<Reader> for process::Stdio {
    fn from(value: Reader) -> Self {
        match value {
            Reader::Stdin => process::Stdio::inherit(),
            Reader::PipeReader(pipe_reader) => process::Stdio::from(pipe_reader),
            Reader::File(file) => process::Stdio::from(file),
        }
    }
}

impl From<io::PipeReader> for Reader {
    fn from(value: io::PipeReader) -> Self {
        Reader::PipeReader(value)
    }
}

impl From<fs::File> for Reader {
    fn from(value: fs::File) -> Self {
        Reader::File(value)
    }
}
