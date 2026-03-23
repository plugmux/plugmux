# The definitive guide to MCP servers by profession

**The MCP ecosystem has exploded to over 12,000 servers** as of early 2026, giving every profession — from designers to lawyers — AI-powered tool access through Claude, Cursor, Windsurf, Claude Code, and other AI platforms. This guide catalogs the most popular and useful MCP servers organized by profession, drawing from the official MCP Registry, Smithery.ai (5,000+ servers), mcp.so (18,800+ servers), PulseMCP (12,370+ servers), Glama.ai (76 categories), and GitHub's curated awesome-lists. Each entry notes whether the server is **official** (maintained by the service provider) or **community-built**.

---

## Cross-profession essentials everyone should know

Before diving into profession-specific servers, several MCP servers are universally useful regardless of your role. These are the "essential stack" recommended across every community list and directory.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **Filesystem** | Secure file read/write with configurable access controls | https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem | Official Reference |
| **Memory** | Knowledge graph-based persistent memory across sessions | https://github.com/modelcontextprotocol/servers/tree/main/src/memory | Official Reference |
| **Sequential Thinking** | Structured, reflective problem-solving through thought chains — the most-used server on Smithery (5,550+ uses) | https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking | Official Reference |
| **Fetch** | Web content fetching and conversion for LLM consumption | https://github.com/modelcontextprotocol/servers/tree/main/src/fetch | Official Reference |
| **Brave Search** | Privacy-focused web and local search via Brave's 30B+ page index | https://github.com/brave/brave-search-mcp-server | Official (Brave) |
| **Exa Search** | AI-native semantic search engine with embeddings-based discovery | https://github.com/exa-labs/exa-mcp-server | Official (Exa) |
| **Zapier** | Connect AI agents to 8,000+ apps with zero custom code | https://zapier.com/mcp | Official (Zapier) |
| **Composio / Rube** | Managed MCP for 500+ apps (Gmail, Slack, GitHub, CRM, etc.) with auth built in | https://mcp.composio.dev | Official (Composio) |
| **DeepL** | High-quality translation and text rewriting | https://github.com/DeepLcom/deepl-mcp-server | Official (DeepL) |
| **Context7** | Up-to-date library documentation for AI code editors (9,000+ libraries) | https://github.com/upstash/context7-mcp | Official (Upstash) |

---

## UI/UX designers

Figma's official MCP server has become the centerpiece of design-to-code workflows, letting AI agents read layout data, component structures, and design tokens directly from Figma files. The broader ecosystem adds 3D modeling, icon libraries, and chart generation.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **Figma MCP Server** | Official server providing design context (layout, variables, components, styles) to AI coding tools. Remote at `mcp.figma.com/mcp` and local desktop mode. Available on all paid plans with Dev Mode | https://developers.figma.com/docs/figma-mcp-server/ | Official (Figma) |
| **Framelink Figma MCP** | Community alternative that simplifies Figma API responses to relevant layout/styling info, optimized for Cursor | https://github.com/GLips/Figma-Context-MCP | Community |
| **21st.dev Magic** | Generate crafted UI components inspired by top design engineers | https://github.com/21st-dev/magic-mcp | Official (21st.dev) |
| **Blender MCP** | Connect Blender to Claude for 3D modeling, scene creation, and manipulation | Listed on mcp.so | Community |
| **Icons8 MCP** | Access the Icons8 icon library for design assets | https://smithery.ai/servers/icons8community/icons8mpc | Community |
| **AntV Chart Server** | Generate charts and data visualizations using the AntV library | https://smithery.ai/servers/antvis/mcp-server-chart | Community |
| **FlyonUI** | Build modern, production-ready UI blocks and components | https://github.com/themeselection/flyonui-mcp | Official (FlyonUI) |
| **Gluestack UI** | React Native–first UI development with Gluestack components | https://github.com/gauravsaini/gluestack-ui-mcp-server | Official (Gluestack) |
| **Cloudinary** | Media upload, transformation, and AI-powered image/video analysis | https://github.com/cloudinary/mcp-servers | Official (Cloudinary) |
| **JupyterCAD MCP** | Control JupyterCAD via natural language for CAD work | https://github.com/asmith26/jupytercad-mcp | Community |
| **SlideSpeak** | AI-powered presentation creation | Listed on awesome-mcp-servers | Official (SlideSpeak) |

---

## Software engineers

The developer category is by far the largest, spanning version control, databases, testing, package management, and code quality. **GitHub's official MCP server** alone offers repository management, PR automation, Actions intelligence, and security scanning.

### Version control and code collaboration

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **GitHub MCP Server** | The flagship developer MCP — repos, issues, PRs, Actions, code search, security findings, Dependabot. Remote (OAuth) and local (Docker/binary) | https://github.com/github/github-mcp-server | Official (GitHub) |
| **GitLab MCP Server** | Project management, CI/CD, merge requests, issues, pipelines. Beta for Premium/Ultimate | https://docs.gitlab.com/user/gitlab_duo/model_context_protocol/mcp_server/ | Official (GitLab) |
| **Git** | Direct local Git repository operations — read, search, analyze | https://github.com/modelcontextprotocol/servers/tree/main/src/git | Official Reference |
| **Gitea MCP** | Interact with self-hosted Gitea instances | https://gitea.com/gitea/gitea-mcp | Official (Gitea) |
| **Gitee MCP** | Gitee API integration for China-based repos | https://github.com/oschina/mcp-gitee | Official (Gitee) |
| **JetBrains MCP** | Work on code within JetBrains IDEs (IntelliJ, WebStorm, etc.) | https://github.com/JetBrains/mcp-jetbrains | Official (JetBrains) |
| **Gitingest MCP** | Generate quick GitHub repo summaries | https://github.com/puravparab/Gitingest-MCP | Community |

