# Superpowers Integration Guide

Map of superpowers skills to VTO product development stages. Each superpowers skill is
invoked via `Skill(skill: "superpowers:<name>")`. If superpowers are not available, the
VTO still works — it uses its built-in flows without methodology enforcement.

## Stage-by-Stage Mapping

### Stage 0: Discovery (CEO-led)

**Invoke:** `superpowers:brainstorming`

The CEO invokes brainstorming at the start of discovery to structure the exploration of
user intent, requirements, and design constraints. This ensures thorough exploration of
the problem space before the CEO writes the Product Brief.

**Flow:**
1. CEO greets the user
2. CEO invokes `Skill(skill: "superpowers:brainstorming")` to explore intent
3. Brainstorming explores requirements, constraints, and design considerations
4. CEO synthesizes findings into the Product Brief
5. Domain Expert (Riley) validates domain-specific aspects

**Without superpowers:** CEO uses the built-in discovery questions (archetype detection,
motivation, scope, audience).

### Stage 1: Architecture (CTO-led)

**Invoke:** `superpowers:writing-plans`

After the architecture design is complete, the CTO invokes writing-plans to formalize
the architecture into an executable implementation plan. This plan becomes the blueprint
for Stages 2-4.

**Flow:**
1. CTO designs architecture with the Architect agent
2. CTO invokes `Skill(skill: "superpowers:writing-plans")` with the architecture doc
3. The plan defines steps, critical files, dependencies, and review checkpoints
4. Plan is stored alongside the architecture doc as the execution blueprint

**Without superpowers:** CTO produces architecture doc + tech stack decision, and
decomposes work informally for agent dispatch.

### Stage 2: Prototype (Team execution)

**Invoke:**
- `superpowers:using-git-worktrees` — isolate prototype work from main branch
- `superpowers:dispatching-parallel-agents` — coordinate independent tasks
- `superpowers:subagent-driven-development` — execute plan tasks via parallel subagents

**No TDD at prototype stage** — this is intentionally rough. Shortcuts are logged as
technical debt with resolution targets.

**Flow:**
1. CTO invokes `Skill(skill: "superpowers:using-git-worktrees")` for isolation
2. CTO uses `Skill(skill: "superpowers:dispatching-parallel-agents")` to parallelize
   independent implementation tasks across team members
3. Subagents handle execution within the worktree
4. Results are reviewed at the Stage 2 gate

**Without superpowers:** CTO dispatches agents directly on the main branch.

### Stage 3: MVP (Full team)

**Invoke:**
- `superpowers:executing-plans` — follow the Stage 1 plan with review checkpoints
- `superpowers:test-driven-development` — **mandatory** for all feature implementation
- `superpowers:dispatching-parallel-agents` — coordinate parallel work
- `superpowers:systematic-debugging` — structured debugging when issues arise

**TDD is non-negotiable from Stage 3 onward.** The CTO enforces this for every feature
implementation. No code is written without tests first.

**Flow:**
1. CTO invokes `Skill(skill: "superpowers:executing-plans")` with the Stage 1 plan
2. For each feature: CTO enforces `Skill(skill: "superpowers:test-driven-development")`
3. For independent tasks: `Skill(skill: "superpowers:dispatching-parallel-agents")`
4. When issues arise: `Skill(skill: "superpowers:systematic-debugging")` — no guessing,
   structured investigation (reproduce → isolate → diagnose → fix)
5. Technical debt from prototype is resolved using the same TDD discipline

**Without superpowers:** CTO enforces engineering standards through agent prompts
and manual review.

### Stage 4: Production (Full team + hardening)

**Invoke:** Everything from Stage 3, plus:
- `superpowers:requesting-code-review` — security audit and quality review

**Flow:**
1. Same as Stage 3 for implementation work
2. CTO invokes `Skill(skill: "superpowers:requesting-code-review")` for:
   - Security audit of all user-facing code
   - Performance review of critical paths
   - Quality review against conventions and standards
3. Review findings are addressed before the final gate

**Without superpowers:** CTO assigns security and quality review tasks to subagents.

### Gate Reviews (between every stage)

**Invoke:**
- `superpowers:verification-before-completion` — **mandatory** before claiming any stage done
- `superpowers:requesting-code-review` — thorough review at each gate

**Flow:**
1. Before presenting gate results to the user, CTO MUST invoke
   `Skill(skill: "superpowers:verification-before-completion")`
2. This runs all tests, verifies all deliverables, and confirms output
3. CTO invokes `Skill(skill: "superpowers:requesting-code-review")` for quality review
4. Only after verification passes does the CEO present gate results to the user

**Without superpowers:** CEO/CTO present results based on agent output. Manual
verification still happens but without the structured enforcement.

### Stage 4 Completion

**Invoke:** `superpowers:finishing-a-development-branch`

**Flow:**
1. After Stage 4 gate passes, CTO invokes
   `Skill(skill: "superpowers:finishing-a-development-branch")`
2. This guides the final integration: merge strategy, PR creation, cleanup
3. CEO presents the completed project to the user with all deliverables

**Without superpowers:** CTO handles integration manually.

## Summary Table

| Stage | Superpowers | Who Invokes |
|-------|-------------|-------------|
| 0: Discovery | `brainstorming` | CEO |
| 1: Architecture | `writing-plans` | CTO |
| 2: Prototype | `using-git-worktrees`, `dispatching-parallel-agents`, `subagent-driven-development` | CTO |
| 3: MVP | `executing-plans`, `test-driven-development`, `dispatching-parallel-agents`, `systematic-debugging` | CTO |
| 4: Production | All of Stage 3 + `requesting-code-review` | CTO |
| Gate Reviews | `verification-before-completion`, `requesting-code-review` | CTO |
| Completion | `finishing-a-development-branch` | CTO |
