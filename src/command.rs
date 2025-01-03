use anyhow::anyhow;
use std::str::FromStr;

use crate::{
    executable::{ExecutablePathFinder, ExecutableRunner},
    prompt::Prompter,
    redirection::{self, Redirection},
};

#[derive(Debug, PartialEq)]
enum TypeCommand {
    WellKnown { cmd: String },
    Unknown { cmd: String },
}

#[derive(Debug, PartialEq)]
enum BuiltinCommand {
    Exit { code: i32 },
    Echo { input: String },
    Type(TypeCommand),
    Pwd,
    Cd { path: String },
}

#[derive(Debug, PartialEq)]
enum CommandKind {
    Builtin(BuiltinCommand),
    Unknown { cmd: String, args: Vec<String> },
}

impl CommandKind {
    fn new(args: Vec<String>) -> anyhow::Result<Self> {
        let [cmd, args @ ..] = args.as_slice() else {
            return Err(anyhow!("Failed to construct CommandKind"));
        };
        let cmd = cmd.trim();

        match cmd {
            "exit" => {
                let code = args
                    .get(0)
                    .ok_or(anyhow!("Invalid arguments"))?
                    .parse::<i32>()?;

                let command = Self::Builtin(BuiltinCommand::Exit { code });
                return Ok(command);
            }
            "echo" => {
                let input = args.join(" ");
                let command = Self::Builtin(BuiltinCommand::Echo { input });
                return Ok(command);
            }
            "type" => {
                let cmd = args.get(0).ok_or(anyhow!("Invalid arguments"))?;
                let built_ins = vec![
                    String::from("exit"),
                    String::from("echo"),
                    String::from("type"),
                    String::from("pwd"),
                ];

                if built_ins.contains(cmd) {
                    let command = Self::Builtin(BuiltinCommand::Type(TypeCommand::WellKnown {
                        cmd: cmd.to_string(),
                    }));

                    return Ok(command);
                }

                let command = Self::Builtin(BuiltinCommand::Type(TypeCommand::Unknown {
                    cmd: cmd.to_string(),
                }));
                return Ok(command);
            }
            "pwd" => {
                let command = Self::Builtin(BuiltinCommand::Pwd);
                return Ok(command);
            }
            "cd" => {
                let path = args.get(0).ok_or(anyhow!("Invalid arguments"))?.to_string();
                let command = Self::Builtin(BuiltinCommand::Cd { path });
                return Ok(command);
            }
            _ => {
                let cmd = cmd.to_string();
                let args: Vec<String> = args.iter().map(|v| v.to_string()).collect();

                let command = Self::Unknown { cmd, args };
                return Ok(command);
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Command {
    kind: CommandKind,
    redirection: Option<Redirection>,
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input_args = parse_args(input);
        let redirection_start_index = input_args
            .iter()
            .position(|input_arg| return Redirection::is_redirection_arg(&input_arg));

        match redirection_start_index {
            Some(index) => {
                let cmd = CommandKind::new(input_args[..index].to_vec())?;
                let redirection = Redirection::new(input_args[index..].to_vec())?;

                return Ok(Command {
                    kind: cmd,
                    redirection: Some(redirection),
                });
            }
            None => {
                let cmd = CommandKind::new(input_args)?;
                return Ok(Command {
                    kind: cmd,
                    redirection: None,
                });
            }
        }
    }
}

#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl Command {
    pub fn run(
        self,
        prompter: &mut impl Prompter,
        finder: &impl ExecutablePathFinder,
        runner: &impl ExecutableRunner,
    ) -> anyhow::Result<()> {
        let Some(output) = (match self.kind {
            CommandKind::Builtin(builtin_command) => {
                match run_builtin_command(builtin_command, finder) {
                    Ok(output) => Some(output),
                    Err(e) => {
                        println!("Command error");
                        prompter.prompt(&e.to_string())?;
                        None
                    }
                }
            }
            CommandKind::Unknown { cmd, args } => match run_unknown_command(runner, cmd, args) {
                Ok(output) => Some(output),
                Err(e) => {
                    println!("Command error");
                    prompter.prompt(&e.to_string())?;
                    None
                }
            },
        }) else {
            return Ok(());
        };

        if let Some(redirection) = self.redirection {
            redirection.run(&output)?;

            match redirection.source {
                redirection::Source::Stdout(_) if output.stderr.is_some() => {
                    prompter.prompt(&output.stderr.unwrap_or("".to_string()))?
                }
                redirection::Source::Stderr(_) => {
                    prompter.prompt(&output.stdout.unwrap_or("".to_string()))?
                }
                _ => {}
            }
        } else if let Some(prompt_output) = output.stderr.or(output.stdout) {
            prompter.prompt(&prompt_output)?;
        }

        return Ok(());
    }
}

fn run_builtin_command(
    command: BuiltinCommand,
    finder: &impl ExecutablePathFinder,
) -> anyhow::Result<CommandOutput> {
    match command {
        BuiltinCommand::Exit { code } => {
            std::process::exit(code);
        }
        BuiltinCommand::Echo { input } => {
            return Ok(CommandOutput {
                stdout: Some(format!("{}\n", input)),
                stderr: None,
            });
        }
        BuiltinCommand::Type(command) => match command {
            TypeCommand::WellKnown { cmd } => {
                return Ok(CommandOutput {
                    stdout: Some(format!("{} is a shell builtin\n", cmd)),
                    stderr: None,
                })
            }
            TypeCommand::Unknown { cmd } => {
                let env_path = std::env::var("PATH")?;
                let result = finder.find_executable_path(&env_path, &cmd);

                match result {
                    Some(full_path) => {
                        return Ok(CommandOutput {
                            stdout: Some(format!("{} is {}\n", cmd, full_path)),
                            stderr: None,
                        });
                    }
                    None => {
                        return Ok(CommandOutput {
                            stdout: None,
                            stderr: Some(format!("{}: not found\n", cmd)),
                        });
                    }
                }
            }
        },
        BuiltinCommand::Pwd => {
            let pwd = std::env::current_dir()?;
            let pwd = pwd
                .into_os_string()
                .into_string()
                .expect("Failed to convert path");

            return Ok(CommandOutput {
                stdout: Some(format!("{}\n", pwd)),
                stderr: None,
            });
        }
        BuiltinCommand::Cd { path } => {
            let home_path =
                std::env::home_dir().ok_or(anyhow!("Could not get the home directory"))?;
            let home_path = home_path.to_str().expect("Could not convert the path");

            let path = path.replace("~", home_path);

            let result = std::env::set_current_dir(&path);
            if let Err(e) = result {
                match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        return Ok(CommandOutput {
                            stdout: None,
                            stderr: Some(format!("cd: {}: No such file or directory\n", path)),
                        });
                    }
                    _ => return Err(anyhow!("Unknown error")),
                }
            };

            return Ok(CommandOutput {
                stdout: None,
                stderr: None,
            });
        }
    }
}

