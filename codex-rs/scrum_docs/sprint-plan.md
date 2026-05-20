# Sprint Plan

## Release Goal

Ship a provider-independent redesign TUI crate inside the workspace while
preserving Codex redesign behavior through a Codex host adapter.

## Sprint 0: Boundary Discovery and Contract Lock

Duration: 2 to 3 days

Goal: create the smallest stable contract before moving production rendering.

Deliverables:

- Neutral `RedesignViewModel` draft in `tui-redesign/src/state.rs`.
- Neutral `RedesignAction` draft and reducer module outline.
- Dependency boundary review for `tui-redesign/Cargo.toml`.
- Codex adapter module skeleton under `tui/src/redesign_host/`.

Review gate:

- `codex-tui-redesign` compiles without Codex runtime dependencies.
- Contract can represent the current wide, narrow, plan-window, and
  terminal-window snapshots.

Suggested verification:

```bash
cargo test -p codex-tui-redesign
```

## Sprint 1: Renderer Extraction

Duration: 4 to 6 days

Goal: move production layout and rendering into the extracted crate.

Deliverables:

- Layout, theme, sidebar, transcript, system rail, composer, plan-window, and
  terminal-window renderers live in `codex-tui-redesign`.
- Neutral renderer snapshots cover tiny, narrow, standard, and wide viewports.
- Codex still renders through its existing path until parity is proven.

Review gate:

- Snapshot diffs are understood and intentional.
- Extracted renderer does not import `App`, `ChatWidget`, `HistoryCell`,
  `ThreadId`, or Codex provider/runtime types.

Suggested verification:

```bash
cargo test -p codex-tui-redesign
cargo test -p codex-tui redesign_chrome::tests
```

## Sprint 2: Codex View-Model Adapter

Duration: 3 to 5 days

Goal: make Codex production mode build and render the neutral view model.

Deliverables:

- `tui/src/redesign_host/view_model.rs` maps `App` and `ChatWidget` into
  `RedesignViewModel`.
- Existing redesign render entrypoint calls the extracted renderer.
- Adapter tests cover labels, context, sidebar entries, transcript blocks,
  queued messages, plan content, and terminal sessions.

Review gate:

- Existing Codex redesign snapshots pass or have reviewed intentional diffs.
- The adapter is the only layer that knows Codex app-state details.

Suggested verification:

```bash
cargo test -p codex-tui redesign_chrome::tests
cargo test -p codex-tui chatwidget::tests::exec_flow
cargo test -p codex-tui chatwidget::tests::plan_mode
```

## Sprint 3: Input Reducer and Action Executor

Duration: 4 to 6 days

Goal: move provider-neutral shortcut interpretation into the extracted crate and
leave Codex to execute emitted actions.

Deliverables:

- `tui-redesign/src/input.rs` emits `RedesignAction`.
- `tui/src/redesign_host/actions.rs` maps actions to current Codex behavior.
- App input tests continue to cover global quit, Ctrl-C clear-first behavior,
  sidebar focus, new chat, close chat, plan window, terminal window, transcript
  scroll, and composer cursor behavior.

Review gate:

- Shortcut priority is preserved.
- Slow chat start is still non-blocking.
- Terminal-window selection and expansion remain per chat.

Suggested verification:

```bash
cargo test -p codex-tui-redesign input
cargo test -p codex-tui input::tests
```

## Sprint 4: Cleanup and Host Enablement

Duration: 2 to 4 days

Goal: remove duplicate production code and document how another provider hosts
the extracted TUI.

Deliverables:

- Old direct production renderer path is removed or reduced to a compatibility
  shim with a removal condition.
- `tui-redesign/README.md` describes the crate as production UI infrastructure.
- Host adapter guide is added near the crate.
- Mock host fixture proves the crate can render without Codex state.

Review gate:

- No provider-specific dependencies are introduced into `codex-tui-redesign`.
- A new provider host can be described as "implement view-model builder and
  action executor" rather than "copy Codex TUI."

Suggested verification:

```bash
cargo test -p codex-tui-redesign
cargo test -p codex-tui redesign_chrome::tests
cargo test -p codex-tui input::tests
git diff --check
```

## Release Review

Release when:

- All P0 stories in `product-backlog.md` are complete.
- Definition of done in `definition-of-done.md` passes.
- The Codex redesign path has no known behavior regressions.
- The extracted crate can be consumed by a non-Codex host in design without
  adding provider-specific imports.
