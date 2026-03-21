import { Circle, Trash2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";
import type { ServerConfig } from "@/lib/commands";

interface ServerCardProps {
  server: ServerConfig;
  onToggle: () => void;
  onRemove: () => void;
}

export function ServerCard({ server, onToggle, onRemove }: ServerCardProps) {
  return (
    <div
      className={cn(
        "flex items-center gap-3 rounded-lg border px-4 py-3 transition-opacity",
        !server.enabled && "opacity-50",
      )}
    >
      <Circle
        className={cn(
          "h-3 w-3 shrink-0 fill-current",
          server.enabled ? "text-green-500" : "text-muted-foreground",
        )}
      />

      <div className="flex min-w-0 flex-1 items-center gap-2">
        <span className="truncate font-medium">{server.name}</span>
        <Badge
          variant="outline"
          className={cn(
            "shrink-0",
            server.connectivity === "local"
              ? "border-green-500/50 text-green-600"
              : "border-blue-500/50 text-blue-600",
          )}
        >
          {server.connectivity}
        </Badge>
      </div>

      {server.description && (
        <span className="hidden truncate text-sm text-muted-foreground md:block md:max-w-[200px]">
          {server.description}
        </span>
      )}

      <div className="flex shrink-0 items-center gap-2">
        <Switch checked={server.enabled} onCheckedChange={onToggle} />
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 text-muted-foreground hover:text-destructive"
          onClick={onRemove}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
