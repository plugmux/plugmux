import { Sidebar } from "./Sidebar";

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
    <div className="flex h-screen bg-background text-foreground">
      <Sidebar
        activePage={activePage}
        onNavigate={onNavigate}
        onNewEnvironment={onNewEnvironment}
      />
      <main className="flex-1 overflow-auto">{children}</main>
    </div>
  );
}
