# Testing Plan

## Snapshot Coverage

Any visible redesigned TUI change should update `insta` snapshots.

Important existing redesigned snapshots:

- `redesign_chrome_wide_100x24`
- `redesign_chrome_three_column_132x24`
- `redesign_chrome_focused_sidebar_72x18`
- `redesign_chrome_narrow_72x18`
- `redesign_chrome_queued_messages_52x7`
- `redesign_chrome_terminal_window_100x24`
- `redesign_chrome_plan_window_100x24`

Add or update snapshots for:

- Working indicator with command summary.
- Working indicator with apply-patch summary.
- Working indicator with approval/review summary.
- Sidebar with multiple chats: working, needs input, failed, done, idle.
- Sidebar with no secondary actions available.
- Sidebar with terminal action visible because terminals exist.
- Narrow terminal where sidebar is hidden or compact and transcript remains readable.

## Unit Tests

### Activity Summary

Add tests for:

- Higher priority replaces lower priority.
- Same priority does not churn on identical summary.
- Tool summary remains while command is still running.
- Approval summary overrides command summary.
- Completion clears summary when task-running state ends.
- Command classifier maps common commands:
  - `just test -p codex-tui`
  - `just fmt`
  - `just fix -p codex-tui`
  - `rg -n ...`
  - `git status --short`

### Redesign UI State

Add tests for:

- Removing a thread clears chat name, activity, unread, plan, terminal, and selection state.
- Terminal selected index clamps when terminal count shrinks.
- Sidebar selection normalizes after chat close.
- Reset clears all redesign state except global defaults that should persist.

### Sidebar Model

Add tests for:

- Action availability at idle.
- Action availability while a plan exists.
- Action availability while terminal sessions exist.
- Active chat row uses active state over unread state.
- Done state becomes idle or less prominent after seen.

## Integration Tests

Prefer existing TUI integration helpers where possible.

Scenarios:

- Turn starts, runs a command, and the status row updates to `Running command`.
- Turn applies a patch and status row updates to `Editing files`.
- Guardian review in progress updates the status row and remains interruptible.
- Switching to another redesign chat preserves that chat's sidebar/terminal state.
- Closing a running chat interrupts it and removes all redesign UI state.

## Manual Test Matrix

Run the redesigned TUI manually for these workflows:

- Start a new chat.
- Open two chats and switch between them from the sidebar.
- Run a command that produces background terminal output.
- Open terminal overlay, expand output, scroll, collapse, close.
- Enter Plan mode and open the plan overlay.
- Trigger slash command popup from redesigned composer.
- Trigger model picker.
- Trigger permission popup.
- Queue a follow-up message while work is running.
- Resize between 72, 100, and 132 columns.

## Commands

From `codex-rs`:

```sh
just fmt
just test -p codex-tui
cargo insta pending-snapshots -p codex-tui
```

If snapshot changes are intentional:

```sh
cargo insta accept -p codex-tui
```

For larger Rust changes before final review:

```sh
just fix -p codex-tui
```

Do not run `cargo test` directly for this repo. Use `just test`.
