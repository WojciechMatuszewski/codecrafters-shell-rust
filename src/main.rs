use std::io::{self};

use anyhow::Ok;

fn main() {
    let mut prompter = Prompter::new(io::stdin().lock(), io::stdout().lock());

    loop {
        prompter.write("$ ").unwrap();

        let input = prompter.read().unwrap();
        let result: Vec<&str> = input.split_whitespace().collect();
        if let [cmd] = result.as_slice() {
            match cmd {
                _ => println!("{}: command not found", cmd),
            }
        }
    }
}

struct Prompter<R: io::BufRead, W: io::Write> {
    input: R,
    output: W,
}

impl<R: io::BufRead, W: io::Write> Prompter<R, W> {
    fn new(input: R, output: W) -> Self {
        return Prompter { input, output };
    }

    fn read(&mut self) -> anyhow::Result<String> {
        let mut input = String::new();
        self.input.read_line(&mut input)?;

        return Ok(input.trim().to_string());
    }

    fn write(&mut self, prompt: &str) -> anyhow::Result<()> {
        write!(self.output, "{}", prompt)?;
        self.output.flush()?;

        return Ok(());
    }
}
