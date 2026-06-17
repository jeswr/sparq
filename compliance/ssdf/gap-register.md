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
(see `controls.md` coverage summary: 28 implemented & verified / 13 audit-ready / **1
gap**, across 42 task rows). The cross-cutting supply-chain/secure-coding gaps that other frameworks share
(GX-1 advisories PR-gate, GX-5 unsafe register, GX-6 secure-coding section, GX-7
cargo-auditable/vet) have **already landed** in the codebase and are cited as evidence in
`controls.md` — they are *not* re-listed as SSDF gaps.

## Open gaps

### SSDF-G1 — PW.6.2: no reproducible-build evidence

| Field | Value |
|---|---|
| **SSDF task** | PW.6.2 (use build features that preserve provenance/reproducibility) |
| **Cross-cutting id** | GX-8 |
| **Severity** | P2 (raises assurance; not a vulnerability) |
| **Bead** | **sq-toze.9** (`[cert][gap GX-8] Reproducible-build evidence`) — already open under epic `sq-toze` |
| **Status** | Open |
| **What's missing** | The release build is `cargo auditable build --release --locked` and SLSA-provenance-attested, but there is **no documented reproducible-build claim or evidence** — neither a "byte-for-byte reproducible from this commit + toolchain" demonstration nor an honest "not reproducible because <X>" statement. SSDF PW.6.2 (and CRA integrity / SLSA higher levels) want one or the other. |
| **Remediation plan** | (1) Attempt a double-build of `sparq-cli` for one target and diff the artifacts; (2) if reproducible, document the pinned inputs (toolchain version, `--locked`, `RUSTFLAGS`, `SOURCE_DATE_EPOCH`) and add a CI job that rebuilds + compares digests; (3) if **not** reproducible, document the specific non-determinism source(s) (e.g. embedded build paths, parallel-codegen ordering, timestamps) as an honest PW.6.2 statement and link it from `controls.md`. Either outcome closes the gap honestly. |
| **Target** | Before a 1.0 release; not a pre-1.0 blocker (the project is explicitly pre-1.0, `SECURITY.md`). |
| **Owner** | @jeswr |

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
