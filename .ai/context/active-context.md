# Active Context

## Current Focus

- Integrating `git@github.com:SPRAGE/dev-template.git` into this Codex fork
  without losing the existing OpenAI-specific `.codex` skills.

## Recent Decisions

- Created `.codex-bak/` as a local backup of the original `.codex` directory
  before template onboarding.
- Accepted the dev-template layout where `.ai/skills/` is the shared source and
  `.agents/skills`, `.claude/skills`, and `.codex/skills` link back to it.
- Restored the original `.codex-bak/skills/` entries into `.ai/skills/`, making
  them visible through `.codex/skills`.

## Key Files in Play

- `AGENTS.md`
- `AI.md`
- `.ai/instructions.md`
- `.ai/context/*.md`
- `.ai/skills/`
- `.agents/skills`
- `.claude/skills`
- `.codex/skills`
- `.codex-bak/`

## Blockers / Questions

- None currently.

## Next Steps

- Review the resulting git status.
- Keep `.codex-bak/` local unless a fresh backup is needed.
- Run dev-template validation with `nix run github:SPRAGE/dev-template#ai-doctor`
  when remote Nix execution is acceptable.
