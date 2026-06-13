mod shared;
mod streaming;

use crate::helper::*;
use crate::types::*;
use shared::*;
use streaming::*;
use tauri::Manager;
use std::process::Stdio;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, info, error};

static DOCKER_INFO_CACHE_DOCKER: Mutex<Option<(DockerInfo, Instant)>> = Mutex::new(None);
static DOCKER_INFO_CACHE_PODMAN: Mutex<Option<(DockerInfo, Instant)>> = Mutex::new(None);
static CURRENT_EVENT_PROCESS: Mutex<Option<(Engine, tokio::process::Child)>> = Mutex::new(None);
static CURRENT_STATS_PROCESS: Mutex<Option<tokio::process::Child>> = Mutex::new(None);
static CURRENT_LOG_PROCESS: Mutex<Option<tokio::process::Child>> = Mutex::new(None);
static PULL_CANCEL_TX: Mutex<Option<tokio::sync::oneshot::Sender<()>>> = Mutex::new(None);
static BUILD_CANCEL_TX: Mutex<Option<tokio::sync::oneshot::Sender<()>>> = Mutex::new(None);
static SYSTEM_HANDLER: Mutex<Option<sysinfo::System>> = Mutex::new(None);

// ─── Event listening ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn docker_listen_events(app: AppHandle, engine: Engine) {
    let old_child = {
        let mut current_proc = CURRENT_EVENT_PROCESS.lock().expect("CURRENT_EVENT_PROCESS lock poisoned");
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

    let child = Command::new(engine.to_string())
        .args(&["events", "--format", "json"])
        .stdout(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to spawn engine events for {}: {}", engine, e);
            let mut current_proc = CURRENT_EVENT_PROCESS.lock().expect("CURRENT_EVENT_PROCESS lock poisoned");
            *current_proc = None;
            return;
        }
    };

    let stdout = child.stdout.take().expect("stdout should be available after spawn");
    let mut reader = BufReader::new(stdout).lines();
    let engine_clone = engine.clone();

    tokio::spawn(async move {
        while let Ok(Some(line)) = reader.next_line().await {
            if let Ok(event) = serde_json::from_str::<DockerEvent>(&line) {
                let _ = app.emit("docker-event", event);
            }
        }

        let mut current_proc = CURRENT_EVENT_PROCESS.lock().expect("CURRENT_EVENT_PROCESS lock poisoned");
        if let Some((proc_engine, _)) = &*current_proc {
            if *proc_engine == engine_clone {
                *current_proc = None;
            }
        }
    });

    let mut current_proc = CURRENT_EVENT_PROCESS.lock().expect("CURRENT_EVENT_PROCESS lock poisoned");
    *current_proc = Some((engine, child));
}

// ─── Containers ──────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn docker_ps(engine: Engine, all: bool) -> Result<Vec<DockerContainer>, AppError> {
    debug!("Executing docker_ps (engine: {}, all: {})", engine, all);
    let mut args = vec!["ps", "--format", "json"];
    if all { args.push("--all"); }
    run_and_parse(&engine, &args, parse_docker_containers).await
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
pub async fn docker_remove_container(engine: Engine, container_id: String, force: bool) -> CommandResult {
    if force {
        run_engine_command(&engine, &["rm", "-f", &container_id]).await
    } else {
        run_engine_command(&engine, &["rm", &container_id]).await
    }
}

#[tauri::command]
pub async fn docker_export_container(engine: Engine, container_id: String, path: String) -> CommandResult {
    run_engine_command(&engine, &["export", "-o", &path, &container_id]).await
}

#[tauri::command]
pub async fn docker_create_container(
    engine: Engine, image: String, name: String, ports: String,
    envs: String, volumes: String, restart_policy: String,
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

    for mapping in parse_csv_list(&ports) {
        args.push("-p");
        args.push(mapping);
    }

    for env in parse_csv_list(&envs) {
        args.push("-e");
        args.push(env);
    }

    for vol in parse_csv_list(&volumes) {
        args.push("-v");
        args.push(vol);
    }

    args.push(&image);
    run_engine_command(&engine, &args).await
}

#[tauri::command]
pub async fn docker_inspect(engine: Engine, id: String) -> Result<String, AppError> {
    run_and_get_output(&engine, &["inspect", &id], "Failed to inspect").await
}

#[tauri::command]
pub async fn docker_exec(engine: Engine, container_id: String, command: String) -> CommandResult {
    run_engine_command(&engine, &["exec", &container_id, "sh", "-c", &command]).await
}

