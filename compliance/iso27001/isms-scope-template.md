<!-- [OPUS-4.8] sq-ez5z — ISO/IEC 27001:2022 clause 4 ISMS scope-definition TEMPLATE.
     Org-adoptable scaffold; NOT a certificate and NOT a signed scope statement.
     Remediation of gap GAP-ISO-1. Re-review when Fable returns. -->

# ISO/IEC 27001:2022 — ISMS scope-definition template (clause 4)

> **THIS IS AN ADOPTABLE TEMPLATE, NOT A CERTIFICATION CLAIM.** It is provided for an
> **adopting organization** to populate, decide, and sign as part of operating its own ISMS.
> Nothing here is a passed control, a management decision, or an accredited audit result.
> Populating it does **not** make sparq, or the adopting organization, "ISO 27001 certified" —
> certification is an external act of an **accredited certification body**. Remediates part of
> gap **GAP-ISO-1**. Read [`README.md`](./README.md) (status-label semantics) and
> [`controls.md`](./controls.md) (the Annex A evidence spine) first.

## What this template is for

ISO/IEC 27001:2022 **clause 4 (Context of the organization)** requires the organization to:

- **4.1** Determine external and internal **issues** relevant to its purpose that affect the
  ISMS's ability to achieve its intended outcomes.
- **4.2** Determine the **interested parties** relevant to the ISMS and their relevant
  requirements (including legal/regulatory/contractual).
- **4.3** Determine the **scope** of the ISMS (its boundaries and applicability), considering
  4.1, 4.2, and the interfaces/dependencies with other organizations.
- **4.4** Establish, implement, maintain and continually improve the ISMS.

The scope statement (4.3) is **documented information** an accredited auditor reads first; it
bounds everything else (risk assessment, SoA, audit programme). This template gives the
adopting organization a head start by pre-filling the **sparq-as-a-component** facts and
leaving every **organizational decision** as a `<FILL-IN>` placeholder.

## How sparq sits inside an adopting organization's scope

sparq is a **Rust RDF/SPARQL data-engine library** plus a **reference HTTP server**, a WASM
port, and a **ZK/MPC research estate** — consumed **as a dependency**. So in an adopting
organization's ISMS, sparq is almost never *the* scope; it is a **component inside** the
in-scope information system the organization operates. The organization decides whether the
relevant scope is:

