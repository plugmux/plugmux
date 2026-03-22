import { useState } from "react";
import { Layout } from "@/components/layout/Layout";
import { EnvironmentPage } from "@/pages/EnvironmentPage";
import { CatalogPage } from "@/pages/CatalogPage";
import { PresetsPage } from "@/pages/PresetsPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { CreateEnvironmentDialog } from "@/components/environments/CreateEnvironmentDialog";

function App() {
  const [activePage, setActivePage] = useState("env:default");
  const [newEnvOpen, setNewEnvOpen] = useState(false);

  function renderPage() {
    if (activePage.startsWith("env:")) {
      const envId = activePage.slice(4);
      return <EnvironmentPage envId={envId} />;
    }

    switch (activePage) {
      case "catalog":
        return <CatalogPage />;
      case "presets":
        return <PresetsPage />;
      case "settings":
        return <SettingsPage />;
      default:
        return <EnvironmentPage envId="default" />;
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
    </Layout>
  );
}

export default App;
