use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutableError {
    #[error("{0}: command not found\n")]
    CommandNotFound(String),

    #[error("{0}")]
    CommandFailed(String),
}

pub trait ExecutableRunner {
    fn execute(&self, exec_name: &str, args: &[&str]) -> anyhow::Result<String, ExecutableError> {
        let result = std::process::Command::new(exec_name)
            .args(args)
            .output()
            .map_err(|_| return ExecutableError::CommandNotFound(exec_name.to_string()))?;

        if !result.status.success() {
            let error = String::from_utf8_lossy(&result.stderr).to_string();
            return Err(ExecutableError::CommandFailed(error));
        }

        let output = String::from_utf8(result.stdout)
            .map_err(|error| return ExecutableError::CommandFailed(error.to_string()))?;
        return Ok(output);
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
