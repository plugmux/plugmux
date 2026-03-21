import { useState, useEffect, useCallback } from "react";
import {
  getConfig,
  addMainServer,
  removeMainServer,
  toggleMainServer,
  createEnvironment,
  deleteEnvironment,
  addEnvServer,
  removeEnvServer,
  toggleEnvOverride,
} from "@/lib/commands";
import type { PlugmuxConfig, ServerConfig } from "@/lib/commands";
import { useEvents } from "./useEvents";

export function useConfig() {
  const [config, setConfig] = useState<PlugmuxConfig | null>(null);
  const [loading, setLoading] = useState(true);

  const reload = useCallback(async () => {
    const cfg = await getConfig();
    setConfig(cfg);
    setLoading(false);
  }, []);

  useEffect(() => {
    reload();
  }, [reload]);

  useEvents("server_added", reload);
  useEvents("server_removed", reload);
  useEvents("server_toggled", reload);
  useEvents("environment_created", reload);
  useEvents("environment_deleted", reload);
  useEvents("config_reloaded", reload);

  return {
    config,
    loading,
    reload,
    addMainServer: async (server: ServerConfig) => {
      await addMainServer(server);
    },
    removeMainServer: async (id: string) => {
      await removeMainServer(id);
    },
    toggleMainServer: async (id: string) => {
      await toggleMainServer(id);
    },
    createEnvironment: async (name: string) => {
      return await createEnvironment(name);
    },
    deleteEnvironment: async (id: string) => {
      await deleteEnvironment(id);
    },
    addEnvServer: async (envId: string, server: ServerConfig) => {
      await addEnvServer(envId, server);
    },
    removeEnvServer: async (envId: string, serverId: string) => {
      await removeEnvServer(envId, serverId);
    },
    toggleEnvOverride: async (envId: string, serverId: string) => {
      await toggleEnvOverride(envId, serverId);
    },
  };
}
