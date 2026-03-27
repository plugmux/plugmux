import { useState, useEffect, useCallback } from "react";
import {
  apiListServers,
  apiListCollections,
  listCatalogServers,
  listPresets,
} from "@/lib/commands";
import type {
  CatalogEntry,
  Preset,
  RemoteCatalogServer,
  RemoteCollection,
} from "@/lib/commands";

export function useCatalog() {
  const [servers, setServers] = useState<RemoteCatalogServer[]>([]);
  const [collections, setCollections] = useState<RemoteCollection[]>([]);
  // Keep bundled data as fallback
  const [bundledServers, setBundledServers] = useState<CatalogEntry[]>([]);
  const [presets, setPresets] = useState<Preset[]>([]);
  const [loading, setLoading] = useState(true);
  const [isRemote, setIsRemote] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  async function loadData() {
    setLoading(true);
    try {
      // Try remote API first
      const [catalogRes, collectionsRes] = await Promise.all([
        apiListServers({ limit: 200 }),
        apiListCollections(),
      ]);

      if (catalogRes.servers.length > 0) {
        setServers(catalogRes.servers);
        setCollections(collectionsRes.collections);
        setIsRemote(true);
        setLoading(false);
        return;
      }
    } catch {
      // API unavailable — fall through to bundled
    }

    // Fallback to bundled catalog
    try {
      const [s, p] = await Promise.all([listCatalogServers(), listPresets()]);
      setBundledServers(s);
      setPresets(p);
      // Convert bundled to remote format for uniform rendering
      setServers(
        s.map((entry) => ({
          id: entry.id,
          name: entry.name,
          description: entry.description,
          icon_key: entry.icon || null,
          icon_hash: null,
          categories: entry.categories ?? [entry.category],
          transport: entry.transport,
          command: entry.command ?? null,
          args: entry.args ?? null,
          url: entry.url ?? null,
          connectivity: entry.connectivity,
          official: entry.official ?? false,
          tool_count: null,
          security_score: null,
          smithery_url: null,
          added_at: entry.added ?? "",
          updated_at: entry.added ?? "",
        })),
      );
      setIsRemote(false);
    } catch {
      // Both failed
    }
    setLoading(false);
  }

  const search = useCallback(
    async (
      query: string,
      category?: string,
    ): Promise<RemoteCatalogServer[]> => {
      if (isRemote) {
        try {
          const res = await apiListServers({ search: query, category });
          return res.servers;
        } catch {
          return [];
        }
      }
      // Fallback: client-side filter
      const q = query.toLowerCase();
      return servers.filter(
        (s) =>
          s.name.toLowerCase().includes(q) ||
          s.description.toLowerCase().includes(q),
      );
    },
    [isRemote, servers],
  );

  return {
    servers,
    collections,
    bundledServers,
    presets,
    loading,
    isRemote,
    search,
    reload: loadData,
  };
}
