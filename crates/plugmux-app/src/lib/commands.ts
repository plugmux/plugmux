import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface Config {
  port: number;
  permissions: Permissions;
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
  categories?: string[];
  transport: "stdio" | "http";
  command?: string;
  args?: string[];
  url?: string;
  connectivity: "local" | "online";
  official?: boolean;
  installs?: number;
  added?: string;
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

// ---------------------------------------------------------------------------
// Agent types
// ---------------------------------------------------------------------------

export interface AgentEntry {
  id: string;
  name: string;
  icon: string | null;
  config_format: "json" | "toml";
  mcp_key: string;
  tier: "auto" | "manual";
  config_paths: {
    macos: string | null;
    linux: string | null;
    windows: string | null;
  } | null;
}

export interface DetectedAgent {
  id: string;
  name: string;
  icon: string | null;
  config_path: string | null;
  installed: boolean;
  status: "green" | "yellow" | "gray";
  source: string;
  tier: "auto" | "manual" | "custom";
  install_url: string | null;
  setup_hint: string | null;
}

// ---------------------------------------------------------------------------
// Agent commands
// ---------------------------------------------------------------------------

export const getAgentRegistry = () =>
  invoke<AgentEntry[]>("get_agent_registry");

export const detectAgents = () =>
  invoke<DetectedAgent[]>("detect_agents");

export const connectAgent = (agentId: string) =>
  invoke<string | null>("connect_agent_cmd", { agentId });

export const disconnectAgent = (agentId: string, restore: boolean) =>
  invoke<void>("disconnect_agent_cmd", { agentId, restore });

export const hasAgentBackup = (agentId: string) =>
  invoke<boolean>("has_agent_backup", { agentId });

export const addAgentFromRegistry = (agentId: string, configPath: string) =>
  invoke<void>("add_agent_from_registry", { agentId, configPath });

export const addCustomAgent = (
  name: string,
  configPath: string,
  configFormat: string,
  mcpKey: string,
) => invoke<void>("add_custom_agent", { name, configPath, configFormat, mcpKey });

export const dismissAgent = (agentId: string) =>
  invoke<void>("dismiss_agent", { agentId });

// ---------------------------------------------------------------------------
// Cloud API types
// ---------------------------------------------------------------------------

export interface RemoteCatalogServer {
  id: string;
  name: string;
  description: string;
  icon_key: string | null;
  icon_hash: string | null;
  categories: string[];
  transport: "stdio" | "http";
  command: string | null;
  args: string[] | null;
  url: string | null;
  connectivity: "local" | "online";
  official: boolean;
  tool_count: number | null;
  security_score: string | null;
  smithery_url: string | null;
  added_at: string;
  updated_at: string;
}

export interface RemoteCatalogResponse {
  servers: RemoteCatalogServer[];
  next_cursor: string | null;
}

export interface RemoteCollection {
  id: string;
  name: string;
  description: string;
  icon: string | null;
  sort_order: number;
  server_ids?: string[];
  servers?: RemoteCatalogServer[];
}

export interface RemoteCollectionsResponse {
  collections: RemoteCollection[];
}

export interface ApiHealthResponse {
  status: string;
  version: string;
}

export interface AuthUser {
  id: string;
  github_username: string;
  email: string | null;
}

// ---------------------------------------------------------------------------
// Cloud API commands
// ---------------------------------------------------------------------------

export const apiHealth = () =>
  invoke<ApiHealthResponse>("api_health");

export const apiListServers = (opts?: {
  limit?: number;
  cursor?: string;
  search?: string;
  category?: string;
}) =>
  invoke<RemoteCatalogResponse>("api_list_servers", {
    limit: opts?.limit ?? null,
    cursor: opts?.cursor ?? null,
    search: opts?.search ?? null,
    category: opts?.category ?? null,
  });

export const apiGetServer = (id: string) =>
  invoke<RemoteCatalogServer>("api_get_server", { id });

export const apiListCollections = () =>
  invoke<RemoteCollectionsResponse>("api_list_collections");

export const apiGetCollection = (id: string) =>
  invoke<RemoteCollection>("api_get_collection", { id });

export const apiGetAuthUrl = () =>
  invoke<string>("api_get_auth_url");

export const apiSetToken = (token: string) =>
  invoke<void>("api_set_token", { token });

export const apiGetProfile = () =>
  invoke<AuthUser>("api_get_profile");

export const apiGetBaseUrl = () =>
  invoke<string>("api_get_base_url");
