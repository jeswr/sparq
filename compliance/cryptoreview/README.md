<!-- [OPUS-4.8] sq-toze / sq-toze.20 — Cryptographic-review framework readiness doc.
     THE most honesty-critical framework. Authored by Opus 4.8 while Fable 5 unavailable —
     re-review when Fable returns. Engineer↔auditor loop (epic sq-toze). -->

# Cryptographic review — readiness (sparq crypto estate)

**Framework:** Cryptographic review of sparq's crypto estate.
**Bead:** `sq-toze.20` (this framework) under epic `sq-toze`; the ZK/MPC remediation it
references lives under epic `sq-1s2`.
**Companion docs:** [`controls.md`](./controls.md) (control → status → evidence table),
[`evidence.md`](./evidence.md) (per-claim file/line/test verification),
[`gap-register.md`](./gap-register.md) (open gaps + the **external-cryptographer
requirement** + beads), [`fips-posture.md`](./fips-posture.md) (the explicit
**no-FIPS / no-CMVP** negative claim for a FIPS-constrained operator, CR-G4),
[`side-channel-analysis.md`](./side-channel-analysis.md) (the **source-level**
constant-time / side-channel review per sensitive op, CR-G5 / CR-14).

> **This is a readiness document, not a certificate.** It inventories the crypto, separates
> what is production-grade-and-sound from what is research-only-and-NOT-yet-externally-verified,
> and states plainly what an accredited external cryptographer must still do. It does **not**
> certify the ZK/MPC estate as secure, because nobody outside this project has.

## 0. The single most load-bearing sentence in this doc

**No external accredited cryptographer has reviewed any part of sparq's bespoke
cryptography. The ZK verifier-soundness claim rests entirely on sparq's own internal,
single-model (Opus 4.8) self-audits, and `sparq-mpc` carries NO security guarantee. An
external cryptographer audit is REQUIRED before any ZK/MPC security, privacy, integrity, or
attestation claim may be relied upon in production.** (Gap `CR-G1`, severity CRITICAL — see
[`gap-register.md`](./gap-register.md).)

Everything below refines that sentence; nothing below softens it.

## 1. The crypto estate, split by assurance

sparq's cryptography falls into two cleanly separable tiers. The split is load-bearing — the
honesty failure this framework exists to prevent is letting Tier-B borrow Tier-A's assurance.

### Tier A — production-grade, SOUND (third-party, standard, externally-vetted primitives)

These are **not bespoke**. They are well-known primitives / services used in the standard
way, vetted outside this project, and they protect the artifacts a downstream dependency
consumer actually relies on.

| Item | What it is | Why it is sound |
|---|---|---|
| **Sigstore build-provenance attestation** | `actions/attest-build-provenance` over every release archive, the CycloneDX SBOM, and the VEX (`.github/workflows/release.yml`); buildkit `provenance: mode=max` on the container image. Verifiable with `gh attestation verify`. | Sigstore (Fulcio/Rekor + Cosign) is an externally-operated, independently-audited transparency-log + keyless-signing system. sparq *consumes* it correctly; it does not roll its own. **This is real and sound.** |
| **SHA-256 release digests** | `SHA256SUMS` over every release artifact (`release.yml`). | SHA-256 is a FIPS 180-4 approved hash used for integrity, in the standard way. |
| **RDFC-1.0 (RDFC10) canonicalization** | W3C RDF Dataset Canonicalization via the maintained `rdf-canon` crate, W3C-test-suite-validated (`crates/sparq-zk/src/canon.rs`, `crates/sparq-zk/tests/rdf_canon_suite.rs`). | A published W3C Recommendation algorithm in a maintained, test-suite-conformant implementation. It is a *canonicalization* algorithm (deterministic labeling), not a security primitive on its own — sound for what it is. |
| **CSPRNG sourcing** | `getrandom` (OS entropy) for the per-graph salt mint; `rand_chacha::ChaCha20Rng` seeded from OS entropy for MPC secret-sharing masks (`sparq-zk/src/ingest.rs`, `sparq-mpc/src/rng.rs`). The insecure deterministic PRNG is gated behind an OFF-by-default `insecure-test-rng` feature whose name is the warning. | ChaCha20 is a vetted CSPRNG; OS entropy seeding is the standard pattern. Correct *sourcing* of randomness (a necessary, not sufficient, condition for the Tier-B protocols above it). |

