import { useState, useEffect } from "react";
import { Check, Copy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Modal } from "@/components/ui/modal";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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
  const [configFormat, setConfigFormat] = useState("json");
  const [mcpKey, setMcpKey] = useState("mcpServers");
  const [port, setPort] = useState(4242);
  const [copied, setCopied] = useState(false);
  const [adding, setAdding] = useState(false);

  useEffect(() => {
    if (open) {
      setName("");
      setConfigPath("");
      setConfigFormat("json");
      setMcpKey("mcpServers");
      setCopied(false);
      setAdding(false);
      getPort().then(setPort);
    }
  }, [open]);

  // Update mcpKey suggestion when format changes
  useEffect(() => {
    setMcpKey(configFormat === "toml" ? "mcp_servers" : "mcpServers");
  }, [configFormat]);

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

  const canAdd = name.trim().length > 0 && configPath.trim().length > 0;

  function handleCopy() {
    navigator.clipboard.writeText(snippet);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  async function handleAdd() {
    if (!canAdd) return;
    setAdding(true);
    try {
      await addCustomAgent(name.trim(), configPath.trim(), configFormat, mcpKey);
      onAdded();
      onOpenChange(false);
    } catch (e) {
      toast.error(`Failed to add agent: ${e}`);
    } finally {
      setAdding(false);
    }
  }

  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title="Add Custom Agent"
      description="Register an agent that plugmux doesn't detect automatically."
      size="md"
      footer={
        <div className="flex w-full justify-end gap-2">
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleAdd} disabled={!canAdd || adding}>
            {adding ? "Adding..." : "Add"}
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
          <Label htmlFor="config-path">Config file path</Label>
          <Input
            id="config-path"
            placeholder="e.g. ~/.myagent/mcp.json"
            value={configPath}
            onChange={(e) => setConfigPath(e.target.value)}
          />
          <p className="text-xs text-muted-foreground">
            The path to the agent's MCP configuration file.
          </p>
        </div>

        <div className="flex gap-4">
          <div className="space-y-1.5">
            <Label>Config format</Label>
            <Select value={configFormat} onValueChange={setConfigFormat}>
              <SelectTrigger className="w-[120px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="json">JSON</SelectItem>
                <SelectItem value="toml">TOML</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex-1 space-y-1.5">
            <Label htmlFor="mcp-key">MCP key</Label>
            <Input
              id="mcp-key"
              value={mcpKey}
              onChange={(e) => setMcpKey(e.target.value)}
            />
          </div>
        </div>

        <div className="space-y-1.5">
          <p className="text-sm font-medium">
            Add this to the agent's{" "}
            <code className="rounded bg-muted px-1 py-0.5 text-xs">
              {mcpKey}
            </code>{" "}
            configuration:
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
      </div>
    </Modal>
  );
}
