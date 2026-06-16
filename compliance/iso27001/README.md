<!-- [OPUS-4.8] sq-toze — ISO/IEC 27001 (ISMS) framework intro. Authored while Fable
     unavailable — re-review when Fable returns. Engineer↔auditor loop (epic sq-toze).
     NON-CANONICAL timing (EC2 work box). -->

# ISO/IEC 27001:2022 — readiness mapping (sparq)

**Framework:** ISO/IEC 27001:2022 — Information Security Management System (ISMS),
with the **Annex A** reference set of controls (the 93 controls, structured per
ISO/IEC 27002:2022 into the four themes A.5 Organizational, A.6 People, A.7
Physical, A.8 Technological).

**Companion docs in this directory:**
- [`controls.md`](./controls.md) — the spine: every *applicable* Annex A control →
  status → repo evidence (file / test / CI job) → owner. One row per applicable control.
- [`evidence.md`](./evidence.md) — re-runnable verification for each technical claim
  (the exact command / grep / file an auditor re-runs).
- [`gap-register.md`](./gap-register.md) — open gaps, severity, remediation, and the
  `bd` bead that tracks each (epic `sq-toze`).
- [`soa-template.md`](./soa-template.md) — a **Statement of Applicability + ISMS
  clauses-4–10 scaffold** *template* for an adopting organization (remediation of
  GAP-ISO-1). A template, **not** a certificate.

## The single most important honesty point

**ISO/IEC 27001 certifies a *management system*, not source code.** A certificate is
issued only by an **accredited certification body** after a Stage 1 (documentation) +
Stage 2 (implementation) audit of an organization's ISMS — its scope statement, risk
assessment + treatment, Statement of Applicability (SoA), management review, internal
audit programme, and operating evidence over time. **No artifact in this repository,
and nothing in this directory, is or can be such a certificate.** This mapping is a
*readiness* pack: it shows, control by control, where sparq's repo/CI already provides
the **technical** evidence an ISMS would point to, and where a control is purely an
**organizational act** that the adopting organization must perform.

We therefore label every control honestly:

- **IMPLEMENTED & VERIFIED** — a *technical* control in the codebase/CI with a concrete,
  re-runnable artifact (file path + line, test name, or CI job). The auditor can re-run it.
- **AUDIT-READY** — the control's *technical substrate or doc-of-record exists in the
  repo* (e.g. `SECURITY.md`, `research/threat-model.md`, `CONTRIBUTING.md`,
  `deny.toml`), but the **certificate-grade evidence is an organizational act** (a signed
  policy, a risk-treatment decision, a management review, an internal audit) that an
  accredited body must witness. The repo gives the auditor a head start; it is **not** a
  pass.
- **GAP** — not met; in [`gap-register.md`](./gap-register.md) with a bead.
- **N/A (operator)** — the control is a property of the *deployed environment* (physical
  data centre, network firewall, user-account lifecycle, capacity of a running service),
  which for a **library + reference server** is the responsibility of the **adopting
  operator**, not of sparq's source. Flagged explicitly, not silently dropped.

## Scope of this mapping — what is sparq, and what is the operator's

sparq is a **Rust RDF/SPARQL data-engine library** (`sparq-core`, `sparq-engine`,
`sparq-parse`, …) plus a **reference HTTP server** (`sparq-server`, `sparq-serve`), a
**WASM port**, and a **ZK/MPC research estate**. It is consumed **as a dependency**.

Per `research/production-certification-plan.md` (§0) and `research/threat-model.md`, this
has three consequences for ISO 27001 scope:

1. **The ISMS subject is the open-source development organization** (the maintainers'
   secure-development process), **not a running service**. So the controls that map well
   are A.8's *secure-development*, *change-management*, *vulnerability-management*,
   *supplier/dependency*, *cryptography*, and *logging* technical controls — these have
   real repo/CI evidence. The A.5/A.6/A.7 organizational, people, and physical controls
   are **audit-ready at best** (they require an org to exist and act) or **N/A**.
2. **`sparq-server` has, by documented design, no authentication / per-user authz**
   (threat-model boundary **B3**: "front with a gateway / sparq-solid"). The Annex A
   *access-control* family (A.5.15–A.5.18, A.8.2–A.8.5) is therefore largely the
   **operator's** responsibility when they expose the server — mapped as an explicit
   architectural decision, **not a silent gap**.
3. **sparq does not itself collect personal data** — it is an engine; the deploying
   operator is the data controller for whatever RDF they load. Privacy-specific controls
   (A.5.34 PII, A.8.11 data masking) are scoped in the `privacy` worktree
   (`compliance/data-flow.md` + `compliance/dpia.md`); this file cross-references them.

### Explicitly OUT OF SCOPE for a library/server (operator-owned)

- **A.7 (Physical & environmental)** — data-centre access, equipment, cabling, supporting
  utilities, secure disposal of *hardware*. sparq has no premises or hardware; these are
  100% the operator's. Marked **N/A (operator)** with a one-line note, not a hollow row.
- **A.6 (People)** — employment screening, terms & conditions, disciplinary process,
  on/off-boarding. An open-source project has *contributors*, not employees; the residual
  (contributor agreement, conduct) is audit-ready via `CONTRIBUTING.md` + `LICENSE`, the
  rest is N/A for a project without an employer relationship.
- **Operational A.5/A.8 items requiring a running estate** — capacity management of a live
  service (A.8.6), backup of operator data (A.8.13), redundancy/availability SLAs (A.8.14),
  user-account lifecycle of a deployed instance (A.8.2–A.8.5). These are properties of the
  **operator's deployment**, marked **N/A (operator)** and pointed at the data-flow doc.

## Cryptography honesty gate (load-bearing)

**A.8.24 (Use of cryptography) makes NO claim about the `sparq-zk` / `sparq-zk-compose` /
`sparq-mpc` estate.** The documented verdict — **the v1 ZK verifier is NOT sound** and MPC
provides **no guarantee** — stands (`SECURITY.md`, `research/zk-soundness-audit.md`). Any
A.8.24 evidence here concerns only the cryptography sparq *relies on operationally* (TLS via
the operator's gateway, the Sigstore/SLSA signing of release attestations, dependency
crypto governed by the supply-chain lane). The soundness assessment of the ZK/MPC estate is
the **`cryptoreview`** framework's job, not this one. Presenting the research scaffold as a
"verified cryptographic control" would be a high-severity honesty finding — this mapping
does not do it.

## How to read the status column in `controls.md`

| Label | Means | Who closes the certificate gap |
|---|---|---|
| IMPLEMENTED & VERIFIED | Technical control + re-runnable repo/CI evidence | already met (repo) |
| AUDIT-READY | Doc-of-record / substrate in repo; cert needs an org act | adopting org's ISMS + accredited body |
| GAP | Not met | tracked bead, see gap-register |
| N/A (operator) | Property of the deployed environment | adopting operator |
