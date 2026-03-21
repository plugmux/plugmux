import { useState } from "react";
import { Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useConfig } from "@/hooks/useConfig";
import { ServerCard } from "@/components/servers/ServerCard";
import { AddServerDialog } from "@/components/servers/AddServerDialog";

export function MainPage() {
  const { config, addMainServer, removeMainServer, toggleMainServer } =
    useConfig();
  const [dialogOpen, setDialogOpen] = useState(false);

  const servers = config?.main.servers ?? [];

  return (
    <div className="p-6">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Main</h1>
          <p className="text-sm text-muted-foreground">
            Global MCP servers shared across all environments.
          </p>
        </div>
        <Button onClick={() => setDialogOpen(true)}>
          <Plus className="h-4 w-4" />
          Add Server
        </Button>
      </div>

      {servers.length === 0 ? (
        <div className="flex flex-col items-center justify-center rounded-lg border border-dashed py-12 text-center">
          <p className="text-sm text-muted-foreground">
            No servers configured yet.
          </p>
          <Button
            variant="link"
            className="mt-2"
            onClick={() => setDialogOpen(true)}
          >
            Add your first server
          </Button>
        </div>
      ) : (
        <div className="grid gap-2">
          {servers.map((server) => (
            <ServerCard
              key={server.id}
              server={server}
              onToggle={() => toggleMainServer(server.id)}
              onRemove={() => removeMainServer(server.id)}
            />
          ))}
        </div>
      )}

      <AddServerDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onAdd={addMainServer}
      />
    </div>
  );
}
