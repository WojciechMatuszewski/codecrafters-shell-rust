use anyhow::anyhow;
use std::{fs::File, io::Write};

#[derive(Debug, PartialEq)]
pub enum OutputMode {
    Append,
    Override,
}

#[derive(Debug, PartialEq)]
pub enum OutputSource {
    Stdout(OutputMode),
    Stderr(OutputMode),
}

#[derive(Debug, PartialEq)]
pub struct Redirection {
    pub source: OutputSource,
    pub target: String,
}

impl Redirection {
    pub fn new(args: Vec<String>) -> anyhow::Result<Self> {
        let Some(target) = args.get(1) else {
            return Err(anyhow!("Failed to create redirection: target not found"));
        };

        return Ok(Self {
            source: OutputSource::Stdout(OutputMode::Override),
            target: target.to_string(),
        });
    }

    pub fn execute(self, input: &str) -> anyhow::Result<()> {
        let mut file = File::create(self.target)?;
        file.write(input.as_bytes())?;

        return Ok(());
    }
}