### Databases

The database MCP ecosystem is remarkably mature, with **official servers from nearly every major database vendor**. DBHub deserves special mention as a universal gateway supporting multiple databases through a single integration.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **PostgreSQL** | Read-only database access with schema inspection | https://github.com/modelcontextprotocol/servers-archived/tree/main/src/postgres | Official Reference |
| **Supabase** | Full platform access — tables, queries, edge functions, storage, migrations, TypeScript types | https://github.com/supabase-community/supabase-mcp | Official (Supabase) |
| **Neon** | Serverless Postgres with instant branching and auto-scaling | https://github.com/neondatabase/mcp-server-neon | Official (Neon) |
| **MongoDB** | Official MongoDB Community Server and Atlas integration | https://github.com/mongodb/mongodb-mcp-server | Official (MongoDB) |
| **MySQL** | MySQL with configurable access controls and schema inspection | https://github.com/designcomputer/mysql_mcp_server | Community |
| **SQLite** | Database interaction and business intelligence for SQLite | https://github.com/modelcontextprotocol/servers-archived/tree/main/src/sqlite | Official Reference |
| **Redis** | Natural language interface for Redis key-value operations | https://github.com/redis/mcp-redis | Official (Redis) |
| **ClickHouse** | Query ClickHouse analytical databases | https://github.com/ClickHouse/mcp-clickhouse | Official (ClickHouse) |
| **Neo4j** | Graph database with schema and read/write Cypher queries | https://github.com/neo4j-contrib/mcp-neo4j/ | Official (Neo4j) |
| **Elasticsearch** | Query data in Elasticsearch clusters | https://github.com/elastic/mcp-server-elasticsearch | Official (Elastic) |
| **Prisma Postgres** | Spin up databases, run migrations and queries | https://github.com/prisma/mcp | Official (Prisma) |
| **DuckDB** | Analytical database with schema inspection | https://github.com/ktanaka101/mcp-server-duckdb | Community |
| **MotherDuck** | MotherDuck cloud and local DuckDB | https://github.com/motherduckdb/mcp-server-motherduck | Official (MotherDuck) |
| **Qdrant** | Vector search engine for semantic memory | https://github.com/qdrant/mcp-server-qdrant/ | Official (Qdrant) |
| **Chroma** | Vector search, document storage, and full-text search | https://github.com/chroma-core/chroma-mcp | Official (Chroma) |
| **Milvus** | Milvus vector database interaction | https://github.com/zilliztech/mcp-server-milvus | Official (Milvus) |
| **DBHub** | Universal gateway — PostgreSQL, MySQL, SQLite, DuckDB via single integration | https://github.com/bytebase/dbhub | Community |
| **Airtable** | Airtable read/write with schema inspection | https://github.com/domdomegg/airtable-mcp-server | Community |
| **Couchbase** | Natural language Couchbase interaction | https://github.com/Couchbase-Ecosystem/mcp-server-couchbase | Official (Couchbase) |
| **SingleStore** | SingleStore database platform | https://github.com/singlestore-labs/mcp-server-singlestore | Official (SingleStore) |
| **Tinybird** | Serverless ClickHouse platform | https://github.com/tinybirdco/mcp-tinybird | Official (Tinybird) |
| **Google Toolbox for Databases** | Unified server for AlloyDB, BigQuery, Cloud SQL, Spanner, and more | https://github.com/googleapis/genai-toolbox | Official (Google) |

### Testing and browser automation

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **Playwright** | Microsoft's browser automation — tests, screenshots, scraping, multi-browser support | https://github.com/microsoft/playwright-mcp | Official (Microsoft) |
| **Puppeteer** | Chrome/Chromium browser automation and scraping | https://github.com/modelcontextprotocol/servers-archived/tree/main/src/puppeteer | Official Reference (archived) |
| **Browserbase** | Cloud-hosted headless browser automation | https://github.com/browserbase/mcp-server-browserbase | Official (Browserbase) |
| **BrowserStack** | Cross-browser real device testing | https://github.com/browserstack/mcp-server | Official (BrowserStack) |
| **Chrome DevTools** | Debug web pages directly in Chrome | https://github.com/ChromeDevTools/chrome-devtools-mcp | Official (Chrome) |
| **Browser MCP** | Automate your local browser instance | https://github.com/browsermcp/mcp | Community |

### APIs and backend

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **Postman** | Connect AI agents to API collections on Postman | https://github.com/postmanlabs/postman-mcp-server | Official (Postman) |
| **Apollo GraphQL** | Connect GraphQL APIs to AI agents | https://github.com/apollographql/apollo-mcp-server/ | Official (Apollo) |
| **APIMatic** | Validate OpenAPI specifications | https://github.com/apimatic/apimatic-validator-mcp | Official (APIMatic) |

