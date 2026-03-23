import { Settings2, Trash2 } from "lucide-react";
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
  onManualSetup: (agent: DetectedAgent) => void;
}

function AgentRow({
  agent,
  onConnect,
  onDisable,
  onDelete,
  onManualSetup,
}: {
  agent: DetectedAgent;
  onConnect: (id: string) => void;
  onDisable: (agent: DetectedAgent) => void;
  onDelete: (agent: DetectedAgent) => void;
  onManualSetup: (agent: DetectedAgent) => void;
}) {
  const isConnected = agent.status === "green" || agent.status === "yellow";
  const isInstalled = agent.installed || agent.source === "custom";
  const isManual = !isInstalled && agent.source !== "custom";

  return (
    <div className="flex min-h-[52px] items-center gap-3 rounded-md border border-border px-3 py-2.5">
      {/* Status dot */}
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

      {/* Agent icon */}
      <span className={isManual ? "opacity-40" : ""}>
        <AgentIcon icon={agent.icon} name={agent.name} />
      </span>

      {/* Name + config path */}
      <div className="min-w-0 flex-1">
        <p className={`text-sm font-medium ${isManual ? "opacity-40" : ""}`}>
          {agent.name}
        </p>
        {isInstalled && agent.config_path && (
          <p className="truncate text-xs text-muted-foreground">
            {agent.config_path}
          </p>
        )}
      </div>

      {/* Right side actions */}
      {isInstalled && (
        <>
          {agent.source === "custom" && (
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7 shrink-0 text-muted-foreground hover:text-destructive"
              onClick={() => onDelete(agent)}
            >
              <Trash2 className="h-3.5 w-3.5" />
            </Button>
          )}
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
        </>
      )}

      {isManual && (
        <Button
          variant="outline"
          size="sm"
          className="shrink-0 text-xs"
          onClick={() => onManualSetup(agent)}
        >
          <Settings2 className="mr-1 h-3 w-3" />
          Setup
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
  onManualSetup,
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
                onManualSetup={onManualSetup}
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
                onManualSetup={onManualSetup}
              />
            ))}
          </div>
        )}
      </div>
    </TooltipProvider>
  );
}
