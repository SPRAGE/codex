# Current State

## Code Map

- `codex-rs/tui/src/redesign_chrome.rs`
  - Main redesigned renderer.
  - Builds `RedesignChromeContext` directly from `App`.
  - Renders chat header, transcript, composer, footer, right rail, plan window, and terminal window.
  - Also contains a large amount of snapshot test code.
  - Current size: about 3900 lines.

- `codex-rs/tui/src/redesign_chrome/sidebar.rs`
  - Owns sidebar state, sidebar item enum, chat activity enum, chat row rendering, and action rendering.
  - Current size: about 377 lines.

- `codex-rs/tui/src/redesign_chrome/layout.rs`
  - Owns sidebar, main, right rail, footer, transcript, and composer rectangles.
  - Uses fixed breakpoints: sidebar appears at width >= 88, compact sidebar appears only when focused and width >= 56, right rail appears at width >= 120.

- `codex-rs/tui/src/redesign_chrome/background_terminals.rs`
  - Renders the terminal-session overlay.

- `codex-rs/tui/src/app/input.rs`
  - Owns redesigned shortcut routing, sidebar focus/navigation, transcript scroll, plan window keys, terminal window keys, and normal app fallback routing.
  - Current size: about 1848 lines.

- `codex-rs/tui/src/app.rs`
  - Stores redesign state directly on `App`: sidebar state, transcript scroll, final-only flag, plan window thread set, terminal window thread sets/maps, chat names, activity, unread, and pending new-chat data.

- `codex-rs/tui/src/chatwidget.rs`
  - Provides many `redesign_*` getters that adapt existing `ChatWidget` state into redesigned chrome data.
  - Bridges back to the legacy bottom pane whenever slash input, popups, or modal views are active.

- `codex-rs/tui/src/status_indicator_widget.rs`
  - Already supports a strong status indicator shape: header, details, inline message, elapsed time, interrupt hint, wrapping, and animation.

## Visible UX Problems

The 100x24 snapshot shows the main clutter:

- The sidebar takes 24 columns and spends a lot of its height on static actions.
- Chat rows use padded words like `active`, `unread`, and `needs`, which consume width before the user sees the chat label.
- The main header packs model, reasoning, context, tokens, permissions, and approval into two rows at the very top.
- The working row usually says only `Working`, even when the app knows the current action.
- The footer repeats several actions already shown in the sidebar.
- The right rail appears only on wide terminals and currently acts as a secondary activity log, which makes the information architecture vary sharply by width.

The 72x18 focused-sidebar snapshot shows a stronger issue:

- The sidebar still takes enough width to force narrow chat bubbles.
- Long labels and footer status truncate aggressively.
- The same actions remain visible even though the terminal is height constrained.

## Behavioral Problems

The redesign currently works as a chrome layer over existing `ChatWidget` behavior. That is pragmatic, but the ownership boundaries are now blurry.

- `App` owns thread switching and per-thread redesign state.
- `ChatWidget` owns task lifecycle, status indicator state, transcript cells, modal views, and composer behavior.
- `BottomPane` owns the reusable status widget, queued-message preview, slash popups, footer, and composer.
- `redesign_chrome.rs` reaches into both `App` and `ChatWidget` to render derived state.

This makes it hard to fix a visible issue without touching unrelated plumbing.

## Working Indicator Today

The reusable status widget can already render:

- A header such as `Working`, `Thinking`, or guardian review text.
- Wrapped details below the header.
- Inline suffix text such as unified-exec process summaries.
- Elapsed time and interrupt key.

The missing piece is not rendering capability. The missing piece is a consistent activity summary source. Many event handlers still set generic `Working`, while specific context exists nearby:

- MCP startup can say which startup phase is active.
- Streaming reasoning can expose a thinking headline.
- Tool calls know command, patch, MCP tool, or custom tool names.
- Guardian review knows what is being reviewed.
- Unified exec knows running processes and command display text.
- Plan streaming knows when a plan is being updated.

## Sidebar Today

`RedesignSidebarItem` currently mixes:

- Primary session actions: new chat, close chat through `Alt-W`.
- Filters: final-only.
- Launchers: commands, models, history, transcript, terminals, editor.
- Chat rows: active, unread, working, done, needs input, failed, closed.

The result is a sidebar that behaves like both a navigator and a shortcut legend. That is why it feels cluttered.

## Redundant State Patterns

The largest cleanup target is parallel per-thread redesign state on `App`:

- `redesign_plan_window_open_threads`
- `redesign_terminal_window_open_threads`
- `redesign_terminal_window_scroll`
- `redesign_terminal_window_selected`
- `redesign_terminal_window_expanded`
- `redesign_chat_names`
- `redesign_chat_activity`
- `redesign_chat_unread`

These should become one per-thread UI state map with explicit fields. That reduces cleanup bugs when closing or switching threads and gives the sidebar a single source for chat state.

## Constraints

- Keep redesign work inside `codex-rs/tui`.
- Avoid adding code to `codex-core`.
- Keep changed lines under reviewable limits by staging work.
- Run `just fmt` after Rust edits.
- Use `just test -p codex-tui` for TUI changes.
- Update `insta` snapshots for user-visible UI changes.
