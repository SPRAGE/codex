# Claude Code Adapter

Claude Code auto-loads `CLAUDE.md`. Read `AI.md`, then `.ai/instructions.md` — it owns the tiered read-order, response style, skill locations, and rules.

Shared skills live in `.ai/skills/`; `.claude/skills/` is a relative symlink to it (Claude Code slash-command path).
