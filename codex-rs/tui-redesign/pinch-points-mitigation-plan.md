# Redesigned TUI Pinch Points Mitigation Plan

## Goal

Make the redesigned TUI easier to trust and operate under real terminal
conditions: narrow panes, tmux, long sessions, approvals, multiple agent
threads, and keyboard-only use.

This plan targets the current redesign surfaces in:

- `tui/src/redesign_chrome.rs`
- `tui/src/app/input.rs`
- `tui/src/chatwidget.rs`
- `tui/src/bottom_pane/`
- `tui-redesign/`

## Guiding Principle

The redesign should behave like a calm workspace cockpit. The user should always
be able to answer three questions quickly:

1. What is Codex doing now?
2. Does Codex need input from me?
3. Where do I type or act next?

Everything else is secondary.

## Phase 1: Establish Information Hierarchy

### Pinch Point

The wide layout exposes many competing status surfaces: top bar, chat bar, right
rail, footer, sidebar, transcript, work indicator, and composer. This creates
status noise.

### Mitigation

Define three priority tiers:

- Primary: active work state, pending approval/input, composer.
- Secondary: model, context remaining, current thread, branch, files changed.
- Tertiary: token totals, shortcuts, system events, chat metadata.

Implementation direction:

- Pick one primary "what is happening now" surface.
- Avoid showing the same state in multiple high-visibility places.
- Make "needs user action" more prominent than "Codex is working."
- Keep detailed event/status history in the right rail or transcript overlay.

### Acceptance Criteria

- At `132x24`, current work state is visually obvious.
- At `72x18`, critical status is still visible.
- No high-priority status appears redundantly in more than two surfaces.

## Phase 2: Keyboard Ergonomics Audit

### Pinch Point

`Ctrl+B` is central to the redesigned sidebar, but it conflicts with the common
tmux prefix. Some actions also depend on function keys, which are unreliable in
some terminal setups.

### Mitigation

Audit all redesign-specific shortcuts:

- `Ctrl+B`
- `F1`
- `F2`
- `F4`
- `Ctrl+T`
- `Ctrl+R`
- `Ctrl+G`
- `N`
- `F`

Implementation direction:

- Decide whether sidebar focus remains `Ctrl+B` or moves to a less conflicted
  binding.
- Ensure every redesign action has a command-palette path.
- Make `?` behavior focus-aware: type `?` in the composer, open help in UI
  navigation contexts.
- Add redesigned TUI navigation to the help/keymap view.

### Recommended Decision

Keep `F2` for commands and `F4` for model selection. Reconsider `Ctrl+B`; if it
stays, provide an obvious alternate binding for tmux users.

### Acceptance Criteria

- Every redesign-only action is reachable without function keys.
- Help text matches actual behavior.
- Tests cover the redesign shortcut set.
- tmux users have a non-conflicting path.

## Phase 3: Approval UX Safety Pass

### Pinch Point

Approval actions are safety-critical, but compact labels such as `APV_SESS` can
make decisions less clear.

### Mitigation

Use plain action labels where possible:

- `Approve`
- `Approve for session`
- `Deny`

Implementation direction:

- Prefer wrapping approval action text over unsafe abbreviations.
- Keep command, reason, and consequence visible.
- Highlight destructive or broad approvals clearly.
- In narrow layouts, drop lower-priority metadata before shortening action
  language.

### Acceptance Criteria

- Approval UI is understandable without documentation.
- No approval action uses ambiguous abbreviations.
- Narrow approval snapshots exist.
- Deny remains visually distinct without overwhelming low-risk approvals.

## Phase 4: Narrow-Terminal Hardening

### Pinch Point

The redesigned TUI must work well in tmux splits and small panes, not only wide
terminal windows.

### Mitigation

Define supported viewport classes:

- Tiny: `44x4`
- Narrow: `72x18`
- Standard: `100x24`
- Wide: `132x24`

Expected behavior:

