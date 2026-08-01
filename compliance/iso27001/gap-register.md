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

ISO 27001 has **zero Annex-A controls at status `GAP`** for sparq — though **two are now
`PARTIAL`** (A.8.7, A.8.28) under the cross-cutting **GX-14** SAST gap recorded below, which is a
real technical residual and not a classification artefact. The remaining potential gaps are
correctly classified one of two ways (and so are *not* gaps in the "sparq must fix code" sense):

1. **AUDIT-READY** — the control needs an *organization* to run an ISMS (a signed policy,
   a risk-treatment decision, a management review, an accredited internal audit). The repo
   supplies the doc-of-record; the certificate is an external/org act we cannot substitute
   for. Papering these over as "PASS" would be the exact overclaim the honesty contract
   forbids.
2. **N/A (operator)** — physical/operational controls of a *deployed* environment, owned by
   the adopting operator, not by sparq's source.

What remained were **two readiness gaps**: the organizational ISMS artifact set (needed for
an actual audit, **GAP-ISO-1**) and an explicit operator-deployment-security guidance doc (so
the N/A(operator) controls aren't left implicit, **GAP-ISO-2**). Both are
**documentation/templates**, not code controls, and **both in-repo deliverables are now
addressed**:

- **GAP-ISO-1's in-repo deliverable** is addressed by the org-adoptable ISMS template set
  (clauses 4–10 + the full Annex A SoA, bead `sq-ez5z`) — see the *ADDRESSED by the
  org-adoptable ISMS template set* section below; its **certificate residual remains external**
  and must never be claimed as closed.
- **GAP-ISO-2** is addressed (bead `sq-v48f`) by
  [`operator-deployment-security.md`](./operator-deployment-security.md) — see the *ADDRESSED in
  this directory* section below.

So there are **no remaining open readiness gaps owned by this slice**; the residuals are (a) the
**external certificate act** under GAP-ISO-1 (labelled, never claimed as closed) and (b) the
cross-cutting, **open and technical** SAST gap **GX-14** recorded in the OPEN-gaps section below.
One earlier suspected gap (CODEOWNERS) was verified **false** and is recorded as
resolved-on-inspection below.

## OPEN gaps

_No open readiness gaps **owned by this slice**._ Both readiness gaps have their
in-repo deliverable addressed — GAP-ISO-1 by the ISMS template set (below) and GAP-ISO-2 by the
operator-deployment-security doc (below). The **only residual is the external certificate act**
under GAP-ISO-1, which is labelled in the residual note and the external section, and **must
never be claimed as closed**.

**But one *cross-cutting* open gap now bears on this slice's control statuses:**

| ID | Gap (anchor — not restated here) | Sev | Effect on this slice |
|---|---|---|---|
| **GX-14** | **SAST is not running** — CodeQL is operationally disabled and nothing compensates. Full statement, evidence and remediation options live in the top-level [`../gap-register.md`](../gap-register.md) (and `ASSURANCE.md` §11); the durable-posture decision is open maintainer issue **#4620**. Do **not** restate it here. | **P1** | **A.8.7** and **A.8.28** downgraded **IMPL → PARTIAL** in [`controls.md`](./controls.md) + [`soa-template.md`](./soa-template.md). CodeQL struck from the evidence of **A.5.6, A.5.7, A.5.8, A.5.35, A.8.8, A.8.25** (each stands on its other named controls). The **A.5.36** claim that `ci-summary` gates on a CodeQL lane was **false** and is corrected. The `evidence.md` SAST step — a `grep` over the intact workflow *file* — was **misleading verification** and is replaced with a workflow-*state* check. So the Annex-A roll-up is now **22 IMPL / 2 PARTIAL / 27 AUDIT-READY / 42 N/A(op)**. |

## ADDRESSED in this directory

| ID | Gap | Sev | How addressed |
|---|---|---|---|
| **GAP-ISO-2** | **The N/A(operator) controls were implicit, not documented in one place for the operator.** 42 Annex A controls are correctly the adopting *operator's* responsibility (A.7 physical, access-control families under boundary B3, runtime monitoring/backup/availability, network controls). They were flagged per-row in `controls.md`, but there was no single **operator security-deployment guidance** doc making the operator-vs-sparq split actionable. | **Medium** | **ADDRESSED** (sq-v48f) by [`operator-deployment-security.md`](./operator-deployment-security.md): a 9-section operator-responsibility doc enumerating network/TLS, authN/authZ (boundary B3), secrets, OS/container hardening, resource/DoS limits, logging/PII, backup/durability, and patch cadence — each stating what sparq ships built-in (citing the real flag/feature) vs what the operator MUST supply, mapped to Annex A. It is an **operator-responsibility doc, NOT a certification claim**, and it states sparq's auth/crypto limits honestly (one coarse static Bearer token, no per-user authz; ZK/MPC estate carries **no production guarantee** — v1 verifier originally found NOT sound then remediated (`sq-1s2`) + **internally** re-audited "sound as landed for the assumed threat model," external sign-off STILL PENDING `sq-qhy4`) [OPUS-4.8]. Cross-referenced from `controls.md` (the B3 / N/A(op) rows point at it). |

## ADDRESSED by the org-adoptable ISMS template set (in-repo deliverable; certificate stays external)

| ID | Gap | Status | What was delivered (bead sq-ez5z, epic sq-toze) |
|---|---|---|---|
| **GAP-ISO-1** | No organizational ISMS artifact set (scope, risk assessment + treatment, SoA, management review, internal audit). | **ADDRESSED — templates delivered.** The *in-repo, agent-scoped* part of the remediation is complete: the ISMS clauses-4–10 + SoA artifacts are now org-adoptable Markdown templates with `<FILL-IN>` placeholders. **The certificate itself remains an external organizational act** and is NOT closed by this — see the residual row below. | [`isms-templates-README.md`](./isms-templates-README.md) (index); [`isms-scope-template.md`](./isms-scope-template.md) (clause 4); [`risk-methodology-template.md`](./risk-methodology-template.md) (clauses 6/8, risk methodology + register seeded from the threat model); [`soa-template.md`](./soa-template.md) **productionized to the full Annex A 93-control SoA table** (sparq-side status + evidence from `controls.md`; org columns blank); [`internal-audit-programme-template.md`](./internal-audit-programme-template.md) (clauses 9.1/9.2); [`management-review-template.md`](./management-review-template.md) (clauses 9.3/10). Cross-framework *policies* (vuln-mgmt/CRA, SDLC, dependency, release-signing) remain owned under `compliance/policies/` by the cra/ssdf/sbom/slsa worktrees — referenced, not duplicated. |

> **GAP-ISO-1 residual (external — never claim as closed).** The ISMS templates are the head
> start; running the ISMS over time and obtaining the certificate is an act of an **adopting
> organization + an accredited certification body**. The consolidated cross-framework register
> tracks this residual (the "ISMS / Statement-of-Applicability org act") under GAP-ISO-1 (P1) /
> the external-residuals table. Populating the templates does **not** make sparq "ISO 27001
> certified."

## Resolved on inspection (recorded so the auditor sees the check was made)

| ID | Suspected gap | Verdict |
|---|---|---|
| ~~GAP-ISO-3~~ | An early draft of `evidence.md` mis-checked `.github/CODEOWNERS` (wrong path) and suspected an empty/missing CODEOWNERS, which the A.5.2/A.5.3/A.8.4 claims depend on. | **FALSE / RESOLVED.** `CODEOWNERS` is at the **repo root** (a valid GitHub location), 37 lines, with a catch-all `* @jeswr` and explicit owner lines for the high-risk paths (`sparq-zk*`, `sparq-mpc`, `sparq-core`, `sparq-server`, CI). No gap. `evidence.md` corrected. |

## Explicitly NOT a gap (documented decisions — do not re-flag)

- **A.5.15 / A.8.3 / A.8.5 access control on `sparq-server`** is a **documented architectural
  decision** (threat-model boundary **B3**: no per-user authz; front with a gateway /
  sparq-solid; one optional bearer token exists). It is **operator-owned by design**, not a
  silent gap. Captured in `controls.md` (N/A(op)→AUDIT-READY rows) and made actionable by the
  [`operator-deployment-security.md`](./operator-deployment-security.md) doc (GAP-ISO-2
  ADDRESSED). Re-raising it as a sparq code gap would contradict the cert plan §0 and the
  threat model.
- **A.8.24 cryptography over the ZK/MPC estate** is **not a gap to "fix here"** — the v1 ZK
  verifier was **originally found NOT sound** (`research/zk-soundness-audit.md`, kept on record
  for the `sq-1gir` regression map); `sq-1s2` then landed the verifier-side binding layer and an
  **internal** re-audit (`research/zk-verifier-reaudit.md`, `sq-gbp4`) found all findings closed
  → "sound as landed for the assumed threat model," with **external accredited-cryptographer
  sign-off STILL PENDING** (`sq-qhy4`, P0) and **NO production guarantee** (`SECURITY.md`)
  [OPUS-4.8]. It is assessed by the `cryptoreview` framework and tracked by the ZK soundness
  beads, **not** this epic. Treating it as an ISO 27001 control gap would mislabel a
  research-scaffold disclaimer as an ISMS finding.
  <!-- [OPUS-4.8] reconciled with post-remediation re-audit (sq-gbp4); see ZK-verdict cross-ref sweep -->

## External / out-of-agent-scope (label, do not claim)

- **The ISO/IEC 27001 certificate itself** is issued only by an **accredited certification
  body** after a Stage 1 + Stage 2 audit of an operating ISMS over time. Nothing in this
  repo or this directory is, or can be, that certificate. This pack is *readiness*: it gives
  such an auditor the technical-evidence half of the SoA. Stated plainly in `README.md`.
