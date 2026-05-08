# Claude Code Adapter

Claude Code loads `CLAUDE.md` automatically, so this file stays as a compatibility adapter.

Read `AI.md` first. Then read the shared context files:

1. `.ai/instructions.md`
2. `.ai/context/active-context.md`
3. `.ai/context/architecture-snapshot.md`
4. `.ai/context/conventions.md`
5. `.ai/context/decisions.md`

Shared skills live in `.ai/skills/`. Claude Code slash-command skills live at `.claude/skills/`; Codex repo-scoped skills live at `.agents/skills/`; Codex compatibility skills live at `.codex/skills/`. Those provider paths are relative symlinks to `.ai/skills/`, so additions through any provider path update the same shared catalog.