### Code quality and security

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **SonarQube** | Code quality integration with SonarQube Server or Cloud | https://github.com/SonarSource/sonarqube-mcp-server | Official (SonarSource) |
| **Semgrep** | Static analysis and secure code scanning | https://github.com/semgrep/mcp | Official (Semgrep) |
| **GitGuardian** | Secret detection with 500+ detectors | https://github.com/GitGuardian/gg-mcp | Official (GitGuardian) |
| **CrowdStrike Falcon** | Security analysis, detections, threat intelligence | https://github.com/CrowdStrike/falcon-mcp | Official (CrowdStrike) |
| **Burp Suite** | PortSwigger web security testing | https://github.com/PortSwigger/mcp-server | Official (PortSwigger) |
| **Cycode** | SAST, SCA, Secrets, and IaC scanning | https://github.com/cycodehq/cycode-cli | Official (Cycode) |
| **Endor Labs** | Vulnerability and secret leak scanning | https://docs.endorlabs.com/deployment/ide/mcp/ | Official (Endor Labs) |

---

## DevOps and infrastructure

The DevOps MCP landscape features **official servers from all three major cloud providers** plus infrastructure-as-code tools. HashiCorp's Terraform MCP and Pulumi's MCP server both let AI agents provision and manage infrastructure via natural language.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **AWS MCP Servers** | Specialized servers for Lambda, ECS, S3, EC2, CDK, Bedrock, cost analysis, and docs | https://github.com/awslabs/mcp | Official (AWS) |
| **Azure MCP Server** | Azure Storage, Cosmos DB, AKS, Sentinel, Azure CLI, and more in a single server | https://github.com/microsoft/mcp | Official (Microsoft) |
| **Azure DevOps** | Repos, work items, builds, releases, test plans, code search | https://github.com/microsoft/azure-devops-mcp | Official (Microsoft) |
| **Google Cloud Run** | Deploy code to Google Cloud Run | https://github.com/GoogleCloudPlatform/cloud-run-mcp | Official (Google) |
| **Cloudflare** | Workers, KV, R2, D1 management and deployment | https://github.com/cloudflare/mcp-server-cloudflare | Official (Cloudflare) |
| **Terraform** | Registry APIs, workspace management, HCP integration, AGENTS.md support | https://github.com/hashicorp/terraform-mcp-server | Official (HashiCorp) |
| **AWS Terraform** | Terraform on AWS best practices, IaC patterns, Checkov compliance | https://awslabs.github.io/mcp/servers/terraform-mcp-server | Official (AWS) |
| **Pulumi** | Manage infra via Automation API — preview, deploy, get outputs | https://github.com/pulumi/mcp-server | Official (Pulumi) |
| **Docker** | Container and compose stack lifecycle management | https://github.com/QuantGeekDev/docker-mcp | Community |
| **Kubernetes** | Read-only cluster access — list resources, get pods, retrieve logs | Community implementations; Microsoft's AKS MCP is official | Community / Official (AKS) |
| **Firebase** | Firebase Authentication, Firestore, and Storage | https://github.com/firebase/firebase-tools/blob/master/src/mcp | Official (Firebase) |
| **Daytona** | AI-generated code execution in sandboxes | https://github.com/daytonaio/daytona/tree/main/apps/cli/mcp | Official (Daytona) |
| **E2B** | Secure cloud sandboxes for code execution | https://github.com/e2b-dev/mcp-server | Official (E2B) |
| **Render** | Deploy services, query databases, access logs | https://render.com/docs/mcp-server | Official (Render) |

### CI/CD

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **CircleCI** | Diagnose build failures, retrieve logs, identify flaky tests | https://github.com/CircleCI-Public/mcp-server-circleci | Official (CircleCI) |
| **Buildkite** | Pipelines, builds, jobs, and test analytics | https://github.com/buildkite/buildkite-mcp-server | Official (Buildkite) |
| **Jenkins** | Manage builds, check job statuses, retrieve build logs | Listed on modelcontextprotocol/servers | Official (Jenkins) |
| **GitHub Actions** | Included in GitHub MCP Server — workflow monitoring and failure analysis | https://github.com/github/github-mcp-server | Official (GitHub) |
| **Harness** | Access pipelines, repos, logs, artifact registries | https://github.com/harness/mcp-server | Official (Harness) |
| **JFrog** | Repository management, build tracking, release lifecycle | Listed on modelcontextprotocol/servers | Official (JFrog) |

