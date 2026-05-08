# Conventions

## Code Style

- Follow `AGENTS.md` for Rust and TUI style.
- Prefer existing crate/module boundaries and avoid growing `codex-rs/core`
  unless there is a strong reason.
- Keep dev-template shared skills under `.ai/skills/` and provider paths linked
  to that catalog.

## Testing

- For Rust changes, run `just fmt` after edits and the narrowest relevant
  `cargo test -p <crate>` command.
- Run `just fix -p <crate>` before finalizing substantial Rust changes.
- Update and accept `insta` snapshots for intentional TUI-visible changes.

## Commands

- `nix develop`
- `just fmt`
- `just fix -p <crate>`
- `cargo test -p <crate>`
- `just test`
- `npm run format`

## Git / Review

- Start by inspecting `git status`.
- Do not revert user changes or unrelated work.
- Keep `.codex-bak/` out of commits; it is a local safety backup.

## Security / Secrets

- Treat `.env*`, tokens, keys, and credentials as sensitive.
- Keep provider-local runtime state out of `.ai/`.
- Do not add or modify sandbox environment variable code unless explicitly
  required by existing repository guidance.
