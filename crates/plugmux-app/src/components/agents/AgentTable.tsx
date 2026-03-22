import { Trash2 } from "lucide-react";
import { AgentIcon } from "@/components/agents/AgentIcon";
import { Button } from "@/components/ui/button";
import type { DetectedAgent } from "@/lib/commands";

const statusColor: Record<DetectedAgent["status"], string> = {
  green: "bg-green-500",
  yellow: "bg-yellow-500",
  gray: "bg-gray-500",
};

interface AgentTableProps {
  agents: DetectedAgent[];
  onConnect: (id: string) => void;
  onDisable: (agent: DetectedAgent) => void;
  onDelete: (agent: DetectedAgent) => void;
}

export function AgentTable({
  agents,
  onConnect,
  onDisable,
  onDelete,
}: AgentTableProps) {
  return (
    <div className="space-y-1">
      {agents.map((agent) => (
        <div
          key={agent.id}
          className="flex items-center gap-3 rounded-md px-3 py-2.5 hover:bg-accent"
        >
          {/* Status dot */}
          <span
            className={`h-2 w-2 shrink-0 rounded-full ${statusColor[agent.status]}`}
          />

          {/* Agent icon */}
          <AgentIcon icon={agent.icon} name={agent.name} />

          {/* Name + config path */}
          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium">{agent.name}</p>
            <p className="truncate text-xs text-muted-foreground">
              {agent.config_path ?? "Not configured"}
            </p>
          </div>

          {/* Enable / Disable toggle */}
          {agent.status === "gray" ? (
            <Button
              variant="outline"
              size="sm"
              onClick={() => onConnect(agent.id)}
            >
              Connect
            </Button>
          ) : (
            <Button
              variant="secondary"
              size="sm"
              className="text-xs"
              onClick={() => onDisable(agent)}
            >
              Enabled
            </Button>
          )}

          {/* Delete */}
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 text-muted-foreground"
            onClick={() => onDelete(agent)}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      ))}
    </div>
  );
}
