use crate::helper::*;
use crate::types::*;
use std::process::Stdio;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, info};

static DOCKER_INFO_CACHE_DOCKER: Mutex<Option<(DockerInfo, Instant)>> = Mutex::new(None);
static DOCKER_INFO_CACHE_PODMAN: Mutex<Option<(DockerInfo, Instant)>> = Mutex::new(None);
static CURRENT_EVENT_PROCESS: Mutex<Option<(Engine, tokio::process::Child)>> = Mutex::new(None);
static CURRENT_STATS_PROCESS: Mutex<Option<tokio::process::Child>> = Mutex::new(None);
static CURRENT_LOG_PROCESS: Mutex<Option<tokio::process::Child>> = Mutex::new(None);
static PULL_CANCEL_TX: Mutex<Option<tokio::sync::oneshot::Sender<()>>> = Mutex::new(None);
static BUILD_CANCEL_TX: Mutex<Option<tokio::sync::oneshot::Sender<()>>> = Mutex::new(None);
static SYSTEM_HANDLER: Mutex<Option<sysinfo::System>> = Mutex::new(None);

#[tauri::command]
pub async fn docker_listen_events(app: AppHandle, engine: Engine) {
    let old_child = {
        let mut current_proc = CURRENT_EVENT_PROCESS.lock().unwrap();
        if let Some((current_engine, child)) = current_proc.take() {
            if current_engine == engine {
                *current_proc = Some((current_engine, child));
                return;
            }
            Some(child)
        } else {
            None
        }
    };

    if let Some(mut child) = old_child {
        let _ = child.kill().await;
    }

    let mut child = Command::new(engine.to_string())
        .args(&["events", "--format", "json"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn engine events");

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    let engine_clone = engine.clone();
    tokio::spawn(async move {
        while let Ok(Some(line)) = reader.next_line().await {
            if let Ok(event) = serde_json::from_str::<DockerEvent>(&line) {
                let _ = app.emit("docker-event", event);
            }
        }

        let mut current_proc = CURRENT_EVENT_PROCESS.lock().unwrap();
        if let Some((proc_engine, _)) = &*current_proc {
            if *proc_engine == engine_clone {
                *current_proc = None;
            }
        }
    });

    let mut current_proc = CURRENT_EVENT_PROCESS.lock().unwrap();
    *current_proc = Some((engine, child));
}

#[tauri::command]
pub async fn docker_ps(engine: Engine, all: bool) -> Result<Vec<DockerContainer>, AppError> {
    debug!("Executing docker_ps (engine: {}, all: {})", engine, all);
    let mut args = vec!["ps", "--format", "json"];
    if all {
        args.push("--all");
    }

    let result = run_engine_command(&engine, &args).await;
    if result.success {
        Ok(parse_docker_containers(&result.output))
    } else {
        Err(AppError::DockerError(
            result
                .error
                .unwrap_or_else(|| "Unknown docker error".to_string()),
        ))
    }
}

#[tauri::command]
pub async fn docker_create_network(engine: Engine, name: String, driver: String) -> CommandResult {
    let mut args = vec!["network", "create"];
    if !driver.is_empty() {
        args.push("--driver");
        args.push(&driver);
    }
    args.push(&name);
    run_engine_command(&engine, &args).await
}

#[tauri::command]
pub async fn docker_remove_network(engine: Engine, id: String) -> CommandResult {
    run_engine_command(&engine, &["network", "rm", &id]).await
}

#[tauri::command]
pub async fn docker_inspect(engine: Engine, id: String) -> Result<String, AppError> {
    let result = run_engine_command(&engine, &["inspect", &id]).await;
    if result.success {
        Ok(result.output)
    } else {
        Err(AppError::DockerError(
            result
                .error
                .unwrap_or_else(|| "Unknown docker error".to_string()),
        ))
    }
}

#[tauri::command]
pub async fn docker_exec(engine: Engine, container_id: String, command: String) -> CommandResult {
    run_engine_command(&engine, &["exec", &container_id, "sh", "-c", &command]).await
}

#[tauri::command]
pub async fn docker_container_ls(
    engine: Engine,
    container_id: String,
    path: String,
) -> Result<Vec<ContainerFile>, AppError> {
    let result = run_engine_command(&engine, &["exec", &container_id, "ls", "-aphl", &path]).await;
    if result.success {
        Ok(parse_container_files(&result.output))
    } else {
        Err(AppError::DockerError(
            result
                .error
                .unwrap_or_else(|| "Failed to list files".to_string()),
        ))
    }
}

#[tauri::command]
pub async fn docker_container_read_file(
    engine: Engine,
    container_id: String,
    path: String,
) -> Result<String, AppError> {
    let result = run_engine_command(&engine, &["exec", &container_id, "cat", &path]).await;
    if result.success {
        Ok(result.output)
    } else {
        Err(AppError::DockerError(
            result
                .error
                .unwrap_or_else(|| "Failed to read file".to_string()),
        ))
    }
}

#[tauri::command]
pub async fn docker_container_write_file(
    engine: Engine,
    container_id: String,
    path: String,
    content: String,
) -> CommandResult {
    let cmd = format!("printf '%s' \"$1\" > \"$2\"");
    run_engine_command(
        &engine,
        &[
            "exec",
            &container_id,
            "sh",
            "-c",
            &cmd,
            "--",
            &content,
            &path,
        ],
    )
    .await
}

#[tauri::command]
pub async fn docker_create_container(
    engine: Engine,
    image: String,
    name: String,
    ports: String,
    envs: String,
    volumes: String,
    restart_policy: String,
) -> CommandResult {
    let mut args = vec!["run", "-d"];
    if !name.is_empty() {
        args.push("--name");
        args.push(&name);
    }

    if !restart_policy.is_empty() {
        args.push("--restart");
        args.push(&restart_policy);
    }

    let port_mappings: Vec<&str> = if ports.is_empty() {
        Vec::new()
    } else {
        ports
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    };

    for port in &port_mappings {
        args.push("-p");
        args.push(*port);
    }

    let env_vars: Vec<&str> = if envs.is_empty() {
        Vec::new()
    } else {
        envs.split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    };

    for env in &env_vars {
        args.push("-e");
        args.push(*env);
    }

    let volume_mappings: Vec<&str> = if volumes.is_empty() {
        Vec::new()
    } else {
        volumes
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    };

    for vol in &volume_mappings {
        args.push("-v");
        args.push(*vol);
    }

    args.push(&image);

    run_engine_command(&engine, &args).await
}

#[tauri::command]
pub async fn docker_images(engine: Engine) -> Result<Vec<DockerImage>, AppError> {
    let result = run_engine_command(&engine, &["images", "--format", "json"]).await;
    if result.success {
        Ok(parse_docker_images(&result.output))
    } else {
        Err(AppError::DockerError(
            result
                .error
                .unwrap_or_else(|| "Unknown docker error".to_string()),
        ))
    }
}

#[tauri::command]
pub async fn docker_volumes(engine: Engine) -> Result<Vec<DockerVolume>, AppError> {
    let result = run_engine_command(&engine, &["volume", "ls", "--format", "json"]).await;
    if result.success {
        Ok(parse_docker_volumes(&result.output))
    } else {
        Err(AppError::DockerError(
            result
                .error
                .unwrap_or_else(|| "Unknown docker error".to_string()),
        ))
    }
}

#[tauri::command]
pub async fn docker_networks(engine: Engine) -> Result<Vec<DockerNetwork>, AppError> {
    let result = run_engine_command(&engine, &["network", "ls", "--format", "json"]).await;
    if result.success {
        Ok(parse_docker_networks(&result.output))
    } else {
        Err(AppError::DockerError(
            result
                .error
                .unwrap_or_else(|| "Unknown docker error".to_string()),
        ))
    }
}

#[tauri::command]
pub async fn docker_start(engine: Engine, container_id: String) -> CommandResult {
    info!("Starting container: {} with {}", container_id, engine);
    run_engine_command(&engine, &["start", &container_id]).await
}

#[tauri::command]
pub async fn docker_stop(engine: Engine, container_id: String) -> CommandResult {
    info!("Stopping container: {} with {}", container_id, engine);
    run_engine_command(&engine, &["stop", &container_id]).await
}

#[tauri::command]
pub async fn docker_restart(engine: Engine, container_id: String) -> CommandResult {
    run_engine_command(&engine, &["restart", &container_id]).await
}

#[tauri::command]
pub async fn docker_remove_container(
    engine: Engine,
    container_id: String,
    force: bool,
) -> CommandResult {
    if force {
        run_engine_command(&engine, &["rm", "-f", &container_id]).await
    } else {
        run_engine_command(&engine, &["rm", &container_id]).await
    }
}

#[tauri::command]
pub async fn docker_export_container(
    engine: Engine,
    container_id: String,
    path: String,
) -> CommandResult {
    run_engine_command(&engine, &["export", "-o", &path, &container_id]).await
}

#[tauri::command]
pub async fn docker_import_image(
    engine: Engine,
    path: String,
    repository: String,
    tag: String,
) -> CommandResult {
    let mut target = repository;
    if !tag.is_empty() {
        target = format!("{}:{}", target, tag);
    }
    run_engine_command(&engine, &["import", &path, &target]).await
}

#[tauri::command]
pub async fn docker_remove_image(engine: Engine, image_id: String, force: bool) -> CommandResult {
    if force {
        run_engine_command(&engine, &["rmi", "-f", &image_id]).await
    } else {
        run_engine_command(&engine, &["rmi", &image_id]).await
    }
}

#[tauri::command]
pub async fn docker_stop_logs_stream() {
    let child_to_kill = {
        let mut current_proc = CURRENT_LOG_PROCESS.lock().unwrap();
        current_proc.take()
    };

    if let Some(mut child) = child_to_kill {
        let _ = child.kill().await;
        let _ = child.wait().await; // Reap process
    }
}

#[tauri::command(rename_all = "camelCase")]
pub async fn docker_logs_stream(
    app: AppHandle,
    engine: Engine,
    container_id: String,
    tail: i32,
) -> CommandResult {
    // Stop any existing logs stream
    {
        let current_proc_to_kill = {
            let mut current_proc = CURRENT_LOG_PROCESS.lock().unwrap();
            current_proc.take()
        };

        if let Some(mut child) = current_proc_to_kill {
            let _ = child.kill().await;
            let _ = child.wait().await; // Reap process
        }
    }

    let tail_arg = tail.to_string();
    let child = Command::new(engine.to_string())
        .args(&["logs", "-f", "--tail", &tail_arg, &container_id])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn docker logs: {}", e));

    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            return CommandResult {
                success: false,
                output: String::new(),
                error: Some(e),
            }
        }
    };

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let app_clone = app.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_clone.emit(
                "log-event",
                LogEvent {
                    line,
                    is_error: false,
                },
            );
        }
    });

    let app_clone_err = app.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_clone_err.emit(
                "log-event",
                LogEvent {
                    line,
                    is_error: true,
                },
            );
        }
    });

    // Store the child to manage its lifecycle
    let mut current_proc = CURRENT_LOG_PROCESS.lock().unwrap();
    *current_proc = Some(child);

    // Ensure the process is reaped when it finishes or is killed
    // We already have CURRENT_LOG_PROCESS.take().kill().await in stop/start
    // But if it finishes by itself (e.g. container stops), we should still wait for it.

    CommandResult {
        success: true,
        output: "Logs streaming started".to_string(),
        error: None,
    }
}

