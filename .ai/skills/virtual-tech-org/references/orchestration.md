# Orchestration — Provider Capability Map

The org's **persona and process layers are provider-neutral**. Only the **execution layer** — how the CTO actually dispatches engineering work — differs by runtime. This file is the single place that difference lives.

**How to use this map:** You know which runtime you are (Claude Code or Codex). The skill (`SKILL.md`) dispatches work as neutral *capabilities*; look each one up below and use **your runtime's column**. The engineering standard is identical across runtimes; only the mechanism changes.

**Codex dispatch mechanism (read once):** The orchestrator delegates with Codex's multi-agent collaboration tools — `spawn_agent`, `wait_agent`, `send_input`, `resume_agent`, `close_agent` (stable, on by default under `features.multi_agent`). In this runtime they're exposed as `multi_agent_v1.spawn_agent` etc.; treat `multi_agent_v1` as the *current* namespace, not a durable public API name. The four shipped agents in `.codex/agents/*.toml` (`repo_explorer`, `reviewer`, `test_verifier`, `docs_researcher`) are agent **definitions** selected at spawn — not directly callable tools. Spawned subagents run in parallel, each in its own **forked workspace**; the parent `wait_agent`s and integrates their returned changes. `agents.max_depth = 1` → flat topology: workers cannot spawn further workers. (Refs: developers.openai.com/codex/config-reference, developers.openai.com/codex/subagents.)

## Capability → runtime primitive

| Capability | Claude Code | Codex |
|------------|-------------|-------|
| `explore` — read-only code/architecture survey | `Agent(subagent_type:"Explore", …)` | spawn the `repo_explorer` subagent |
| `research` — docs / tech evaluation | `Agent(subagent_type:"general-purpose")` + web search | spawn the `docs_researcher` subagent |
| `plan` — architecture & work breakdown | `Agent(subagent_type:"Plan", …)` | spawn a subagent at high reasoning (`-c model_reasoning_effort=high`) |
| `implement` — write code | `Agent(subagent_type:"general-purpose", …)` | spawn a coding subagent with `spawn_agent`, or implement in the main agent |
| `test` — write/run tests, verify | `Agent(subagent_type:"general-purpose")` + run suite | spawn the `test_verifier` subagent |
| `review:code` — quality/maintainability review | `Agent(subagent_type:"superpowers:code-reviewer")` | spawn the `reviewer` subagent |
| `review:security` — threat/vuln/secret audit | `Agent(subagent_type:"general-purpose")`, security prompt | spawn the `reviewer` subagent, security-focused prompt |
| `parallelize` — independent tasks at once | dispatch N `Agent` calls in one message | spawn independent subagents in parallel; bounded by `agents.max_threads` (6) and `agents.max_depth = 1` (flat — workers don't sub-spawn) |
| `isolate` — conflict-free parallel writes | `Agent(isolation:"worktree", …)` | spawned subagents' native forked workspaces; keep write scopes disjoint and integrate at the gate |
| `background` — non-blocking long work | `Agent(run_in_background:true, …)` | spawned subagents run asynchronously — continue local work, then `wait_agent` / `close_agent` when results are needed |
| `track` — progress & project state | `TaskCreate` / `TaskUpdate` + `.ai/context/` | `.ai/context/` (decisions, active-context) + an in-context checklist |

**Dispatch profile:** read-only capabilities (`explore`, `research`, `review:*`) run sandboxed/read-only; `implement` and `test` need write access. On Codex this is each agent's `sandbox_mode = "read-only"` vs `"workspace-write"` — the shipped custom agents already set the right mode.

**Extending to a new runtime:** add a column. **Giving Codex a dedicated specialist** (e.g. an `implementer` or `architect`): drop a `.codex/agents/<name>.toml` and select it at spawn — no SKILL.md change needed. This keeps the org skill-only by default while leaving the door open.

## Methodology map (the disciplined cycle)

The org enforces **plan → test-first (Stage 3+) → verify-before-gate → review** regardless of runtime. The skill *owns* this discipline; the runtime only changes how it's enforced.

| Discipline | Claude Code (enforcer) | Codex / no-plugin (enforcer) |
|------------|------------------------|------------------------------|
| Structured discovery | `superpowers:brainstorming` | CEO runs the built-in discovery questions |
| Plan before build | `superpowers:writing-plans` | CTO writes the plan inline into the architecture doc |
| Test-driven (Stage 3+) | `superpowers:test-driven-development` | RED→GREEN→REFACTOR required in the `implement` prompt + `test_verifier` |
| Parallel execution | `superpowers:dispatching-parallel-agents` | parallel `spawn_agent` subagents (see `parallelize`) |
| Isolated work | `superpowers:using-git-worktrees` | spawned subagents' forked workspaces (see `isolate`) |
| Debug methodically | `superpowers:systematic-debugging` | reproduce → isolate → diagnose → fix, inline |
| Verify before gate | `superpowers:verification-before-completion` | `test_verifier` runs the suite; CTO confirms output before the gate |
| Code review at gate | `superpowers:requesting-code-review` | spawn the `reviewer` subagent |
| Finish the branch | `superpowers:finishing-a-development-branch` | CTO does merge / PR / cleanup inline |

**Superpowers is an optional accelerator, not a dependency.** On Claude Code it enforces the discipline mechanically; on Codex — or Claude Code without the plugin — the CTO follows the same cycle using the runtime's agents and inline process. The standard is identical; only the enforcement mechanism differs.
