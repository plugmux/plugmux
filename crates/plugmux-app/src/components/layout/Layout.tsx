import { Sidebar } from "./Sidebar";
import { Titlebar } from "./Titlebar";

interface LayoutProps {
  activePage: string;
  onNavigate: (page: string) => void;
  onNewEnvironment: () => void;
  children: React.ReactNode;
}

export function Layout({
  activePage,
  onNavigate,
  onNewEnvironment,
  children,
}: LayoutProps) {
  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <Titlebar />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar
          activePage={activePage}
          onNavigate={onNavigate}
          onNewEnvironment={onNewEnvironment}
        />
        <main className="flex-1 overflow-auto">{children}</main>
      </div>
    </div>
  );
}