### Monitoring and observability

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **Sentry** | Error tracking and AI-powered root cause analysis (Seer). Remote at `mcp.sentry.dev` | https://github.com/getsentry/sentry-mcp | Official (Sentry) |
| **Datadog** | APM, infrastructure, logs, and RUM monitoring | Listed on PulseMCP | Official (Datadog) |
| **Grafana** | Search dashboards, query Prometheus/Loki datasources, investigate incidents | https://github.com/grafana/mcp-grafana | Official (Grafana) |
| **Axiom** | Query and analyze logs, traces, events in natural language | https://github.com/axiomhq/mcp-server-axiom | Official (Axiom) |
| **Dynatrace** | Real-time observability and monitoring | https://github.com/dynatrace-oss/dynatrace-mcp | Official (Dynatrace) |
| **Logfire** | OpenTelemetry traces and metrics via Pydantic Logfire | https://github.com/pydantic/logfire-mcp | Official (Pydantic) |
| **Raygun** | Crash reporting and real user monitoring | https://github.com/MindscapeHQ/mcp-server-raygun | Official (Raygun) |
| **Last9** | Real-time production context — logs, metrics, traces | https://github.com/last9/last9-mcp-server | Official (Last9) |
| **Arize Phoenix** | AI/LLM observability — inspect traces, manage prompts, run experiments | https://github.com/Arize-ai/phoenix/tree/main/js/packages/phoenix-mcp | Official (Arize) |

---

## Content writers and marketers

Content professionals benefit from **WordPress MCP servers** for direct publishing, **SEO tools** for optimization, and **ad platform integrations** for campaign management. The WordPress MCP ecosystem alone has multiple mature implementations.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **WordPress MCP (Automattic)** | Official WordPress MCP plugin — REST API CRUD, dual transport | https://github.com/WordPress/mcp-adapter | Official (Automattic/WordPress) |
| **Claudeus WordPress MCP** | 145 production-ready tools covering content, media, users, plugins, themes, taxonomies | https://github.com/deus-h/claudeus-wp-mcp | Community |
| **WordPress MCP (5unnykum4r)** | 46 tools for posts, pages, media, SEO, comments, blocks. Built for Claude Code | https://github.com/5unnykum4r/wordpress-mcp | Community |
| **InstaWP WordPress MCP** | Multi-site WordPress management with staging environments | https://github.com/InstaWP/mcp-wp | Community |
| **Webflow** | CMS management, SEO auditing, content localization, site publishing | https://github.com/webflow/mcp-server | Official (Webflow) |
| **Shopify** | Product catalogs, pricing, inventory, cart management — auto-enabled on all stores | https://shopify.dev/docs/apps/build/storefront-mcp | Official (Shopify) |
| **Audiense Insights** | Marketing audience analysis and insights from Audiense reports | https://github.com/AudienseCo/mcp-audiense-insights | Official (Audiense) |
| **Google Ads MCP** | Natural-language advertising data analysis and campaign insights | https://github.com/cohnen/mcp-google-ads | Community |
| **Facebook Ads MCP** | Programmatic access to Facebook Ads data management | https://github.com/gomarble-ai/facebook-ads-mcp-server | Community |
| **Airano MCP SEO Bridge** | WordPress plugin exposing Rank Math/Yoast SEO meta fields via REST API | https://wordpress.org/plugins/airano-mcp-seo-bridge/ | Community |
| **FetchSERP** | All-in-one SEO and web intelligence toolkit | https://github.com/fetchSERP/fetchserp-mcp-server-node | Official (FetchSERP) |
| **Bing Webmaster Tools** | Site management, URL submission, and traffic analysis | https://github.com/zizzfizzix/mcp-server-bwt | Community |
| **SegmentStream** | Cross-channel attribution, anomaly detection, budget optimization | https://segmentstream.com | Official (SegmentStream) |
| **Mailgun** | Email sending via Mailgun API | https://github.com/mailgun/mailgun-mcp-server | Official (Mailgun) |
| **Mailtrap** | Email API integration and testing | https://github.com/railsware/mailtrap-mcp | Official (Mailtrap) |
| **Klaviyo** | Marketing data and campaign interaction | Listed on modelcontextprotocol/servers | Official (Klaviyo) |
| **ElevenLabs** | Text-to-speech for podcast/audio content creation | https://github.com/elevenlabs/elevenlabs-mcp | Official (ElevenLabs) |

---

## Data analysts and scientists

Data professionals get **official MCP servers from every major warehouse vendor** — BigQuery, Snowflake, Databricks, and ClickHouse all have first-party support. The **dbt MCP server** bridges the gap between data modeling and AI-assisted analysis.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **BigQuery (Google)** | Official Google Cloud BigQuery MCP — natural language querying, Cortex AI, semantic views | https://docs.cloud.google.com/bigquery/docs/use-bigquery-mcp | Official (Google) |
| **BigQuery (Community)** | Community server with schema inspection and SQL execution | https://github.com/LucasHild/mcp-server-bigquery | Community |
| **Snowflake** | Cortex AI, object management, SQL orchestration, RBAC permissions | https://github.com/Snowflake-Labs/mcp | Official (Snowflake) |
| **Databricks** | Managed MCP for data, AI tools, and agents within Databricks governance | https://docs.databricks.com/aws/en/generative-ai/mcp/ | Official (Databricks) |
| **dbt** | Data build tool — metadata discovery, semantic layer, model lineage | https://github.com/dbt-labs/dbt-mcp | Official (dbt Labs) |
| **ClickHouse** | Query analytical ClickHouse databases | https://github.com/ClickHouse/mcp-clickhouse | Official (ClickHouse) |
| **Google Sheets MCP** | Read, write, and manipulate spreadsheets | https://mcp.so/server/mcp-google-sheets/xing5 | Community |
| **Excel MCP** | Excel workbook manipulation without Microsoft Excel | https://github.com/haris-musa/excel-mcp-server | Community |
| **Dot (GetDot.ai)** | AI data analyst for Snowflake, BigQuery, Redshift, Databricks, ClickHouse | https://getdot.ai | Official (GetDot) |
| **Grafana MCP** | Monitor, analyze, and visualize data; query Prometheus/Loki datasources | https://github.com/grafana/mcp-grafana | Official (Grafana) |
| **Coupler.io** | 70+ data source integrations including QuickBooks, Xero, Stripe, Salesforce | https://www.coupler.io/mcp | Official (Coupler.io) |
| **CData Connect AI** | Managed MCP for 350+ enterprise data sources | https://www.cdata.com | Official (CData) |
| **Snowflake (Community)** | Schema exploration, SQL execution, and data insight aggregation | https://github.com/isaacwasserman/mcp-snowflake-server | Community |
| **Tinybird** | Real-time analytics on ClickHouse | https://github.com/tinybirdco/mcp-tinybird | Official (Tinybird) |
| **MotherDuck** | Cloud DuckDB analytical queries | https://github.com/motherduckdb/mcp-server-motherduck | Official (MotherDuck) |

