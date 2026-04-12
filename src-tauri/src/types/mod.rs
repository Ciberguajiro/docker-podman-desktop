use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
    pub created: String,
    pub ports: Vec<String>,
    pub compose_project: Option<String>,
    pub compose_service: Option<String>,

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerFile {
    pub name: String,
    pub is_dir: bool,
    pub size: String,
    pub modified: String,
    pub permissions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerImage {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: String,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageHistoryEntry {
    pub id: String,
    pub created: String,
    pub created_by: String,
    pub size: String,
    pub comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerVolume {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerNetwork {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub subnet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerInfo {
    pub containers: i32,
    pub containers_running: i32,
    pub containers_stopped: i32,
    pub images: i32,
    pub server_version: String,
    pub storage_driver: String,
    pub operating_system: String,
    pub architecture: String,
    pub cpus: i32,
    pub memory: String,
    pub kernel_version: String,
    pub os_type: String,
    pub cgroup_driver: String,
    pub cgroup_version: String,
    pub logging_driver: String,
    pub docker_root_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStats {
    pub id: String,
    pub name: String,
    pub cpu_percent: String,
    pub mem_usage: String,
    pub mem_limit: String,
    pub mem_percent: String,
    pub net_io: String,
    pub block_io: String,
    pub pids: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: Vec<f32>,
    pub mem_total: u64,
    pub mem_used: u64,
    pub mem_free: u64,
    pub mem_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllMetrics {
    pub system: SystemMetrics,
    pub containers: Vec<ContainerStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AppError {
    DockerError(String),
    SystemError(String),
    TimeoutError(String),
    ParseError(String),
    NotFound(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::DockerError(e) => write!(f, "Docker error: {}", e),
            AppError::SystemError(e) => write!(f, "System error: {}", e),
            AppError::TimeoutError(e) => write!(f, "Timeout error: {}", e),
            AppError::ParseError(e) => write!(f, "Parse error: {}", e),
            AppError::NotFound(e) => write!(f, "Not found: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullEvent {
    pub status: String,
    pub progress: Option<String>,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildEvent {
    pub line: String,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsEvent {
    pub stats: Vec<ContainerStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub line: String,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerEvent {
    #[serde(alias = "Status")]
    pub status: String,
    #[serde(alias = "ID", alias = "Id")]
    pub id: String,
    #[serde(alias = "From")]
    pub from: String,
    #[serde(rename = "Type", alias = "type")]
    pub event_type: String,
    #[serde(rename = "Action", alias = "action")]
    pub action: String,
    #[serde(rename = "Actor", alias = "actor")]
    pub actor: DockerEventActor,
    #[serde(rename = "time", alias = "Time")]
    pub time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerEventActor {
    #[serde(rename = "ID", alias = "Id")]
    pub id: String,
    #[serde(rename = "Attributes", alias = "attributes")]
    pub attributes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Engine {
    #[serde(rename = "docker")]
    Docker,
    #[serde(rename = "podman")]
    Podman,
}

impl std::fmt::Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Engine::Docker => write!(f, "docker"),
            Engine::Podman => write!(f, "podman"),
        }
    }
}
