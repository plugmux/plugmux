import { useState } from "react";
import { Plus, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { AgentTable } from "@/components/agents/AgentTable";
import { AddAgentDialog } from "@/components/agents/AddAgentDialog";
import { useAgents } from "@/hooks/useAgents";

export function AgentsPage() {
  const { agents, loading, connect, disconnect, dismiss, reload } =
    useAgents();

  const [addAgentOpen, setAddAgentOpen] = useState(false);

  function handleConnect(id: string) {
    connect(id);
  }

  function handleDisable(agent: { id: string }) {
    disconnect(agent.id, false);
  }

  function handleDelete(agent: { id: string; status: string }) {
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
      {/* Header */}
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

      {/* Agent table */}
      <AgentTable
        agents={agents}
        onConnect={handleConnect}
        onDisable={handleDisable}
        onDelete={handleDelete}
      />

      {/* Add custom agent dialog */}
      <AddAgentDialog
        open={addAgentOpen}
        onOpenChange={setAddAgentOpen}
        onAdded={reload}
      />
    </div>
  );
}
