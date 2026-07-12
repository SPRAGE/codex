# Recommended decisions

## D1 — native selection wins by default

**Decision:** Default redesigned mouse capture to off. Add an explicit enum
preference only if a complete pointer mode is retained.

**Why:** Current code requests five mouse modes to consume one event family. The
lost mobile selection/paste affordance is more important than direct wheel
events; alternate scroll and keyboard navigation remain available.

**Rejected:** Detect Termius from `$TERM`. The client identity is unreliable
through SSH, tmux, and Mosh.

## D2 — screen buffer and mouse are separate policies

**Decision:** Honor existing `tui.alternate_screen` in redesigned mode and never
make alternate-screen entry imply mouse capture.

**Why:** Full-screen rendering, scrollback preservation, wheel input, and pointer
capture are separate user choices and terminal capabilities.

## D3 — fix behavior before extraction parity

**Decision:** Stage the P0 interaction fixes before promoting
`codex-tui-redesign`.

**Why:** The existing extraction plan treats current UX as a baseline. Known
terminal ownership and tiny-layout defects must not become public neutral-crate
contracts.

## D4 — retain the mature composer engine, remove duplicate presentation

**Decision:** Adapt the existing composer document/paste engine into a neutral
view model before replacing it.

**Why:** Its sanitization, large-paste placeholders, attachments, history, IME,
and paste-burst behavior are substantial. The bug is mostly before that engine.
Rewriting it with the renderer would increase risk.

## D5 — plain transcript is a first-class surface

**Decision:** Implement copy mode as a stable, undecorated transcript view, not a
style toggle on rich bubbles.

**Why:** Borders, gutters, inferred speaker labels, rewrapping, scrollbars, and
animation all contaminate terminal selection. Plain and rich views should share
structured source blocks.

## D6 — composer/action surface gets the last row

**Decision:** In constrained height, remove footer, metadata, header, rails, and
borders before reducing the active interaction below one usable row.

**Why:** Mobile keyboards and split panes routinely create tiny heights. A status
panel without an input/action surface is not a functioning application.

## D7 — visible shortcuts are data

**Decision:** Render shortcut hints from the resolved keymap and capability set.

**Why:** Hard-coded Alt bindings are difficult on mobile and can lie after user
reconfiguration. Every action also needs a command-palette route.

## D8 — one terminal transition owner

**Decision:** Centralize terminal mode application/restoration behind an
idempotent controller and make job control restore its resolved state.

**Why:** Editing only `enter_alt_screen()` would leave suspend/resume re-enabling
the old hard-coded mouse policy. The controller must also unwind alternate
scroll/screen on early errors and never report a failed transition as complete.

## D9 — client-specific guidance, client-neutral implementation

**Decision:** Document Termius Paste mode and small-screen shortcuts, but keep the
runtime policy conservative and terminal-neutral.

**Why:** Codex should work with Termius without baking a vendor name into event
logic. Device-specific behavior belongs in validation and help copy, not core
state transitions.

## D10 — retire the sandbox only after it runs production code

**Decision:** Convert the preview into a neutral fixture host, then delete its
parallel renderer/state.

**Why:** A visually similar sandbox is a liability if production bugs can be
fixed in the wrong codebase. A preview is valuable only when it exercises the
same renderer and reducer that ship.

## D11 — semantic copy, isolated partial selection

**Decision:** Copy whole messages from canonical content. For exact partial
selection, provide a stable message-only surface with no adjacent panes or
decorative chrome.

**Why:** A terminal exposes a grid, not semantic pane boundaries. If Termius
selects a physical line, the remote process cannot tell it to omit the left
navigation or right activity rail from that line.

**Rejected:** Reading selected cells back from the Ratatui buffer or repairing
the clipboard after native selection. Rendered cells have already mixed content
with presentation, and the remote Codex process neither receives the client's
selection range nor owns the phone clipboard.
