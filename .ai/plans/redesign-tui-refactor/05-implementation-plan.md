# Implementation Plan

## Phase 0: Planning Docs

Status: complete when this folder lands.

Deliverables:

- Current-state notes.
- Sidebar direction.
- Working indicator model.
- Architecture cleanup plan.
- Test plan.

Verification:

- Markdown files exist and reference current code paths.

## Phase 1: Behavior-Preserving Extraction

Purpose: reduce risk before visual changes.

Changes:

- Split `redesign_chrome.rs` into focused renderer modules.
- Keep output identical.
- Keep current `RedesignChromeContext` shape initially.
- Move plan-window rendering into `plan_window.rs`.
- Move composer rendering into `composer.rs`.
- Move footer rendering into `footer.rs`.
- Move system rail rendering into `system_rail.rs`.
- Move transcript rendering into `transcript.rs`.

Tests:

- `just fmt`
- `just test -p codex-tui`
- No intentional snapshot changes.

Acceptance:

- Existing redesigned snapshots pass unchanged.
- No behavior changes.
- Main file size is materially smaller.

## Phase 2: Redesign UI State Owner

Purpose: remove parallel per-thread redesign maps.

Changes:

- Add `RedesignUiState`.
- Add `RedesignThreadUiState`.
- Move plan window, terminal window, chat display name, activity, and unread state into this owner.
- Replace close/reset/thread-switch cleanup with methods on `RedesignUiState`.
- Keep public app behavior unchanged.

Tests:

- Existing app close/start/thread-routing tests.
- Add focused tests for:
  - removing a thread clears all associated redesign state
  - thread state defaults are stable
  - terminal window selected/expanded state clamps correctly

Acceptance:

- `App` no longer stores multiple redesign thread maps/sets directly.
- Chat close uses one state removal path.

## Phase 3: Activity Summary Model

Purpose: make the working indicator useful.

Changes:

- Add `ActivitySummary`, `ActivityKind`, and `ActivityPriority`.
- Add `ActivitySummaryState` to `ChatWidget` state.
- Feed summaries from:
  - turn start and completion
  - reasoning status
  - assistant streaming
  - plan updates
  - exec command start/wait/completion
  - apply-patch activity
  - MCP tool calls
  - guardian review and approval prompts
  - MCP startup
- Convert activity summary into existing `StatusIndicatorState`.
- Keep `StatusIndicatorWidget` mostly unchanged.

Tests:

- Unit tests for priority/flicker rules.
- Existing status tests updated from generic `Working` to expected summaries where appropriate.
- Snapshot coverage for the redesigned composer status row.

Acceptance:

- The bottom row names the current action for common workflows.
- Waiting/approval states override generic working text.
- No duplicated status phrasing between legacy and redesigned surfaces.

## Phase 4: Sidebar UX Update

Purpose: make sidebar usage clearer and reduce clutter.

Changes:

- Add sidebar render model.
- Add optional chat activity summary line.
- Reduce always-visible action list.
- Move final-only into transcript view controls or footer/context action.
- Make terminal and plan actions conditional.
- Improve compact/narrow behavior.
- Keep footer shortcut row as the shortcut reference.

Tests:

- Update snapshots:
  - 72x18 focused sidebar
  - 100x24 wide layout
  - 132x24 three-column layout
  - queued messages narrow layout
- Add unit tests for action availability.

Acceptance:

- Sidebar prioritizes chats and attention.
- The main transcript keeps enough width on 72-column terminals.
- Actions shown in sidebar are actually useful in the current state.

## Phase 5: Input Extraction

Purpose: make redesigned input behavior easier to maintain.

Changes:

- Extract redesigned shortcut routing to `app/redesign_input.rs`.
- Extract terminal-window key handling if needed.
- Extract sidebar key handling if needed.
- Keep generic app input flow short.

Tests:

- Existing input tests.
- Add focused tests for:
  - sidebar focus does not swallow global quit/interruption keys
  - terminal overlay Esc collapse/close
  - plan/terminal/sidebar focus interactions

Acceptance:

- `app/input.rs` no longer contains most redesigned-specific key matching.
- Shortcut behavior remains stable.

## Phase 6: Legacy Bridge Cleanup

Purpose: make native redesigned composer behavior explicit.

Changes:

- Replace boolean `legacy_bottom_pane` with `RedesignComposerSurface`.
- Document which states use native composer vs legacy bottom pane.
- Remove redundant adapter getters that become unnecessary after render model extraction.

Tests:

- Slash command popup snapshots.
- Modal/popup interaction tests.
- Composer cursor tests.

Acceptance:

- The fallback behavior is named and intentional.
- Native redesigned composer and legacy bottom pane do not fight over status/queue display.

## Suggested Landing Order

1. Phase 1 extraction.
2. Phase 2 state owner.
3. Phase 3 working indicator summary.
4. Phase 4 sidebar UX.
5. Phase 5 input extraction.
6. Phase 6 bridge cleanup.

This order keeps screenshot churn mostly after the code is easier to change.

## Main Risks

| Risk | Mitigation |
| --- | --- |
| Snapshot churn hides behavior regressions | Do extraction first with no snapshot changes. |
| Activity summaries flicker too much | Add priority and stale-retention tests. |
| Sidebar loses discoverability | Keep footer shortcut hints and command palette entries. |
| Thread switching leaks old overlay state | Consolidate state before UX changes. |
| Legacy bottom-pane fallback breaks slash/modals | Keep fallback behavior unchanged until Phase 6. |
