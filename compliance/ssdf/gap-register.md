<!-- [OPUS-4.8] SSDF gap register (cert framework `ssdf`, epic sq-toze, bead sq-toze.13).
     Re-review when Fable returns. NON-CANONICAL timing. -->

# NIST SSDF (SP 800-218) — gap register

Open gaps for the SSDF mapping, each with severity, the affected SSDF task, a remediation
plan, a target, and the **`bd` bead** that tracks the fix (gap-fix beads live under epic
`sq-toze`). A gap is recorded here only when a practice is **not** met by current
codebase/CI evidence — controls that are met are in [`controls.md`](./controls.md), and
*audit-ready* practices (documented + automated, formal attestation = org act) are **not**
gaps and are not listed here.

SSDF coverage is high because it is largely a **mapping** of sparq's existing gate stack
(see `controls.md` coverage summary: 26 implemented & verified / **2 partial** / 13
audit-ready / **1 gap**, across 42 task rows). The cross-cutting supply-chain/secure-coding gaps that other frameworks share
(GX-1 advisories PR-gate, GX-5 unsafe register, GX-6 secure-coding section, GX-7
cargo-auditable/vet) have **already landed** in the codebase and are cited as evidence in
`controls.md` — they are *not* re-listed as SSDF gaps.

## Open gaps

### SSDF-G1 — PW.6.2: reproducible-build (statement DELIVERED; CI enforcement remaining)

| Field | Value |
|---|---|
| **SSDF task** | PW.6.2 (use build features that preserve provenance/reproducibility) |
| **Cross-cutting id** | GX-8 |
| **Severity** | P2 (raises assurance; not a vulnerability) |
| **Bead** | **sq-toze.9** (`[cert][gap GX-8] Reproducible-build evidence`) — already open under epic `sq-toze` |
| **Status** | Open — **PW.6.2 honest statement DELIVERED** ([`../slsa/reproducible-build.md`](../slsa/reproducible-build.md)); bead stays open for the optional CI rebuild-and-diff ratchet. |
| **What's missing** | ~~No documented reproducible-build claim or evidence.~~ **Now documented** — remediation step (3) is done: a measured double-build of `sparq-cli` (`--release --locked`, same tier flags) is **identical size + byte-identical apart from 22 bytes**, all from **one** non-determinism source (the C-compiled `mimalloc` `__DATE__`/`__TIME__` `.rodata` banner + the build-id it perturbs). That is the honest PW.6.2 statement SSDF wanted. What remains for a *byte-for-byte* claim (and to flip the bead to closed): the `SOURCE_DATE_EPOCH`/feature-drop fix + a CI lane that rebuilds and diffs digests. |
| **Remediation plan** | (1) ✓ DONE — double-build + diff of `sparq-cli`, recorded in [`../slsa/reproducible-build.md`](../slsa/reproducible-build.md). (2) result was **not** byte-identical; (3) ✓ DONE — the specific non-determinism source (`mimalloc` build-time `__DATE__`/`__TIME__` banner; the build-id is downstream) is documented as the honest PW.6.2 statement and linked from `controls.md`. REMAINING (keeps the bead open): pin `SOURCE_DATE_EPOCH` (or drop the opt-in `mimalloc` default) for the reproducible artifact + add a CI rebuild-and-diff ratchet → a byte-for-byte claim. |
| **Target** | Before a 1.0 release; not a pre-1.0 blocker (the project is explicitly pre-1.0, `SECURITY.md`). |
| **Owner** | @jeswr |

### SSDF-G2 — PW.7.1 / PW.7.2: SAST is not running (CodeQL disabled; nothing compensates)

<!-- [OPUS-5] Slice entry for cross-cutting gap GX-14; do not duplicate the remediation
     decision here — it is one maintainer decision, tracked in issue #4620. -->

