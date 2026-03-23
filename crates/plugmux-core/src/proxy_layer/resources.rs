//! Resource aggregation and routing. Uses plugmux-res://{server_id}/{original_uri}.

use crate::proxy::ResourceInfo;

const SCHEME: &str = "plugmux-res://";

pub fn namespace_resource(server_id: &str, resource: &ResourceInfo) -> ResourceInfo {
    ResourceInfo {
        uri: format!("{SCHEME}{server_id}/{}", resource.uri),
        name: format!("[{}] {}", server_id, resource.name),
        description: resource.description.clone(),
        mime_type: resource.mime_type.clone(),
    }
}

pub fn parse_namespaced_uri(uri: &str) -> Option<(String, String)> {
    let rest = uri.strip_prefix(SCHEME)?;
    let slash_pos = rest.find('/')?;
    let server_id = &rest[..slash_pos];
    let original_uri = &rest[slash_pos + 1..];
    if server_id.is_empty() || original_uri.is_empty() {
        return None;
    }
    Some((server_id.to_string(), original_uri.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_resource() -> ResourceInfo {
        ResourceInfo {
            uri: "file:///logs/app.log".to_string(),
            name: "App Log".to_string(),
            description: Some("Application log file".to_string()),
            mime_type: Some("text/plain".to_string()),
        }
    }

    #[test]
    fn test_namespace_resource() {
        let namespaced = namespace_resource("figma", &sample_resource());
        assert_eq!(namespaced.uri, "plugmux-res://figma/file:///logs/app.log");
        assert_eq!(namespaced.name, "[figma] App Log");
    }

    #[test]
    fn test_parse_namespaced_uri() {
        let (sid, orig) = parse_namespaced_uri("plugmux-res://figma/file:///logs/app.log").unwrap();
        assert_eq!(sid, "figma");
        assert_eq!(orig, "file:///logs/app.log");
    }

    #[test]
    fn test_parse_namespaced_uri_invalid() {
        assert!(parse_namespaced_uri("file:///logs/app.log").is_none());
        assert!(parse_namespaced_uri("plugmux-res://").is_none());
        assert!(parse_namespaced_uri("plugmux-res:///file:///x").is_none());
    }

    #[test]
    fn test_roundtrip() {
        let original = sample_resource();
        let namespaced = namespace_resource("myserver", &original);
        let (sid, orig_uri) = parse_namespaced_uri(&namespaced.uri).unwrap();
        assert_eq!(sid, "myserver");
        assert_eq!(orig_uri, original.uri);
    }
}
