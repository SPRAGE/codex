# Current architecture

## Source of truth

The shipped redesign is not the `codex-tui-redesign` crate. The sandbox README
explicitly assigns production rendering to `tui/src/redesign_chrome.rs` and
production input to `tui/src/app/input.rs`
([source](../../codex-rs/tui-redesign/README.md#L3-L19)).

At audit time:

- `tui/src/redesign_chrome.rs` is 3,425 lines and mixes view-model adaptation,
  semantic transcript conversion, rendering, wrapping, and many tests.
- `tui/src/redesign_chrome/` contains layout, sidebar, composer, plan-window,
  terminal-window, and window-chrome helpers.
- `App` stores redesign state across fourteen fields rather than one coherent UI
  state object ([source](../../codex-rs/tui/src/app.rs#L604-L617)).
- `ChatWidget` remains the real owner of composer, approval, popup, paste,
  streaming, and terminal-process state.
- `tui-redesign/` holds a separate demo state, renderer, and live preview.

## Runtime flow

```text
Android clipboard / touch
        |
        v
Termius terminal emulator -- SSH/Mosh/tmux --> remote PTY
        ^                                      |
        | terminal control sequences           v
TerminalMode code <------------------------ Crossterm EventStream
                                                   |
                                  Key / Paste / wheel-only Mouse
                                                   |
                                                   v
                              App input routing and ChatWidget
                                                   |
                              state adapters + HistoryCell rendering
                                                   |
                                                   v
                                      redesign_chrome renderer
```

The important boundary is the first one: text paste exists only after the local
terminal sends clipboard text through the PTY. A remote Codex process cannot
directly read an Android clipboard.

## Startup and terminal ownership

1. `set_modes()` enables bracketed paste, raw mode, keyboard enhancement, and
   focus reporting ([source](../../codex-rs/tui/src/tui.rs#L178-L193)).
2. Redesigned mode computes alternate-screen use from `!no_alt_screen`, bypassing
   `config.tui_alternate_screen`
   ([source](../../codex-rs/tui/src/lib.rs#L1792-L1806)).
3. Redesigned startup enters the alternate screen
   ([source](../../codex-rs/tui/src/lib.rs#L1836-L1838)).
4. Alternate-screen entry enables alternate scroll and `EnableMouseCapture`
   ([source](../../codex-rs/tui/src/tui.rs#L739-L768)).
5. Crossterm's command enables normal, drag, and any-motion tracking (DEC modes
   1000, 1002, and 1003), plus coordinate encodings 1015 and 1006.
6. Codex's event mapper forwards only four wheel variants; press, release, drag,
   and motion events fall through to `_ => None`
   ([source](../../codex-rs/tui/src/tui/event_stream.rs#L237-L280)).

Terminal-mode mutations are duplicated in normal entry/exit, common restore,
external-editor handoff, and Unix suspend/resume
([normal](../../codex-rs/tui/src/tui.rs#L648-L690),
[job control](../../codex-rs/tui/src/tui/job_control.rs#L59-L99),
[resume](../../codex-rs/tui/src/tui/job_control.rs#L183-L201)).
`restore_common()` disables mouse and bracketed paste but neither disables
alternate scroll nor leaves the alternate screen
([source](../../codex-rs/tui/src/tui.rs#L250-L280)). Because fallible bootstrap
work occurs after redesign enters the alternate screen but before normal leave,
an early error can strand terminal state
([source](../../codex-rs/tui/src/lib.rs#L1836-L1864)).

## Input and paste flow

Bracketed text paste is reasonably robust after it reaches Codex:

1. Crossterm maps a bracketed payload to `TuiEvent::Paste`.
2. `App` normalizes carriage returns and forwards text to `ChatWidget`
   ([source](../../codex-rs/tui/src/app.rs#L1492-L1499)).
3. The composer sanitizes text, preserves large pastes behind placeholders, and
   falls back to paste-burst detection for terminals without reliable bracketed
   paste ([source](../../codex-rs/tui/src/bottom_pane/chat_composer.rs#L871-L907)).

`Ctrl+V` is different: it is reserved for **image** clipboard paste
([source](../../codex-rs/tui/src/chatwidget/interaction.rs#L73-L99)). Over SSH it
usually attempts to read the remote host's clipboard, not the phone clipboard.
Text paste therefore depends on the terminal client's paste action.

## Render and state flow

`redesign_chrome::render_app()` receives the entire `App`, constructs chrome
context from it, converts concrete `HistoryCell` types, and queries `ChatWidget`
through redesign-specific adapter methods
([source](../../codex-rs/tui/src/redesign_chrome.rs#L245-L296)).

The renderer also switches between two composer implementations. Normal text is
drawn by `redesign_chrome/composer.rs`; a popup, active bottom-pane view, or draft
starting with `/` switches to the legacy bottom pane
([condition](../../codex-rs/tui/src/chatwidget.rs#L2205-L2209),
[render branch](../../codex-rs/tui/src/redesign_chrome.rs#L245-L289)).

Transcript semantics are inferred late from concrete cell types and sometimes
from already-rendered text, including parsing possible speaker prefixes. This
forces the renderer to reflow pre-rendered prose, tables, lists, shell commands,
and borders rather than receiving structured blocks.

Wide layouts paint a left navigation pane, the central transcript, and an
optional right activity rail into separate rectangles on the same physical
terminal rows
([layout](../../codex-rs/tui/src/redesign_chrome/layout.rs#L3-L85),
[render order](../../codex-rs/tui/src/redesign_chrome.rs#L245-L263)). Individual
messages then add speaker labels, alignment padding, and box-drawing borders
inside the transcript rectangle
([source](../../codex-rs/tui/src/redesign_chrome.rs#L1540-L1633)). Those regions
are meaningful only inside Codex; native terminal selection sees one grid of
cells and cannot exclude an adjacent pane from a client-selected physical line.

The redesign also asks the composer for expanded pending-paste text but uses the
unexpanded textarea cursor index
([text](../../codex-rs/tui/src/chatwidget.rs#L2121-L2127),
[render](../../codex-rs/tui/src/redesign_chrome.rs#L279-L289)). A large paste can
therefore make displayed text and cursor coordinates describe different strings.

## Existing extraction plan

`codex-rs/scrum_docs/` already proposes a neutral `RedesignViewModel`, reducer,
and Codex host adapter. That direction is sound. Two assumptions need revision:

- “Preserve current behavior” cannot include terminal ownership, raw-copy, or
  tiny-height bugs.
- Terminal policy and paste/capability boundaries must be designed before input
  extraction, or the new crate will encode the current mistakes as its public
  contract.
