use crate::types::CommandResult;
use serde::Serialize;
use std::process::Stdio;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::oneshot;

/// Stores a cancel sender in a global Mutex, replacing any existing one.
pub fn store_cancel_token(
    cancel_lock: &Mutex<Option<oneshot::Sender<()>>>,
    tx: oneshot::Sender<()>,
) {
    let mut guard = cancel_lock.lock().expect("cancel token lock poisoned");
    *guard = Some(tx);
}

/// Clears the cancel token from the global Mutex.
pub fn clear_cancel_token(cancel_lock: &Mutex<Option<oneshot::Sender<()>>>) {
    let mut guard = cancel_lock.lock().expect("cancel token lock poisoned");
    *guard = None;
}

/// Sends the cancel signal through the token, if present.
pub fn fire_cancel_token(cancel_lock: &Mutex<Option<oneshot::Sender<()>>>) {
    let mut guard = cancel_lock.lock().expect("cancel token lock poisoned");
    if let Some(tx) = guard.take() {
        let _ = tx.send(());
    }
}

/// Spawns a process with stdout+stderr piped, runs two line-reading tokio
/// tasks that emit events, and waits for the process to complete OR the cancel
/// receiver to fire. Returns a CommandResult.
///
/// # Parameters
/// - `engine`: "docker" or "podman"
/// - `args`: CLI arguments
/// - `event_name`: Tauri event name to emit
/// - `make_event`: closure that creates an event struct from (line, is_error)
/// - `cancel_rx`: oneshot receiver for cancellation
/// - `success_msg`, `fail_prefix`, `terminated_msg`: messages for the result
pub async fn run_cancellable_streaming<E: Clone + Send + 'static + Serialize>(
    app: AppHandle,
    engine: &str,
    args: &[&str],
    event_name: &'static str,
    make_event: impl Fn(String, bool) -> E + Send + Clone + 'static,
    cancel_rx: oneshot::Receiver<()>,
    success_msg: String,
    fail_prefix: &str,
    terminated_msg: &str,
) -> CommandResult {
    let child = Command::new(engine)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            return CommandResult {
                success: false,
                output: String::new(),
                error: Some(format!("{}: {}", fail_prefix, e)),
            };
        }
    };

    let stdout = child.stdout.take().expect("stdout pipe should be available");
    let stderr = child.stderr.take().expect("stderr pipe should be available");

    let ev = make_event.clone();
    let app_out = app.clone();
    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_out.emit(event_name, ev(line, false));
        }
    });

    let ev_err = make_event;
    let app_err = app.clone();
    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_err.emit(event_name, ev_err(line, true));
        }
    });

    let status = tokio::select! {
        res = child.wait() => res,
        _ = cancel_rx => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return CommandResult {
                success: false,
                output: String::new(),
                error: Some(terminated_msg.to_string()),
            };
        }
    };

    let _ = tokio::join!(stdout_task, stderr_task);

    match status {
        Ok(s) if s.success() => CommandResult {
            success: true,
            output: success_msg,
            error: None,
        },
        Ok(s) => CommandResult {
            success: false,
            output: String::new(),
            error: Some(format!("{}: exit code {}", fail_prefix, s)),
        },
        Err(e) => CommandResult {
            success: false,
            output: String::new(),
            error: Some(format!("{}: {}", fail_prefix, e)),
        },
    }
}
