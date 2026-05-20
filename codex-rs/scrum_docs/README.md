# Provider-Independent Redesign TUI Scrum Docs

This folder defines the work to extract the production redesign TUI into a
provider-independent Rust TUI crate that can be hosted by Codex or by another AI
provider integration.

## Document Map

- `product-brief.md` - product goal, scope, non-goals, success metrics, and
  assumptions.
- `architecture.md` - target crate boundaries, host contract, state model,
  event/action flow, and extraction sequence.
- `product-backlog.md` - epics and user stories with priorities and acceptance
  criteria.
- `sprint-plan.md` - ordered sprints with deliverables, review gates, and test
  commands.
- `implementation-plan.md` - agent-ready engineering handoff with concrete file
  ownership and task steps.
- `definition-of-done.md` - release gates, test expectations, review checklist,
  and risk controls.

## Current Source Anchors

The extraction plan is grounded in the current repo shape:

- `tui/src/redesign_chrome.rs` is the shipped redesign renderer.
- `tui/src/app/input.rs` owns redesign shortcut routing and input behavior.
- `tui/src/app/redesign_chat_start.rs` and
  `tui/src/app/redesign_chat_close.rs` own chat lifecycle behavior exposed in
  the redesign.
- `tui/src/chatwidget.rs` exposes the current redesign data adapter methods for
  model labels, transcript lines, composer state, queued messages, plan window
  content, and background terminal output.
- `tui-redesign/` is already a lightweight sandbox crate and is the best seed
  for a provider-independent UI crate.
- `model-provider-info/` and `model-provider/` already own provider metadata and
  runtime provider behavior. The extracted TUI should consume provider labels
  and capabilities from a host adapter, not become a provider runtime itself.

## Working Rule

Keep the extracted TUI provider-independent by making it depend on neutral
view-models and host actions only. Provider-specific networking, auth, model
catalogs, app-server sessions, tool execution, and persistence stay in the host
application.
