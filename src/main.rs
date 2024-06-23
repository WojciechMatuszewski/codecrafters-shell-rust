use std::{io, str::FromStr};

use anyhow::anyhow;

fn main() -> anyhow::Result<()> {
    let reader = io::stdin().lock();
    let writer = io::stdout();
    let mut prompter = Prompter::new(reader, writer);

    loop {
        prompter.prompt("$ ")?;

        let input = prompter.read()?;
        let result: Vec<&str> = input.split_whitespace().collect();

        if let [cmd, args @ ..] = result.as_slice() {
            match cmd.trim().parse::<Command>()? {
                Command::Exit => {
                    let code = args
                        .get(0)
                        .ok_or(anyhow!("Invalid arguments"))?
                        .parse::<i32>()?;

                    std::process::exit(code)
                }
                _ => println!("{}: command not found", cmd),
            }
        }
    }
}

enum Command {
    Exit,
    Unknown,
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Command> {
        match s {
            "exit" => Ok(Command::Exit),
            _ => Ok(Command::Unknown),
        }
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
    fn prompter() {
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
}
