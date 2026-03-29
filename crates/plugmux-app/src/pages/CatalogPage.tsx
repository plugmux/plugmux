import { useState, useMemo, useEffect } from "react";
import { Search } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { CatalogCard } from "@/components/catalog/CatalogCard";
import { CatalogDetail } from "@/components/catalog/CatalogDetail";
import { CategoryFilter } from "@/components/catalog/CategoryFilter";
import { Pagination } from "@/components/catalog/Pagination";
import { useCatalog } from "@/hooks/useCatalog";
import { useConfig } from "@/hooks/useConfig";
import type { RemoteCatalogServer } from "@/lib/commands";

const CATEGORIES = [
  { id: "developer-tools", label: "Development" },
  { id: "databases", label: "Databases" },
  { id: "design", label: "Design Tools" },
  { id: "browser-automation", label: "Browser & Testing" },
  { id: "search-data-extraction", label: "Search & Research" },
  { id: "communication", label: "Communication" },
  { id: "workplace-and-productivity", label: "Productivity" },
  { id: "cloud-platforms", label: "Cloud & DevOps" },
  { id: "knowledge-and-memory", label: "Knowledge & Memory" },
  { id: "monitoring", label: "Monitoring" },
  { id: "finance-and-fintech", label: "Finance" },
  { id: "security", label: "Security" },
  { id: "version-control", label: "Version Control" },
  { id: "coding-agents", label: "Coding Agents" },
  { id: "data-science-tools", label: "Data Science" },
  { id: "social-media", label: "Social Media" },
];

const PER_PAGE = 12;

const Tab = {
  DISCOVER: "discover",
  COLLECTIONS: "collections",
  BOOKMARKS: "bookmarks",
  INSTALLED: "installed",
} as const;

type TabValue = (typeof Tab)[keyof typeof Tab];

function emptyTitle(tab: TabValue): string {
  switch (tab) {
    case Tab.BOOKMARKS: return "No bookmarked servers yet";
    case Tab.INSTALLED: return "No installed servers";
    default: return "No servers match your filters";
  }
}

function emptySubtitle(tab: TabValue): string {
  switch (tab) {
    case Tab.BOOKMARKS: return "Bookmark servers from the Discover tab to save them here";
    case Tab.INSTALLED: return "Add servers to an environment to see them here";
    default: return "Try adjusting your search or category filters";
  }
}

/** Shape expected by CatalogCard/CatalogDetail (legacy) */
interface CatalogEntry {
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

function toCatalogEntry(s: RemoteCatalogServer): CatalogEntry {
  return {
    id: s.id,
    name: s.name,
    description: s.description,
    icon: s.icon_key ?? "",
    category: s.categories[0] ?? "",
    categories: s.categories,
    transport: s.transport,
    command: s.command ?? undefined,
    args: s.args ?? undefined,
    url: s.url ?? undefined,
    connectivity: s.connectivity,
    official: s.official,
    installs: s.tool_count ?? undefined,
    added: s.added_at,
  };
}

export function CatalogPage() {
  const { servers, collections, loading, error } = useCatalog();
  const { environments, addServerToEnv } = useConfig();

  const [tab, setTab] = useState<TabValue>(Tab.DISCOVER);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategories, setSelectedCategories] = useState<string[]>([]);
  const [sort, setSort] = useState("popular");
  const [page, setPage] = useState(1);
  const [detailEntry, setDetailEntry] = useState<CatalogEntry | null>(null);
  const [activeCollection, setActiveCollection] = useState<string>("all");
  const [bookmarks, setBookmarks] = useState<Set<string>>(() => {
    try {
      const saved = localStorage.getItem("plugmux-bookmarks");
      return saved ? new Set(JSON.parse(saved)) : new Set();
    } catch {
      return new Set();
    }
  });

  // Persist bookmarks
  useEffect(() => {
    localStorage.setItem("plugmux-bookmarks", JSON.stringify([...bookmarks]));
  }, [bookmarks]);

  // Reset page on filter change
  useEffect(() => {
    setPage(1);
  }, [tab, searchQuery, selectedCategories, sort]);

