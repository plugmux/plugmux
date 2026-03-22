import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface Config {
  port: number;
  permissions: Permissions;
  environments: Environment[];
}

export interface Permissions {
  enable_server: "allow" | "approve" | "disable";
  disable_server: "allow" | "approve" | "disable";
}

export interface Environment {
  id: string;
  name: string;
  servers: string[];
}

export interface ServerConfig {
  id: string;
  name: string;
  transport: "stdio" | "http";
  command?: string;
  args?: string[];
  url?: string;
  connectivity: "local" | "online";
  description?: string;
}

export interface CatalogEntry {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  transport: "stdio" | "http";
  command?: string;
  args?: string[];
  url?: string;
  connectivity: "local" | "online";
}

export interface Preset {
  id: string;
  name: string;
  description: string;
  icon: string;
  servers: string[];
}

export type HealthStatus =
  | { status: "healthy" }
  | { status: "degraded"; reason: string }
  | { status: "unavailable"; reason: string };

// ---------------------------------------------------------------------------
// Engine commands
// ---------------------------------------------------------------------------

export const getEngineStatus = () => invoke<string>("get_engine_status");
export const startEngine = () => invoke<void>("start_engine");
export const stopEngine = () => invoke<void>("stop_engine");

// ---------------------------------------------------------------------------
// Config commands
// ---------------------------------------------------------------------------

export const getConfig = () => invoke<Config>("get_config");
export const getPort = () => invoke<number>("get_port");
export const setPort = (port: number) => invoke<void>("set_port", { port });

// ---------------------------------------------------------------------------
// Permission commands
// ---------------------------------------------------------------------------

export const getPermissions = () => invoke<Permissions>("get_permissions");
export const setPermission = (
  action: string,
  level: "allow" | "approve" | "disable",
) => invoke<void>("set_permission", { action, level });

// ---------------------------------------------------------------------------
// Environment commands
// ---------------------------------------------------------------------------

export const listEnvironments = () =>
  invoke<Environment[]>("list_environments");
export const createEnvironment = (name: string) =>
  invoke<Environment>("create_environment", { name });
export const deleteEnvironment = (id: string) =>
  invoke<void>("delete_environment", { id });
export const renameEnvironment = (id: string, name: string) =>
  invoke<void>("rename_environment", { id, name });

// ---------------------------------------------------------------------------
// Server-in-environment commands
// ---------------------------------------------------------------------------

export const addServerToEnv = (envId: string, serverId: string) =>
  invoke<void>("add_server_to_env", { envId: envId, serverId: serverId });
export const removeServerFromEnv = (envId: string, serverId: string) =>
  invoke<void>("remove_server_from_env", { envId: envId, serverId: serverId });

// ---------------------------------------------------------------------------
// Custom server commands
// ---------------------------------------------------------------------------

export const listCustomServers = () =>
  invoke<ServerConfig[]>("list_custom_servers");
export const addCustomServer = (config: ServerConfig) =>
  invoke<void>("add_custom_server", { config });
export const updateCustomServer = (id: string, config: ServerConfig) =>
  invoke<void>("update_custom_server", { id, config });
export const removeCustomServer = (id: string) =>
  invoke<void>("remove_custom_server", { id });

// ---------------------------------------------------------------------------
// Catalog commands
// ---------------------------------------------------------------------------

export const listCatalogServers = () =>
  invoke<CatalogEntry[]>("list_catalog_servers");
export const searchCatalog = (query: string, category: string | null) =>
  invoke<CatalogEntry[]>("search_catalog", { query, category });
export const getCatalogEntry = (id: string) =>
  invoke<CatalogEntry>("get_catalog_entry", { id });

// ---------------------------------------------------------------------------
// Preset commands
// ---------------------------------------------------------------------------

export const listPresets = () => invoke<Preset[]>("list_presets");
export const createEnvFromPreset = (presetId: string, name: string) =>
  invoke<Environment>("create_env_from_preset", { presetId, name });

// ---------------------------------------------------------------------------
// Health commands
// ---------------------------------------------------------------------------

export const getServerHealth = (serverId: string) =>
  invoke<HealthStatus>("get_server_health", { serverId });

// ---------------------------------------------------------------------------
// Migration commands
// ---------------------------------------------------------------------------

export const migrateConfig = () => invoke<void>("migrate_config");
