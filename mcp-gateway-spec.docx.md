**MCP Gateway**

Design Spec \+ Search Architecture

Lasha Rela  •  March 2026

# **1\. The Problem**

When you connect multiple MCP servers to Claude or Cursor, every tool schema from every server gets injected into the context window before the first message. With a typical setup of 5–8 servers, this consumes 40,000–60,000 tokens upfront — before the agent has done anything useful.

| Today (broken) github: 12 tools \= \~14,400 tokens postgres: 8 tools \= \~9,600 tokens brave: 5 tools \= \~6,000 tokens linear: 10 tools \= \~12,000 tokens filesystem: 8 tools \= \~9,600 tokens Total: \~51,600 tokens burned before first message | With gateway (fixed) gateway: 3 tools \= \~800 tokens list\_categories() search\_tools(query) execute\_tool(name, args) Agent searches on demand. Only loads schemas it needs. |
| :---- | :---- |

# **2\. Gateway Architecture**

The gateway sits between the AI client and all MCP servers. It acts as both an MCP server (facing the agent) and an MCP client (facing the real servers). The agent only ever sees 3 tools.

Claude / Cursor / any agent

    |  single MCP connection (stdio or HTTP+SSE)

    v

\+-- GATEWAY (localhost:4242) \------------------+

|                                              |

|  MCP Server Layer  (exposes 3 tools)         |

|    |                                         |

|  Tool Registry  \<----  SQLite                |

|    |                                         |

|  Embedding Index  \<--  nomic-embed-text      |

|    |                                         |

|  Proxy Router                                |

|    |         |         |                     |

\+-------------------------------------  \-------+

     |         |         |

   github   postgres   brave

  (stdio)   (stdio)   (http)

## **The 3 Tools**

### **1\. list\_categories()**

Returns a compact overview of all registered servers and categories. \~200 tokens. The agent uses this to orient itself before searching.

list\_categories() \-\> {

  "categories": \[

    { "id": "dev",  "count": 12, "servers": \["github","linear"\] },

    { "id": "data", "count": 8,  "servers": \["postgres","sqlite"\] },

    { "id": "web",  "count": 5,  "servers": \["brave","fetch"\] }

  \],

  "total\_tools": 47

}

### **2\. search\_tools(query)**

Takes a natural language query, runs it through the embedding index, and returns the top 5 matching tools with their full schemas. Only these schemas enter the context window.

search\_tools("create github issue") \-\> {

  "tools": \[

    {

      "name": "github\_\_create\_issue",

      "description": "Create a new issue in a repository",

      "inputSchema": { ... },  // full schema, only this one

      "score": 0.94

    }

  \]

}

### **3\. execute\_tool(name, args)**

Proxies the call to the correct upstream MCP server transparently. Tool naming convention: serverid\_\_toolname prevents collisions across servers.

execute\_tool("github\_\_create\_issue", {

  "owner": "lasharela", "repo": "hulbu",

  "title": "Add ANP support"

}) \-\> { "issue\_number": 42, "url": "..." }

# **3\. BM25: Keyword Search**

| TL;DR BM25 is a fast keyword ranking algorithm. It scores documents based on how often query words appear, adjusted for document length. No machine learning, no embeddings — pure math. Works out of the box with zero dependencies. |
| :---- |

## **How BM25 Works**

BM25 (Best Match 25\) is the industry standard for keyword search — it powers Elasticsearch, Lucene, and most search engines under the hood. It scores each document against a query using two factors:

| Term Frequency (TF) How often does the query word appear in this document? More occurrences \= higher score. But the boost is diminishing — going from 1 to 2 occurrences helps a lot, going from 10 to 11 barely matters. | Inverse Document Frequency (IDF) How rare is this word across ALL documents? Rare words ("pgvector") are more meaningful than common words ("the", "a", "tool"). Rare \= higher weight. |
| :---- | :---- |

### **The Formula**

For each query term t in document d:

score(t, d) \= IDF(t) \* TF(t, d) \* (k1 \+ 1\)

                          \_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_

                         TF(t,d) \+ k1 \* (1 \- b \+ b \* |d| / avgdl)

k1 \= 1.2  (term frequency saturation — how quickly TF boost flattens)

b  \= 0.75 (length normalization — penalizes long documents)

|d| \= length of document d in tokens

avgdl \= average document length across corpus

### **Concrete Example**

**Query:** "create issue")   Tool descriptions:

| Tool A "Create a new issue in a GitHub repository. Issues track bugs and features." create: appears 1x  \-\> high issue:  appears 2x  \-\> high BM25 score: 0.87 | Tool B "Query a PostgreSQL database and return results as JSON. Supports SELECT statements." create: appears 0x  \-\> zero issue:  appears 0x  \-\> zero BM25 score: 0.00 |
| :---- | :---- |

## **BM25 Strengths and Weaknesses**

| Strengths Zero dependencies — pure Python/JS Instant startup, no model to load Deterministic and auditable Great for exact technical terms Works perfectly for tool names | Weaknesses No synonym understanding "fetch URL" won't find "make HTTP request" No concept of meaning — only words Order of words mostly ignored Fails on paraphrased queries |
| :---- | :---- |

# **4\. Semantic Search (Embeddings)**

| TL;DR Semantic search converts text into vectors (lists of numbers) that capture meaning. Similar meanings produce similar vectors. "Create an issue" and "open a ticket" land near each other in vector space — even though they share zero words. |
| :---- |

## **How Embeddings Work**

An embedding model (like nomic-embed-text) reads a piece of text and outputs a vector — a list of \~768 numbers. This vector is a coordinate in "meaning space". Texts with similar meaning have vectors that point in similar directions.

