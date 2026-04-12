use crate::types::CommandResult;
use std::process::Stdio;
use std::process::Command as StdCommand;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tracing::{debug, error};

/// Validates that the command is allowed
fn is_valid_command(cmd: &str) -> bool {
    cmd == "docker" || cmd == "podman" || cmd == "podman-compose"
}

#[cfg(unix)]
fn is_root() -> bool {
    unsafe { libc::getuid() == 0 }
}

/// Runs a command with optional elevation and timeout.
/// Centralized CLI execution logic.
pub async fn run_command_with_elevation_check(
    cmd: &str,
    args: &[&str],
    elevated: bool,
    seconds: u64,
) -> CommandResult {
    if !is_valid_command(cmd) {
        return CommandResult {
            success: false,
            output: String::new(),
            error: Some(format!("Invalid command: {}. Only docker, podman, and podman-compose are allowed.", cmd)),
        };
    }

    let already_elevated = if cfg!(unix) {
        #[cfg(unix)]
        { is_root() }
        #[cfg(not(unix))]
        { false }
    } else {
        false
    };

    let cmd_timeout = Duration::from_secs(seconds);

    // If elevation is requested but we are already root (on Unix),
    // or if elevation is not requested, run normally.
    if !elevated || already_elevated {
        debug!("Running normal command (timeout {}s): {} {:?}", seconds, cmd, args);

        let process = Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        let result = timeout(cmd_timeout, process).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                CommandResult {
                    success: output.status.success(),
                    output: stdout,
                    error: if stderr.is_empty() { None } else { Some(stderr) },
                }
            }
            Ok(Err(e)) => {
                error!("Command failed: {}", e);
                CommandResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Command failed: {}", e)),
                }
            }
            Err(_) => CommandResult {
                success: false,
                output: String::new(),
                error: Some(format!("Command timed out after {} seconds", seconds)),
            },
        }
    } else {
        debug!("Running elevated command (timeout {}s): {} {:?}", seconds, cmd, args);
        // elevated-command is synchronous, so we run it in a spawn_blocking
        let cmd_name = cmd.to_string();
        let cmd_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

        let result = timeout(cmd_timeout, tokio::task::spawn_blocking(move || {
            let mut std_cmd = StdCommand::new(cmd_name);
            std_cmd.args(cmd_args);

            let elevated_cmd = elevated_command::Command::new(std_cmd);

            // elevated-command returns std::process::Output
            elevated_cmd.output()
        })).await;

        match result {
            Ok(Ok(Ok(output))) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                CommandResult {
                    success: output.status.success(),
                    output: stdout,
                    error: if stderr.is_empty() { None } else { Some(stderr) },
                }
            }
            Ok(Ok(Err(e))) => {
                error!("Elevated command failed: {}", e);
                CommandResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Elevated command failed: {}", e)),
                }
            }
            Ok(Err(e)) => {
                error!("Task joining failed: {}", e);
                CommandResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Internal task error: {}", e)),
                }
            }
            Err(_) => CommandResult {
                success: false,
                output: String::new(),
                error: Some(format!("Elevated command timed out after {} seconds", seconds)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_command() {
        let result = run_command_with_elevation_check("ls", &["-la"], false, 5).await;
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Only docker, podman, and podman-compose are allowed"));
    }

    #[test]
    fn test_is_valid_command() {
        assert!(is_valid_command("docker"));
        assert!(is_valid_command("podman"));
        assert!(is_valid_command("podman-compose"));
        assert!(!is_valid_command("ls"));
        assert!(!is_valid_command("sudo"));
    }
}
