# MCP Server Catalog

MCP servers extend Claude's capabilities by connecting to external tools and services.
This catalog maps project characteristics to recommended MCP servers with install commands.

**Setup & Team Sharing:**
- **Project config** (`.mcp.json`) — Available only in that directory
- **Global config** (`~/.claude.json`) — Available across all projects
- **Checked-in `.mcp.json`** — Available to entire team (recommended!)
- **Debugging**: Use `claude --mcp-debug` to identify configuration issues.

---

## Quick Reference: Detection Patterns

| Look For | Suggests MCP Server |
|----------|-------------------|
| Popular npm packages | context7 |
| React/Vue/Next.js | Playwright MCP |
| `@supabase/supabase-js` | Supabase MCP |
| `pg` or `postgres` | PostgreSQL MCP |
| GitHub remote | GitHub MCP |
| `.linear` or Linear refs | Linear MCP |
| `@aws-sdk/*` | AWS MCP |
| `@sentry/*` | Sentry MCP |
| `docker-compose.yml` | Docker MCP |
| Slack webhook URLs | Slack MCP |

---

## Documentation & Research

### context7
**Best for**: Projects using popular libraries/SDKs — Claude fetches live documentation instead of relying on training data.

| Recommend When | Examples |
|----------------|----------|
| Using React, Vue, Angular | Frontend frameworks |
| Using Express, FastAPI, Django | Backend frameworks |
| Using Prisma, Drizzle | ORMs |
| Using Stripe, Twilio, SendGrid | Third-party APIs |
| Using AWS SDK, Google Cloud | Cloud SDKs |
| Using LangChain, OpenAI SDK | AI/ML libraries |

**Install:** `claude mcp add context7 -- npx -y @upstash/context7-mcp@latest`

---

## Databases

### PostgreSQL MCP
**Best for**: Direct PostgreSQL database access, migrations, data analysis.
**Install:** `claude mcp add postgres -- npx -y @modelcontextprotocol/server-postgres`

### Supabase MCP
**Best for**: Projects using Supabase for backend/database/auth.
**Install:** `claude mcp add supabase --transport http https://mcp.supabase.com`

### SQLite MCP
**Best for**: SQLite-based projects.
**Install:** `claude mcp add sqlite -- npx -y @modelcontextprotocol/server-sqlite`

### Neon MCP
**Best for**: Neon serverless Postgres users.

### Turso MCP
**Best for**: Turso/libSQL edge database users.

---

## Version Control & DevOps

### GitHub MCP
**Best for**: GitHub-hosted repos needing issue/PR integration, CI/CD access.
**Install:** `claude mcp add github --transport http https://api.githubcopilot.com/mcp/ -H "Authorization: Bearer $GITHUB_PAT"`

### GitLab MCP
**Best for**: GitLab-hosted repositories.

### Linear MCP
**Best for**: Teams using Linear for issue tracking.

---

## Testing & Browser Automation

### Playwright MCP
**Best for**: Frontend projects needing E2E tests, browser automation, screenshots.
**Install:** `claude mcp add playwright -- npx -y @anthropic/mcp-playwright`

### Puppeteer MCP
**Best for**: Headless browser automation, web scraping, PDF generation.

---

## Error Monitoring & Observability

### Sentry MCP
**Best for**: Error tracking, production debugging, release correlation.
**Install:** `claude mcp add sentry --transport http https://mcp.sentry.dev/mcp`

### Datadog MCP
**Best for**: APM, logs, and metrics.

---

## Cloud Infrastructure

### AWS MCP
**Best for**: AWS infrastructure management (Lambda, S3, DynamoDB, CDK).

### Cloudflare MCP
**Best for**: Cloudflare Workers, Pages, R2, D1.

### Vercel MCP
**Best for**: Vercel deployment and configuration.

---

## Communication & Docs

### Slack MCP
**Best for**: Slack workspace integration (notifications, incident response).

### Notion MCP
**Best for**: Notion workspace for documentation and knowledge base.

---

## Design

### Figma MCP
**Best for**: Projects with Figma design files.
**Install:** `claude mcp add figma --transport http https://mcp.figma.com`

---

## Containers & DevOps

### Docker MCP
**Best for**: Container management, Dockerfile/Compose debugging.

### Kubernetes MCP
**Best for**: Kubernetes cluster management, Helm charts.

---

## .mcp.json Format

```json
{
  "mcpServers": {
    "server-name": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@scope/package-name"],
      "env": {
        "API_KEY": "${ENV_VAR_NAME}"
      }
    }
  }
}
```

For HTTP/remote servers:
```json
{
  "mcpServers": {
    "server-name": {
      "type": "http",
      "url": "https://mcp.example.com/mcp",
      "headers": {
        "Authorization": "Bearer ${API_TOKEN}"
      }
    }
  }
}
```

Environment variable expansion: `${VAR}` and `${VAR:-default}`.

---

## .claude/rules/ Patterns

Common rule files by project type:

| Project Type | Rule Files |
|--------------|-----------|
| Web App | `api-rules.md`, `testing-rules.md`, `component-rules.md` |
| Rust | `rust-conventions.md`, `testing-rules.md` |
| Python | `python-style.md`, `testing-rules.md` |
| Data Pipeline | `schema-rules.md`, `query-rules.md` |
| NixOS / Nix | `nix-conventions.md` |

---

## Workflow Suggestions by Project Type

| Project Type | Recommended Setup |
|--------------|-------------------|
| Single-dev CLI/Library | CLAUDE.md + Context7 + GitHub MCP |
| Full-stack Web App | CLAUDE.md + rules (frontend/backend/test) + GitHub + Playwright + DB MCP + Context7 |
| Data Pipeline / ETL | CLAUDE.md + schema rules + DB MCP |
| Multi-service | Per-service CLAUDE.md + GitHub MCP + `--add-dir` |
| Nix-managed Config | CLAUDE.md + module rules + Context7 |