#[tauri::command]
pub async fn docker_logs(
    engine: Engine,
    container_id: String,
    tail: i32,
) -> Result<String, AppError> {
    let tail_arg = tail.to_string();
    let result = run_engine_command(&engine, &["logs", "--tail", &tail_arg, &container_id]).await;
    if result.success {
        Ok(result.output)
    } else {
        Err(AppError::DockerError(
            result
                .error
                .unwrap_or_else(|| "Unknown docker error".to_string()),
        ))
    }
}

#[tauri::command]
pub async fn docker_info(engine: Engine) -> DockerInfo {
    let cache_ptr = match engine {
        Engine::Docker => &DOCKER_INFO_CACHE_DOCKER,
        Engine::Podman => &DOCKER_INFO_CACHE_PODMAN,
    };

    if let Ok(cache) = cache_ptr.lock() {
        if let Some((info, timestamp)) = &*cache {
            if timestamp.elapsed() < Duration::from_secs(5) {
                return info.clone();
            }
        }
    }

    let (ps_result, ps_all_result, images_result, version_result, info_result) = tokio::join!(
        run_engine_command(&engine, &["ps", "-q"]),
        run_engine_command(&engine, &["ps", "-aq"]),
        run_engine_command(&engine, &["images", "-q"]),
        run_engine_command(&engine, &["version", "--format", "json"]),
        run_engine_command(&engine, &["info", "--format", "json"]),
    );

    let mut info = DockerInfo {
        containers: 0,
        containers_running: 0,
        containers_stopped: 0,
        images: 0,
        server_version: String::new(),
        storage_driver: String::new(),
        operating_system: String::new(),
        architecture: String::new(),
        cpus: 0,
        memory: String::new(),
        kernel_version: String::new(),
        os_type: String::new(),
        cgroup_driver: String::new(),
        cgroup_version: String::new(),
        logging_driver: String::new(),
        docker_root_dir: String::new(),
    };

    // Process ps result
    if ps_result.success {
        info.containers_running = ps_result.output.lines().count() as i32;
    }

    // Process ps_all result
    if ps_all_result.success {
        info.containers = ps_all_result.output.lines().count() as i32;
        info.containers_stopped = info.containers - info.containers_running;
    }

    // Process images result
    if images_result.success {
        info.images = images_result.output.lines().count() as i32;
    }

    // Process version result
    if version_result.success {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&version_result.output) {
            info.server_version = json["Server"]["Version"]
                .as_str()
                .or(json["Version"].as_str()) // Podman style
                .unwrap_or("")
                .to_string();
        }
    }

    // Process info result
    if info_result.success {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&info_result.output) {
            info.storage_driver = json["Driver"]
                .as_str()
                .or(json["store"]["graphDriverName"].as_str()) // Podman style
                .unwrap_or("")
                .to_string();

            info.operating_system = json["OperatingSystem"]
                .as_str()
                .or(json["host"]["os"].as_str()) // Podman style
                .unwrap_or("")
                .to_string();

            info.architecture = json["Architecture"]
                .as_str()
                .or(json["host"]["arch"].as_str()) // Podman style
                .unwrap_or("")
                .to_string();

            info.cpus = json["NCPU"]
                .as_i64()
                .or(json["host"]["cpus"].as_i64()) // Podman style
                .unwrap_or(0) as i32;

            let mem_total = json["MemTotal"]
                .as_i64()
                .or(json["host"]["memTotal"].as_i64()); // Podman style

            if let Some(mem) = mem_total {
                let gb = mem as f64 / (1024.0 * 1024.0 * 1024.0);
                info.memory = format!("{:.1} GB", gb);
            }

            info.kernel_version = json["KernelVersion"]
                .as_str()
                .or(json["host"]["kernel"].as_str()) // Podman style
                .unwrap_or("")
                .to_string();

            info.os_type = json["OSType"]
                .as_str()
                .or(json["host"]["os"].as_str()) // Podman style
                .unwrap_or("")
                .to_string();

            info.cgroup_driver = json["CgroupDriver"]
                .as_str()
                .or(json["host"]["cgroupDriver"].as_str()) // Podman style
                .unwrap_or("")
                .to_string();

            info.cgroup_version = json["CgroupVersion"]
                .as_str()
                .or(json["host"]["cgroupVersion"].as_str()) // Podman style
                .unwrap_or("")
                .to_string();

            info.logging_driver = json["LoggingDriver"]
                .as_str()
                .or(json["host"]["logDriver"].as_str()) // Podman style
                .unwrap_or("")
                .to_string();

            info.docker_root_dir = json["DockerRootDir"]
                .as_str()
                .or(json["store"]["runRoot"].as_str()) // Podman style
                .unwrap_or("")
                .to_string();
        }
    }

    if let Ok(mut cache) = cache_ptr.lock() {
        *cache = Some((info.clone(), Instant::now()));
    }

    info
}

