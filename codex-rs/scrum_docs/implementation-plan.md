# Provider-Independent Redesign TUI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the production redesign TUI into `codex-tui-redesign` so any
provider host can render it by supplying neutral state and executing neutral
actions.

**Architecture:** `codex-tui-redesign` owns rendering, layout, neutral state,
and input-to-action reduction. `codex-tui` owns the Codex host adapter that maps
`App` and `ChatWidget` into the neutral view model and maps actions back into
current Codex behavior.

**Tech Stack:** Rust 2024, ratatui, crossterm, insta snapshots, existing Cargo
workspace crates.

---

## File Structure

Create or modify these files during implementation:

- Modify: `tui-redesign/src/lib.rs` - public exports for view model, renderer,
  reducer, and actions.
- Modify: `tui-redesign/src/state.rs` - provider-neutral view model, IDs, and
  display structs.
- Create: `tui-redesign/src/input.rs` - provider-neutral key and mouse reducer.
- Modify: `tui-redesign/src/render.rs` - production renderer entrypoint over
  `RedesignViewModel`.
- Modify: `tui-redesign/src/render/overlay.rs` - neutral overlay rendering.
- Modify: `tui-redesign/src/render/tests.rs` - neutral snapshot coverage.
- Create: `tui/src/redesign_host.rs` - Codex host adapter module root.
- Create: `tui/src/redesign_host/view_model.rs` - Codex state to neutral view
  model conversion.
- Create: `tui/src/redesign_host/actions.rs` - neutral action execution for
  Codex.
- Modify: `tui/src/redesign_chrome.rs` - temporary compatibility wrapper or
  removal after extracted renderer parity.
- Modify: `tui/src/app/input.rs` - call reducer and execute emitted actions.
- Modify: `tui/src/lib.rs` - module registration for `redesign_host`.
- Modify: `tui-redesign/README.md` - update from sandbox to production
  infrastructure once the migration is complete.

## Task 1: Define Neutral UI Contract

**Files:**

- Modify: `tui-redesign/src/state.rs`
- Modify: `tui-redesign/src/lib.rs`

- [ ] **Step 1: Add failing compile-focused tests for neutral contract**

Add a test in `tui-redesign/src/render/tests.rs` that constructs a
`RedesignViewModel` without any Codex types and renders it at `100x24`.

- [ ] **Step 2: Run the test and confirm missing contract types**

Run:

```bash
cargo test -p codex-tui-redesign render::tests::neutral_view_model_renders_standard
```

Expected result: compile failure because the new neutral contract types are not
defined yet.

- [ ] **Step 3: Add the neutral structs**

Add crate-owned structs and enums for:

- `ChatId`
- `RedesignViewModel`
- `ChromeViewModel`
- `SidebarViewModel`
- `ChatListEntry`
- `TranscriptViewModel`
- `TranscriptBlock`
- `TranscriptRole`
- `SystemRailViewModel`
- `ComposerViewModel`
- `OverlayViewModel`
- `PlanWindowViewModel`
- `TerminalWindowViewModel`
- `TerminalSessionViewModel`

Use doc comments on public types explaining that host adapters must convert
provider-specific runtime state before constructing these values.

- [ ] **Step 4: Export the contract**

Update `tui-redesign/src/lib.rs` to export the new view-model types and keep
existing demo exports only if they are still used by tests or the preview
binary.

- [ ] **Step 5: Verify**

Run:

```bash
cargo test -p codex-tui-redesign
```

Expected result: tests pass or only renderer tests fail because rendering still
uses the older sandbox state.

## Task 2: Move Renderer to the Neutral View Model

**Files:**

- Modify: `tui-redesign/src/render.rs`
- Modify: `tui-redesign/src/render/overlay.rs`
- Modify: `tui-redesign/src/render/tests.rs`
- Modify: `tui-redesign/src/theme.rs`

- [ ] **Step 1: Port production layout behavior**

Move the current production layout concepts from `tui/src/redesign_chrome.rs`
and `tui/src/redesign_chrome/layout.rs` into `tui-redesign/src/render.rs` or
small submodules under `tui-redesign/src/render/`.

- [ ] **Step 2: Render from `RedesignViewModel` only**

Replace renderer references to demo-only `RedesignState` with
`RedesignViewModel`. Keep demo fixture constructors in tests or a fixture module
instead of making them part of the runtime contract.

- [ ] **Step 3: Add snapshot fixtures**

Add provider-neutral snapshots for:

- `44x4` composer survival
- `72x18` narrow transcript
- `100x24` standard layout
- `132x24` wide layout
- plan window open
- terminal window open with one expanded session

- [ ] **Step 4: Verify**

Run:

```bash
cargo test -p codex-tui-redesign
```

Expected result: all neutral renderer snapshots pass after review and
acceptance.

## Task 3: Add Codex View-Model Adapter

**Files:**

- Create: `tui/src/redesign_host.rs`
- Create: `tui/src/redesign_host/view_model.rs`
- Modify: `tui/src/lib.rs`
- Modify: `tui/Cargo.toml`

- [ ] **Step 1: Add dependency from `codex-tui` to `codex-tui-redesign`**

Add this dependency to `tui/Cargo.toml`:

```toml
codex-tui-redesign = { workspace = true }
```

Do not add the reverse dependency.

- [ ] **Step 2: Register the adapter module**

Add `mod redesign_host;` in `tui/src/lib.rs` near the existing redesign module
registration.

- [ ] **Step 3: Build `RedesignViewModel` from Codex state**

