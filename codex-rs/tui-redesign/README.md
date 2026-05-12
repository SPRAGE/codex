# TUI Redesign Sandbox

This crate is a prototype sandbox for redesigned TUI interaction ideas. The
production redesigned TUI lives in `../tui/src/redesign_chrome.rs` and the
surrounding `../tui/src/app/` input/session code.

Use this crate to explore layout, focus, approval, and composer behavior in
isolation. Once an idea is validated, port the production version into `tui/`
and add the relevant production snapshot or input tests there.

Do not treat this crate as the source of truth for shipped behavior. If a bug is
visible in the production redesign, fix it in `tui/` first.

## Current Ownership

- Production renderer: `tui/src/redesign_chrome.rs`
- Production input handling: `tui/src/app/input.rs`
- Legacy bottom-pane fallback: `tui/src/chatwidget.rs` and `tui/src/bottom_pane/`
- Sandbox renderer and demo runtime: `tui-redesign/src/`

## Retirement Criteria

Retire this crate once the production redesign has stable coverage for:

- Standard, wide, narrow, and tiny terminal layouts.
- Sidebar focus and navigation.
- Composer cursor movement and wrapping.
- Approval wording and selection.
- Slash command and modal fallback states.
