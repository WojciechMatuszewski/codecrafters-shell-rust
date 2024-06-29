use std::path::PathBuf;

pub trait ExecutableRunner {
    fn execute(&self, exec_name: &str, args: &[&str]) -> anyhow::Result<String> {
        let result = std::process::Command::new(exec_name).args(args).output()?;
        let output = String::from_utf8(result.stdout)?;

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

pub struct Runner {}

impl ExecutableRunner for Runner {}
