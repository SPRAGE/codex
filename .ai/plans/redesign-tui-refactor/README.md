# Redesign TUI Refactor Plan

This folder captures a concrete refactor direction for the redesigned TUI. It is intentionally written as implementation guidance, not product documentation.

## Goals

- Make the sidebar useful as a workspace navigator rather than a dense shortcut list.
- Make the bottom working indicator explain what Codex is currently doing.
- Remove redundant redesign-specific plumbing and keep rendering modules small.
- Preserve existing Codex TUI behavior outside `--redesign-tui`.
- Keep each implementation stage reviewable and under the repository change-size guidance.

## Files

- `01-current-state.md` - what exists today and where the clutter comes from.
- `02-sidebar.md` - proposed sidebar information architecture and behavior.
- `03-working-indicator.md` - deterministic activity summary model for the bottom "Working" row.
- `04-architecture-cleanup.md` - code ownership, extraction targets, and redundant state to remove.
- `05-implementation-plan.md` - staged execution plan.
- `06-testing.md` - snapshot, unit, integration, and manual test coverage.

## High-Level Recommendation

Do not start by repainting the chrome. The redesign should first get a small data model for "what the UI should say" and "which sidebar items are actionable", then render that model. Today the renderer reaches directly into `App` and `ChatWidget` through many redesign-specific getters, while the `App` stores redesign state in several parallel collections.

The best first landing stage is a pure extraction:

1. Split `redesign_chrome.rs` into focused renderer modules without changing screenshots.
2. Add a `RedesignUiState` owner for per-thread plan, terminal, sidebar, unread, and activity state.
3. Add an `ActivitySummary` model and feed it from existing event handlers.
4. Redesign the sidebar around chat navigation and active work, moving secondary actions to command palette, overlays, and the footer.

## Non-Goals

- No app-server or protocol changes unless an activity source is unavailable from existing TUI events.
- No changes to `codex-core`.
- No broad rewrite of `ChatWidget`.
- No user-facing docs in `docs/`.
