# AI Instructions

## Project

PROJECTNAME - TODO: replace with one-line description.

## Read Order

- Always: `AI.md`, `.ai/instructions.md`.
- Before code or design work: `.ai/context/architecture-snapshot.md`, `.ai/context/conventions.md`.
- When relevant: `.ai/context/decisions.md` (architectural decisions); `.ai/context/active-context.md` (only when it holds real rolling state).

## Response Style

- Answer first; no preamble or filler.
- Cite evidence (file:line, command output) for repo claims; don't assert from memory.
- Prefer concise diffs and summaries; don't restate context already loaded.

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
- `nix run github:SPRAGE/dev-template#sync-skills` - pull latest skills, Codex repo skills/config/custom agents, provider links, hooks, and AI context templates.
- `nix run github:SPRAGE/dev-template#ai-doctor` - validate AI context files, shared skills, provider links, Codex config/custom agents, and provider-specific settings layout.

## Rules

- Treat `.ai/context/` as the shared project context source of truth.
- Treat `.ai/skills/` as the shared skill source of truth.
- Keep `.agents/skills/`, `.claude/skills/`, and `.codex/skills/` linked to `.ai/skills/`, preserving local provider-specific skills.
- Keep provider-specific runtime settings out of `.ai/`.
- Do not edit permission or settings files unless the user explicitly asks for that settings change.