#[tauri::command]
pub async fn docker_container_ls(
    engine: Engine, container_id: String, path: String,
) -> Result<Vec<ContainerFile>, AppError> {
    run_and_parse(&engine, &["exec", &container_id, "ls", "-aphl", &path], parse_container_files).await
}

#[tauri::command]
pub async fn docker_container_read_file(
    engine: Engine, container_id: String, path: String,
) -> Result<String, AppError> {
    run_and_get_output(&engine, &["exec", &container_id, "cat", &path], "Failed to read file").await
}

#[tauri::command]
pub async fn docker_container_write_file(
    engine: Engine, container_id: String, path: String, content: String,
) -> CommandResult {
    let cmd = "printf '%s' \"$1\" > \"$2\"";
    run_engine_command(
        &engine,
        &["exec", &container_id, "sh", "-c", cmd, "--", &content, &path],
    )
    .await
}

#[tauri::command]
pub async fn docker_stop_logs_stream() {
    kill_and_reap(&CURRENT_LOG_PROCESS).await;
}

#[tauri::command(rename_all = "camelCase")]
pub async fn docker_logs_stream(
    app: AppHandle, engine: Engine, container_id: String, tail: i32,
) -> CommandResult {
    kill_and_reap(&CURRENT_LOG_PROCESS).await;

    let tail_arg = tail.to_string();
    let child = Command::new(engine.to_string())
        .args(&["logs", "-f", "--tail", &tail_arg, &container_id])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn docker logs: {}", e));

    let mut child = match child {
        Ok(c) => c,
        Err(e) => return command_error(e),
    };

    let stdout = child.stdout.take().expect("stdout pipe should be available");
    let stderr = child.stderr.take().expect("stderr pipe should be available");

    let app_out = app.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_out.emit("log-event", LogEvent { line, is_error: false });
        }
    });

    let app_err = app.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_err.emit("log-event", LogEvent { line, is_error: true });
        }
    });

    let mut guard = CURRENT_LOG_PROCESS.lock().expect("CURRENT_LOG_PROCESS lock poisoned");
    *guard = Some(child);

    CommandResult { success: true, output: "Logs streaming started".to_string(), error: None }
}

#[tauri::command]
pub async fn docker_logs(engine: Engine, container_id: String, tail: i32) -> Result<String, AppError> {
    let tail_arg = tail.to_string();
    run_and_get_output(&engine, &["logs", "--tail", &tail_arg, &container_id], "Failed to get logs").await
}

// ─── Images ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn docker_images(engine: Engine) -> Result<Vec<DockerImage>, AppError> {
    run_and_parse(&engine, &["images", "--format", "json"], parse_docker_images).await
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
pub async fn docker_tag_image(engine: Engine, image_id: String, repository: String, tag: String) -> CommandResult {
    let target = if tag.is_empty() { repository } else { format!("{}:{}", repository, tag) };
    run_engine_command(&engine, &["tag", &image_id, &target]).await
}

#[tauri::command]
pub async fn docker_image_history(engine: Engine, image_id: String) -> Result<Vec<ImageHistoryEntry>, AppError> {
    run_and_parse(
        &engine,
        &["history", "--format", "json", "--no-trunc", &image_id],
        parse_docker_image_history,
    )
    .await
}

#[tauri::command]
pub async fn docker_import_image(engine: Engine, path: String, repository: String, tag: String) -> CommandResult {
    let target = if tag.is_empty() { repository } else { format!("{}:{}", repository, tag) };
    run_engine_command(&engine, &["import", &path, &target]).await
}

#[tauri::command]
pub async fn docker_stop_pull() {
    fire_cancel_token(&PULL_CANCEL_TX);
}

#[tauri::command]
pub async fn docker_pull(app: AppHandle, engine: Engine, image: String) -> CommandResult {
    docker_stop_pull().await;

    let (tx, rx) = tokio::sync::oneshot::channel();
    store_cancel_token(&PULL_CANCEL_TX, tx);

    let clamp = image.clone();
    let result = run_cancellable_streaming(
        app, &engine.to_string(), &["pull", &image],
        "pull-progress",
        move |line, is_error| PullEvent { status: line, progress: None, is_error },
        rx,
        format!("Successfully pulled image {}", clamp),
        "Docker pull failed",
        "Pull process was terminated",
    )
    .await;

    clear_cancel_token(&PULL_CANCEL_TX);
    result
}