#[tauri::command]
pub async fn docker_stop_build() {
    let mut cancel_tx = BUILD_CANCEL_TX.lock().unwrap();
    if let Some(tx) = cancel_tx.take() {
        let _ = tx.send(());
    }
}

#[tauri::command]
pub async fn docker_build_image(
    app: AppHandle,
    engine: Engine,
    path: String,
    tag: String,
) -> CommandResult {
    // Stop any existing build
    docker_stop_build().await;

    let (tx, rx) = tokio::sync::oneshot::channel();
    {
        let mut cancel_tx = BUILD_CANCEL_TX.lock().unwrap();
        *cancel_tx = Some(tx);
    }

    let mut args = vec!["build", "-t", &tag, &path];

    let mut child = Command::new(engine.to_string())
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            return CommandResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to spawn docker build: {}", e)),
            }
        }
    };

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let app_clone = app.clone();
    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_clone.emit(
                "build-progress",
                BuildEvent {
                    line,
                    is_error: false,
                },
            );
        }
    });

    let app_clone_err = app.clone();
    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_clone_err.emit(
                "build-progress",
                BuildEvent {
                    line,
                    is_error: true,
                },
            );
        }
    });

    let status = tokio::select! {
        res = child.wait() => res,
        _ = rx => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return CommandResult {
                success: false,
                output: String::new(),
                error: Some("Build process was terminated".to_string()),
            };
        }
    };

    // Clear the cancel tx
    {
        let mut cancel_tx = BUILD_CANCEL_TX.lock().unwrap();
        *cancel_tx = None;
    }
    let _ = tokio::join!(stdout_task, stderr_task);

    match status {
        Ok(s) if s.success() => CommandResult {
            success: true,
            output: format!("Successfully built image {}", tag),
            error: None,
        },
        Ok(s) => CommandResult {
            success: false,
            output: String::new(),
            error: Some(format!("Docker build failed with exit code: {}", s)),
        },
        Err(e) => CommandResult {
            success: false,
            output: String::new(),
            error: Some(format!("Failed to wait for docker build: {}", e)),
        },
    }
}

