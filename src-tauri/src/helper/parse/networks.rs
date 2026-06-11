use crate::types::DockerNetwork;

pub use super::super::parse_dual_format as parse_dual;

pub fn parse_docker_networks(output: &str) -> Vec<DockerNetwork> {
    parse_dual(output, parse_single_network)
}

pub fn parse_single_network(json: &serde_json::Value) -> Option<DockerNetwork> {
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

    let subnet = if let Some(ipam) = json["IPAM"].as_str() {
        ipam.to_string()
    } else if let Some(config) = json["IPAM"]["Config"].as_array() {
        config
            .get(0)
            .and_then(|c| c["Subnet"].as_str())
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    };

    Some(DockerNetwork {
        id,
        name,
        driver,
        scope,
        subnet,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
