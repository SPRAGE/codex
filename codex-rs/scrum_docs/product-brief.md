# Product Brief: Provider-Independent Redesign TUI

## Goal

Extract the production redesign TUI into a reusable Rust crate that renders and
routes terminal UI behavior through provider-neutral state and action contracts.
Codex remains the first host implementation, but the same crate must be usable
by another provider host without importing Codex runtime, app-server, core, or
protocol crates.

## Problem

The current redesign is visually close to the desired product surface, but its
production implementation is coupled to Codex application state. Rendering reads
directly from `App` and `ChatWidget`; input handling directly mutates Codex app
state and calls Codex-specific popups, thread lifecycle, model selection, and
terminal surfaces. This makes it difficult to reuse the TUI with a different AI
provider or a different runtime orchestration layer.

## Users

- Primary user: a local developer who wants one high-quality terminal UI for
  multiple AI runtimes.
- Secondary user: an engineer adding a new provider host who wants to implement
  an adapter instead of forking UI code.
- Reviewer: a maintainer who needs small, testable changes with clear crate
  boundaries.

## Scope

In scope:

- Promote `tui-redesign` into the provider-independent UI crate.
- Move production redesign layout, sidebar, transcript, composer, plan-window,
  and terminal-window rendering behind neutral view-models.
- Move redesign input interpretation into a neutral reducer that emits host
  actions instead of mutating Codex state directly.
- Add a Codex host adapter in `tui/` that maps Codex `App` and `ChatWidget`
  state into the neutral view-models and maps neutral actions back into current
  Codex behavior.
- Preserve current production behavior for Codex redesign mode.
- Keep snapshot coverage for the current viewport classes: tiny, narrow,
  standard, and wide.

Out of scope for this extraction:

- Replacing Codex provider runtime or app-server behavior.
- Adding a new non-Codex provider implementation in the same change.
- Rewriting the legacy bottom pane.
- Changing model-provider auth, networking, or protocol behavior.
- Publishing the extracted crate outside this workspace.

## Success Metrics

- `codex-tui-redesign` can compile without depending on `codex-tui`,
  `codex-core`, `codex-protocol`, `codex-app-server-*`, or `codex-model-provider`.
- The production Codex redesign path renders through the extracted crate.
- Redesign shortcut tests still cover global quit, clear-first Ctrl-C behavior,
  sidebar navigation, plan window toggling, terminal window toggling, transcript
  scroll, and composer cursor behavior.
- Snapshot coverage exists in the extracted crate for layout rendering, and in
  `codex-tui` for Codex host adapter behavior.
- Adding a new provider host requires implementing the host adapter contract and
  does not require copying or modifying the core renderer.

## Assumptions

- The first implementation should keep the crate inside this workspace.
- Provider independence means provider-neutral UI contracts, not runtime
  provider execution inside the UI crate.
- Codex remains the reference host until another provider host is implemented.
- Current UX behavior is the compatibility baseline unless a sprint explicitly
  states otherwise.
