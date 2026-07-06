---
name: cc-setup
description: >
  Set up, onboard, or optimize a project's AI-agent configuration for any runtime
  (Claude Code, Codex, or both). Works in three modes: (1) Greenfield — generates AI.md,
  .ai/ context, provider adapters (AGENTS.md/CODEX.md/CLAUDE.md), .mcp.json, and rules from a
  brainstorm brief or project description. (2) Brownfield — scans an existing codebase to
  generate that config. (3) Recommend — analyzes an existing setup and recommends automations
  (hooks, skills, MCP servers, subagents, plugins). Use when the user says "set up Claude Code",
  "set up Codex", "configure AI agents", "generate AGENTS.md", "generate a CLAUDE.md", "onboard
  codex", "bootstrap claude code", "add AI config to this project", "what MCP servers should I
  use", "recommend automations", "optimize my agent setup", "improve agent workflows", "prepare
  my project for Claude Code or Codex", or any variation of configuring, onboarding, or
  optimizing AI agents for a project.
tools: Read, Glob, Grep, Bash, Edit, Write, Agent
---

# Project AI Setup

Set up a project's AI-agent configuration for any runtime (Claude Code, Codex, or both) —
whether starting fresh, onboarding an existing codebase, or optimizing an existing setup.
Outputs are provider-neutral (`.ai/`, `AI.md`) plus thin per-provider adapters; runtime-only
artifacts (`.claude/` hooks/rules, `.codex/` config/agents) are written only for the runtimes in use.

**This skill writes files.** It generates or updates `AI.md`, `.ai/instructions.md`,
`.ai/context/`, `.codex/`, `AGENTS.md`, `CODEX.md`, `CLAUDE.md`, `.mcp.json`, and `.claude/rules/`.
All outputs require user approval.

## Mode Detection

Auto-detect the appropriate mode from context:

| Signal | Mode |
|--------|------|
| Brainstorm brief in conversation, or user describes a new project | **Greenfield** |
| Existing code but no/minimal AI config | **Brownfield** |
| Existing AI config, user wants improvements | **Recommend** |
| User explicitly says "onboard" or "bootstrap" | **Brownfield** |
| User explicitly says "recommend" or "optimize" | **Recommend** |

If ambiguous, ask: "Is this a new project, an existing project that needs Claude Code
setup, or an existing setup you want to optimize?"

---

## Greenfield Mode

The user has a project plan (ideally from a brainstorm session) or describes a new project.

### Quick Intake (if no brainstorm brief in conversation)

Ask at most 5 questions:
1. "Describe the project in a sentence or two — what does it do and what's the stack?"
2. "What languages and key frameworks are you using?"
3. "Where does the code live — GitHub, GitLab, local only?"
4. "What's your package manager / build tool?"
5. "Is this solo or team?"

Make reasonable assumptions and state them. Don't over-interview.

### Generate

Read `references/mcp-catalog.md` before generating. Produce all deliverables (see
Shared Output Sections below).

---

## Brownfield Mode

The user has an existing repository with code but no (or minimal) Claude Code setup.

### Prerequisites

- If `.ai/context/` exists with populated files → warn user this repo appears
  already configured, suggest `/cc-refresh` instead. Proceed only if user confirms.
- If `.ai/context/` doesn't exist → create it with empty templates, then
  proceed to scanning.

### Scanning Phase

Dispatch 4 parallel Agent tool subagents:

#### Agent 1: Language & Framework Detector
> Scan this repository to identify all programming languages and frameworks in use.
> Check indicator files: `Cargo.toml` (Rust), `package.json` (Node.js), `pyproject.toml`
> (Python), `go.mod` (Go), `flake.nix` (Nix), `Gemfile` (Ruby), `pom.xml` (Java),
> `.csproj` (.NET), `Makefile`/`CMakeLists.txt` (C/C++), `mix.exs` (Elixir).
> Also check for framework indicators: `next.config.*`, `vite.config.*`, `svelte.config.*`,
> `tailwind.config.*`, `tsconfig.json`, `.eslintrc*`, `prettier.config.*`.
> Return: Languages, Frameworks, Package Managers, Key Config Files.

#### Agent 2: Structure Mapper
> Map the directory structure. Focus on: top-level directories and purpose, entry points,
> test directories and naming patterns, config file locations, build output directories,
> monorepo structure if present. Return: Directory Structure, Entry Points, Test Locations,
> Config Locations, Build Artifacts.

#### Agent 3: Convention Detector
> Analyze coding conventions by sampling 5-10 representative source files. Detect:
> naming patterns (functions, variables, types, files), error handling approach, testing
> patterns (framework, naming, assertions), import style, code style (indentation, line
> length), documentation patterns. Return findings with evidence.

