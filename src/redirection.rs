use anyhow::anyhow;
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
};

use crate::command::CommandOutput;

#[derive(Debug, PartialEq)]
pub enum OutputMode {
    Append,
    Override,
}

#[derive(Debug, PartialEq)]
pub enum Source {
    Stdout(OutputMode),
    Stderr(OutputMode),
}

#[derive(Debug, PartialEq)]
pub struct Redirection {
    pub source: Source,
    pub target: PathBuf,
}

const STDOUT_OVERRIDE: &[&str] = &[">", "1>"];
const STDOUT_APPEND: &[&str] = &[">>", "1>>"];
const STDERR_OVERRIDE: &[&str] = &["2>"];
const STDERR_APPEND: &[&str] = &["2>>"];

impl Redirection {
    pub fn new(args: Vec<String>) -> anyhow::Result<Self> {
        let Some(output_source) = args
            .get(0)
            .and_then(|raw_source| match raw_source.as_str() {
                s if STDOUT_OVERRIDE.contains(&s) => Some(Source::Stdout(OutputMode::Override)),
                s if STDOUT_APPEND.contains(&s) => Some(Source::Stdout(OutputMode::Append)),
                s if STDERR_OVERRIDE.contains(&s) => Some(Source::Stderr(OutputMode::Override)),
                s if STDERR_APPEND.contains(&s) => Some(Source::Stderr(OutputMode::Append)),
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
            target: PathBuf::from(target),
        });
    }

    pub fn run(&self, command_output: &CommandOutput) -> anyhow::Result<()> {
        let path = PathBuf::from(&self.target);

        match &self.source {
            Source::Stdout(output_mode) => match output_mode {
                OutputMode::Append => {
                    let mut file = OpenOptions::new().append(true).create(true).open(path)?;

                    file.write(
                        command_output
                            .stdout
                            .clone()
                            .unwrap_or("".to_string())
                            .as_bytes(),
                    )?;

                    return Ok(());
                }
                OutputMode::Override => {
                    let mut file = File::create(path)?;

                    file.write(
                        command_output
                            .stdout
                            .clone()
                            .unwrap_or("".to_string())
                            .as_bytes(),
                    )?;

                    return Ok(());
                }
            },
            Source::Stderr(output_mode) => match output_mode {
                OutputMode::Append => {
                    let mut file = OpenOptions::new().append(true).create(true).open(path)?;

                    file.write(
                        command_output
                            .stderr
                            .clone()
                            .unwrap_or("".to_string())
                            .as_bytes(),
                    )?;

                    return Ok(());
                }
                OutputMode::Override => {
                    let mut file = File::create(path)?;

                    file.write(
                        command_output
                            .stderr
                            .clone()
                            .unwrap_or("".to_string())
                            .as_bytes(),
                    )?;

                    return Ok(());
                }
            },
        }
    }

    pub fn is_redirection_arg(arg: &str) -> bool {
        return [
            STDOUT_APPEND,
            STDOUT_OVERRIDE,
            STDERR_APPEND,
            STDERR_OVERRIDE,
        ]
        .concat()
        .iter()
        .any(|&redirection_arg| return redirection_arg == arg);
    }
}

#[cfg(test)]
mod redirection_tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use crate::{
        command::CommandOutput,
        redirection::{STDOUT_APPEND, STDOUT_OVERRIDE},
    };

    use super::Redirection;

    #[test]
    fn test_stdout_override() -> anyhow::Result<()> {
        let file = NamedTempFile::new()?;
        let path = file.path();

        let initial_content = "initial_content";
        let expected_content = "expected_content";

        fs::write(path, initial_content)?;

        let command_output = CommandOutput {
            stdout: Some(expected_content.to_string()),
            stderr: None,
        };

        let redirection = Redirection::new(vec![
            STDOUT_OVERRIDE[0].to_string(),
            path.to_string_lossy().to_string(),
        ])?;
        redirection.run(&command_output)?;

        let file_content = fs::read_to_string(path)?;
        assert_eq!(file_content, expected_content);

        return Ok(());
    }

    #[test]
    fn test_stdout_append() -> anyhow::Result<()> {
        let file = NamedTempFile::new()?;
        let path = file.path();

        let initial_content = "initial_content";
        let additional_content = "additional_content";

        fs::write(path, initial_content)?;

        let command_output = CommandOutput {
            stdout: Some(additional_content.to_string()),
            stderr: None,
        };

        let redirection = Redirection::new(vec![
            STDOUT_APPEND[0].to_string(),
            path.to_string_lossy().to_string(),
        ])?;
        redirection.run(&command_output)?;

        let file_content = fs::read_to_string(path)?;
        assert_eq!(
            file_content,
            format!("{}{}", initial_content, additional_content)
        );

        return Ok(());
    }
}
