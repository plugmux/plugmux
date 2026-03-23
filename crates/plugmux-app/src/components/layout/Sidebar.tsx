import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { useConfig } from "@/hooks/useConfig";

import terminalIcon from "@assets/icons/terminal.svg";
import mcpIcon from "@assets/icons/mcp.svg";
import layersIcon from "@assets/icons/layers.svg";
import plusIcon from "@assets/icons/plus.svg";
import settingsIcon from "@assets/icons/settings.svg";
import logsIcon from "@assets/icons/logs.svg";

function Icon({ src, className = "h-4 w-4" }: { src: string; className?: string }) {
  return <img src={src} alt="" className={`dark:invert ${className}`} />;
}

interface SidebarProps {
  activePage: string;
  onNavigate: (page: string) => void;
  onNewEnvironment: () => void;
}

export function Sidebar({
  activePage,
  onNavigate,
  onNewEnvironment,
}: SidebarProps) {
  const { config } = useConfig();

  return (
    <div className="flex h-full w-[220px] flex-col border-r bg-muted/30">
      <nav className="flex flex-1 flex-col gap-1 px-2 pt-4">
        <button
          onClick={() => onNavigate("agents")}
          className={cn(
            "flex items-center gap-2 rounded-md border-l-[3px] px-2 py-1.5 text-sm",
            activePage === "agents"
              ? "border-primary bg-accent text-accent-foreground"
              : "border-transparent text-muted-foreground hover:bg-accent hover:text-accent-foreground",
          )}
        >
          <Icon src={terminalIcon} />
          Agents
        </button>
        <button
          onClick={() => onNavigate("catalog")}
          className={cn(
            "flex items-center gap-2 rounded-md border-l-[3px] px-2 py-1.5 text-sm",
            activePage === "catalog"
              ? "border-primary bg-accent text-accent-foreground"
              : "border-transparent text-muted-foreground hover:bg-accent hover:text-accent-foreground",
          )}
        >
          <Icon src={mcpIcon} />
          MCP Servers
        </button>
        <button
          onClick={() => onNavigate("logs")}
          className={cn(
            "flex items-center gap-2 rounded-md border-l-[3px] px-2 py-1.5 text-sm",
            activePage === "logs"
              ? "border-primary bg-accent text-accent-foreground"
              : "border-transparent text-muted-foreground hover:bg-accent hover:text-accent-foreground",
          )}
        >
          <Icon src={logsIcon} />
          Logs
        </button>
        {/* Environments section */}
        <div className="mt-4">
          <div className="flex items-center justify-between pl-2 pr-1 pb-1">
            <span className="text-xs font-medium uppercase text-muted-foreground">
              Environments
            </span>
            <button
              className="inline-flex h-5 w-5 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              onClick={onNewEnvironment}
            >
              <Icon src={plusIcon} className="h-3 w-3" />
            </button>
          </div>

          {/* Global environment (always first, not removable) */}
          <button
            onClick={() => onNavigate("env:global")}
            className={cn(
              "flex w-full items-center justify-between rounded-md border-l-[3px] px-2 py-1.5 text-sm",
              activePage === "env:global"
                ? "border-primary bg-accent text-accent-foreground"
                : "border-transparent text-muted-foreground hover:bg-accent hover:text-accent-foreground",
            )}
          >
            <div className="flex items-center gap-2">
              <Icon src={layersIcon} />
              <span className="truncate">Global</span>
            </div>
          </button>

          {/* Other environments */}
          {config?.environments
            .filter((env) => env.id !== "global")
            .map((env) => {
              const page = `env:${env.id}`;
              const serverCount = env.servers.length;
              return (
                <button
                  key={env.id}
                  onClick={() => onNavigate(page)}
                  className={cn(
                    "flex w-full items-center justify-between rounded-md border-l-[3px] px-2 py-1.5 text-sm",
                    activePage === page
                      ? "border-primary bg-accent text-accent-foreground"
                      : "border-transparent text-muted-foreground hover:bg-accent hover:text-accent-foreground",
                  )}
                >
                  <div className="flex items-center gap-2">
                    <Icon src={layersIcon} />
                    <span className="truncate">{env.name}</span>
                  </div>
                  <Badge variant="secondary" className="h-5 px-1.5 text-xs">
                    {serverCount}
                  </Badge>
                </button>
              );
            })}
        </div>

        {/* Settings at bottom */}
        <div className="mt-auto pb-2">
          <button
            onClick={() => onNavigate("settings")}
            className={cn(
              "flex w-full items-center gap-2 rounded-md border-l-[3px] px-2 py-1.5 text-sm",
              activePage === "settings"
                ? "border-primary bg-accent text-accent-foreground"
                : "border-transparent text-muted-foreground hover:bg-accent hover:text-accent-foreground",
            )}
          >
            <Icon src={settingsIcon} />
            Settings
          </button>
        </div>
      </nav>
    </div>
  );
}
