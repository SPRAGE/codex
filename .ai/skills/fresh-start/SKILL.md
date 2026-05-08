---
name: fresh-start
description: >
  Removes AI, Codex, and Claude Code configuration and re-creates it from dev-template
  defaults. Deletes .ai/, .agents/, .claude/, .codex/ managed files, AI.md, AGENTS.md, CODEX.md, CLAUDE.md, .mcp.json,
  .claude.local.md — then restores shared AI context, provider adapters,
  settings, hooks, skills, Codex repo skill links/config/custom agents, compatibility links, and MCP config. Preserves
  auto-memory and .agents/.codex local runtime state. After reset, attempts nix-based skill sync; otherwise tells user
  to run direnv reload. Trigger when user says "fresh start", "reset claude
  code", "start fresh", "nuke claude config", "clean slate", "reset everything",
  "start over", "wipe claude code".
---

# Fresh Start

Removes all shared AI, Codex, and Claude Code configuration from the current project and re-creates it from dev-template defaults.

**Preserves:** auto-memory (`~/.claude/projects/*/memory/`) and `.agents/local/`, `.agents/tmp/`, `.agents/sessions/`, `.agents/logs/`, `.codex/local/`, `.codex/tmp/`, `.codex/sessions/`, `.codex/logs/`
**Deletes:** `.ai/`, `.agents/`, `.claude/`, managed `.codex/` files, `AI.md`, `AGENTS.md`, `CODEX.md`, `CLAUDE.md`, `.mcp.json`, `.claude.local.md`
**Restores:** `.ai/`, `.agents/skills/`, `.codex/config.toml`, `.codex/agents/`, `.codex/skills/`, provider adapters, settings.json, hooks, skills, .mcp.json

## Step 1: Scan

Run these commands to detect what exists:

```bash
echo "=== Fresh Start: scanning current state ==="
[ -d ".ai" ] && echo "  FOUND: .ai/" || echo "  MISSING: .ai/"
[ -d ".agents" ] && echo "  FOUND: .agents/" || echo "  MISSING: .agents/"
[ -d ".claude" ] && echo "  FOUND: .claude/" || echo "  MISSING: .claude/"
[ -d ".codex" ] && echo "  FOUND: .codex/" || echo "  MISSING: .codex/"
[ -f "AI.md" ] && echo "  FOUND: AI.md" || echo "  MISSING: AI.md"
[ -f "AGENTS.md" ] && echo "  FOUND: AGENTS.md" || echo "  MISSING: AGENTS.md"
[ -f "CODEX.md" ] && echo "  FOUND: CODEX.md" || echo "  MISSING: CODEX.md"
[ -f "CLAUDE.md" ] && echo "  FOUND: CLAUDE.md" || echo "  MISSING: CLAUDE.md"
[ -f ".mcp.json" ] && echo "  FOUND: .mcp.json" || echo "  MISSING: .mcp.json"
[ -f ".claude.local.md" ] && echo "  FOUND: .claude.local.md" || echo "  MISSING: .claude.local.md"
[ -d ".claude/skills" ] && echo "  Skills: $(ls -d .claude/skills/*/ 2>/dev/null | wc -l)" || true
[ -d ".agents/skills" ] && echo "  Codex repo skills: $(ls -d .agents/skills/*/ 2>/dev/null | wc -l)" || true
[ -d ".codex/skills" ] && echo "  Codex skills: $(ls -d .codex/skills/*/ 2>/dev/null | wc -l)" || true
[ -d ".claude/hooks" ] && echo "  Hooks: $(ls .claude/hooks/*.sh 2>/dev/null | wc -l)" || true
```

Show the user what will be removed and ask for confirmation:
> "This will delete managed AI/Codex/Claude config and re-create from dev-template defaults. Auto-memory and `.agents/local/`/`.codex/local/` runtime state are preserved. Continue?"

**Do NOT proceed without explicit user confirmation.**

## Step 2: Delete

After confirmation, remove everything:

```bash
preserve_dir=$(mktemp -d)
for local_dir in local tmp sessions logs; do
  [ -e ".agents/$local_dir" ] || continue
  mkdir -p "$preserve_dir/.agents"
  mv ".agents/$local_dir" "$preserve_dir/.agents/$local_dir"
done
for local_dir in local tmp sessions logs; do
  [ -e ".codex/$local_dir" ] || continue
  mkdir -p "$preserve_dir/.codex"
  mv ".codex/$local_dir" "$preserve_dir/.codex/$local_dir"
done
rm -rf .ai .agents .claude .codex
rm -f AI.md AGENTS.md CODEX.md CLAUDE.md .mcp.json .claude.local.md
mkdir -p .agents
if [ -d "$preserve_dir/.agents" ]; then
  mv "$preserve_dir/.agents"/* .agents/ 2>/dev/null || true
fi
mkdir -p .codex
if [ -d "$preserve_dir/.codex" ]; then
  mv "$preserve_dir/.codex"/* .codex/ 2>/dev/null || true
fi
rm -rf "$preserve_dir"
echo "Removed managed AI, Codex, and Claude Code configuration."
```

