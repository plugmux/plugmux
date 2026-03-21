import { invoke } from "@tauri-apps/api/core";

export interface ServerConfig {
  id: string;
  name: string;
  transport: "stdio" | "http";
  command?: string;
  args?: string[];
  url?: string;
  connectivity: "local" | "online";
  enabled: boolean;
  description?: string;
}

export interface ServerOverride {
  server_id: string;
  enabled?: boolean;
}

export interface EnvironmentConfig {
  id: string;
  name: string;
  endpoint: string;
  servers: ServerConfig[];
  overrides: ServerOverride[];
}

export interface PlugmuxConfig {
  main: { servers: ServerConfig[] };
  environments: EnvironmentConfig[];
}

// Engine
export const getEngineStatus = () => invoke<string>("get_engine_status");
export const startEngine = () => invoke<void>("start_engine");
export const stopEngine = () => invoke<void>("stop_engine");

// Config
export const getConfig = () => invoke<PlugmuxConfig>("get_config");
export const getMainServers = () => invoke<ServerConfig[]>("get_main_servers");
export const addMainServer = (config: ServerConfig) =>
  invoke<void>("add_main_server", { config });
export const removeMainServer = (id: string) =>
  invoke<void>("remove_main_server", { id });
export const toggleMainServer = (id: string) =>
  invoke<void>("toggle_main_server", { id });
export const renameServer = (id: string, name: string) =>
  invoke<void>("rename_server", { id, name });

// Environments
export const listEnvironments = () =>
  invoke<EnvironmentConfig[]>("list_environments");
export const createEnvironment = (name: string) =>
  invoke<EnvironmentConfig>("create_environment", { name });
export const deleteEnvironment = (id: string) =>
  invoke<void>("delete_environment", { id });
export const renameEnvironment = (id: string, name: string) =>
  invoke<void>("rename_environment", { id, name });
export const addEnvServer = (envId: string, config: ServerConfig) =>
  invoke<void>("add_env_server", { envId, config });
export const removeEnvServer = (envId: string, serverId: string) =>
  invoke<void>("remove_env_server", { envId, serverId });
export const toggleEnvOverride = (envId: string, serverId: string) =>
  invoke<void>("toggle_env_override", { envId, serverId });

// Settings
export const getPort = () => invoke<number>("get_port");
export const setPort = (port: number) => invoke<void>("set_port", { port });
