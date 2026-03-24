import { useState, useEffect } from "react";
import { Check, Copy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Modal } from "@/components/ui/modal";
import { addCustomAgent, getPort } from "@/lib/commands";
import { toast } from "sonner";

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
  const [configPath, setConfigPath] = useState("");
  const [port, setPort] = useState(4242);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (open) {
      setName("");
      setConfigPath("");
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

  async function handleAdd() {
    if (!name.trim()) return;
    try {
      await addCustomAgent(name.trim(), "", "json", "mcpServers");
      onAdded();
      onOpenChange(false);
    } catch (e) {
      toast.error(`Failed to add agent: ${e}`);
    }
  }

  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title="Add Custom Agent"
      description="Register an MCP client that connects to plugmux."
      size="md"
      footer={
        <div className="flex w-full justify-end gap-2">
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleAdd} disabled={!name.trim()}>
            Add
          </Button>
        </div>
      }
    >
      <div className="space-y-4 py-2">
        <div className="space-y-1.5">
          <Label htmlFor="agent-name">Agent name</Label>
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
            Add this to the agent's MCP configuration:
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
          After adding the configuration, restart the agent and use the
          refresh button on the Agents page to verify.
        </p>
      </div>
    </Modal>
  );
}
