import { useState, useRef, useEffect } from "react";

// ── Fonts ──────────────────────────────────────────────────────────
const font = `'Geist', 'DM Sans', -apple-system, sans-serif`;
const mono = `'Geist Mono', 'JetBrains Mono', monospace`;

// ── Palette ────────────────────────────────────────────────────────
const C = {
  bg: "#09090b",
  surface: "#131316",
  surfaceHover: "#1a1a1f",
  surfaceActive: "#111114",
  border: "#23232b",
  borderHover: "#33333f",
  borderFocus: "#6c5ce7",
  text: "#ececf1",
  textSec: "#9898ab",
  textDim: "#55556a",
  accent: "#6c5ce7",
  accentSoft: "rgba(108,92,231,0.10)",
  accentBorder: "rgba(108,92,231,0.3)",
  green: "#22c55e",
  greenSoft: "rgba(34,197,94,0.08)",
  greenBorder: "rgba(34,197,94,0.25)",
  amber: "#f59e0b",
  amberSoft: "rgba(245,158,11,0.08)",
  amberBorder: "rgba(245,158,11,0.25)",
  blue: "#3b82f6",
  blueSoft: "rgba(59,130,246,0.08)",
};

// ── Categories (mapped from awesome-mcp-servers + glama) ───────────
const CATEGORIES = [
  { id: "dev-tools", label: "Development" },
  { id: "database", label: "Databases" },
  { id: "design", label: "Design Tools" },
  { id: "browser", label: "Browser & Testing" },
  { id: "search", label: "Search & Research" },
  { id: "communication", label: "Communication" },
  { id: "productivity", label: "Productivity & PM" },
  { id: "cloud", label: "Cloud & DevOps" },
  { id: "ai-ml", label: "AI & ML" },
  { id: "cms", label: "CMS & Content" },
  { id: "monitoring", label: "Monitoring & Logs" },
  { id: "payments", label: "Payments & Finance" },
  { id: "automation", label: "Automation" },
  { id: "security", label: "Security" },
  { id: "data-science", label: "Data Science" },
  { id: "other", label: "Other" },
];

const CAT_MAP = Object.fromEntries(CATEGORIES.map(c => [c.id, c.label]));

