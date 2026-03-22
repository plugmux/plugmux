import {
  Layers,
  BookOpen,
  LayoutDashboard,
  Settings,
  Plus,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useConfig } from "@/hooks/useConfig";

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
      <nav className="flex flex-1 flex-col gap-1 px-2 pt-3">
        <button
          onClick={() => onNavigate("dashboard")}
          className={cn(
            "flex items-center gap-2 rounded-md px-2 py-1.5 text-sm",
            activePage === "dashboard"
              ? "bg-accent text-accent-foreground"
              : "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
          )}
        >
          <LayoutDashboard className="h-4 w-4" />
          Dashboard
        </button>
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
        {/* Environments section */}
        <div className="mt-4">
          <div className="flex items-center justify-between pl-2 pr-1 pb-1">
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

          {/* Default environment */}
          <button
            onClick={() => onNavigate("env:default")}
            className={cn(
              "flex w-full items-center justify-between rounded-md px-2 py-1.5 text-sm",
              activePage === "env:default"
                ? "bg-accent text-accent-foreground"
                : "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
            )}
          >
            <div className="flex items-center gap-2">
              <Layers className="h-4 w-4" />
              <span className="truncate">Default</span>
            </div>
          </button>

          {/* Other environments */}
          {config?.environments
            .filter((env) => env.id !== "default")
            .map((env) => {
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
