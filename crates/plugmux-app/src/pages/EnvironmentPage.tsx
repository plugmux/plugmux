import { useState } from "react";
import { Copy, Check, Plus, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { useConfig } from "@/hooks/useConfig";
import { InheritedServers } from "@/components/environments/InheritedServers";
import { EnvironmentServers } from "@/components/environments/EnvironmentServers";
import { PermissionsPanel } from "@/components/environments/PermissionsPanel";
import { AddServerDialog } from "@/components/servers/AddServerDialog";

interface EnvironmentPageProps {
  envId: string;
}

export function EnvironmentPage({ envId }: EnvironmentPageProps) {
  const {
    config,
    toggleEnvOverride,
    addEnvServer,
    removeEnvServer,
    deleteEnvironment,
  } = useConfig();
  const [copied, setCopied] = useState(false);
  const [addDialogOpen, setAddDialogOpen] = useState(false);

  const env = config?.environments.find((e) => e.id === envId);
  const mainServers = config?.main.servers ?? [];

  if (!env) {
    return (
      <div className="p-6">
        <p className="text-sm text-muted-foreground">
          Environment not found.
        </p>
      </div>
    );
  }

  function copyEndpoint() {
    navigator.clipboard.writeText(env!.endpoint);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  async function handleDelete() {
    const confirmed = confirm(
      `Delete environment "${env!.name}"? This cannot be undone.`,
    );
    if (confirmed) {
      await deleteEnvironment(envId);
    }
  }

  return (
    <div className="p-6">
      {/* Header */}
      <div className="mb-6">
        <h1 className="text-2xl font-bold">{env.name}</h1>
        <div className="mt-1 flex items-center gap-2">
          <code className="text-sm text-muted-foreground">{env.endpoint}</code>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6"
            onClick={copyEndpoint}
          >
            {copied ? (
              <Check className="h-3 w-3 text-green-500" />
            ) : (
              <Copy className="h-3 w-3" />
            )}
          </Button>
        </div>
      </div>

      {/* Inherited Servers */}
      <InheritedServers
        servers={mainServers}
        overrides={env.overrides}
        onToggleOverride={(serverId) => toggleEnvOverride(envId, serverId)}
      />

      <Separator className="my-6" />

      {/* Environment Servers */}
      <div className="mb-3 flex items-center justify-between">
        <div /> {/* Spacer — heading is inside the component */}
        <Button onClick={() => setAddDialogOpen(true)}>
          <Plus className="h-4 w-4" />
          Add Server
        </Button>
      </div>
      <EnvironmentServers
        servers={env.servers}
        onToggle={(serverId) =>
          toggleEnvOverride(envId, serverId)
        }
        onRemove={(serverId) => removeEnvServer(envId, serverId)}
      />

      <Separator className="my-6" />

      {/* Permissions */}
      <PermissionsPanel envId={envId} />

      <Separator className="my-6" />

      {/* Danger Zone */}
      <div className="rounded-lg border border-destructive/50 p-4">
        <h2 className="text-lg font-semibold text-destructive">Danger Zone</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          Permanently delete this environment and all its configuration.
        </p>
        <Button
          variant="destructive"
          className="mt-3"
          onClick={handleDelete}
        >
          <Trash2 className="h-4 w-4" />
          Delete Environment
        </Button>
      </div>

      {/* Add Server Dialog */}
      <AddServerDialog
        open={addDialogOpen}
        onOpenChange={setAddDialogOpen}
        onAdd={(server) => addEnvServer(envId, server)}
      />
    </div>
  );
}
