# Product Backlog

## Epic 1: Provider-Independent UI Contract

Goal: define the neutral types and boundaries that let any host drive the
redesign UI.

### Story 1.1: Neutral View Model

As a provider host, I want to pass one complete `RedesignViewModel` to the TUI
crate so that the renderer does not need provider-specific state.

Priority: P0

Acceptance criteria:

- `codex-tui-redesign` exposes provider-neutral view-model structs.
- The structs do not import Codex runtime, app-server, protocol, or model
  provider crates.
- The model can represent top chrome, sidebar chats, transcript blocks, system
  rail, composer, queued messages, plan window, and terminal sessions.

### Story 1.2: Neutral Actions

As a provider host, I want user input to produce neutral actions so that each
host can decide how to execute them.

Priority: P0

Acceptance criteria:

- The extracted crate exposes `RedesignAction`.
- Input reducer tests cover shortcuts for help, commands, model picker, new
  chat, close chat, plan window, terminal window, final-only transcript,
  transcript scroll, and terminal session selection.
- The reducer does not call Codex app methods.

### Story 1.3: Dependency Guard

As a maintainer, I want the extracted crate to reject provider-specific
dependencies so that provider independence does not drift.

Priority: P0

Acceptance criteria:

- `tui-redesign/Cargo.toml` depends only on UI and utility crates needed for
  rendering and input interpretation.
- Review checklist explicitly rejects dependencies on `codex-core`,
  `codex-protocol`, `codex-app-server-*`, `codex-model-provider`, and
  `codex-tui`.

## Epic 2: Renderer Extraction

Goal: move shipped redesign rendering into the extracted crate.

### Story 2.1: Layout and Theme Extraction

As a host, I want layout and theme logic to live in the reusable crate so that
all hosts get the same visual behavior.

Priority: P0

Acceptance criteria:

- Layout logic for tiny, narrow, standard, and wide terminal sizes lives in
  `codex-tui-redesign`.
- The extracted renderer preserves terminal-friendly background behavior.
- Snapshot coverage exists in `codex-tui-redesign`.

### Story 2.2: Transcript Rendering Extraction

As a user, I want transcript rendering to behave the same after extraction so
that provider independence does not break readability.

Priority: P0

Acceptance criteria:

- User, assistant, system, reasoning, tool, and final-output blocks are
  represented by neutral transcript data.
- Speaker labels are view-model fields, not hardcoded to Codex.
- System/tool content remains inline when the right rail is not available.
- Existing Codex snapshots have matching output or reviewed intentional diffs.

### Story 2.3: Plan and Terminal Windows

As a user, I want plan and terminal windows to keep working after extraction so
that high-value redesign workflows survive the refactor.

Priority: P0

Acceptance criteria:

- Plan-window rendering accepts neutral plan display lines or structured plan
  items.
- Terminal-window rendering accepts neutral terminal session summaries.
- Accordion behavior remains keyboard navigable.

## Epic 3: Codex Host Adapter

Goal: keep Codex as the reference host while removing UI knowledge from Codex
runtime modules.

### Story 3.1: Codex View-Model Adapter

As Codex, I want to map `App` and `ChatWidget` into the neutral view model so
that production mode can use the extracted renderer.

Priority: P0

Acceptance criteria:

- `tui/src/redesign_host/` contains Codex adapter code.
- `tui/src/redesign_chrome.rs` no longer reads `App` and `ChatWidget` directly
  for renderer-owned state.
- Adapter tests cover model labels, reasoning labels, context remaining,
  sidebar entries, queued messages, plan window content, and terminal sessions.

### Story 3.2: Codex Action Executor

As Codex, I want neutral actions mapped to current Codex behavior so that the
refactor does not regress user workflows.

Priority: P0

Acceptance criteria:

- `StartNewChat` uses the existing non-blocking startup path.
- `CloseChat` preserves unsubscribe and local-state cleanup.
- Plan and terminal windows remain per-chat state.
- Global Ctrl-C and clear-first composer behavior keep their current priority.

## Epic 4: Migration Safety

Goal: make the extraction reviewable and reversible in small increments.

### Story 4.1: Snapshot Parity

As a reviewer, I want focused snapshot parity so that visual regressions are
obvious.

Priority: P0

Acceptance criteria:

- Neutral renderer snapshots cover `44x4`, `72x18`, `100x24`, and `132x24`
  viewports.
- Codex adapter snapshots or existing Codex snapshots verify host parity.
- Snapshot diffs are reviewed before old renderer code is removed.

### Story 4.2: Old Path Retirement

As a maintainer, I want old direct-render paths removed only after parity so
that the migration stays safe.

Priority: P1

Acceptance criteria:

- No duplicate production renderer remains after extraction.
- `tui-redesign/README.md` is updated from sandbox wording to production crate
  wording.
- Any remaining compatibility shim has a documented owner and removal condition.

## Epic 5: Future Provider Host Enablement

Goal: make the first non-Codex integration straightforward.

### Story 5.1: Host Adapter Guide

As an engineer adding another provider, I want a host adapter guide so that I
can implement the minimum required integration.

Priority: P1

Acceptance criteria:

- Documentation lists required host responsibilities.
- Documentation lists each `RedesignAction` and expected host behavior.
- Documentation includes an example host loop using pseudocode.

### Story 5.2: Mock Host Fixture

As a maintainer, I want a mock provider host fixture so that provider-neutral
behavior can be tested without Codex.

Priority: P2

Acceptance criteria:

- `codex-tui-redesign` has test fixtures that do not use Codex state.
- The fixture can render idle, running, approval-needed, queued-message,
  plan-window, and terminal-window states.
