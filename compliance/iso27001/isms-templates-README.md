<!-- [OPUS-4.8] sq-ez5z — Index for the org-adoptable ISO/IEC 27001 ISMS template set
     (clauses 4-10 + SoA). These are ADOPTABLE TEMPLATES, NOT a certification claim.
     Remediation of gap GAP-ISO-1. Re-review when Fable returns. -->

# ISO/IEC 27001:2022 — org-adoptable ISMS template set (index)

> **THESE ARE ADOPTABLE TEMPLATES, NOT A CERTIFICATION CLAIM.** Nothing in this directory makes
> sparq — or any adopting organization — "ISO 27001 certified." ISO/IEC 27001 certifies a
> **management system**, not source code, and a certificate is issued **only** by an
> **accredited certification body** after a Stage 1 (documentation) + Stage 2 (implementation)
> audit of an **operating ISMS over time**. This template set is the *head start*: it pre-fills
> the **sparq-component facts** (boundaries, supply-chain posture, documented exclusions) and
> leaves every **organizational decision** as a `<FILL-IN>` placeholder for an adopting
> organization to complete and sign. Remediates gap **GAP-ISO-1**.

## Why this set exists

The ISO 27001 readiness pack in this directory ([`README.md`](./README.md),
[`controls.md`](./controls.md), [`evidence.md`](./evidence.md)) maps sparq's **technical**
Annex A evidence. But ISO 27001 certification also requires the **management-system artifacts**
(clauses 4–10 + the Statement of Applicability) that **no repo file can be** — they are
organizational acts (a signed scope, a risk-treatment decision, a management review, an
internal audit). Gap **GAP-ISO-1** tracked exactly this. This set productionizes those
artifacts into **adoptable Markdown templates** so an organization deploying sparq can build
its ISMS on top of sparq's evidence rather than from scratch.

## The templates (ISMS clauses 4–10 + SoA)

| Template | ISO 27001 clause(s) | What it is | What the org must do |
|---|---|---|---|
| [`isms-scope-template.md`](./isms-scope-template.md) | **4** Context | ISMS scope-definition scaffold: issues (4.1), interested parties (4.2), scope statement (4.3) with the **B3 no-auth** + **ZK/MPC-excluded** interfaces pre-named | Decide & sign the scope, issues, and interested parties |
| [`risk-methodology-template.md`](./risk-methodology-template.md) | **6.1.2 / 6.1.3 / 6.2 / 8.2 / 8.3** | Risk-assessment methodology + acceptance criteria + **risk register** (sparq-seeded candidate risks from the threat model) + risk-treatment plan + objectives | Set the scoring/acceptance criteria; make & sign the risk decisions |
| [`soa-template.md`](./soa-template.md) | **6.1.3(d)** SoA | **Full Annex A 93-control SoA table**: sparq-side status + evidence pre-filled from `controls.md`; org applicability/justification/status/sign-off blank | Decide applicability per control; justify exclusions; top-management sign-off |
| [`internal-audit-programme-template.md`](./internal-audit-programme-template.md) | **9.1 / 9.2** | Monitoring & measurement plan + internal-audit programme schedule + per-audit plan + finding log | Assign **impartial** auditors; run the audits; retain records |
| [`management-review-template.md`](./management-review-template.md) | **9.3 / 10** | Management-review agenda + mandated input/output schema + clause-10 nonconformity & corrective-action log | Hold the review with top management; act on nonconformities |

> **Cross-framework policies.** The information-security *policies* (vulnerability-management /
> CRA disclosure, secure-SDLC, dependency, release-signing) live under the shared
> `compliance/policies/` directory, owned across the `cra` / `ssdf` / `sbom` / `slsa`
> worktrees to avoid duplication. The clause-5 (leadership policy) and clause-7 (support) rows
> of the SoA point to those, not to a duplicate here.

## How the templates fit together

```text
isms-scope-template.md (clause 4)        ── defines scope ──┐
                                                            v
risk-methodology-template.md (6/8) ── risks + treatment ──> soa-template.md (6.1.3d)
        │                                                       │
        │  objectives (6.2)                                     │ controls determined
        v                                                       v
internal-audit-programme-template.md (9.1/9.2) ── findings ──> management-review-template.md (9.3/10)
                                                                    │
                                                          decisions, corrective actions
                                                          feed back into scope + risk
```

The loop is the ISMS PDCA cycle: **scope → plan (risk + SoA) → operate → audit → review →
improve → re-scope.** sparq supplies the *technical evidence substrate* at each step; the
organization supplies every *decision* and *sign-off*.

## The single most important honesty point (restated)

- **No certificate.** Nothing here is, or can be, an ISO 27001 certificate.
- **No org decisions pre-made.** Every applicability, risk, acceptance, and sign-off cell is a
  `<FILL-IN>` placeholder. sparq does not, and cannot, make these for an organization.
- **A.8.24 / ZK-MPC.** The `sparq-zk` / `sparq-zk-compose` / `sparq-mpc` estate was **originally
  found NOT cryptographically sound** (`research/zk-soundness-audit.md`); `sq-1s2` then landed
  the verifier-side binding layer and an **internal** re-audit (`research/zk-verifier-reaudit.md`,
  `sq-gbp4`) found all findings closed → "sound as landed for the assumed threat model," but the
  re-audit is internal/single-model/read-only, **external accredited-cryptographer sign-off is
  STILL PENDING** (`sq-qhy4`, P0) and there is **NO production guarantee** (`SECURITY.md`)
  [OPUS-4.8]. It is **excluded** from every cryptographic-control claim across all five
  templates; no SoA or risk register completion may credit it as a control.
  <!-- [OPUS-4.8] reconciled with post-remediation re-audit (sq-gbp4); see ZK-verdict cross-ref sweep -->

## Provenance

Authored under epic `sq-toze`, bead **sq-ez5z** (productionizing the ISMS artifact set from the
earlier `soa-template.md` scaffold), remediating **GAP-ISO-1**. Single-model Opus 4.8 authorship
while Fable is unavailable — carries the repo's standard `re-review when Fable returns` flag.
