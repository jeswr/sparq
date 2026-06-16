<!-- [OPUS-4.8] sq-toze — ISO/IEC 27001 gap register. Honest open readiness gaps +
     remediation beads. Authored while Fable unavailable — re-review when Fable returns. -->

# ISO/IEC 27001:2022 — gap register

Open gaps for the ISO 27001 readiness mapping, with severity, remediation, and the `bd`
bead that tracks each. Read [`controls.md`](./controls.md) and [`README.md`](./README.md)
first.

> `bd` is **not available in this isolated worktree**, so beads are listed here for the
> orchestrator to create on the main checkout (`bd create … --epic sq-toze`); they are
> **NOT** hand-edited into `.beads/`. Each remediation names the exact deliverable.

## Framing — why this register is short, and why that is honest

ISO 27001 has **zero open Annex-A *control* gaps** for sparq, because the gaps that would
otherwise appear are correctly classified one of two ways (and so are *not* gaps in the
"sparq must fix code" sense):

1. **AUDIT-READY** — the control needs an *organization* to run an ISMS (a signed policy,
   a risk-treatment decision, a management review, an accredited internal audit). The repo
   supplies the doc-of-record; the certificate is an external/org act we cannot substitute
   for. Papering these over as "PASS" would be the exact overclaim the honesty contract
   forbids.
2. **N/A (operator)** — physical/operational controls of a *deployed* environment, owned by
   the adopting operator, not by sparq's source.

What *does* remain are **two readiness gaps**: the organizational ISMS artifact set
(needed for an actual audit, GAP-ISO-1) and an explicit operator-deployment-security
guidance doc (so the N/A(operator) controls aren't left implicit, GAP-ISO-2). These are
**documentation/templates**, not code controls. One earlier suspected gap (CODEOWNERS) was
verified **false** and is recorded as resolved-on-inspection below.

## OPEN gaps

| ID | Gap | Sev | Remediation | Bead (to create) |
|---|---|---|---|---|
| **GAP-ISO-1** | **No organizational ISMS artifact set.** ISO 27001 certification needs the management-system artifacts that no repo file can be: a documented **ISMS scope statement**, a **risk assessment + risk-treatment plan**, a **Statement of Applicability (SoA)** mapping each of the 93 Annex A controls to applicable/justification/status, a **management-review** record, and an **internal-audit programme**. The repo has the *technical evidence* and *docs-of-record* (`SECURITY.md`, threat model, `CONTRIBUTING.md`, this mapping) that an SoA would cite, but the SoA + ISMS clauses 4–10 are an org act. | **High** (blocks certification, not security) | Provide org-adoptable **policy + SoA templates** under `compliance/policies/` (ISMS scope, risk-treatment, an SoA skeleton seeded from `controls.md`, incident-response plan), clearly marked **templates needing org sign-off**. This mapping (`controls.md`) is the SoA's applicability column; the templates are the remaining clauses-4–10 scaffolding. **No accredited certificate is in agent scope** — label it external. | `iso27001: org-adoptable ISMS policy + SoA templates (clauses 4-10 scaffold)` (P1, sq-toze) |
| **GAP-ISO-2** | **The N/A(operator) controls are implicit, not documented in one place for the operator.** 42 Annex A controls are correctly the adopting *operator's* responsibility (A.7 physical, access-control families under boundary B3, runtime monitoring/backup/availability, network controls). They are flagged per-row in `controls.md`, but there is no single **operator security-deployment guidance** doc telling an operator "to run sparq-server safely you MUST: front it with an authenticating/TLS-terminating gateway (B3), set resource/`QueryBudget` limits, run it non-root in the distroless image, restrict network exposure, own backup/monitoring of your data." Without it, the operator-vs-sparq split is asserted but not actionable. | **Medium** | Author `compliance/iso27001/operator-responsibilities.md` (or fold into the cross-cutting `compliance/threat-model.md` + `compliance/data-flow.md` the privacy worktree owns) enumerating the operator-owned controls with the concrete action for each, anchored on the Dockerfile guidance + boundary B3 + `QueryBudget`. Cross-reference from `controls.md` N/A(op) rows. | `iso27001: operator deployment-security responsibilities doc (B3 + N/A(op) controls)` (P2, sq-toze) |

## Resolved on inspection (recorded so the auditor sees the check was made)

| ID | Suspected gap | Verdict |
|---|---|---|
| ~~GAP-ISO-3~~ | An early draft of `evidence.md` mis-checked `.github/CODEOWNERS` (wrong path) and suspected an empty/missing CODEOWNERS, which the A.5.2/A.5.3/A.8.4 claims depend on. | **FALSE / RESOLVED.** `CODEOWNERS` is at the **repo root** (a valid GitHub location), 37 lines, with a catch-all `* @jeswr` and explicit owner lines for the high-risk paths (`sparq-zk*`, `sparq-mpc`, `sparq-core`, `sparq-server`, CI). No gap. `evidence.md` corrected. |

## Explicitly NOT a gap (documented decisions — do not re-flag)

- **A.5.15 / A.8.3 / A.8.5 access control on `sparq-server`** is a **documented architectural
  decision** (threat-model boundary **B3**: no per-user authz; front with a gateway /
  sparq-solid; one optional bearer token exists). It is **operator-owned by design**, not a
  silent gap. Captured in `controls.md` (N/A(op)→AUDIT-READY rows) and to be made actionable
  by GAP-ISO-2. Re-raising it as a sparq code gap would contradict the cert plan §0 and the
  threat model.
- **A.8.24 cryptography over the ZK/MPC estate** is **not a gap to "fix here"** — it is the
  documented **NOT-sound** verdict (`SECURITY.md`, `research/zk-soundness-audit.md`),
  assessed by the `cryptoreview` framework and remediated by the existing ZK soundness
  beads, **not** this epic. Treating it as an ISO 27001 control gap would mislabel a
  research-scaffold disclaimer as an ISMS finding.

## External / out-of-agent-scope (label, do not claim)

- **The ISO/IEC 27001 certificate itself** is issued only by an **accredited certification
  body** after a Stage 1 + Stage 2 audit of an operating ISMS over time. Nothing in this
  repo or this directory is, or can be, that certificate. This pack is *readiness*: it gives
  such an auditor the technical-evidence half of the SoA. Stated plainly in `README.md`.
