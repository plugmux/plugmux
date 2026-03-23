import { useState, useEffect } from "react";
import { Check, Copy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Modal } from "@/components/ui/modal";
import { getPort } from "@/lib/commands";

interface AddAgentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onAdded: () => void;
}

export function AddAgentDialog({
  open,
  onOpenChange,
  onAdded,
}: AddAgentDialogProps) {
  const [name, setName] = useState("");
  const [port, setPort] = useState(4242);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (open) {
      setName("");
      setCopied(false);
      getPort().then(setPort);
    }
  }, [open]);

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

  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title="Add Custom Agent"
      description="Add plugmux MCP to any agent that supports the Model Context Protocol."
      size="md"
      footer={
        <div className="flex w-full justify-end">
          <Button onClick={() => { onAdded(); onOpenChange(false); }}>
            Done
          </Button>
        </div>
      }
    >
      <div className="space-y-4 py-2">
        <div className="space-y-1.5">
          <label htmlFor="agent-name" className="text-sm font-medium">
            Agent name
          </label>
          <Input
            id="agent-name"
            placeholder="e.g. My Custom Agent"
            value={name}
            onChange={(e) => setName(e.target.value)}
            autoFocus
          />
        </div>

        <div className="space-y-1.5">
          <p className="text-sm font-medium">
            Add this to the agent's <code className="rounded bg-muted px-1 py-0.5 text-xs">mcpServers</code> configuration:
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
        </div>

        <p className="text-xs text-muted-foreground">
          After adding the configuration, save the file and restart the agent. Use the refresh button on the Agents page to verify the connection.
        </p>
      </div>
    </Modal>
  );
}
