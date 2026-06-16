<!-- [OPUS-4.8] sq-toze — memsafety gap register. Honest open gaps + remediation beads.
     Re-review when Fable returns. -->

# Memory-safety attestation — gap register

Open gaps for the memsafety framework, with severity, remediation, and the `bd` bead that
tracks the fix. The headline GX-5 gap (no unsafe register + cargo-geiger informational-only)
from `research/production-certification-plan.md` is **CLOSED** by this framework's work
(register + gating ratchet — see below). The remaining gaps are the honest residue: a
documentation overclaim, two deferred deeper-coverage lanes, a formal-verification ceiling,
and a cross-doc count drift.

> `bd` is not available in this isolated worktree, so beads are listed here for the
> orchestrator to create (`bd create … --epic sq-toze`); they are NOT hand-edited into
> `.beads/`. Each remediation names the exact change.

## CLOSED by this framework (was GX-5 — P0)

| ID | What | Evidence it is closed |
|---|---|---|
| GX-5a | No per-site unsafe justification register | `compliance/memsafety/unsafe-register.md` — 56 rows, 100% of first-party sites. |
| GX-5b | cargo-geiger informational only (no gating ratchet) | `scripts/unsafe-gate.py` + `bench/unsafe-snapshot.json` + the **gating** `unsafe-register (count ratchet)` CI lane in `ci.yml`. cargo-geiger stays as a separate visibility lane. |
| GX-5c | B5 mmap sites "not attested in one place" | The coverage matrix in `controls.md` + the oracle (`mmap_corruption_oracle.rs`) + fuzz (`graph_open.rs`) citations attest each B5 site. |
| **MS-G2** | First-party `clippy::undocumented_unsafe_blocks` lint missing (the `// SAFETY:` token was enforced by review + register, not mechanically) | **CLOSED (sq-8wbn, [OPUS-4.8]).** `#![warn(clippy::undocumented_unsafe_blocks)]` is now a crate-root attribute on all 5 unsafe-bearing crates (sparq-core, sparq-vectors, sparq-cli, sparq-zk-compose, sparq-bench); the 6 sites the lint flagged were normalised to a literal `// SAFETY:` comment immediately preceding each `unsafe` block/impl (the two `SlotPtr` `Send`/`Sync` pairs in `dict.rs`/`dictspill.rs`, plus `lib.rs` `from_raw_parts_mut` over `MmapMut` and the test `remove_var`). The existing `clippy --all-targets -D warnings` gate now mechanically rejects any new undocumented `unsafe`. |

## OPEN gaps

| ID | Gap | Sev | Remediation | Bead (to create) |
|---|---|---|---|---|
| **MS-G3** | **No standalone AddressSanitizer lane** outside cargo-fuzz. ASan currently runs only *inside* the fuzz build (MS-9), so ASan coverage of the B5 reads is only as deep as the fuzz corpus reaches; the deterministic oracle (MS-7) runs *without* ASan. | **Low** | Add a CI lane that runs `crates/sparq-core/tests/mmap_corruption_oracle.rs` (+ the vector/diskann corruption tests) under `-Zsanitizer=address` on the gnu target (nightly), so the deterministic corruption corpus executes under ASan. Deferred in the original miri.yml header; this makes it a tracked gap, not a silent omission. | `memsafety: ASan lane over the corruption oracle (not just fuzz)` (P2, sq-toze) |
| **MS-G4** | **No formal verification / model checking of the unsafe core.** Assurance is Miri + oracle + fuzz + per-site argument (strong *testing/justification*) but not a *proof* of the mmap validators (`MappedDict::validate`, `VectorStore::open_validated`, DiskANN `open`). | **Low** (assurance ceiling, not a defect) | Evaluate Kani (bounded model checking of the offset/length validators) or a Prusti/Creusot annotation of the `validate` functions; treat as the certificate-grade external/formal-methods step. AUDIT-READY label stands until then. | `memsafety: evaluate Kani bounded-proof of the mmap validators` (P2, sq-toze) |
| **MS-G5** | **Cross-doc unsafe-count drift.** `research/threat-model.md` says sparq-core has "39 sites"; the register/snapshot/ratchet say **42**. The register is authoritative; the threat-model prose is stale. | **Low** | One-line fix to `research/threat-model.md` (owned by the threat-model doc, not this framework) — update "39" → "42" (or reference the ratchet snapshot so it can't drift again). | `docs: sync threat-model unsafe-count to the ratchet snapshot (39→42)` (P2, sq-toze) |

## NOT gaps (decisions, recorded so the auditor does not re-flag them)

- **Miri does not gate per-PR.** Intentional — Miri is ~100× native; it is a nightly UB
  safety net (like fuzz-nightly / zk-toolchain). The per-PR gate is the *ratchet* (MS-3) +
  `clippy -D warnings` (MS-10) + the oracle/fuzz smoke. Not a gap.
- **Miri does not run the 16 mmap + 7 dict-spill sites.** Structural (Miri rejects
  file-backed mappings) — covered by the oracle + fuzz+ASan instead. Documented, not a gap.
- **Third-party `unsafe` (memmap2/libc/rayon/hdt) is not in this register.** Scoped to the
  supply-chain lane (cargo-deny, SBOM/VEX). Correct separation of concerns, not a gap.

## Honest overall posture

The memsafety framework is **substantively PASS**: the unsafe surface is confined (20 forbid
crates), fully enumerated + justified (56-site register), gated by a merge-blocking ratchet,
and covered by a Miri+oracle+fuzz+ASan matrix with the B5 boundary explicitly handled. The
open gaps are **not** missing safety — they are (a) [CLOSED] the first-party
`undocumented_unsafe_blocks` lint, now enforced on all 5 unsafe crates (MS-G2, sq-8wbn),
(b) two *deeper-assurance* lanes that are deferred-but-tracked (MS-G3 ASan-standalone,
MS-G4 formal proof), and (c) a stale number in a neighbouring doc (MS-G5). None of these
contradicts the safety claim; the certificate-grade ceiling (formal proof + external audit) is
external by definition and labelled AUDIT-READY.