- the **service** it builds on top of `sparq-server` (most common — then sparq is a supplier
  component and most of the operator-owned controls in [`controls.md`](./controls.md) become
  the org's own applicable controls), or
- the **secure-development process** of a team that contributes to / forks sparq (then the A.8
  development controls that sparq already evidences map directly), or
- both.

The honest framing from [`README.md`](./README.md) §"Scope of this mapping" carries straight
into the org's clause-4 statement: **the access-control / network / physical / runtime
controls are the operator's**, and sparq's documented **no-auth boundary B3** (front with a
gateway) is an interface the scope statement must name explicitly.

---

## 4.1 — External & internal issues (org fills)

List the issues that affect the ISMS outcomes. The sparq-relevant seeds are pre-filled as
*candidate* rows; the org adds, removes, and owns the final list.

| # | Issue (external / internal) | Type | Relevance to the ISMS | Owner |
|---|---|---|---|---|
| 1 | `<FILL-IN: e.g. regulatory regime(s) the deployed service operates under — GDPR/CRA/sector rules>` | External | Drives clause-4.2 requirements + Annex A A.5.31 (legal) | `<FILL-IN>` |
| 2 | **Use of sparq as a third-party open-source data engine** (supplier dependency) | External | A.5.19–A.5.22 supplier controls; the org inherits sparq's supply-chain posture but owns its *own* dependency policy | `<FILL-IN>` |
| 3 | **sparq's documented no-authentication boundary (B3)** — `sparq-server` ships with no per-user authz | External (component design) | The org **must** front it with an authenticating/TLS gateway; names this in 4.3 interfaces | `<FILL-IN>` |
| 4 | **sparq's ZK/MPC estate carries NO production cryptographic guarantee** — v1 ZK verifier **originally found NOT sound** (`research/zk-soundness-audit.md`), then `sq-1s2` landed the binding layer + an **internal** re-audit (`research/zk-verifier-reaudit.md`, `sq-gbp4`) found all findings closed → "sound as landed for the assumed threat model"; **external sign-off STILL PENDING** (`sq-qhy4`, P0), no production guarantee (`SECURITY.md`) [OPUS-4.8] | External (component limitation) | The org must **not** rely on it for any confidentiality/integrity guarantee; record as an explicit exclusion | `<FILL-IN>` |
| 5 | `<FILL-IN: internal — team structure, skills, change-rate, hosting model (cloud/on-prem)>` | Internal | Affects resourcing (clause 7) + operator-owned controls | `<FILL-IN>` |
| 6 | `<FILL-IN: data sensitivity of the RDF the org loads into sparq>` | Internal | Drives A.5.12 classification + privacy scope (`compliance/dpia.md`) | `<FILL-IN>` |

## 4.2 — Interested parties & their requirements (org fills)

| # | Interested party | Their relevant requirement | How the ISMS addresses it |
|---|---|---|---|
| 1 | `<FILL-IN: data subjects, if the loaded RDF contains personal data>` | `<FILL-IN: lawful processing, rights>` | Cross-ref `compliance/dpia.md` (operator is controller) |
| 2 | `<FILL-IN: customers / users of the deployed service>` | `<FILL-IN: availability, confidentiality, SLAs>` | `<FILL-IN>` |
| 3 | `<FILL-IN: regulators / certification body>` | `<FILL-IN: applicable standards, reporting>` | A.5.31 legal register; this ISMS |
| 4 | **The sparq maintainers / open-source community** | Coordinated vulnerability disclosure (`SECURITY.md`) | The org's incident process feeds sparq's GHSA channel for component vulns (A.5.6) |
| 5 | `<FILL-IN: the org's own management / shareholders>` | `<FILL-IN: risk appetite, cost>` | Clause-6 risk acceptance criteria |

## 4.3 — ISMS scope statement (org writes; sparq facts pre-filled)

> Replace every `<FILL-IN>`. Keep the sparq-component facts — an auditor will expect the
> no-auth boundary and the ZK/MPC exclusion to be named explicitly, not buried.

**Organization / scope name:** `<FILL-IN: legal entity + the business function in scope>`

**In scope (boundaries & applicability):**

- **Information system(s):** `<FILL-IN: e.g. "the X service, which embeds sparq-server vN.N as
  its RDF/SPARQL query engine, fronted by the Y API gateway">`.
- **Locations / hosting:** `<FILL-IN: cloud account(s) / region(s) / on-prem sites>`.
- **Organizational units / teams:** `<FILL-IN>`.
- **Data:** `<FILL-IN: the RDF datasets loaded, their classification (A.5.12), whether they
  contain personal data (→ DPIA)>`.
- **sparq as an in-scope component:** the `sparq-*` crates / `sparq-server` binary / WASM port
  the org actually uses (`<FILL-IN: list the crates + version + enabled cargo features>`).

**Interfaces & dependencies (clause 4.3 explicit):**

- **B3 no-auth interface (mandatory to name):** `sparq-server` exposes the W3C SPARQL Protocol
  with **no per-user authentication** by design (one optional `SPARQ_AUTH_TOKEN` bearer token
  only). The org's scope **must** state how it controls this interface — e.g. "fronted by
  `<gateway>` providing TLS termination, authn/z, and rate-limiting." See `controls.md` rows
  A.5.15 / A.8.3 / A.8.5 and gap **GAP-ISO-2** (operator-responsibilities doc).
- **Supply-chain interface:** sparq is pulled as a dependency; the org's dependency-management
  policy (A.5.19–A.5.22) governs ingestion of sparq updates and sparq's own transitive deps.
- **Excluded crypto interface:** the `sparq-zk` / `sparq-zk-compose` / `sparq-mpc` estate is
  **excluded from any production security guarantee** — v1 ZK verifier originally found NOT
  sound, then remediated (`sq-1s2`) with an **internal** re-audit (`research/zk-verifier-reaudit.md`,
  `sq-gbp4`) judging it "sound as landed for the assumed threat model," but **external sign-off
  STILL PENDING** (`sq-qhy4`) and no production guarantee [OPUS-4.8]. State this as an exclusion
  so no auditor or reader infers a cryptographic control from it.
  <!-- [OPUS-4.8] reconciled with post-remediation re-audit (sq-gbp4); see ZK-verdict cross-ref sweep -->

**Out of scope (with justification):** `<FILL-IN: e.g. corporate IT not touching the service;
must justify under clause 4.3 that exclusions do not undermine the ISMS>`.

**Approved by (top management — clause 5.1):** `<FILL-IN: name / role / date / signature>`.

## 4.4 — ISMS establishment (pointer)

The org establishes and maintains the ISMS via the rest of this template set:

- **Clause 6 / 8** risk assessment + treatment → [`risk-methodology-template.md`](./risk-methodology-template.md)
- **Clause 6.1.3(d)** Statement of Applicability → [`soa-template.md`](./soa-template.md)
- **Clause 9.2** internal audit → [`internal-audit-programme-template.md`](./internal-audit-programme-template.md)
- **Clause 9.3** management review → [`management-review-template.md`](./management-review-template.md)
- **Clause 10** improvement → folded into the management-review + audit templates

## Honesty footer

Completing this scope statement is the **first** clause of an ISMS, not the certificate.
sparq supplies the *component facts* (its boundaries, its supply-chain posture, its documented
exclusions); the **organization** owns the issues, the interested parties, the boundary
decisions, and the top-management sign-off. **No artifact in this repository asserts that
sparq is ISO 27001 certified.**