---

## Project managers

Project management has some of the strongest official MCP adoption. **Notion, Asana, Atlassian (Jira + Confluence), and Linear** all provide first-party remote MCP servers with OAuth authentication, enabling AI agents to create tasks, update statuses, and search across project data.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **Notion** | Official hosted server — search workspace, manage pages/databases, add comments, search connected tools | https://github.com/makenotion/notion-mcp-server | Official (Notion) |
| **Atlassian (Jira + Confluence)** | Official remote MCP for Jira issues/workflows and Confluence pages/spaces | https://www.atlassian.com/platform/remote-mcp-server | Official (Atlassian) |
| **Atlassian (Community)** | Popular community fork supporting Jira + Confluence with Cloud and Server/Data Center | https://github.com/sooperset/mcp-atlassian | Community |
| **Asana** | Official remote server — tasks, workspaces, projects with one-click setup | https://mcp.asana.com/sse | Official (Asana) |
| **Linear** | Official remote server at `mcp.linear.app/mcp` — issues, projects, teams | https://linear.app/docs/mcp | Official (Linear) |
| **Plane** | AI automation of Plane projects, work items, and cycles | https://github.com/makeplane/plane-mcp-server | Official (Plane) |
| **Taskade** | Tasks, projects, workflows, and AI agents | https://github.com/taskade/mcp | Official (Taskade) |
| **Dart** | AI-native project management — tasks, docs, projects | https://github.com/its-dart/dart-mcp-server | Official (Dart) |
| **ClickUp MCP** | Create, update, organize tasks, lists, folders, and tags | https://mcpservers.org/servers/imjoshnewton/clickup-mcp-server | Community |
| **Trello MCP** | Board, list, card, and checklist management | https://github.com/andypost/mcp-server-ts-trello | Community |
| **Monday.com** | Manage boards, items, and workspaces (via Composio) | https://mcp.composio.dev | Community |
| **Todoist MCP** | Natural language task management with Todoist | Listed on awesome-mcp-servers | Community |
| **GitKraken** | CLI spanning GitKraken, Jira, GitHub, and GitLab | https://github.com/gitkraken/gk-cli | Official (GitKraken) |

---

## Sales and CRM

**HubSpot leads CRM adoption** with both remote and local MCP servers in public beta — it was the first CRM to launch a production MCP integration and the first third-party MCP connector in ChatGPT's registry. Salesforce followed with hosted MCP servers in beta as of October 2025.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **HubSpot (Remote)** | Official remote MCP — contacts, companies, deals, tickets, products, invoices via natural language. OAuth 2.0 with PKCE | https://developers.hubspot.com/mcp | Official (HubSpot) |
| **HubSpot (Developer)** | CLI-based server for HubSpot Developer Platform | https://developers.hubspot.com/mcp | Official (HubSpot) |
| **HubSpot (Community)** | Community server with vector storage (FAISS) and caching for improved performance | https://github.com/peakmojo/mcp-hubspot | Community |
| **Salesforce Hosted MCP** | Official hosted servers — SOQL, metadata, record CRUD. Beta since Oct 2025 | https://developer.salesforce.com | Official (Salesforce) |
| **Salesforce DX MCP** | Deploy code, create scratch orgs, run tests via natural language | https://developer.salesforce.com | Official (Salesforce) |
| **mcp-server-salesforce** | Community Salesforce — SOQL, SOSL, Apex, object management | https://github.com/tsmztech/mcp-server-salesforce | Community |
| **Pipedrive MCP** | Access users, leads, pipelines, organizations (16+ tools) | https://github.com/jusFood/pipedrive-mcp | Community |
| **Dynamics 365 CRM** | Sales and customer service data, quotes, and orders | Microsoft official | Official (Microsoft) |
| **Intercom** | Customer messaging integration, remote MCP | Listed on modelcontextprotocol/servers | Official (Intercom) |
| **Coresignal** | B2B intelligence — company data, employees, job postings | https://github.com/Coresignal-com/coresignal-mcp/ | Official (Coresignal) |
| **GoHighLevel MCP** | Automate CRM, messaging, calendars, marketing, billing | Listed on MCPBench | Community |
| **Freshdesk MCP** | AI-driven ticket management and support operations | https://github.com/effytech/freshdesk_mcp | Community |

