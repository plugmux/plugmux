import { useState } from "react";
import { Check, Copy, Pencil, Plus, Trash2, Wrench } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { ServerCard } from "@/components/servers/ServerCard";
import { AddServerDialog } from "@/components/servers/AddServerDialog";
import { useConfig } from "@/hooks/useConfig";
import { useCatalog } from "@/hooks/useCatalog";
import { useCustomServers } from "@/hooks/useCustomServers";
import type { ServerConfig } from "@/lib/commands";

interface EnvironmentPageProps {
  envId: string;
  onNavigate: (page: string) => void;
}

export function EnvironmentPage({ envId, onNavigate }: EnvironmentPageProps) {
  const { config, environments, renameEnvironment, removeServerFromEnv, addServerToEnv, deleteEnvironment } = useConfig();
  const { servers: catalogServers } = useCatalog();
  const { servers: customServers, addServer: addCustomServer } = useCustomServers();

  const [editing, setEditing] = useState(false);
  const [editName, setEditName] = useState("");
  const [copied, setCopied] = useState(false);
  const [addDialogOpen, setAddDialogOpen] = useState(false);

  const env = environments.find((e) => e.id === envId);
  const port = config?.port ?? 4242;
  const envUrl = `http://localhost:${port}/env/${envId}`;

  if (!env) {
    return (
      <div className="flex h-full items-center justify-center p-6">
        <p className="text-muted-foreground">Environment not found.</p>
      </div>
    );
  }

  // Resolve server info from catalog or custom servers
  function resolveServer(serverId: string): { id: string; name: string; description?: string } {
    const catalog = catalogServers.find((s) => s.id === serverId);
    if (catalog) return { id: catalog.id, name: catalog.name, description: catalog.description };

    const custom = customServers.find((s) => s.id === serverId);
    if (custom) return { id: custom.id, name: custom.name, description: custom.description };

    return { id: serverId, name: serverId };
  }

  function startEditing() {
    setEditName(env!.name);
    setEditing(true);
  }

  async function finishEditing() {
    const trimmed = editName.trim();
    if (trimmed && trimmed !== env!.name) {
      await renameEnvironment(envId, trimmed);
    }
    setEditing(false);
  }

  function handleEditKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      finishEditing();
    } else if (e.key === "Escape") {
      setEditing(false);
    }
  }

  async function handleCopyUrl() {
    await navigator.clipboard.writeText(envUrl);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  async function handleRemoveServer(serverId: string) {
    await removeServerFromEnv(envId, serverId);
  }

  async function handleAddCustomServer(server: ServerConfig) {
    await addCustomServer(server);
    await addServerToEnv(envId, server.id);
  }

  async function handleDelete() {
    await deleteEnvironment(envId);
    onNavigate("env:default");
  }

  return (
    <div className="p-6">
      {/* Header: name + URL */}
      <div className="mb-6">
        <div className="flex items-center gap-2">
          {editing ? (
            <Input
              value={editName}
              onChange={(e) => setEditName(e.target.value)}
              onBlur={finishEditing}
              onKeyDown={handleEditKeyDown}
              className="h-9 w-64 text-2xl font-bold"
              autoFocus
            />
          ) : (
            <>
              <h1 className="text-2xl font-bold">{env.name}</h1>
              {envId !== "global" && (
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-7 w-7 text-muted-foreground"
                  onClick={startEditing}
                >
                  <Pencil className="h-3.5 w-3.5" />
                </Button>
              )}
            </>
          )}
        </div>

        <div className="mt-2 flex items-center gap-2">
          <code className="rounded bg-muted px-2 py-1 text-xs text-muted-foreground">
            {envUrl}
          </code>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 text-muted-foreground"
            onClick={handleCopyUrl}
          >
            {copied ? (
              <Check className="h-3.5 w-3.5 text-green-500" />
            ) : (
              <Copy className="h-3.5 w-3.5" />
            )}
          </Button>
        </div>
      </div>

      <Separator className="mb-6" />

      {/* Server list */}
      <div className="mb-4 flex items-center gap-2">
        <h2 className="text-lg font-semibold">Servers</h2>
        <Badge variant="secondary">{env.servers.length}</Badge>
      </div>

      {env.servers.length === 0 ? (
        <div className="flex flex-col items-center gap-3 rounded-lg border border-dashed py-10">
          <Wrench className="h-8 w-8 text-muted-foreground" />
          <p className="text-sm text-muted-foreground">
            No servers yet. Add from the catalog or create a custom server.
          </p>
          <div className="flex gap-2">
            <Button size="sm" onClick={() => onNavigate("catalog")}>
              <Plus className="mr-1.5 h-3.5 w-3.5" />
              Browse Catalog
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => setAddDialogOpen(true)}
            >
              <Plus className="mr-1.5 h-3.5 w-3.5" />
              Custom Server
            </Button>
          </div>
        </div>
      ) : (
        <div className="grid gap-2">
          {env.servers.map((serverId) => {
            const resolved = resolveServer(serverId);
            return (
              <ServerCard
                key={serverId}
                server={resolved}
                onRemove={() => handleRemoveServer(serverId)}
              />
            );
          })}
        </div>
      )}

      {/* Action buttons */}
      {env.servers.length > 0 && (
        <div className="mt-4 flex gap-2">
          <Button size="sm" onClick={() => onNavigate("catalog")}>
            <Plus className="mr-1.5 h-3.5 w-3.5" />
            Add Server
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => setAddDialogOpen(true)}
          >
            <Plus className="mr-1.5 h-3.5 w-3.5" />
            Add Custom Server
          </Button>
        </div>
      )}

      {/* Delete environment */}
      {envId !== "global" && (
        <>
          <Separator className="my-6" />
          <Button
            variant="destructive"
            size="sm"
            onClick={handleDelete}
          >
            <Trash2 className="mr-1.5 h-3.5 w-3.5" />
            Delete Environment
          </Button>
        </>
      )}

      {/* Add custom server dialog */}
      <AddServerDialog
        open={addDialogOpen}
        onOpenChange={setAddDialogOpen}
        onAdd={handleAddCustomServer}
      />
    </div>
  );
}