#[tauri::command]
pub async fn docker_stop_build() {
    fire_cancel_token(&BUILD_CANCEL_TX);
}

#[tauri::command]
pub async fn docker_build_image(app: AppHandle, engine: Engine, path: String, tag: String) -> CommandResult {
    docker_stop_build().await;

    let (tx, rx) = tokio::sync::oneshot::channel();
    store_cancel_token(&BUILD_CANCEL_TX, tx);

    let clone_tag = tag.clone();
    let result = run_cancellable_streaming(
        app, &engine.to_string(), &["build", "-t", &tag, &path],
        "build-progress",
        move |line, is_error| BuildEvent { line, is_error },
        rx,
        format!("Successfully built image {}", clone_tag),
        "Docker build failed",
        "Build process was terminated",
    )
    .await;

    clear_cancel_token(&BUILD_CANCEL_TX);
    result
}

// ─── Resources ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn docker_volumes(engine: Engine) -> Result<Vec<DockerVolume>, AppError> {
    run_and_parse(&engine, &["volume", "ls", "--format", "json"], parse_docker_volumes).await
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
pub async fn docker_networks(engine: Engine) -> Result<Vec<DockerNetwork>, AppError> {
    run_and_parse(&engine, &["network", "ls", "--format", "json"], parse_docker_networks).await
}

#[tauri::command]
pub async fn docker_create_network(engine: Engine, name: String, driver: String) -> CommandResult {
    let mut args = vec!["network", "create"];
    if !driver.is_empty() { args.push("--driver"); args.push(&driver); }
    args.push(&name);
    run_engine_command(&engine, &args).await
}

#[tauri::command]
pub async fn docker_remove_network(engine: Engine, id: String) -> CommandResult {
    run_engine_command(&engine, &["network", "rm", &id]).await
}

#[tauri::command]
pub async fn docker_prune(engine: Engine, type_: String) -> CommandResult {
    match type_.as_str() {
        "system"     => run_engine_command(&engine, &["system", "prune", "-f"]).await,
        "images"     => run_engine_command(&engine, &["image", "prune", "-f"]).await,
        "volumes"    => run_engine_command(&engine, &["volume", "prune", "-f"]).await,
        "containers" => run_engine_command(&engine, &["container", "prune", "-f"]).await,
        "networks"   => run_engine_command(&engine, &["network", "prune", "-f"]).await,
        _ => command_error("Invalid prune type".to_string()),
    }
}

// ─── Compose ─────────────────────────────────────────────────────────────────

fn compose_command<'a>(engine: Engine, project_name: &'a str, action: &'a str, daemon: bool) -> (String, Vec<&'a str>) {
    let cmd = match engine {
        Engine::Docker => "docker".to_string(),
        Engine::Podman => "podman-compose".to_string(),
    };
    let mut args = if engine == Engine::Docker {
        vec!["compose", "-p", project_name, action]
    } else {
        vec!["-p", project_name, action]
    };
    if daemon {
        args.push("-d");
    }
    (cmd, args)
}

#[tauri::command]
pub async fn docker_compose_up(engine: Engine, project_name: String) -> CommandResult {
    info!("Running compose up for project: {} with {}", project_name, engine);
    let (cmd, args) = compose_command(engine, &project_name, "up", true);
    run_generic_command_with_timeout(&cmd, &args, 600).await
}

#[tauri::command]
pub async fn docker_compose_down(engine: Engine, project_name: String) -> CommandResult {
    info!("Running compose down for project: {} with {}", project_name, engine);
    let (cmd, args) = compose_command(engine, &project_name, "down", false);
    run_generic_command_with_timeout(&cmd, &args, 600).await
}

#[tauri::command]
pub async fn docker_compose_restart(engine: Engine, project_name: String) -> CommandResult {
    info!("Running compose restart for project: {} with {}", project_name, engine);
    let (cmd, args) = compose_command(engine, &project_name, "restart", false);
    run_generic_command_with_timeout(&cmd, &args, 600).await
}

// ─── System & Info ───────────────────────────────────────────────────────────

fn info_from_cache(cache_ptr: &Mutex<Option<(DockerInfo, Instant)>>) -> Option<DockerInfo> {
    cache_ptr.lock().ok().and_then(|cache| {
        cache.as_ref().and_then(|(info, ts)| {
            if ts.elapsed() < Duration::from_secs(5) {
                Some(info.clone())
            } else {
                None
            }
        })
    })
}

