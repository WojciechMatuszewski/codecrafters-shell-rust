use std::str::FromStr;

use anyhow::anyhow;

use crate::{
    executable::{ExecutablePathFinder, ExecutableRunner},
    prompt::Prompter,
};

#[derive(Debug, PartialEq)]
pub enum TypeCommand {
    WellKnown { cmd: String },
    Unknown { cmd: String },
}

#[derive(Debug, PartialEq)]
pub enum BuiltinCommand {
    Exit { code: i32 },
    Echo { input: String },
    Type(TypeCommand),
    Pwd,
    Cd { path: String },
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Builtin(BuiltinCommand),
    Unknown { cmd: String, args: Vec<String> },
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let (cmd, args) = parse_input(input);

        let cmd = cmd.trim();
        match cmd {
            "exit" => {
                let code = args
                    .get(0)
                    .ok_or(anyhow!("Invalid arguments"))?
                    .parse::<i32>()?;

                let command = Command::Builtin(BuiltinCommand::Exit { code });
                return Ok(command);
            }
            "echo" => {
                let input = args.join(" ");
                let command = Command::Builtin(BuiltinCommand::Echo { input });
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
                    let command = Command::Builtin(BuiltinCommand::Type(TypeCommand::WellKnown {
                        cmd: cmd.to_string(),
                    }));

                    return Ok(command);
                }

                let command = Command::Builtin(BuiltinCommand::Type(TypeCommand::Unknown {
                    cmd: cmd.to_string(),
                }));
                return Ok(command);
            }
            "pwd" => {
                let command = Command::Builtin(BuiltinCommand::Pwd);
                return Ok(command);
            }
            "cd" => {
                let path = args.get(0).ok_or(anyhow!("Invalid arguments"))?.to_string();
                let command = Command::Builtin(BuiltinCommand::Cd { path });
                return Ok(command);
            }
            _ => {
                let cmd = cmd.to_string();
                let args: Vec<String> = args.iter().map(|v| v.to_string()).collect();

                let command = Command::Unknown { cmd, args };
                return Ok(command);
            }
        }
    }
}

fn parse_input(input: &str) -> (String, Vec<String>) {
    let cmd_parts = input.split_once(' ');
    match cmd_parts {
        None => return (input.to_string(), vec![]),
        Some((cmd, args)) => return (cmd.to_string(), parse_args(args)),
    }
}

#[cfg(test)]
mod command_from_str_tests {
    use std::vec;

    use super::*;

    #[test]
    fn exit_command() {
        let input = "exit 19";

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Builtin(BuiltinCommand::Exit { code: 19 });

        assert_eq!(got_command, expected_command)
    }

    #[test]
    fn built_in_command() {
        let input = "pwd";

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Builtin(BuiltinCommand::Pwd);

        assert_eq!(got_command, expected_command)
    }

    #[test]
    fn echo_command() {
        let input = "echo foo bar baz";

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Builtin(BuiltinCommand::Echo {
            input: "foo bar baz".to_string(),
        });

        assert_eq!(got_command, expected_command)
    }

    #[test]
    fn echo_command_quoted_backslash() {
        let input = r#"echo "before\   after""#;

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Builtin(BuiltinCommand::Echo {
            input: r#"before\   after"#.to_string(),
        });

        assert_eq!(got_command, expected_command);
    }

    #[test]
    fn echo_command_non_quoted_backslash() {
        let input = r#"echo world\ \ \ \ \ \ script"#;

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Builtin(BuiltinCommand::Echo {
            input: r#"world      script"#.to_string(),
        });

        assert_eq!(got_command, expected_command);
    }

    #[test]
    fn echo_command_single_quoted_backslash() {
        let input = r#"echo "/'f \21\'""#;

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Builtin(BuiltinCommand::Echo {
            input: r#"/'f \21\'"#.to_string(),
        });

        assert_eq!(got_command, expected_command);
    }

    #[test]
    fn echo_command_single_quoted_backslash2() {
        let input = r#"echo "/'f  \78'""#;

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Builtin(BuiltinCommand::Echo {
            input: r#""/'f  \78'""#.to_string(),
        });

        assert_eq!(got_command, expected_command);
    }

    #[test]
    fn echo_command_single_quotes() {
        let input = "echo 'fo      bar' baz";

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Builtin(BuiltinCommand::Echo {
            input: "fo      bar baz".to_string(),
        });

        assert_eq!(got_command, expected_command)
    }

