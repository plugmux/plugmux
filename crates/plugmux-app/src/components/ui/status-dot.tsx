import { cn } from "@/lib/utils";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

export type StatusVariant = "success" | "error" | "warning" | "neutral";

interface StatusDotProps {
  status: StatusVariant;
  label?: string;
  className?: string;
}

const dotStyles: Record<StatusVariant, string> = {
  success: "bg-green-500",
  error: "bg-red-500",
  warning: "bg-yellow-500",
  neutral: "bg-muted-foreground/40",
};

export function StatusDot({ status, label, className }: StatusDotProps) {
  const dot = (
    <span
      className={cn(
        "inline-block h-2 w-2 shrink-0 rounded-full",
        dotStyles[status],
        className,
      )}
    />
  );

  if (!label) return dot;

  return (
    <TooltipProvider delayDuration={300}>
      <Tooltip>
        <TooltipTrigger asChild>{dot}</TooltipTrigger>
        <TooltipContent side="right" className="text-xs">
          {label}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
