# Decisions

## Use Dev-Template Shared AI Context
- **Date:** 2026-05-08
- **Status:** active
- **Decision:** Use SPRAGE dev-template's `.ai/` shared context, provider
  adapters, custom Codex agents, and shared skill catalog in this Codex fork.
- **Why:** The fork needs reusable AI workflow context while keeping
  provider-specific settings separate.
- **Alternatives considered:** Keep only the original `.codex/skills` layout.
  This would preserve existing behavior but would miss the shared template
  workflow.

## Preserve Original Codex Skills Through Shared Skill Links
- **Date:** 2026-05-08
- **Status:** active
- **Decision:** Restore the original `.codex-bak/skills/` entries into
  `.ai/skills/` and expose them through `.codex/skills` as a symlink.
- **Why:** Dev-template expects `.ai/skills/` to be the source of truth, while
  this fork must retain OpenAI-specific review, issue, PR, and TUI skills.
- **Alternatives considered:** Keep `.codex/skills/` as a real directory. This
  would avoid git path churn but would diverge from the template's shared skill
  model.