fn run_unknown_command(
    runner: &impl ExecutableRunner,
    cmd: String,
    args: Vec<String>,
) -> anyhow::Result<CommandOutput> {
    let args: Vec<&str> = args.iter().map(|arg| arg.as_str()).collect();
    let args = args.as_slice();

    let output = runner.execute(&cmd, args)?;
    return Ok(CommandOutput {
        stdout: output.stdout,
        stderr: output.stderr,
    });
}

fn parse_args(args: &str) -> Vec<String> {
    let mut current_arg = String::new();
    let mut parsed_args: Vec<String> = vec![];

    let mut inside_single_quotes = false;
    let mut inside_double_quotes = false;

    for (index, current_char) in args.chars().enumerate() {
        let prev_char = if index > 0 {
            args.chars().nth(index - 1)
        } else {
            None
        };

        let next_char = args.chars().nth(index + 1);

        match current_char {
            '\'' => {
                let is_previous_escape_char = prev_char == Some('\\');
                if is_previous_escape_char {
                    current_arg.push(current_char);
                } else if inside_double_quotes {
                    current_arg.push(current_char)
                } else {
                    inside_single_quotes = !inside_single_quotes;
                }
            }
            '\\' => {
                if inside_single_quotes {
                    current_arg.push(current_char);
                }

                if inside_double_quotes {
                    if next_char == Some('"') {
                        continue;
                    }

                    if next_char == Some('$') {
                        continue;
                    }

                    if prev_char == Some('\\') {
                        continue;
                    }

                    current_arg.push(current_char);
                }
            }
            '"' => {
                if inside_double_quotes && next_char.is_none() {
                    continue;
                }

                let is_previous_escape_char = prev_char == Some('\\');
                if is_previous_escape_char {
                    current_arg.push(current_char)
                } else if inside_single_quotes {
                    current_arg.push(current_char);
                } else {
                    inside_double_quotes = !inside_double_quotes
                }
            }
            ' ' => {
                let is_previous_escape_char = prev_char == Some('\\');
                if inside_single_quotes {
                    current_arg.push(current_char);
                } else if inside_double_quotes {
                    current_arg.push(current_char);
                } else if is_previous_escape_char {
                    current_arg.push(current_char);
                } else if !current_arg.is_empty() {
                    parsed_args.push(current_arg.clone());
                    current_arg.clear();
                }
            }
            _ => {
                current_arg.push(current_char);
            }
        }
    }

    parsed_args.push(current_arg);
    return parsed_args;
}