// ── Servers (multi-category) ───────────────────────────────────────
const SERVERS = [
  { id: 1, name: "GitHub", desc: "Repositories, issues, PRs, Actions, code search, and security findings", cats: ["dev-tools", "security"], installs: 48200, official: true, color: "#2ea043", installed: true, bookmarked: false, added: "2024-11" },
  { id: 2, name: "Figma", desc: "Read designs, components, variables, and layout data for design-to-code", cats: ["design", "dev-tools"], installs: 41500, official: true, color: "#a259ff", installed: true, bookmarked: true, added: "2025-01" },
  { id: 3, name: "Sequential Thinking", desc: "Structured problem-solving through dynamic, reflective thought chains", cats: ["ai-ml"], installs: 55100, official: true, color: "#f59e0b", installed: true, bookmarked: false, added: "2024-11" },
  { id: 4, name: "Brave Search", desc: "Privacy-focused web and local search via 30B+ page index", cats: ["search"], installs: 38900, official: true, color: "#fb542b", installed: true, bookmarked: false, added: "2024-11" },
  { id: 5, name: "Memory", desc: "Knowledge graph-based persistent memory across sessions", cats: ["ai-ml", "productivity"], installs: 44800, official: true, color: "#ec4899", installed: true, bookmarked: false, added: "2024-11" },
  { id: 6, name: "PostgreSQL", desc: "Query and inspect PostgreSQL databases with schema exploration", cats: ["database"], installs: 32100, official: true, color: "#336791", installed: false, bookmarked: false, added: "2024-11" },
  { id: 7, name: "Supabase", desc: "Tables, queries, edge functions, storage, and migrations", cats: ["database", "cloud"], installs: 29400, official: true, color: "#3ecf8e", installed: false, bookmarked: true, added: "2025-02" },
  { id: 8, name: "Playwright", desc: "Browser automation — tests, screenshots, scraping, multi-browser", cats: ["browser", "dev-tools"], installs: 31200, official: true, color: "#2ead33", installed: false, bookmarked: false, added: "2025-03" },
  { id: 9, name: "Slack", desc: "Search channels, send messages, read threads, manage canvases", cats: ["communication"], installs: 36700, official: true, color: "#4a154b", installed: true, bookmarked: false, added: "2025-01" },
  { id: 10, name: "Notion", desc: "Search workspace, manage pages and databases, add comments", cats: ["productivity", "cms"], installs: 35200, official: true, color: "#191919", installed: false, bookmarked: true, added: "2025-04" },
  { id: 11, name: "Context7", desc: "Up-to-date documentation and code examples for 9,000+ libraries", cats: ["dev-tools", "search"], installs: 42300, official: true, color: "#6366f1", installed: true, bookmarked: false, added: "2025-02" },
  { id: 12, name: "Filesystem", desc: "Secure file read/write with configurable access controls", cats: ["dev-tools", "productivity"], installs: 39000, official: true, color: "#10b981", installed: true, bookmarked: false, added: "2024-11" },
  { id: 13, name: "Stripe", desc: "Customers, products, payments, invoices, and subscriptions", cats: ["payments"], installs: 21500, official: true, color: "#635bff", installed: false, bookmarked: false, added: "2025-04" },
  { id: 14, name: "Sentry", desc: "Error tracking and AI-powered root cause analysis", cats: ["monitoring", "dev-tools"], installs: 24800, official: true, color: "#362d59", installed: false, bookmarked: false, added: "2025-05" },
  { id: 15, name: "Linear", desc: "Issues, projects, and team workflows via remote MCP", cats: ["productivity", "dev-tools"], installs: 27600, official: true, color: "#5e6ad2", installed: false, bookmarked: true, added: "2025-03" },
  { id: 16, name: "HubSpot", desc: "Contacts, companies, deals, tickets via natural language", cats: ["communication", "automation"], installs: 19200, official: true, color: "#ff7a59", installed: false, bookmarked: false, added: "2025-06" },
  { id: 17, name: "Terraform", desc: "Registry APIs, workspace management, HCP integration", cats: ["cloud"], installs: 22100, official: true, color: "#7b42bc", installed: false, bookmarked: false, added: "2025-03" },
  { id: 18, name: "AWS MCP", desc: "Lambda, S3, EC2, CDK, Bedrock, and cost analysis servers", cats: ["cloud"], installs: 34600, official: true, color: "#ff9900", installed: false, bookmarked: false, added: "2025-01" },
  { id: 19, name: "Atlassian", desc: "Jira issues and workflows, Confluence pages and spaces", cats: ["productivity", "dev-tools"], installs: 28900, official: true, color: "#0052cc", installed: false, bookmarked: false, added: "2025-07" },
  { id: 20, name: "shadcn/ui", desc: "Browse and add components to your project from the registry", cats: ["design", "dev-tools"], installs: 26300, official: false, color: "#eeeeee", installed: false, bookmarked: false, added: "2025-02" },
  { id: 21, name: "WordPress MCP", desc: "Content management — posts, pages, media, SEO, taxonomies", cats: ["cms", "design"], installs: 18400, official: true, color: "#21759b", installed: false, bookmarked: false, added: "2025-05" },
  { id: 22, name: "Zapier", desc: "Connect AI agents to 8,000+ apps with zero custom code", cats: ["automation"], installs: 31800, official: true, color: "#ff4a00", installed: false, bookmarked: false, added: "2025-04" },
  { id: 23, name: "MongoDB", desc: "MongoDB Community Server and Atlas integration", cats: ["database"], installs: 23700, official: true, color: "#47a248", installed: false, bookmarked: false, added: "2025-02" },
  { id: 24, name: "Puppeteer", desc: "Headless Chrome automation, scraping, and screenshots", cats: ["browser", "dev-tools"], installs: 25100, official: true, color: "#00d8a2", installed: false, bookmarked: false, added: "2024-11" },
  { id: 25, name: "Grafana", desc: "Search dashboards, query Prometheus and Loki datasources", cats: ["monitoring", "data-science"], installs: 17800, official: true, color: "#f46800", installed: false, bookmarked: false, added: "2025-06" },
  { id: 26, name: "Salesforce", desc: "SOQL, metadata, record CRUD — hosted MCP beta", cats: ["communication", "automation"], installs: 16900, official: true, color: "#00a1e0", installed: false, bookmarked: false, added: "2025-10" },
  { id: 27, name: "Exa Search", desc: "AI-native semantic search with embeddings-based discovery", cats: ["search", "ai-ml"], installs: 20400, official: true, color: "#5046e5", installed: false, bookmarked: false, added: "2025-01" },
  { id: 28, name: "ElevenLabs", desc: "Text-to-speech, voice cloning, and audio generation", cats: ["ai-ml", "cms"], installs: 15200, official: true, color: "#1a1a2e", installed: false, bookmarked: false, added: "2025-08" },
  { id: 29, name: "Snowflake", desc: "Cortex AI, SQL orchestration, and object management", cats: ["database", "data-science"], installs: 16400, official: true, color: "#29b5e8", installed: false, bookmarked: false, added: "2025-05" },
  { id: 30, name: "Cloudflare", desc: "Workers, KV, R2, D1 management and deployment", cats: ["cloud", "dev-tools"], installs: 21900, official: true, color: "#f38020", installed: false, bookmarked: false, added: "2025-03" },
  { id: 31, name: "SonarQube", desc: "Code quality integration and security scanning", cats: ["security", "dev-tools"], installs: 14200, official: true, color: "#4e9bcd", installed: false, bookmarked: false, added: "2025-07" },
  { id: 32, name: "Obsidian", desc: "Read, search, and modify Obsidian vault notes", cats: ["productivity", "search"], installs: 19800, official: false, color: "#7c3aed", installed: false, bookmarked: false, added: "2025-03" },
  { id: 33, name: "Google Drive", desc: "Read and search files stored in Google Drive", cats: ["productivity"], installs: 37200, official: true, color: "#4285f4", installed: true, bookmarked: false, added: "2024-12" },
  { id: 34, name: "ArXiv", desc: "Search papers with filtering, download and analyze as markdown", cats: ["search", "data-science"], installs: 11200, official: false, color: "#b31b1b", installed: false, bookmarked: false, added: "2025-04" },
  { id: 35, name: "Framelink Figma", desc: "Community Figma server optimized for Cursor with cleaner data", cats: ["design", "dev-tools"], installs: 22800, official: false, color: "#a259ff", installed: false, bookmarked: false, added: "2025-01" },
  { id: 36, name: "Composio", desc: "500+ managed app integrations with built-in auth", cats: ["automation", "communication"], installs: 18700, official: true, color: "#6366f1", installed: false, bookmarked: false, added: "2025-06" },
  { id: 37, name: "Semgrep", desc: "Static analysis, secure code scanning, and SAST", cats: ["security", "dev-tools"], installs: 12800, official: true, color: "#2bbb5a", installed: false, bookmarked: false, added: "2025-08" },
  { id: 38, name: "BigQuery", desc: "Google Cloud BigQuery — natural language querying and AI", cats: ["database", "data-science", "cloud"], installs: 15800, official: true, color: "#4285f4", installed: false, bookmarked: false, added: "2025-07" },
  { id: 39, name: "21st.dev Magic", desc: "Generate crafted UI components from top design engineers", cats: ["design", "dev-tools"], installs: 13400, official: true, color: "#ff6b6b", installed: false, bookmarked: false, added: "2025-09" },
  { id: 40, name: "Postman", desc: "Connect AI agents to your API collections and workspaces", cats: ["dev-tools"], installs: 16100, official: true, color: "#ff6c37", installed: false, bookmarked: false, added: "2025-06" },
];

