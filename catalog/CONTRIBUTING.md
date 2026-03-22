# Contributing to the plugmux Catalog

The plugmux catalog is the curated list of MCP servers available for one-click installation through the plugmux GUI. Contributing a server makes it discoverable to everyone who uses plugmux — no config file editing required.

This guide explains how to add your MCP server to the catalog via a GitHub pull request.

---

## How to Add a Server

### 1. Fork the repository

Fork [plugmux on GitHub](https://github.com/lasharela/plugmux) and clone your fork locally:

```bash
git clone https://github.com/<your-username>/plugmux.git
cd plugmux
```

### 2. Add an entry to `catalog/servers.json`

Open `catalog/servers.json` and add your server object to the `"servers"` array. Follow the field reference below and see the full example at the end of this section.

Keep the list alphabetically sorted by `id` within each category — this makes diffs easier to review.

### 3. Add an SVG icon to `catalog/icons/`

Place a single SVG file in `catalog/icons/`. The filename must match the `icon` field in your entry (e.g., `my-server.svg`). Read the icon guidelines below before designing your icon.

### 4. Open a pull request

Push your branch and open a PR against the `main` branch of the upstream repo. Use the title format described in the PR title section below.

---

## Required Fields

| Field | Type | Description |
|---|---|---|
| `id` | string | Unique identifier — lowercase, hyphen-separated (e.g. `"my-server"`) |
| `name` | string | Human-readable display name shown in the UI |
| `description` | string | One-line description of what the server does (max ~80 chars) |
| `icon` | string | Filename of the SVG icon in `catalog/icons/` (e.g. `"my-server.svg"`) |
| `category` | string | One of the allowed category values (see below) |
| `transport` | string | `"stdio"` or `"http"` |
| `command` | string | *(stdio only)* The executable to run (e.g. `"npx"`, `"uvx"`) |
| `url` | string | *(http only)* The full MCP endpoint URL |
| `args` | array | *(optional, stdio only)* Arguments passed to `command` |
| `connectivity` | string | `"local"` (no network required) or `"online"` (calls external APIs) |

### Allowed categories

| Value | Use for |
|---|---|
| `design` | Design tools, prototyping, UI/UX |
| `dev-tools` | Code, version control, SDKs, documentation |
| `database` | SQL, NoSQL, graph databases |
| `browser` | Web automation, search, scraping |
| `ai` | AI models, embeddings, memory, agents |
| `productivity` | Communication, calendar, notes, task management |
| `testing` | Test runners, QA, coverage |
| `infrastructure` | Cloud, CI/CD, monitoring, containers |
| `marketing` | Analytics, CRM, advertising |
| `content` | CMS, media, publishing |

If your server does not fit any of these categories, open an issue to propose a new one before submitting the PR.

---

## Full Example Entry

```json
{
  "id": "linear",
  "name": "Linear",
  "description": "Create and manage Linear issues, projects, and cycles",
  "icon": "linear.svg",
  "category": "productivity",
  "transport": "stdio",
  "command": "npx",
  "args": ["-y", "@linear/mcp-server"],
  "connectivity": "online"
}
```

**Field-by-field explanation:**

- `"id": "linear"` — Unique key used internally. Must not collide with any existing entry. Use the service's canonical lowercase name; add a suffix if there is a conflict (e.g. `"postgres-readonly"`).
- `"name": "Linear"` — Displayed in the plugmux server list. Title case, matching the service's official branding.
- `"description": "..."` — One sentence. Start with a verb ("Create", "Query", "Send"). Do not repeat the name.
- `"icon": "linear.svg"` — Must exactly match the filename you place in `catalog/icons/`.
- `"category": "productivity"` — Pick the single best-fit category from the allowed list.
- `"transport": "stdio"` — `"stdio"` for process-based servers, `"http"` for remote servers.
- `"command": "npx"` — The binary plugmux will invoke. Common values: `"npx"`, `"uvx"`, `"node"`, `"python"`.
- `"args": ["-y", "@linear/mcp-server"]` — Passed verbatim as process arguments. Omit the field entirely if no args are needed.
- `"connectivity": "online"` — `"online"` if the server makes outbound network requests; `"local"` if it works entirely offline.

**HTTP transport example** (no `command` or `args`):

```json
{
  "id": "my-remote-server",
  "name": "My Remote Server",
  "description": "Connects to the My Remote Service API",
  "icon": "my-remote-server.svg",
  "category": "infrastructure",
  "transport": "http",
  "url": "https://mcp.example.com/mcp",
  "connectivity": "online"
}
```

---

## Icon Guidelines

Icons appear at 24px in the plugmux UI. Keep them clean and recognisable at small sizes.

- **Format:** SVG only. No PNG, WEBP, or raster fallbacks.
- **ViewBox:** Must be `viewBox="0 0 24 24"`.
- **Color:** Use `currentColor` for all fill and stroke values. Do not hardcode hex colors — plugmux applies its own theming.
- **Style:** Monochrome. Single color, no gradients, no shadows, no `<defs>` filters.
- **Complexity:** Prefer simple geometric shapes. If the official logo is too detailed, use an abstracted version.
- **File size:** 2 KB maximum.
- **Naming:** Match the `id` field exactly (e.g. id `"my-server"` → file `my-server.svg`).

Minimal valid example:

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
  <rect x="3" y="3" width="18" height="18" rx="2"/>
  <path d="M8 12h8M12 8v8"/>
</svg>
```

---

## PR Title Format

```
catalog: add <server-name>
```

Examples:
- `catalog: add linear`
- `catalog: add postgres-readonly`
- `catalog: add my-remote-server`

Use the `id` value, not the display name. Keep it lowercase.

---

## What to Expect

1. **Automated checks** — A CI job validates that the JSON is well-formed, all required fields are present, the `id` is unique, and the icon file exists and matches.

2. **Maintainer review** — A maintainer will check that the server works as described, the icon meets the guidelines, and the category is appropriate. This typically takes a few days.

3. **Testing** — The maintainer will install the server locally via plugmux and verify it connects successfully.

4. **Merge** — Once approved, the PR is squash-merged and the new server appears in the catalog on the next release.

If your PR needs changes, a reviewer will leave comments explaining what to fix. Feel free to ask questions in the PR thread if anything is unclear.