#[cfg(test)]
mod parse_args_tests {
    use super::*;

    #[test]
    fn single_arg() {
        let args = r#"single"#;

        let output = parse_args(args);
        let expected = vec![r#"single"#.to_string()];

        assert_eq!(output, expected)
    }

    #[test]
    fn single_arg_single_quotes() {
        let args = r#"'single'"#;

        let output = parse_args(args);
        let expected = vec![r#"single"#.to_string()];

        assert_eq!(output, expected)
    }

    #[test]
    fn single_arg_escaped_single_quote() {
        {
            let args = r#"sing\'le"#;

            let output = parse_args(args);
            let expected = vec![r#"sing'le"#.to_string()];

            assert_eq!(output, expected)
        }

        {
            let args = r#"\'single"#;

            let output = parse_args(args);
            let expected = vec![r#"'single"#.to_string()];

            assert_eq!(output, expected)
        }

        {
            let args = r#"single\'"#;

            let output = parse_args(args);
            let expected = vec![r#"single'"#.to_string()];

            assert_eq!(output, expected)
        }
    }

    #[test]
    fn multiple_args_escaped_quote() {
        let args = r#"f\'irst secon\'d"#;

        let output = parse_args(args);
        let expected = vec![r#"f'irst"#.to_string(), r#"secon'd"#.to_string()];

        assert_eq!(output, expected);
    }

    #[test]
    fn single_arg_double_quotes() {
        let args = r#""single""#;

        let output = parse_args(args);
        let expected = vec![r#"single"#.to_string()];

        assert_eq!(output, expected)
    }

    #[test]
    fn escaped_double_quotes() {
        let args = r#"first\"second"#;

        let output = parse_args(args);
        let expected = vec![r#"first"second"#.to_string()];

        assert_eq!(output, expected);
    }

    #[test]
    fn escaped_double_quotes_inside_single_quotes() {
        let args = r#"'sin\"gle'"#;

        let output = parse_args(args);
        let expected = vec![r#"sin\"gle"#.to_string()];

        assert_eq!(output, expected)
    }

    #[test]
    fn multiple_args() {
        let args = r#"first second"#;

        let output = parse_args(args);
        let expected = vec![r#"first"#.to_string(), r#"second"#.to_string()];

        assert_eq!(output, expected)
    }

    #[test]
    fn double_quotes_inside_single_quotes() {
        let args = r#"'first"second' '"first second'"#;

        let output = parse_args(args);
        let expected = vec![
            r#"first"second"#.to_string(),
            r#""first second"#.to_string(),
        ];

        assert_eq!(output, expected);
    }

    #[test]
    fn multiple_args_double_quotes_whitespace() {
        let args = r#""first  second"   "first""#;

        let output = parse_args(args);
        let expected = vec![r#"first  second"#.to_string(), r#"first"#.to_string()];

        assert_eq!(output, expected);
    }

    #[test]
    fn single_quoted_backslash() {
        let args = r#""/'f \21\'""#;

        let output = parse_args(args);
        let expected = vec![r#"/'f \21\'"#.to_string()];

        assert_eq!(output, expected)
    }

    #[test]
    fn single_quoted_backslash2() {
        let args = r#""/'f  \78'""#;

        let output = parse_args(args);
        let expected = vec![r#"/'f  \78'"#.to_string()];

        assert_eq!(output, expected)
    }

    #[test]
    fn non_quoted_backslash_space() {
        let args = r#"first\ \ \second"#;

        let output = parse_args(args);
        let expected = vec![r#"first  second"#.to_string()];

        assert_eq!(output, expected)
    }

    #[test]
    fn double_quoted_backslash() {
        let args = r#""test'world'\\n'example""#;

        let output = parse_args(args);
        let expected = vec![r#"test'world'\n'example"#.to_string()];

        assert_eq!(output, expected)
    }

    #[test]
    fn double_quoted_backslash2() {
        let args = r#""mixed\"quote'test'\\""#;

        let output = parse_args(args);
        let expected = vec![r#"mixed"quote'test'\"#.to_string()];

        assert_eq!(output, expected);
    }

    #[test]
    fn double_quoted_backslash3() {
        let args = r#""example\"insidequotes"script\""#;

        let output = parse_args(args);
        let expected = vec![r#"example"insidequotesscript""#.to_string()];

        assert_eq!(output, expected)
    }

    #[test]
    fn stdout_redirect() {
        let args = r#"'hello james' 1> /tmp/foo/foo.md"#;

        let output = parse_args(args);
        let expected = vec![
            r#"hello james"#.to_string(),
            "1>".to_string(),
            "/tmp/foo/foo.md".to_string(),
        ];

        assert_eq!(output, expected)
    }
}
