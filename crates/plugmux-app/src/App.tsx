import { useState } from "react";
import { Layout } from "@/components/layout/Layout";
import { EnvironmentPage } from "@/pages/EnvironmentPage";
import { CatalogPage } from "@/pages/CatalogPage";
import { PresetsPage } from "@/pages/PresetsPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { AgentsPage } from "@/pages/AgentsPage";
import { LogsPage } from "@/pages/LogsPage";
import { CreateEnvironmentDialog } from "@/components/environments/CreateEnvironmentDialog";
import { Toaster } from "@/components/ui/sonner";

function App() {
  const [activePage, setActivePage] = useState("agents");
  const [newEnvOpen, setNewEnvOpen] = useState(false);

  function renderPage() {
    if (activePage.startsWith("env:")) {
      const envId = activePage.slice(4);
      return <EnvironmentPage envId={envId} onNavigate={setActivePage} />;
    }

    switch (activePage) {
      case "agents":
        return <AgentsPage />;
      case "catalog":
        return <CatalogPage />;
      case "presets":
        return <PresetsPage />;
      case "settings":
        return <SettingsPage />;
      case "logs":
        return <LogsPage />;
      default:
        return <AgentsPage />;
    }
  }

  return (
    <Layout
      activePage={activePage}
      onNavigate={setActivePage}
      onNewEnvironment={() => setNewEnvOpen(true)}
    >
      {renderPage()}
      <CreateEnvironmentDialog
        open={newEnvOpen}
        onOpenChange={setNewEnvOpen}
        onCreated={(envId) => {
          setNewEnvOpen(false);
          setActivePage(`env:${envId}`);
        }}
      />
      <Toaster />
    </Layout>
  );
}

export default App;
