import { useState } from "react";
import { Plus } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";
import { useCatalog } from "@/hooks/useCatalog";
import { createEnvFromPreset } from "@/lib/commands";

interface PresetsPageProps {
  onNavigate: (page: string) => void;
}

const COLORS = [
  "bg-blue-500",
  "bg-green-500",
  "bg-purple-500",
  "bg-orange-500",
  "bg-pink-500",
  "bg-teal-500",
  "bg-indigo-500",
  "bg-rose-500",
];

function colorForId(id: string): string {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash * 31 + id.charCodeAt(i)) | 0;
  }
  return COLORS[Math.abs(hash) % COLORS.length];
}

export function PresetsPage({ onNavigate }: PresetsPageProps) {
  const { presets, loading } = useCatalog();
  const [dialogPresetId, setDialogPresetId] = useState<string | null>(null);
  const [envName, setEnvName] = useState("");

  const activePreset = presets.find((p) => p.id === dialogPresetId);

  async function handleCreate() {
    const trimmed = envName.trim();
    if (!trimmed || !dialogPresetId) return;

    const env = await createEnvFromPreset(dialogPresetId, trimmed);
    setEnvName("");
    setDialogPresetId(null);
    if (env) {
      onNavigate(`env:${env.id}`);
    }
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      handleCreate();
    }
  }

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center p-6">
        <p className="text-muted-foreground">Loading presets...</p>
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold">Presets</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Create environments from preset templates with pre-configured servers.
        </p>
      </div>

      {presets.length === 0 ? (
        <div className="flex flex-col items-center gap-2 py-12">
          <p className="text-muted-foreground">No presets available.</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {presets.map((preset) => {
            const initial = preset.name.charAt(0).toUpperCase();
            const color = colorForId(preset.id);
            return (
              <div
                key={preset.id}
                className="flex flex-col gap-3 rounded-lg border p-4"
              >
                <div className="flex items-start gap-3">
                  <div
                    className={cn(
                      "flex h-10 w-10 shrink-0 items-center justify-center rounded-full text-sm font-semibold text-white",
                      color,
                    )}
                  >
                    {initial}
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="truncate font-medium">{preset.name}</p>
                    <p className="mt-0.5 text-sm text-muted-foreground">
                      {preset.description}
                    </p>
                  </div>
                </div>

                {/* Server names list */}
                <div className="flex flex-wrap gap-1.5">
                  {preset.servers.map((serverId) => (
                    <Badge key={serverId} variant="secondary" className="text-xs">
                      {serverId}
                    </Badge>
                  ))}
                </div>

                {/* Create button */}
                <div className="mt-auto flex justify-end">
                  <Button
                    size="sm"
                    onClick={() => setDialogPresetId(preset.id)}
                    className="gap-1.5"
                  >
                    <Plus className="h-3.5 w-3.5" />
                    Create Environment
                  </Button>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Name dialog */}
      <Dialog
        open={dialogPresetId !== null}
        onOpenChange={(open) => {
          if (!open) {
            setDialogPresetId(null);
            setEnvName("");
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create from Preset</DialogTitle>
            <DialogDescription>
              {activePreset
                ? `Create an environment using the "${activePreset.name}" preset.`
                : "Name your new environment."}
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4 py-2">
            <div className="grid gap-2">
              <Label htmlFor="preset-env-name">Environment Name</Label>
              <Input
                id="preset-env-name"
                placeholder="e.g. my-project"
                value={envName}
                onChange={(e) => setEnvName(e.target.value)}
                onKeyDown={handleKeyDown}
                autoFocus
              />
            </div>
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setDialogPresetId(null);
                setEnvName("");
              }}
            >
              Cancel
            </Button>
            <Button disabled={!envName.trim()} onClick={handleCreate}>
              Create
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
