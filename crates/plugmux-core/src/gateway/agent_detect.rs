//! Agent detection from HTTP headers.

const AGENT_PATTERNS: &[(&str, &str)] = &[
    ("claude-code", "claude-code"),
    ("claude-desktop", "claude-desktop"),
    ("cursor", "cursor"),
    ("windsurf", "windsurf"),
    ("codex", "codex"),
    ("vscode", "vscode"),
    ("zed", "zed"),
    ("continue", "continue"),
];

pub fn detect_agent(user_agent: &str) -> Option<String> {
    let ua_lower = user_agent.to_lowercase();
    for (pattern, agent_id) in AGENT_PATTERNS {
        if ua_lower.contains(pattern) {
            return Some(agent_id.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_claude_code() {
        assert_eq!(
            detect_agent("Claude-Code/1.0"),
            Some("claude-code".to_string())
        );
    }

    #[test]
    fn test_detect_cursor() {
        assert_eq!(
            detect_agent("Mozilla/5.0 Cursor/0.48"),
            Some("cursor".to_string())
        );
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect_agent("SomeRandomAgent/1.0"), None);
    }
}
