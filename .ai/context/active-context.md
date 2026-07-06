# Active Context

## Current Focus

- Planning a scoped refactor of the redesigned TUI to improve sidebar usage,
  make the bottom working indicator show deterministic activity summaries, and
  remove redundant redesign plumbing.
- Previous dev-template integration work is complete enough that its remaining
  follow-up is validation rather than active implementation.

## Recent Decisions

- Added `.ai/plans/redesign-tui-refactor/` as the working plan folder for the
  redesigned TUI cleanup.
- Keep the first implementation stage behavior-preserving: extract
  `redesign_chrome.rs` into smaller renderer modules before changing UI output.
- Treat the working indicator problem as a semantic activity-summary issue, not
  a render-only issue, because the status widget already supports rich display.
- Created `.codex-bak/` as a local backup of the original `.codex` directory
  before template onboarding.
- Accepted the dev-template layout where `.ai/skills/` is the shared source and
  `.agents/skills`, `.claude/skills`, and `.codex/skills` link back to it.
- Restored the original `.codex-bak/skills/` entries into `.ai/skills/`, making
  them visible through `.codex/skills`.

## Key Files in Play

- `.ai/plans/redesign-tui-refactor/*.md`
- `codex-rs/tui/src/redesign_chrome.rs`
- `codex-rs/tui/src/redesign_chrome/`
- `codex-rs/tui/src/app/input.rs`
- `codex-rs/tui/src/app/thread_routing.rs`
- `codex-rs/tui/src/status_indicator_widget.rs`
- `codex-rs/tui/src/bottom_pane/mod.rs`
- `codex-rs/tui/src/chatwidget.rs`
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

- For redesign TUI implementation, start with
  `.ai/plans/redesign-tui-refactor/05-implementation-plan.md`.
- Prefer the behavior-preserving renderer extraction before sidebar or status
  copy changes so snapshot churn remains reviewable.
- Review the resulting git status.
- Keep `.codex-bak/` local unless a fresh backup is needed.
- Run dev-template validation with `nix run github:SPRAGE/dev-template#ai-doctor`
  when remote Nix execution is acceptable.
