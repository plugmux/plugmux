import { useEffect, useState } from "react";
import { Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Modal } from "@/components/ui/modal";
import { addCustomAgent } from "@/lib/commands";

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
  const [configFormat, setConfigFormat] = useState<"json" | "toml">("json");
  const [mcpKey, setMcpKey] = useState("mcpServers");
  const [adding, setAdding] = useState(false);

  useEffect(() => {
    if (open) {
      setName("");
      setConfigPath("");
      setConfigFormat("json");
      setMcpKey("mcpServers");
    }
  }, [open]);

  async function handleAdd() {
    if (!name.trim() || !configPath.trim()) return;
    setAdding(true);
    try {
      await addCustomAgent(
        name.trim(),
        configPath.trim(),
        configFormat,
        mcpKey.trim() || "mcpServers",
      );
      onAdded();
      onOpenChange(false);
    } catch {
      // stay open on error
    } finally {
      setAdding(false);
    }
  }

  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title="Add Custom Agent"
      description="Add an agent that's not in the registry by providing its config file details."
      size="md"
      footer={
        <div className="flex w-full justify-end">
          <Button
            onClick={handleAdd}
            disabled={adding || !name.trim() || !configPath.trim()}
          >
            {adding && <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />}
            Add
          </Button>
        </div>
      }
    >
      <div className="space-y-4 py-2">
        <div className="space-y-1.5">
          <label htmlFor="custom-agent-name" className="text-sm font-medium">
            Name
          </label>
          <Input
            id="custom-agent-name"
            placeholder="e.g. My Custom Agent"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>
        <div className="space-y-1.5">
          <label htmlFor="custom-config-path" className="text-sm font-medium">
            Config file path
          </label>
          <Input
            id="custom-config-path"
            placeholder="e.g. ~/.config/myagent/config.json"
            value={configPath}
            onChange={(e) => setConfigPath(e.target.value)}
          />
        </div>
        <div className="space-y-1.5">
          <span className="text-sm font-medium">Config format</span>
          <div className="flex gap-1.5">
            <button
              type="button"
              onClick={() => setConfigFormat("json")}
              className={`rounded-md border px-3 py-1.5 text-sm font-medium transition-colors ${configFormat === "json" ? "border-primary bg-primary/10 text-primary" : "border-border text-muted-foreground hover:border-foreground/40 hover:text-foreground"}`}
            >
              JSON
            </button>
            <button
              type="button"
              onClick={() => setConfigFormat("toml")}
              className={`rounded-md border px-3 py-1.5 text-sm font-medium transition-colors ${configFormat === "toml" ? "border-primary bg-primary/10 text-primary" : "border-border text-muted-foreground hover:border-foreground/40 hover:text-foreground"}`}
            >
              TOML
            </button>
          </div>
        </div>
        <div className="space-y-1.5">
          <label htmlFor="custom-mcp-key" className="text-sm font-medium">
            MCP key
          </label>
          <Input
            id="custom-mcp-key"
            placeholder="mcpServers"
            value={mcpKey}
            onChange={(e) => setMcpKey(e.target.value)}
          />
        </div>
      </div>
    </Modal>
  );
}
