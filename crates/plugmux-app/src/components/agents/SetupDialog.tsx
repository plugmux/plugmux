import { useEffect, useState } from "react";
import {
  Loader2,
  Zap,
  Code,
  ArrowLeft,
  Check,
  X,
  Copy,
  CheckCircle2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
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
  detectAgents,
  connectAgent,
  getAgentRegistry,
  getPort,
  type DetectedAgent,
  type AgentEntry,
} from "@/lib/commands";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

type Step = "choose" | "auto" | "manual";

interface ConnectResult {
  id: string;
  name: string;
  icon: string | null;
  success: boolean;
  error?: string;
}

interface SetupDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete: () => void;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function SetupDialog({
  open,
  onOpenChange,
  onComplete,
}: SetupDialogProps) {
  const [step, setStep] = useState<Step>("choose");

  // Auto-connect state
  const [scanning, setScanning] = useState(false);
  const [agents, setAgents] = useState<DetectedAgent[]>([]);
  const [selectedAgents, setSelectedAgents] = useState<Set<string>>(new Set());
  const [connecting, setConnecting] = useState(false);
  const [connectResults, setConnectResults] = useState<ConnectResult[]>([]);

  // Manual setup state
  const [port, setPort] = useState<number>(9315);
  const [registry, setRegistry] = useState<AgentEntry[]>([]);
  const [copied, setCopied] = useState(false);
  const [validating, setValidating] = useState(false);
  const [validated, setValidated] = useState<DetectedAgent[]>([]);
  const [hasValidated, setHasValidated] = useState(false);

  // Reset state when dialog opens/closes
  useEffect(() => {
    if (!open) {
      setStep("choose");
      setScanning(false);
      setAgents([]);
      setSelectedAgents(new Set());
      setConnecting(false);
      setConnectResults([]);
      setCopied(false);
      setValidating(false);
      setValidated([]);
      setHasValidated(false);
    }
  }, [open]);

  // -------------------------------------------------------------------------
  // Auto-connect helpers
  // -------------------------------------------------------------------------

  async function startAutoScan() {
    setStep("auto");
    setScanning(true);
    setConnectResults([]);
    try {
      const detected = await detectAgents();
      const installed = detected.filter((a) => a.installed);
      setAgents(installed);
      setSelectedAgents(new Set(installed.map((a) => a.id)));
    } catch {
      setAgents([]);
    } finally {
      setScanning(false);
    }
  }

  function toggleAgent(id: string) {
    setSelectedAgents((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  async function handleConnect() {
    setConnecting(true);
    const results: ConnectResult[] = [];

    for (const agent of agents) {
      if (!selectedAgents.has(agent.id)) continue;
      try {
        const error = await connectAgent(agent.id);
        results.push({
          id: agent.id,
          name: agent.name,
          icon: agent.icon,
          success: !error,
          error: error ?? undefined,
        });
      } catch (err) {
        results.push({
          id: agent.id,
          name: agent.name,
          icon: agent.icon,
          success: false,
          error: String(err),
        });
      }
    }

    setConnectResults(results);
    setConnecting(false);
  }

  // -------------------------------------------------------------------------
  // Manual setup helpers
  // -------------------------------------------------------------------------

  async function startManual() {
    setStep("manual");
    try {
      const [p, reg] = await Promise.all([getPort(), getAgentRegistry()]);
      setPort(p);
      // Only show auto-tier agents that have config_paths
      setRegistry(
        reg.filter((e) => e.tier === "auto" && e.config_paths !== null),
      );
    } catch {
      // fallback
    }
  }

  const jsonSnippet = JSON.stringify(
    {
      plugmux: {
        url: `http://localhost:${port}/env/default`,
      },
    },
    null,
    2,
  );

  async function handleCopy() {
    await navigator.clipboard.writeText(jsonSnippet);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  async function handleValidate() {
    setValidating(true);
    try {
      const detected = await detectAgents();
      setValidated(detected);
      setHasValidated(true);
    } catch {
      setValidated([]);
      setHasValidated(true);
    } finally {
      setValidating(false);
    }
  }

  function isAgentConfigured(agentId: string): boolean {
    const agent = validated.find((a) => a.id === agentId);
    return agent?.status === "green" || agent?.status === "yellow";
  }

  // -------------------------------------------------------------------------
  // Shared
  // -------------------------------------------------------------------------

  function handleDone() {
    onComplete();
    onOpenChange(false);
  }

  function handleBack() {
    setStep("choose");
    setConnectResults([]);
    setHasValidated(false);
  }

  const hasResults = connectResults.length > 0;

  // -------------------------------------------------------------------------
  // Render
  // -------------------------------------------------------------------------

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[80vw] max-w-none overflow-hidden">
        {/* ---- Step 1: Choose method ---- */}
        {step === "choose" && (
          <>
            <DialogHeader>
              <DialogTitle>Setup plugmux</DialogTitle>
              <DialogDescription>
                Choose how you'd like to connect your code agents to plugmux.
              </DialogDescription>
            </DialogHeader>

            <div className="grid grid-cols-2 gap-4 py-4">
              {/* Auto Connect card */}
              <button
                type="button"
                onClick={startAutoScan}
                className="group flex flex-col items-start gap-3 rounded-lg border border-border p-6 text-left transition-colors hover:border-primary hover:bg-accent"
              >
                <div className="flex h-10 w-10 items-center justify-center rounded-md bg-primary/10 text-primary">
                  <Zap className="h-5 w-5" />
                </div>
                <div>
                  <p className="text-sm font-semibold">
                    Auto Connect
                    <Badge variant="secondary" className="ml-2 text-[10px]">
                      Recommended
                    </Badge>
                  </p>
                  <p className="mt-1 text-xs text-muted-foreground leading-relaxed">
                    plugmux will find all installed agents, back up their
                    configs, and connect them automatically.
                  </p>
                </div>
              </button>

              {/* Manual Setup card */}
              <button
                type="button"
                onClick={startManual}
                className="group flex flex-col items-start gap-3 rounded-lg border border-border p-6 text-left transition-colors hover:border-primary hover:bg-accent"
              >
                <div className="flex h-10 w-10 items-center justify-center rounded-md bg-primary/10 text-primary">
                  <Code className="h-5 w-5" />
                </div>
                <div>
                  <p className="text-sm font-semibold">Manual Setup</p>
                  <p className="mt-1 text-xs text-muted-foreground leading-relaxed">
                    Add plugmux to your agent configs yourself, with step-by-step
                    guidance.
                  </p>
                </div>
              </button>
            </div>
          </>
        )}

        {/* ---- Step 2a: Auto Connect ---- */}
        {step === "auto" && (
          <>
            <DialogHeader>
              <DialogTitle>Auto Connect</DialogTitle>
              <DialogDescription>
                {hasResults
                  ? "Connection results"
                  : "Select which agents to connect to plugmux."}
              </DialogDescription>
            </DialogHeader>

            {scanning ? (
              <div className="flex items-center justify-center py-12">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                <span className="ml-3 text-sm text-muted-foreground">
                  Scanning for installed agents...
                </span>
              </div>
            ) : connecting ? (
              <div className="flex items-center justify-center py-12">
                <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                <span className="ml-3 text-sm text-muted-foreground">
                  Connecting agents...
                </span>
              </div>
            ) : hasResults ? (
              /* Results view */
              <ScrollArea className="max-h-[50vh]">
                <div className="space-y-1 py-2">
                  {connectResults.map((r) => (
                    <div
                      key={r.id}
                      className="flex items-center gap-3 rounded-md px-3 py-2.5"
                    >
                      <AgentIcon icon={r.icon} name={r.name} />
                      <div className="min-w-0 flex-1">
                        <p className="text-sm font-medium">{r.name}</p>
                        {r.error && (
                          <p className="truncate text-xs text-destructive">
                            {r.error}
                          </p>
                        )}
                      </div>
                      {r.success ? (
                        <Badge className="bg-green-900/20 text-green-400 border-green-800 text-xs">
                          <Check className="mr-1 h-3 w-3" />
                          Connected
                        </Badge>
                      ) : (
                        <Badge variant="destructive" className="text-xs">
                          <X className="mr-1 h-3 w-3" />
                          Failed
                        </Badge>
                      )}
                    </div>
                  ))}
                </div>
              </ScrollArea>
            ) : (
              /* Agent selection view */
              <>
                {agents.length === 0 ? (
                  <p className="text-center text-sm text-muted-foreground py-8">
                    No installed agents detected on this machine.
                  </p>
                ) : (
                  <ScrollArea className="max-h-[50vh]">
                    <div className="space-y-1 py-2">
                      {agents.map((agent) => (
                        <label
                          key={agent.id}
                          className="flex cursor-pointer items-center gap-3 rounded-md px-3 py-2.5 hover:bg-accent"
                        >
                          <Checkbox
                            checked={selectedAgents.has(agent.id)}
                            onCheckedChange={() => toggleAgent(agent.id)}
                          />
                          <AgentIcon icon={agent.icon} name={agent.name} />
                          <div className="min-w-0 flex-1">
                            <p className="text-sm font-medium">{agent.name}</p>
                            <p className="truncate text-xs text-muted-foreground">
                              {agent.config_path ?? "Unknown config path"}
                            </p>
                          </div>
                        </label>
                      ))}
                    </div>
                  </ScrollArea>
                )}
              </>
            )}

            <DialogFooter className="sm:justify-between">
              <Button variant="ghost" onClick={handleBack} disabled={connecting}>
                <ArrowLeft className="mr-1 h-4 w-4" />
                Back
              </Button>
              <div className="flex items-center gap-2">
                {hasResults ? (
                  <Button
                    onClick={handleDone}
                    className="bg-[#7A67D1] hover:bg-[#6A57C1]"
                  >
                    Done
                  </Button>
                ) : (
                  <Button
                    onClick={handleConnect}
                    disabled={
                      scanning || connecting || selectedAgents.size === 0
                    }
                    className="bg-[#7A67D1] hover:bg-[#6A57C1]"
                  >
                    Connect
                  </Button>
                )}
              </div>
            </DialogFooter>
          </>
        )}

        {/* ---- Step 2b: Manual Setup ---- */}
        {step === "manual" && (
          <>
            <DialogHeader>
              <DialogTitle>Manual Setup</DialogTitle>
              <DialogDescription>
                Add the following MCP server entry to your agent's config file.
              </DialogDescription>
            </DialogHeader>

            <ScrollArea className="max-h-[60vh]">
              <div className="space-y-6 py-2">
                {/* JSON snippet */}
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <p className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
                      MCP Server Config
                    </p>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="h-7 gap-1.5 text-xs"
                      onClick={handleCopy}
                    >
                      {copied ? (
                        <>
                          <CheckCircle2 className="h-3 w-3 text-green-400" />
                          Copied
                        </>
                      ) : (
                        <>
                          <Copy className="h-3 w-3" />
                          Copy
                        </>
                      )}
                    </Button>
                  </div>
                  <pre className="rounded-md border bg-muted/50 p-4 text-sm font-mono leading-relaxed select-all">
                    {jsonSnippet}
                  </pre>
                </div>

                {/* Instruction text */}
                <p className="text-sm text-muted-foreground leading-relaxed">
                  Add this to the{" "}
                  <code className="rounded bg-muted px-1.5 py-0.5 text-xs font-mono">
                    mcpServers
                  </code>{" "}
                  section of your agent's config file, save, and restart the
                  agent.
                </p>

                {/* Config paths table */}
                {registry.length > 0 && (
                  <div className="space-y-2">
                    <p className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
                      Known Config Paths
                    </p>
                    <div className="rounded-md border">
                      <div className="grid grid-cols-[auto_1fr_auto] gap-x-4 px-3 py-2 border-b bg-muted/30 text-xs font-medium text-muted-foreground">
                        <span>Agent</span>
                        <span>Config File</span>
                        {hasValidated && <span>Status</span>}
                      </div>
                      {registry.map((entry) => (
                        <div
                          key={entry.id}
                          className="grid grid-cols-[auto_1fr_auto] items-center gap-x-4 px-3 py-2 border-b last:border-b-0 text-sm"
                        >
                          <div className="flex items-center gap-2">
                            <AgentIcon
                              icon={entry.icon}
                              name={entry.name}
                              className="h-4 w-4"
                            />
                            <span className="font-medium">{entry.name}</span>
                          </div>
                          <code className="truncate text-xs font-mono text-muted-foreground">
                            {entry.config_paths?.macos ?? "N/A"}
                          </code>
                          {hasValidated && (
                            <span>
                              {isAgentConfigured(entry.id) ? (
                                <CheckCircle2 className="h-4 w-4 text-green-400" />
                              ) : (
                                <X className="h-4 w-4 text-muted-foreground" />
                              )}
                            </span>
                          )}
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </ScrollArea>

            <DialogFooter className="sm:justify-between">
              <Button variant="ghost" onClick={handleBack}>
                <ArrowLeft className="mr-1 h-4 w-4" />
                Back
              </Button>
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  onClick={handleValidate}
                  disabled={validating}
                >
                  {validating && (
                    <Loader2 className="mr-1 h-4 w-4 animate-spin" />
                  )}
                  Validate
                </Button>
                <Button
                  onClick={handleDone}
                  className="bg-[#7A67D1] hover:bg-[#6A57C1]"
                >
                  Done
                </Button>
              </div>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
