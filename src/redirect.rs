use std::{fs, io, process};

#[derive(Debug)]
pub enum Writer {
    Stdout(io::Stdout),
    Stderr(io::Stderr),
    PipeWriter(io::PipeWriter),
    File(fs::File),
}

impl io::Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Writer::Stdout(stdout) => stdout.write(buf),
            Writer::Stderr(stderr) => stderr.write(buf),
            Writer::PipeWriter(pipe_writer) => pipe_writer.write(buf),
            Writer::File(file) => file.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Writer::Stdout(stdout) => stdout.flush(),
            Writer::Stderr(stderr) => stderr.flush(),
            Writer::PipeWriter(pipe_writer) => pipe_writer.flush(),
            Writer::File(file) => file.flush(),
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

pub enum Reader {
    PipeReader(io::PipeReader),
}

impl io::Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Reader::PipeReader(pipe_reader) => pipe_reader.read(buf),
        }
    }
}

impl From<Reader> for process::Stdio {
    fn from(value: Reader) -> Self {
        match value {
            Reader::PipeReader(pipe_reader) => process::Stdio::from(pipe_reader),
        }
    }
}
