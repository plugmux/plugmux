import { useState, useEffect, useCallback } from "react";
import {
  listCustomServers,
  addCustomServer,
  updateCustomServer,
  removeCustomServer,
} from "@/lib/commands";
import type { ServerConfig } from "@/lib/commands";
import { useEvents } from "./useEvents";

export function useCustomServers() {
  const [servers, setServers] = useState<ServerConfig[]>([]);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    const s = await listCustomServers();
    setServers(s);
    setLoading(false);
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  useEvents("custom_server_added", load);
  useEvents("custom_server_updated", load);
  useEvents("custom_server_removed", load);

  return {
    servers,
    loading,
    reload: load,
    addServer: async (config: ServerConfig): Promise<void> => {
      await addCustomServer(config);
    },
    updateServer: async (id: string, config: ServerConfig): Promise<void> => {
      await updateCustomServer(id, config);
    },
    removeServer: async (id: string): Promise<void> => {
      await removeCustomServer(id);
    },
  };
}
