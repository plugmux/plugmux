import { useState, useMemo } from "react";
import { Search } from "lucide-react";
import { Input } from "@/components/ui/input";
import { CatalogCard } from "@/components/catalog/CatalogCard";
import { CatalogDetail } from "@/components/catalog/CatalogDetail";
import { CategoryFilter } from "@/components/catalog/CategoryFilter";
import { useCatalog } from "@/hooks/useCatalog";
import { useConfig } from "@/hooks/useConfig";
import type { CatalogEntry } from "@/lib/commands";

export function CatalogPage() {
  const { servers, loading } = useCatalog();
  const { config, addServerToEnv } = useConfig();

  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [detailEntry, setDetailEntry] = useState<CatalogEntry | null>(null);

  const environments = config?.environments ?? [];

  // Extract unique categories
  const categories = useMemo(() => {
    const cats = new Set(servers.map((s) => s.category));
    return Array.from(cats).sort();
  }, [servers]);

  // Filter by search + category
  const filtered = useMemo(() => {
    const q = searchQuery.toLowerCase();
    return servers.filter((entry) => {
      const matchesSearch =
        !q ||
        entry.name.toLowerCase().includes(q) ||
        entry.description.toLowerCase().includes(q);
      const matchesCategory =
        !selectedCategory || entry.category === selectedCategory;
      return matchesSearch && matchesCategory;
    });
  }, [servers, searchQuery, selectedCategory]);

  // For each catalog entry, find which environments it's installed in
  function getInstalledIn(entryId: string): string[] {
    return environments
      .filter((env) => env.servers.includes(entryId))
      .map((env) => env.id);
  }

  async function handleAdd(entryId: string, envId: string) {
    await addServerToEnv(envId, entryId);
  }

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center p-6">
        <p className="text-muted-foreground">Loading catalog...</p>
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold">Catalog</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Browse and install community MCP servers.
        </p>
      </div>

      {/* Search */}
      <div className="relative mb-4">
        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          placeholder="Search servers..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="pl-9"
        />
      </div>

      {/* Category filter */}
      {categories.length > 0 && (
        <div className="mb-6">
          <CategoryFilter
            categories={categories}
            selected={selectedCategory}
            onSelect={setSelectedCategory}
          />
        </div>
      )}

      {/* Grid of cards */}
      {filtered.length === 0 ? (
        <div className="flex flex-col items-center gap-2 py-12">
          <p className="text-muted-foreground">No servers found.</p>
          {searchQuery && (
            <p className="text-xs text-muted-foreground">
              Try a different search term or clear the filter.
            </p>
          )}
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {filtered.map((entry) => (
            <CatalogCard
              key={entry.id}
              entry={entry}
              installedIn={getInstalledIn(entry.id)}
              environments={environments}
              onAdd={(envId) => handleAdd(entry.id, envId)}
              onClick={() => setDetailEntry(entry)}
            />
          ))}
        </div>
      )}

      {/* Detail dialog */}
      {detailEntry && (
        <CatalogDetail
          entry={detailEntry}
          installedIn={getInstalledIn(detailEntry.id)}
          environments={environments}
          onAdd={(envId) => handleAdd(detailEntry.id, envId)}
          onClose={() => setDetailEntry(null)}
        />
      )}
    </div>
  );
}