| Field | Value |
|---|---|
| **SSDF tasks** | PW.7.1 (determine + configure code review / analysis), PW.7.2 (perform the review / static analysis). Also struck as evidence from PO.1.3, PO.3.1, PO.3.2 and RV.3.4, which retain their status on other, live evidence. |
| **Cross-cutting id** | **GX-14** (P1) — see [`../gap-register.md`](../gap-register.md) |
| **Severity** | **P1** — a stated control that is switched off is worse than an acknowledged gap, because it stops anyone looking. |
| **Tracking** | **issue #4620** (durable posture decision) · issue #4615 (alert triage + doc reconciliation). No `sq-toze` bead: the remediation is a maintainer *decision*, not an implementable task, until #4620 resolves. |
| **Status** | **OPEN.** |
| **What's missing** | `.github/workflows/codeql.yml` has been **disabled at the Actions level (`disabled_manually`) since 2026-07-18**, by separate maintainer direction (merge latency). The file and its triggers are retained on `main`, but GitHub schedules **no** run on **any** event (push, pull_request, merge_group, schedule): no `CodeQL analysis (rust)` check-run, no SARIF upload to code scanning, nothing fed to `ci-summary`, nothing gated. **No compensating SAST control exists** — clippy `-D warnings`, the unsafe-count ratchet, cargo-deny/cargo-vet, the fuzz lane and the Miri lane are all live and genuine, but **none** performs taint or crypto-misuse analysis. **35 open `critical` `rust/hard-coded-cryptographic-value` alerts** remain; they were **triaged** under issue #4615 and found to be false positives of a single query-model defect (the query matches its sink by parameter name while classifying test code by file path, and Rust colocates `#[cfg(test)]` code inside source files) — **triaged is not covered**, and it says nothing about what an enabled scanner would find now. |
| **Remediation plan** | Not agent-decidable. The durable posture is an **open maintainer decision (#4620)** between: (1) re-enable advisory-only (non-gating, keeps the signal without the merge latency); (2) re-enable on a schedule only; (3) adopt a different SAST tool; (4) accept and document *no* SAST as a standing, disclosed residual. **No replacement control is claimed here** — until one of those lands, SSDF PW.7.1/PW.7.2 stay **Partial** and the static-analysis limb stays unmet. |
| **Target** | Tied to #4620; not self-resolving. Before any 1.0 / external-attestation use of this SSDF mapping. |
| **Owner** | @jeswr (maintainer decision) |

## Watch items (not gaps — evidence present, bead still open)

These are **not** SSDF gaps: the technical control is present and cited in `controls.md`.
They are listed only so the auditor knows the corresponding `sq-toze` bead is still open
(closing it is a bookkeeping act once the lead consolidates), so there is no
record-vs-reality drift.

| Item | SSDF tasks | Evidence (present) | Bead (open) |
|---|---|---|---|
| cargo-auditable + cargo-vet | PW.4.1, PS.3.2, PW.6.1 | `supply-chain.yml#vet` (gating), `release.yml` (`cargo auditable build`), `supply-chain/config.toml` | sq-toze.8 (GX-7) |

> The bead for the **advisories PR-gate** (GX-1, sq-toze.2), the **per-release SBOM+VEX**
> (GX-2, sq-toze.3), **`security.txt`** (GX-3, sq-toze.4), the **unsafe register + ratchet**
> (GX-5, sq-toze.6), and the **secure-coding section** (GX-6, sq-toze.7) are all **closed**
> and their controls are cited as met evidence in `controls.md` — they are not gaps.

## Discovered work (new beads filed by this SSDF pass)

This pass filed the following bead(s) under epic `sq-toze` for SSDF-specific follow-up that
the existing gap-fix beads did not already cover. (`bd` beads are created from the main
checkout per repo policy; they are referenced here, never hand-edited into `.beads/`.)

| Bead | Title | Why |
|---|---|---|
| **sq-5ty0** | SSDF PO.1.1 — publish a stand-alone Secure-SDLC policy template | **DELIVERED** — [`../policies/policy-secure-sdlc.md`](../policies/policy-secure-sdlc.md) consolidates the PO.1.1/PO.2.1 substance (previously evidenced mapping-level by `CONTRIBUTING.md` + `SECURITY.md` + the threat model) into one org-adoptable policy template with `<FILL-IN>` sign-off placeholders. It lifts PO.1.1/PO.2.1 from audit-ready toward a clean org-level attestation; the *signature* itself remains an org act. Not a codebase gap — a policy-template deliverable. Filed under epic `sq-toze` (blocks it). |
