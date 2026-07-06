# Redesign TUI Refactor Progress

Checkpoint commit: `f92dd3d402` (`docs: checkpoint redesign tui refactor plan`)

## Active Slice

Behavior-preserving renderer extraction plus first live activity-summary wiring for the
bottom status indicator.

## Task Ledger

| Task | Owner | Status | Notes |
| --- | --- | --- | --- |
| Create planning checkpoint | Main | Done | Commit `f92dd3d402`. |
| Extract plan-window rendering | Raman | Done | Moved overlay rect/rendering into `redesign_chrome/plan_window.rs` with existing tests kept in `redesign_chrome.rs`. |
| Extract composer rendering | Raman | Done | Moved native composer/status/queue rendering into `redesign_chrome/composer.rs` without touching snapshots or neighboring modules. |
| Extract footer rendering | Pending | Pending | Move footer rows out of `redesign_chrome.rs` without snapshot changes. |
| Map activity-summary integration points | Delta | Done | Found central status mutators, command/apply-patch/MCP/guardian data sources, and recommended a TUI-local `chatwidget/activity_summary.rs` slice. |
| Implement activity-summary status copy | Main | Done | Added `chatwidget/activity_summary.rs` and wired command, patch, MCP, and reasoning fallback statuses through the existing status indicator. |
| Add activity-summary snapshot coverage | Main | Done | Added command, patch, and MCP status-widget snapshots in `chatwidget/tests/status_and_layout.rs`. |
| Code review integrated patch | Sage | Done | No confirmed code issues found; noted new module files must be included in any implementation commit. |
| Final code review | Iris | Done | Required completion-path status snapshots were added; remaining commit risk is making sure all untracked files are included. |
| Run targeted verification | Main | Done | `cargo fmt -p codex-tui` passed; focused `just test -p codex-tui status_widget_` passed; full unsandboxed `just test -p codex-tui` passed 3072/3072 with 4 skipped; final unsandboxed `just fix -p codex-tui` passed; final `cargo fmt -p codex-tui` passed. Repo-wide `just fmt` blocked by missing `dotslash`/`uv` tools. |

## Review Log

- Delta read-only review: activity-summary slice should preserve `StatusIndicatorWidget` and route existing `set_status` behavior through a semantic TUI-local model. Main risk is preserving guardian/waiting precedence and stream-idle restoration.
- Main verification: sandboxed `just test -p codex-tui` ran 3069 tests with 3066 passed and 3 Unix-socket permission failures. Focused unsandboxed rerun of those 3 passed. Full unsandboxed rerun passed 3069 tests with 4 skipped.
- Sage code review: no confirmed issues in the scoped extraction. The render order and moved logic look behavior-preserving, `git diff --check` passed, and no pending `*.snap.new` files were present. Residual risk is only that `.ai/plans/redesign-tui-refactor/progress.md`, `codex-rs/tui/src/redesign_chrome/composer.rs`, and `codex-rs/tui/src/redesign_chrome/plan_window.rs` are untracked until the implementation is committed.
- Main implementation: command starts now summarize as `Running tests`, `Formatting`, `Linting`, etc.; patch begin summarizes edited files; MCP begin prefers app/action names when available; reasoning without a bold heading falls back to `Thinking`. Tool completion restores the reasoning or working label when no higher-priority guardian/MCP-startup status owns the footer.
- Iris code review: renderer extraction and status summaries looked sound. Findings were bookkeeping for untracked files and missing direct completion-path tests. Main added command, patch, and MCP completion snapshots proving the footer restores to `Working` after those activities complete.
