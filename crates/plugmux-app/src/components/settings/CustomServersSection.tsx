import { useState } from "react";
import { Server, Plus, Pencil, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { AddServerDialog } from "@/components/servers/AddServerDialog";
import { useCustomServers } from "@/hooks/useCustomServers";
import type { ServerConfig } from "@/lib/commands";

export function CustomServersSection() {
  const { servers, addServer, updateServer, removeServer } = useCustomServers();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editTarget, setEditTarget] = useState<ServerConfig | null>(null);

  const handleOpenAdd = () => {
    setEditTarget(null);
    setDialogOpen(true);
  };

  const handleOpenEdit = (server: ServerConfig) => {
    setEditTarget(server);
    setDialogOpen(true);
  };

  const handleDialogOpenChange = (open: boolean) => {
    setDialogOpen(open);
    if (!open) setEditTarget(null);
  };

  const handleSave = async (config: ServerConfig) => {
    if (editTarget) {
      await updateServer(editTarget.id, config).catch(console.error);
    } else {
      await addServer(config).catch(console.error);
    }
  };

  const handleDelete = (server: ServerConfig) => {
    if (
      window.confirm(
        `Remove custom server "${server.name}"? This cannot be undone.`,
      )
    ) {
      removeServer(server.id).catch(console.error);
    }
  };

  const commandOrUrl = (server: ServerConfig): string => {
    if (server.transport === "stdio") return server.command ?? "—";
    return server.url ?? "—";
  };

  return (
    <section className="space-y-4">
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-2">
          <Server className="h-4 w-4 text-muted-foreground" />
          <div>
            <h2 className="text-lg font-semibold leading-none">
              Custom Servers
            </h2>
            <p className="text-sm text-muted-foreground mt-1">
              Manually configured MCP servers
            </p>
          </div>
        </div>
        <Button
          size="sm"
          variant="outline"
          onClick={handleOpenAdd}
          className="shrink-0"
        >
          <Plus className="h-4 w-4 mr-1" />
          Add Custom Server
        </Button>
      </div>

      {servers.length === 0 ? (
        <div className="rounded-lg border border-dashed px-4 py-8 text-center">
          <p className="text-sm text-muted-foreground">
            No custom servers configured
          </p>
        </div>
      ) : (
        <div className="rounded-lg border divide-y">
          {servers.map((server) => (
            <div
              key={server.id}
              className="flex items-center gap-3 px-4 py-3"
            >
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-0.5">
                  <span className="font-medium text-sm truncate">
                    {server.name}
                  </span>
                  <Badge variant="secondary" className="text-xs shrink-0">
                    {server.transport}
                  </Badge>
                </div>
                <p className="text-xs text-muted-foreground truncate">
                  {commandOrUrl(server)}
                </p>
              </div>
              <div className="flex items-center gap-1 shrink-0">
                <Button
                  size="icon"
                  variant="ghost"
                  className="h-7 w-7"
                  onClick={() => handleOpenEdit(server)}
                  title="Edit server"
                >
                  <Pencil className="h-3.5 w-3.5" />
                </Button>
                <Button
                  size="icon"
                  variant="ghost"
                  className="h-7 w-7 text-destructive hover:text-destructive"
                  onClick={() => handleDelete(server)}
                  title="Delete server"
                >
                  <Trash2 className="h-3.5 w-3.5" />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}

      <AddServerDialog
        open={dialogOpen}
        onOpenChange={handleDialogOpenChange}
        onAdd={handleSave}
        initialValues={editTarget ?? undefined}
      />
    </section>
  );
}
