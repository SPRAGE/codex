# AI Instructions

## Project

Codex fork - a personal fork of the open source OpenAI Codex CLI, optimized for
Shaun's local development workflow while staying close to upstream.

## Read Order

1. `AI.md`
2. `.ai/instructions.md`
3. `.ai/context/active-context.md`
4. `.ai/context/architecture-snapshot.md`
5. `.ai/context/conventions.md`
6. `.ai/context/decisions.md`

## Provider Adapters

- `AI.md` is the shared top-level guide for all agents.
- `AGENTS.md` is the Codex-compatible entry point.
- `CODEX.md` is a named Codex adapter alias for humans and tools.
- `CLAUDE.md` is the Claude Code compatibility entry point.
- `.ai/skills/` contains shared skills for all agents.
- `.agents/skills/` is the official Codex repo-scoped skill link.
- `.claude/` contains Claude Code settings, hooks, and the Claude skill link.
- `.codex/` contains Codex project config, custom agents, the compatibility skill link, and local Codex runtime state.

## Commands

- `nix develop` - enter dev shell.
- `just fmt` - format Rust code.
- `just fix -p <crate>` - run scoped Rust lint fixes.
- `cargo test -p <crate>` - run a crate-specific Rust test from `codex-rs`.
- `nix run github:SPRAGE/dev-template#sync-skills` - pull latest skills, Codex repo skills/config/custom agents, provider links, hooks, and AI context templates.
- `nix run github:SPRAGE/dev-template#ai-doctor` - validate AI context files, shared skills, provider links, Codex config/custom agents, and provider-specific settings layout.

## Rules

- Treat `.ai/context/` as the shared project context source of truth.
- Treat `.ai/skills/` as the shared skill source of truth.
- Keep `.agents/skills/`, `.claude/skills/`, and `.codex/skills/` linked to `.ai/skills/`, preserving local provider-specific skills.
- Preserve the OpenAI-specific skills restored from `.codex-bak/skills/` when refreshing dev-template assets.
- Keep provider-specific runtime settings out of `.ai/`.
- Do not edit permission or settings files unless the user explicitly asks for that settings change.
