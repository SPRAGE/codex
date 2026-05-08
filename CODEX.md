# Codex Adapter

`CODEX.md` is a named Codex adapter for humans and tools. Codex-compatible agents usually auto-load `AGENTS.md`, which contains the same compatibility guidance.

Read `AI.md` first. Then read the shared context files:

1. `.ai/instructions.md`
2. `.ai/context/active-context.md`
3. `.ai/context/architecture-snapshot.md`
4. `.ai/context/conventions.md`
5. `.ai/context/decisions.md`

Shared skills live in `.ai/skills/`. Codex discovers repo-scoped skills from `.agents/skills/`, a relative symlink to `.ai/skills/`; `.codex/skills/` is a compatibility link. When the user names a skill, with or without a leading slash, read `.ai/skills/<skill-name>/SKILL.md`; the provider paths resolve to the same shared files.