## Step 3: Re-create config from defaults

Create the directory structure:

```bash
mkdir -p .ai/context .ai/skills .agents/skills .claude/hooks .claude/skills .codex/agents .codex/skills
```

Then use the Write tool to create each file with the content specified below.

### .claude/settings.json

```json
{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true,
    "code-simplifier@claude-plugins-official": true
  },
  "permissions": {
    "allow": [
      "Bash(git:*)",
      "Bash(nix:*)",
      "Bash(npx:*)",
      "Bash(cargo:*)",
      "Bash(uv:*)",
      "mcp__context7__resolve-library-id",
      "mcp__context7__query-docs"
    ],
    "deny": [
      "Edit(//.env)",
      "Edit(//.env.*)",
      "Read(//.env)",
      "Read(//.env.*)",
      "Edit(//.git/**)",
      "Bash(git push --force:*)",
      "Bash(sudo:*)",
      "Bash(curl:*)",
      "Bash(wget:*)",
      "Bash(ssh:*)",
      "Bash(scp:*)",
      "Bash(nix-store --delete:*)"
    ]
  },
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": ".claude/hooks/session-start.sh"
          }
        ]
      }
    ]
  },
  "statusLine": {
    "type": "command",
    "command": ".claude/hooks/statusline.sh"
  }
}
```

### .claude/hooks/session-start.sh

```bash
#!/usr/bin/env bash
# session-start.sh — surfaces active architectural decisions at session start
set -euo pipefail

DECISIONS_FILE=".ai/context/decisions.md"

[ -f "$DECISIONS_FILE" ] || exit 0
grep -q "^## " "$DECISIONS_FILE" 2>/dev/null || exit 0

ACTIVE_DECISIONS=$(grep -B1 -A3 "active" "$DECISIONS_FILE" 2>/dev/null | grep -v "^--$" || true)
if [ -n "$ACTIVE_DECISIONS" ]; then
  echo "=== Active Decisions ==="
  echo "$ACTIVE_DECISIONS"
  echo ""
fi
```

Make executable: `chmod +x .claude/hooks/session-start.sh`

### .claude/hooks/statusline.sh

```bash
#!/usr/bin/env bash
# statusline.sh — renders a persistent status bar in Claude Code
set -euo pipefail

SEGMENTS=()

BRANCH=$(git branch --show-current 2>/dev/null || echo "")
[ -n "$BRANCH" ] && SEGMENTS+=("$BRANCH")

if [ -n "${CLAUDE_CONTEXT_TOKENS_USED:-}" ] && [ -n "${CLAUDE_CONTEXT_WINDOW:-}" ]; then
  USED_K=$((CLAUDE_CONTEXT_TOKENS_USED / 1000))
  WINDOW_K=$((CLAUDE_CONTEXT_WINDOW / 1000))
  SEGMENTS+=("ctx: ${USED_K}k/${WINDOW_K}k")
fi

[ -n "${CLAUDE_SESSION_COST:-}" ] && SEGMENTS+=("\$${CLAUDE_SESSION_COST}")

if [ ${#SEGMENTS[@]} -gt 0 ]; then
  IFS=" | "
  echo "${SEGMENTS[*]}"
fi
```

Make executable: `chmod +x .claude/hooks/statusline.sh`

### .ai/instructions.md

```markdown
# AI Instructions

## Project

PROJECTNAME - TODO: replace with one-line description.

## Read Order

1. `.ai/instructions.md`
2. `.ai/context/active-context.md`
3. `.ai/context/architecture-snapshot.md`
4. `.ai/context/conventions.md`
5. `.ai/context/decisions.md`

## Provider Adapters

- `AI.md` is the shared top-level guide for all agents.
- `AGENTS.md` is the Codex-compatible entry point.
- `CODEX.md` is a named Codex adapter alias for humans and tools.
- `CLAUDE.md` is the Claude Code compatibility entry point.
- `.agents/skills/` is the official Codex repo-scoped skill link.
- `.claude/` contains Claude Code settings, hooks, and skills.
- `.codex/` contains Codex project config, custom agents, the compatibility skill link, and local Codex runtime state.

## Commands

- `nix develop` - enter dev shell.
- `nix run github:SPRAGE/dev-template#sync-skills` - pull latest shared skills, managed adapters, provider skill links, Codex config/custom agents, hooks, and AI context templates.
- `nix run github:SPRAGE/dev-template#ai-doctor` - validate AI context files, shared skills, provider skill links, Codex config/custom agents, and provider-specific settings layout.

