import {
  Server,
  Layers,
  BookOpen,
  LayoutTemplate,
  Settings,
  Plus,
  Circle,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useConfig } from "@/hooks/useConfig";
import { useEngine } from "@/hooks/useEngine";

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
  const { status } = useEngine();

  const statusColor =
    status === "running"
      ? "text-green-500"
      : status === "conflict"
        ? "text-yellow-500"
        : "text-muted-foreground";

  return (
    <div className="flex h-full w-[220px] flex-col border-r bg-muted/30">
      {/* Header */}
      <div className="flex items-center gap-2 px-4 py-4">
        <Circle className={cn("h-3 w-3 fill-current", statusColor)} />
        <span className="text-sm font-semibold">plugmux</span>
      </div>

      {/* Navigation */}
      <nav className="flex flex-1 flex-col gap-1 px-2">
        {/* Main */}
        <button
          onClick={() => onNavigate("main")}
          className={cn(
            "flex items-center gap-2 rounded-md px-2 py-1.5 text-sm",
            activePage === "main"
              ? "bg-accent text-accent-foreground"
              : "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
          )}
        >
          <Server className="h-4 w-4" />
          Main
        </button>

        {/* Environments section */}
        <div className="mt-4">
          <div className="flex items-center justify-between px-2 pb-1">
            <span className="text-xs font-medium uppercase text-muted-foreground">
              Environments
            </span>
            <Button
              variant="ghost"
              size="icon"
              className="h-5 w-5"
              onClick={onNewEnvironment}
            >
              <Plus className="h-3 w-3" />
            </Button>
          </div>

          {config?.environments.map((env) => {
            const page = `env:${env.id}`;
            const serverCount = env.servers.length;
            return (
              <button
                key={env.id}
                onClick={() => onNavigate(page)}
                className={cn(
                  "flex w-full items-center justify-between rounded-md px-2 py-1.5 text-sm",
                  activePage === page
                    ? "bg-accent text-accent-foreground"
                    : "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
                )}
              >
                <div className="flex items-center gap-2">
                  <Layers className="h-4 w-4" />
                  <span className="truncate">{env.name}</span>
                </div>
                <Badge variant="secondary" className="ml-1 h-5 px-1.5 text-xs">
                  {serverCount}
                </Badge>
              </button>
            );
          })}
        </div>

        {/* Phase 3 placeholders */}
        <div className="mt-4 flex flex-col gap-1">
          <button
            onClick={() => onNavigate("catalog")}
            className={cn(
              "flex items-center gap-2 rounded-md px-2 py-1.5 text-sm",
              activePage === "catalog"
                ? "bg-accent text-accent-foreground"
                : "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
            )}
          >
            <BookOpen className="h-4 w-4" />
            Catalog
          </button>
          <button
            onClick={() => onNavigate("presets")}
            className={cn(
              "flex items-center gap-2 rounded-md px-2 py-1.5 text-sm",
              activePage === "presets"
                ? "bg-accent text-accent-foreground"
                : "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
            )}
          >
            <LayoutTemplate className="h-4 w-4" />
            Presets
          </button>
        </div>

        {/* Settings at bottom */}
        <div className="mt-auto pb-2">
          <button
            onClick={() => onNavigate("settings")}
            className={cn(
              "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm",
              activePage === "settings"
                ? "bg-accent text-accent-foreground"
                : "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
            )}
          >
            <Settings className="h-4 w-4" />
            Settings
          </button>
        </div>
      </nav>
    </div>
  );
}
