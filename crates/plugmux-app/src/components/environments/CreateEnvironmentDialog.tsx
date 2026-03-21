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
import { useConfig } from "@/hooks/useConfig";

interface CreateEnvironmentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: (envId: string) => void;
}

export function CreateEnvironmentDialog({
  open,
  onOpenChange,
  onCreated,
}: CreateEnvironmentDialogProps) {
  const [name, setName] = useState("");
  const { createEnvironment } = useConfig();

  async function handleCreate() {
    const trimmed = name.trim();
    if (!trimmed) return;

    const env = await createEnvironment(trimmed);
    setName("");
    onOpenChange(false);
    if (env) {
      onCreated(env.id);
    }
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      handleCreate();
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create Environment</DialogTitle>
          <DialogDescription>
            Create a new environment that inherits servers from Main.
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-2">
          <div className="grid gap-2">
            <Label htmlFor="env-name">Name</Label>
            <Input
              id="env-name"
              placeholder="e.g. development, staging"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={handleKeyDown}
              autoFocus
            />
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button disabled={!name.trim()} onClick={handleCreate}>
            Create
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