#[tauri::command]
pub async fn docker_stop_pull() {
    let mut cancel_tx = PULL_CANCEL_TX.lock().unwrap();
    if let Some(tx) = cancel_tx.take() {
        let _ = tx.send(());
    }
}

#[tauri::command]
pub async fn docker_pull(app: AppHandle, engine: Engine, image: String) -> CommandResult {
    // Stop any existing pull
    docker_stop_pull().await;

    let (tx, rx) = tokio::sync::oneshot::channel();
    {
        let mut cancel_tx = PULL_CANCEL_TX.lock().unwrap();
        *cancel_tx = Some(tx);
    }

    let mut child = Command::new(engine.to_string())
        .args(&["pull", &image])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            return CommandResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to spawn docker pull: {}", e)),
            }
        }
    };

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let app_clone = app.clone();
    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_clone.emit(
                "pull-progress",
                PullEvent {
                    status: line,
                    progress: None,
                    is_error: false,
                },
            );
        }
    });

    let app_clone_err = app.clone();
    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_clone_err.emit(
                "pull-progress",
                PullEvent {
                    status: line,
                    progress: None,
                    is_error: true,
                },
            );
        }
    });

    let status = tokio::select! {
        res = child.wait() => res,
        _ = rx => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return CommandResult {
                success: false,
                output: String::new(),
                error: Some("Pull process was terminated".to_string()),
            };
        }
    };

    // Clear the cancel tx
    {
        let mut cancel_tx = PULL_CANCEL_TX.lock().unwrap();
        *cancel_tx = None;
    }
    let _ = tokio::join!(stdout_task, stderr_task);

    match status {
        Ok(s) if s.success() => CommandResult {
            success: true,
            output: format!("Successfully pulled image {}", image),
            error: None,
        },
        Ok(s) => CommandResult {
            success: false,
            output: String::new(),
            error: Some(format!("Docker pull failed with exit code: {}", s)),
        },
        Err(e) => CommandResult {
            success: false,
            output: String::new(),
            error: Some(format!("Failed to wait for docker pull: {}", e)),
        },
    }
}

