import { Bookmark, BadgeCheck, Check, Plus } from "lucide-react";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
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
  isBookmarked: boolean;
  onAdd: (envId: string) => void;
  onToggleBookmark: () => void;
  onClick: () => void;
}

function colorForId(id: string): string {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = (hash * 31 + id.charCodeAt(i)) | 0;
  }
  const hue = Math.abs(hash) % 360;
  return `hsl(${hue}, 60%, 55%)`;
}

export function CatalogCard({
  entry,
  installedIn,
  environments,
  isBookmarked,
  onAdd,
  onToggleBookmark,
  onClick,
}: CatalogCardProps) {
  const initial = entry.name.charAt(0).toUpperCase();
  const color = colorForId(entry.id);
  const isInstalled = installedIn.length > 0;

  return (
    <div
      className="group flex cursor-pointer flex-col gap-3 rounded-xl border bg-card p-5 transition-colors hover:border-primary/40"
      onClick={onClick}
    >
      {/* Header: avatar + name + badges + bookmark */}
      <div className="flex items-start gap-3">
        <Avatar className="h-10 w-10 shrink-0">
          {entry.icon && (
            <AvatarImage src={entry.icon} alt={entry.name} />
          )}
          <AvatarFallback
            className="font-mono text-[15px] font-bold"
            style={{
              color,
              background: `color-mix(in srgb, ${color} 12%, transparent)`,
            }}
          >
            {initial}
          </AvatarFallback>
        </Avatar>

        <div className="flex min-w-0 flex-1 flex-col gap-1">
          <div className="flex items-center gap-2">
            <span className="truncate text-[15px] font-semibold">
              {entry.name}
            </span>
            {entry.official && (
              <TooltipProvider delayDuration={200}>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <BadgeCheck className="h-4 w-4 shrink-0 text-green-500" />
                  </TooltipTrigger>
                  <TooltipContent side="top">
                    <p>Official</p>
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>
            )}
          </div>
        </div>

        {/* Bookmark */}
        <button
          onClick={(e) => {
            e.stopPropagation();
            onToggleBookmark();
          }}
          className="shrink-0 p-1"
          title={isBookmarked ? "Remove bookmark" : "Bookmark"}
        >
          <Bookmark
            className={cn(
              "h-4 w-4",
              isBookmarked
                ? "fill-amber-500 text-amber-500"
                : "text-muted-foreground/30 hover:text-muted-foreground/60",
            )}
          />
        </button>
      </div>

      {/* Description */}
      <p className="line-clamp-2 text-[13px] leading-relaxed text-muted-foreground">
        {entry.description}
      </p>

      {/* Footer: tags + action */}
      <div className="mt-auto flex items-center justify-between gap-2 pt-1">
        <div className="flex min-w-0 flex-wrap gap-1">
          {(Array.isArray(entry.categories) ? entry.categories : [entry.category])
            .filter(Boolean)
            .slice(0, 2)
            .map((cat) => (
              <span
                key={cat}
                className="truncate rounded-[5px] border border-muted-foreground/15 bg-muted-foreground/8 px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground"
              >
                {cat.replace(/-/g, " ")}
              </span>
            ))}
        </div>

        <div className="shrink-0" onClick={(e) => e.stopPropagation()}>
          {isInstalled ? (
            <Button
              size="sm"
              variant="outline"
              className="h-7 gap-1 border-green-500/25 bg-green-500/8 px-2 text-xs text-green-500 hover:bg-green-500/15 hover:text-green-500"
            >
              <Check className="h-3 w-3" />
              Installed
            </Button>
          ) : (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button size="sm" variant="outline" className="h-7 gap-1 px-2 text-xs">
                  <Plus className="h-3 w-3" />
                  Install
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
          )}
        </div>
      </div>
    </div>
  );
}
