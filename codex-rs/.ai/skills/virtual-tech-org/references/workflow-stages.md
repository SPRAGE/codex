# Product Development Lifecycle

The virtual tech org delivers products through 5 stages. Each stage has clear inputs, outputs, and a gate review before proceeding.

## Stage 0: Discovery

**Led by**: CEO (Alex), with Domain Expert (Riley) actively contributing
**Duration**: 1-3 conversation turns
**Goal**: Turn a vague idea into a concrete, domain-informed product brief

### What Happens
The CEO runs a structured brainstorming session with the user. This is not a requirements dump — it's a conversation. The CEO asks questions, challenges assumptions, and helps the user think through what they actually need.

Riley (Domain Expert) participates actively in discovery — identifying the project's domain, providing industry context, flagging regulatory requirements, and ensuring the product brief reflects real-world domain needs rather than assumptions.

The CEO also identifies the **project archetype** (web app, API, CLI, library, data pipeline, system, mobile/desktop, full-stack) and the user's **tech stack preferences** during this stage. These shape everything that follows.

### Key Questions the CEO Asks
1. **The Problem**: What problem are we solving? Who has this problem?
2. **The User**: Who's the primary user? What does their day look like?
3. **The Core Flow**: What's the one thing this product absolutely must do?
4. **Success Metrics**: How will we know if this works?
5. **Constraints**: Budget? Timeline? Preferred tech? Existing systems to integrate with?
6. **Non-goals**: What are we explicitly NOT building? (This prevents scope creep later)
7. **Archetype**: What kind of project is this? (CEO confirms with CTO)

### Domain Expert Involvement
Riley identifies the project's domain early and provides critical context throughout discovery:
- "In this industry, the standard workflow looks like [X] — your product needs to fit into that, not fight it."
- "There's a regulatory requirement here you should know about: [specific regulation]. That's a hard constraint, not a nice-to-have."
- "The existing players in this space all do [X]. Your differentiator could be [Y], but don't skip [X] — users expect it."
- "The terminology matters here — your users will expect [domain term], not [generic term]. Let's get that right from the start."
- "Before we scope features, let me walk you through how [domain workflow] actually works in practice. It's different from what most people assume."

Riley also helps the founder articulate their specific goals within the domain — understanding not just *what* they want to build, but *why* from a domain perspective, and what success looks like to people in that industry.

### CTO Involvement
The CTO listens during discovery and occasionally jumps in:
- "That's technically feasible, but it'll triple the timeline. Can we start simpler?"
- "If we use X instead of Y, we get that for free."
- "I'd want Drew to research whether an existing solution covers this before we build from scratch."
- "Do you have a preferred language or framework? That'll shape my architecture decisions."
- "Riley, does this domain have any standard data formats or integrations we need to plan for?"

### Output: Product Brief

Save to `project/product-brief.md`:

```markdown
# Product Brief: [Project Name]

## Problem Statement
[1-2 sentences on the core problem]

## Target User
[Who uses this, what's their context]

## Project Archetype
[web-app | api | cli | library | data-pipeline | system | mobile-desktop | full-stack]

## Core Features (MVP)
1. [Feature 1 — what it does, not how]
2. [Feature 2]
3. [Feature 3]
(Keep to 3-5 features maximum for MVP)

## Non-Goals
- [What we're explicitly not building]

## Success Criteria
- [Measurable outcome 1]
- [Measurable outcome 2]

## Constraints
- Tech preferences: [user's preferred stack, if any]
- Timeline: [any deadlines]
- Integration: [existing systems]

## Domain Context (provided by Riley)
- Industry/field: [the specific domain this project operates in]
- Key regulations/compliance: [relevant regulations, standards, or compliance requirements]
- Domain workflows: [how the target users currently work — the processes this product fits into]
- Industry terminology: [key domain terms the product should use]
- Domain-specific risks: [risks unique to this industry — regulatory, competitive, adoption barriers]
- Existing landscape: [what current solutions exist, what they get right/wrong]
- Domain constraints: [things the domain requires that may not be obvious — data formats, integration standards, certification needs]

## Open Questions
- [Things we still need to figure out]

## Initial Risk Assessment
- [Risk 1 — severity, owner, mitigation idea]
- [Risk 2]
```

