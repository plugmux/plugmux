import { Circle } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";
import type { ServerConfig } from "@/lib/commands";

interface ServerOverride {
  server_id: string;
  enabled?: boolean;
}

interface InheritedServersProps {
  servers: ServerConfig[];
  overrides: ServerOverride[];
  onToggleOverride: (serverId: string) => void;
}

export function InheritedServers({
  servers,
  overrides,
  onToggleOverride,
}: InheritedServersProps) {
  function isOverridden(serverId: string): boolean {
    const override = overrides.find((o) => o.server_id === serverId);
    return override?.enabled === false;
  }

  return (
    <div>
      <div className="mb-3 flex items-center gap-2">
        <h2 className="text-lg font-semibold">Inherited Servers</h2>
        <Badge variant="secondary">{servers.length}</Badge>
      </div>

      {servers.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          No servers in Main configuration.
        </p>
      ) : (
        <div className="grid gap-2">
          {servers.map((server) => {
            const overridden = isOverridden(server.id);
            return (
              <div
                key={server.id}
                className={cn(
                  "flex items-center gap-3 rounded-lg border px-4 py-3 transition-opacity",
                  overridden && "opacity-40",
                )}
              >
                <Circle
                  className={cn(
                    "h-3 w-3 shrink-0 fill-current",
                    !overridden
                      ? "text-green-500"
                      : "text-muted-foreground",
                  )}
                />

                <div className="flex min-w-0 flex-1 items-center gap-2">
                  <span
                    className={cn(
                      "truncate font-medium",
                      overridden && "line-through",
                    )}
                  >
                    {server.name}
                  </span>
                  <Badge variant="outline" className="shrink-0 text-xs">
                    from Main
                  </Badge>
                </div>

                <Switch
                  checked={!overridden}
                  onCheckedChange={() => onToggleOverride(server.id)}
                />
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