// ── Helpers ─────────────────────────────────────────────────────────
function fmtInstalls(n) {
  if (n >= 1000) return (n / 1000).toFixed(n >= 10000 ? 0 : 1) + "k";
  return n.toString();
}

// ── Dropdown Component ─────────────────────────────────────────────
function MultiSelectDropdown({ label, options, selected, onChange, allLabel = "All" }) {
  const [open, setOpen] = useState(false);
  const [q, setQ] = useState("");
  const ref = useRef(null);

  useEffect(() => {
    const handler = (e) => { if (ref.current && !ref.current.contains(e.target)) setOpen(false); };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const filtered = options.filter(o => o.label.toLowerCase().includes(q.toLowerCase()));
  const selectedLabels = selected.length === 0 ? allLabel : selected.length <= 2
    ? selected.map(id => options.find(o => o.id === id)?.label).join(", ")
    : `${selected.length} selected`;

  return (
    <div ref={ref} style={{ position: "relative" }}>
      <button
        onClick={() => setOpen(!open)}
        style={{
          display: "flex", alignItems: "center", gap: 8,
          padding: "8px 14px", background: selected.length > 0 ? C.accentSoft : C.surface,
          border: `1px solid ${open ? C.borderFocus : selected.length > 0 ? C.accentBorder : C.border}`,
          borderRadius: 9, color: C.text, fontSize: 13, fontFamily: font,
          cursor: "pointer", whiteSpace: "nowrap", transition: "all 0.15s",
          fontWeight: selected.length > 0 ? 500 : 400,
        }}
      >
        <span style={{ color: C.textDim, fontSize: 12 }}>{label}:</span>
        <span style={{ maxWidth: 180, overflow: "hidden", textOverflow: "ellipsis" }}>{selectedLabels}</span>
        <svg width="10" height="6" style={{ opacity: 0.5, flexShrink: 0, transform: open ? "rotate(180deg)" : "none", transition: "transform 0.15s" }}>
          <path d="M0 0l5 6 5-6z" fill={C.textSec} />
        </svg>
      </button>

      {open && (
        <div style={{
          position: "absolute", top: "calc(100% + 6px)", left: 0, zIndex: 100,
          width: 260, background: C.surface, border: `1px solid ${C.border}`,
          borderRadius: 12, boxShadow: "0 16px 48px rgba(0,0,0,0.5)", overflow: "hidden",
        }}>
          <div style={{ padding: "10px 10px 6px" }}>
            <input
              value={q} onChange={e => setQ(e.target.value)}
              placeholder="Filter..." autoFocus
              style={{
                width: "100%", padding: "7px 10px", background: C.bg,
                border: `1px solid ${C.border}`, borderRadius: 7, color: C.text,
                fontSize: 12, fontFamily: font, outline: "none", boxSizing: "border-box",
              }}
            />
          </div>
          <div style={{ maxHeight: 240, overflowY: "auto", padding: "2px 6px 8px" }}>
            {selected.length > 0 && (
              <button
                onClick={() => onChange([])}
                style={{
                  display: "block", width: "100%", padding: "6px 8px", background: "none",
                  border: "none", color: C.accent, fontSize: 11, fontFamily: font,
                  cursor: "pointer", textAlign: "left", fontWeight: 500,
                }}
              >
                Clear all
              </button>
            )}
            {filtered.map(o => {
              const checked = selected.includes(o.id);
              return (
                <button
                  key={o.id}
                  onClick={() => onChange(checked ? selected.filter(s => s !== o.id) : [...selected, o.id])}
                  style={{
                    display: "flex", alignItems: "center", gap: 10, width: "100%",
                    padding: "7px 8px", background: checked ? C.accentSoft : "none",
                    border: "none", borderRadius: 6, color: C.text, fontSize: 13,
                    fontFamily: font, cursor: "pointer", textAlign: "left",
                    transition: "background 0.1s",
                  }}
                  onMouseEnter={e => !checked && (e.target.style.background = C.surfaceHover)}
                  onMouseLeave={e => !checked && (e.target.style.background = "none")}
                >
                  <span style={{
                    width: 16, height: 16, borderRadius: 4, flexShrink: 0,
                    border: checked ? `1.5px solid ${C.accent}` : `1.5px solid ${C.border}`,
                    background: checked ? C.accent : "none",
                    display: "flex", alignItems: "center", justifyContent: "center",
                    transition: "all 0.15s",
                  }}>
                    {checked && (
                      <svg width="10" height="8" viewBox="0 0 10 8" fill="none" stroke="#fff" strokeWidth="2" strokeLinecap="round"><path d="M1 4l3 3 5-6" /></svg>
                    )}
                  </span>
                  {o.label}
                </button>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}

// ── Single Select Dropdown ─────────────────────────────────────────
function SingleSelect({ options, value, onChange }) {
  return (
    <select
      value={value} onChange={e => onChange(e.target.value)}
      style={{
        padding: "8px 30px 8px 12px", background: C.surface,
        border: `1px solid ${C.border}`, borderRadius: 9, color: C.text,
        fontSize: 13, fontFamily: font, cursor: "pointer", outline: "none",
        appearance: "none", WebkitAppearance: "none",
        backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M0 0l5 6 5-6z' fill='%2355556a'/%3E%3C/svg%3E")`,
        backgroundRepeat: "no-repeat", backgroundPosition: "right 10px center",
      }}
    >
      {options.map(o => <option key={o.value} value={o.value}>{o.label}</option>)}
    </select>
  );
}

// ── Server Card ────────────────────────────────────────────────────
function ServerCard({ server, isBookmarked, onToggleBookmark, onToggleInstall }) {
  const [hovered, setHovered] = useState(false);
  const clr = server.color === "#eeeeee" || server.color === "#1a1a2e" || server.color === "#191919" ? C.textSec : server.color;

  return (
    <div
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        background: C.surface,
        border: `1px solid ${hovered ? C.borderHover : C.border}`,
        borderRadius: 14, padding: 20,
        display: "flex", flexDirection: "column", gap: 12,
        transition: "all 0.2s ease", cursor: "default",
        transform: hovered ? "translateY(-1px)" : "none",
        boxShadow: hovered ? "0 8px 24px rgba(0,0,0,0.25)" : "none",
      }}
    >
      {/* Header: icon + name + badges + bookmark */}
      <div style={{ display: "flex", alignItems: "flex-start", gap: 12 }}>
        <div style={{
          width: 40, height: 40, borderRadius: 10, flexShrink: 0,
          background: `${clr}15`, border: `1px solid ${clr}30`,
          display: "flex", alignItems: "center", justifyContent: "center",
          fontSize: 17, fontWeight: 700, color: clr, fontFamily: mono,
        }}>
          {server.name[0]}
        </div>

        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
            <span style={{ fontSize: 15, fontWeight: 600, color: C.text }}>{server.name}</span>
            {server.official && (
              <span style={{
                display: "inline-flex", alignItems: "center", gap: 3,
                fontSize: 10, fontWeight: 600, color: C.accent,
                background: C.accentSoft, padding: "2px 7px", borderRadius: 4,
                border: `1px solid ${C.accentBorder}`,
              }}>
                <svg width="8" height="8" viewBox="0 0 10 10" fill={C.accent}>
                  <path d="M5 0l1.3 3.2L10 3.8 7.2 6.2 8 10 5 8 2 10l.8-3.8L0 3.8l3.7-.6z" />
                </svg>
                Official
              </span>
            )}
          </div>
        </div>

        {/* Bookmark */}
        <button
          onClick={() => onToggleBookmark(server.id)}
          style={{
            background: "none", border: "none", cursor: "pointer", padding: 4,
            opacity: hovered || isBookmarked ? 1 : 0.25, transition: "opacity 0.2s",
            flexShrink: 0,
          }}
          title={isBookmarked ? "Remove bookmark" : "Bookmark"}
        >
          <svg width="16" height="16" viewBox="0 0 16 16"
            fill={isBookmarked ? C.amber : "none"}
            stroke={isBookmarked ? C.amber : C.textSec}
            strokeWidth="1.5">
            <path d="M3 2.5A1.5 1.5 0 014.5 1h7A1.5 1.5 0 0113 2.5v12l-5-3-5 3V2.5z" />
          </svg>
        </button>
      </div>

      {/* Description */}
      <p style={{ fontSize: 13, color: C.textSec, lineHeight: 1.55, margin: 0 }}>
        {server.desc}
      </p>

      {/* Category tags (multi) */}
      <div style={{ display: "flex", gap: 5, flexWrap: "wrap" }}>
        {server.cats.map(catId => (
          <span key={catId} style={{
            fontSize: 11, color: C.textDim, fontFamily: mono, letterSpacing: "0.01em",
            background: `${C.textDim}12`, padding: "3px 8px", borderRadius: 5,
            border: `1px solid ${C.textDim}20`,
          }}>
            {CAT_MAP[catId] || catId}
          </span>
        ))}
      </div>

      {/* Footer: installs + action */}
      <div style={{
        display: "flex", alignItems: "center", justifyContent: "space-between",
        marginTop: "auto", paddingTop: 4,
      }}>
        <span style={{ fontSize: 12, color: C.textDim, display: "flex", alignItems: "center", gap: 5 }}>
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke={C.textDim} strokeWidth="1.3">
            <path d="M6 1v7M3 5.5L6 8.5 9 5.5M2 11h8" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
          {fmtInstalls(server.installs)} installs
        </span>

        <button
          onClick={() => onToggleInstall(server.id)}
          style={{
            padding: server.installed ? "5px 12px" : "5px 14px",
            borderRadius: 7, fontSize: 12, fontWeight: 600, fontFamily: font,
            cursor: "pointer", transition: "all 0.15s",
            border: server.installed ? `1px solid ${C.greenBorder}` : `1px solid ${C.border}`,
            background: server.installed ? C.greenSoft : "transparent",
            color: server.installed ? C.green : C.text,
            display: "flex", alignItems: "center", gap: 5,
          }}
          onMouseEnter={e => {
            if (!server.installed) {
              e.currentTarget.style.background = C.accentSoft;
              e.currentTarget.style.borderColor = C.accentBorder;
              e.currentTarget.style.color = C.accent;
            }
          }}
          onMouseLeave={e => {
            if (!server.installed) {
              e.currentTarget.style.background = "transparent";
              e.currentTarget.style.borderColor = C.border;
              e.currentTarget.style.color = C.text;
            }
          }}
        >
          {server.installed ? (
            <>
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke={C.green} strokeWidth="2" strokeLinecap="round"><path d="M2 6.5l3 3 5-5.5" /></svg>
              Active
            </>
          ) : (
            <>+ Add</>
          )}
        </button>
      </div>
    </div>
  );
}

// ── Pagination ─────────────────────────────────────────────────────
function Pagination({ total, page, perPage, onChange }) {
  const pages = Math.ceil(total / perPage);
  if (pages <= 1) return null;
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
      <button
        disabled={page === 1}
        onClick={() => onChange(page - 1)}
        style={{
          padding: "5px 8px", background: "none", border: `1px solid ${C.border}`,
          borderRadius: 6, color: page === 1 ? C.textDim : C.textSec, fontSize: 12,
          cursor: page === 1 ? "default" : "pointer", fontFamily: font, opacity: page === 1 ? 0.4 : 1,
        }}
      >←</button>
      {Array.from({ length: pages }, (_, i) => i + 1).slice(
        Math.max(0, page - 3),
        Math.min(pages, page + 2)
      ).map(p => (
        <button
          key={p} onClick={() => onChange(p)}
          style={{
            padding: "5px 10px", background: p === page ? C.accentSoft : "none",
            border: `1px solid ${p === page ? C.accentBorder : C.border}`,
            borderRadius: 6, color: p === page ? C.text : C.textSec, fontSize: 12,
            cursor: "pointer", fontFamily: mono, fontWeight: p === page ? 600 : 400,
          }}
        >{p}</button>
      ))}
      <button
        disabled={page === pages}
        onClick={() => onChange(page + 1)}
        style={{
          padding: "5px 8px", background: "none", border: `1px solid ${C.border}`,
          borderRadius: 6, color: page === pages ? C.textDim : C.textSec, fontSize: 12,
          cursor: page === pages ? "default" : "pointer", fontFamily: font, opacity: page === pages ? 0.4 : 1,
        }}
      >→</button>
    </div>
  );
}

// ── Main ───────────────────────────────────────────────────────────
const PER_PAGE = 12;

export default function MCPCatalog() {
  const [tab, setTab] = useState("discover");
  const [search, setSearch] = useState("");
  const [selCats, setSelCats] = useState([]);
  const [selType, setSelType] = useState("all"); // all | official | community
  const [sort, setSort] = useState("popular");
  const [page, setPage] = useState(1);
  const [servers, setServers] = useState(SERVERS);

  const toggleBookmark = (id) => {
    setServers(prev => prev.map(s => s.id === id ? { ...s, bookmarked: !s.bookmarked } : s));
  };
  const toggleInstall = (id) => {
    setServers(prev => prev.map(s => s.id === id ? { ...s, installed: !s.installed } : s));
  };

  // Reset page on filter change
  useEffect(() => { setPage(1); }, [tab, search, selCats, selType, sort]);

  // Filter
  let filtered = servers.filter(s => {
    if (tab === "bookmarks" && !s.bookmarked) return false;
    if (tab === "installed" && !s.installed) return false;
    if (selCats.length > 0 && !s.cats.some(c => selCats.includes(c))) return false;
    if (selType === "official" && !s.official) return false;
    if (selType === "community" && s.official) return false;
    if (search) {
      const q = search.toLowerCase();
      return s.name.toLowerCase().includes(q) || s.desc.toLowerCase().includes(q)
        || s.cats.some(c => (CAT_MAP[c] || "").toLowerCase().includes(q));
    }
    return true;
  });

  // Sort
  if (sort === "popular") filtered.sort((a, b) => b.installs - a.installs);
  if (sort === "name") filtered.sort((a, b) => a.name.localeCompare(b.name));
  if (sort === "official") filtered.sort((a, b) => (b.official ? 1 : 0) - (a.official ? 1 : 0) || b.installs - a.installs);
  if (sort === "recent") filtered.sort((a, b) => b.added.localeCompare(a.added));

  const total = filtered.length;
  const paged = filtered.slice((page - 1) * PER_PAGE, page * PER_PAGE);

  const installedCount = servers.filter(s => s.installed).length;
  const bookmarkCount = servers.filter(s => s.bookmarked).length;

  return (
    <div style={{ fontFamily: font, background: C.bg, color: C.text, minHeight: "100vh" }}>
      <link href="https://fonts.googleapis.com/css2?family=DM+Sans:wght@400;500;600;700&display=swap" rel="stylesheet" />

      <div style={{ maxWidth: 1120, margin: "0 auto", padding: "32px 28px" }}>

        {/* ── Title ─────────────────────────────────────── */}
        <div style={{ marginBottom: 28 }}>
          <h1 style={{
            fontSize: 24, fontWeight: 700, margin: 0, letterSpacing: "-0.03em",
            display: "flex", alignItems: "center", gap: 10,
          }}>
            <span style={{
              width: 30, height: 30, borderRadius: 8,
              background: `linear-gradient(135deg, ${C.accent}, #a78bfa)`,
              display: "inline-flex", alignItems: "center", justifyContent: "center",
              fontSize: 15,
            }}>⚡</span>
            MCP Catalog
          </h1>
        </div>

        {/* ── Tab Switcher ──────────────────────────────── */}
        <div style={{
          display: "flex", gap: 0, marginBottom: 24,
          borderBottom: `1px solid ${C.border}`,
        }}>
          {[
            { id: "discover", label: "Discover", count: null },
            { id: "bookmarks", label: "Bookmarked", count: bookmarkCount },
            { id: "installed", label: "Installed", count: installedCount },
          ].map(t => (
            <button
              key={t.id} onClick={() => setTab(t.id)}
              style={{
                padding: "10px 20px", background: "none", border: "none",
                borderBottom: tab === t.id ? `2px solid ${C.accent}` : "2px solid transparent",
                color: tab === t.id ? C.text : C.textSec,
                fontSize: 14, fontWeight: tab === t.id ? 600 : 400,
                cursor: "pointer", fontFamily: font, transition: "all 0.15s",
                display: "flex", alignItems: "center", gap: 7,
              }}
            >
              {t.label}
              {t.count !== null && (
                <span style={{
                  fontSize: 11, fontFamily: mono, fontWeight: 500,
                  color: tab === t.id ? C.accent : C.textDim,
                  background: tab === t.id ? C.accentSoft : `${C.textDim}18`,
                  padding: "1px 7px", borderRadius: 10, minWidth: 20, textAlign: "center",
                }}>{t.count}</span>
              )}
            </button>
          ))}
        </div>

        {/* ── Filter Bar (only in Discover) ─────────────── */}
        {tab === "discover" && (
          <div style={{ display: "flex", flexWrap: "wrap", gap: 10, marginBottom: 20, alignItems: "center" }}>
            {/* Search */}
            <div style={{ position: "relative", flex: "1 1 240px", minWidth: 200 }}>
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke={C.textDim} strokeWidth="1.5"
                style={{ position: "absolute", left: 12, top: "50%", transform: "translateY(-50%)" }}>
                <circle cx="6" cy="6" r="4.5" /><path d="M9.5 9.5L13 13" strokeLinecap="round" />
              </svg>
              <input
                value={search} onChange={e => setSearch(e.target.value)}
                placeholder="Search servers..."
                style={{
                  width: "100%", padding: "8px 12px 8px 34px", background: C.surface,
                  border: `1px solid ${C.border}`, borderRadius: 9, color: C.text,
                  fontSize: 13, fontFamily: font, outline: "none", boxSizing: "border-box",
                  transition: "border-color 0.2s",
                }}
                onFocus={e => e.target.style.borderColor = C.borderFocus}
                onBlur={e => e.target.style.borderColor = C.border}
              />
            </div>

            {/* Category multi-select */}
            <MultiSelectDropdown
              label="Category"
              options={CATEGORIES}
              selected={selCats}
              onChange={setSelCats}
              allLabel="All categories"
            />

            {/* Type filter */}
            <SingleSelect
              value={selType}
              onChange={setSelType}
              options={[
                { value: "all", label: "All types" },
                { value: "official", label: "Official only" },
                { value: "community", label: "Community" },
              ]}
            />
          </div>
        )}

        {/* ── Results bar: count + sort + pagination ────── */}
        <div style={{
          display: "flex", alignItems: "center", justifyContent: "space-between",
          marginBottom: 16, flexWrap: "wrap", gap: 10,
        }}>
          <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
            <span style={{ fontSize: 13, color: C.textDim, fontFamily: mono }}>
              {total} server{total !== 1 ? "s" : ""}
            </span>
            {tab === "discover" && (
              <SingleSelect
                value={sort}
                onChange={setSort}
                options={[
                  { value: "popular", label: "Most popular" },
                  { value: "official", label: "Official first" },
                  { value: "name", label: "A → Z" },
                  { value: "recent", label: "Recently added" },
                ]}
              />
            )}
          </div>
          <Pagination total={total} page={page} perPage={PER_PAGE} onChange={setPage} />
        </div>

        {/* ── Card Grid ─────────────────────────────────── */}
        <div style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fill, minmax(300px, 1fr))",
          gap: 14,
        }}>
          {paged.map(s => (
            <ServerCard
              key={s.id}
              server={s}
              isBookmarked={s.bookmarked}
              onToggleBookmark={toggleBookmark}
              onToggleInstall={toggleInstall}
            />
          ))}
        </div>

        {paged.length === 0 && (
          <div style={{
            textAlign: "center", padding: "60px 20px", color: C.textDim,
          }}>
            <div style={{ fontSize: 36, marginBottom: 12, opacity: 0.3 }}>∅</div>
            <div style={{ fontSize: 15, fontWeight: 500, color: C.textSec }}>
              {tab === "bookmarks" ? "No bookmarked servers yet" : tab === "installed" ? "No installed servers" : "No servers match your filters"}
            </div>
            <div style={{ fontSize: 13, marginTop: 6 }}>
              {tab === "discover" ? "Try adjusting your search or category filters" : "Browse Discover to find servers"}
            </div>
          </div>
        )}

        {/* Bottom pagination */}
        {total > PER_PAGE && (
          <div style={{ display: "flex", justifyContent: "center", marginTop: 24 }}>
            <Pagination total={total} page={page} perPage={PER_PAGE} onChange={setPage} />
          </div>
        )}

        <div style={{ height: 40 }} />
      </div>
    </div>
  );
}
