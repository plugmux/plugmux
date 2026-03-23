import { useState } from "react";
import { Check, ChevronsUpDown, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { cn } from "@/lib/utils";

interface CategoryFilterProps {
  categories: { id: string; label: string }[];
  selected: string[];
  onSelect: (selected: string[]) => void;
}

export function CategoryFilter({
  categories,
  selected,
  onSelect,
}: CategoryFilterProps) {
  const [open, setOpen] = useState(false);

  const selectedLabels =
    selected.length === 0
      ? "All categories"
      : selected.length <= 2
        ? selected
            .map((id) => categories.find((c) => c.id === id)?.label ?? id)
            .join(", ")
        : `${selected.length} selected`;

  function toggle(id: string) {
    onSelect(
      selected.includes(id)
        ? selected.filter((s) => s !== id)
        : [...selected, id],
    );
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          className={cn(
            "gap-2",
            selected.length > 0 &&
              "border-primary/30 bg-primary/5 font-medium",
          )}
        >
          <span className="text-xs text-muted-foreground">Category:</span>
          <span className="max-w-[180px] truncate">{selectedLabels}</span>
          <ChevronsUpDown className="h-3.5 w-3.5 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[240px] p-0" align="start">
        <Command>
          <CommandInput placeholder="Filter categories..." />
          <CommandList>
            <CommandEmpty>No category found.</CommandEmpty>
            <CommandGroup>
              {selected.length > 0 && (
                <CommandItem
                  onSelect={() => onSelect([])}
                  className="text-xs font-medium text-primary"
                >
                  <X className="mr-2 h-3 w-3" />
                  Clear all
                </CommandItem>
              )}
              {categories.map((cat) => {
                const checked = selected.includes(cat.id);
                return (
                  <CommandItem
                    key={cat.id}
                    value={cat.label}
                    onSelect={() => toggle(cat.id)}
                  >
                    <div
                      className={cn(
                        "mr-2 flex h-4 w-4 items-center justify-center rounded-sm border",
                        checked
                          ? "border-primary bg-primary text-primary-foreground"
                          : "border-muted-foreground/30",
                      )}
                    >
                      {checked && <Check className="h-3 w-3" />}
                    </div>
                    {cat.label}
                  </CommandItem>
                );
              })}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
