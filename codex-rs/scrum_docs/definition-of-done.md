# Definition of Done

## Product Done

The extraction is product-complete when:

- `codex-tui-redesign` owns provider-neutral rendering and input reduction.
- `codex-tui` hosts the extracted crate through an adapter.
- Codex redesign mode preserves current user-visible behavior.
- The extracted crate has no provider/runtime dependencies.
- Another provider host can be designed by implementing the view-model builder
  and action executor, without copying renderer code.

## Engineering Done

Every completed story must satisfy:

- Code compiles.
- Tests cover the changed boundary.
- Snapshot diffs are reviewed when rendering changes.
- Public traits and public structs have role-focused doc comments.
- New files have one clear responsibility.
- No new provider-specific dependency is added to `codex-tui-redesign`.
- No new behavior is hidden behind broad compatibility shims without a removal
  condition.

## Dependency Boundary Checklist

`tui-redesign/Cargo.toml` must not depend on:

- `codex-core`
- `codex-protocol`
- `codex-app-server`
- `codex-app-server-client`
- `codex-app-server-protocol`
- `codex-model-provider`
- `codex-model-provider-info`
- `codex-tui`

Allowed dependency categories:

- `ratatui`
- `crossterm`
- text wrapping and unicode width helpers
- snapshot test dependencies
- small provider-neutral utility crates if review confirms they do not pull in
  runtime state

## Test Gates

Minimum local checks before review:

```bash
cargo test -p codex-tui-redesign
cargo test -p codex-tui redesign_chrome::tests
cargo test -p codex-tui input::tests
git diff --check
```

Additional checks when touched:

- If `Cargo.toml` or `Cargo.lock` changes, run the repository lockfile update
  and check commands required by `AGENTS.md`.
- If user-visible TUI snapshots change, review and accept the relevant
  `insta` snapshots.
- If shared crates outside `tui` and `tui-redesign` change, run the narrowest
  additional crate tests for those crates.

## Review Checklist

Reviewers should confirm:

- Renderer code reads only neutral view-model state.
- Host adapter owns all Codex-specific mapping.
- Neutral actions describe intent and do not leak Codex implementation details.
- Shortcut priority matches the current production redesign.
- Plan-window and terminal-window state remain per chat.
- Transcript routing still handles hidden right rail correctly.
- The provider-independent crate remains understandable without reading Codex
  internals.

## Risk Controls

Renderer extraction risk:

- Keep the Codex wrapper until snapshot parity is proven.

Input regression risk:

- Keep existing app-level tests while adding reducer tests.

Provider coupling risk:

- Review `Cargo.toml` and public type imports in every extraction PR.

Large-diff risk:

- Land work in the sprint order from `sprint-plan.md`; avoid combining renderer
  extraction and input reducer extraction in one patch.

User-visible behavior risk:

- Treat existing Codex redesign behavior as the baseline. Any deliberate UX
  change must be represented as a separate backlog story with acceptance
  criteria.