fn store_info_cache(cache_ptr: &Mutex<Option<(DockerInfo, Instant)>>, info: &DockerInfo) {
    if let Ok(mut cache) = cache_ptr.lock() {
        *cache = Some((info.clone(), Instant::now()));
    }
}

fn extract_count(result: &CommandResult) -> i32 {
    if result.success { result.output.lines().count() as i32 } else { 0 }
}

fn parse_server_version(output: &str) -> String {
    serde_json::from_str::<serde_json::Value>(output)
        .ok()
        .and_then(|json| {
            json["Server"]["Version"]
                .as_str()
                .or(json["Version"].as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default()
}

fn parse_engine_info(info: &mut DockerInfo, output: &str) {
    let json = match serde_json::from_str::<serde_json::Value>(output) {
        Ok(j) => j,
        Err(_) => return,
    };

    info.storage_driver = json_str(&json, &["Driver", "store.graphDriverName"]);
    info.operating_system = json_str(&json, &["OperatingSystem", "host.os"]);
    info.architecture = json_str(&json, &["Architecture", "host.arch"]);
    info.cpus = json_i64(&json, &["NCPU", "host.cpus"]).unwrap_or(0) as i32;

    if let Some(mem) = json_i64(&json, &["MemTotal", "host.memTotal"]) {
        let gb = mem as f64 / (1024.0 * 1024.0 * 1024.0);
        info.memory = format!("{:.1} GB", gb);
    }

    info.kernel_version = json_str(&json, &["KernelVersion", "host.kernel"]);
    info.os_type = json_str(&json, &["OSType", "host.os"]);
    info.cgroup_driver = json_str(&json, &["CgroupDriver", "host.cgroupDriver"]);
    info.cgroup_version = json_str(&json, &["CgroupVersion", "host.cgroupVersion"]);
    info.logging_driver = json_str(&json, &["LoggingDriver", "host.logDriver"]);
    info.docker_root_dir = json_str(&json, &["DockerRootDir", "store.runRoot"]);
}

fn json_str(json: &serde_json::Value, paths: &[&str]) -> String {
    for path in paths {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = json;
        for part in &parts {
            current = match current.get(part) {
                Some(v) => v,
                None => break,
            };
        }
        if let Some(s) = current.as_str() {
            return s.to_string();
        }
    }
    String::new()
}

fn json_i64(json: &serde_json::Value, paths: &[&str]) -> Option<i64> {
    for path in paths {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = json;
        for part in &parts {
            current = current.get(part)?;
        }
        if let Some(n) = current.as_i64() {
            return Some(n);
        }
    }
    None
}

#[tauri::command]
pub async fn docker_info(engine: Engine) -> DockerInfo {
    let cache_ptr = match engine {
        Engine::Docker => &DOCKER_INFO_CACHE_DOCKER,
        Engine::Podman => &DOCKER_INFO_CACHE_PODMAN,
    };

    if let Some(cached) = info_from_cache(cache_ptr) {
        return cached;
    }

    let (ps_result, ps_all_result, images_result, version_result, info_result) = tokio::join!(
        run_engine_command(&engine, &["ps", "-q"]),
        run_engine_command(&engine, &["ps", "-aq"]),
        run_engine_command(&engine, &["images", "-q"]),
        run_engine_command(&engine, &["version", "--format", "json"]),
        run_engine_command(&engine, &["info", "--format", "json"]),
    );

    let mut info = DockerInfo::default();

    info.containers_running = extract_count(&ps_result);
    info.containers = extract_count(&ps_all_result);
    info.containers_stopped = info.containers - info.containers_running;
    info.images = extract_count(&images_result);

    if version_result.success {
        info.server_version = parse_server_version(&version_result.output);
    }

    if info_result.success {
        parse_engine_info(&mut info, &info_result.output);
    }

    store_info_cache(cache_ptr, &info);
    info
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
pub async fn docker_hub_search(query: String) -> Result<serde_json::Value, AppError> {
    let url = format!(
        "https://hub.docker.com/v2/search/repositories/?query={}&page_size=20",
        urlencoding::encode(&query)
    );
    let response = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "Docker-Desktop-Clone")
        .send()
        .await
        .map_err(|e| AppError::DockerError(format!("Failed to send request: {}", e)))?;

    response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::DockerError(format!("Failed to parse JSON: {}", e)))
}

// ─── Metrics ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn docker_stop_stats_stream() {
    kill_and_reap(&CURRENT_STATS_PROCESS).await;
}

#[tauri::command]
pub async fn docker_stats_stream(app: AppHandle, engine: Engine) -> CommandResult {
    kill_and_reap(&CURRENT_STATS_PROCESS).await;

    let child = Command::new(engine.to_string())
        .args(&["stats", "--format", "json"])
        .stdout(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => return command_error(format!("Failed to spawn docker stats: {}", e)),
    };

    let stdout = child.stdout.take().expect("stdout pipe should be available");
    let app_clone = app.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                if let Some(stat) = parse_single_stat(&json) {
                    let _ = app_clone.emit("stats-event", StatsEvent { stats: vec![stat] });
                }
            }
        }
    });

    let mut guard = CURRENT_STATS_PROCESS.lock().expect("CURRENT_STATS_PROCESS lock poisoned");
    *guard = Some(child);

    CommandResult { success: true, output: "Stats streaming started".to_string(), error: None }
}