## Rules

- Treat `.ai/context/` as the shared project context source of truth.
- Keep provider-specific runtime settings out of `.ai/`.
- Do not edit permission or settings files unless the user explicitly asks for that settings change.
```

### .ai/context/active-context.md

```markdown
# Active Context

<!-- TEMPLATE: Replace this with current in-progress project context. -->

## Current Focus

- TODO: What is being worked on right now?

## Recent Decisions

- TODO: Short-lived decisions that may later move to decisions.md.

## Key Files in Play

- TODO: Files, modules, or docs that matter to the current work.

## Blockers / Questions

- TODO: Open questions, blockers, or assumptions to verify.

## Next Steps

- TODO: Immediate next actions.
```

### .ai/context/architecture-snapshot.md

```markdown
# Architecture Snapshot

<!-- TEMPLATE: Capture the current shape of the system. Refresh with /cc-refresh. -->

## Stack

- TODO: Languages, frameworks, tools, and runtime requirements.

## Project Structure

- TODO: Important directories and what they contain.

## Entry Points

- TODO: Main binaries, services, commands, pages, APIs, or jobs.

## Data Flow

- TODO: Key data paths, external systems, and persistence boundaries.

## Deployment / Runtime

- TODO: How the project runs locally and in production.

## Known Gaps

- TODO: Architectural unknowns or areas that need verification.
```

### .ai/context/conventions.md

```markdown
# Conventions

<!-- TEMPLATE: Record coding, testing, review, and operational conventions. -->

## Code Style

- TODO: Naming, formatting, typing, and module organization conventions.

## Testing

- TODO: Test framework, test layout, required checks, and coverage expectations.

## Commands

- TODO: Build, test, lint, format, run, and deploy commands.

## Git / Review

- TODO: Branch, commit, pull request, and review expectations.

## Security / Secrets

- TODO: Secret handling, environment variables, and security review requirements.
```

### .ai/context/decisions.md

```markdown
# Decisions

<!-- TEMPLATE: Add decisions that are still in effect below.

Entry format:

## [Decision Title]
- **Date:** YYYY-MM-DD
- **Status:** active | superseded by [other decision]
- **Decision:** [What was decided]
- **Why:** [Reasoning]
- **Alternatives considered:** [What else was on the table]
-->
```

### .ai/context/stale-log.md

```markdown
# Stale Log

<!-- Entries appended by /cc-refresh or sync tooling before stale context is removed. -->

## Log

- No stale context recorded yet.
```

### .ai/context/.gitignore

```
# Hook state files — not tracked in git
.session-loaded
```

### .mcp.json

```json
{
  "mcpServers": {
    "context7": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp@latest"]
    }
  }
}
```

### AI.md

```markdown
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
```

### CLAUDE.md

```markdown
# Claude Code Adapter

Claude Code loads `CLAUDE.md` automatically, so this file stays as a compatibility adapter.

Read `AI.md` first. Then read the shared context files:

1. `.ai/instructions.md`
2. `.ai/context/active-context.md`
3. `.ai/context/architecture-snapshot.md`
4. `.ai/context/conventions.md`
5. `.ai/context/decisions.md`

Shared skills live in `.ai/skills/`. Claude Code slash-command skills live at `.claude/skills/`; Codex repo-scoped skills live at `.agents/skills/`; Codex compatibility skills live at `.codex/skills/`. Those provider paths are relative symlinks to `.ai/skills/`, so additions through any provider path update the same shared catalog.
```

### AGENTS.md

```markdown
# Codex Adapter

Codex-compatible agents load `AGENTS.md` automatically, so this file stays as the Codex compatibility adapter.

Read `AI.md` first. Then read the shared context files:

1. `.ai/instructions.md`
2. `.ai/context/active-context.md`
3. `.ai/context/architecture-snapshot.md`
4. `.ai/context/conventions.md`
5. `.ai/context/decisions.md`

Shared skills live in `.ai/skills/`. Codex discovers repo-scoped skills from `.agents/skills/`, a relative symlink to `.ai/skills/`; `.codex/skills/` is a compatibility link. When the user names a skill, with or without a leading slash, read `.ai/skills/<skill-name>/SKILL.md`; the provider paths resolve to the same shared files.
```

### CODEX.md

```markdown
# Codex Adapter