---

## Communication and productivity

The communication category features **official servers from Slack and Zoom**, alongside the wildly popular community-built Google Workspace MCP (1,700+ stars) that covers Gmail, Calendar, Drive, Docs, Sheets, and more in a single integration. Claude itself now has built-in Google Workspace connectors.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **Slack (Official)** | Official Slack MCP — search channels/messages/files, send messages, read threads, manage canvases | https://docs.slack.dev/ai/slack-mcp-server/ | Official (Slack) |
| **Slack (korotovsky)** | Most powerful community Slack MCP — OAuth, stealth mode, DMs, smart history. 398+ stars | https://github.com/korotovsky/slack-mcp-server | Community |
| **Google Workspace MCP** | Comprehensive: Gmail, Drive, Docs, Sheets, Slides, Calendar, Chat, Forms, Tasks. 1,700+ stars | https://github.com/taylorwilsdon/google_workspace_mcp | Community |
| **Claude Google Connectors** | Native Gmail, Calendar, and Drive integration built into Claude | https://support.claude.com/en/articles/10166901 | Official (Anthropic) |
| **Microsoft 365 MCP** | Full M365 suite via Graph API — Outlook, Teams, OneDrive, SharePoint | https://github.com/softeria/ms-365-mcp-server | Community |
| **Microsoft Teams** | Read messages, create messages, reply to threads, mention members | https://github.com/inditextech/mcp-teams-server | Community |
| **Zoom** | MCP integration in Zoom AI Companion/AI Studio | https://www.zoom.com | Official (Zoom, emerging) |
| **tl;dv** | Record, transcribe, summarize meetings across Google Meet, Zoom, and Teams | https://tldv.io | Official (tl;dv) |
| **LINE Official Account** | LINE Messaging API integration | https://github.com/line/line-bot-mcp-server | Official (LINE) |
| **Twilio** | Messages, phone numbers, and account management | https://github.com/twilio-labs/mcp | Official (Twilio) |
| **Courier** | Multi-channel notifications: email, SMS, push, Slack, Teams | Listed on modelcontextprotocol/servers | Official (Courier) |
| **WhatsApp MCP** | Send messages, search contacts, send files | https://github.com/lharries/whatsapp-mcp | Community |
| **Discord MCP** | Send/read messages, discover channels | https://github.com/v-3/discordmcp | Community |
| **Telegram Bot MCP** | Full Telegram Bot API with 174 tools | https://github.com/FantomaSkaRus1/telegram-bot-mcp | Community |
| **Inbox Zero** | AI personal email assistant | https://github.com/elie222/inbox-zero/tree/main/apps/mcp-server | Official (Inbox Zero) |

---

## Research and knowledge management

Researchers benefit from **arXiv paper search**, **Obsidian vault access**, and knowledge graph tools. Notion and Confluence (covered under Project Management) also serve as primary knowledge bases for many research teams.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **ArXiv MCP** | Search arXiv papers with advanced filtering, download as markdown, deep research analysis | https://github.com/blazickjp/arxiv-mcp-server | Community |
| **Obsidian MCP** | Read, search, and modify Obsidian vault notes via Local REST API | https://github.com/MarkusPfundstein/mcp-obsidian | Community |
| **Obsidian Advanced** | Enhanced Obsidian MCP with advanced search and automation | https://fastmcp.me/mcp/details/1212/obsidian-advanced | Community |
| **Apple Notes** | Read and interact with Apple Notes | https://github.com/RafalWilinski/mcp-apple-notes | Community |
| **Perplexity** | Real-time web research via Sonar API | https://github.com/ppl-ai/modelcontextprotocol | Official (Perplexity) |
| **Tavily** | Search API built for AI agents with RAG focus and citation-ready results | https://github.com/tavily-ai/tavily-mcp | Official (Tavily) |
| **HuggingFace** | Access millions of AI models, datasets, Spaces, and papers | https://huggingface.co/mcp | Official (HuggingFace) |
| **Graphlit** | Ingest data from Slack, Gmail, podcasts, web crawl and query it | https://github.com/graphlit/graphlit-mcp-server | Official (Graphlit) |
| **Meilisearch** | Full-text and semantic search engine | https://github.com/meilisearch/meilisearch-mcp | Official (Meilisearch) |
| **Bright Data** | Web scraping and data extraction for research | https://github.com/luminati-io/brightdata-mcp | Official (Bright Data) |
| **Apify** | 6,000+ cloud tools for web scraping and data extraction | https://github.com/apify/apify-mcp-server | Official (Apify) |
| **Firecrawl** | Intelligent web content extraction with 83% benchmark accuracy | https://github.com/firecrawl/firecrawl-mcp-server | Official (Firecrawl) |
| **Kagi Search** | High-quality web search via Kagi API | https://github.com/kagisearch/kagimcp | Official (Kagi) |
| **Markdownify** | Convert audio, PowerPoint, PDF, and web pages to Markdown | https://github.com/zcaceres/markdownify-mcp | Community |