### Gate Review
CEO presents the brief to the user, with Riley validating domain accuracy: "Here's what I've distilled from our conversation. Riley has reviewed the domain context. Does this capture it? Anything I'm missing or got wrong?"

User can approve, revise, or send back for more brainstorming.

#### Talent Assessment Handoff
After the user approves the product brief, the CTO recommends a talent assessment:

> **Jordan (CTO):** Good — the product brief is solid. Riley, any domain concerns before we move to architecture?

---

## Stage 1: Architecture

**Led by**: CTO (Jordan) + Architect (Priya), Domain Expert (Riley) advisory
**Duration**: 1-2 conversation turns
**Goal**: Technical design that guides all subsequent development, informed by domain requirements

### What Happens
The CTO takes the product brief and works with the Architect to design the system. Riley advises on domain-specific technical requirements — compliance-driven architecture choices, required data formats, industry-standard integrations, and regulatory constraints that affect the design. The CTO presents the design to the user in plain language, with technical details available on request.

### Design Decisions to Make
1. **Architecture pattern**: Monolith? Microservices? Serverless? Modular monolith? Single binary? Library with examples?
2. **Tech stack**: Languages, frameworks, databases, message queues — based on the user's preferences and the project's needs
3. **Data model**: Core entities and their relationships (if applicable)
4. **API / Interface design**: REST? gRPC? GraphQL? CLI flags? Library public API? Depends on archetype.
5. **Infrastructure**: Where does this run? How is it deployed? (if applicable)
6. **Security model**: Auth strategy, data protection, access control (if applicable)

### CTO Decision Framework
When choosing tech, the CTO considers:
- **User's stated preferences first** — if the user has a preferred language, framework, or toolchain, use it. Ask during Discovery or infer from context (their existing projects, their background, what they mention casually). Never default to a specific stack.
- **Riley's domain requirements** — compliance standards, required data formats, industry-standard integrations, and regulatory constraints that may dictate certain technical choices
- User's stated constraints and existing systems to integrate with
- Simplicity over cleverness (especially for prototype/MVP)
- What minimizes risk for the specific problem domain
- Community size and documentation quality (smaller teams benefit from well-documented ecosystems)

The CTO should ask early: "Do you have a preferred stack, or should I recommend one based on the project needs?" If the user says "you decide", the CTO picks the simplest mainstream option for the problem domain and explains why. The CTO also consults Riley: "Riley, are there any domain-specific technical requirements that should influence our stack choice?"

### Archetype-Specific Design Focus

| Archetype | Primary Design Concerns |
|-----------|------------------------|
| Web App | Frontend/backend separation, data model, auth, deployment |
| API / Service | Endpoint design, data model, auth, rate limiting, versioning |
| CLI Tool | Command structure, flag design, output formatting, configuration |
| Library / SDK | Public API surface, type safety, versioning strategy, zero/minimal dependencies |
| Data Pipeline | Data flow diagram, failure recovery, idempotency, scheduling |
| System / Infra | Component interactions, configuration management, rollback strategy |
| Mobile / Desktop | Platform targets, offline capability, update mechanism |
| Full-Stack | Service boundaries, communication patterns, shared types, deployment topology |

### Output: Architecture Doc

Save to `project/architecture.md`:

```markdown
# Architecture: [Project Name]

## High-Level Design
[Diagram description or ASCII art showing major components]

## Tech Stack
- Language: [X] — [why]
- Framework: [X] — [why]
- Database: [X] — [why] (if applicable)
- [Any other significant choices]

## Data Model
[Core entities, relationships, key fields] (if applicable)

## Interface Surface
[Main endpoints/operations/commands/public API with brief descriptions]

## Project Structure
[Directory layout]

## Deployment Strategy
[How it runs — dev, staging, prod] (if applicable)

## Domain Requirements (from Riley)
- Compliance: [regulations that affect architecture — HIPAA, PCI-DSS, GDPR, etc.]
- Data formats: [industry-standard formats — HL7 FHIR, FIX protocol, etc.]
- Required integrations: [industry-standard systems this must connect to]
- Domain constraints: [architectural decisions driven by domain realities]

## Key Design Decisions
| Decision | Choice | Rationale | Alternatives Considered |
|----------|--------|-----------|------------------------|
| ... | ... | ... | ... |
```

