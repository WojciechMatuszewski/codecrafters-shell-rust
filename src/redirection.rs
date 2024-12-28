use anyhow::anyhow;
use std::{
    fs::{File, OpenOptions},
    io::Write,
};

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
        let Some(output_source) = args
            .get(0)
            .and_then(|raw_source| match raw_source.as_str() {
                ">" | "1>" => Some(OutputSource::Stdout(OutputMode::Override)),
                "2>" => Some(OutputSource::Stderr(OutputMode::Override)),
                ">>" | "1>>" => Some(OutputSource::Stdout(OutputMode::Append)),
                "2>>" => Some(OutputSource::Stderr(OutputMode::Append)),
                _ => None,
            })
        else {
            return Err(anyhow!(
                "Failed to create redirection: could not parse the output source"
            ));
        };

        let Some(target) = args.get(1) else {
            return Err(anyhow!("Failed to create redirection: target not found"));
        };

        return Ok(Self {
            source: output_source,
            target: target.to_string(),
        });
    }

    pub fn execute(self, stdout: &str, stderr: &str) -> anyhow::Result<()> {
        match self.source {
            OutputSource::Stdout(output_mode) => match output_mode {
                OutputMode::Append => {
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(self.target)?;
                    file.write(stdout.as_bytes())?;

                    return Ok(());
                }
                OutputMode::Override => {
                    let mut file = File::create(self.target)?;
                    file.write(stdout.as_bytes())?;

                    return Ok(());
                }
            },
            OutputSource::Stderr(output_mode) => match output_mode {
                OutputMode::Append => {
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(self.target)?;

                    file.write(stderr.as_bytes())?;

                    return Ok(());
                }
                OutputMode::Override => {
                    let mut file = File::create(self.target)?;
                    file.write(stderr.as_bytes())?;

                    return Ok(());
                }
            },
        }
    }
}
