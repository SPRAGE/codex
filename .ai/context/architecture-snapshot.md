# Architecture Snapshot

## Stack

- Rust workspace under `codex-rs/`.
- Node/pnpm metadata for npm package distribution and repo-wide formatting.
- Bazel, Nix, and `just` for development and CI workflows.
- SPRAGE dev-template for shared AI context, shared skills, provider adapters,
  and Codex custom agents.

## Project Structure

- `codex-rs/` - Rust crates for Codex CLI, TUI, app server, protocols, MCP,
  sandboxing, auth, state, and supporting libraries.
- `codex-cli/` - npm-facing CLI package assets.
- `sdk/` - SDK packages and runtime support.
- `.ai/` - provider-neutral AI instructions, context, and shared skills.
- `.agents/` - Codex repo-scoped skill view.
- `.codex/` - Codex project config, custom agents, environment config, and
  compatibility skill view.
- `.claude/` - Claude Code settings, hooks, and shared skill view.

## Entry Points

- `codex-rs/cli` builds the `codex` binary.
- `codex-rs/tui` owns the terminal UI.
- `codex-rs/app-server` exposes app-server behavior.
- `justfile` defines common local commands from the repository root.

## Data Flow

- CLI and TUI operations route through Rust crates under `codex-rs/`.
- MCP and app-server API boundaries are handled in the dedicated protocol,
  app-server, and MCP crates.
- Shared AI skill discovery resolves through `.ai/skills/`, with provider
  paths linking back to the shared catalog.

## Deployment / Runtime

- Local development uses `nix develop`, `just`, Cargo, Bazel, and pnpm as
  needed.
- Release and package workflows remain governed by the upstream Codex project
  structure.

## Known Gaps

- Refresh this snapshot after larger fork-specific changes.
