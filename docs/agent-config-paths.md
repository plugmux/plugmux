# Agent Configuration Paths

Reference for all supported AI agents and their MCP configuration file locations.

## Auto-detected Agents

These agents have known config paths. Plugmux detects them automatically.

### Claude Code
| OS | Path |
|----|------|
| macOS | `~/.claude.json` |
| Linux | `~/.claude.json` |
| Windows | `%USERPROFILE%\.claude.json` |
| Format | JSON |
| Key | `mcpServers` |
| Install | https://docs.anthropic.com/en/docs/claude-code/overview |

### Claude Desktop
| OS | Path |
|----|------|
| macOS | `~/Library/Application Support/Claude/claude_desktop_config.json` |
| Linux | `~/.config/Claude/claude_desktop_config.json` |
| Windows | `%APPDATA%\Claude\claude_desktop_config.json` |
| Format | JSON |
| Key | `mcpServers` |
| Note | Stdio transport only |
| Install | https://claude.ai/download |

### Cline (VS Code Extension)
| OS | Path |
|----|------|
| macOS | `~/Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json` |
| Linux | `~/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json` |
| Windows | `%APPDATA%\Code\User\globalStorage\saoudrizwan.claude-dev\settings\cline_mcp_settings.json` |
| Format | JSON |
| Key | `mcpServers` |
| Install | https://marketplace.visualstudio.com/items?itemName=saoudrizwan.claude-dev |

### Codex
| OS | Path |
|----|------|
| macOS | `~/.codex/config.toml` |
| Linux | `~/.codex/config.toml` |
| Windows | `%USERPROFILE%\.codex\config.toml` |
| Format | TOML |
| Key | `mcp_servers` |
| Install | https://github.com/openai/codex |

### Cursor
| OS | Path |
|----|------|
| macOS | `~/.cursor/mcp.json` |
| Linux | `~/.cursor/mcp.json` |
| Windows | `%USERPROFILE%\.cursor\mcp.json` |
| Format | JSON |
| Key | `mcpServers` |
| Install | https://www.cursor.com/downloads |

### Gemini CLI
| OS | Path |
|----|------|
| macOS | `~/.gemini/settings.json` |
| Linux | `~/.gemini/settings.json` |
| Windows | `%USERPROFILE%\.gemini\settings.json` |
| Format | JSON |
| Key | `mcpServers` |
| Install | https://github.com/google-gemini/gemini-cli |

### Kiro
| OS | Path |
|----|------|
| macOS | `~/.kiro/settings/mcp.json` |
| Linux | `~/.kiro/settings/mcp.json` |
| Windows | `%USERPROFILE%\.kiro\settings\mcp.json` |
| Format | JSON |
| Key | `mcpServers` |
| Install | https://kiro.dev/downloads/ |

### LM Studio
| OS | Path |
|----|------|
| macOS | `~/.lmstudio/mcp.json` |
| Linux | `~/.lmstudio/mcp.json` |
| Windows | `%USERPROFILE%\.lmstudio\mcp.json` |
| Format | JSON |
| Key | `mcpServers` |
| Install | https://lmstudio.ai/ |

### OpenCode
| OS | Path |
|----|------|
| macOS | `~/.config/opencode/opencode.json` |
| Linux | `~/.config/opencode/opencode.json` |
| Windows | `%APPDATA%\opencode\opencode.json` |
| Format | JSON |
| Key | `mcp` |
| Install | https://opencode.ai/download |

### VS Code
| OS | Path |
|----|------|
| macOS | `~/Library/Application Support/Code/User/mcp.json` |
| Linux | `~/.config/Code/User/mcp.json` |
| Windows | `%APPDATA%\Code\User\mcp.json` |
| Format | JSON |
| Key | `servers` |
| Install | https://code.visualstudio.com/download |

### Windsurf
| OS | Path |
|----|------|
| macOS | `~/.codeium/windsurf/mcp_config.json` |
| Linux | `~/.codeium/windsurf/mcp_config.json` |
| Windows | `%USERPROFILE%\.codeium\windsurf\mcp_config.json` |
| Format | JSON |
| Key | `mcpServers` |
| Install | https://windsurf.com/download |

### Zed
| OS | Path |
|----|------|
| macOS | `~/.config/zed/settings.json` |
| Linux | `~/.config/zed/settings.json` |
| Windows | Not supported |
| Format | JSON |
| Key | `context_servers` |
| Install | https://zed.dev/download |

### GitHub Copilot CLI
| OS | Path |
|----|------|
| macOS | `~/.copilot/mcp-config.json` |
| Linux | `~/.copilot/mcp-config.json` |
| Windows | `%USERPROFILE%\.copilot\mcp-config.json` |
| Format | JSON |
| Key | `mcpServers` |
| Install | https://docs.github.com/en/copilot/github-copilot-in-the-cli |

### Antigravity
| OS | Path |
|----|------|
| macOS | `~/.gemini/antigravity/mcp_config.json` |
| Linux | `~/.gemini/antigravity/mcp_config.json` |
| Windows | `%USERPROFILE%\.gemini\antigravity\mcp_config.json` |
| Format | JSON |
| Key | `mcpServers` |
| Install | https://antigravity.codes/ |

## Manual Configuration Agents

These agents require manual setup through their UI.

### ChatGPT Desktop
| | |
|----|------|
| Config | Through Settings > MCP Servers in the app UI |
| Install | https://openai.com/chatgpt/download/ |

### Goose
| | |
|----|------|
| Config | Through settings UI. Uses YAML format. |
| Install | https://github.com/block/goose |

## Sources

- [neondatabase/add-mcp](https://github.com/neondatabase/add-mcp) — cross-agent MCP config tool
- [Kiro MCP docs](https://kiro.dev/docs/mcp/configuration/)
- [LM Studio MCP docs](https://lmstudio.ai/docs/app/mcp)
- [Continue.dev MCP docs](https://docs.continue.dev/customize/deep-dives/mcp)