- Tiny: composer survival only.
- Narrow: transcript, composer, and minimal status.
- Standard: transcript plus optional sidebar.
- Wide: sidebar, transcript, and right rail.

Implementation direction:

- Add or expand snapshots for each viewport class.
- Add resize/reflow coverage for redesigned mode.
- Ensure composer row does not drift after split/restore cycles.
- Ensure scrollbars and borders do not consume too much content width in narrow
  mode.

### Acceptance Criteria

- Snapshots exist for all target viewport classes.
- Repeated resize tests pass with redesigned chrome enabled.
- Composer remains usable at tiny sizes.
- Critical "needs input" state remains visible in narrow mode.

## Phase 5: Legacy Bottom Pane Integration Cleanup

### Pinch Point

The redesign falls back to the legacy bottom pane for slash commands, popups,
and active bottom-pane views. This can feel like two TUIs stitched together.

### Mitigation

Inventory every condition in `redesign_should_render_bottom_pane()`.

For each condition, choose one outcome:

- Keep legacy rendering intentionally.
- Adapt legacy rendering to match redesign chrome.
- Replace with redesign-native rendering.

Priority flows:

- Slash command popup.
- Model popup.
- External editor handoff.
- Image and paste attachment states.

### Acceptance Criteria

- Slash command flow feels native to the redesigned TUI.
- Modal and popup cursor positioning remains correct.
- Snapshot coverage includes fallback states.
- No duplicated footer or composer chrome appears during fallback.

## Phase 6: Renderer Ownership Decision

### Pinch Point

There are two redesign renderers: production code under `tui/src/` and a
standalone sandbox under `tui-redesign/`. Without clear ownership, fixes can land
in the wrong place.

### Mitigation Options

Option A: Keep `tui-redesign` as a sandbox.

- Document that it is prototype-only.
- Keep production behavior in `tui/src/redesign_chrome.rs`.
- Port useful experiments manually.

Option B: Retire `tui-redesign`.

- Move useful fixtures/tests into `tui`.
- Delete the sandbox crate once production redesign stabilizes.

Option C: Extract shared primitives.

- Only do this if production and sandbox genuinely share reusable behavior.

### Recommended Decision

Use Option A short-term. Move to Option B once the production redesign is stable.
Avoid Option C unless duplication becomes painful and stable.

### Acceptance Criteria

- Contributors know which renderer is production.
- No production design bug is fixed only in the sandbox.
- CI/test ownership is clear.

## Phase 7: Split Rendering Modules

### Pinch Point

`tui/src/redesign_chrome.rs` owns too much: state/context types, layout,
sidebar, transcript, system rail, composer, and tests. It risks becoming another
high-touch orchestration file.

### Mitigation

Split the redesign renderer into focused modules before adding major features:

- `redesign_chrome/layout.rs`
- `redesign_chrome/sidebar.rs`
- `redesign_chrome/transcript.rs`
- `redesign_chrome/system_rail.rs`
- `redesign_chrome/composer.rs`
- `redesign_chrome/context.rs`

Keep the top-level module focused on orchestration.

### Acceptance Criteria

- Each module has local tests where practical.
- Snapshot tests remain close to rendering behavior.
- The top-level redesign module stays focused on composition.
- No new public crate API is exposed unless needed.

## Phase 8: Style and Theme Alignment

### Pinch Point

The redesign should follow `tui/styles.md`. Custom colors and hardcoded
foregrounds can render poorly across terminal themes.

### Mitigation

Implementation direction:

- Prefer default foreground, dim, cyan, green, red, and magenta.
- Avoid hardcoded white, yellow, and blue unless explicitly justified.
- Re-check the custom composer background against common terminal themes.
- Ensure focus state is visible without relying only on color.

### Acceptance Criteria

- No unnecessary hardcoded white/yellow/blue foregrounds.
- Focus state is visible through shape, symbol, or weight as well as color.
- Snapshots cover focused and unfocused sidebar, approval, running work, and
  narrow composer states.

## Phase 9: Critical Flow Validation

### Pinch Point

