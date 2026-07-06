# Architecture Cleanup

## Problem

The redesign began as a chrome layer over existing TUI behavior. That was a good way to ship quickly, but the code now has several ownership leaks:

- Rendering code derives application state.
- `App` stores redesign state in many parallel maps and sets.
- `ChatWidget` exposes many narrow `redesign_*` getters.
- `app/input.rs` owns both generic app input and detailed redesigned input behavior.
- `redesign_chrome.rs` is too large and owns unrelated surfaces.

The cleanup should reduce cross-module knowledge before adding more visual behavior.

## Target Ownership

### App

Owns:

- Current displayed thread.
- Thread switching and thread lifecycle.
- Redesign UI state container.
- Input routing at the app boundary.

Should not own:

- Rendering details.
- Individual sidebar line formatting.
- Parallel redesign maps.

### ChatWidget

Owns:

- Turn lifecycle.
- Transcript cells.
- Composer and bottom-pane behavior.
- Activity summary source for the active chat.

Should not own:

- Sidebar row rendering.
- Redesign chrome layout.
- Per-thread overlay state unrelated to the active widget.

### Redesign Chrome

Owns:

- Rendering pure models into buffers.
- Layout decisions.
- Snapshot-testable display behavior.

Should not own:

- Deriving thread state from app-server routing.
- Mutating app state.
- Calling many `ChatWidget` getters directly.

## State Consolidation

Replace the parallel redesign fields on `App` with one owner.

Current state:

```rust
redesign_sidebar_state: RedesignSidebarState,
redesign_transcript_scroll: usize,
redesign_final_only_transcript: bool,
redesign_plan_window_open_threads: HashSet<ThreadId>,
redesign_terminal_window_open_threads: HashSet<ThreadId>,
redesign_terminal_window_scroll: HashMap<ThreadId, usize>,
redesign_terminal_window_selected: HashMap<ThreadId, usize>,
redesign_terminal_window_expanded: HashMap<ThreadId, usize>,
redesign_chat_names: HashMap<ThreadId, String>,
redesign_chat_activity: HashMap<ThreadId, RedesignChatActivity>,
redesign_chat_unread: HashSet<ThreadId>,
```

Proposed:

```rust
pub(crate) struct RedesignUiState {
    pub(crate) sidebar: RedesignSidebarState,
    pub(crate) transcript: RedesignTranscriptState,
    pub(crate) threads: HashMap<ThreadId, RedesignThreadUiState>,
    pub(crate) pending_chat_start: Option<PendingRedesignChatStart>,
    pub(crate) pending_notifications: VecDeque<String>,
}

pub(crate) struct RedesignThreadUiState {
    pub(crate) display_name: Option<String>,
    pub(crate) activity: RedesignChatActivity,
    pub(crate) unread: bool,
    pub(crate) plan_window_open: bool,
    pub(crate) terminal_window: RedesignTerminalWindowState,
}

pub(crate) struct RedesignTerminalWindowState {
    pub(crate) open: bool,
    pub(crate) scroll: usize,
    pub(crate) selected: usize,
    pub(crate) expanded: Option<usize>,
}
```

This makes chat close, thread switch, and reset operations less error-prone.

## Renderer Module Split

Split `redesign_chrome.rs` into modules that match surfaces.

Proposed layout:

```text
codex-rs/tui/src/redesign_chrome.rs
codex-rs/tui/src/redesign_chrome/
  background_terminals.rs
  composer.rs
  footer.rs
  layout.rs
  model.rs
  plan_window.rs
  sidebar.rs
  system_rail.rs
  transcript.rs
  window.rs
```

Initial extraction should be behavior-preserving:

- Move code, do not redesign output.
- Keep existing tests passing before changing snapshots.
- Keep helper visibility narrow.

Likely extraction order:

1. `model.rs` for `RedesignChromeContext` and pure render model types.
2. `composer.rs` for composer height, queue preview, and work status row rendering.
3. `footer.rs` for footer info and shortcut rows.
4. `system_rail.rs` for the right rail.
5. `plan_window.rs` for plan overlay.
6. `transcript.rs` for transcript block conversion and rendering.

## Input Module Split

Extract redesigned input routing from `app/input.rs`.

Proposed:

```text
codex-rs/tui/src/app/input.rs
codex-rs/tui/src/app/redesign_input.rs
codex-rs/tui/src/app/redesign_terminal_input.rs
codex-rs/tui/src/app/redesign_sidebar_input.rs
```

The first extraction can keep the same public helper methods on `App`, but move:

- `handle_redesign_shortcut_key`
- sidebar key matching and activation
- terminal window key handling
- transcript scroll key handling
- redesign key matcher functions

This makes the app input fallback path easier to reason about.

## Activity Summary Ownership

Add a semantic activity layer between event handlers and status rendering.

Current pattern:

- Event handlers call `set_status_header("Working")` or `set_status(...)` directly.
- Some handlers also update terminal title state.
- The redesign asks the bottom pane for a line.

Proposed pattern:

- Event handlers call `set_activity_summary(...)`.
- `ActivitySummaryState` converts semantic activity into:
  - status indicator state
  - terminal title run-state
  - sidebar chat summary
  - optional right-rail activity item

This removes duplicated phrasing across status row, terminal title, sidebar, and right rail.

## Legacy Bottom Pane Bridge

`redesign_should_render_bottom_pane()` currently decides when to fall back to the legacy bottom pane:

- modal or popup active
- active view present
- composer text starts with `/`

This is a useful compatibility bridge, but it should be made explicit.

Proposed enum:

```rust
pub(crate) enum RedesignComposerSurface {
    Native,
    LegacyBottomPane,
}
```

Then `layout_for(...)` and `render_app(...)` can switch on a named mode instead of a boolean.

## Redundant Code to Remove or Reduce

- Repeated per-thread overlay cleanup in close/reset paths after `RedesignUiState`.
- Sidebar action rendering for actions that should move to command palette/footer.
- `redesign_*` getters that only forward bottom-pane fields after a render model exists.
- Generic `Working` status updates that become semantic activity summaries.
- Duplicate shortcut hints in sidebar and footer.

## Acceptance Criteria

- `redesign_chrome.rs` drops below 800 lines, excluding tests if the tests remain nearby.
- Each renderer module has a single visible surface responsibility.
- Closing a chat requires one state removal call.
- Input routing for redesigned mode is readable without scanning the entire app input file.
- Status phrasing is produced from one activity summary layer.
