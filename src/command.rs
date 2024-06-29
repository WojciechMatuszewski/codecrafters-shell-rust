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

pub enum Command {
    Builtin(BuiltinCommand),
    Unknown { cmd: String, args: Vec<String> },
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed: Vec<&str> = s.split_whitespace().collect();

        let [cmd, args @ ..] = parsed.as_slice() else {
            return Err(anyhow!("Failed to parse the command"));
        };

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
                let built_ins = vec!["exit", "echo", "type", "pwd"];

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
