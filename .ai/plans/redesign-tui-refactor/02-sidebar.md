# Sidebar Refactor

## Objective

The sidebar should answer three questions quickly:

1. Which chat am I in?
2. Which chats need attention?
3. Where do I go next if I need another workspace surface?

It should not be the primary place to explain every shortcut.

## Proposed Information Architecture

### Top: Workspace Identity

Keep this compact.

Current:

- Product/version.
- Separate top header in main area repeats model and policy metadata.

Proposed sidebar content:

- Product/version only when useful.
- Active project/cwd shortened to one row if it helps orientation.
- Avoid duplicating model, approval, and token data in the sidebar.

### Middle: Chats

Make chat navigation the sidebar's main job.

Each chat row should include:

- A compact status marker.
- A short label.
- Optional unread/attention marker.
- Optional one-line activity summary when width allows.

Suggested row shape:

```text
> Main        working
  Running tests

! Patch       needs input
  Approve apply_patch

  Scout       done
```

For narrow widths, collapse to:

```text
> Main  work
! Patch ask
  Scout done
```

### Bottom: Primary Actions Only

Keep bottom actions to the smallest stable set:

- New Chat
- Commands
- Terminals, only when there are active or recent terminal sessions
- Plan, only when there is a plan or the current mode is Plan

Move these out of the always-visible sidebar:

- Models: keep as `Alt-M` and command palette entry.
- History: keep in command palette or footer hint.
- Transcript: keep `Ctrl-T` or footer hint.
- Editor: keep app keybinding and command palette.
- Final Only: make it a transcript/view filter near the transcript, not a global sidebar action.

The footer can still show a compact shortcut row. The sidebar should not duplicate it unless focused and in help mode.

## Interaction Model

### Focus

Keep `Alt-B` or `Ctrl-B` as the focus toggle, but separate focus from visibility:

- Wide terminals: sidebar remains visible by default; `Alt-B` moves focus in/out.
- Medium terminals: sidebar can collapse to a compact rail; `Alt-B` opens full sidebar focus.
- Narrow terminals: sidebar should be overlay-style or hidden by default.

### Navigation

When focused:

- Up/down and `j`/`k` move through visible rows.
- Enter activates the selected chat or action.
- Esc exits sidebar focus.
- `Alt-W` closes the selected chat only when a chat row is selected.

Avoid catching arbitrary text keys while focused unless they are local sidebar shortcuts.

### Chat Attention

Use attention states that map to user decisions:

- `working` - background turn is running.
- `needs input` - approval, elicitation, or user input is pending.
- `failed` - terminal turn failed.
- `done` - completed and unread.
- `idle` - no special state.
- `closed` - only if closed chats remain visible.

The current `Done` state should not stay visually loud once the user has visited the chat.

## Render Model

Introduce a sidebar render model instead of building rows directly from `RedesignChromeContext`:

```rust
pub(crate) struct RedesignSidebarModel {
    pub(crate) focused: bool,
    pub(crate) selection: RedesignSidebarSelection,
    pub(crate) chats: Vec<RedesignSidebarChatRow>,
    pub(crate) actions: Vec<RedesignSidebarActionRow>,
}

pub(crate) struct RedesignSidebarChatRow {
    pub(crate) thread_id: ThreadId,
    pub(crate) label: String,
    pub(crate) state: RedesignChatActivity,
    pub(crate) attention: RedesignAttentionState,
    pub(crate) summary: Option<String>,
    pub(crate) active: bool,
}
```

This model should be created from `App`, not by the renderer. Rendering should not know how to derive unread state, thread labels, or action availability.

## Implementation Steps

1. Add `redesign_chrome/sidebar/model.rs` or `app/redesign_sidebar_model.rs`.
2. Keep old rendering output identical while routing through the new model.
3. Add `summary: Option<String>` to chat rows, populated from the same `ActivitySummary` work described in `03-working-indicator.md`.
4. Split `RedesignSidebarItem::ALL` into action groups with availability:
   - Always: New Chat, Commands.
   - Conditional: Plan, Terminals.
   - Hidden from sidebar by default: Models, History, Transcript, Editor, Final Only.
5. Update snapshots for 72x18, 100x24, and 132x24.

## Acceptance Criteria

- A user can identify active, running, and blocked chats without reading long labels.
- The active chat remains obvious when the sidebar is focused and unfocused.
- The sidebar does not duplicate the footer shortcut list.
- On a 72-column terminal, the sidebar does not make the transcript unreadable.
- Closing a chat removes all associated sidebar, plan, terminal, unread, and activity state through one state owner.
