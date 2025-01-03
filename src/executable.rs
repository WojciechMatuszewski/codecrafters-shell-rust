use std::path::PathBuf;

#[derive(Debug)]
pub struct ExecutableOutput {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

pub trait ExecutableRunner {
    fn execute(&self, exec_name: &str, args: &[&str]) -> anyhow::Result<ExecutableOutput> {
        let result = std::process::Command::new(exec_name).args(args).output();

        match result {
            Ok(result) => {
                let mut output = ExecutableOutput {
                    stdout: None,
                    stderr: None,
                };

                let stderr = String::from_utf8_lossy(&result.stderr).to_string();
                if !stderr.is_empty() {
                    output.stderr = Some(stderr)
                }

                let stdout = String::from_utf8_lossy(&result.stdout).to_string();
                if !stdout.is_empty() {
                    output.stdout = Some(stdout)
                }

                return Ok(output);
            }
            Err(_) => {
                return Ok(ExecutableOutput {
                    stderr: Some(format!("{}: command not found\n", exec_name)),
                    stdout: None,
                })
            }
        }
    }
}

pub trait ExecutablePathFinder {
    fn find_executable_path(&self, env_path: &str, name: &str) -> Option<String> {
        let env_paths = env_path.split(":");

        for env_path in env_paths {
            let full_path: PathBuf = [env_path, name].iter().collect();
            if full_path.exists() {
                return Some(
                    full_path
                        .into_os_string()
                        .into_string()
                        .expect("Failed to convert path"),
                );
            }
        }

        return None;
    }
}

pub struct PathFinder {}

impl ExecutablePathFinder for PathFinder {}

impl PathFinder {
    pub fn new() -> Self {
        return Self {};
    }
}

pub struct Runner {}

impl ExecutableRunner for Runner {}

impl Runner {
    pub fn new() -> Self {
        return Self {};
    }
}