---

## Finance and accounting

The finance MCP ecosystem ranges from **Stripe's payment processing** to **stock market data** and **accounting integrations**. AlphaVantage provides 100+ financial data APIs, while newer entrants like Ramp and Norman Finance target spend analysis and bookkeeping.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **Stripe** | Official MCP — customers, products, payments, invoices, subscriptions. Remote at `mcp.stripe.com` | https://github.com/stripe/agent-toolkit | Official (Stripe) |
| **PayPal** | Payment processing via agent toolkit | https://github.com/paypal/agent-toolkit | Official (PayPal) |
| **Block (Square)** | Block/Square payment services | Listed on modelcontextprotocol/servers | Official (Block) |
| **AlphaVantage** | 100+ financial market data APIs — stocks, ETFs, options, forex, crypto | https://mcp.alphavantage.co/ | Official (AlphaVantage) |
| **Financial Datasets** | Stock market API for AI agents | https://github.com/financial-datasets/mcp-server | Official |
| **CoinGecko** | Crypto price and market data, 200+ blockchain networks, 8M+ tokens | https://docs.coingecko.com/reference/mcp-server/ | Official (CoinGecko) |
| **Alpaca** | Stock and options trading | https://github.com/alpacahq/alpaca-mcp-server | Official (Alpaca) |
| **Twelve Data** | Real-time and historical financial data | https://github.com/twelvedata/mcp | Official (Twelve Data) |
| **Octagon** | Real-time investment research, private and public market data | https://github.com/OctagonAI/octagon-mcp-server | Official (Octagon) |
| **Ramp** | Spend analysis and corporate card insights | https://github.com/ramp-public/ramp-mcp | Official (Ramp) |
| **Norman Finance** | Accounting and tax management | https://github.com/norman-finance/norman-mcp-server | Official (Norman) |
| **QuickBooks (via Coupler.io)** | Cash flow, P&L, balance sheet, expense tracking via natural language | https://www.coupler.io/mcp/quickbooks | Third-party (Coupler.io) |
| **Xero (via Coupler.io)** | Cash flow analysis, accounts receivable, expense monitoring | https://www.coupler.io/mcp/xero | Third-party (Coupler.io) |
| **Midday** | Business finance platform with native MCP — integrates QuickBooks, Xero, Stripe | https://midday.ai | Official (Midday) |
| **Dynamics 365 ERP** | Finance/procurement — purchase requisitions, vendor management | Microsoft official | Official (Microsoft) |
| **Chronulus AI** | Forecasting and prediction agents | https://github.com/ChronulusAI/chronulus-mcp | Official (Chronulus) |

---

## Legal professionals

Legal MCP servers are still an emerging category, but several useful implementations already exist. The **US Legal MCP** provides access to Congress bills, Federal Register documents, and court opinions, while specialized servers cover **German law**, **sanctions frameworks**, and **compliance analysis**.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **US Legal MCP** | Search Congress bills, Federal Register, court opinions (CourtListener). No API key required | https://github.com/JamesANZ/us-legal-mcp | Community |
| **Open Legal Compliance** | Multi-jurisdiction compliance: US Code, Federal Register, EUR-Lex, SEC EDGAR, UK Legislation, CanLII | https://github.com/TCoder920x/open-legal-compliance-mcp | Community |
| **German Law MCP** | 6,870 German statutes, 91,843 provisions, EU cross-references. Hosted remote server available | https://github.com/Ansvar-Systems/German-law-mcp | Community |
| **Sanctions Law MCP** | UN, EU, US, UK sanctions frameworks and CJEU case law (1,280 provisions, 174 executive orders) | https://github.com/Ansvar-Systems/sanctions-law-mcp | Community |
| **Cerebra Legal** | Enterprise-grade legal reasoning with auto-detection across 8 legal domains | https://github.com/yoda-digital/mcp-cerebra-legal-server | Community |
| **Legal Document Analysis** | AI summarization and classification of legal documents | https://github.com/vinothezhumalai/legalmcpserver | Community |
| **Lex API (UK)** | Semantic search for UK legislation and caselaw | https://lex.lab.i.ai.gov.uk/ | Government |
| **Drata** | Real-time compliance intelligence for SOC 2, ISO 27001, etc. | Listed on modelcontextprotocol/servers | Official (Drata) |

---

## E-commerce

Shopify dominates with **three official MCP servers** — storefront, customer accounts, and developer tooling — all auto-enabled for every Shopify store. The broader ecosystem covers WooCommerce, Mercado Libre, and multi-platform tools.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **Shopify Storefront** | Product search, cart, policies — each store gets its own endpoint | https://shopify.dev/docs/apps/build/storefront-mcp/servers/storefront | Official (Shopify) |
| **Shopify Customer Accounts** | Order tracking, returns, account information | https://shopify.dev/docs/apps/build/storefront-mcp | Official (Shopify) |
| **Shopify Dev MCP** | Search Shopify docs, explore API schemas, build Functions | https://shopify.dev/docs/apps/build/devmcp | Official (Shopify) |
| **shopify-mcp (Community)** | Full CRUD — products, orders, customers, inventory (31+ tools) | https://github.com/GeLi2001/shopify-mcp | Community |
| **WooCommerce MCP** | Read-only access to WooCommerce products, categories, reviews | Listed on modelcontextprotocol/servers | Community |
| **Mercado Libre** | Official marketplace integration | https://mcp.mercadolibre.com/ | Official (Mercado Libre) |
| **Mercado Pago** | Official payment processing | https://mcp.mercadopago.com/ | Official (Mercado Pago) |