**Containment fact (honesty-positive):** all three bespoke-crypto crates (`sparq-zk`,
`sparq-zk-compose`, `sparq-mpc`) are `publish = false` and are **NOT** dependencies of
`sparq-wasm`. They are never shipped to crates.io / npm / PyPI and never enter the browser
bundle. A downstream consumer of the published sparq artifacts **cannot reach the Tier-B
code at all** unless they deliberately pull the crate by git path and opt into it. (Verified:
`grep 'publish = false'` in all three manifests; `sparq-zk`/`sparq-mpc` absent from
`crates/sparq-wasm/Cargo.toml`.)

### Tier B — research scaffold, NOT externally verified (bespoke ZK/MPC)

This is sparq's novel cryptography. It is the subject of the soundness audits. **It must
never be presented as providing the security property it models without an external
cryptographer's sign-off.**

| Item | What it models | Assurance status (HONEST) |
|---|---|---|
| **`sparq-zk` + `sparq-zk-compose`** (+ Noir circuits under `zk/`) | A zero-knowledge proof that a SPARQL query result is faithfully derived from issuer-attested RDF credentials, without revealing the underlying graph. | **NOT externally verified.** An internal 2026-06-13 audit found the v1 verifier **BROKEN** (12 findings, 5 CRITICAL — `research/zk-soundness-audit.md`). A subsequent internal *re-audit* of the remediated verifier (`research/zk-verifier-reaudit.md`, bead `sq-gbp4`) found it **"sound as landed for the assumed threat model"** — **but that re-audit is itself internal, single-model (Opus 4.8, Fable unavailable, self-flagged "re-review when Fable returns"), read-only, with the cryptographic-chain forge tests `#[ignore]`d out of default CI.** Self-review is **necessary but not sufficient.** See [`controls.md`](./controls.md) CR-2/CR-3 for the precise calibration. |
| **`sparq-mpc`** | Multi-party computation over federated SPARQL (confidentiality / correctness / attestation / malicious-security across distrusting holders). | **NO security guarantee — explicitly deferred.** Honest-majority semi-honest Shamir sharing exists (M0–M3), but malicious security, the collaborative-proof / distributed-signature core (M4, Q1), and authenticated-input MPC are **DEFERRED behind honest `NotYetImplemented` stubs** (`crates/sparq-mpc`, `research/mpc-m4-distributed-sig-feasibility.md`). Per `SECURITY.md`: "no confidentiality, correctness, attestation, or malicious-security guarantee today." |

## 2. The canonical public posture (do not contradict)

`SECURITY.md` is sparq's canonical, published security posture. It states the v1 ZK verifier
is **NOT sound** and that a relying party must **"treat any 'verified' result from it as
untrusted,"** and that `sparq-mpc` provides **no guarantee**. This document **does not
override** `SECURITY.md`. The internal re-audit's more favourable "sound as landed" verdict
is recorded here as *internal evidence of remediation progress*, **not** as a basis to soften
the published disclaimer — because (a) it is internal/self-review and (b) external sign-off
(CR-G1) has not happened. **Until CR-G1 closes, `SECURITY.md`'s conservative posture is the
one a relying party must follow.**

## 3. What is in / out of scope for this framework

**In scope:** inventory of primitives & libraries; the sound/not-sound split; the documented
soundness-gap status; the internal review controls that *do* exist (the two audits, the
forge-gate negative-test suite, the gate-count ratchet, the differential fuzzer, CSPRNG +
constant-time notes, FIPS considerations); and the explicit statement of the external
requirement.

**Out of scope (external / non-agent):**
- **The external accredited-cryptographer audit itself** (CR-G1) — an agent cannot substitute
  for it. This doc is the evidence pack such an auditor would consume.
- **Formal verification** of the circuits / verifier binding layer (e.g. a machine-checked
  proof of `reconstruct_public_inputs` ↔ circuit `main` correspondence). The current assurance
  is code-reading + an empirical golden-vector anchor + the forge suite — strong testing, not
  proof.
- **FIPS 140-3 cryptographic-module validation.** sparq uses no validated module and makes no
  FIPS claim (CR-7 / CR-G4); BN254/Poseidon2/Schnorr-over-Baby-JubJub are not FIPS-approved
  algorithms and are not intended to be.
- **The crypto remediation work itself** (the in-circuit privacy upgrades, MPC M4) — tracked
  by epic `sq-1s2`, not by this certification epic.

## 4. How to read the rest of this slice

- [`controls.md`](./controls.md) — one row per review control, each marked **PASS /
  AUDIT-READY / NOT-SOUND (research-only) / EXTERNAL-REQUIRED / OPEN-gap**, with evidence.
- [`evidence.md`](./evidence.md) — the file/line/test verification behind each control,
  re-runnable.
- [`gap-register.md`](./gap-register.md) — the headline `CR-G1` external requirement plus the
  residual privacy/test gaps, each with a bead.
