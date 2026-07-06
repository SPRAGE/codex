# Working Indicator Summary

## Objective

The bottom working indicator should show a concise summary of current work, similar to the Codex UI behavior:

```text
Working  Running tests: just test -p codex-tui
Working  Editing files: redesign_chrome/sidebar.rs
Thinking Reviewing layout options
Waiting  Approve command: git add
```

The summary must be deterministic. It should come from events and tool metadata, not from a model-generated free-form summary.

## Current Rendering Capability

`StatusIndicatorWidget` already supports:

- Header text.
- Optional details.
- Inline suffix text.
- Elapsed time.
- Interrupt key hint.
- Animation control.

`BottomPane` already exposes:

- `update_status(...)`
- `status_indicator_line()`
- `set_unified_exec_processes(...)`
- `ensure_status_indicator()`

The redesign composer currently consumes only `redesign_work_status_line()`, so it usually gets a single line. That is fine for a first pass: use a compact one-line summary there, and let the legacy bottom pane continue using wrapped details when active.

## Proposed Data Model

Add a small activity model in TUI code, for example under `codex-rs/tui/src/chatwidget/activity_summary.rs`.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActivitySummary {
    pub(crate) kind: ActivityKind,
    pub(crate) headline: String,
    pub(crate) details: Option<String>,
    pub(crate) priority: ActivityPriority,
    pub(crate) source: ActivitySource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum ActivityPriority {
    Background,
    Streaming,
    Tool,
    WaitingForReview,
    WaitingForUser,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ActivityKind {
    StartingTurn,
    Thinking,
    Writing,
    UpdatingPlan,
    RunningCommand,
    EditingFiles,
    CallingMcpTool,
    CallingCustomTool,
    StartingMcp,
    WaitingForApproval,
    WaitingForBackgroundTerminal,
    ProcessingHook,
}
```

Keep this type TUI-local. It is view state, not protocol state.

## Summary Rules

### Turn Start

When the turn starts:

- Header: `Working`
- Summary: `Starting turn`

This should be short-lived and replaced as soon as richer activity arrives.

### Reasoning

When reasoning text provides a useful first bold heading:

- Header: `Thinking`
- Summary: first bold heading, trimmed.

If no heading exists:

- Header: `Thinking`
- Summary: `Planning next step`

### Assistant Text

When answer text streams:

- Header: `Writing`
- Summary: `Drafting response`

This has lower priority than tool calls and approvals, so it should not overwrite a command that is still running.

### Plan Updates

When `update_plan` is called or plan deltas stream:

- Header: `Working`
- Summary: `Updating plan`
- Details: current in-progress item if available.

### Exec Commands

When a shell command starts:

- Header: `Working`
- Summary: `Running command`
- Details: shell display string, truncated.

Examples:

```text
Running command: rg -n "Working" codex-rs/tui/src
Running tests: just test -p codex-tui
Formatting Rust: just fmt
```

Use simple classification for common commands:

- `just test`, `cargo test`, `nextest` -> `Running tests`
- `just fmt`, `cargo fmt`, `rustfmt` -> `Formatting`
- `just fix`, `cargo clippy` -> `Checking lint`
- `rg`, `grep`, `fd`, `find`, `sed`, `git grep` -> `Searching`
- `apply_patch` -> handled separately as editing files

### Apply Patch

When an apply-patch starts:

- Header: `Working`
- Summary: `Editing files`
- Details: file count or first few paths.

Example:

```text
Editing files: redesign_chrome.rs, sidebar.rs
```

### MCP and Custom Tools

For MCP:

```text
Calling MCP: google-calendar search events
Calling MCP: GitHub list pull requests
```

For custom tools:

```text
Using tool: image generation
Using tool: finance lookup
```

### Guardian Review and Approvals

These should take priority because the user may need to understand why work paused.

Examples:

```text
Reviewing command: git add
Reviewing patch: 3 files
Waiting for approval: network access to api.github.com
```

### Unified Exec Waiting

The current inline message already says things like background terminal counts. Keep that, but merge it into the activity model so the redesigned TUI and legacy bottom pane agree.

Example:

```text
Waiting for background terminal: just fix
```

## Priority and Flicker Control

Avoid replacing useful text too often.

Recommended rules:

- Higher priority replaces lower priority immediately.
- Same priority replaces only when the source changes or the previous summary is stale.
- Tool summaries remain visible until completion or until a higher-priority waiting/error state appears.
- A completed command can briefly show `Finished command` only if there is no assistant stream already visible.
- On turn completion, clear the summary with task-running state.

## Integration Points

Likely event handlers to update:

- `chatwidget/turn_runtime.rs`
  - `on_task_started`
  - `on_task_complete`

- `chatwidget/streaming.rs`
  - `restore_reasoning_status_header`
  - plan delta and plan completion paths
  - answer stream start paths

- `chatwidget/tool_requests.rs`
  - guardian review
  - approval request paths

- Exec and unified exec handlers
  - command begin, output, wait, completion

- MCP startup paths
  - startup status can become `Starting MCP servers`.

## API Shape

Avoid growing `set_status(...)` into a generic dumping ground. Add a clearer API:

```rust
impl ChatWidget {
    pub(crate) fn set_activity_summary(&mut self, summary: ActivitySummary) {
        self.activity_summary.set(summary);
        self.sync_activity_status();
    }

    fn sync_activity_status(&mut self) {
        let status = self.activity_summary.status_indicator_state();
        self.set_status(
            status.header,
            status.details,
            StatusDetailsCapitalization::Preserve,
            status.details_max_lines,
        );
    }
}
```

`StatusIndicatorState` can remain the render-facing shape. `ActivitySummary` should be the semantic source.

## Acceptance Criteria

- A running command shows the command or a classified command summary.
- Applying a patch shows that files are being edited.
- Reasoning changes the headline to a useful thinking summary.
- Approval/review states override generic work text.
- Existing tests that assert only `Working` are updated to assert specific summaries where behavior is intentional.
- The redesigned one-line composer indicator and the legacy bottom-pane status indicator use the same source data.
