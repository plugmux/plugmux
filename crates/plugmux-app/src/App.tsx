import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

function App() {
  return (
    <div className="h-screen bg-background text-foreground flex items-center justify-center gap-4">
      <Button>plugmux</Button>
      <Badge variant="secondary">v0.1.0</Badge>
    </div>
  );
}

export default App;
