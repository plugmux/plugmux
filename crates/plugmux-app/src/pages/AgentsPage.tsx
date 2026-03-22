import { useState } from "react";
import { Cable, ArrowRightIcon, Plus } from "lucide-react";
import { Banner } from "@/components/ui/banner";
import { Button } from "@/components/ui/button";
import { AgentTable } from "@/components/agents/AgentTable";
import { DisableDialog } from "@/components/agents/DisableDialog";
import { SetupDialog } from "@/components/agents/SetupDialog";
import { AddAgentDialog } from "@/components/agents/AddAgentDialog";
import { useAgents } from "@/hooks/useAgents";
import { hasAgentBackup, type DetectedAgent } from "@/lib/commands";

export function AgentsPage() {
  const { agents, loading, hasConnected, connect, disconnect, dismiss, reload } =
    useAgents();

  // Setup & AddAgent dialogs
  const [setupOpen, setSetupOpen] = useState(false);
  const [addAgentOpen, setAddAgentOpen] = useState(false);

  // Disable dialog state
  const [disableAgent, setDisableAgent] = useState<DetectedAgent | null>(null);
  const [hasBackup, setHasBackup] = useState(false);

  async function openDisableDialog(agent: DetectedAgent) {
    const backup = await hasAgentBackup(agent.id);
    setHasBackup(backup);
    setDisableAgent(agent);
  }

  function closeDisableDialog() {
    setDisableAgent(null);
    setHasBackup(false);
  }

  async function handleDisable() {
    if (!disableAgent) return;
    await disconnect(disableAgent.id, false);
    closeDisableDialog();
  }

  async function handleDisableAndRestore() {
    if (!disableAgent) return;
    await disconnect(disableAgent.id, true);
    closeDisableDialog();
  }

  function handleConnect(id: string) {
    connect(id);
  }

  function handleDelete(agent: DetectedAgent) {
    if (agent.status === "green" || agent.status === "yellow") {
      // Agent is connected — open disable dialog first
      openDisableDialog(agent);
    } else {
      dismiss(agent.id);
    }
  }

  if (loading) {
    return null;
  }

  return (
    <div className="space-y-6 p-6">
      {/* Onboarding banner */}
      {!hasConnected && (
        <Banner
          show={true}
          variant="premium"
          title="Connect your code agents"
          description="To start using plugmux, make plugmux MCP available to your agents."
          showShade={true}
          closable={false}
          icon={<Cable />}
          action={
            <Button
              onClick={() => setSetupOpen(true)}
              variant="ghost"
              className="inline-flex items-center gap-1 rounded-md bg-black/10 px-3 py-1.5 text-sm font-medium transition-colors hover:bg-black/20 dark:bg-white/10 dark:hover:bg-white/20"
            >
              Setup
              <ArrowRightIcon className="h-3 w-3" />
            </Button>
          }
        />
      )}

      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-semibold">Agents</h2>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setAddAgentOpen(true)}
        >
          <Plus className="h-4 w-4" />
          Add agent
        </Button>
      </div>

      {/* Agent table */}
      <AgentTable
        agents={agents}
        onConnect={handleConnect}
        onDisable={openDisableDialog}
        onDelete={handleDelete}
      />

      {/* Disable dialog */}
      <DisableDialog
        agent={disableAgent}
        open={disableAgent !== null}
        onOpenChange={(open) => {
          if (!open) closeDisableDialog();
        }}
        onDisable={handleDisable}
        onDisableAndRestore={handleDisableAndRestore}
        hasBackup={hasBackup}
      />

      {/* Setup dialog */}
      <SetupDialog
        open={setupOpen}
        onOpenChange={setSetupOpen}
        onComplete={reload}
      />

      {/* Add agent dialog */}
      <AddAgentDialog
        open={addAgentOpen}
        onOpenChange={setAddAgentOpen}
        existingAgentIds={agents.map((a) => a.id)}
        onAdded={reload}
      />
    </div>
  );
}
