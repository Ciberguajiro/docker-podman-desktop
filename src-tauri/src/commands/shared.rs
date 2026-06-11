use crate::types::AppError;
use crate::helper::*;
use tokio::io::BufReader;
use tauri::AppHandle;
use std::sync::Mutex;

/// Runs an engine command and applies a parsing function to the result.
/// Eliminates the repeated "run command → if success parse else error" pattern.
pub async fn run_and_parse<T>(
    engine: &crate::types::Engine,
    args: &[&str],
    parser: fn(&str) -> Vec<T>,
) -> Result<Vec<T>, AppError> {
    let result = run_engine_command(engine, args).await;
    if result.success {
        Ok(parser(&result.output))
    } else {
        Err(AppError::DockerError(
            result
                .error
                .unwrap_or_else(|| "Unknown docker error".to_string()),
        ))
    }
}

/// Runs an engine command and returns the raw output on success.
pub async fn run_and_get_output(
    engine: &crate::types::Engine,
    args: &[&str],
    error_msg: &str,
) -> Result<String, AppError> {
    let result = run_engine_command(engine, args).await;
    if result.success {
        Ok(result.output)
    } else {
        Err(AppError::DockerError(
            result
                .error
                .unwrap_or_else(|| error_msg.to_string()),
        ))
    }
}

/// Parses a comma-separated string into a Vec of trimmed &str slices.
/// Used by docker_create_container for ports, env vars, and volumes.
pub fn parse_csv_list(input: &str) -> Vec<&str> {
    if input.is_empty() {
        Vec::new()
    } else {
        input
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Kills and reaps a child process if present, taking it from a Mutex<Option<Child>>.
pub async fn kill_and_reap(process_lock: &Mutex<Option<tokio::process::Child>>) {
    let child_to_kill = {
        let mut guard = process_lock.lock().expect("process lock poisoned");
        guard.take()
    };

    if let Some(mut child) = child_to_kill {
        let _ = child.kill().await;
        let _ = child.wait().await;
    }
}

/// Spawns a tokio task that reads lines from a BufReader<T> (stdout or stderr)
/// and emits them as events to the Tauri frontend via the provided closure.
#[allow(dead_code)]
pub fn spawn_line_emitter<F>(
    _app: AppHandle,
    reader: tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    emit_fn: F,
) -> tokio::task::JoinHandle<()>
where
    F: Fn(String) + Send + 'static,
{
    tokio::spawn(async move {
        let mut reader = reader;
        while let Ok(Some(line)) = reader.next_line().await {
            emit_fn(line);
        }
    })
}
