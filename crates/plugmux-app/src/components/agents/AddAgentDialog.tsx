import { useEffect, useState } from "react";
import { Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import { AgentIcon } from "./AgentIcon";
import {
  getAgentRegistry,
  addAgentFromRegistry,
  addCustomAgent,
  type AgentEntry,
} from "@/lib/commands";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

type Tab = "agents" | "custom";

interface AddAgentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  existingAgentIds: string[];
  onAdded: () => void;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function AddAgentDialog({
  open,
  onOpenChange,
  existingAgentIds,
  onAdded,
}: AddAgentDialogProps) {
  const [tab, setTab] = useState<Tab>("agents");

  // ---- Registry tab state ----
  const [registry, setRegistry] = useState<AgentEntry[]>([]);
  const [loadingRegistry, setLoadingRegistry] = useState(false);
  const [selectedAgent, setSelectedAgent] = useState<AgentEntry | null>(null);
  const [registryConfigPath, setRegistryConfigPath] = useState("");
  const [addingFromRegistry, setAddingFromRegistry] = useState(false);

  // ---- Custom tab state ----
  const [customName, setCustomName] = useState("");
  const [customConfigPath, setCustomConfigPath] = useState("");
  const [customFormat, setCustomFormat] = useState<"json" | "toml">("json");
  const [customMcpKey, setCustomMcpKey] = useState("mcpServers");
  const [addingCustom, setAddingCustom] = useState(false);

  // Reset all state when dialog opens/closes
  useEffect(() => {
    if (open) {
      setTab("agents");
      setSelectedAgent(null);
      setRegistryConfigPath("");
      setCustomName("");
      setCustomConfigPath("");
      setCustomFormat("json");
      setCustomMcpKey("mcpServers");
    }
  }, [open]);

  // Load registry when dialog opens
  useEffect(() => {
    if (!open) return;
    setLoadingRegistry(true);
    getAgentRegistry()
      .then((entries) => {
        setRegistry(entries.filter((e) => !existingAgentIds.includes(e.id)));
      })
      .catch(() => setRegistry([]))
      .finally(() => setLoadingRegistry(false));
  }, [open]);

  // -------------------------------------------------------------------------
  // Registry tab handlers
  // -------------------------------------------------------------------------

  function handleSelectAgent(agent: AgentEntry) {
    setSelectedAgent(agent);
    // Pre-fill config path from registry defaults (macOS)
    const defaultPath = agent.config_paths?.macos ?? "";
    setRegistryConfigPath(defaultPath);
  }

  function handleBackToList() {
    setSelectedAgent(null);
    setRegistryConfigPath("");
  }

  async function handleAddFromRegistry() {
    if (!selectedAgent || !registryConfigPath.trim()) return;
    setAddingFromRegistry(true);
    try {
      await addAgentFromRegistry(selectedAgent.id, registryConfigPath.trim());
      onAdded();
      onOpenChange(false);
    } catch {
      // stay open on error so user can correct
    } finally {
      setAddingFromRegistry(false);
    }
  }

  // -------------------------------------------------------------------------
  // Custom tab handlers
  // -------------------------------------------------------------------------

  async function handleAddCustom() {
    if (!customName.trim() || !customConfigPath.trim()) return;
    setAddingCustom(true);
    try {
      await addCustomAgent(
        customName.trim(),
        customConfigPath.trim(),
        customFormat,
        customMcpKey.trim() || "mcpServers",
      );
      onAdded();
      onOpenChange(false);
    } catch {
      // stay open on error so user can correct
    } finally {
      setAddingCustom(false);
    }
  }

  // -------------------------------------------------------------------------
  // Render
  // -------------------------------------------------------------------------

  const availableRegistry = registry;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Add Agent</DialogTitle>
          <DialogDescription>
            Choose an agent from the registry or add a custom one.
          </DialogDescription>
        </DialogHeader>

        {/* Tab switcher */}
        <div className="flex gap-1 rounded-md border border-border bg-muted/40 p-1">
          <button
            type="button"
            onClick={() => { setTab("agents"); setSelectedAgent(null); setRegistryConfigPath(""); }}
            className={[
              "flex-1 rounded px-3 py-1.5 text-sm font-medium transition-colors",
              tab === "agents"
                ? "bg-background text-foreground shadow-sm"
                : "text-muted-foreground hover:text-foreground",
            ].join(" ")}
          >
            Agents
          </button>
          <button
            type="button"
            onClick={() => setTab("custom")}
            className={[
              "flex-1 rounded px-3 py-1.5 text-sm font-medium transition-colors",
              tab === "custom"
                ? "bg-background text-foreground shadow-sm"
                : "text-muted-foreground hover:text-foreground",
            ].join(" ")}
          >
            Custom
          </button>
        </div>

        {/* ---- Tab: Agents ---- */}
        {tab === "agents" && (
          <>
            {loadingRegistry ? (
              <div className="flex items-center justify-center py-10">
                <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                <span className="ml-2 text-sm text-muted-foreground">
                  Loading registry...
                </span>
              </div>
            ) : selectedAgent ? (
              /* Config path prompt */
              <div className="space-y-4 py-2">
                <div className="flex items-center gap-3">
                  <AgentIcon
                    icon={selectedAgent.icon}
                    name={selectedAgent.name}
                    className="h-8 w-8"
                  />
                  <div>
                    <p className="text-sm font-semibold">{selectedAgent.name}</p>
                    <p className="text-xs text-muted-foreground">
                      Enter the path to the agent's config file
                    </p>
                  </div>
                </div>
                <Input
                  placeholder="Path to config file"
                  value={registryConfigPath}
                  onChange={(e) => setRegistryConfigPath(e.target.value)}
                  autoFocus
                />
              </div>
            ) : availableRegistry.length === 0 ? (
              <p className="py-8 text-center text-sm text-muted-foreground">
                All registry agents are already in your list.
              </p>
            ) : (
              /* Agent list */
              <ScrollArea className="max-h-64">
                <div className="space-y-0.5 py-1 pr-3">
                  {availableRegistry.map((agent) => (
                    <button
                      key={agent.id}
                      type="button"
                      onClick={() => handleSelectAgent(agent)}
                      className="flex w-full items-center gap-3 rounded-md px-3 py-2.5 text-left transition-colors hover:bg-accent"
                    >
                      <AgentIcon
                        icon={agent.icon}
                        name={agent.name}
                        className="h-6 w-6 shrink-0"
                      />
                      <span className="text-sm font-medium">{agent.name}</span>
                    </button>
                  ))}
                </div>
              </ScrollArea>
            )}

            <DialogFooter className={selectedAgent ? "sm:justify-between" : "sm:justify-end"}>
              {selectedAgent && (
                <Button
                  type="button"
                  variant="ghost"
                  onClick={handleBackToList}
                  disabled={addingFromRegistry}
                >
                  Back
                </Button>
              )}
              <div className="flex gap-2">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => onOpenChange(false)}
                >
                  Cancel
                </Button>
                {selectedAgent && (
                  <Button
                    type="button"
                    onClick={handleAddFromRegistry}
                    disabled={
                      addingFromRegistry || !registryConfigPath.trim()
                    }
                  >
                    {addingFromRegistry && (
                      <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
                    )}
                    Add
                  </Button>
                )}
              </div>
            </DialogFooter>
          </>
        )}

        {/* ---- Tab: Custom ---- */}
        {tab === "custom" && (
          <>
            <div className="space-y-4 py-2">
              {/* Name */}
              <div className="space-y-1.5">
                <label
                  htmlFor="custom-agent-name"
                  className="text-sm font-medium"
                >
                  Name
                </label>
                <Input
                  id="custom-agent-name"
                  placeholder="e.g. My Custom Agent"
                  value={customName}
                  onChange={(e) => setCustomName(e.target.value)}
                />
              </div>

              {/* Config file path */}
              <div className="space-y-1.5">
                <label
                  htmlFor="custom-config-path"
                  className="text-sm font-medium"
                >
                  Config file path
                </label>
                <Input
                  id="custom-config-path"
                  placeholder="e.g. ~/.config/myagent/config.json"
                  value={customConfigPath}
                  onChange={(e) => setCustomConfigPath(e.target.value)}
                />
              </div>

              {/* Config format toggle */}
              <div className="space-y-1.5">
                <span className="text-sm font-medium">Config format</span>
                <div className="flex gap-1.5">
                  <button
                    type="button"
                    onClick={() => setCustomFormat("json")}
                    className={[
                      "rounded-md border px-3 py-1.5 text-sm font-medium transition-colors",
                      customFormat === "json"
                        ? "border-primary bg-primary/10 text-primary"
                        : "border-border text-muted-foreground hover:border-foreground/40 hover:text-foreground",
                    ].join(" ")}
                  >
                    JSON
                  </button>
                  <button
                    type="button"
                    onClick={() => setCustomFormat("toml")}
                    className={[
                      "rounded-md border px-3 py-1.5 text-sm font-medium transition-colors",
                      customFormat === "toml"
                        ? "border-primary bg-primary/10 text-primary"
                        : "border-border text-muted-foreground hover:border-foreground/40 hover:text-foreground",
                    ].join(" ")}
                  >
                    TOML
                  </button>
                </div>
              </div>

              {/* MCP key */}
              <div className="space-y-1.5">
                <label
                  htmlFor="custom-mcp-key"
                  className="text-sm font-medium"
                >
                  MCP key
                </label>
                <Input
                  id="custom-mcp-key"
                  placeholder="mcpServers"
                  value={customMcpKey}
                  onChange={(e) => setCustomMcpKey(e.target.value)}
                />
              </div>
            </div>

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
              >
                Cancel
              </Button>
              <Button
                type="button"
                onClick={handleAddCustom}
                disabled={
                  addingCustom ||
                  !customName.trim() ||
                  !customConfigPath.trim()
                }
              >
                {addingCustom && (
                  <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
                )}
                Add
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
