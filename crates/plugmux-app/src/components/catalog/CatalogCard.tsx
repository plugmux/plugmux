import { Check, Plus } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";
import type { CatalogEntry, Environment } from "@/lib/commands";

interface CatalogCardProps {
  entry: CatalogEntry;
  installedIn: string[];
  environments: Environment[];
  onAdd: (envId: string) => void;
  onClick: () => void;
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

export function CatalogCard({
  entry,
  installedIn,
  environments,
  onAdd,
  onClick,
}: CatalogCardProps) {
  const initial = entry.name.charAt(0).toUpperCase();
  const color = colorForId(entry.id);
  const isInstalled = installedIn.length > 0;

  return (
    <div
      className="flex cursor-pointer flex-col gap-3 rounded-lg border p-4 transition-colors hover:bg-accent/50"
      onClick={onClick}
    >
      <div className="flex items-start gap-3">
        {/* Icon circle */}
        <div
          className={cn(
            "flex h-10 w-10 shrink-0 items-center justify-center rounded-full text-sm font-semibold text-white",
            color,
          )}
        >
          {initial}
        </div>

        <div className="flex min-w-0 flex-1 flex-col gap-1">
          <div className="flex items-center gap-2">
            <span className="truncate font-medium">{entry.name}</span>
            {isInstalled && (
              <Check className="h-4 w-4 shrink-0 text-green-500" />
            )}
          </div>
          <Badge variant="secondary" className="w-fit text-xs">
            {entry.category}
          </Badge>
        </div>
      </div>

      {/* Description — 2 lines max */}
      <p className="line-clamp-2 text-sm text-muted-foreground">
        {entry.description}
      </p>

      {/* Add button with dropdown */}
      <div className="mt-auto flex justify-end" onClick={(e) => e.stopPropagation()}>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button size="sm" variant="outline" className="gap-1.5">
              <Plus className="h-3.5 w-3.5" />
              Add
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
    </div>
  );
}