In `tui/src/redesign_host/view_model.rs`, add a function with this shape:

```rust
pub(crate) fn view_model_from_app(app: &App, area: Rect) -> RedesignViewModel
```

Map the current data exposed by `ChatWidget` redesign methods into neutral
fields. Convert Codex thread IDs into `ChatId(thread_id.to_string())` at the
adapter boundary.

- [ ] **Step 4: Add adapter tests**

Add tests that verify the adapter maps:

- model label
- reasoning label
- context remaining
- sidebar chat entries
- queued messages
- plan display lines
- terminal sessions

- [ ] **Step 5: Verify**

Run:

```bash
cargo test -p codex-tui redesign_host
```

Expected result: adapter tests pass.

## Task 4: Route Codex Rendering Through Extracted Renderer

**Files:**

- Modify: `tui/src/redesign_chrome.rs`
- Modify: `tui/src/redesign_chrome/*.rs`
- Modify: `tui/src/snapshots/*redesign_chrome*.snap`

- [ ] **Step 1: Keep a compatibility wrapper**

Keep `tui/src/redesign_chrome.rs::render_app` as the production entrypoint for
now, but make it build a `RedesignViewModel` through `redesign_host` and call
the renderer in `codex-tui-redesign`.

- [ ] **Step 2: Preserve cursor behavior**

Return cursor position and cursor style from the extracted renderer or from the
Codex bottom-pane fallback adapter. Do not let the extracted crate reach back
into `ChatWidget`.

- [ ] **Step 3: Verify snapshot parity**

Run:

```bash
cargo test -p codex-tui redesign_chrome::tests
```

Expected result: snapshots pass or produce reviewed intentional diffs.

## Task 5: Extract Input Reducer

**Files:**

- Create: `tui-redesign/src/input.rs`
- Modify: `tui-redesign/src/lib.rs`
- Modify: `tui/src/app/input.rs`
- Create: `tui/src/redesign_host/actions.rs`

- [ ] **Step 1: Define `RedesignAction`**

Create an enum in `tui-redesign/src/input.rs` that represents user intent. It
must include actions for redraw, help, commands, model picker, new chat, close
chat, plan window, terminal window, final-only transcript, transcript scroll,
terminal selection, composer clear, and interrupt or quit.

- [ ] **Step 2: Move neutral shortcut interpretation**

Move shortcut matching that does not require Codex state from
`tui/src/app/input.rs` into the reducer. Pass neutral facts into the reducer,
including composer empty state, sidebar focus, terminal window state, plan
window state, terminal session count, and viewport size.

- [ ] **Step 3: Add reducer tests**

Add tests in `tui-redesign` for:

- Esc closes plan window.
- Esc collapses terminal output before closing terminal window.
- Alt+N emits `StartNewChat`.
- Alt+W emits `CloseChat`.
- Alt+T emits `ToggleTerminalWindow`.
- Alt+P emits `TogglePlanWindow`.
- Empty up/down scrolls transcript.
- Non-empty left/right stays with composer by returning no host navigation
  action.

- [ ] **Step 4: Execute actions in Codex**

In `tui/src/redesign_host/actions.rs`, map neutral actions to the existing Codex
behavior. Keep async operations in the Codex layer.

- [ ] **Step 5: Verify**

Run:

```bash
cargo test -p codex-tui-redesign input
cargo test -p codex-tui input::tests
```

Expected result: reducer tests and Codex app-level behavior tests pass.

## Task 6: Remove Duplicate Production Renderer Code

**Files:**

- Modify: `tui/src/redesign_chrome.rs`
- Modify: `tui/src/redesign_chrome/*.rs`
- Modify: `tui-redesign/README.md`

- [ ] **Step 1: Remove direct production rendering helpers from `codex-tui`**

After snapshot parity, delete renderer-only helpers from `tui/src/redesign_chrome.rs`
that now live in `codex-tui-redesign`.

- [ ] **Step 2: Keep Codex-only adapters in `codex-tui`**

Keep only Codex-specific view-model building, action execution, and compatibility
entrypoints in `codex-tui`.

- [ ] **Step 3: Update README**

Change `tui-redesign/README.md` so it describes the crate as production
provider-independent TUI infrastructure, not a sandbox.

- [ ] **Step 4: Verify final checks**

Run:

```bash
cargo test -p codex-tui-redesign
cargo test -p codex-tui redesign_chrome::tests
cargo test -p codex-tui input::tests
git diff --check
```

Expected result: all checks pass and no whitespace errors remain.

## Task 7: Add Host Adapter Guide

**Files:**

- Create: `tui-redesign/HOST_ADAPTER.md`

- [ ] **Step 1: Document host responsibilities**

Document that a provider host must:

- Build `RedesignViewModel`.
- Execute `RedesignAction`.
- Own provider auth, model catalogs, streaming events, persistence, and tool
  execution.
- Convert provider/runtime IDs into neutral UI IDs.
- Schedule redraws after state changes.

- [ ] **Step 2: Add pseudocode host loop**

Include a short host loop:

```rust
loop {
    host.apply_runtime_events();
    let view_model = host.build_redesign_view_model();
    terminal.draw(|frame| codex_tui_redesign::render(frame, &view_model))?;
    if let Some(input) = host.next_input_event().await {
        for action in codex_tui_redesign::reduce_input(input, &view_model) {
            host.execute_redesign_action(action).await?;
        }
    }
}
```

- [ ] **Step 3: Verify docs**

Run:

```bash
git diff --check
```

Expected result: no whitespace errors.
