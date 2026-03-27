import { useState, useEffect, useCallback } from "react";
import {
  getConfig,
  listEnvironments,
  createEnvironment,
  deleteEnvironment,
  renameEnvironment,
  addServerToEnv,
  removeServerFromEnv,
} from "@/lib/commands";
import type { Config, Environment } from "@/lib/commands";
import { useEvents } from "./useEvents";

export function useConfig() {
  const [config, setConfig] = useState<Config | null>(null);
  const [environments, setEnvironments] = useState<Environment[]>([]);
  const [loading, setLoading] = useState(true);

  const reload = useCallback(async () => {
    const [cfg, envs] = await Promise.all([getConfig(), listEnvironments()]);
    setConfig(cfg);
    setEnvironments(envs);
    setLoading(false);
  }, []);

  useEffect(() => {
    reload();
  }, [reload]);

  useEvents("server_added", reload);
  useEvents("server_removed", reload);
  useEvents("environment_created", reload);
  useEvents("environment_deleted", reload);
  useEvents("config_reloaded", reload);

  return {
    config,
    environments,
    loading,
    reload,
    createEnvironment: async (name: string): Promise<Environment> => {
      return await createEnvironment(name);
    },
    deleteEnvironment: async (id: string): Promise<void> => {
      await deleteEnvironment(id);
    },
    renameEnvironment: async (id: string, name: string): Promise<void> => {
      await renameEnvironment(id, name);
    },
    addServerToEnv: async (
      envId: string,
      serverId: string,
    ): Promise<void> => {
      await addServerToEnv(envId, serverId);
    },
    removeServerFromEnv: async (
      envId: string,
      serverId: string,
    ): Promise<void> => {
      await removeServerFromEnv(envId, serverId);
    },
  };
}