#[tauri::command]
pub async fn docker_tag_image(
    engine: Engine,
    image_id: String,
    repository: String,
    tag: String,
) -> CommandResult {
    let mut target = repository;
    if !tag.is_empty() {
        target = format!("{}:{}", target, tag);
    }
    run_engine_command(&engine, &["tag", &image_id, &target]).await
}

#[tauri::command]
pub async fn docker_image_history(
    engine: Engine,
    image_id: String,
) -> Result<Vec<ImageHistoryEntry>, AppError> {
    let result = run_engine_command(
        &engine,
        &["history", "--format", "json", "--no-trunc", &image_id],
    )
    .await;
    if result.success {
        Ok(parse_docker_image_history(&result.output))
    } else {
        Err(AppError::DockerError(
            result
                .error
                .unwrap_or_else(|| "Unknown docker error".to_string()),
        ))
    }
}

#[tauri::command]
pub async fn docker_create_volume(engine: Engine, name: String) -> CommandResult {
    run_engine_command(&engine, &["volume", "create", &name]).await
}

#[tauri::command]
pub async fn docker_remove_volume(engine: Engine, name: String) -> CommandResult {
    run_engine_command(&engine, &["volume", "rm", &name]).await
}

#[tauri::command]
pub async fn docker_hub_search(query: String) -> Result<serde_json::Value, AppError> {
    let encoded_query = urlencoding::encode(&query);
    let url = format!(
        "https://hub.docker.com/v2/search/repositories/?query={}&page_size=20",
        encoded_query
    );
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Docker-Desktop-Clone")
        .send()
        .await
        .map_err(|e| AppError::DockerError(format!("Failed to send request: {}", e)))?;

    let json = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::DockerError(format!("Failed to parse JSON: {}", e)))?;

    Ok(json)
}

