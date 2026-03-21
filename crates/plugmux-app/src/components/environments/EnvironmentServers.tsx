import { Badge } from "@/components/ui/badge";
import { ServerCard } from "@/components/servers/ServerCard";
import type { ServerConfig } from "@/lib/commands";

interface EnvironmentServersProps {
  servers: ServerConfig[];
  onToggle: (serverId: string) => void;
  onRemove: (serverId: string) => void;
}

export function EnvironmentServers({
  servers,
  onToggle,
  onRemove,
}: EnvironmentServersProps) {
  return (
    <div>
      <div className="mb-3 flex items-center gap-2">
        <h2 className="text-lg font-semibold">Environment Servers</h2>
        <Badge variant="secondary">{servers.length}</Badge>
      </div>

      {servers.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          No environment-specific servers. Add one to extend this environment.
        </p>
      ) : (
        <div className="grid gap-2">
          {servers.map((server) => (
            <ServerCard
              key={server.id}
              server={server}
              onToggle={() => onToggle(server.id)}
              onRemove={() => onRemove(server.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
