import { Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface ServerCardProps {
  server: { id: string; name: string; description?: string };
  onRemove: () => void;
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

export function ServerCard({ server, onRemove }: ServerCardProps) {
  const initial = server.name.charAt(0).toUpperCase();
  const color = colorForId(server.id);

  return (
    <div className="flex items-center gap-3 rounded-lg border px-4 py-3">
      {/* Icon circle with initial */}
      <div
        className={cn(
          "flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-semibold text-white",
          color,
        )}
      >
        {initial}
      </div>

      {/* Name + description */}
      <div className="flex min-w-0 flex-1 flex-col">
        <span className="truncate font-medium">{server.name}</span>
        {server.description && (
          <span className="truncate text-xs text-muted-foreground">
            {server.description}
          </span>
        )}
      </div>

      {/* Health dot (static green) */}
      <span className="h-2 w-2 shrink-0 rounded-full bg-green-500" />

      {/* Remove button */}
      <Button
        variant="ghost"
        size="icon"
        className="h-8 w-8 shrink-0 text-muted-foreground hover:text-destructive"
        onClick={onRemove}
      >
        <Trash2 className="h-4 w-4" />
      </Button>
    </div>
  );
}
