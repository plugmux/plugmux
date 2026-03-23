import { useState, useEffect } from "react";
import { Check, Copy, ExternalLink, RefreshCw } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Modal } from "@/components/ui/modal";
import { StatusDot } from "@/components/ui/status-dot";
import { getPort } from "@/lib/commands";
import type { DetectedAgent } from "@/lib/commands";

interface ManualSetupDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  agent: DetectedAgent | null;
}

interface LogEntry {
  agent_info?: { agent_id?: string };
  timestamp: string;
}

export function ManualSetupDialog({
  open,
  onOpenChange,
  agent,
}: ManualSetupDialogProps) {
  const [port, setPort] = useState(4242);
  const [copied, setCopied] = useState(false);
  const [validating, setValidating] = useState(false);
  const [validated, setValidated] = useState<boolean | null>(null);

  useEffect(() => {
    if (open) {
      setCopied(false);
      setValidated(null);
      getPort().then(setPort);
    }
  }, [open]);

  if (!agent) return null;

  const snippet = JSON.stringify(
    {
      plugmux: {
        type: "http",
        url: `http://localhost:${port}/env/global`,
      },
    },
    null,
    2,
  );

  function handleCopy() {
    navigator.clipboard.writeText(snippet);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  async function handleValidate() {
    setValidating(true);
    setValidated(null);
    try {
      const logs = await invoke<LogEntry[]>("get_recent_logs", { limit: 200 });
      const cutoff = new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString();
      const found = logs.some(
        (log) =>
          log.timestamp >= cutoff &&
          log.agent_info?.agent_id &&
          log.agent_info.agent_id
            .toLowerCase()
            .includes(agent!.id.toLowerCase()),
      );
      setValidated(found);
    } catch {
      setValidated(false);
    } finally {
      setValidating(false);
    }
  }

  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title={`Setup ${agent.name}`}
      description="This agent requires manual MCP configuration."
      size="md"
      footer={
        <div className="flex w-full items-center justify-between">
          <a
            href="https://www.plugmux.com/docs/agents"
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground"
          >
            <ExternalLink className="h-3 w-3" />
            Documentation
          </a>
          <Button onClick={() => onOpenChange(false)}>Done</Button>
        </div>
      }
    >
      <div className="space-y-4 py-2">
        <p className="text-sm text-muted-foreground">
          Open <strong>{agent.name}</strong> settings, find the MCP server
          configuration section, and add the following:
        </p>

        <div className="relative">
          <pre className="overflow-x-auto rounded-md bg-muted p-3 text-xs">
            {snippet}
          </pre>
          <Button
            variant="ghost"
            size="icon"
            className="absolute right-2 top-2 h-7 w-7"
            onClick={handleCopy}
          >
            {copied ? (
              <Check className="h-3.5 w-3.5 text-green-500" />
            ) : (
              <Copy className="h-3.5 w-3.5" />
            )}
          </Button>
        </div>

        <p className="text-xs text-muted-foreground">
          After adding the configuration, save and restart {agent.name}. Then
          press Validate to confirm the connection.
        </p>

        <div className="flex items-center gap-3">
          <Button
            variant="outline"
            size="sm"
            onClick={handleValidate}
            disabled={validating}
          >
            <RefreshCw
              className={`mr-1.5 h-3.5 w-3.5 ${validating ? "animate-spin" : ""}`}
            />
            Validate
          </Button>
          {validated !== null && (
            <div className="flex items-center gap-1.5 text-sm">
              <StatusDot
                status={validated ? "success" : "error"}
                label={
                  validated
                    ? "Agent connected successfully"
                    : "No recent activity detected"
                }
              />
              <span
                className={
                  validated ? "text-green-600 dark:text-green-400" : "text-muted-foreground"
                }
              >
                {validated ? "Connected" : "Not detected yet"}
              </span>
            </div>
          )}
        </div>
      </div>
    </Modal>
  );
}
