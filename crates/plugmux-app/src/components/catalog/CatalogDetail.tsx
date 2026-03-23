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
import type { CatalogEntry, Environment } from "@/lib/commands";

interface CatalogDetailProps {
  entry: CatalogEntry;
  installedIn: string[];
  environments: Environment[];
  onAdd: (envId: string) => void;
  onClose: () => void;
}

function colorForId(id: string): string {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash * 31 + id.charCodeAt(i)) | 0;
  }
  const hue = Math.abs(hash) % 360;
  return `hsl(${hue}, 60%, 55%)`;
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
  const categories = entry.categories ?? [entry.category];

  return (
    <Dialog open onOpenChange={(open) => { if (!open) onClose(); }}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <div
              className="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl font-mono text-lg font-bold"
              style={{
                color,
                background: `color-mix(in srgb, ${color} 12%, transparent)`,
                border: `1px solid color-mix(in srgb, ${color} 25%, transparent)`,
              }}
            >
              {initial}
            </div>
            <div>
              <DialogTitle className="flex items-center gap-2">
                {entry.name}
                {entry.official && (
                  <Badge
                    variant="outline"
                    className="gap-1 border-primary/30 bg-primary/10 px-1.5 py-0 text-[10px] font-semibold text-primary"
                  >
                    Official
                  </Badge>
                )}
              </DialogTitle>
              <DialogDescription className="mt-1.5 flex flex-wrap gap-1.5">
                {categories.filter(Boolean).map((cat) => (
                  <span
                    key={cat}
                    className="rounded-[5px] border border-muted-foreground/15 bg-muted-foreground/8 px-2 py-0.5 font-mono text-[11px] text-muted-foreground"
                  >
                    {cat}
                  </span>
                ))}
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
