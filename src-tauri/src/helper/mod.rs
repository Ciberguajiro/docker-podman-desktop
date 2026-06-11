pub mod cli;
pub mod parse;

use crate::types::{CommandResult, ContainerFile, ContainerStats, DockerContainer, DockerImage, DockerNetwork, DockerVolume, Engine, ImageHistoryEntry};

pub use parse::parse_dual_format;

pub async fn check_engine_cli(engine: &Engine) -> bool {
    let cmd = engine.to_string();
    let result = cli::run_command_with_elevation_check(&cmd, &["--version"], false, 5).await;
    result.success
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
    cli::run_command_with_elevation_check(cmd, args, false, seconds).await
}

pub fn parse_docker_containers(output: &str) -> Vec<DockerContainer> {
    parse::containers::parse_docker_containers(output)
}

pub fn parse_docker_images(output: &str) -> Vec<DockerImage> {
    parse::images::parse_docker_images(output)
}

pub fn parse_docker_volumes(output: &str) -> Vec<DockerVolume> {
    parse::volumes::parse_docker_volumes(output)
}

pub fn parse_docker_networks(output: &str) -> Vec<DockerNetwork> {
    parse::networks::parse_docker_networks(output)
}

pub fn parse_docker_stats(output: &str) -> Vec<ContainerStats> {
    parse::stats::parse_docker_stats(output)
}

pub fn parse_single_stat(json: &serde_json::Value) -> Option<ContainerStats> {
    parse::stats::parse_single_stat(json)
}

pub fn parse_container_files(output: &str) -> Vec<ContainerFile> {
    parse::files::parse_container_files(output)
}

pub fn parse_docker_image_history(output: &str) -> Vec<ImageHistoryEntry> {
    parse::stats::parse_docker_image_history(output)
}
