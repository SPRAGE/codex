# Test and validation plan

## Mobile and selection release criteria

The mobile/selection fix is complete only when all are true:

1. Default redesigned startup emits no broad mouse-tracking enable sequence.
2. `tui.alternate_screen = "never"` works in redesigned mode.
3. A full `44x4` frame contains an editable composer or actionable approval.
4. Exact single-line, multiline, Unicode, and large text pastes reach the
   composer without duplication or loss.
5. Selecting a known substring from the message-only copy surface while Codex
   is working copies exactly that text—without sidebars, rails, labels, borders,
   or scrollbar cells—on the agreed Termius/Android test version.
6. Exit, panic, editor handoff, suspend, and resume leave the terminal usable.
7. Large-paste display text and cursor use one coordinate system.
8. Full-screen thread switching never emits a scrollback purge.

## Automated terminal-policy tests

Use a recording writer/backend to assert emitted bytes and controller state.

| Scenario | Required assertion |
|---|---|
| Default redesigned entry | Alternate-screen policy is honored; no 1000/1002/1003 mouse enable bytes. |
| Mouse `Always` opt-in | Enable bytes emitted once; complete disable bytes emitted on exit. |
| Default inline mode | No alternate-screen or alternate-scroll enable bytes; default mouse policy remains off. |
| External editor | Modes are released before launch and exact policy restored after. |
| `Ctrl+Z` then `fg` | Exact pre-suspend policy restored; no hard-coded capture. |
| Partial write failure | Controller reports/applies a recoverable state and cleanup remains safe. |
| Repeated enter/leave | Idempotent; no mode-stack drift. |
| Error after alt-screen entry | Unwinds alternate scroll and screen buffer before returning. |
| Full-screen thread switch | Clears only the active screen; never emits `CSI 3J`. |

Add event-normalizer tests for key press/repeat/release, bracketed paste, resize,
focus, every mouse kind, and ignored events. Do not merely test that wheel maps;
test that the policy never requests event classes the reducer cannot use.

## Layout tests

- Full-frame snapshots: `32x3`, `44x4`, `44x6`, `44x8`, `56x12`, `72x18`,
  `100x24`, and `132x24`.
- Dynamic sequences:
  - mobile keyboard open/close: `100x24 -> 44x6 -> 100x24`;
  - portrait/landscape: `44x18 -> 100x10 -> 44x18`;
  - tmux split/restore: `132x24 -> 72x18 -> 132x24`.
- Property checks for every width `1..=200` and height `1..=80`:
  rectangles remain in bounds, do not overlap illegally, and the active
  interaction surface wins its minimum size.
- Snapshot blocking approval/question, non-empty draft, queued input, slash
  popup, model popup, plain transcript, plan window, and terminal window in tiny
  and narrow modes.

## Input and paste tests

- Bracketed: empty, one line, multiline LF, CRLF, Unicode, combining marks,
  emoji, control-character sanitization, and payload over the placeholder limit.
- Non-bracketed burst: the same text classes at simulated SSH latency/jitter.
- Paste followed immediately by Enter; paste during a running turn; paste into a
  modal free-form field; paste after resize.
- Distinguish `PasteText`, `AttachClipboardImage`, and `AttachUploadedPath` in
  capability tests.
- Verify remote/SSH image clipboard action is hidden or gives a specific upload
  path instruction rather than a generic remote clipboard failure.
- Verify large-paste placeholder, display text, display cursor, submission text,
  and deletion all remain consistent in the redesign integration.
- Reducer model tests cover every focus state and action; compare whole state
  objects after each transition.

## Copy and selection tests

Automated tests can verify plain output and stable repaint behavior but cannot
fully emulate an Android selection menu.

- Plain transcript contains no bubble borders, scrollbars, spinner cells, ANSI
  style, or inferred labels.
- `CopyMessage` serializes canonical content and never reads text back from the
  rendered terminal buffer.
- At `132x24`, where both side regions normally exist, the message-only copy
  surface renders neither region on any message row.
- Exact selections across a wrapped paragraph, code block, table, and list do
  not gain navigation text, activity text, border glyphs, or artificial
  newlines from adjacent panes.
- `/copy` and configured copy binding use raw assistant markdown and respect the
  existing OSC 52 size cap.
- Working plain transcript generates no animation frames.
- Rich and plain views share the same source blocks and message ordering.

## Manual client matrix

Record client version, Android/iOS version, keyboard, transport, `$TERM`, tmux,
viewport, and effective Codex modes for every run.

| Client | Transport | Required flows |
|---|---|---|
| Termius Android | SSH | Paste mode, long-press gesture, exact selection, keyboard open/close, orientation change. |
| Termius Android | SSH + tmux | Same flows plus detach/reattach and scroll behavior. |
| Termius iOS | SSH | Paste mode/gesture differences and shortcut bar. |
| Termius desktop | SSH | Native selection, wheel, Shift override, OSC 52 copy. |
| xterm/WezTerm/Kitty/iTerm2 | local + SSH | Selection override, alternate scroll, raw transcript, resize. |
| Windows Terminal | local + SSH | Mode cleanup, selection, paste, resize. |

### Termius reproduction script

1. Start current and fixed builds in equivalent sessions.
2. Note the viewport before and after opening the Android keyboard.
3. Copy a known 20-character token from another Android app.
4. Try Termius Paste mode into an empty and a non-empty composer.
5. Long-press without dragging; then long-press and drag.
6. Use a wide layout with both side regions visible. Select the token
   `selection-0123456789` first in the rich view to record current client
   behavior, then in the message-only copy surface. Wait three seconds while
   Codex works and compare exact clipboard contents; the reliable surface must
   contain only the token.
7. Repeat with inline/alternate screen, animations on/off, and tmux on/off.
8. Capture a screen recording and Codex debug-mode summary; do not capture
   secrets or clipboard contents beyond the synthetic token.

## Performance gates

- Idle: zero scheduled redraws after the screen settles.
- Working rich view: no more than 5 visible animation updates per second.
- Working reduced/plain view: no more than 1 update per second, preferably event
  driven.
- Any-motion mouse reports: zero in the default profile.
- Compare bytes written over a 30-second SSH session before and after the fix.

## Repository checks during implementation

For each code stage, follow the repository instructions: focused `just test -p
codex-tui` or `just test -p codex-tui-redesign`, required snapshot review, `just
fix -p <crate>` for large changes, then `just fmt`. Ask before the complete
workspace test suite.
