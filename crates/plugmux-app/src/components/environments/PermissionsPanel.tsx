import { useState, useEffect, useCallback } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { invoke } from "@tauri-apps/api/core";

interface PermissionsPanelProps {
  envId: string;
}

type PermissionLevel = "allow" | "approve" | "disable";

interface PermissionEntry {
  action: string;
  level: PermissionLevel;
}

const ACTIONS = ["enable_server", "disable_server"] as const;

export function PermissionsPanel({ envId }: PermissionsPanelProps) {
  const [expanded, setExpanded] = useState(false);
  const [permissions, setPermissions] = useState<PermissionEntry[]>(
    ACTIONS.map((action) => ({ action, level: "approve" })),
  );

  const loadPermissions = useCallback(async () => {
    try {
      const result = await invoke<PermissionEntry[]>("get_permissions", {
        envId,
      });
      setPermissions(result);
    } catch {
      // Use defaults if command not available yet
    }
  }, [envId]);

  useEffect(() => {
    if (expanded) {
      loadPermissions();
    }
  }, [expanded, loadPermissions]);

  async function handleChange(action: string, level: PermissionLevel) {
    setPermissions((prev) =>
      prev.map((p) => (p.action === action ? { ...p, level } : p)),
    );
    try {
      await invoke("set_permission", { envId, action, level });
    } catch {
      // Revert on error
      loadPermissions();
    }
  }

  return (
    <div>
      <Button
        variant="ghost"
        className="flex items-center gap-2 px-0 font-semibold hover:bg-transparent"
        onClick={() => setExpanded(!expanded)}
      >
        {expanded ? (
          <ChevronDown className="h-4 w-4" />
        ) : (
          <ChevronRight className="h-4 w-4" />
        )}
        Permissions
      </Button>

      {expanded && (
        <div className="mt-3 rounded-lg border">
          <table className="w-full">
            <thead>
              <tr className="border-b text-left text-sm text-muted-foreground">
                <th className="px-4 py-2 font-medium">Action</th>
                <th className="px-4 py-2 font-medium">Level</th>
              </tr>
            </thead>
            <tbody>
              {permissions.map((perm) => (
                <tr key={perm.action} className="border-b last:border-b-0">
                  <td className="px-4 py-2 text-sm font-mono">
                    {perm.action}
                  </td>
                  <td className="px-4 py-2">
                    <Select
                      value={perm.level}
                      onValueChange={(val: PermissionLevel) =>
                        handleChange(perm.action, val)
                      }
                    >
                      <SelectTrigger className="h-8 w-32">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="allow">Allow</SelectItem>
                        <SelectItem value="approve">Approve</SelectItem>
                        <SelectItem value="disable">Disable</SelectItem>
                      </SelectContent>
                    </Select>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
