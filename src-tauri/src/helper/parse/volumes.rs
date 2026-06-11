use crate::types::DockerVolume;

pub fn parse_docker_volumes(output: &str) -> Vec<DockerVolume> {
    let mut volumes = Vec::new();

    if let Ok(json_array) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(array) = json_array.as_array() {
            for item in array {
                if let Some(v) = parse_single_volume(item) {
                    volumes.push(v);
                }
            }
            return volumes;
        }
    }

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(vols) = json["Volumes"].as_array() {
            for item in vols {
                if let Some(v) = parse_single_volume(item) {
                    volumes.push(v);
                }
            }
        }
    }

    volumes
}

pub fn parse_single_volume(json: &serde_json::Value) -> Option<DockerVolume> {
    let name = json["Name"].as_str().unwrap_or("").to_string();
    if name.is_empty() {
        return None;
    }

    let driver = json["Driver"].as_str().unwrap_or("").to_string();
    let mountpoint = json["Mountpoint"].as_str().unwrap_or("").to_string();
    let created = json["CreatedAt"].as_str().unwrap_or("").to_string();

    Some(DockerVolume {
        name,
        driver,
        mountpoint,
        created,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_docker_volumes() {
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
}
