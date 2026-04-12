use crate::types::{
    CommandResult, ContainerFile, ContainerStats, DockerContainer, DockerImage, DockerNetwork,
    DockerVolume, Engine, ImageHistoryEntry,
};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub async fn check_engine_cli(engine: &Engine) -> bool {
    let cmd = engine.to_string();
    let output = Command::new(cmd).arg("--version").output().await;

    output.map(|o| o.status.success()).unwrap_or(false)
}

pub async fn run_engine_command(engine: &Engine, args: &[&str]) -> CommandResult {
    run_generic_command(&engine.to_string(), args).await
}

pub async fn run_generic_command(cmd: &str, args: &[&str]) -> CommandResult {
    run_generic_command_with_timeout(cmd, args, 30).await
}

pub async fn run_generic_command_with_timeout(
    cmd: &str,
    args: &[&str],
    seconds: u64,
) -> CommandResult {
    let cmd_timeout = Duration::from_secs(seconds);

    let process = Command::new(cmd).args(args).output();

    let result = timeout(cmd_timeout, process).await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if output.status.success() {
                CommandResult {
                    success: true,
                    output: stdout,
                    error: if stderr.is_empty() {
                        None
                    } else {
                        Some(stderr)
                    },
                }
            } else {
                CommandResult {
                    success: false,
                    output: stdout,
                    error: Some(stderr),
                }
            }
        }
        Ok(Err(e)) => CommandResult {
            success: false,
            output: String::new(),
            error: Some(format!("Failed to execute {} command: {}", cmd, e)),
        },
        Err(_) => CommandResult {
            success: false,
            output: String::new(),
            error: Some(format!(
                "{} command timed out after {} seconds",
                cmd, seconds
            )),
        },
    }
}

pub fn parse_docker_containers(output: &str) -> Vec<DockerContainer> {
    let mut containers = Vec::new();

    // Try parsing as a JSON array first (Podman style)
    if let Ok(json_array) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(array) = json_array.as_array() {
            for item in array {
                if let Some(c) = parse_single_container(item) {
                    containers.push(c);
                }
            }
            return containers;
        }
    }

    // Fallback to line-delimited JSON (Docker style)
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(c) = parse_single_container(&json) {
                containers.push(c);
            }
        }
    }

    containers
}

fn parse_single_container(json: &serde_json::Value) -> Option<DockerContainer> {
    let id = json["ID"]
        .as_str()
        .or(json["Id"].as_str())
        .unwrap_or("")
        .to_string();

    // Podman may return Names as an array
    let name = if let Some(names) = json["Names"].as_array() {
        names
            .get(0)
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        json["Names"].as_str().unwrap_or("").to_string()
    };

    let image = json["Image"].as_str().unwrap_or("").to_string();
    let status = json["Status"].as_str().unwrap_or("").to_string();
    let state = json["State"].as_str().unwrap_or("").to_string();
    let created = json["CreatedAt"].as_str().unwrap_or("").to_string();

    let ports: Vec<String> = if let Some(p) = json["Ports"].as_array() {
        // Podman style array of port objects or strings
        p.iter()
            .map(|v| {
                if let Some(obj) = v.as_object() {
                    let host_port = obj.get("hostPort").and_then(|hp| hp.as_u64());
                    let container_port = obj.get("containerPort").and_then(|cp| cp.as_u64());
                    if let (Some(hp), Some(cp)) = (host_port, container_port) {
                        format!("{}:{}", hp, cp)
                    } else {
                        v.to_string().replace("\"", "")
                    }
                } else {
                    v.to_string().replace("\"", "")
                }
            })
            .collect()
    } else if let Some(p_str) = json["Ports"].as_str() {
        if p_str.is_empty() {
            Vec::new()
        } else {
            p_str.split(", ").map(|s| s.to_string()).collect()
        }
    } else {
        Vec::new()
    };

    // Extract compose project and service from labels
    let labels_str = json["Labels"].as_str().unwrap_or("");
    let compose_project = if labels_str.is_empty() {
        json["Labels"]["com.docker.compose.project"]
            .as_str()
            .map(|s| s.to_string())
    } else {
        labels_str.split(", ").find_map(|label| {
            if let Some(pos) = label.find("com.docker.compose.project=") {
                Some(label[pos + 27..].to_string())
            } else {
                None
            }
        })
    };

    let compose_service = if labels_str.is_empty() {
        json["Labels"]["com.docker.compose.service"]
            .as_str()
            .map(|s| s.to_string())
    } else {
        labels_str.split(", ").find_map(|label| {
            if let Some(pos) = label.find("com.docker.compose.service=") {
                Some(label[pos + 27..].to_string())
            } else {
                None
            }
        })
    };

    Some(DockerContainer {
        id,
        name,
        image,
        status,
        state,
        created,
        ports,
        compose_project,
        compose_service,
    })
}

