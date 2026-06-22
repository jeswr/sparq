<!-- [OPUS-4.8] sq-ez5z — ISO/IEC 27001:2022 clause 6/8 risk-assessment + risk-treatment
     methodology + register TEMPLATE. Org-adoptable scaffold; NOT a certificate, NOT a signed
     risk decision. Remediation of gap GAP-ISO-1. Re-review when Fable returns. -->

# ISO/IEC 27001:2022 — risk methodology + treatment + register template (clauses 6 & 8)

> **THIS IS AN ADOPTABLE TEMPLATE, NOT A CERTIFICATION CLAIM.** It is provided for an
> **adopting organization** to populate and sign as part of operating its own ISMS. The risk
> *decisions* (acceptance, treatment, residual-risk sign-off) are **the organization's**, not
> sparq's — every such cell is a `<FILL-IN>` placeholder. Populating it does **not** make
> sparq or the adopting organization "ISO 27001 certified." Remediates part of gap
> **GAP-ISO-1**. Read [`README.md`](./README.md) and [`controls.md`](./controls.md) first.

## What this template is for

ISO/IEC 27001:2022 requires, across clauses 6 and 8:

- **6.1.2** A documented **information-security risk-assessment process**: risk-acceptance
  criteria + criteria for performing assessments; repeatable, comparable results; identifies
  risks (to confidentiality, integrity, availability) with owners; analyses likelihood &
  consequence; evaluates against criteria.
- **6.1.3** A documented **risk-treatment process**: select options, determine the controls
  needed, compare to **Annex A** (no necessary control omitted), produce the **Statement of
  Applicability** ([`soa-template.md`](./soa-template.md)), formulate a **risk-treatment
  plan**, and obtain risk owners' approval of the plan and the **residual** risks.
- **6.2** Information-security **objectives** consistent with the policy.
- **8.2 / 8.3** **Operate** the risk assessment (at planned intervals / on significant change)
  and **implement** the risk-treatment plan; retain documented results.

This template supplies a methodology the org can adopt **as-is or adapt**, plus a register
schema and **sparq-seeded candidate risks** drawn from `research/threat-model.md` (the STRIDE
model). The org owns the scoring, the acceptance decision, and the sign-off.

## Where sparq helps and where the org must decide

- **sparq supplies risk *inputs*:** the asset inventory and trust boundaries (B1–B5) in
  `research/threat-model.md`, the documented limitations (B3 no-auth; the ZK/MPC estate carries
  **no production guarantee** — v1 ZK verifier originally found NOT sound then remediated
  (`sq-1s2`) with an **internal** re-audit ("sound as landed for the assumed threat model"),
  external sign-off STILL PENDING `sq-qhy4` [OPUS-4.8]), and
  the per-control technical evidence in [`controls.md`](./controls.md) (which lowers the
  likelihood/impact of several component-level risks).
- **The org owns the risk *decisions*:** its risk-acceptance criteria, the likelihood/impact
  scoring for *its* deployment and *its* data, which residual risks it accepts, and who the
  risk owners are. **sparq cannot make these decisions and does not pretend to.**

---

## Part A — Risk-assessment methodology (org adopts/adapts)

### A.1 Scope & assets

Assets are drawn from the ISMS scope ([`isms-scope-template.md`](./isms-scope-template.md))
and seeded from `research/threat-model.md` §Assets. The org completes the inventory (A.5.9).

| Asset ID | Asset | C / I / A relevance | Owner |
|---|---|---|---|
| AS-1 | Query-result integrity (correct answers) | I | `<FILL-IN>` |
| AS-2 | Memory-safety of the `sparq-core` mmap path (boundary B5) | I, A | `<FILL-IN>` |
| AS-3 | Availability of the query service | A | `<FILL-IN>` |
| AS-4 | Confidentiality of the loaded RDF dataset | C | `<FILL-IN>` |
| AS-5 | On-disk index/archive integrity | I | `<FILL-IN>` |
| AS-6 | Host environment / supply chain of the build | C, I, A | `<FILL-IN>` |
| AS-n | `<FILL-IN: the org's own assets — data stores, credentials, gateway, keys>` | `<FILL-IN>` | `<FILL-IN>` |

### A.2 Likelihood & consequence scales (org sets thresholds)

The 1–5 anchors below are a **starting suggestion**; the org calibrates them to its context
and **documents** the final scale (clause 6.1.2 requires the criteria be defined).

| Score | Likelihood (annualised, suggested) | Consequence (suggested) |
|---|---|---|
| 1 | Rare | Negligible |
| 2 | Unlikely | Minor |
| 3 | Possible | Moderate |
| 4 | Likely | Major |
| 5 | Almost certain | Severe / catastrophic |

**Risk score = Likelihood × Consequence** (1–25). The org may substitute its own method
(qualitative, FAIR, etc.) — the standard requires *consistency and repeatability*, not a
specific formula.

### A.3 Risk-acceptance criteria (org decides — clause 6.1.2 a)

> The org **must** set and approve these. Suggested banding shown; replace with the org's.

| Band | Score range | Default disposition | Approval required |
|---|---|---|---|
| Low | `<FILL-IN: e.g. 1–5>` | Accept / monitor | Risk owner |
| Medium | `<FILL-IN: e.g. 6–12>` | Treat (reduce) | `<FILL-IN: role>` |
| High | `<FILL-IN: e.g. 13–19>` | Treat — priority | `<FILL-IN: senior role>` |
| Critical | `<FILL-IN: e.g. 20–25>` | Treat before go-live | Top management |

### A.4 Assessment cadence (clause 8.2)

Perform the assessment `<FILL-IN: at planned intervals, e.g. annually>` **and** on significant
change (new dataset class, new sparq major version, change to the B3 fronting gateway, a
sparq GHSA advisory). Retain results as documented information.