Text: "create a github issue"

Vector: \[0.12, \-0.84, 0.33, 0.71, ..., 0.22\]  // 768 numbers

         ^                                  ^

         first dimension                    last dimension

Text: "open a new ticket in github"

Vector: \[0.14, \-0.81, 0.35, 0.69, ..., 0.19\]  // very similar\!

Text: "query a postgres database"

Vector: \[0.82,  0.33, \-0.12, \-0.44, ..., 0.61\]  // very different

## **Cosine Similarity**

To compare two vectors, we use cosine similarity — it measures the angle between them. Two identical vectors have similarity 1.0. Two completely unrelated vectors have similarity near 0\.

similarity \= (A · B) / (|A| \* |B|)

"create issue" vs "open ticket"    \= 0.91  // nearly same direction

"create issue" vs "query database" \= 0.23  // very different

At search time: embed the query, compute similarity against all tool embeddings, return top-K results. For 500 tools this takes \~5ms on CPU with sqlite-vec.

## **The Model: nomic-embed-text**

This gateway uses nomic-embed-text via Ollama — a local embedding model that runs on your machine with no API key and no cloud dependency.

| Model details 768-dimensional output vectors Runs fully local via Ollama \~275MB download, once \~5ms per search on CPU MIT licensed, free forever | Fallback: BM25 Used if Ollama is not installed No dependencies at all Still works well for exact terms Upgrade path: install Ollama anytime Same search\_tools() API surface |
| :---- | :---- |

# **5\. BM25 vs Semantic vs Hybrid**

The gateway defaults to semantic search with BM25 fallback. Here is how they compare for tool discovery:

|  | BM25 | Semantic | Hybrid |
| :---- | :---- | :---- | :---- |
| Speed | Very fast | Moderate | Moderate |
| Exact match | Excellent | Poor | Good |
| Synonyms | Fails | Excellent | Excellent |
| Infrastructure | Zero | Needs model | Needs model |
| Token count | \~300 tokens | \~300 tokens | \~300 tokens |

| Why not hybrid (both at once)? Hybrid (RRF: Reciprocal Rank Fusion) combines BM25 and semantic scores. It handles both exact matches and synonyms. Recommended for v0.3 when the tool catalog grows beyond 200 entries. For MVP, semantic alone is sufficient. |
| :---- |

# **6\. Technical Stack**

Single binary, no Docker, ships as npx or brew. The whole stack is chosen to minimize friction for developers.

| Runtime | TypeScript \+ Bun | Bun compiles to a single binary. No node\_modules for end users. Fast startup, native SQLite. |
| :---- | :---- | :---- |

| MCP Server | @modelcontextprotocol/sdk | Official SDK. Handles stdio \+ HTTP+SSE transports. Register your 3 tools and done. |
| :---- | :---- | :---- |

| Database | SQLite \+ sqlite-vec | sqlite-vec adds vector search as a native SQLite extension. Zero infra. Single .db file. |
| :---- | :---- | :---- |

| Embeddings | nomic-embed-text (Ollama) | 100% local. 768 dims. \~5ms search. No API key. Fallback: BM25 (pure JS, zero deps). |
| :---- | :---- | :---- |

| UI | Vite \+ React | Bundled into the binary at build time. User opens localhost:4243. No separate install. |
| :---- | :---- | :---- |

| Config | \~/.config/gatekeeper/config.json | XDG standard path. Import from Claude Desktop config. Export as share link. |
| :---- | :---- | :---- |

# **7\. Existing Solutions & Gap Analysis**

The MCP gateway space is crowded at the enterprise level but completely empty at the developer-local level. Here is what exists and what is missing:

| Tool | Type | Search | Gap |
| :---- | :---- | :---- | :---- |
| **mcpproxy-go** | Local proxy | BM25 only | No embeddings, no tray UI |
| **MetaMCP** | Docker gateway | None | Heavy, needs ops knowledge |
| **mcp-gateway-registry** | Enterprise | FAISS \+ sentence-transformers | Cloud/server deploy, not local |
| **ContextForge (IBM)** | Enterprise | Full-text | Docker only, complex setup |
| **Lunar.dev MCPX** | SaaS | None public | Cloud, not developer-local |
| **Obot** | Enterprise platform | Registry search | Full platform, not a simple tool |

| The gap Nobody has built: npx install \-\> tray icon \-\> opens localhost UI \-\> drag-drop MCP servers \-\> copy one config line into Claude Desktop. Every existing tool requires Docker knowledge, is a cloud SaaS, or has no UI. The developer-local, zero-config, tray-first experience does not exist yet. |
| :---- |

# **8\. Roadmap**

  **v0.1 — Core Gateway (\~2 days)**

* MCP server exposing exactly 3 tools

* SQLite tool registry, manual config.json

* Proxy router to child MCP servers (stdio only)

* BM25 keyword search (zero dependencies)

* Works with Claude Desktop and Cursor

  **v0.2 — Embeddings \+ UI (\~3 days)**

* nomic-embed-text via Ollama with BM25 fallback

* sqlite-vec integration for vector search

* Basic React UI: server list, toggle on/off

* Add server from npm package name

* Export claude\_desktop\_config.json snippet

  **v0.3 — Polish (\~2 days)**

* Category drag-and-drop editor

* Tool explorer with test-call functionality

* Built-in registry of top 100 MCP servers

* HTTP+SSE transport (not just stdio)

* Hot reload on config change

  **v1.0 — Ship (\~1 day)**

* Bundle as single npx binary

* Homebrew formula

* README \+ demo GIF

* Share config via URL

* Open source on GitHub

MCP Gateway Spec  •  Lasha Rela  •  March 2026  •  Draft v1