pub fn parse_docker_images(output: &str) -> Vec<DockerImage> {
    let mut images = Vec::new();

    // Try parsing as a JSON array first (Podman style)
    if let Ok(json_array) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(array) = json_array.as_array() {
            for item in array {
                if let Some(img) = parse_single_image(item) {
                    images.push(img);
                }
            }
            return images;
        }
    }

    // Fallback to line-delimited JSON (Docker style)
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(img) = parse_single_image(&json) {
                images.push(img);
            }
        }
    }

    images
}

fn parse_single_image(json: &serde_json::Value) -> Option<DockerImage> {
    let id = json["ID"]
        .as_str()
        .or(json["Id"].as_str())
        .unwrap_or("")
        .to_string();

    // Podman may use Names array for repository and tag
    let (repository, tag) = if let Some(repo) = json["Repository"].as_str() {
        (
            repo.to_string(),
            json["Tag"].as_str().unwrap_or("").to_string(),
        )
    } else if let Some(names) = json["Names"].as_array() {
        let full_name = names.get(0).and_then(|n| n.as_str()).unwrap_or("");
        if let Some(pos) = full_name.find(':') {
            (
                full_name[..pos].to_string(),
                full_name[pos + 1..].to_string(),
            )
        } else {
            (full_name.to_string(), "latest".to_string())
        }
    } else {
        ("".to_string(), "".to_string())
    };

    let size = json["Size"]
        .as_str()
        .or(json["Size"].as_u64().map(|s| s.to_string()).as_deref())
        .unwrap_or("")
        .to_string();
    let created = json["CreatedSince"]
        .as_str()
        .or(json["CreatedAt"].as_str())
        .unwrap_or("")
        .to_string();

    Some(DockerImage {
        id,
        repository,
        tag,
        size,
        created,
    })
}

pub fn parse_docker_volumes(output: &str) -> Vec<DockerVolume> {
    let mut volumes = Vec::new();

    // Try to parse as a single JSON object or array
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(vols) = json["Volumes"].as_array() {
            // Docker style nested Volumes array
            for vol in vols {
                if let Some(v) = parse_single_volume(vol) {
                    volumes.push(v);
                }
            }
            return volumes;
        } else if let Some(array) = json.as_array() {
            // Podman style flat array
            for item in array {
                if let Some(v) = parse_single_volume(item) {
                    volumes.push(v);
                }
            }
            return volumes;
        }
    }

    // Fallback to line-delimited JSON
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(v) = parse_single_volume(&json) {
                volumes.push(v);
            }
        }
    }

    volumes
}

fn parse_single_volume(json: &serde_json::Value) -> Option<DockerVolume> {
    // If it's a wrapper object with Volumes key (can happen in line-delimited sometimes)
    if let Some(vols) = json["Volumes"].as_array() {
        return None; // This case should be handled by the caller if they see a wrapper
    }

    let name = json["Name"].as_str().unwrap_or("").to_string();
    if name.is_empty() {
        return None;
    }

    Some(DockerVolume {
        name,
        driver: json["Driver"].as_str().unwrap_or("").to_string(),
        mountpoint: json["Mountpoint"].as_str().unwrap_or("").to_string(),
        created: json["CreatedAt"].as_str().unwrap_or("").to_string(),
    })
}

pub fn parse_docker_networks(output: &str) -> Vec<DockerNetwork> {
    let mut networks = Vec::new();

    // Try parsing as a JSON array first (Podman style)
    if let Ok(json_array) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(array) = json_array.as_array() {
            for item in array {
                if let Some(n) = parse_single_network(item) {
                    networks.push(n);
                }
            }
            return networks;
        }
    }

    // Fallback to line-delimited JSON (Docker style)
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(n) = parse_single_network(&json) {
                networks.push(n);
            }
        }
    }

    networks
}

