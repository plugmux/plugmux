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
import type { CatalogEntry } from "@/lib/commands";

const CATEGORIES = [
  { id: "dev-tools", label: "Development" },
  { id: "database", label: "Databases" },
  { id: "design", label: "Design Tools" },
  { id: "browser", label: "Browser & Testing" },
  { id: "search", label: "Search & Research" },
  { id: "communication", label: "Communication" },
  { id: "productivity", label: "Productivity" },
  { id: "cloud", label: "Cloud & DevOps" },
  { id: "ai", label: "AI & ML" },
  { id: "ai-ml", label: "AI & ML" },
  { id: "cms", label: "CMS & Content" },
  { id: "monitoring", label: "Monitoring" },
  { id: "payments", label: "Payments" },
  { id: "automation", label: "Automation" },
  { id: "security", label: "Security" },
  { id: "data-science", label: "Data Science" },
];

const PER_PAGE = 12;

export function CatalogPage() {
  const { servers, loading } = useCatalog();
  const { environments, addServerToEnv } = useConfig();

  const [tab, setTab] = useState("discover");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategories, setSelectedCategories] = useState<string[]>([]);
  const [sort, setSort] = useState("popular");
  const [page, setPage] = useState(1);
  const [detailEntry, setDetailEntry] = useState<CatalogEntry | null>(null);
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
    localStorage.setItem(
      "plugmux-bookmarks",
      JSON.stringify([...bookmarks]),
    );
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

  // Derive categories from actual data, mapped to labels
  const availableCategories = useMemo(() => {
    const catIds = new Set<string>();
    for (const s of servers) {
      const cats = Array.isArray(s.categories)
        ? s.categories
        : [s.category];
      cats.filter(Boolean).forEach((c: string) => catIds.add(c));
    }
    // Return known categories that exist in data, plus any unknown ones
    const known = CATEGORIES.filter((c) => catIds.has(c.id));
    const knownIds = new Set(known.map((c) => c.id));
    const unknown = [...catIds]
      .filter((id) => !knownIds.has(id))
      .map((id) => ({ id, label: id }));
    return [...known, ...unknown];
  }, [servers]);

  // Filter + sort
  const filtered = useMemo(() => {
    const q = searchQuery.toLowerCase();

    let result = servers.filter((s) => {
      // Tab filter
      if (tab === "bookmarks" && !bookmarks.has(s.id)) return false;
      if (tab === "installed" && getInstalledIn(s.id).length === 0)
        return false;

      // Category filter (multi)
      if (selectedCategories.length > 0) {
        const serverCats: string[] = Array.isArray(s.categories)
          ? s.categories
          : [s.category];
        if (!serverCats.some((c) => selectedCategories.includes(c)))
          return false;
      }

      // Search
      if (q) {
        return (
          s.name.toLowerCase().includes(q) ||
          s.description.toLowerCase().includes(q) ||
          s.category.toLowerCase().includes(q)
        );
      }

      return true;
    });

    // Sort
    result = [...result];
    if (sort === "popular") {
      result.sort((a, b) => (b.installs ?? 0) - (a.installs ?? 0));
    } else if (sort === "name") {
      result.sort((a, b) => a.name.localeCompare(b.name));
    } else if (sort === "recent") {
      result.sort((a, b) => (b.added ?? "").localeCompare(a.added ?? ""));
    }

    return result;
  }, [servers, tab, searchQuery, selectedCategories, sort, bookmarks]);

  const total = filtered.length;
  const paged = filtered.slice((page - 1) * PER_PAGE, page * PER_PAGE);

  const installedCount = servers.filter(
    (s) => getInstalledIn(s.id).length > 0,
  ).length;
  const bookmarkCount = [...bookmarks].filter((id) =>
    servers.some((s) => s.id === id),
  ).length;

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
    <div className="flex h-full flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto p-6">
        {/* Title */}
        <div className="mb-6">
          <h1 className="text-2xl font-bold tracking-tight">MCP Catalog</h1>
        </div>

        {/* Tabs */}
        <Tabs value={tab} onValueChange={setTab} className="mb-5">
          <TabsList>
            <TabsTrigger value="discover">Discover</TabsTrigger>
            <TabsTrigger value="bookmarks" className="gap-1.5">
              Bookmarked
              {bookmarkCount > 0 && (
                <Badge
                  variant="secondary"
                  className="ml-1 h-5 px-1.5 text-[11px]"
                >
                  {bookmarkCount}
                </Badge>
              )}
            </TabsTrigger>
            <TabsTrigger value="installed" className="gap-1.5">
              Installed
              {installedCount > 0 && (
                <Badge
                  variant="secondary"
                  className="ml-1 h-5 px-1.5 text-[11px]"
                >
                  {installedCount}
                </Badge>
              )}
            </TabsTrigger>
          </TabsList>
        </Tabs>

        {/* Filter bar (Discover tab only) */}
        {tab === "discover" && (
          <div className="mb-5 flex flex-wrap items-center gap-2.5">
            {/* Search */}
            <div className="relative min-w-[200px] flex-1">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="Search servers..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-9"
              />
            </div>

            {/* Category multi-select */}
            <CategoryFilter
              categories={availableCategories}
              selected={selectedCategories}
              onSelect={setSelectedCategories}
            />
          </div>
        )}

        {/* Results bar: count + sort + pagination */}
        <div className="mb-4 flex flex-wrap items-center justify-between gap-2.5">
          <div className="flex items-center gap-3">
            <span className="font-mono text-sm text-muted-foreground">
              {total} server{total !== 1 ? "s" : ""}
            </span>
            {tab === "discover" && (
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
          <Pagination
            total={total}
            page={page}
            perPage={PER_PAGE}
            onChange={setPage}
          />
        </div>

        {/* Card grid */}
        {paged.length === 0 ? (
          <div className="flex flex-col items-center gap-2 py-16">
            <p className="text-lg font-medium text-muted-foreground">
              {tab === "bookmarks"
                ? "No bookmarked servers yet"
                : tab === "installed"
                  ? "No installed servers"
                  : "No servers match your filters"}
            </p>
            <p className="text-sm text-muted-foreground/60">
              {tab === "discover"
                ? "Try adjusting your search or category filters"
                : "Browse Discover to find servers"}
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-3.5 md:grid-cols-2 xl:grid-cols-3">
            {paged.map((entry) => (
              <CatalogCard
                key={entry.id}
                entry={entry}
                installedIn={getInstalledIn(entry.id)}
                environments={environments}
                isBookmarked={bookmarks.has(entry.id)}
                onAdd={(envId) => handleAdd(entry.id, envId)}
                onToggleBookmark={() => toggleBookmark(entry.id)}
                onClick={() => setDetailEntry(entry)}
              />
            ))}
          </div>
        )}

        {/* Bottom pagination */}
        {total > PER_PAGE && (
          <div className="mt-6 flex justify-center">
            <Pagination
              total={total}
              page={page}
              perPage={PER_PAGE}
              onChange={setPage}
            />
          </div>
        )}

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
