pub mod containers;
pub mod images;
pub mod volumes;
pub mod networks;
pub mod stats;
pub mod files;

/// Generic parser that handles both Docker (line-delimited JSON) and Podman
/// (JSON array) output formats. Tries JSON array first, falls back to LD-JSON.
pub fn parse_dual_format<T>(
    output: &str,
    parse_item: fn(&serde_json::Value) -> Option<T>,
) -> Vec<T> {
    let mut results = Vec::new();

    if let Ok(json_array) = serde_json::from_str::<serde_json::Value>(output) {
        if let Some(array) = json_array.as_array() {
            for item in array {
                if let Some(parsed) = parse_item(item) {
                    results.push(parsed);
                }
            }
            return results;
        }
    }

    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(parsed) = parse_item(&json) {
                results.push(parsed);
            }
        }
    }

    results
}