### Gate Review
CTO presents to the user, with Riley validating domain alignment: "Here's how we're going to build this. Riley has confirmed it meets [domain requirements]. The key tradeoffs are [X]. Questions?"

If the user has strong tech opinions (like preferring one language over another), the CTO adjusts and explains any implications. If Riley flags a domain concern with a technical choice, the CTO explains the tradeoff.

#### Talent Assessment Handoff
After the user approves the architecture, the CTO recommends a tech-stack-informed talent assessment:

> **Jordan (CTO):** Architecture is locked. Riley, validate the domain-specific technical choices before the team starts building.

---

## Stage 2: Prototype

**Led by**: CTO (Jordan)
**Team**: Architect (Priya), Core Dev (Marcus), UI/Client Dev (Lina — if archetype has UI)
**Goal**: First working code — ugly but functional
**Agent execution**: Yes — parallel Claude Code subagents

### What Happens
This is where the rubber meets the road. The CTO dispatches Claude Code subagents to build a working prototype. Expectations are deliberately low — this is about proving the concept works, not about polish.

### Prototype Standards
- Core feature works end-to-end
- No auth, no error handling, no tests (those come in MVP)
- Hardcoded values are fine
- UI can be ugly / minimal (if applicable)
- README with "how to run it" is the only doc needed

### Technical Debt Tracking
**Every shortcut in the prototype gets logged.** The CTO maintains a tech debt list:
- "Hardcoded config: needs environment variables (resolve by Stage 3)"
- "No error handling on external calls: needs retry/circuit-breaker (resolve by Stage 3)"
- "In-memory storage: needs persistent store (resolve by Stage 3)"

This makes the prototype→MVP transition explicit and plannable.

### Agent Execution
The CTO dispatches parallel subagents. The typical execution:
1. Architect sets up project structure and scaffolding
2. Core dev implements core logic
3. UI/Client dev builds minimal interface (if applicable — parallel with core dev)
4. CTO reviews output and assembles

### Output
- Working code in `project/prototype/`
- Basic README with run instructions
- Technical debt log (in project-state.json)
- CTO summary of what works and what's duct-taped

### Gate Review
CTO demos to the user: "Here's the prototype. [Feature X] works — try it. It's rough around the edges, but the core concept is solid. Tech debt logged: [N items]. Ready to move to MVP?"

---

## Stage 3: MVP

**Led by**: CTO + VP Engineering (Sam)
**Team**: Full engineering team minus Security and Performance (adjusted per archetype)
**Goal**: Feature-complete for core use case, tested, deployable
**Agent execution**: Yes — full parallel Claude Code subagents

### What Happens
The team takes the prototype and turns it into a real product. This is the biggest stage — multiple agents working in coordinated parallel/sequential stages. **From this stage onward, engineering standards are enforced.** All technical debt from Stage 2 must be resolved.

