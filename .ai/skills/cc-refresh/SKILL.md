---
name: cc-refresh
description: >
  Audit and refresh all AI, Codex, and Claude Code context — AI.md, .ai/instructions.md, .ai/context,
  AGENTS.md, CODEX.md, CLAUDE.md, auto-memory files, session history, and rules. Scans for stale content,
  scores project guidance quality (A-F grades),
  proposes targeted fixes, and executes approved changes. Use when the user says "refresh
  claude context", "clean up claude memory", "prune stale context", "context cleanup",
  "refresh knowledge", "audit AI.md", "improve AI.md", "check AI.md quality",
  "audit CLAUDE.md", "improve CLAUDE.md", "project memory optimization", or any variation of wanting to
  clean up, audit, or improve Claude Code configuration. Also trigger when context feels
  stale, Claude is referencing things that no longer exist, or CLAUDE.md seems outdated.
  Supports --dry-run for audit-only mode, --guidance-only for AI.md/.ai/adapters focused audit,
  and --sync-only for pulling latest dev-template guidance without auditing.
tools: Read, Glob, Grep, Bash, Edit, Write, Agent
---

# AI Context Refresh

Audit all shared AI, Codex, and Claude Code artifacts for staleness, score guidance quality, report findings,
propose fixes, and execute approved changes. This is the maintenance counterpart to `/cc-setup`.

**This skill modifies files.** It updates `AI.md`, `.ai/`, `.agents/`, `.codex/`, `AGENTS.md`, `CODEX.md`, `CLAUDE.md`, memory files,
and can archive old sessions. All changes require user approval.

## Modes

**Full refresh** (default): Sync latest dev-template-managed assets, then audit all targets — `AI.md`, `.ai/`, `.agents/`, `.codex/`, `AGENTS.md`, `CODEX.md`, `CLAUDE.md`, memory, rules.

**Dry-run** (`--dry-run`): Audit and Report phases only. Skip Propose, Approve, Execute.

**Guidance only** (`--guidance-only`): Run only Agent 1 (Project Guidance Auditor) with full
quality scoring. Skip memory and rules agents. Use when you just want to audit or improve
`AI.md`, `.ai/`, `AGENTS.md`, `CODEX.md`, and `CLAUDE.md` files without touching anything else.

**Sync only** (`--sync-only`): Run only Phase 0 to pull the latest dev-template shared skills,
Codex repo skill links/config/custom agents, Codex compatibility links, Claude skill links, provider adapters, hooks, and missing AI context templates into the current repo.

## Phase 0: Pull Latest dev-template Assets

Unless the user passed `--dry-run`, first run:

```bash
nix run github:SPRAGE/dev-template#sync-skills
```

This is safe for existing repos:
- It updates shared skills and refreshes Codex/Claude provider skill links while migrating existing provider-specific additions into `.ai/skills/`.
- It adds or refreshes managed Codex project config and custom agents under `.codex/`.
- It updates managed adapters (`AGENTS.md`, `CODEX.md`, `CLAUDE.md`) when they are still dev-template adapters.
- It adds missing `AI.md`, `.ai/` templates, hooks, and adapters.
- It preserves customized `AI.md`, customized adapters, and populated `.ai/context/` files.

If the command fails because the repo cannot access GitHub, report the failure and continue
with the audit using local files. In `--sync-only` mode, stop after this phase.

## Phase 1: Audit

Dispatch parallel Agent tool subagents, each auditing one target area:

### Agent 1: Project Guidance Auditor (with Quality Scoring)