#[tauri::command]
pub async fn docker_container_stats(engine: Engine) -> Result<Vec<ContainerStats>, AppError> {
    run_and_parse(&engine, &["stats", "--no-stream", "--format", "json"], parse_docker_stats).await
}

#[tauri::command]
pub async fn run_elevated_command(command: String, args: Vec<String>, elevated: bool) -> CommandResult {
    crate::helper::cli::run_command_with_elevation_check(
        &command,
        &args.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
        elevated, 60,
    )
    .await
}

#[tauri::command]
pub fn get_system_metrics() -> SystemMetrics {
    use sysinfo::System;

    let mut lock = SYSTEM_HANDLER.lock().expect("SYSTEM_HANDLER lock poisoned");
    if lock.is_none() {
        *lock = Some(System::new_all());
    }

    let sys = lock.as_mut().expect("SYSTEM_HANDLER should be initialized");
    sys.refresh_all();

    let cpu_usage = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
    let mem_total = sys.total_memory();
    let mem_used = sys.used_memory();
    let mem_percent = if mem_total > 0 {
        (mem_used as f32 / mem_total as f32) * 100.0
    } else {
        0.0
    };

    SystemMetrics { cpu_usage, mem_total, mem_used, mem_free: sys.free_memory(), mem_percent }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn command_error(msg: String) -> CommandResult {
    CommandResult { success: false, output: String::new(), error: Some(msg) }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::shared::parse_csv_list;

    #[tokio::test]
    async fn test_docker_import_image_args() {}

    #[test]
    fn test_parse_csv_list() {
        assert_eq!(parse_csv_list("8080:80, 3000:3000"), vec!["8080:80", "3000:3000"]);
        assert_eq!(parse_csv_list("KEY1=VAL1, KEY2=VAL2"), vec!["KEY1=VAL1", "KEY2=VAL2"]);
        assert_eq!(
            parse_csv_list("/host/path:/container/path, myvol:/data"),
            vec!["/host/path:/container/path", "myvol:/data"]
        );
        assert!(parse_csv_list("").is_empty());
    }
}

#[tauri::command]
pub async fn get_container_templates(app: tauri::AppHandle) -> Result<Vec<ContainerTemplate>, AppError> {
    let mut templates = Vec::new();

    // 1. Load embedded templates (from the templates directory in the project)
    // In a real production app, we might use include_dir! or similar,
    // but for now we'll read from the known path relative to the executable or just the source for development.
    // Actually, let's use a simpler approach for this task: read from the directory we just created.

    let resource_path = app.path().resource_dir().map_err(|e| AppError::SystemError(e.to_string()))?;
    // During dev, resource_dir might not point to where we want.
    // Let's also check a "templates" folder in the current working directory.

    let mut search_paths = vec![
        resource_path.join("templates"),
        std::env::current_dir().unwrap_or_default().join("src-tauri").join("templates"),
        std::env::current_dir().unwrap_or_default().join("templates"),
    ];

    // Add user-defined templates path (e.g. in AppData)
    if let Ok(app_config_dir) = app.path().app_config_dir() {
        let user_templates_dir = app_config_dir.join("templates");
        let _ = std::fs::create_dir_all(&user_templates_dir);
        search_paths.push(user_templates_dir);
    }

    let mut seen_ids = std::collections::HashSet::new();

    for path in search_paths {
        if path.exists() && path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let file_path = entry.path();
                    if file_path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(content) = std::fs::read_to_string(&file_path) {
                            if let Ok(template) = serde_json::from_str::<ContainerTemplate>(&content) {
                                if !seen_ids.contains(&template.id) {
                                    seen_ids.insert(template.id.clone());
                                    templates.push(template);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(templates)
}
