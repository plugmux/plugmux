import { Trash2 } from "lucide-react";
import { AgentIcon } from "@/components/agents/AgentIcon";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { DetectedAgent } from "@/lib/commands";

const statusColor: Record<DetectedAgent["status"], string> = {
  green: "bg-green-500",
  yellow: "bg-yellow-500",
  gray: "bg-gray-500",
};

const statusTooltip: Record<DetectedAgent["status"], string> = {
  green: "Connected",
  yellow: "Connected (manually configured)",
  gray: "Not connected",
};

interface AgentTableProps {
  agents: DetectedAgent[];
  onConnect: (id: string) => void;
  onDisable: (agent: DetectedAgent) => void;
  onDelete: (agent: DetectedAgent) => void;
}

function AgentRow({
  agent,
  onConnect,
  onDisable,
  onDelete,
}: {
  agent: DetectedAgent;
  onConnect: (id: string) => void;
  onDisable: (agent: DetectedAgent) => void;
  onDelete: (agent: DetectedAgent) => void;
}) {
  const isConnected = agent.status === "green" || agent.status === "yellow";
  const isInstalled = agent.installed || agent.source === "custom";

  return (
    <div className="flex min-h-[52px] items-center gap-3 rounded-md border border-border px-3 py-2.5">
      <Tooltip>
        <TooltipTrigger asChild>
          <span
            className={`h-2 w-2 shrink-0 cursor-pointer rounded-full ${statusColor[agent.status]}`}
          />
        </TooltipTrigger>
        <TooltipContent side="top">
          <p>{statusTooltip[agent.status]}</p>
        </TooltipContent>
      </Tooltip>

      <span className={!isInstalled ? "opacity-40" : ""}>
        <AgentIcon icon={agent.icon} name={agent.name} />
      </span>

      <div className="min-w-0 flex-1">
        <p className={`text-sm font-medium ${!isInstalled ? "opacity-40" : ""}`}>{agent.name}</p>
        {isInstalled && agent.config_path && (
          <p className="truncate text-xs text-muted-foreground">
            {agent.config_path}
          </p>
        )}
      </div>

      {isInstalled && (
        <Switch
          checked={isConnected}
          onCheckedChange={(checked) => {
            if (checked) {
              onConnect(agent.id);
            } else {
              onDisable(agent);
            }
          }}
          className="data-[state=checked]:bg-primary"
        />
      )}

      {agent.source === "custom" && (
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 text-muted-foreground"
          onClick={() => onDelete(agent)}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      )}
    </div>
  );
}

export function AgentTable({
  agents,
  onConnect,
  onDisable,
  onDelete,
}: AgentTableProps) {
  const installed = agents.filter((a) => a.installed || a.source === "custom");
  const notInstalled = agents.filter((a) => !a.installed && a.source !== "custom");

  return (
    <TooltipProvider delayDuration={300}>
      <div className="space-y-6">
        {installed.length > 0 && (
          <div className="space-y-1">
            <p className="px-1 pb-1 text-xs font-medium uppercase text-muted-foreground">
              Configuration found
            </p>
            {installed.map((agent) => (
              <AgentRow
                key={agent.id}
                agent={agent}
                onConnect={onConnect}
                onDisable={onDisable}
                onDelete={onDelete}
              />
            ))}
          </div>
        )}

        {notInstalled.length > 0 && (
          <div className="space-y-1">
            <p className="px-1 pb-1 text-xs font-medium uppercase text-muted-foreground">
              Other supported agents
            </p>
            {notInstalled.map((agent) => (
              <AgentRow
                key={agent.id}
                agent={agent}
                onConnect={onConnect}
                onDisable={onDisable}
                onDelete={onDelete}
              />
            ))}
          </div>
        )}
      </div>
    </TooltipProvider>
  );
}
