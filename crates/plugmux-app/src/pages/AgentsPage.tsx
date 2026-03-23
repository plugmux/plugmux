import { useState } from "react";
import { Plus, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { AgentTable } from "@/components/agents/AgentTable";
import { AddAgentDialog } from "@/components/agents/AddAgentDialog";
import { InstallDialog } from "@/components/agents/InstallDialog";
import { ManualSetupDialog } from "@/components/agents/ManualSetupDialog";
import { useAgents } from "@/hooks/useAgents";
import type { DetectedAgent } from "@/lib/commands";

export function AgentsPage() {
  const { agents, loading, connect, disconnect, dismiss, reload } =
    useAgents();

  const [addAgentOpen, setAddAgentOpen] = useState(false);
  const [installAgent, setInstallAgent] = useState<DetectedAgent | null>(null);
  const [manualAgent, setManualAgent] = useState<DetectedAgent | null>(null);

  function handleDisable(agent: DetectedAgent) {
    disconnect(agent.id, false);
  }

  function handleDelete(agent: DetectedAgent) {
    if (agent.status === "green" || agent.status === "yellow") {
      disconnect(agent.id, false);
    } else {
      dismiss(agent.id);
    }
  }

  if (loading) {
    return null;
  }

  return (
    <div className="space-y-6 px-6 pt-4 pb-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <h2 className="text-lg font-semibold">Agents</h2>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 text-muted-foreground"
            onClick={reload}
          >
            <RefreshCw className="h-3.5 w-3.5" />
          </Button>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setAddAgentOpen(true)}
        >
          <Plus className="h-4 w-4" />
          Add custom agent
        </Button>
      </div>

      <AgentTable
        agents={agents}
        onConnect={connect}
        onDisable={handleDisable}
        onDelete={handleDelete}
        onInstall={setInstallAgent}
        onManualSetup={setManualAgent}
      />

      <AddAgentDialog
        open={addAgentOpen}
        onOpenChange={setAddAgentOpen}
        onAdded={reload}
      />

      <InstallDialog
        open={installAgent !== null}
        onOpenChange={(open) => {
          if (!open) setInstallAgent(null);
        }}
        agent={installAgent}
      />

      <ManualSetupDialog
        open={manualAgent !== null}
        onOpenChange={(open) => {
          if (!open) setManualAgent(null);
        }}
        agent={manualAgent}
      />
    </div>
  );
}
