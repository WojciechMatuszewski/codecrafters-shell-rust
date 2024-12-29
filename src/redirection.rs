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
    pub target: String,
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
            target: target.to_string(),
        });
    }

    // pub fn execute(self, stdout: &str, stderr: &str) -> anyhow::Result<()> {
    //     match self.source {
    //         Source::Stdout(output_mode) => match output_mode {
    //             OutputMode::Append => {
    //                 let mut file = OpenOptions::new()
    //                     .append(true)
    //                     .create(true)
    //                     .open(self.target)?;
    //                 file.write(stdout.as_bytes())?;

    //                 return Ok(());
    //             }
    //             OutputMode::Override => {
    //                 let mut file = File::create(self.target)?;
    //                 file.write(stdout.as_bytes())?;

    //                 return Ok(());
    //             }
    //         },
    //         Source::Stderr(output_mode) => match output_mode {
    //             OutputMode::Append => {
    //                 let mut file = OpenOptions::new()
    //                     .append(true)
    //                     .create(true)
    //                     .open(self.target)?;
    //                 file.write(stderr.as_bytes())?;

    //                 return Ok(());
    //             }
    //             OutputMode::Override => {
    //                 let mut file = File::create(self.target)?;
    //                 file.write(stderr.as_bytes())?;

    //                 return Ok(());
    //             }
    //         },
    //     }
    // }

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