`CODEX.md` is a named Codex adapter for humans and tools. Codex-compatible agents usually auto-load `AGENTS.md`, which contains the same compatibility guidance.

Read `AI.md` first. Then read the shared context files:

1. `.ai/instructions.md`
2. `.ai/context/active-context.md`
3. `.ai/context/architecture-snapshot.md`
4. `.ai/context/conventions.md`
5. `.ai/context/decisions.md`

Shared skills live in `.ai/skills/`. Codex discovers repo-scoped skills from `.agents/skills/`, a relative symlink to `.ai/skills/`; `.codex/skills/` is a compatibility link. When the user names a skill, with or without a leading slash, read `.ai/skills/<skill-name>/SKILL.md`; the provider paths resolve to the same shared files.
```

## Step 4: Attempt nix skill sync

Try to sync skills from dev-template via the project's flake. This is a bonus step — if it fails, the user syncs manually.

```bash
# Attempt to resolve dev-template path from flake and sync skills
if [ -f "flake.nix" ] && grep -q "dev-template" flake.nix 2>/dev/null; then
  echo "Attempting nix-based skill sync..."
  # Build the dev-template input path
  DEV_TEMPLATE_PATH=$(nix eval --raw .#devShells.$(nix eval --raw --impure --expr 'builtins.currentSystem').default.inputDerivation.passthru.dev-template 2>/dev/null || true)

  if [ -z "$DEV_TEMPLATE_PATH" ]; then
    # Fallback: try to find it via flake metadata
    DEV_TEMPLATE_PATH=$(nix flake metadata --json 2>/dev/null | grep -o '"path":"[^"]*dev-template[^"]*"' | head -1 | sed 's/"path":"//;s/"//' || true)
  fi

  if [ -z "$DEV_TEMPLATE_PATH" ]; then
    # Fallback: resolve from nix store via flake lock
    DEV_TEMPLATE_PATH=$(nix build .#devShells.$(nix eval --raw --impure --expr 'builtins.currentSystem').default --dry-run --json 2>/dev/null | grep -o '/nix/store/[^"]*dev-template[^"]*' | head -1 || true)
  fi

  if [ -n "$DEV_TEMPLATE_PATH" ] && [ -d "$DEV_TEMPLATE_PATH/template/.ai/skills" ]; then
    SKILLS_SRC="$DEV_TEMPLATE_PATH/template/.ai/skills"
    for skill_dir in "$SKILLS_SRC"/*/; do
      [ -d "$skill_dir" ] || continue
      skill_name=$(basename "$skill_dir")
      mkdir -p ".ai/skills"
      cp -rL "$skill_dir" ".ai/skills/$skill_name"
      chmod -R u+w ".ai/skills/$skill_name"
      mkdir -p ".agents/skills"
      cp -rL "$skill_dir" ".agents/skills/$skill_name"
      chmod -R u+w ".agents/skills/$skill_name"
      cp -rL "$skill_dir" ".claude/skills/$skill_name"
      chmod -R u+w ".claude/skills/$skill_name"
      mkdir -p ".codex/skills"
      cp -rL "$skill_dir" ".codex/skills/$skill_name"
      chmod -R u+w ".codex/skills/$skill_name"
    done
    echo "Synced $(ls -d .ai/skills/*/ 2>/dev/null | wc -l) skills from dev-template."
  else
    echo "Could not resolve dev-template path from nix. Skills will sync on next shell entry."
  fi
else
  echo "No flake.nix with dev-template input found. Skills will sync on next shell entry."
fi
```

## Step 5: Report and next steps

After everything is done, tell the user:

> **Fresh start complete.**
>
> Restored:
> - `.ai/` (shared instructions and context)
> - `.agents/skills/` (Codex repo-scoped skill link)
> - `AI.md` (shared top-level guide)
> - `AGENTS.md` (Codex-compatible adapter)
> - `CODEX.md` (Codex named adapter)
> - `CLAUDE.md` (Claude Code compatibility adapter)
> - `.codex/config.toml` and `.codex/agents/` (Codex project config and custom agents)
> - `.codex/skills/` (Codex compatibility skill link)
> - `.claude/settings.json` (plugins, permissions, hooks)
> - `.claude/hooks/` (session-start, statusline)
> - `.mcp.json` (context7)
>
> [If nix sync succeeded]: Skills synced: [count] skills from dev-template.
> [If nix sync failed]: Run `direnv reload` or `nix develop` to sync skills from dev-template.
>
> **Restart Claude Code to pick up the new settings.**
> Then run `/cc-setup` to scan your codebase and generate tailored config.
