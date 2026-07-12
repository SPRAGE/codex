# Redesigned TUI audit

**Audit date:** 2026-07-12

**Scope:** production redesigned TUI, its terminal/input runtime, the standalone
`tui-redesign` sandbox, and the existing extraction plans.

## Conclusion

The Android/Termius paste problem is not primarily a composer text bug. It is an
ownership bug at the terminal boundary.

Redesigned mode enters the alternate screen and enables Crossterm's broad mouse
capture. That capture asks the terminal to report presses, releases, drags, and
all pointer motion, while Codex forwards only wheel events and discards the
rest. Codex therefore takes native selection/touch ownership without providing
equivalent selection, focus, or paste behavior.

Copying from the rich chat view has a related structural failure. On wide
layouts, side navigation, message bubbles, the transcript scrollbar, and the
right activity rail occupy the same physical terminal rows. Native terminal
selection understands rows and cells, not Codex's semantic panes. When Termius
promotes a gesture to line selection, the copied range can therefore include
the sidebars and bubble chrome along with the requested message fragment.

Termius adds a client-specific constraint: its current mobile guidance assigns
long-press-and-drag to arrow-key gestures and provides a separate **Paste mode**
above the shortcut bar. Codex cannot force Android to show a Paste menu or read
the phone clipboard from a remote SSH process. It can, however, stop interfering
with the terminal client's native input path.

The highest-priority changes are:

1. Disable mouse capture by default and decouple it from alternate-screen use.
2. Make terminal restoration transactional; current early-error and panic paths
   can leave the alternate screen or alternate-scroll mode active.
3. Make redesigned mode honor `tui.alternate_screen` instead of overriding it.
4. Prioritize the active input surface when the mobile keyboard leaves only a
   few terminal rows.
5. Add semantic message-copy and a message-only partial-selection surface;
   make copy mode genuinely plain and selection-safe.
6. Centralize terminal-mode transitions so suspend/resume and external-editor
   handoffs cannot silently restore a different policy.

## Implementation drive status

The current working tree implements the first terminal-safety slice:

- Redesigned mode disables mouse capture while legacy alternate-screen callers
  retain their existing capture policy.
- `tui.alternate_screen` now applies consistently to redesigned and legacy UI,
  with `--no-alt-screen` retaining precedence.
- Defensive restoration now attempts mouse, alternate-scroll, and
  alternate-screen cleanup and preserves failed-leave state for another retry.
- The restore guard now covers terminal initialization and the initial clear.

Byte-level policy and partial-failure tests pass. This slice is not complete
until it has passed the Termius/Android matrix; the transactional terminal
controller, responsive layout, and semantic partial-copy surface remain later
roadmap stages.

## Workarounds before a released fix

- Start the redesign with `codex --redesign-tui --no-alt-screen`. This prevents
  the current alternate-screen entry path from enabling mouse capture. It is a
  workaround, not a complete responsive-layout fix.
- In Termius on Android, use the terminal's **Paste mode** above the shortcut
  bar instead of relying on long-press. Termius documents long-press-and-drag as
  an arrow-key gesture.
- Set `animations = false` under `[tui]` in `~/.codex/config.toml` if selection
  moves or clears while Codex is working.
- Use `/copy` or the default `Ctrl+O` binding to copy the last response when the
  client permits terminal-mediated clipboard writes (OSC 52). This avoids
  selecting decorated transcript cells.
- For a partial excerpt today, copy the whole response and select the excerpt in
  a local editor or note. The rich split-pane view cannot reliably exclude its
  sidebars from client-owned line selection.

## Document map

- [Current architecture](01-current-architecture.md)
- [Failure analysis](02-failure-analysis.md)
- [Target UX and architecture](03-target-ux-and-architecture.md)
- [Migration roadmap](04-migration-roadmap.md)
- [Test and validation plan](05-test-and-validation-plan.md)
- [Recommended decisions](06-recommended-decisions.md)

## Confidence labels

- **Confirmed:** directly established by current source or an authoritative
  terminal/client reference.
- **Strong inference:** the code and terminal protocol explain the symptom, but
  the exact Android build has not been instrumented here.
- **Needs device validation:** must be reproduced on Termius/Android before the
  implementation is declared complete.

## External references

- [Termius: mobile AI-agent tips](https://termius.com/blog/8-tips-for-using-ai-agents-on-mobile-in-termius)
- [Crossterm `EnableMouseCapture` source](https://docs.rs/crossterm/latest/src/crossterm/event.rs.html#313-346)
- [xterm mouse protocol and selection override](https://invisible-island.net/xterm/manpage/xterm.html#Mouse-Protocol)
- [xterm control sequences](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html)