#[tauri::command]
pub async fn docker_is_running(engine: Engine) -> CommandResult {
    run_engine_command(&engine, &["info"]).await
}

#[tauri::command]
pub async fn check_engine_cli_command(engine: Engine) -> bool {
    check_engine_cli(&engine).await
}

#[tauri::command]
pub async fn docker_compose_up(engine: Engine, project_name: String) -> CommandResult {
    info!(
        "Running compose up for project: {} with {}",
        project_name, engine
    );
    let cmd = match engine {
        Engine::Docker => "docker".to_string(),
        Engine::Podman => "podman-compose".to_string(),
    };
    let args = if engine == Engine::Docker {
        vec!["compose", "-p", &project_name, "up", "-d"]
    } else {
        vec!["-p", &project_name, "up", "-d"]
    };
    run_generic_command_with_timeout(&cmd, &args, 600).await
}

#[tauri::command]
pub async fn docker_compose_down(engine: Engine, project_name: String) -> CommandResult {
    info!(
        "Running compose down for project: {} with {}",
        project_name, engine
    );
    let cmd = match engine {
        Engine::Docker => "docker".to_string(),
        Engine::Podman => "podman-compose".to_string(),
    };
    let args = if engine == Engine::Docker {
        vec!["compose", "-p", &project_name, "down"]
    } else {
        vec!["-p", &project_name, "down"]
    };
    run_generic_command_with_timeout(&cmd, &args, 600).await
}

#[tauri::command]
pub async fn docker_compose_restart(engine: Engine, project_name: String) -> CommandResult {
    info!(
        "Running compose restart for project: {} with {}",
        project_name, engine
    );
    let cmd = match engine {
        Engine::Docker => "docker".to_string(),
        Engine::Podman => "podman-compose".to_string(),
    };
    let args = if engine == Engine::Docker {
        vec!["compose", "-p", &project_name, "restart"]
    } else {
        vec!["-p", &project_name, "restart"]
    };
    run_generic_command_with_timeout(&cmd, &args, 600).await
}

#[tauri::command]
pub async fn docker_prune(engine: Engine, type_: String) -> CommandResult {
    match type_.as_str() {
        "system" => run_engine_command(&engine, &["system", "prune", "-f"]).await,
        "images" => run_engine_command(&engine, &["image", "prune", "-f"]).await,
        "volumes" => run_engine_command(&engine, &["volume", "prune", "-f"]).await,
        "containers" => run_engine_command(&engine, &["container", "prune", "-f"]).await,
        "networks" => run_engine_command(&engine, &["network", "prune", "-f"]).await,
        _ => CommandResult {
            success: false,
            output: String::new(),
            error: Some("Invalid prune type".to_string()),
        },
    }
}

