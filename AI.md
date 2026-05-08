# AI.md

## Project

Codex fork - a personal fork of the open source OpenAI Codex CLI, optimized for
Shaun's local development workflow while staying close to the upstream project.

## Getting Started

1. Read `AGENTS.md` first for the repository-specific Rust and Codex rules.
2. Read `.ai/instructions.md` and the files under `.ai/context/` for shared
   agent context.
3. Use `nix develop` when you need the repository development environment.

## Stack

- Rust workspace under `codex-rs/` for the CLI, TUI, app server, protocol,
  sandboxing, MCP, and supporting crates.
- Node/pnpm package metadata for the npm-distributed CLI wrapper and
  repo-wide formatting.
- Bazel, Nix, and `just` for local and CI-oriented build/test workflows.
- SPRAGE dev-template shared AI context, skills, provider adapters, and Codex
  project agents.

## Commands

- `nix develop` - enter dev shell
- `just fmt` - format Rust code from the repository root
- `just fix -p <crate>` - run scoped Clippy fixes from `codex-rs`
- `cargo test -p <crate>` - run a crate-specific Rust test from `codex-rs`
- `just test` - run the Rust workspace tests through nextest when available
- `npm run format` - check repository markdown/json/workflow formatting
- `nix run github:SPRAGE/dev-template#sync-skills` - pull latest shared skills, managed adapters, provider skill links, Codex config/custom agents, hooks, and AI context templates
- `nix run github:SPRAGE/dev-template#ai-doctor` - validate AI context files, shared skills, provider skill links, Codex config/custom agents, and hooks

## Architecture

- `codex-rs/cli` builds the `codex` binary.
- `codex-rs/tui` owns the terminal user interface.
- `codex-rs/core` contains central orchestration and should be kept from
  growing unnecessarily.
- `codex-rs/protocol`, `app-server-*`, and `codex-rs/mcp-*` own API and MCP
  boundaries.
- `.ai/skills/` is the shared skill source; `.agents/skills`,
  `.claude/skills`, and `.codex/skills` point to it.

## Conventions

- Follow `AGENTS.md` for Rust, TUI, testing, snapshot, and app-server API
  guidance.
- Keep changes scoped and prefer existing crate/module boundaries.
- Preserve OpenAI-specific repo skills when refreshing dev-template assets.
- Treat `.codex-bak/` as a local safety backup, not a source directory to
  commit.

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

## Shared AI Context

Project context is tracked in `.ai/` so Claude Code, Codex, and future agents read the same base files:

- `instructions.md` - provider-neutral project instructions
- `active-context.md` - current work and next steps
- `architecture-snapshot.md` - stack, structure, and runtime map
- `conventions.md` - coding, testing, and review conventions
- `decisions.md` - active architectural decisions
- `stale-log.md` - audit trail for removed or superseded context

## Shared Skills

Shared skill sources live in `.ai/skills/`. Codex discovers repo-scoped skills from `.agents/skills/`; Claude Code uses `.claude/skills/` for slash commands; `.codex/skills/` remains a compatibility path. These provider paths are relative symlinks to `.ai/skills/`, so additions through any provider path update the same shared catalog. Agents should read `.ai/skills/<skill-name>/SKILL.md` when a provider-neutral source is needed.

## Provider Adapters

`AI.md` is the shared top-level guide. `AGENTS.md` is the Codex-compatible auto-load adapter. `CODEX.md` is a named Codex adapter alias. `CLAUDE.md` is the Claude Code compatibility adapter. Provider-specific settings remain in provider-specific folders such as `.agents/`, `.claude/`, and `.codex/`.
