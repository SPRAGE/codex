# AI.md

## Project

PROJECTNAME - TODO: replace with one-line description.

## Getting Started

1. Replace `PROJECTNAME` in `.ai/instructions.md`, `AI.md`, `AGENTS.md`, `CODEX.md`, and `flake.nix`
2. `direnv allow` to enter the dev shell
3. Use `planner` to brainstorm your project, then `cc-setup` to generate config
   - OR use `virtual-tech-org` for autonomous staged delivery (discovery -> production)

## Stack

TODO: fill in after running `cc-setup`.

## Commands

- `nix develop` - enter dev shell
- `nix run github:SPRAGE/dev-template#sync-skills` - pull latest shared skills, managed adapters, provider skill links, Codex config/custom agents, hooks, and AI context templates
- `nix run github:SPRAGE/dev-template#ai-doctor` - validate AI context files, shared skills, provider skill links, Codex config/custom agents, and hooks

## Architecture

TODO: fill in after running `cc-setup` or manually.

## Conventions

TODO: fill in after running `cc-setup` or manually.

## Agent Workflow

- Start by inspecting the current tree and git status.
- Prefer `rg`, `fd`, and `jq` for codebase exploration when available.
- Keep edits scoped to the requested behavior and existing project style.
- Update `.ai/context/active-context.md` when work spans sessions or changes project direction.
- Run the relevant build, test, lint, or format checks listed above before finishing.

## Safety

- Treat `.env*`, key files, tokens, and credentials as sensitive.
- Do not overwrite local AI settings such as `.claude/settings.local.json`, `.agents/local/`, `.agents/tmp/`, `.codex/local/`, or `.codex/tmp/`.
- Do not run destructive git or filesystem operations unless the user explicitly asks.

## Shared AI Context & Skills

Provider-neutral context and skills live in `.ai/`. Read order, response style, rules, and the provider-adapter map are in `.ai/instructions.md`. Shared skills are in `.ai/skills/`; `.agents/skills/`, `.claude/skills/`, and `.codex/skills/` are symlinks to it — read `.ai/skills/<name>/SKILL.md` for a skill's source.

## Knowledge base (RAG)

This environment provides a shared, system-wide `kb` CLI (the `rag-kb` tool) — a project-agnostic semantic knowledge base backed by Qdrant + Ollama on the dataserver. A project opts in with a `.kb.toml` (its own Qdrant collection) plus a `knowledge/` folder for source material.

- If this project has a `.kb.toml` and a `knowledge` MCP server (in `.mcp.json`), use `kb_search(query, source_type?, top_k?)` and `kb_list_sources()` to ground answers on ingested research, notes, and API/vendor docs — prefer it over guessing when the answer may live in ingested material. `source_type` is one of `research | notes | api | data`.
- Numeric/tabular data does NOT belong in the KB (semantic search over number grids is weak) — keep that in a database.
- To add a KB to this project: create `.kb.toml` (`collection = "kb_<project>"`, `sources = ["knowledge/sources"]`), add a stdio `knowledge` server to `.mcp.json` (`command = "kb"`, `args = ["mcp"]`), then drop files in `knowledge/sources/` and run `kb ingest`. See `kb --help`.
