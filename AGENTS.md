# Codex Adapter

Codex-compatible agents auto-load `AGENTS.md`. Read `AI.md`, then `.ai/instructions.md` — it owns the tiered read-order, response style, skill locations, and rules. Keep this adapter thin; project guidance lives in `AI.md` and `.ai/`.

## Codex runtime notes

- Explore in parallel; prefer `rg`/`fd` over slow shell scans.
- Delegate bounded work to the shipped custom agents in `.codex/agents/`: `repo_explorer` before broad edits, `reviewer` before a PR or large change, `test_verifier` after implementing, `docs_researcher` for OpenAI/Codex questions (via the `openaiDeveloperDocs` MCP).
- Parallelize with `spawn_agent` (multi-agent: `max_threads` 6, `max_depth` 1 — workers don't sub-spawn); each runs in its own forked workspace; `wait_agent`, then integrate.
- Skills: when the user names a skill (with or without a leading slash), read `.ai/skills/<name>/SKILL.md`. `.agents/skills/` and `.codex/skills/` are relative symlinks to `.ai/skills/`.
- Keep `.codex/local/` and `.codex/tmp/` local; never commit them.