**Prompt for agent:**
> Audit ALL shared AI guidance files in the repository for accuracy and quality.
>
> **Step 1: Discovery**
> Find all guidance files:
> ```bash
> find . \( -name "AI.md" -o -name "CLAUDE.md" -o -name "AGENTS.md" -o -name "CODEX.md" -o -path "*/.ai/instructions.md" -o -path "*/.ai/context/*.md" \) 2>/dev/null | head -100
> ```
>
> File types to look for:
> | Type | Location | Purpose |
> |------|----------|---------|
> | Shared top-level guide | `./AI.md` | Provider-neutral project guide for all agents |
> | Shared instructions | `./.ai/instructions.md` | Provider-neutral project instructions |
> | Shared context | `./.ai/context/*.md` | Active context, architecture, conventions, decisions |
> | Codex auto-load adapter | `./AGENTS.md` | Codex-compatible entry point |
> | Codex named adapter | `./CODEX.md` | Named Codex adapter for humans and tools |
> | Claude adapter | `./CLAUDE.md` | Claude Code compatibility entry point |
> | Local overrides | `./.claude.local.md` | Personal settings (gitignored) |
> | Global defaults | `~/.claude/CLAUDE.md` | User-wide defaults |
> | Package-specific | `./packages/*/CLAUDE.md` | Module-level context in monorepos |
>
> **Step 2: Staleness Audit** (for each file)
> 1. Every command listed — does it actually work? Check if the binary/script exists.
> 2. Architecture section — do the listed directories exist? Are there new ones not mentioned?
> 3. Conventions — sample 3-5 source files and check if stated conventions match actual code.
> 4. Stack — does the listed stack match config files (package.json, Cargo.toml, etc.)?
>
> **Step 3: Quality Scoring** (for each file)
> Score against 6 criteria (see `references/quality-criteria.md` for full rubric):
> | Criterion | Max Points | What to Check |
> |-----------|-----------|---------------|
> | Commands/workflows | 20 | Are build/test/deploy commands present and working? |
> | Architecture clarity | 20 | Can Claude understand the codebase structure? |
> | Non-obvious patterns | 15 | Are gotchas and quirks documented? |
> | Conciseness | 15 | No verbose explanations or obvious info? |
> | Currency | 15 | Does it reflect current codebase state? |
> | Actionability | 15 | Are instructions executable, not vague? |
>
> Grades: A (90-100), B (70-89), C (50-69), D (30-49), F (0-29)
>
> **Step 4: Red Flags**
> Flag: commands that would fail, references to deleted files, outdated tech versions,
> copy-paste from templates without customization, generic advice, unresolved TODOs,
> duplicate info across AI.md and provider adapters.
>
> Return structured findings per file:
> ```
> ## Project Guidance Audit
>
> ### ./AI.md (Project Root)
> **Score: XX/100 (Grade: X)**
> | Criterion | Score | Notes |
> |-----------|-------|-------|
> | Commands/workflows | X/20 | ... |
> | Architecture clarity | X/20 | ... |
> | Non-obvious patterns | X/15 | ... |
> | Conciseness | X/15 | ... |
> | Currency | X/15 | ... |
> | Actionability | X/15 | ... |
>
> #### Stale Items
> - [item]: [what's wrong] (severity: HIGH/MEDIUM/LOW)
> #### Missing Items
> - [item]: [what should be added]
> #### Accurate Items
> - [item]: confirmed current
> #### Red Flags
> - [flag]: [details]
> ```

### Agent 2: AI Context Auditor

**Prompt for agent:**
> Audit all files in `.ai/context/` for accuracy:
> 1. `active-context.md` — does "Current Focus" reference real work? Do "Key Files in Play"
>    exist? Are "Blockers" still relevant?
> 2. `decisions.md` — for each decision marked "active", is it still reflected in the code?
>    Any decisions that appear to have been reversed?
> 3. `architecture-snapshot.md` — do listed directories/entry points exist? Any new ones?
> 4. `conventions.md` — sample files and compare to stated conventions.
> 5. Check frontmatter `last_updated` dates — flag anything older than 14 days.
>
> Return structured findings per file with staleness indicators.

### Agent 3: Memory & Session Auditor