---

## Part B — Risk register (schema + sparq-seeded candidate rows)

> The **seed rows** below are *candidate* risks derived from `research/threat-model.md`, shown
> with sparq's **mitigation evidence** pre-filled (so the org can credit the control) and with
> every **scoring + decision cell left as `<FILL-IN>`**. The org owns the numbers and the
> disposition. Add the org's own deployment/data risks.

| Risk ID | Threat (STRIDE / source) | Asset | Existing sparq mitigation (evidence) | Likelihood (org) | Consequence (org) | Score (org) | Treatment option (org) | Risk owner (org) | Residual (org) | Approved (org) |
|---|---|---|---|---|---|---|---|---|---|---|
| R-1 | DoS via expensive query (T-DoS) | AS-3 | `QueryBudget` DoS-limit primitive in `sparq-engine`; `controls.md` A.8.6 | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN: Treat — operator sets budget + gateway rate-limit>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` |
| R-2 | Memory-safety UB in unsafe mmap path (T-Corrupt, B5) | AS-2 | `#![forbid(unsafe_code)]` in 31/36 crates; Miri lane; fuzz; mmap corruption oracle; `compliance/memsafety/`; `controls.md` A.8.27/A.8.29 | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` |
| R-3 | Unauthenticated access to query/update API (T-HTTP-EoP, B3) | AS-4 | **No sparq-side authz by design**; optional `SPARQ_AUTH_TOKEN`; mitigation is **operator-owned** (front with authn/TLS gateway); `controls.md` A.5.15/A.8.3/A.8.5; gap GAP-ISO-2 | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN: Treat — org deploys fronting gateway>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` |
| R-4 | Sensitive data / path leak in HTTP error body (T-Info) | AS-4 | Error-body sanitization shipped (PR #241) + no-echo regression tests; `controls.md` A.8.12 | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` |
| R-5 | Vulnerable / malicious dependency (supply chain) | AS-6 | `cargo deny` advisories/bans/licenses/sources GATING; daily advisory watchdog; Dependabot; SBOM + SLSA provenance; `controls.md` A.5.19–A.5.22 / A.8.8 | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` |
| R-6 | **Misplaced reliance on the ZK/MPC estate for confidentiality/integrity** | AS-4, AS-1 | **NO production-grade mitigation to credit — the estate carries NO production guarantee.** The v1 ZK verifier was originally found NOT sound (`research/zk-soundness-audit.md`); `sq-1s2` landed the binding layer + an **internal** re-audit (`research/zk-verifier-reaudit.md`, `sq-gbp4`) found all findings closed → "sound as landed for the assumed threat model," but external sign-off is **STILL PENDING** (`sq-qhy4`, P0) and there is no production guarantee (`SECURITY.md`) [OPUS-4.8]. The treatment is **do not rely on it**; record as accepted-by-avoidance, not as a control | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN: Treat by exclusion — do NOT use ZK/MPC for any guarantee>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` |
| R-n | `<FILL-IN: org/deployment-specific risks — key management, gateway misconfig, backup, data retention>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` |

> **Honesty constraint on R-6 (load-bearing).** No completion of this register may list the
> `sparq-zk` / `sparq-zk-compose` / `sparq-mpc` estate as a *risk-reducing control*. Even though
> the verifier was remediated (`sq-1s2`) and an **internal** re-audit
> (`research/zk-verifier-reaudit.md`, `sq-gbp4`) judged it "sound as landed for the assumed
> threat model," that re-audit is internal/single-model/read-only, **external sign-off is STILL
> PENDING** (`sq-qhy4`, P0) and there is **no production guarantee**; the only valid treatment is
> to **not depend on it** for any security property. Crediting it as a mitigation would be the
> exact overclaim the honesty contract (and `controls.md` A.8.24) forbids. [OPUS-4.8]
> <!-- [OPUS-4.8] reconciled with post-remediation re-audit (sq-gbp4); see ZK-verdict cross-ref sweep -->

---

## Part C — Risk-treatment plan (clause 6.1.3 e / 8.3)

For every risk the org elects to **treat**, record the plan. Treatment options (clause 6.1.3):
modify (apply controls), retain (accept), avoid, or share (transfer/insure).

| Risk ID | Selected option | Controls to apply (Annex A ref → `soa-template.md`) | Action / task | Owner | Target date | Status |
|---|---|---|---|---|---|---|
| `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN: e.g. A.8.6, A.5.15>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` | `<FILL-IN>` |

**Necessary-controls cross-check (clause 6.1.3 c/d):** confirm every control determined here
appears in the **SoA** with its justification; confirm no Annex A control needed to treat a
risk was omitted. This is the link between this register and
[`soa-template.md`](./soa-template.md).

**Risk-owner approval of the plan and the residual risks (clause 6.1.3 f):**
`<FILL-IN: names / roles / dates / signatures>`.

## Part D — Security objectives (clause 6.2)

| Objective | Measure / target | Linked risks | Owner | Review date |
|---|---|---|---|---|
| `<FILL-IN: e.g. "zero unpatched critical dependency advisories > 7 days">` | `<FILL-IN>` | R-5 | `<FILL-IN>` | `<FILL-IN>` |

## Honesty footer

This methodology + register is the org's **risk-assessment documented information** — it is
not a certificate and not a sparq claim. sparq seeds the candidate risks and pre-fills the
*technical mitigation evidence*; the **organization** owns the scoring, the acceptance
criteria, the treatment decisions, and the risk-owner sign-off. **No artifact in this
repository asserts that sparq is ISO 27001 certified**, and the ZK/MPC estate is **never** to
be entered as a risk-reducing control.
