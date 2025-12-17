use std::io::Write;

use crate::{
    Result,
    builtin::{BUILTIN_COMMANDS, ExitCode},
    command::{Execute, Parse, ParseCommandError},
    executable::find_in_path,
    redirect::{Reader, Writer},
};

#[derive(Debug, PartialEq, Eq)]
pub struct Type {
    commands: Vec<String>,
}

impl Type {
    pub fn new(commands: Vec<String>) -> Self {
        Self { commands }
    }
}

impl Parse for Type {
    fn parse(command: &str, args: &[String]) -> Result<Self>
    where
        Self: std::marker::Sized,
    {
        if args.is_empty() {
            return Err(ParseCommandError::LessArgs(command.to_string(), args.to_vec(), 1).into());
        }

        Ok(Type::new(args.to_vec()))
    }
}

impl Execute for Type {
    fn execute(
        &self,
        _reader: Reader,
        mut output_writer: Writer,
        _error_writer: Writer,
    ) -> ExitCode {
        for cmd in &self.commands {
            let exec_res = if BUILTIN_COMMANDS.contains(cmd.as_str()) {
                writeln!(output_writer, "{} is a shell builtin", cmd)
            } else if let Some(path) = find_in_path(cmd) {
                writeln!(output_writer, "{} is {}", cmd, path.display())
            } else {
                writeln!(output_writer, "{}: not found", cmd)
            };
            if exec_res.is_err() {
                return -1;
            }
        }

        0
    }
}

#[cfg(test)]
mod test {
    use std::{fs, io};

    use crate::{
        command::{Execute, Parse},
        redirect::{Reader, Writer},
        utils::{set_env_path, vec_str_to_vec_string},
    };

    use super::Type;

    #[test]
    fn test_parse_type() {
        set_env_path();
        let output_file = "/tmp/test_parse_type.txt";
        let ty = Type::parse(
            "type",
            &vec_str_to_vec_string::<Vec<_>>(&["echo", "type", "exit", "ls", "invalid_command"]),
        )
        .unwrap();
        let file = fs::File::create(output_file).unwrap();
        let exit_code = ty.execute(Reader::Stdin, file.into(), Writer::Stderr(io::stderr()));
        assert_eq!(exit_code, 0);

        let exec_res = fs::read_to_string(output_file).unwrap();
        assert_eq!(
            exec_res,
            [
                "echo is a shell builtin",
                "type is a shell builtin",
                "exit is a shell builtin",
                "ls is /usr/bin/ls",
                "invalid_command: not found\n",
            ]
            .join("\n")
        );
    }
}
