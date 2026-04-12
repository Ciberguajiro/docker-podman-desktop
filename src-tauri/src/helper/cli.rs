use crate::types::CommandResult;
use std::process::Stdio;
use std::process::Command as StdCommand;
use tokio::process::Command;
use tracing::{debug, error};

/// Validates that the command is either 'docker' or 'podman'
fn is_valid_command(cmd: &str) -> bool {
    cmd == "docker" || cmd == "podman"
}

#[cfg(unix)]
fn is_root() -> bool {
    unsafe { libc::getuid() == 0 }
}

/// Runs a command with elevation if requested.
/// For elevation, it uses the elevated-command crate.
/// For normal execution, it uses tokio::process::Command.
pub async fn run_command_with_elevation_check(
    cmd: &str,
    args: &[String],
    elevated: bool,
) -> CommandResult {
    if !is_valid_command(cmd) {
        return CommandResult {
            success: false,
            output: String::new(),
            error: Some(format!("Invalid command: {}. Only docker and podman are allowed.", cmd)),
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

    // If elevation is requested but we are already root (on Unix),
    // or if elevation is not requested, run normally.
    if !elevated || already_elevated {
        debug!("Running normal command: {} {:?}", cmd, args);
        let output = Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                CommandResult {
                    success: output.status.success(),
                    output: stdout,
                    error: if stderr.is_empty() { None } else { Some(stderr) },
                }
            }
            Err(e) => {
                error!("Command failed: {}", e);
                CommandResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Command failed: {}", e)),
                }
            }
        }
    } else {
        debug!("Running elevated command: {} {:?}", cmd, args);
        // elevated-command is synchronous, so we run it in a spawn_blocking
        let cmd_name = cmd.to_string();
        let cmd_args = args.to_owned();

        let result = tokio::task::spawn_blocking(move || {
            let mut std_cmd = StdCommand::new(cmd_name);
            std_cmd.args(cmd_args);

            let elevated_cmd = elevated_command::Command::new(std_cmd);

            // elevated-command returns std::process::Output
            elevated_cmd.output()
        }).await;

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
                error!("Elevated command failed: {}", e);
                CommandResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Elevated command failed: {}", e)),
                }
            }
            Err(e) => {
                error!("Task joining failed: {}", e);
                CommandResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Internal task error: {}", e)),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_command() {
        let result = run_command_with_elevation_check("ls", &["-la".to_string()], false).await;
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Only docker and podman are allowed"));
    }

    #[test]
    fn test_is_valid_command() {
        assert!(is_valid_command("docker"));
        assert!(is_valid_command("podman"));
        assert!(!is_valid_command("ls"));
        assert!(!is_valid_command("sudo"));
    }
}
