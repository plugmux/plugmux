import { useState } from "react";
import { Cable, ArrowRightIcon } from "lucide-react";
import { Banner } from "@/components/ui/banner";
import { Button } from "@/components/ui/button";

export function DashboardPage() {
  const [showBanner, setShowBanner] = useState(true);

  return (
    <div className="p-6">
      <Banner
        show={showBanner}
        onHide={() => setShowBanner(false)}
        variant="premium"
        title="Connect your code agents"
        description="To start using plugmux, make plugmux MCP available to your agents."
        showShade={true}
        closable={false}
        icon={<Cable />}
        action={
          <Button
            variant="ghost"
            className="inline-flex items-center gap-1 rounded-md bg-black/10 px-3 py-1.5 text-sm font-medium transition-colors hover:bg-black/20 dark:bg-white/10 dark:hover:bg-white/20"
          >
            Setup
            <ArrowRightIcon className="h-3 w-3" />
          </Button>
        }
      />
    </div>
  );
}
