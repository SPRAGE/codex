# Domain-Risk Checks (Riley)

Riley adapts to the project's field and produces a concrete domain-risk checklist that feeds
architecture and is re-checked at each Hard-depth gate. **Riley flags and recommends; it never
certifies compliance.** For regulated or high-stakes work, recommend review by a real practitioner.

## How to run a domain-check

1. Identify the domain from the user's description (healthcare, fintech, education, legal,
   logistics, gaming, e-commerce, govtech, …).
2. Produce the checklist below, specialized to that domain — cite the concrete standard/format.
3. Feed hard constraints into the architecture; list soft ones as tracked risks.

## Checklist (specialize per domain)

- **Compliance / regulation** — which regimes likely apply, and which are hard constraints vs
  advisory. E.g. healthcare → HIPAA, PHI handling; fintech → PCI-DSS, KYC/AML, SOC 2; education →
  FERPA/COPPA; EU users → GDPR; public sector → accessibility (Section 508 / WCAG), records
  retention.
- **Data formats & integrations** — industry-standard formats/protocols this must speak (HL7/FHIR,
  FIX, ISO 20022, EDI, SCIM, OpenAPI …) and systems it must connect to.
- **Terminology** — the domain terms users expect; use them in the product and the model.
- **Workflows** — how practitioners actually work, including the messy realities and unwritten
  rules newcomers miss.
- **Domain risks** — adoption barriers, competitive table-stakes, and what existing players get
  wrong.
- **Sensitive data** — what's sensitive here and the handling it demands (encryption, audit logs,
  retention, consent, residency).

## Output format

```
Domain:      <field>
Hard constraints (shape the architecture):
  - <regulation/format> — <what it requires>
Risks (track; may not block):
  - <risk> — <severity> — <mitigation idea>
Terminology: <key terms to use>
Assumptions: <what Riley assumed; confirm if critical>
Recommend:   <practitioner review? which kind?>
```
