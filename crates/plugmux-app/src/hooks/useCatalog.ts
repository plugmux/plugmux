import { useState, useEffect } from "react";
import { listCatalogServers, listPresets, searchCatalog } from "@/lib/commands";
import type { CatalogEntry, Preset } from "@/lib/commands";

export function useCatalog() {
  const [servers, setServers] = useState<CatalogEntry[]>([]);
  const [presets, setPresets] = useState<Preset[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([listCatalogServers(), listPresets()])
      .then(([s, p]) => {
        setServers(s);
        setPresets(p);
      })
      .finally(() => setLoading(false));
  }, []);

  const search = async (
    query: string,
    category?: string,
  ): Promise<CatalogEntry[]> => {
    return await searchCatalog(query, category ?? null);
  };

  return { servers, presets, loading, search };
}
