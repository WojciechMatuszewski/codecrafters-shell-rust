use std::{io, path::PathBuf, str::FromStr};

use anyhow::anyhow;

fn main() -> anyhow::Result<()> {
    let reader = io::stdin().lock();
    let writer = io::stdout();
    let mut prompter = Prompter::new(reader, writer);

    loop {
        prompter.prompt("$ ")?;

        let input = prompter.read()?;

        match input.parse::<Command>()? {
            Command::Exit(code) => std::process::exit(code),
            Command::Echo(prompt) => prompter.prompt(&prompt)?,
            Command::Type(prompt) => prompter.prompt(&prompt)?,
            Command::Pwd(prompt) => prompter.prompt(&prompt)?,
            Command::Executable(output) => prompter.prompt(&output)?,
            Command::Unknown(prompt) => prompter.prompt(&prompt)?,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Command {
    Exit(i32),
    Echo(String),
    Type(String),
    Pwd(String),
    Executable(String),
    Unknown(String),
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Command> {
        let path_env = std::env::var("PATH")?;

        let exec_path_looker = ExecPathLooker::new(path_env);
        let exec_runner = ExecRunner::new();

        let parsed: Vec<&str> = s.split_whitespace().collect();

        let [cmd, args @ ..] = parsed.as_slice() else {
            panic!("Could not parse arguments")
        };

        let cmd = cmd.trim();
        match cmd {
            "exit" => {
                let code = args
                    .get(0)
                    .ok_or(anyhow!("Invalid arguments"))?
                    .parse::<i32>()?;

                return Ok(Self::Exit(code));
            }
            "echo" => {
                let prompt = args.join(" ");
                let prompt = format!("{}\n", prompt);

                return Ok(Self::Echo(prompt));
            }
            "type" => {
                let cmd = args.get(0).ok_or(anyhow!("Invalid arguments"))?;
                let built_ins = vec!["exit", "echo", "type"];

                if built_ins.contains(cmd) {
                    let prompt = format!("{} is a shell builtin\n", cmd);
                    return Ok(Self::Type(prompt));
                }

                if let Some(full_path) = exec_path_looker.look_path(cmd) {
                    let prompt = format!("{} is {}\n", cmd, full_path);
                    return Ok(Self::Type(prompt));
                }

                let prompt = format!("{}: not found\n", cmd);
                return Ok(Self::Type(prompt));
            }
            "pwd" => {
                let pwd = std::env::current_dir()?;
                let pwd = pwd
                    .into_os_string()
                    .into_string()
                    .expect("Failed to convert path");

                return Ok(Self::Pwd(format!("{}\n", pwd)));
            }
            _ => {
                let output = exec_runner.execute(&cmd, args);
                if let Ok(result) = output {
                    return Ok(Self::Executable(result));
                }

                let prompt = format!("{}: command not found\n", cmd);
                return Ok(Self::Unknown(prompt));
            }
        }
    }
}

trait PathLooker {
    fn look_path(&self, name: &str) -> Option<String>;
}

struct ExecPathLooker {
    env_path: String,
}

impl ExecPathLooker {
    fn new(env_path: String) -> Self {
        return ExecPathLooker { env_path };
    }
}

impl PathLooker for ExecPathLooker {
    fn look_path(&self, exec_name: &str) -> Option<String> {
        let env_paths = self.env_path.split(":");

        for env_path in env_paths {
            let full_path: PathBuf = [env_path, exec_name].iter().collect();
            if full_path.exists() {
                return Some(
                    full_path
                        .into_os_string()
                        .into_string()
                        .expect("Failed to convert path"),
                );
            }
        }

        return None;
    }
}

struct ExecRunner {}

impl ExecRunner {
    fn new() -> Self {
        return ExecRunner {};
    }
    fn execute(&self, exec_name: &str, args: &[&str]) -> anyhow::Result<String> {
        let result = std::process::Command::new(exec_name).args(args).output()?;
        let output = String::from_utf8(result.stdout)?;

        return Ok(output);
    }
}

struct Prompter<R: io::BufRead, W: io::Write> {
    reader: R,
    writer: W,
}

impl<R: io::BufRead, W: io::Write> Prompter<R, W> {
    fn new(reader: R, writer: W) -> Self {
        return Prompter { reader, writer };
    }

    fn read(&mut self) -> anyhow::Result<String> {
        let mut input = String::new();
        self.reader.read_line(&mut input)?;

        return Ok(input.trim().to_string());
    }

    fn prompt(&mut self, prompt: &str) -> anyhow::Result<()> {
        write!(self.writer, "{}", prompt)?;
        self.writer.flush()?;

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompter_success() {
        let input = b"Hi there";
        let mut output = Vec::new();

        let mut prompter = Prompter::new(input.as_slice(), &mut output);

        prompter.prompt("first line\n").unwrap();
        prompter.prompt("second line\n").unwrap();

        let answer = prompter.read().unwrap();

        let written = String::from_utf8(output).unwrap();

        assert_eq!("first line\nsecond line\n", written);
        assert_eq!("Hi there", answer);
    }

    #[test]
    fn command_exit() {
        let input = "exit 10";
        let expected = Command::Exit(10);

        let command: Command = input.parse().unwrap();
        assert_eq!(command, expected)
    }

    #[test]
    fn command_echo() {
        let input = "echo foo bar baz";
        let expected = Command::Echo(String::from("foo bar baz\n"));

        let command: Command = input.parse().unwrap();
        assert_eq!(command, expected)
    }

    #[test]
    fn command_unknown() {
        let input = "idonotexist foo bar baz";
        let expected = Command::Unknown(String::from("idonotexist: command not found\n"));

        let command: Command = input.parse().unwrap();
        assert_eq!(command, expected)
    }
}
