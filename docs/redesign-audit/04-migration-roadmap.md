# Migration roadmap

## Sequencing rule

Fix terminal ownership before extracting the renderer. Otherwise the neutral
crate and reducer will preserve known-bad behavior as compatibility requirements.
Keep each complex stage under roughly 500 changed lines and every review under
800 lines; do not combine renderer extraction with terminal lifecycle refactoring.

## Stage 0: immediate safety fixes

### PR 0.1 — stop unused mouse capture

- Remove unconditional `EnableMouseCapture` from alternate-screen entry.
- Keep bracketed paste independent.
- Keep keyboard scroll and evaluate alternate-scroll wheel translation.
- Add byte-level tests proving default entry emits no 1000/1002/1003 enable
  sequence and exit remains defensive.

**Exit:** Termius Paste mode inserts text; native selection is not reserved by
Codex; desktop transcript scrolling remains keyboard accessible.

### PR 0.2 — close unsafe terminal cleanup paths

- Disable alternate scroll and leave alternate screen during defensive restore.
- Ensure errors after alternate-screen entry pass through cleanup.
- Do not mark a terminal mode inactive when its disable transition failed.
- Use an alternate-screen-aware clear on chat switching; never emit `CSI 3J`
  from the active full-screen surface.

**Exit:** failure injection after each startup transition restores the parent
terminal and preserves main scrollback.

### PR 0.3 — honor screen-buffer configuration

- Resolve redesigned and legacy screen-buffer policy through one function.
- Make `tui.alternate_screen = "never"` work in redesigned mode.
- Keep `--no-alt-screen` as the highest-priority override.

**Exit:** config/CLI table tests cover every combination.

### PR 0.4 — make tiny height actually usable

- Replace fixed vertical subtraction with an interaction-first budget.
- Add full-frame snapshots at `44x4`, `44x6`, and `44x8`.
- Test the resize sequence `100x24 -> 44x4 -> 100x24`.

**Exit:** the cursor and at least one input row remain visible at four rows.

### PR 0.5 — reduce selection churn

- Request frames at actual indicator boundaries rather than every 32 ms.
- Add static/reduced behavior for copy mode and remote profiles.
- Measure bytes and frames during a 30-second running state.

### PR 0.6 — add semantic message-copy surfaces

- Add `CopyMessage` using canonical message blocks rather than rendered lines.
- Add a message-only partial-copy view with no side navigation, right rail,
  labels, bubble borders, scrollbar, footer, or animation.
- Make previous/next-message and enter/exit-copy-view available through the
  command palette as well as the resolved keymap.
- Keep native selection inside the rich multi-pane view explicitly best-effort.

**Exit:** copying a whole message or selecting a known substring from the
message-only view contains no Codex chrome, including at widths where both side
rails would normally be visible.

## Stage 1: centralize terminal lifecycle

- Add `TerminalPolicy` and `TerminalModeController`.
- Represent screen buffer, alternate scroll, mouse capture, bracketed paste,
  focus reporting, keyboard enhancement, cursor, and motion explicitly.
- Route enter/leave, restore, `Ctrl+Z`/`fg`, panic cleanup, and editor handoff
  through the controller.
- Add effective-mode diagnostics.

**Exit:** every lifecycle transition is idempotent and suspend/resume restores the
same resolved policy.

## Stage 2: define the neutral UI contract

- Revise the existing `scrum_docs` view-model/action proposal with terminal
  capabilities, one focus model, plain transcript, and `LayoutMap`.
- Add production-shaped neutral fixtures to `codex-tui-redesign`.
- Add a dependency guard against Codex runtime types.
- Create the Codex view-model adapter without changing production rendering yet.

**Exit:** the model can express all shipped states plus mobile/tiny/copy states.

## Stage 3: extract pure rendering

Move one surface per review:

1. layout and responsive budget;
2. transcript blocks and plain mode;
3. message focus and semantic copy serialization;
4. composer presentation;
5. sidebar/header/footer;
6. plan and terminal windows;
7. overlays and activity rail.

Keep the production wrapper until reviewed snapshot parity exists. Parity means
preserving intended behavior, not preserving the P0 defects fixed in Stage 0.

**Exit:** the renderer accepts only `RedesignViewModel`; it does not import
`App`, `ChatWidget`, `HistoryCell`, or `ThreadId`.

## Stage 4: extract input reduction

- Normalize terminal events before UI interpretation.
- Move focus, scroll, overlay, and sidebar navigation into an exhaustive reducer.
- Emit neutral `UiAction` values.
- Use `LayoutMap` for region-aware pointer events if mouse mode is explicitly
  enabled.
- Keep async chat start/close, app-server calls, model selection, and persistence
  in the Codex host executor.

**Exit:** keyboard and optional pointer paths produce the same intent actions;
no renderer or reducer mutates Codex runtime state.

## Stage 5: remove legacy visual fallbacks

- Inventory every condition in `redesign_should_render_bottom_pane()`.
- Reuse the mature composer document engine through an adapter.
- Replace slash, model, approval, request-user-input, and other popup visuals one
  flow at a time.
- Delete the second composer renderer only after cursor, paste, attachment, IME,
  and large-paste parity.

**Exit:** one focus tree and one visual layer own every redesign surface.

## Stage 6: retire sandbox drift

- Make the preview binary consume the production neutral view model and renderer.
- Delete demo-only rendering/state that no longer exercises production code.
- Convert useful sandbox scenarios into fixtures and snapshots.
- Update `tui-redesign/README.md` only when it genuinely becomes production
  infrastructure.

## Rollout and rollback

- Keep `--redesign-tui` gated through the migration.
- Add a temporary `terminal_interaction_v2` feature gate only if Stage 1 cannot
  land behavior-preservingly.
- Log the resolved mode locally at debug level: screen buffer, mouse capture,
  bracketed paste, alternate scroll, motion, SSH/tmux detection, and viewport.
- Roll back individual stages, not the entire redesign; the neutral adapter and
  old renderer should coexist only during a bounded parity window.
