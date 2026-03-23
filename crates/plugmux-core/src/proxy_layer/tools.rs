//! Tool aggregation and routing for the proxy layer.

use crate::proxy::ToolInfo;

pub const NS_SEP: &str = "__";

pub fn namespace_tool(server_id: &str, tool: &ToolInfo) -> ToolInfo {
    ToolInfo {
        name: format!("{server_id}{NS_SEP}{}", tool.name),
        description: format!("[{}] {}", server_id, tool.description),
        input_schema: tool.input_schema.clone(),
        output_schema: tool.output_schema.clone(),
        annotations: tool.annotations.clone(),
    }
}

pub fn parse_namespaced_tool(name: &str) -> Option<(&str, &str)> {
    name.split_once(NS_SEP)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_tool() -> ToolInfo {
        ToolInfo {
            name: "get_screenshot".to_string(),
            description: "Capture a screenshot".to_string(),
            input_schema: json!({"type": "object"}),
            output_schema: None,
            annotations: None,
        }
    }

    #[test]
    fn test_namespace_tool() {
        let namespaced = namespace_tool("figma", &sample_tool());
        assert_eq!(namespaced.name, "figma__get_screenshot");
        assert_eq!(namespaced.description, "[figma] Capture a screenshot");
    }

    #[test]
    fn test_parse_namespaced_tool() {
        let (server_id, tool_name) = parse_namespaced_tool("figma__get_screenshot").unwrap();
        assert_eq!(server_id, "figma");
        assert_eq!(tool_name, "get_screenshot");
    }

    #[test]
    fn test_parse_namespaced_tool_no_separator() {
        assert!(parse_namespaced_tool("get_screenshot").is_none());
    }

    #[test]
    fn test_parse_namespaced_tool_multiple_separators() {
        let (server_id, tool_name) = parse_namespaced_tool("figma__get__screenshot").unwrap();
        assert_eq!(server_id, "figma");
        assert_eq!(tool_name, "get__screenshot");
    }
}
