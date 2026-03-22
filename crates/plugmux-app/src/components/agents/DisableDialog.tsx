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
import type { DetectedAgent } from "@/lib/commands";

interface DisableDialogProps {
  agent: DetectedAgent | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDisable: () => void;
  onDisableAndRestore: () => void;
  hasBackup: boolean;
}

export function DisableDialog({
  agent,
  open,
  onOpenChange,
  onDisable,
  onDisableAndRestore,
  hasBackup,
}: DisableDialogProps) {
  const [done, setDone] = useState(false);

  function handleDisable() {
    onDisable();
    setDone(true);
  }

  function handleDisableAndRestore() {
    onDisableAndRestore();
    setDone(true);
  }

  function handleOpenChange(next: boolean) {
    if (!next) setDone(false);
    onOpenChange(next);
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            Disable plugmux for {agent?.name ?? "agent"}
          </DialogTitle>
          <DialogDescription>
            {done
              ? `Restart ${agent?.name ?? "the agent"} to apply changes.`
              : "plugmux MCP will be removed from this agent's configuration."}
          </DialogDescription>
        </DialogHeader>

        {!done && (
          <DialogFooter>
            {hasBackup && (
              <Button variant="outline" onClick={handleDisableAndRestore}>
                Disable &amp; Restore
              </Button>
            )}
            <Button variant="destructive" onClick={handleDisable}>
              Disable
            </Button>
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  );
}
