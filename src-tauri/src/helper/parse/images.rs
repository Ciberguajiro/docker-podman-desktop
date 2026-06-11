use crate::types::DockerImage;

pub use super::super::parse_dual_format as parse_dual;

pub fn parse_docker_images(output: &str) -> Vec<DockerImage> {
    parse_dual(output, parse_single_image)
}

pub fn parse_single_image(json: &serde_json::Value) -> Option<DockerImage> {
    let id = json["ID"]
        .as_str()
        .or(json["Id"].as_str())
        .unwrap_or("")
        .to_string();

    let (repository, tag) = if let Some(names) = json["Names"].as_array() {
        if let Some(name) = names.get(0).and_then(|n| n.as_str()) {
            if let Some(pos) = name.rfind(':') {
                (name[..pos].to_string(), name[pos + 1..].to_string())
            } else {
                (name.to_string(), "latest".to_string())
            }
        } else {
            (String::new(), String::new())
        }
    } else {
        (
            json["Repository"].as_str().unwrap_or("").to_string(),
            json["Tag"].as_str().unwrap_or("").to_string(),
        )
    };

    if (id.is_empty() && repository.is_empty()) || repository == "<none>" {
        return None;
    }

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
