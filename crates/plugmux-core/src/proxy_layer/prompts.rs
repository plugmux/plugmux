//! Prompt aggregation and routing for the proxy layer.

use crate::proxy::PromptInfo;
use super::tools::NS_SEP;

pub fn namespace_prompt(server_id: &str, prompt: &PromptInfo) -> PromptInfo {
    PromptInfo {
        name: format!("{server_id}{NS_SEP}{}", prompt.name),
        description: prompt.description.as_ref().map(|d| format!("[{}] {}", server_id, d)),
        arguments: prompt.arguments.clone(),
    }
}

pub fn parse_namespaced_prompt(name: &str) -> Option<(&str, &str)> {
    name.split_once(NS_SEP)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::PromptArgument;

    #[test]
    fn test_namespace_prompt() {
        let prompt = PromptInfo {
            name: "code-review".to_string(),
            description: Some("Review code".to_string()),
            arguments: vec![PromptArgument {
                name: "language".to_string(),
                description: None,
                required: true,
            }],
        };
        let ns = namespace_prompt("figma", &prompt);
        assert_eq!(ns.name, "figma__code-review");
        assert_eq!(ns.description.unwrap(), "[figma] Review code");
        assert_eq!(ns.arguments.len(), 1);
    }

    #[test]
    fn test_parse_namespaced_prompt() {
        let (sid, name) = parse_namespaced_prompt("figma__code-review").unwrap();
        assert_eq!(sid, "figma");
        assert_eq!(name, "code-review");
    }
}