### MVP Standards
- All core features from the product brief working
- Proper error handling
- Input validation on all user inputs
- No hardcoded secrets
- Basic auth if needed
- Unit and integration tests (80%+ coverage target)
- TDD workflow: RED→GREEN→REFACTOR
- Conventional commits: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`
- CI pipeline (even if simple)
- Clean code structure
- Interface documentation (API docs, CLI help text, library API reference — per archetype)
- User-facing README

### Development Cycle
Each feature within the MVP follows a disciplined cycle:
1. **Plan** — Sam (planner) breaks the feature into implementation phases
2. **TDD** — Marcus/Lina write failing tests first, then implement
3. **Review** — Casey reviews code for quality, security, maintainability
4. **Fix** — Developers address CRITICAL and HIGH findings
5. **Ship** — Feature merged with conventional commit

### Agent Execution
The CTO dispatches parallel subagents. Typical execution:

**Phase 1: Foundation** (parallel)
- Architect refactors prototype into clean structure
- Core dev implements remaining features
- UI/Client dev builds proper interface (if applicable)

**Phase 2: Quality** (sequential after Phase 1)
- QA writes and runs tests
- Code reviewer reviews all code
- DevOps sets up CI/CD pipeline

**Phase 3: Documentation** (parallel with Phase 2)
- Technical writer creates docs
- CTO reviews overall quality

### Output
- Production-ready code in `project/mvp/`
- Tests passing
- CI/CD config
- Interface docs and README
- CTO summary with quality metrics

### Gate Review
CEO and CTO present together:
- CEO: "Here's what we built against the original brief. [Coverage of features]."
- CTO: "Tech quality report: tests pass, CI is green, code structure is clean. Tech debt: zero. Here's what I'd improve before production."

---

## Stage 4: Production

**Led by**: CTO + VP Engineering
**Team**: ALL applicable roles active
**Goal**: Production-hardened, secure, performant, fully documented
**Agent execution**: Yes — full agent dispatch with security and performance roles

### What Happens
The final stage adds the "ilities" — security, reliability, scalability, observability. This is where the full org earns its keep.

### Production Standards
- Security audit complete — Ash runs OWASP Top 10, dependency CVEs, secret detection, input validation
- Performance benchmarks meet targets (Taylor)
- Monitoring and alerting configured (Kai) — if applicable
- Error recovery / graceful degradation
- Load testing passed (if applicable)
- E2E tests cover critical user flows (Robin)
- Complete documentation (Morgan) — synced with code
- Deployment automation (Kai) — if applicable
- Code review sign-off — Casey reviews with CRITICAL/HIGH/MEDIUM/LOW ratings
- All CRITICAL and HIGH findings resolved before launch
- 80%+ test coverage maintained
- Conventional commits throughout git history

### Agent Execution
The CTO dispatches parallel subagents. Typical execution:

**Phase 1: Audit** (parallel)
- Security engineer audits code + dependencies
- Performance engineer profiles and benchmarks
- Code reviewer does final review pass

**Phase 2: Harden** (sequential, based on audit findings)
- Core/UI devs fix security findings
- Performance engineer optimizes hot paths
- DevOps adds monitoring, logging, alerting (if applicable)

**Phase 3: Polish** (parallel)
- Technical writer completes all documentation
- DevOps finalizes deployment automation (if applicable)
- QA runs final regression suite

### Output
- Production code in `project/production/`
- Security audit report
- Performance benchmark results
- Complete documentation
- Deployment playbook (if applicable)
- Final CTO sign-off

### Gate Review
The CEO delivers a "launch brief":
- What we built
- How it meets the original success criteria
- Known limitations
- Recommended next steps (V2 features, scaling considerations)

The CTO adds a technical handover:
- How to run it
- How to deploy it (if applicable)
- How to monitor it (if applicable)
- What to watch out for

---

## Stage Skipping

Not every project needs all 5 stages. The CEO/CTO should recommend skipping when appropriate:

- **Quick script / utility**: Skip to Stage 2 (prototype = final product), add tests if non-trivial
- **Internal tool**: Stages 0-3, skip Stage 4 production hardening
- **User just wants a prototype**: Stages 0-2 only
- **Library with small API surface**: Stages 0-3, light Stage 4 (security + docs, skip performance/monitoring)
- **Production system**: All stages

The CEO should explicitly confirm: "For this project, I'd recommend going through Stages 0-3 and skipping the full production hardening. Sound right?"

## Rollback

If a stage produces poor results:
1. CTO explains what went wrong
2. Options: re-run the stage, go back to previous stage, adjust scope
3. User decides (or CEO/CTO decide in auto-pilot mode)
4. The team re-executes with adjusted parameters

Never silently push forward with broken output.
