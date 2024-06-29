use std::io;

pub trait Prompter {
    fn read(&mut self) -> anyhow::Result<String>;
    fn prompt(&mut self, prompt: &str) -> anyhow::Result<()>;
}

pub struct ConsolePrompter<R: io::BufRead, W: io::Write> {
    reader: R,
    writer: W,
}

impl<R: io::BufRead, W: io::Write> Prompter for ConsolePrompter<R, W> {
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

impl<R: io::BufRead, W: io::Write> ConsolePrompter<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        return ConsolePrompter { reader, writer };
    }
}
