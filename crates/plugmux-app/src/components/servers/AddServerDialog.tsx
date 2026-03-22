import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { ServerConfig } from "@/lib/commands";

interface AddServerDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onAdd: (server: ServerConfig) => void;
}

const initialForm = {
  id: "",
  name: "",
  transport: "stdio" as "stdio" | "http",
  command: "",
  url: "",
  connectivity: "local" as "local" | "online",
};

export function AddServerDialog({
  open,
  onOpenChange,
  onAdd,
}: AddServerDialogProps) {
  const [form, setForm] = useState(initialForm);

  function reset() {
    setForm(initialForm);
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!form.id.trim()) return;

    const server: ServerConfig = {
      id: form.id.trim(),
      name: form.name.trim() || form.id.trim(),
      transport: form.transport,
      connectivity: form.connectivity,
      ...(form.transport === "stdio" && form.command.trim()
        ? { command: form.command.trim() }
        : {}),
      ...(form.transport === "http" && form.url.trim()
        ? { url: form.url.trim() }
        : {}),
    };

    onAdd(server);
    reset();
    onOpenChange(false);
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add Custom Server</DialogTitle>
          <DialogDescription>
            Define a custom MCP server that is not in the catalog.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="grid gap-4 py-2">
          <div className="grid gap-2">
            <Label htmlFor="server-id">Server ID</Label>
            <Input
              id="server-id"
              placeholder="e.g. my-mcp-server"
              value={form.id}
              onChange={(e) => setForm({ ...form, id: e.target.value })}
              required
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="server-name">Display Name</Label>
            <Input
              id="server-name"
              placeholder="Optional display name"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
            />
          </div>

          <div className="grid gap-2">
            <Label>Transport</Label>
            <Select
              value={form.transport}
              onValueChange={(val: "stdio" | "http") =>
                setForm({ ...form, transport: val })
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="stdio">stdio</SelectItem>
                <SelectItem value="http">http</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {form.transport === "stdio" && (
            <div className="grid gap-2">
              <Label htmlFor="server-command">Command</Label>
              <Input
                id="server-command"
                placeholder="e.g. npx -y @modelcontextprotocol/server"
                value={form.command}
                onChange={(e) => setForm({ ...form, command: e.target.value })}
              />
            </div>
          )}

          {form.transport === "http" && (
            <div className="grid gap-2">
              <Label htmlFor="server-url">URL</Label>
              <Input
                id="server-url"
                placeholder="e.g. http://localhost:3000/mcp"
                value={form.url}
                onChange={(e) => setForm({ ...form, url: e.target.value })}
              />
            </div>
          )}

          <div className="grid gap-2">
            <Label>Connectivity</Label>
            <Select
              value={form.connectivity}
              onValueChange={(val: "local" | "online") =>
                setForm({ ...form, connectivity: val })
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="local">Local</SelectItem>
                <SelectItem value="online">Online</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit">Add Server</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
