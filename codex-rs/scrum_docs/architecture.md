# Target Architecture

## Current Coupling

The production redesign currently lives in `tui/src/redesign_chrome.rs` and
renders from `App` and `ChatWidget` directly. The surrounding app modules own
input, chat lifecycle, sidebars, plan windows, terminal windows, and thread
state. `tui-redesign/` is already a separate crate, but it is a sandbox and does
not represent shipped behavior.

The extraction should convert this shape:

```text
codex-tui App + ChatWidget
  -> redesign_chrome.rs reads app state directly
  -> input.rs mutates app state directly
```

Into this shape:

```text
host runtime
  -> host adapter builds RedesignViewModel
  -> codex-tui-redesign renders RedesignViewModel
  -> codex-tui-redesign reducer emits RedesignAction
  -> host adapter executes RedesignAction
```

## Crate Boundary

`codex-tui-redesign` should own:

- Provider-neutral state structs.
- Rendering widgets and layout logic.
- Redesign input reducer.
- Snapshot tests for provider-neutral rendering.
- Unit tests for provider-neutral navigation and action emission.

`codex-tui` should own:

- Codex host adapter from `App` and `ChatWidget` into `RedesignViewModel`.
- Codex action executor from `RedesignAction` into existing app behavior.
- Codex-specific bottom-pane fallback integration.
- Codex app-server session, thread lifecycle, model popups, approvals, and
  tool/runtime integration.

Provider runtime crates should own:

- Auth and provider config.
- Model catalog and model selection persistence.
- Stream/event conversion into the host state model.
- Tool execution and approval backends.

## Neutral View Model

The extracted crate should expose a single render input:

```rust
pub struct RedesignViewModel {
    pub chrome: ChromeViewModel,
    pub sidebar: SidebarViewModel,
    pub transcript: TranscriptViewModel,
    pub system_rail: SystemRailViewModel,
    pub composer: ComposerViewModel,
    pub overlays: OverlayViewModel,
    pub shortcuts: ShortcutViewModel,
}
```

Provider-neutral model rules:

- Use `String`, `usize`, `bool`, neutral enums, and `ratatui` display types only
  where the UI crate already owns the display responsibility.
- Do not expose Codex `ThreadId`; use `ChatId(String)` or a crate-owned newtype.
- Do not expose `HistoryCell`; convert host transcript content into
  `TranscriptBlock` or `TranscriptLine` before entering the crate.
- Do not expose Codex `PermissionProfile`; pass already formatted labels and
  neutral capability flags.
- Do not expose model-provider runtime objects; pass provider display name,
  model display name, reasoning label, and feature-capability booleans.

## Host Action Contract

The reducer should produce actions that describe intent, not implementation:

```rust
pub enum RedesignAction {
    Redraw,
    SubmitComposer,
    ClearComposerForInterrupt,
    InterruptOrQuit,
    OpenHelp,
    OpenCommands,
    OpenModelPicker,
    StartNewChat,
    CloseChat(ChatId),
    SelectChat(ChatId),
    TogglePlanWindow,
    ClosePlanWindow,
    ToggleTerminalWindow,
    CloseTerminalWindow,
    ToggleFinalOnlyTranscript,
    ScrollTranscript(ScrollIntent),
    SelectTerminalSession(TerminalSelectionIntent),
}
```

The host adapter decides how each action maps to provider-specific behavior.
For Codex, `StartNewChat` continues to use the non-blocking path in
`tui/src/app/redesign_chat_start.rs`; `CloseChat` continues to unsubscribe and
scrub local state through `tui/src/app/redesign_chat_close.rs`.

## Data Flow

1. Host receives provider/runtime events.
2. Host updates its provider-specific session state.
3. Host adapter builds `RedesignViewModel`.
4. `codex-tui-redesign` renders the view model into the terminal frame.
5. User input enters the neutral reducer.
6. Reducer emits `RedesignAction`.
7. Host adapter executes the action using provider-specific runtime APIs.
8. Host schedules redraw and repeats.

## Extraction Sequence

1. Stabilize the neutral data model in `tui-redesign/src/state.rs`.
2. Move production layout and rendering modules into `tui-redesign/src/`.
3. Add a Codex adapter module in `tui/src/redesign_host/`.
4. Replace direct `render_app(area, buf, app)` rendering with
   `render_app(area, buf, &view_model)`.
5. Move neutral keyboard and mouse interpretation into the extracted crate.
6. Convert `tui/src/app/input.rs` to execute emitted actions.
7. Retire duplicated sandbox-only state once production rendering uses the crate.

## Compatibility Rules

- Preserve current Codex redesign behavior unless a story explicitly changes it.
- Keep global quit and clear-first Ctrl-C routing at the same priority as today.
- Preserve the terminal window accordion behavior.
- Preserve right-rail fallback behavior: if the right rail is hidden, system and
  tool-status content stays inline in the transcript.
- Preserve plan-window behavior for both `update_plan` checklist snapshots and
  proposed-plan markdown.

## Testing Strategy

- `codex-tui-redesign` unit tests cover reducer action emission.
- `codex-tui-redesign` snapshot tests cover neutral rendering.
- `codex-tui` adapter tests cover Codex state-to-view-model mapping.
- Existing `codex-tui` input tests continue to cover app-level behavior.
- Focused commands should be used during implementation:
  - `cargo test -p codex-tui-redesign`
  - `cargo test -p codex-tui redesign_chrome::tests`
  - `cargo test -p codex-tui input::tests`

## Key Risks

- The renderer may accidentally pull Codex types into the extracted crate.
- Transcript conversion may lose streaming, system-cell, or indentation
  semantics.
- Input reducer extraction can regress shortcut priority.
- Bottom-pane fallback can keep too much Codex UI embedded in the neutral crate.
- Provider-independent naming can become too generic and obscure Codex behavior.

The mitigation is to preserve Codex as the reference host, require snapshot
parity before deleting old code, and enforce dependency boundaries in
`Cargo.toml` review.
