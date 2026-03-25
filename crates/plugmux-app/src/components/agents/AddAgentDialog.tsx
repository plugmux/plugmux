import { useState, useEffect } from "react";
import { Check, Copy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Modal } from "@/components/ui/modal";
import { getPort } from "@/lib/commands";

interface AddAgentDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function AddAgentDialog({
  open,
  onOpenChange,
}: AddAgentDialogProps) {
  const [port, setPort] = useState(4242);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (open) {
      setCopied(false);
      getPort().then(setPort);
    }
  }, [open]);

  const snippet = JSON.stringify(
    {
      plugmux: {
        type: "http",
        url: `http://localhost:${port}/env/global`,
      },
    },
    null,
    2,
  );

  function handleCopy() {
    navigator.clipboard.writeText(snippet);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title="Add Custom Agent"
      description="Add this to the agent's MCP configuration:"
      size="md"
      footer={
        <div className="flex w-full justify-end">
          <Button size="sm" onClick={() => onOpenChange(false)}>
            Done
          </Button>
        </div>
      }
    >
      <div className="space-y-4 py-2">
        <div className="relative">
          <pre className="overflow-x-auto rounded-md bg-muted p-3 text-xs">
            {snippet}
          </pre>
          <Button
            variant="ghost"
            size="icon"
            className="absolute right-2 top-2 h-7 w-7"
            onClick={handleCopy}
          >
            {copied ? (
              <Check className="h-3.5 w-3.5 text-green-500" />
            ) : (
              <Copy className="h-3.5 w-3.5" />
            )}
          </Button>
        </div>

        <p className="text-xs text-muted-foreground">
          Once your agent makes its first MCP call, it will appear in the
          agents list automatically.
        </p>
      </div>
    </Modal>
  );
}
