use crate::types::DockerContainer;

pub use super::super::parse_dual_format as parse_dual;

pub fn parse_docker_containers(output: &str) -> Vec<DockerContainer> {
    parse_dual(output, parse_single_container)
}

pub fn parse_single_container(json: &serde_json::Value) -> Option<DockerContainer> {
    let id = json["ID"]
        .as_str()
        .or(json["Id"].as_str())
        .unwrap_or("")
        .to_string();

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
        p.iter()
            .map(|v| {
                if let Some(obj) = v.as_object() {
                    let host_port = obj.get("hostPort").and_then(|hp| hp.as_u64());
                    let container_port = obj.get("containerPort").and_then(|cp| cp.as_u64());
                    if let (Some(hp), Some(cp)) = (host_port, container_port) {
                        format!("{}:{}", hp, cp)
                    } else {
                        v.to_string().replace('"', "")
                    }
                } else {
                    v.to_string().replace('"', "")
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

    let (compose_project, compose_service) = if let Some(labels) = json["Labels"].as_object() {
        (
            labels
                .get("com.docker.compose.project")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            labels
                .get("com.docker.compose.service")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        )
    } else if let Some(labels_str) = json["Labels"].as_str() {
        let mut project = None;
        let mut service = None;
        for label in labels_str.split(", ") {
            if label.starts_with("com.docker.compose.project=") {
                project = Some(label.replace("com.docker.compose.project=", ""));
            } else if label.starts_with("com.docker.compose.service=") {
                service = Some(label.replace("com.docker.compose.service=", ""));
            }
        }
        (project, service)
    } else {
        (None, None)
    };

    if id.is_empty() && name.is_empty() {
        None
    } else {
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
}