    #[test]
    fn echo_command_double_quotes() {
        let input = "echo \"quz              hello\"   \"bar\"";

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Builtin(BuiltinCommand::Echo {
            input: "quz              hello bar".to_string(),
        });

        assert_eq!(got_command, expected_command)
    }

    #[test]
    fn echo_command_double_quotes2() {
        let input = "echo \"quz              hello\" \"bar\"";

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Builtin(BuiltinCommand::Echo {
            input: "quz              hello bar".to_string(),
        });

        assert_eq!(got_command, expected_command)
    }

    #[test]
    fn echo_command_double_and_single_quotes() {
        let input = "echo \"bar\" \"shell's\" \"foo\"";

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Builtin(BuiltinCommand::Echo {
            input: "bar shell's foo".to_string(),
        });

        assert_eq!(got_command, expected_command)
    }

    #[test]
    fn type_well_known() {
        let input = "type echo";

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Builtin(BuiltinCommand::Type(TypeCommand::WellKnown {
            cmd: "echo".to_string(),
        }));

        assert_eq!(got_command, expected_command)
    }

    #[test]
    fn type_unknown_command() {
        let input = "type i_do_not_exist";

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Builtin(BuiltinCommand::Type(TypeCommand::Unknown {
            cmd: "i_do_not_exist".to_string(),
        }));

        assert_eq!(got_command, expected_command)
    }

    #[test]
    fn unknown_command() {
        let input = "unknown_command foo bar baz";

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Unknown {
            cmd: "unknown_command".to_string(),
            args: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
        };

        assert_eq!(got_command, expected_command)
    }
}

impl Command {
    pub fn run(
        self,
        prompter: &mut impl Prompter,
        finder: &impl ExecutablePathFinder,
        runner: &impl ExecutableRunner,
    ) -> anyhow::Result<()> {
        match self {
            Command::Builtin(builtin_command) => {
                return run_builtin_command(builtin_command, prompter, finder);
            }

            Command::Unknown { cmd, args } => {
                return run_unknown_command(prompter, runner, cmd, args)
            }
        }
    }
}

fn run_builtin_command(
    command: BuiltinCommand,
    prompter: &mut impl Prompter,
    finder: &impl ExecutablePathFinder,
) -> anyhow::Result<()> {
    match command {
        BuiltinCommand::Exit { code } => {
            std::process::exit(code);
        }
        BuiltinCommand::Echo { input } => {
            let prompt = format!("{}\n", input);
            prompter.prompt(&prompt)?;
        }
        BuiltinCommand::Type(command) => match command {
            TypeCommand::WellKnown { cmd } => {
                let prompt = format!("{} is a shell builtin\n", cmd);
                prompter.prompt(&prompt)?;
            }
            TypeCommand::Unknown { cmd } => {
                let env_path = std::env::var("PATH")?;
                let result = finder.find_executable_path(&env_path, &cmd);

                match result {
                    Some(full_path) => {
                        let prompt = format!("{} is {}\n", cmd, full_path);
                        prompter.prompt(&prompt)?;
                    }
                    None => {
                        let prompt = format!("{}: not found\n", cmd);
                        prompter.prompt(&prompt)?;
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

            let prompt = format!("{}\n", pwd);
            prompter.prompt(&prompt)?;
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
                        let prompt = format!("cd: {}: No such file or directory\n", path);
                        prompter.prompt(&prompt)?;
                    }
                    _ => return Err(anyhow!("Unknown error")),
                }
            };
        }
    }

    return Ok(());
}

fn run_unknown_command(
    prompter: &mut impl Prompter,
    runner: &impl ExecutableRunner,
    cmd: String,
    args: Vec<String>,
) -> anyhow::Result<()> {
    let args: Vec<&str> = args.iter().map(|arg| arg.as_str()).collect();
    let args = args.as_slice();

    let output = runner.execute(&cmd, args);

    if let Ok(result) = output {
        return prompter.prompt(&result);
    }

    let prompt = format!("{}: command not found\n", cmd);
    return prompter.prompt(&prompt);
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
                    let special_chars = vec!['$', '"', 'n'];
                    let is_special_char =
                        next_char.map_or(false, |c| return special_chars.contains(&c));

                    if is_special_char {
                        continue;
                    }

                    current_arg.push(current_char);
                }
            }
            '"' => {
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
}
