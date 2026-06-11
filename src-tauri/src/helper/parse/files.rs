use crate::types::ContainerFile;

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