---

## HR and recruiting

HR-specific MCP servers are in early stages, though **BambooHR** and **LinkedIn Jobs** integrations exist. The **Knit MCP platform** provides the most comprehensive coverage, connecting to HRIS, payroll, and ATS systems through a single integration.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **BambooHR MCP** | Employee data, time tracking, and HR management | Listed on modelcontextprotocol/servers | Community |
| **LinkedIn Jobs MCP** | Real-time LinkedIn job search with geocoding and intelligent filtering | https://github.com/GhoshSrinjoy/linkedin-job-mcp | Community |
| **ZIZAI Recruitment** | Intelligent recruitment platform | Listed on modelcontextprotocol/servers | Third-party |
| **Knit MCP** | Production-ready remote MCP for HRIS, Payroll, ATS, and 10,000+ tools | https://developers.getknit.dev/docs/knit-mcp-server-getting-started | Official (Knit) |

---

## Healthcare

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **FHIR MCP** | Fast Healthcare Interoperability Resources with SMART-on-FHIR auth | https://github.com/wso2/fhir-mcp-server/ | Official (WSO2) |

---

## Media and entertainment

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **ElevenLabs** | Text-to-speech and voice cloning | https://github.com/elevenlabs/elevenlabs-mcp | Official (ElevenLabs) |
| **Mux** | Video upload, live streams, thumbnails, captions | https://github.com/muxinc/mux-node-sdk/tree/master/packages/mcp-server | Official (Mux) |
| **Mureka** | Generate lyrics, songs, and background music | https://github.com/SkyworkAI/Mureka-mcp | Official (Mureka) |
| **Cartesia** | Voice platform for TTS and voice cloning | https://github.com/cartesia-ai/cartesia-mcp | Official (Cartesia) |
| **DAISYS** | High-quality TTS and voice outputs | https://github.com/daisys-ai/daisys-mcp | Official (DAISYS) |
| **VideoDB Director** | AI-powered video workflows | Listed on modelcontextprotocol/servers | Official (VideoDB) |

---

## Automation and integration platforms

These "meta-servers" connect AI to hundreds or thousands of apps through a single MCP, making them valuable for any profession that needs broad tool access without configuring individual servers.

| Server | Description | URL | Type |
|--------|-------------|-----|------|
| **Zapier** | 8,000+ app integrations with zero custom code | https://zapier.com/mcp | Official (Zapier) |
| **Composio / Rube** | 500+ managed MCP servers with built-in authentication | https://mcp.composio.dev | Official (Composio) |
| **Make** | Turn Make scenarios into callable AI tools | https://github.com/integromat/make-mcp-server | Official (Make) |
| **n8n** | Workflow automation with 525+ nodes | https://n8n.io | Official (n8n) |
| **Knit** | 10,000+ tools across HRIS, ATS, CRM, Accounting, Calendar | https://developers.getknit.dev/docs/knit-mcp-server-getting-started | Official (Knit) |
| **ActionKit (Paragon)** | 130+ SaaS integrations | Listed on modelcontextprotocol/servers | Official (Paragon) |
| **Integration App** | Interact with SaaS apps on behalf of customers | https://github.com/integration-app/mcp-server | Official |
| **Strata** | Guide agents through thousands of tools across apps | Listed on glama.ai | Official (Strata) |

---

## Where to discover more MCP servers

The ecosystem is evolving rapidly. These directories are the best places to find new servers as they launch:

- **Official MCP Registry** — https://registry.modelcontextprotocol.io/ (backed by Anthropic, GitHub, Microsoft)
- **PulseMCP** — https://www.pulsemcp.com/servers (12,370+ servers, updated daily)
- **Glama.ai** — https://glama.ai/mcp/servers (76 categories, security-scored)
- **Smithery.ai** — https://smithery.ai/servers (5,000+ with usage metrics)
- **mcp.so** — https://mcp.so (18,800+ servers, largest collection)
- **GitHub MCP Registry** — https://github.com/mcp
- **awesome-mcp-servers** — https://github.com/wong2/awesome-mcp-servers and https://github.com/punkpeye/awesome-mcp-servers

## Conclusion

The MCP ecosystem has matured remarkably fast since its introduction. **The most significant trend is the shift toward official, company-maintained servers** — major platforms like GitHub, Salesforce, HubSpot, Stripe, Shopify, Notion, Atlassian, and all three major cloud providers now offer first-party MCP servers, often with remote OAuth-authenticated endpoints that require zero local setup. For professions outside software engineering, the strongest official coverage exists in project management, CRM, payments, and communication. Legal, HR, and healthcare remain frontier areas with primarily community-built options. Aggregator platforms like Composio, Zapier, and Knit effectively bridge gaps for any profession by connecting to hundreds of apps through a single MCP server — making them the pragmatic choice when a dedicated MCP server doesn't yet exist for a specific tool.