**Prompt for agent:**
> Audit Claude Code auto-memory and session history.
>
> 1. Find the project memory directory. Try these patterns:
>    - `~/.claude/projects/*/memory/` where the directory name matches the project path
>    - Look for MEMORY.md index file
>    If not found, report "Auto-memory directory not found" and skip memory audit.
>
> 2. For each memory file found:
>    - Read its content
>    - Check if it references files, functions, or patterns that still exist in the codebase
>    - Flag memories that reference deleted/renamed things
>    - Flag memories older than 30 days
>
> 3. Find session JSONL files in the same project directory:
>    - Count total sessions
>    - Sum total file sizes (use `stat` or `ls -l`, don't read the files)
>    - Note age of oldest session
>    - Recommend archiving sessions older than 7 days
>
> Return structured findings with sizes and dates.

### Agent 4: Rules Auditor

**Prompt for agent:**
> Audit all files in `.claude/rules/` (if the directory exists).
> For each rule file:
> 1. Read its content
> 2. Check if the patterns/conventions it enforces match current code
> 3. Check if it references tools, frameworks, or patterns no longer in use
> 4. Flag rules that duplicate what a linter/formatter already handles
>
> If `.claude/rules/` doesn't exist or is empty, report "No rules found" and skip.
>
> Return findings per rule file.

## Phase 2: Report

Synthesize all agent findings into a structured report. Format:

```
## Context Refresh Report

### Summary
- Targets audited: [N]
- Total findings: [N] (HIGH: [n], MEDIUM: [n], LOW: [n])
- Recommended actions: [N]

### Project Guidance
[Agent 1 findings]

### AI Context
[Agent 2 findings]

### Auto-Memory & Sessions
[Agent 3 findings]

### Rules
[Agent 4 findings]
```

Present this report to the user. In dry-run mode, stop here.

## Phase 3: Propose Actions

For each finding, propose a specific action:

| Action | Description |
|--------|-------------|
| **Update** | Rewrite stale content with current state. Show exact diff. |
| **Archive** | Move old sessions to `.claude/archive/` |
| **Prune** | Remove stale memory file (content logged to stale-log.md first) |
| **Supersede** | Mark reversed decision as `superseded by [new decision]` |
| **Add** | Add missing information discovered during audit |
| **Skip** | Finding is informational, no action needed |

Group proposals by target (AI.md, adapters, AI context store, memory, sessions, rules).

## Phase 4: Approve

Present proposals grouped by category. Ask user:
> "Apply all changes? Or review per category? (all / by-category / per-item)"

- **all**: Apply everything proposed
- **by-category**: Ask approval for each category (AI.md changes, adapter changes, knowledge changes, etc.)
- **per-item**: Ask approval for each individual change

Declined items are logged to stdout: "Skipped: [item] — [reason: user declined]"
Do not re-prompt for declined items.

## Phase 5: Execute

Apply approved changes:
1. For each **Prune**: append the full content being removed to `stale-log.md` with timestamp and reason, then delete
2. For each **Update/Add**: apply the edit using Edit tool
3. For each **Archive**: create `.claude/archive/` if needed, move files there
4. For each **Supersede**: update the decision entry in decisions.md
5. Update `active-context.md` frontmatter with current timestamp
6. Append summary to `stale-log.md`:
   ```
   [YYYY-MM-DD HH:MM] [cc-refresh] Refresh completed: N updates, N prunes, N archives, N superseded
   ```

## Stale-Log Format

Every removal/prune appends an entry:

```markdown
---
### [YYYY-MM-DD HH:MM] [cc-refresh] Pruned: [filename or item]
**Reason:** [why it was pruned]
**Original content:**
[full content of what was removed]
---
```

This ensures nothing is permanently lost. Users can recover incorrectly pruned items.

## Guidance Update Format

For AI.md and adapter changes, use diff-based proposals. See `references/update-guidelines.md` for
detailed guidelines on what to add vs what to avoid. For each proposed change, show:

```markdown
### Update: ./AI.md
**Why:** [reason this helps future sessions]
```diff
+ [addition]
- [removal]
```
```

## User Tips

When presenting the audit report, remind users:
- **`#` key shortcut**: During a Claude session, press `#` to auto-incorporate learnings into Claude's adapter, then move durable shared guidance into AI.md or `.ai/`.
- **Keep it concise**: Dense is better than verbose — every line must earn its place
- **Actionable commands**: All documented commands should be copy-paste ready
- **Use `.claude.local.md`**: For personal preferences not shared with team (add to `.gitignore`)
- **Global defaults**: Put user-wide preferences in `~/.claude/CLAUDE.md`

## References

- `references/quality-criteria.md` — Full scoring rubric for project guidance quality assessment
- `references/templates.md` — Project guidance templates by project type (minimal, comprehensive, monorepo, package)
- `references/update-guidelines.md` — What to add, what to avoid, diff format, validation checklist
