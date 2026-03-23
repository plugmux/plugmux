import antigravity from "@assets/agent-icons/antigravity.svg";
import chatgpt from "@assets/agent-icons/chatgpt.svg";
import cherrystudio from "@assets/agent-icons/cherrystudio.svg";
import claude from "@assets/agent-icons/claude.svg";
import claudecode from "@assets/agent-icons/claudecode.svg";
import cline from "@assets/agent-icons/cline.svg";
import codex from "@assets/agent-icons/codex.svg";
import copilot from "@assets/agent-icons/copilot.svg";
import coze from "@assets/agent-icons/coze.svg";
import cursor from "@assets/agent-icons/cursor.svg";
import dify from "@assets/agent-icons/dify.svg";
import gemini from "@assets/agent-icons/gemini.svg";
import githubcopilot from "@assets/agent-icons/githubcopilot.svg";
import goose from "@assets/agent-icons/goose.svg";
import junie from "@assets/agent-icons/junie.svg";
import lmstudio from "@assets/agent-icons/lmstudio.svg";
import monica from "@assets/agent-icons/monica.svg";
import n8n from "@assets/agent-icons/n8n.svg";
import opencode from "@assets/agent-icons/opencode.svg";
import openhands from "@assets/agent-icons/openhands.svg";
import openwebui from "@assets/agent-icons/openwebui.svg";
import poe from "@assets/agent-icons/poe.svg";
import roocode from "@assets/agent-icons/roocode.svg";
import trae from "@assets/agent-icons/trae.svg";
import vscode from "@assets/agent-icons/vscode.svg";
import windsurf from "@assets/agent-icons/windsurf.svg";
import zed from "@assets/agent-icons/zed.svg";

const icons: Record<string, string> = {
  antigravity, chatgpt, cherrystudio, claude, claudecode, cline, codex, copilot, coze,
  cursor, dify, gemini, githubcopilot, goose, junie, lmstudio, monica,
  n8n, opencode, openhands, openwebui, poe, roocode, trae, vscode, windsurf, zed,
};

interface AgentIconProps {
  icon: string | null;
  name: string;
  className?: string;
}

export function AgentIcon({ icon, name, className = "h-5 w-5" }: AgentIconProps) {
  if (icon && icons[icon]) {
    return (
      <img
        src={icons[icon]}
        alt={name}
        className={`dark:invert ${className}`}
      />
    );
  }

  const initials = name.slice(0, 2).toUpperCase();

  return (
    <div
      className={`flex items-center justify-center rounded-md bg-foreground text-background text-[10px] font-bold dark:bg-white dark:text-black ${className}`}
      style={{ width: 24, height: 24, minWidth: 24, minHeight: 24 }}
    >
      {initials}
    </div>
  );
}
