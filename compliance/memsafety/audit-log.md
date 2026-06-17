<!-- [OPUS-4.8] sq-toze — memsafety engineer↔auditor loop log + final verdict.
     Re-review when Fable returns. -->

# Memory-safety attestation — engineer↔auditor loop log

Epic `sq-toze`. Branch `cert-memsafety-certification`. This is the **proof-of-pattern** run
before the loop fans out to the other 11 frameworks. The loop ran the engineer and auditor
roles adversarially against the *actual repo*, not the docs.

> **Count note (sq-pro0, [OPUS-4.8]).** The crate counts quoted in the rounds below are the
> point-in-time figures verified *at audit time* (20 `#![forbid(unsafe_code)]` crates / 25
> total) and are preserved as the historical record — **do not rewrite them**. The workspace
> has since grown: the *current* figures are **26 `#![forbid(unsafe_code)]` crates of 31
> total**, with the unsafe surface still confined to the same 5 crates and the 56-site
> per-crate register unchanged. The live count lives in `unsafe-register.md` + `evidence.md`
> §MS-1; this log is not re-verified per crate-count change.

## Round 0 — engineer

Authored `controls.md` (13 controls + a B5 coverage matrix), `evidence.md` (re-runnable
verification per control), `gap-register.md` (closed GX-5; 4 honest OPEN gaps). Every cited
artifact was verified to exist *before* it was cited: re-ran `scripts/unsafe-gate.py
--check` (56==56 PASS) and `--list` (TOTAL=56); confirmed 20 `forbid(unsafe_code)` crates;
confirmed `mmap_corruption_oracle.rs`, `graph_open.rs`, `miri.yml`, the gating `cargo deny
check advisories`, and the `ci.yml` ratchet job exist and are wired as claimed.

## Round 1 — auditor → `FINDINGS: 3` (1 medium, 2 low)

`compliance/audit/memsafety-findings-1.md`. Re-verified the gating logic independently
against the `ci-summary` aggregator's whole-word `\b(advisory|informational)\b` exclusion
rule and the `ci.yml` PR/merge_group triggers — confirmed the ratchet genuinely gates. ZK
honesty tripwire clear. Findings, all documentation-accuracy (not safety defects):
- **F1 (medium):** the authoritative `unsafe-register.md` still asserted clippy
  `undocumented_unsafe_blocks` "is the local enforcement" — but that lint is enabled only in
  `vendor/spargebra`, not first-party. A live overclaim in the spine doc.
- **F2 (low):** register's "every site has a `// SAFETY:` comment" is literally false (50/56
  use the token; 6 use an adjacent block comment).
- **F3 (low):** `research/threat-model.md` says 39 sparq-core sites vs the register's 42.

## Round 1 fix — engineer

Edited `unsafe-register.md`: line 45 now states enforcement is the register + review +
ratchet, with the clippy lint labelled recommended-but-not-yet-enabled (→ MS-G2); the
NEEDS-REVIEW section now says "50 via the literal token, 6 via adjacent comment". F3 left as
tracked gap MS-G5 (threat-model is owned by another surface).

## Round 2 — auditor → `FINDINGS: 0` → **SIGN-OFF**

`compliance/audit/memsafety-findings-2.md`. F1 + F2 verified resolved in the register; F3
accepted as tracked MS-G5. Sign-off with the standing external caveats (formal
verification + accredited third-party audit are external by definition).

## Final verdict (auditor)

**PASS** on every applicable codebase/CI memory-safety control, with **4 honestly-recorded
OPEN gaps** (none a safety defect):

| Status | Controls |
|---|---|
| **PASS-with-evidence** | MS-1, MS-2, MS-3, MS-4, MS-6, MS-7, MS-8, MS-10, MS-11, MS-12, MS-13 (11) |
| **PASS-with-caveat** (tracked gap) | MS-5 (→MS-G2), MS-9 (→MS-G3) (2) |
| **AUDIT-READY** (external) | formal proof (MS-G4) + accredited third-party audit |

Open gaps: **MS-G2** (enable clippy `undocumented_unsafe_blocks` + normalise 6 sites,
medium/P1), **MS-G3** (standalone ASan lane over the oracle, low/P2), **MS-G4** (Kani/formal
proof of mmap validators, low/P2), **MS-G5** (threat-model 39→42 doc sync, low/P2).

**Rounds to convergence: 2** (one findings round, one sign-off round).

## Pattern readiness for fan-out

The loop pattern is **ready to fan out** to the other 11 frameworks (asvs, cis, sbom,
iso27001, iso27701-folded-into-privacy, iso27017-18-CUT, soc2-folded, gdpr-folded, cyberess-
CUT-replaced-by-cra, cdmc, crypto). What worked and should carry over:
1. **Verify-before-cite in the engineer pass** kept round-1 findings to documentation
   accuracy, not fabricated evidence — the most expensive auditor failure mode (overclaiming
   a control that doesn't exist) never occurred. Brief every engineer to re-run/re-read each
   citation before writing the row.
2. **The auditor independently re-deriving the gating logic** (not trusting the engineer's
   "this gates") is what gives the sign-off weight; budget the auditor a real source pass.
3. **`bd` is unavailable in the isolated worktrees** — gaps must be listed for the
   orchestrator to create as beads. Bake that into every brief.
4. Memsafety had *pre-existing strong evidence* (the GX-5 register/ratchet from PR #217), so
   it converged in 2 rounds. Frameworks with more *missing* technical controls (sbom GX-1/2,
   cis Trivy lane, openssf Best-Practices badge) will likely need a real code/CI change +
   green-gate cycle before the auditor can sign off — expect more rounds there, and gate
   their fan-out on the cross-cutting gap-fix beads landing first (per the orchestration
   runbook's stage gate).
