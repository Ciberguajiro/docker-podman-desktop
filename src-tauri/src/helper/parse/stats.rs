use crate::types::{ContainerStats, ImageHistoryEntry};

pub use super::super::parse_dual_format as parse_dual;

pub fn parse_docker_stats(output: &str) -> Vec<ContainerStats> {
    parse_dual(output, parse_single_stat)
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
    let mem_usage = parts.first().unwrap_or(&"0B").to_string();
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

pub fn parse_docker_image_history(output: &str) -> Vec<ImageHistoryEntry> {
    parse_dual(output, parse_single_history_entry)
}

pub fn parse_single_history_entry(json: &serde_json::Value) -> Option<ImageHistoryEntry> {
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
