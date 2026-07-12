# Target UX and architecture

## Product invariants

1. The active interaction surface—composer, approval, or question—always wins
   layout space over metadata.
2. Native terminal selection and paste remain available unless the user has
   explicitly enabled a complete pointer interaction mode.
3. Alternate screen, alternate scroll, mouse capture, bracketed paste, focus
   reporting, keyboard enhancement, and animation are independent policies.
4. Rendering is pure: it receives a neutral view model and never queries `App`,
   `ChatWidget`, or concrete `HistoryCell` types.
5. Input produces intent actions; only the host mutates Codex runtime state.
6. One composer document and one focus model drive all visual states.
7. Every visible shortcut comes from the resolved runtime keymap and current
   capabilities.
8. Copy payloads come from structured source content, never from rendered
   terminal cells.

## Terminal interaction policy

Introduce explicit enums rather than coupled booleans:

```text
ScreenBufferPreference: Auto | Always | Never
MouseCapturePreference: Auto | Always | Never
ResolvedScreenBuffer: Alternate | Inline
MotionPreference: Full | Reduced | Static
TranscriptPresentation: Rich | Plain
```

Recommended initial resolution:

- `MouseCapturePreference::Auto` resolves to **Never** until the redesign implements
  useful press, drag, focus, and spatial hit-testing.
- `ScreenBufferPreference` honors the existing `tui.alternate_screen` value in every UI.
- Bracketed paste stays enabled in both inline and alternate-screen modes.
- Alternate scroll may remain enabled without broad mouse capture.
- Plain transcript mode disables nonessential animation and decoration.

Do not try to guess “Termius Android” from `$TERM`. SSH commonly hides the local
client identity. Use conservative defaults plus explicit config and a diagnostic
surface.

## Target component flow

```text
TerminalCapabilities + UserPreferences + SurfaceNeeds
                         |
                         v
                  TerminalPolicy
                         |
                         v
              TerminalModeController
               (one transition owner)

CrosstermEvent -> EventNormalizer -> UiInput
                                      |
                 UiState + LayoutMap -> Reducer -> UiAction
                                                   |
                                                   v
                                           CodexHostExecutor
                                                   |
                                                   v
                                              App state
                                                   |
                                                   v
                                       CodexViewModelAdapter
                                                   |
                                                   v
                                      Pure redesign renderer
```

## Proposed module boundaries

```text
tui/src/terminal_session/
  policy.rs             resolve preferences and capabilities
  controller.rs         apply/restore terminal modes idempotently
  event_normalizer.rs   CrosstermEvent -> UiInput
  diagnostics.rs        report effective modes without sensitive data

tui-redesign/src/
  model.rs              neutral view models and IDs
  state.rs              focus, scroll, overlay, per-chat UI state
  action.rs             neutral user intents
  reducer.rs            state/input -> actions
  layout/               budget allocator and LayoutMap
  render/               pure focused widgets
  fixtures/             production-shaped, provider-neutral test data

tui/src/redesign_host/
  view_model.rs         App/ChatWidget -> neutral model
  actions.rs            neutral actions -> Codex operations
  composer_adapter.rs   existing robust composer engine -> one UI model
```

The terminal controller stays outside the provider-neutral renderer. It owns one
desired/applied mode state and is reused for startup, shutdown, panic recovery,
suspend/resume, overlays, and external-editor handoff. Job control must restore
the resolved policy, never hard-code mouse capture. Transitions are transactional:
success records the applied mode, failure unwinds completed steps in reverse
order, and cleanup defensively resets alternate scroll and screen buffer.

## Responsive layout contract

Allocate vertical space by priority, not by subtracting fixed chrome first:

1. Active interaction surface: at least one editable/actionable row.
2. Blocking status or approval reason.
3. Transcript tail.
4. Compact activity/model header.
5. Shortcut/help row.
6. Workspace metadata.
7. Sidebar and system rail.

Suggested modes:

| Viewport | Behavior |
|---|---|
| `height <= 4` | Interaction-only; no borders, rails, metadata, or persistent footer. |
| `height 5..=7` | Interaction + one compact status line; transcript only if space remains. |
| Compact | Transcript + composer; overlays replace content rather than stack over it. |
| Standard | Optional sidebar; one concise header/footer. |
| Wide | Sidebar and optional activity rail, never at the cost of the main minimum width. |

The full-frame snapshot—not an isolated widget—must prove each class. Layout
should return a `LayoutMap` used by both rendering and pointer hit-testing.

## Copy and paste UX

Treat these as distinct capabilities:

- **Paste text:** initiated by the local terminal; arrives as bracketed paste or
  a key burst. It must never be bound to remote clipboard access.
- **Attach clipboard image:** available only when the running process can access
  the user's actual clipboard. Hide or explain it over SSH.
- **Attach uploaded path:** the reliable remote/mobile image path.
- **Copy last response:** structured app action, using native clipboard locally
  and terminal-mediated copy over SSH.
- **Plain transcript:** undecorated, stable, selectable text with mouse capture
  disabled and no spinner updates.

Provide two explicit message actions:

- **Copy message** serializes the canonical message blocks, preserving intended
  code and paragraph newlines while excluding speaker labels, bubble borders,
  gutters, scrollbars, side navigation, and activity rails.
- **Focus message for partial copy** replaces the rich layout with one stable,
  full-width, undecorated message. It disables mouse capture and motion, exposes
  an obvious exit action, and does not render adjacent panes on any selected
  physical row.

Add both actions to the command palette so mobile users do not need an Alt key.
The transcript focus model can use previous/next-message actions to choose the
copy target. Native substring selection in the normal rich multi-pane view is
explicitly best-effort; the reliable contract belongs to the isolated copy
surface. Do not attempt OSC 52 clipboard reads or clipboard post-processing;
client security policies make them unreliable and the remote process does not
receive the terminal's selection range.

## One composer, one focus model

Keep the mature text document/paste engine initially, but expose a neutral
`ComposerViewModel` containing text, cursor, attachments, pending-paste tokens,
validation state, and capability-aware actions. The redesign renderer should not
reimplement cursor/wrap semantics and then swap to a legacy renderer for `/` or
popups. `display_text` and `display_cursor` must always use the same coordinate
space, including large-paste placeholders and attachment elements.

Use a single focus enum such as `Composer`, `Transcript`, `Sidebar`, `Overlay`,
`Approval`, and `TerminalWindow`. Reducer transitions become exhaustive and
testable. A pointer mode, if later enabled, uses `LayoutMap` to emit the same
focus/actions as keyboard input.

## Rendering and motion

- Schedule only when the visible animation frame can change.
- Cap routine activity motion at 5 Hz; use 1 Hz or static state for reduced/remote
  profiles.
- Stop motion in plain transcript and other explicit copy surfaces.
- Keep idle redraw rate at zero.
- Never infer speaker identity or content kind from rendered strings; the host
  adapter must provide structured roles and labels.
