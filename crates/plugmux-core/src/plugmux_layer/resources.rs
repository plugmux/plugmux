//! Resource definitions for the plugmux management layer.
//!
//! These resources are exposed on `/env/global` and provide read-only
//! views into plugmux's state: servers, environments, agents, and logs.

use crate::proxy::ResourceInfo;

/// Return the full set of plugmux management resource schemas.
pub fn list_resources() -> Vec<ResourceInfo> {
    vec![
        ResourceInfo {
            uri: "plugmux://servers".to_string(),
            name: "servers".to_string(),
            description: Some(
                "All servers with health and connection status".to_string(),
            ),
            mime_type: Some("application/json".to_string()),
        },
        ResourceInfo {
            uri: "plugmux://environments".to_string(),
            name: "environments".to_string(),
            description: Some(
                "All environments with their server lists".to_string(),
            ),
            mime_type: Some("application/json".to_string()),
        },
        ResourceInfo {
            uri: "plugmux://agents".to_string(),
            name: "agents".to_string(),
            description: Some(
                "Connected and detected agents".to_string(),
            ),
            mime_type: Some("application/json".to_string()),
        },
        ResourceInfo {
            uri: "plugmux://logs/recent".to_string(),
            name: "logs/recent".to_string(),
            description: Some(
                "Recent gateway activity log".to_string(),
            ),
            mime_type: Some("application/json".to_string()),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_resources_returns_four_resources() {
        let resources = list_resources();
        assert_eq!(resources.len(), 4);
    }

    #[test]
    fn test_resource_uris_use_plugmux_scheme() {
        for res in list_resources() {
            assert!(
                res.uri.starts_with("plugmux://"),
                "resource '{}' should start with 'plugmux://'",
                res.uri,
            );
        }
    }

    #[test]
    fn test_all_resources_have_json_mime_type() {
        for res in list_resources() {
            assert_eq!(
                res.mime_type.as_deref(),
                Some("application/json"),
                "resource '{}' should have application/json mime type",
                res.uri,
            );
        }
    }
}
