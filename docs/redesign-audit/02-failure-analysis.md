# Failure analysis

## Termius symptom: most likely causal chain

1. Redesigned Codex enters a full alternate screen and enables all-purpose mouse
   tracking.
2. xterm-compatible terminals reserve most pointer events for the application
   while mouse tracking is active. Desktop xterm commonly offers Shift as a
   temporary selection override; a phone has no equivalent natural gesture.
3. Termius documents long-press-and-drag as an arrow-key gesture and a separate
   Paste input mode.
4. Codex discards the press/drag events it requested, so they cannot focus the
   composer, place the cursor, select transcript text, or open an app paste path.
5. If Android/Termius enters line selection anyway, the selected physical row
   contains left navigation, rich bubbles, transcript scrollbar, and sometimes
   a right rail. The terminal has no semantic pane boundary with which to omit
   that chrome. Live redraws can also make the selected cells unstable.
6. No paste event reaches the composer, so the otherwise-correct paste pipeline
   never runs.

Steps 1, 2, 3, 4, and 6 are confirmed. The exact selection expansion behavior in
step 5 is a strong inference and needs device capture.

## Ranked findings

| ID | Priority | Finding | Confidence |
|---|---:|---|---|
| T-01 | P0 | Broad mouse capture takes native selection/touch ownership although Codex uses only wheel events. | Confirmed |
| T-02 | P0 | Redesigned mode ignores `tui.alternate_screen`; only the CLI flag escapes full-screen entry. | Confirmed |
| T-03 | P0 | Early error/panic cleanup does not leave alternate screen or disable alternate scroll. | Confirmed |
| L-01 | P0 | A full frame cannot show an editable composer below eight rows, exactly when a mobile keyboard reduces height. | Confirmed |
| C-01 | P1 | “Raw, copy-friendly” mode is not consumed by the redesign renderer's finalized-cell path. | Confirmed |
| C-02 | P1 | Large-paste display text is expanded while its cursor index remains in placeholder coordinates. | Confirmed |
| C-03 | P1 | Native line selection in the rich view can copy sidebars, rails, labels, and bubble borders from the same physical row. | Confirmed layout limitation and reported behavior |
| I-01 | P1 | Wheel routing ignores pointer coordinates and refuses transcript scroll whenever a draft is non-empty. | Confirmed |
| R-01 | P1 | Working state requests another frame every 32 ms, adding SSH traffic and selection churn. | Confirmed cadence; selection effect needs device validation |
| A-01 | P1 | Terminal mode transitions are duplicated; future policy changes can be lost on `Ctrl+Z`/`fg` or editor return. | Confirmed |
| A-02 | P1 | Redesign and legacy composer/popup renderers alternate over one input engine. | Confirmed |
| K-01 | P1 | Footer/sidebar hints and redesign shortcuts are hard-coded instead of derived from the runtime keymap/capabilities. | Confirmed |
| P-01 | P1 | Text paste, image paste, remote clipboard, and client clipboard are presented as if they were one capability. | Confirmed |
| H-01 | P1 | Thread switching emits a visible-screen and scrollback purge without checking alternate-screen state. | Confirmed |
| S-01 | P2 | The sandbox duplicates state/rendering but is not production, so fixes and tests can land in the wrong place. | Confirmed |

## T-01: overbroad mouse ownership

`EnableMouseCapture` enables press/release, button-motion, and all-motion reports.
Codex then filters everything except wheel events. This is the worst trade-off
for touch devices: selection is disrupted, input bytes cross SSH needlessly,
and no tap affordance is gained.

There is no xterm “wheel only” tracking mode. The safer default is no mouse
tracking. Alternate-scroll mode can still translate wheel gestures to cursor
keys on compatible alternate screens; keyboard scrolling remains available.

## T-02: configuration contract violation