The redesign needs validation against real workflows, not just static layout.

### Critical Flows

1. Start a new chat.
2. Switch between multiple chats or agents.
3. Submit normal composer text.
4. Open slash commands.
5. Change model.
6. Review and answer an approval.
7. Scroll transcript while composer is empty.
8. Recover from a narrow tmux split.
9. Find help.
10. Exit safely while work is running.

### Mitigation

For each flow, record:

- Was focus obvious?
- Was the next key obvious?
- Was risk explained clearly?
- Did resize change the meaning of the screen?
- Did the user have a command-palette fallback?

### Acceptance Criteria

- Every critical flow has an automated test, snapshot, or manual TUI test note.
- Any flow requiring hidden knowledge gets a visible hint or command-palette
  route.

## Recommended Execution Order

1. Establish information hierarchy.
2. Audit and adjust keyboard ergonomics.
3. Improve approval clarity.
4. Harden narrow and resize behavior.
5. Clean up legacy bottom-pane integration.
6. Decide renderer ownership.
7. Split rendering modules.
8. Align style and theme usage.
9. Validate critical user flows.

## Risks

- Changing shortcuts can disrupt existing users. Mitigate with alternate
  bindings and command-palette access.
- Splitting modules too early can create churn. Mitigate by splitting only after
  hierarchy and flow decisions are stable.
- Narrow-mode fixes can regress wide-mode polish. Mitigate with snapshot
  coverage across all viewport classes.
- Keeping the sandbox too long can cause drift. Mitigate by documenting ownership
  and periodically pruning stale sandbox behavior.

## Definition of Done

The redesigned TUI is ready for broader use when:

- Work state, pending input, and composer location are obvious in standard and
  narrow terminals.
- Approval actions are clear and safe.
- All redesign actions have discoverable keyboard and command-palette paths.
- Resize/tmux behavior is covered and stable.
- Production renderer ownership is clear.
- Rendering code is split into maintainable modules.
- Snapshot coverage reflects intentional UI behavior across viewport classes.

## Implementation Status

Completed on 2026-05-12:

- Production redesigned chrome now exposes tmux-friendly, non-function-key paths
  for the redesign actions that previously depended on `Ctrl+B`, `F1`, `F2`, or
  `F4`.
- Footer, sidebar, tooltip, sandbox footer, and sandbox help copy now match the
  implemented shortcut behavior.
- Approval copy uses plain labels such as `APPROVE SESSION` in the sandbox
  snapshots.
- Narrow, standard, wide, focused-sidebar, slash-fallback, and wrapped-composer
  rendering are covered by snapshots or focused regression tests.
- Production renderer ownership is documented in `tui-redesign/README.md`.
- Production redesign layout and sidebar code are split into focused modules:
  `tui/src/redesign_chrome/layout.rs` and
  `tui/src/redesign_chrome/sidebar.rs`.
- Theme/style alignment removed unnecessary hardcoded bright foregrounds and the
  custom RGB composer input background.
- Status/title preview root detection no longer treats a missing cwd as an
  ancestor git repository, which keeps tests stable in environments where
  `/tmp` itself is a git repository.
- User-authored transcript indentation is preserved in redesigned message
  bubbles, so pasted code and nested text no longer lose leading spaces.
- System, reasoning, plan, and tool/status cells are routed into the transcript
  when the right rail is hidden, so narrow layouts no longer drop those
  messages.

Verification completed:

- `cargo test -p codex-tui redesign_chrome -- --nocapture`
- `cargo test -p codex-tui app::input::tests::redesign_ -- --nocapture`
- `cargo test -p codex-tui-redesign`
- `env RUST_MIN_STACK=33554432 cargo test -p codex-tui`
- `cargo clippy --fix --tests --allow-dirty -p codex-tui -p codex-tui-redesign`
- `git diff --check`

The full `codex-tui` suite needs the larger stack setting in this environment
for an existing app-server lookup test and needs unsandboxed local socket access
for IDE IPC tests.