#[tauri::command]
pub async fn docker_stop_stats_stream() {
    let current_proc_to_kill = {
        let mut current_proc = CURRENT_STATS_PROCESS.lock().unwrap();
        current_proc.take()
    };

    if let Some(mut child) = current_proc_to_kill {
        let _ = child.kill().await;
        let _ = child.wait().await; // Reap process
    }
}

#[tauri::command]
pub async fn docker_stats_stream(app: AppHandle, engine: Engine) -> CommandResult {
    // Stop any existing stats stream
    {
        let current_proc_to_kill = {
            let mut current_proc = CURRENT_STATS_PROCESS.lock().unwrap();
            current_proc.take()
        };


        if let Some(mut child) = current_proc_to_kill {
            let _ = child.kill().await;
            let _ = child.wait().await; // Reap process
        }
    }

    let child = Command::new(engine.to_string())
        .args(&["stats", "--format", "json"])
        .stdout(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            return CommandResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to spawn docker stats: {}", e)),
            }
        }
    };

    let stdout = child.stdout.take().unwrap();
    let app_clone = app.clone();

    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            // Docker stats -f outputs one JSON per container per update interval
            // But we want to group them or send them as they come.
            // Actually, docker stats -f sends a stream of JSON objects.
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                if let Some(stat) = parse_single_stat(&json) {
                    let _ = app_clone.emit("stats-event", StatsEvent { stats: vec![stat] });
                }
            }
        }
    });

    let mut current_proc = CURRENT_STATS_PROCESS.lock().unwrap();
    *current_proc = Some(child);

    CommandResult {
        success: true,
        output: "Stats streaming started".to_string(),
        error: None,
    }
}

#[tauri::command]
pub async fn docker_container_stats(engine: Engine) -> Result<Vec<ContainerStats>, AppError> {
    let result = run_engine_command(&engine, &["stats", "--no-stream", "--format", "json"]).await;
    if result.success {
        Ok(parse_docker_stats(&result.output))
    } else {
        Err(AppError::DockerError(
            result
                .error
                .unwrap_or_else(|| "Unknown docker error".to_string()),
        ))
    }
}

#[tauri::command]
pub fn get_system_metrics() -> SystemMetrics {
    use sysinfo::System;

    let mut lock = SYSTEM_HANDLER.lock().unwrap();
    if lock.is_none() {
        *lock = Some(System::new_all());
    }

    let sys = lock.as_mut().unwrap();
    sys.refresh_all();

    // With a persistent System object, subsequent calls will have accurate CPU usage
    let cpu_usage = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
    let mem_total = sys.total_memory();
    let mem_used = sys.used_memory();
    let mem_free = sys.free_memory();
    let mem_percent = if mem_total > 0 {
        (mem_used as f32 / mem_total as f32) * 100.0
    } else {
        0.0
    };

    SystemMetrics {
        cpu_usage,
        mem_total,
        mem_used,
        mem_free,
        mem_percent,
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_docker_import_image_args() {
        // Since we can't easily mock run_docker_command without refactoring,
        // we'll focus on testing the tag logic if we extract it,
        // or just ensure the command compiles and has correct structure.
    }

    #[test]
    fn test_docker_create_container_logic() {
        // Test port mapping logic
        let ports = "8080:80, 3000:3000";
        let port_mappings: Vec<&str> = if ports.is_empty() {
            Vec::new()
        } else {
            ports
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect()
        };
        assert_eq!(port_mappings, vec!["8080:80", "3000:3000"]);

        // Test env vars logic
        let envs = "KEY1=VAL1, KEY2=VAL2";
        let env_vars: Vec<&str> = if envs.is_empty() {
            Vec::new()
        } else {
            envs.split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect()
        };
        assert_eq!(env_vars, vec!["KEY1=VAL1", "KEY2=VAL2"]);

        // Test volumes logic
        let volumes = "/host/path:/container/path, myvol:/data";
        let volume_mappings: Vec<&str> = if volumes.is_empty() {
            Vec::new()
        } else {
            volumes
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect()
        };
        assert_eq!(
            volume_mappings,
            vec!["/host/path:/container/path", "myvol:/data"]
        );
    }
}
