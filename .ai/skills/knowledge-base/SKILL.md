---
name: knowledge-base
description: >
  How to use and set up the shared `kb` semantic knowledge base (RAG) in a project.
  Use this skill whenever you need to ground an answer on ingested material — research
  papers, personal notes and decisions, or external API/vendor docs — or when the user
  mentions the knowledge base, `kb_search`, RAG, "search my notes/research/docs", "what
  does the paper/research say", "check the ingested docs", or wants to add/ingest documents
  or set up a knowledge base for a project. Covers querying an existing project's KB via the
  `kb_search` / `kb_list_sources` MCP tools, and wiring a new project's KB (`.kb.toml`,
  `knowledge/`, the `knowledge` MCP server, and `kb ingest`). Prefer this over guessing
  whenever the answer might live in ingested documents rather than in the code itself.
---

# Knowledge base (RAG)

`kb` is a shared, system-wide CLI (the `rag-kb` tool) that gives a project a semantic
knowledge base: documents are chunked, embedded, and stored in a per-project Qdrant
collection on the dataserver (Qdrant + Ollama, `mxbai-embed-large`). Agents query it to
ground answers on material that isn't in the code — research papers, the user's own notes
and decisions, and external API/vendor docs.

Global endpoints live in `~/.config/kb/config.toml`; per-project settings live in the
project's `.kb.toml`. Each project has its own collection, so knowledge never leaks across
projects.

## When to use the KB

Reach for the KB when the answer likely lives in ingested documents rather than the source
tree: model/methodology questions, "what does the research say about X", broker/API/vendor
behaviour, or prior design decisions the user has written down. It complements — it does not
replace — the code and the project's `.ai/context/` files. Numeric/market/tabular data does
**not** belong in the KB (semantic search over number grids is weak); query the project's
database (e.g. ClickHouse) for that.

## Querying an existing project's KB

A project has a KB if it has a `.kb.toml` and a `knowledge` server in `.mcp.json`. When it
does, two MCP tools are available:

- `kb_search(query, source_type?, top_k?)` — semantic search. Returns the top matching
  chunks, each with its source `file`, `source_type` (`research | notes | api | data`),
  similarity `score`, and section `heading`. Pass `source_type` to restrict the search
  (e.g. only `research`); `top_k` widens/narrows results (default 6).
- `kb_list_sources()` — lists what's currently ingested (file, source_type, chunk count).

Prefer `kb_search` over guessing when a question may be answered by ingested material, and
cite the returned `file` so the user can trace the source. If no `knowledge` MCP server is
loaded in this session, the same query works from the shell: `kb search "your question" -k 6`.

## Setting up a KB for a project

If a project should have a KB but doesn't yet, wire it in four small steps:

1. **`.kb.toml`** at the project root:
   ```toml
   collection = "kb_<project>"
   sources    = ["knowledge/sources"]
   ```
2. **`knowledge/`** folder: `knowledge/sources/` for material, plus a `knowledge/.gitignore`
   that ignores `dump/` and `sources/` (raw material is not tracked; the `manifest.jsonl`
   ledger is). A short `knowledge/README.md` helps humans.
3. **`.mcp.json`** — add a stdio server so agents get the retrieval tools:
   ```json
   "knowledge": { "type": "stdio", "command": "kb", "args": ["mcp"] }
   ```
4. **`AI.md`** — add a short "Knowledge base (RAG)" section so agents know to use `kb_search`
   and where material lives.

## Ingesting and maintaining

- `kb ingest [PATH] --source-type <research|notes|api|data>` — chunk, embed, and upsert
  files into the collection. Idempotent (skips unchanged files by hash), so re-run freely
  after adding material. One unreadable file is logged and skipped, not fatal.
- `kb status` — backend health + the collection's point count.
- `kb search "query" -k N [-s source_type]` — quick shell search.
- `kb remove <file>` — drop a source's chunks + its manifest entry.
- `kb --help` — full CLI.

Choose `source_type` honestly (`research` for papers, `notes` for the user's own writing,
`api` for vendor/API docs, `data` for narrative analysis/backtest reports) — agents filter
on it, so accurate tags make retrieval sharper.