fn parse_single_network(json: &serde_json::Value) -> Option<DockerNetwork> {
    let id = json["ID"]
        .as_str()
        .or(json["Id"].as_str())
        .unwrap_or("")
        .to_string();
    let name = json["Name"].as_str().unwrap_or("").to_string();
    if name.is_empty() && id.is_empty() {
        return None;
    }

    let driver = json["Driver"].as_str().unwrap_or("").to_string();
    let scope = json["Scope"].as_str().unwrap_or("").to_string();

    // Podman may have subnet nested in IPAM config array
    let subnet = if let Some(ipam) = json["IPAM"].as_str() {
        ipam.to_string()
    } else if let Some(config) = json["IPAM"]["Config"].as_array() {
        config
            .get(0)
            .and_then(|c| c["Subnet"].as_str())
            .unwrap_or("")
            .to_string()
    } else {
        "".to_string()
    };

    Some(DockerNetwork {
        id,
        name,
        driver,
        scope,
        subnet,
    })
}

pub fn parse_docker_stats(output: &str) -> Vec<ContainerStats> {
    let mut stats = Vec::new();

    // Try parsing as a JSON array first (Podman style)
    if let Ok(json_array) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(array) = json_array.as_array() {
            for item in array {
                if let Some(s) = parse_single_stat(item) {
                    stats.push(s);
                }
            }
            return stats;
        }
    }

    // Fallback to line-delimited JSON (Docker style)
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(s) = parse_single_stat(&json) {
                stats.push(s);
            }
        }
    }

    stats
}

pub fn parse_single_stat(json: &serde_json::Value) -> Option<ContainerStats> {
    let id = json["ID"]
        .as_str()
        .or(json["Container"].as_str())
        .unwrap_or("")
        .to_string();
    let name = json["Name"].as_str().unwrap_or("").to_string();
    if id.is_empty() {
        return None;
    }

    let cpu_percent = json["CPUPerc"].as_str().unwrap_or("0%").to_string();
    let mem_usage_limit = json["MemUsage"].as_str().unwrap_or("0 / 0").to_string();
    let mem_percent = json["MemPerc"].as_str().unwrap_or("0%").to_string();
    let net_io = json["NetIO"].as_str().unwrap_or("0 / 0").to_string();
    let block_io = json["BlockIO"].as_str().unwrap_or("0 / 0").to_string();
    let pids = json["PIDs"]
        .as_str()
        .or(json["PIDs"].as_u64().map(|v| v.to_string()).as_deref())
        .unwrap_or("0")
        .to_string();

    let parts: Vec<&str> = mem_usage_limit.split(" / ").collect();
    let mem_usage = parts.get(0).unwrap_or(&"0B").to_string();
    let mem_limit = parts.get(1).unwrap_or(&"0B").to_string();

    Some(ContainerStats {
        id,
        name,
        cpu_percent,
        mem_usage,
        mem_limit,
        mem_percent,
        net_io,
        block_io,
        pids,
    })
}

pub fn parse_container_files(output: &str) -> Vec<ContainerFile> {
    let mut files = Vec::new();
    for line in output.lines() {
        if line.starts_with("total") || line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let permissions = parts[0].to_string();
        let size = parts[4].to_string();
        let modified = format!("{} {} {}", parts[5], parts[6], parts[7]);
        let mut name = parts[8..].join(" ");

        let is_dir = permissions.starts_with('d') || name.ends_with('/');

        if name.ends_with('/') {
            name.pop();
        }

        // Skip . and ..
        if name == "." || name == ".." {
            continue;
        }

        files.push(ContainerFile {
            name,
            is_dir,
            size,
            modified,
            permissions,
        });
    }
    files
}

pub fn parse_docker_image_history(output: &str) -> Vec<ImageHistoryEntry> {
    let mut history = Vec::new();

    // Try parsing as a JSON array first (Podman style)
    if let Ok(json_array) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(array) = json_array.as_array() {
            for item in array {
                if let Some(h) = parse_single_history_entry(item) {
                    history.push(h);
                }
            }
            return history;
        }
    }

    // Fallback to line-delimited JSON (Docker style)
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(h) = parse_single_history_entry(&json) {
                history.push(h);
            }
        }
    }

    history
}