  function toggleBookmark(id: string) {
    setBookmarks((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  function getInstalledIn(entryId: string): string[] {
    return environments
      .filter((env) => env.servers.includes(entryId))
      .map((env) => env.id);
  }

  // Derive categories from actual data
  const availableCategories = useMemo(() => {
    const catIds = new Set<string>();
    for (const s of servers) {
      for (const c of s.categories) {
        if (c) catIds.add(c);
      }
    }
    const known = CATEGORIES.filter((c) => catIds.has(c.id));
    const knownIds = new Set(known.map((c) => c.id));
    const unknown = [...catIds]
      .filter((id) => !knownIds.has(id))
      .map((id) => ({ id, label: id.replace(/-/g, " ").replace(/\b\w/g, (c) => c.toUpperCase()) }));
    return [...known, ...unknown];
  }, [servers]);

  // Filter + sort
  const filtered = useMemo(() => {
    const q = searchQuery.toLowerCase();

    let result = servers.filter((s) => {
      // Tab filter
      if (tab === Tab.BOOKMARKS && !bookmarks.has(s.id)) return false;
      if (tab === Tab.INSTALLED && getInstalledIn(s.id).length === 0) return false;

      // Category filter
      if (selectedCategories.length > 0) {
        if (!s.categories.some((c) => selectedCategories.includes(c))) return false;
      }

      // Search
      if (q) {
        return (
          s.name.toLowerCase().includes(q) ||
          s.description.toLowerCase().includes(q) ||
          s.categories.some((c) => c.toLowerCase().includes(q))
        );
      }

      return true;
    });

    // Sort
    result = [...result];
    if (sort === "popular") {
      result.sort((a, b) => (b.tool_count ?? 0) - (a.tool_count ?? 0));
    } else if (sort === "name") {
      result.sort((a, b) => a.name.localeCompare(b.name));
    } else if (sort === "recent") {
      result.sort((a, b) => (b.added_at ?? "").localeCompare(a.added_at ?? ""));
    }

    return result;
  }, [servers, tab, searchQuery, selectedCategories, sort, bookmarks]);

  const total = filtered.length;
  const paged = filtered.slice((page - 1) * PER_PAGE, page * PER_PAGE);

  const installedCount = servers.filter((s) => getInstalledIn(s.id).length > 0).length;
  const bookmarkCount = [...bookmarks].filter((id) => servers.some((s) => s.id === id)).length;

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

  if (error) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-2 p-6">
        <p className="text-lg font-medium text-muted-foreground">Could not connect to API</p>
        <p className="text-sm text-muted-foreground/60">{error}</p>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto p-6">
        {/* Title */}
        <div className="mb-6 flex items-center justify-between">
          <h1 className="text-2xl font-bold tracking-tight">MCP Catalog</h1>
        </div>

        {/* Tabs */}
        <Tabs value={tab} onValueChange={(v) => setTab(v as TabValue)} className="mb-5">
          <TabsList>
            <TabsTrigger value={Tab.DISCOVER}>Discover</TabsTrigger>
            <TabsTrigger value={Tab.COLLECTIONS} className="gap-1.5">
              Collections
              {collections.length > 0 && (
                <Badge variant="secondary" className="ml-1 h-5 px-1.5 text-[11px]">
                  {collections.length}
                </Badge>
              )}
            </TabsTrigger>
            <TabsTrigger value={Tab.BOOKMARKS} className="gap-1.5">
              Bookmarked
              {bookmarkCount > 0 && (
                <Badge variant="secondary" className="ml-1 h-5 px-1.5 text-[11px]">
                  {bookmarkCount}
                </Badge>
              )}
            </TabsTrigger>
            <TabsTrigger value={Tab.INSTALLED} className="gap-1.5">
              Installed
              {installedCount > 0 && (
                <Badge variant="secondary" className="ml-1 h-5 px-1.5 text-[11px]">
                  {installedCount}
                </Badge>
              )}
            </TabsTrigger>
          </TabsList>
        </Tabs>

        {/* Collections — sections with titles + same cards */}
        {tab === Tab.COLLECTIONS && (
          <>
            {collections.length === 0 ? (
              <div className="flex flex-col items-center gap-2 py-16">
                <p className="text-lg font-medium text-muted-foreground">No collections yet</p>
                <p className="text-sm text-muted-foreground/60">
                  Collections will appear when the API is connected
                </p>
              </div>
            ) : (
              <>
                {/* Collection filter chips */}
                <div className="mb-5 flex flex-wrap gap-2">
                  <button
                    onClick={() => setActiveCollection("all")}
                    className={`rounded-full border px-3 py-1 text-sm font-medium transition-colors ${
                      activeCollection === "all"
                        ? "border-primary bg-primary text-primary-foreground"
                        : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                    }`}
                  >
                    All
                  </button>
                  {collections
                    .filter((col) => (col.server_ids ?? []).some((sid) => servers.some((s) => s.id === sid)))
                    .map((col) => (
                      <button
                        key={col.id}
                        onClick={() => setActiveCollection(col.id)}
                        className={`rounded-full border px-3 py-1 text-sm font-medium transition-colors ${
                          activeCollection === col.id
                            ? "border-primary bg-primary text-primary-foreground"
                            : "border-border bg-background text-muted-foreground hover:bg-accent hover:text-accent-foreground"
                        }`}
                      >
                        For {col.name}
                      </button>
                    ))}
                </div>

                {/* Sections */}
                <div className="space-y-8">
                  {collections
                    .filter((col) => activeCollection === "all" || activeCollection === col.id)
                    .map((col) => {
                      const colServers = servers.filter((s) =>
                        (col.server_ids ?? []).includes(s.id)
                      );
                      if (colServers.length === 0) return null;
                      return (
                        <section key={col.id}>
                          <h2 className="mb-3 text-lg font-semibold">
                            For {col.name}
                          </h2>
                          <div className="grid grid-cols-1 gap-3.5 md:grid-cols-2 xl:grid-cols-3">
                            {colServers.map((entry) => (
                              <CatalogCard
                                key={entry.id}
                                entry={toCatalogEntry(entry)}
                                installedIn={getInstalledIn(entry.id)}
                                environments={environments}
                                isBookmarked={bookmarks.has(entry.id)}
                                onAdd={(envId) => handleAdd(entry.id, envId)}
                                onToggleBookmark={() => toggleBookmark(entry.id)}
                                onClick={() => setDetailEntry(toCatalogEntry(entry))}
                              />
                            ))}
                          </div>
                        </section>
                      );
                    })}
                </div>
              </>
            )}
          </>
        )}

        {/* Server list — Discover, Bookmarks, Installed */}
        {tab !== Tab.COLLECTIONS && (
          <>
            <div className="mb-5 flex flex-wrap items-center gap-2.5">
              <div className="relative min-w-[200px] flex-1">
                <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                <Input
                  placeholder="Search servers..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-9"
                />
              </div>
              {tab === Tab.DISCOVER && (
                <CategoryFilter
                  categories={availableCategories}
                  selected={selectedCategories}
                  onSelect={setSelectedCategories}
                />
              )}
            </div>

            {/* Results bar */}
            <div className="mb-4 flex flex-wrap items-center justify-between gap-2.5">
              <div className="flex items-center gap-3">
                <span className="font-mono text-sm text-muted-foreground">
                  {total} server{total !== 1 ? "s" : ""}
                </span>
                {tab === Tab.DISCOVER && (
                  <Select value={sort} onValueChange={setSort}>
                    <SelectTrigger className="h-8 w-[150px]">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="popular">Most popular</SelectItem>
                      <SelectItem value="name">A → Z</SelectItem>
                      <SelectItem value="recent">Recently added</SelectItem>
                    </SelectContent>
                  </Select>
                )}
              </div>
              <Pagination total={total} page={page} perPage={PER_PAGE} onChange={setPage} />
            </div>

            {/* Card grid */}
            {paged.length === 0 ? (
              <div className="flex flex-col items-center gap-2 py-16">
                <p className="text-lg font-medium text-muted-foreground">
                  {emptyTitle(tab)}
                </p>
                <p className="text-sm text-muted-foreground/60">
                  {emptySubtitle(tab)}
                </p>
              </div>
            ) : (
              <div className="grid grid-cols-1 gap-3.5 md:grid-cols-2 xl:grid-cols-3">
                {paged.map((entry) => (
                  <CatalogCard
                    key={entry.id}
                    entry={toCatalogEntry(entry)}
                    installedIn={getInstalledIn(entry.id)}
                    environments={environments}
                    isBookmarked={bookmarks.has(entry.id)}
                    onAdd={(envId) => handleAdd(entry.id, envId)}
                    onToggleBookmark={() => toggleBookmark(entry.id)}
                    onClick={() => setDetailEntry(toCatalogEntry(entry))}
                  />
                ))}
              </div>
            )}

            {/* Bottom pagination */}
            {total > PER_PAGE && (
              <div className="mt-6 flex justify-center">
                <Pagination total={total} page={page} perPage={PER_PAGE} onChange={setPage} />
              </div>
            )}
          </>
        )}

        {/* Bookmarks / Installed tab — reuse same card grid */}
        {/* Bookmarks / Installed — rendered via shared ServerGrid below */}

        <div className="h-10" />
      </div>

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
