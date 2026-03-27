import { useState, useEffect, useCallback } from "react";
import { apiListServers, apiListCollections } from "@/lib/commands";
import type { RemoteCatalogServer, RemoteCollection } from "@/lib/commands";

export function useCatalog() {
  const [servers, setServers] = useState<RemoteCatalogServer[]>([]);
  const [collections, setCollections] = useState<RemoteCollection[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  async function loadData() {
    setLoading(true);
    setError(null);
    try {
      const [catalogRes, collectionsRes] = await Promise.all([
        apiListServers({ limit: 200 }),
        apiListCollections(),
      ]);
      setServers(catalogRes.servers);
      setCollections(collectionsRes.collections);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to connect to API");
    }
    setLoading(false);
  }

  const search = useCallback(
    async (query: string, category?: string): Promise<RemoteCatalogServer[]> => {
      try {
        const res = await apiListServers({ search: query, category });
        return res.servers;
      } catch {
        return [];
      }
    },
    [],
  );

  return { servers, collections, loading, error, search, reload: loadData };
}