fn parse_single_history_entry(json: &serde_json::Value) -> Option<ImageHistoryEntry> {
    let id = json["ID"]
        .as_str()
        .or(json["Id"].as_str())
        .unwrap_or("")
        .to_string();
    if id.is_empty() && json["CreatedAt"].is_null() {
        return None;
    }

    Some(ImageHistoryEntry {
        id,
        created: json["CreatedAt"].as_str().unwrap_or("").to_string(),
        created_by: json["CreatedBy"].as_str().unwrap_or("").to_string(),
        size: json["Size"]
            .as_str()
            .or(json["Size"].as_u64().map(|s| s.to_string()).as_deref())
            .unwrap_or("")
            .to_string(),
        comment: json["Comment"].as_str().unwrap_or("").to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_docker_containers() {
        let output = r#"{"ID":"1234567890ab","Names":"test-container","Image":"nginx:latest","Status":"Up 2 hours","State":"running","CreatedAt":"2023-10-27 10:00:00 +0000 UTC","Ports":"80/tcp","Labels":"com.docker.compose.project=test, com.docker.compose.service=web"}"#;
        let containers = parse_docker_containers(output);
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].id, "1234567890ab");
        assert_eq!(containers[0].name, "test-container");
        assert_eq!(containers[0].state, "running");
        assert_eq!(containers[0].compose_project, Some("test".to_string()));
    }

    #[test]
    fn test_parse_podman_containers() {
        let output = r#"[{"Id":"12345","Names":["podman-test"],"Image":"alpine","Status":"Up","State":"running","CreatedAt":"2023-10-27","Ports":[{"hostPort":80,"containerPort":80}],"Labels":{"com.docker.compose.project":"pod-project"}}]"#;
        let containers = parse_docker_containers(output);
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].id, "12345");
        assert_eq!(containers[0].name, "podman-test");
        assert_eq!(
            containers[0].compose_project,
            Some("pod-project".to_string())
        );
        assert_eq!(containers[0].ports, vec!["80:80"]);
    }

    #[test]
    fn test_parse_docker_images() {
        let output = r#"{"ID":"sha256:1234567890ab","Repository":"nginx","Tag":"latest","Size":"123MB","CreatedSince":"2 days ago"}"#;
        let images = parse_docker_images(output);
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].repository, "nginx");
        assert_eq!(images[0].tag, "latest");
    }

    #[test]
    fn test_parse_podman_images() {
        let output = r#"[{"Id":"img123","Names":["localhost/app:v1"],"Size":5000000,"CreatedAt":"2023-10-27"}]"#;
        let images = parse_docker_images(output);
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].id, "img123");
        assert_eq!(images[0].repository, "localhost/app");
        assert_eq!(images[0].tag, "v1");
    }

    #[test]
    fn test_parse_docker_volumes() {
        // Test single object format
        let output = r#"{"Volumes":[{"Name":"test-vol","Driver":"local","Mountpoint":"/var/lib/docker/volumes/test-vol/_data","CreatedAt":"2023-10-27T10:00:00Z"}]}"#;
        let volumes = parse_docker_volumes(output);
        assert_eq!(volumes.len(), 1);
        assert_eq!(volumes[0].name, "test-vol");
    }

    #[test]
    fn test_parse_podman_volumes() {
        let output = r#"[{"Name":"pod-vol","Driver":"local","Mountpoint":"/data"}]"#;
        let volumes = parse_docker_volumes(output);
        assert_eq!(volumes.len(), 1);
        assert_eq!(volumes[0].name, "pod-vol");
    }

    #[test]
    fn test_parse_docker_networks() {
        let output = r#"{"ID":"1234567890ab","Name":"bridge","Driver":"bridge","Scope":"local","IPAM":"172.17.0.0/16"}"#;
        let networks = parse_docker_networks(output);
        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].name, "bridge");
        assert_eq!(networks[0].driver, "bridge");
    }

    #[test]
    fn test_parse_podman_networks() {
        let output = r#"[{"Id":"net123","Name":"pod-net","Driver":"bridge","IPAM":{"Config":[{"Subnet":"10.0.0.0/24"}]}}]"#;
        let networks = parse_docker_networks(output);
        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].name, "pod-net");
        assert_eq!(networks[0].subnet, "10.0.0.0/24");
    }

    #[test]
    fn test_parse_docker_stats() {
        let output = r#"{"ID":"container1","Name":"test-container","CPUPerc":"1.50%","MemUsage":"50MiB / 1GiB","MemPerc":"4.88%","NetIO":"1kB / 2kB","BlockIO":"0B / 0B","PIDs":"10"}"#;
        let stats = parse_docker_stats(output);
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].id, "container1");
        assert_eq!(stats[0].cpu_percent, "1.50%");
        assert_eq!(stats[0].mem_usage, "50MiB");
        assert_eq!(stats[0].mem_limit, "1GiB");
        assert_eq!(stats[0].pids, "10");
    }

    #[test]
    fn test_parse_docker_image_history() {
        let output = r#"{"ID":"sha256:123","CreatedAt":"2023-10-27","CreatedBy":"RUN echo hello","Size":"10MB","Comment":"test comment"}"#;
        let history = parse_docker_image_history(output);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, "sha256:123");
        assert_eq!(history[0].size, "10MB");
    }
}
