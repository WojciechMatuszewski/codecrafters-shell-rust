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
        Some((cmd, args)) => return (cmd.to_string(), parse_input_args(args)),
    }
}

fn parse_input_args(input_args: &str) -> Vec<String> {
    let iter = input_args.chars().enumerate();

    let mut inside_quotes = false;
    let mut current_arg = String::from("");
    let mut retrieved_args: Vec<String> = vec![];

    for (index, args_char) in iter {
        match args_char {
            '\'' => {
                if inside_quotes {
                    inside_quotes = false;
                    continue;
                }

                let has_matching_quote = input_args
                    .get(index + 1..)
                    .and_then(|next_chars| return next_chars.find('\''))
                    .is_some();

                if has_matching_quote {
                    inside_quotes = true;
                } else {
                    current_arg.push(args_char);
                }
            }
            ' ' if !inside_quotes && !current_arg.is_empty() => {
                retrieved_args.push(sanitize_arg(&current_arg));
                current_arg.clear();
            }
            _ => {
                if inside_quotes || !args_char.is_whitespace() {
                    current_arg.push(args_char);
                }
            }
        }
    }
    if !current_arg.is_empty() {
        retrieved_args.push(sanitize_arg(&current_arg));
        current_arg.clear();
    }

    return retrieved_args;
}

fn sanitize_arg(arg: &str) -> String {
    return arg.replace("\"", "");
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
        let input = "echo \"quz hello\" \"bar\"";

        let got_command = input.parse::<Command>().unwrap();
        let expected_command = Command::Builtin(BuiltinCommand::Echo {
            input: "quz hello bar".to_string(),
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