#### Agent 4: Command Discoverer
> Discover all build, test, lint, format, and run commands. Check: Makefile, package.json
> scripts, Cargo.toml, pyproject.toml, flake.nix, justfile, Taskfile.yml, deno.json,
> .github/workflows/*.yml, docker-compose.yml. Return exact copy-paste-ready commands.

### Generate

After all agents return, synthesize findings into deliverables (see Shared Output Sections).

### AI Context Population (Brownfield only)

If `.ai/context/` exists:
- `architecture-snapshot.md` — fill from Agents 1 + 2
- `conventions.md` — fill from Agent 3
- `active-context.md` — set Current Focus to "Project just onboarded via /cc-setup"
- `decisions.md` — leave empty
- `stale-log.md` — append: `[YYYY-MM-DD HH:MM] [cc-setup] AI context store initialized`

---

## Recommend Mode

The user has an existing AI-agent configuration and wants to optimize it.

**This mode is read-only.** It analyzes the codebase and outputs recommendations. It
does NOT create or modify files unless the user asks.

### Analysis Phase

Gather project context:
- Read AI.md, AGENTS.md, CODEX.md, CLAUDE.md, .mcp.json, .claude/rules/ (if they exist)
- Detect project type, frameworks, and dependencies from config files
- Check for existing hooks, subagents, skills

### Generate Recommendations

Recommend 1-2 of each type (don't overwhelm). Skip categories that aren't relevant.
If user asks for a specific type, provide 3-5 recommendations for that type.

See reference files for detailed patterns:
- `references/mcp-catalog.md` — MCP server recommendations with install commands
- `references/hooks-patterns.md` — Hook configurations
- `references/skills-reference.md` — Skill recommendations
- `references/plugins-reference.md` — Plugin recommendations
- `references/subagent-templates.md` — Subagent templates

| Type | Best For |
|------|----------|
| **MCP Servers** | External tool integrations (databases, APIs, browsers, docs) |
| **Hooks** | Automatic actions on tool events (format on save, lint, block edits) |
| **Skills** | Packaged expertise, workflows, repeatable tasks |
| **Plugins** | Collections of skills that can be installed |
| **Subagents** | Specialized reviewers/analyzers that run in parallel |

End with: "Want more? Ask for additional recommendations for any category."

---

## Shared Output Sections

Greenfield and Brownfield modes produce the same deliverables:

### 1. AI.md and .ai/instructions.md

These are the provider-neutral base files that Claude Code, Codex, and future agents should read.

- **Under 100 lines.** Aim for 40-80.
- **Exact commands.** Not "run the tests" but the actual command string.
- **Specific conventions.** Not "write clean code" but "use snake_case for functions."
- **Architecture map.** Brief directory listing showing where key things live.
- **No provider-specific settings.** Keep Claude permissions, hooks, and MCP wiring out of this file.

Use `AI.md` as the shared top-level guide. Use `.ai/instructions.md` for durable provider-neutral project instructions and read order.

Structure:
```
# AI Instructions

## Project
[Project name and one-line description]

## Read Order
[.ai files to read before substantial work]

## Stack
[Languages, frameworks, key libraries — short list]

## Commands
[Build, test, lint, format, run — exact strings]

## Architecture
[Key directories and what lives in each]

## Conventions
[Naming, patterns, error handling approach]

## Important Notes
[Gotchas, things easy to get wrong, non-obvious decisions]
```

### 2. AGENTS.md, CODEX.md, and CLAUDE.md Adapters

Keep these as provider adapters that point back to `.ai/`.

- `AGENTS.md` should tell Codex-compatible agents to read `AI.md`, `.ai/instructions.md`, and `.ai/context/*`.
- `CODEX.md` should be a named Codex adapter alias with the same guidance as `AGENTS.md` for humans and tools.
- `CLAUDE.md` should be a short Claude Code compatibility adapter that tells Claude Code to read `AI.md` and the same `.ai/` files, then use Claude-specific skills/settings as needed.
- `.agents/skills/` should link to `.ai/skills/` for official Codex repo-scoped skill discovery.
- `.codex/skills/` should link to `.ai/skills/` for compatibility when managed by dev-template; local runtime state belongs under `.codex/local/` or `.codex/tmp/`.
- `.codex/config.toml` and `.codex/agents/` hold Codex project-scoped config and custom subagents after the project is trusted.
- Avoid duplicating large project context in both adapters; duplicate only short bootstrapping instructions.

### 3. .ai/context/ Files

Populate or update:

- `active-context.md` — current focus, files in play, blockers, next steps.
- `architecture-snapshot.md` — stack, structure, entry points, runtime map.
- `conventions.md` — coding, testing, review, command, and security conventions.
- `decisions.md` — active decisions only.
- `stale-log.md` — stale context audit trail.

### 4. AI.md Content Quality

- **Under 100 lines.** Aim for 40-80. Every line earns its place.
- **No fluff.** No "Welcome to the project" preamble. Start with the one-liner.
- **Exact commands.** Not "run the tests" but the actual command string.
- **Specific conventions.** Not "write clean code" but "use snake_case for functions."
- **Architecture map.** Brief directory listing showing where key things live.

Structure:
```
# [Project Name]
[One-line description]

## Stack
[Languages, frameworks, key libraries — short list]

## Commands
[Build, test, lint, format, run — exact strings]

## Architecture
[Key directories and what lives in each]

## Conventions
[Naming, patterns, error handling approach]

## Important Notes
[Gotchas, things easy to get wrong, non-obvious decisions]
```

Adapt sections to the project. A CLI tool doesn't need a full architecture section.
If the same details belong in `.ai/instructions.md`, prefer putting them there and keeping
`CLAUDE.md` as a short compatibility adapter.

### 5. .mcp.json

Only include servers genuinely useful for this project (see `references/mcp-catalog.md`).
Start with context7 MCP (always included). Add others based on detected stack.

Use `${ENV_VAR}` expansion for all secrets. Never hardcode credentials. Note which env
vars the user needs to set.

### 6. .claude/rules/ Files

Generate 1-3 focused rule files. Each file 10-25 lines, focused on one concern (testing,
API conventions, styling). Don't create rules for things linters/formatters already handle.

### 7. Automation Recommendations (Final Phase — Every Mode)

After generating config (greenfield/brownfield) or as the main output (recommend mode),
suggest relevant automations:
- MCP servers based on detected stack
- Hooks for repetitive post-edit actions
- Skills for frequently repeated workflows
- Subagents for specialized review needs

### 8. Workflow Recommendations

A short section (advice in your response, not a file) covering:
- **Session workflow**: How to use the agent effectively for this project type (note runtime specifics — Claude Code slash-commands/hooks vs Codex custom agents)
- **Useful commands/agents**: Which built-in commands (Claude Code) or custom agents (Codex) matter most
- **Multi-agent patterns**: When to parallelize or isolate work on the runtime in use
- **What NOT to delegate to the agent**: Things better handled by other tools

## Stack-Specific Adaptations

### Rust Projects
- Emphasize `cargo` commands (build, test, clippy, fmt)
- Note Rust Analyzer integration
- If using a Nix flake, include `nix build` / `nix develop` commands
- Rule file for error handling patterns (Result/Option, thiserror vs anyhow)

### Python Projects
- Include package manager commands (uv, pip, poetry)
- Note virtual environment activation
- Type hinting conventions in rules
- pytest conventions if applicable

### Nix / NixOS Config Projects
- Module structure as the architecture section
- `nixos-rebuild` commands with exact flags
- Flake-specific commands (nix flake check, nix flake update)
- Rule file for Nix idioms (mkIf, lib patterns)

### Web Apps (React, Svelte, etc.)
- Dev server, build, and preview commands
- Component file structure conventions
- API route patterns in rules
- Playwright MCP if E2E testing is in scope

### Data / Analytics Projects
- Database connection details (sanitized) in AI.md
- Query conventions in rules
- Data directory structure in architecture section

## Presentation

Present deliverables in this order:
1. **Quick summary**: "Here's what I'm setting up for [project] and why."
2. **AI.md, .ai/instructions.md, and .ai/context/**: Full content or focused diffs.
3. **AGENTS.md, CODEX.md, and CLAUDE.md**: Adapter content or focused diffs.
4. **.mcp.json**: Config + env vars to set.
5. **Rules files**: Each with one-line explanation.
6. **Automation recommendations**: Hooks, skills, MCP servers, subagents.
7. **Workflow tips**: Advice section.

Ask for approval before writing files.

## Important Constraints

- Never include actual secrets, tokens, or passwords in any generated file.
- Always use `${ENV_VAR}` expansion for sensitive values.
- If using a Nix flake, note that users may prefer to wrap npx-based MCP servers in a devShell.
- AI.md must be useful on day one — no placeholder sections.
- If you can't find something (e.g., test command), say so with `# TODO: verify` rather than guessing.
- Don't guess at commands — if you can't find a test command, say "no test command found."
