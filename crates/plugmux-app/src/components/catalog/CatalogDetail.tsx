import { Check, Plus, X } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";
import type { CatalogEntry, Environment } from "@/lib/commands";

interface CatalogDetailProps {
  entry: CatalogEntry;
  installedIn: string[];
  environments: Environment[];
  onAdd: (envId: string) => void;
  onClose: () => void;
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

export function CatalogDetail({
  entry,
  installedIn,
  environments,
  onAdd,
  onClose,
}: CatalogDetailProps) {
  const initial = entry.name.charAt(0).toUpperCase();
  const color = colorForId(entry.id);
  const installedEnvs = environments.filter((e) =>
    installedIn.includes(e.id),
  );

  return (
    <Dialog open onOpenChange={(open) => { if (!open) onClose(); }}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <div
              className={cn(
                "flex h-12 w-12 shrink-0 items-center justify-center rounded-full text-lg font-semibold text-white",
                color,
              )}
            >
              {initial}
            </div>
            <div>
              <DialogTitle>{entry.name}</DialogTitle>
              <DialogDescription className="mt-1">
                <Badge variant="secondary" className="text-xs">
                  {entry.category}
                </Badge>
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <p className="text-sm text-muted-foreground">{entry.description}</p>

        <Separator />

        {/* Technical details */}
        <div className="grid grid-cols-2 gap-3 text-sm">
          <div>
            <span className="text-muted-foreground">Transport</span>
            <p className="font-medium">{entry.transport}</p>
          </div>
          <div>
            <span className="text-muted-foreground">Connectivity</span>
            <p className="font-medium">{entry.connectivity}</p>
          </div>
          {entry.command && (
            <div className="col-span-2">
              <span className="text-muted-foreground">Command</span>
              <p className="truncate font-mono text-xs">{entry.command}</p>
            </div>
          )}
          {entry.url && (
            <div className="col-span-2">
              <span className="text-muted-foreground">URL</span>
              <p className="truncate font-mono text-xs">{entry.url}</p>
            </div>
          )}
        </div>

        {/* Installed in */}
        {installedEnvs.length > 0 && (
          <>
            <Separator />
            <div>
              <span className="text-sm text-muted-foreground">
                Installed in
              </span>
              <div className="mt-1.5 flex flex-wrap gap-1.5">
                {installedEnvs.map((env) => (
                  <Badge key={env.id} variant="outline" className="gap-1">
                    <Check className="h-3 w-3 text-green-500" />
                    {env.name}
                  </Badge>
                ))}
              </div>
            </div>
          </>
        )}

        <Separator />

        {/* Actions */}
        <div className="flex items-center justify-between">
          <Button variant="outline" size="sm" onClick={onClose}>
            <X className="mr-1.5 h-3.5 w-3.5" />
            Close
          </Button>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button size="sm" className="gap-1.5">
                <Plus className="h-3.5 w-3.5" />
                Add to...
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              {environments.map((env) => (
                <DropdownMenuItem
                  key={env.id}
                  onClick={() => onAdd(env.id)}
                >
                  {env.name}
                  {installedIn.includes(env.id) && (
                    <Check className="ml-auto h-3.5 w-3.5 text-green-500" />
                  )}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </DialogContent>
    </Dialog>
  );
}
