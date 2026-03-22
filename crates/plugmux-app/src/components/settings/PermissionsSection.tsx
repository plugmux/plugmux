import { Shield } from "lucide-react";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { setPermission } from "@/lib/commands";
import type { Permissions } from "@/lib/commands";

interface PermissionsSectionProps {
  permissions: Permissions | undefined;
}

type PermissionLevel = "allow" | "approve" | "disable";

const ACTIONS: { key: keyof Permissions; label: string }[] = [
  { key: "enable_server", label: "Enable Server" },
  { key: "disable_server", label: "Disable Server" },
];

export function PermissionsSection({ permissions }: PermissionsSectionProps) {
  const handleChange = (action: string, level: PermissionLevel) => {
    setPermission(action, level).catch(console.error);
  };

  return (
    <section className="space-y-4">
      <div className="flex items-center gap-2">
        <Shield className="h-4 w-4 text-muted-foreground" />
        <div>
          <h2 className="text-lg font-semibold leading-none">Permissions</h2>
          <p className="text-sm text-muted-foreground mt-1">
            Control what AI agents can do through plugmux
          </p>
        </div>
      </div>

      <div className="rounded-lg border divide-y">
        {ACTIONS.map(({ key, label }) => (
          <div
            key={key}
            className="flex items-center justify-between px-4 py-3"
          >
            <Label className="font-normal">{label}</Label>
            <Select
              value={permissions?.[key] ?? "allow"}
              onValueChange={(val: PermissionLevel) =>
                handleChange(key, val)
              }
            >
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="allow">Allow</SelectItem>
                <SelectItem value="approve">Approve</SelectItem>
                <SelectItem value="disable">Disable</SelectItem>
              </SelectContent>
            </Select>
          </div>
        ))}
      </div>
    </section>
  );
}
