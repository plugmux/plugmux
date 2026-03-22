// Import all available icons - prefer color, fallback to mono
// For color icons:
import antigravityColor from "@/assets/agent-icons/antigravity-color.svg";
import cherrystudioColor from "@/assets/agent-icons/cherrystudio-color.svg";
import claudecodeColor from "@/assets/agent-icons/claudecode-color.svg";
import codexColor from "@/assets/agent-icons/codex-color.svg";
import copilotColor from "@/assets/agent-icons/copilot-color.svg";
import difyColor from "@/assets/agent-icons/dify-color.svg";
import geminiColor from "@/assets/agent-icons/gemini-color.svg";
import junieColor from "@/assets/agent-icons/junie-color.svg";
import monicaColor from "@/assets/agent-icons/monica-color.svg";
import n8nColor from "@/assets/agent-icons/n8n-color.svg";
import openhandsColor from "@/assets/agent-icons/openhands-color.svg";
import poeColor from "@/assets/agent-icons/poe-color.svg";
import traeColor from "@/assets/agent-icons/trae-color.svg";

// For mono icons (used when no color variant exists):
import cursor from "@/assets/agent-icons/cursor.svg";
import windsurf from "@/assets/agent-icons/windsurf.svg";
import githubcopilot from "@/assets/agent-icons/githubcopilot.svg";
import cline from "@/assets/agent-icons/cline.svg";
import roocode from "@/assets/agent-icons/roocode.svg";
import goose from "@/assets/agent-icons/goose.svg";
import opencode from "@/assets/agent-icons/opencode.svg";
import lmstudio from "@/assets/agent-icons/lmstudio.svg";
import openwebui from "@/assets/agent-icons/openwebui.svg";
import coze from "@/assets/agent-icons/coze.svg";

// Map icon field name → imported asset (prefer color)
const icons: Record<string, string> = {
  antigravity: antigravityColor,
  cherrystudio: cherrystudioColor,
  claudecode: claudecodeColor,
  codex: codexColor,
  copilot: copilotColor,
  dify: difyColor,
  gemini: geminiColor,
  junie: junieColor,
  monica: monicaColor,
  n8n: n8nColor,
  openhands: openhandsColor,
  poe: poeColor,
  trae: traeColor,
  cursor: cursor,
  windsurf: windsurf,
  githubcopilot: githubcopilot,
  cline: cline,
  roocode: roocode,
  goose: goose,
  opencode: opencode,
  lmstudio: lmstudio,
  openwebui: openwebui,
  coze: coze,
};

interface AgentIconProps {
  icon: string | null;
  name: string;
  className?: string;
}

export function AgentIcon({ icon, name, className = "h-5 w-5" }: AgentIconProps) {
  if (icon && icons[icon]) {
    // Monochrome icons need the invert filter for dark mode
    const needsFilter = ["cursor", "windsurf", "githubcopilot", "cline", "roocode", "goose", "opencode", "lmstudio", "openwebui", "coze"].includes(icon);
    return (
      <img
        src={icons[icon]}
        alt={name}
        className={className}
        style={needsFilter ? { filter: "brightness(0) invert(1)" } : undefined}
      />
    );
  }

  // 2-letter thumbnail fallback
  const initials = name.slice(0, 2).toUpperCase();
  // Generate consistent color from name hash
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = name.charCodeAt(i) + ((hash << 5) - hash);
  }
  const hue = Math.abs(hash) % 360;
  const bg = `hsl(${hue}, 50%, 30%)`;

  return (
    <div
      className={`flex items-center justify-center rounded-full text-white text-[10px] font-bold ${className}`}
      style={{ backgroundColor: bg }}
    >
      {initials}
    </div>
  );
}