The config type promises `auto`, `always`, and `never`
([source](../../codex-rs/config/src/types.rs#L711-L717)). Redesigned startup
branches around that value. A user who already selected `never` to preserve
scrollback and selection gets different behavior merely by enabling the
redesign.

## T-03: terminal restoration is incomplete

Normal leave disables mouse/alternate scroll and leaves the alternate screen,
but generic exit and panic restoration do not. Bootstrap and startup-hook calls
can return after alternate-screen entry and before normal leave. Normal leave
also swallows transition errors and marks the screen inactive anyway
([source](../../codex-rs/tui/src/tui.rs#L771-L790)). Earlier, raw/bracketed/focus
modes are enabled and a fallible clear runs before `TerminalRestoreGuard` exists
([source](../../codex-rs/tui/src/lib.rs#L1367-L1375)). The lifecycle is therefore
neither transactional nor accurately tracked after partial failure.

## L-01: composer starvation on mobile resize

The layout reserves two footer rows, two header rows, and one separator row
before allocating the chat body
([source](../../codex-rs/tui/src/redesign_chrome/layout.rs#L27-L85)). The redesign
composer requires two chrome rows plus one input row. Therefore:

| Height | Effective result |
|---:|---|
| 4 | Header + footer; no composer |
| 5 | Header + separator + footer; no composer |
| 6-7 | Composer rectangle exists but has no input row |
| 8 | First height with one editable composer row |

The `44x4` snapshot covers the composer widget in isolation, not the full-frame
layout ([fixture](../../codex-rs/tui/src/redesign_chrome.rs#L2173-L2176)). It does
not prove the documented “composer survival” behavior.

## C-01: raw mode does not provide a plain redesign transcript

`ChatWidget` defines raw mode through `HistoryRenderMode::Raw`
([source](../../codex-rs/tui/src/chatwidget.rs#L1668-L1697)). The redesign's
`cell_content_lines()` instead calls `raw_lines()` only for user cells and rich
`display_lines(width)` for other finalized cells
([source](../../codex-rs/tui/src/redesign_chrome.rs#L1450-L1461)). Rich bubbles
and decorations remain, so the advertised selection contract is not met.

## C-02: large-paste cursor coordinates diverge

Large pastes are intentionally stored behind a short atomic placeholder. The
redesign renders `current_text_with_pending()`, which expands that placeholder,
but pairs it with the textarea cursor offset. The cursor can appear near the
start of a large expanded paste, and the expanded text can consume the viewport.
The view model must provide display text and a cursor mapped into that same text.

## C-03: rich rows have no semantic clipboard boundary

At wide widths, layout assigns 24 columns to the left navigation and may assign
30 columns to the right activity rail
([source](../../codex-rs/tui/src/redesign_chrome/layout.rs#L3-L85)). The central
message also contains a speaker label, alignment padding, and box-drawing
borders. All three panes are written into one terminal cell buffer and share
physical rows.

When Termius chooses a whole line, Codex is not told which substring the user
intended and cannot post-process the phone clipboard. The result can include the
navigation label on the left, message decoration, the transcript scrollbar, and
activity text on the right. This is not fixable with different border glyphs.

The redesign needs two semantic alternatives:

1. **Copy message** copies canonical message content, never rendered buffer
   cells.
2. **Focus message for partial copy** opens a stable, undecorated, single-pane
   view containing only that message. Native substring selection then has no
   adjacent Codex pane to cross into.

Arbitrary partial selection directly inside the rich multi-pane view remains
best-effort because terminal protocols do not expose a DOM-like selection range
or clipboard rewrite hook to the remote application.

## I-01: pointer events are not spatial

`handle_mouse_event()` chooses transcript or terminal-window scrolling globally.
It does not test whether the event occurred over the transcript, sidebar,
composer, footer, or overlay
([source](../../codex-rs/tui/src/app/input.rs#L415-L427)). Both scroll handlers
ignore `row` and `column`, and both return early when composer text exists
([source](../../codex-rs/tui/src/app/input.rs#L751-L813)).

## R-01: redraws compete with selection

While a task runs, redesign schedules a frame after 32 ms on every render
([source](../../codex-rs/tui/src/chatwidget.rs#L2098-L2114)). The spinner itself
changes only every 80 ms. This wastes work even locally; over SSH it adds network
and battery cost. Any terminal selection that is sensitive to cell updates is
more likely to shift or clear.

## A-02 and S-01: multiple sources of UI truth

The production renderer, legacy bottom pane, and sandbox each have different
layout, focus, cursor, paste, and overlay assumptions. The sandbox live loop only
matches key events and has no explicit `Event::Paste` path
([source](../../codex-rs/tui-redesign/src/runtime.rs#L27-L43)). Promoting it
without first replacing its demo contract would move drift, not remove it. Its
raw-mode guard is also created only after fallible terminal setup, and its cursor
math counts Unicode scalar values rather than displayed terminal columns
([runtime](../../codex-rs/tui-redesign/src/runtime.rs#L50-L60),
[cursor](../../codex-rs/tui-redesign/src/state.rs#L544-L557)). It is not yet a
safe lifecycle or Unicode-input reference implementation.

## H-01: full-screen thread switching may purge main scrollback

Every thread switch calls a helper that writes `CSI 3J` along with a full screen
clear ([caller](../../codex-rs/tui/src/app/session_lifecycle.rs#L467-L486),
[sequence](../../codex-rs/tui/src/custom_terminal.rs#L535-L550)). It does not use
the existing alternate-screen-aware clear path. Terminal behavior differs, so a
full-screen chat switch should never request a main scrollback purge.

## What Codex cannot fix alone

- It cannot control which Android actions appear in Termius's native menu.
- It cannot securely read the phone clipboard from a remote Linux process.
- It cannot guarantee OSC 52 clipboard support if the client disables it.
- It cannot make long-press mean Paste when the terminal client reserves that
  gesture for arrows.

The product obligation is to avoid blocking the client, expose capability-aware
alternatives, and document the remaining client gesture clearly.
