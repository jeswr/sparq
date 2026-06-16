<!-- [OPUS-4.8] sq-cu32 / CR-G4 (epic sq-toze) — FIPS posture statement.
     Honest negative claim: NO FIPS 140-3/CMVP module, NO FIPS claim.
     Authored by Opus 4.8 while Fable 5 unavailable — re-review when Fable returns. -->

# FIPS posture statement (CR-G4)

> **The whole point of this document is the negative claim.** sparq uses **no
> FIPS 140-3 / CMVP-validated cryptographic module** and makes **no FIPS claim
> and no CMVP claim**. An operator with a FIPS deployment constraint must treat
> sparq's bespoke (Tier-B) cryptography as **out of FIPS scope**. This document
> exists so that absence is *documented, not implied*.

**Framework:** Cryptographic review of sparq's crypto estate (`compliance/cryptoreview/`).
**Gap:** `CR-G4` (see [`gap-register.md`](./gap-register.md)) — LOW severity, AUDIT-READY.
**Companion docs:** [`README.md`](./README.md) (the Tier-A / Tier-B assurance split),
[`controls.md`](./controls.md) CR-15 (FIPS posture control row),
[`evidence.md`](./evidence.md) §CR-15.

## 1. The claim, stated plainly

- sparq does **not** incorporate any FIPS 140-2 or FIPS 140-3 validated
  cryptographic module.
- sparq holds **no** CMVP (Cryptographic Module Validation Program) certificate
  and claims **no** CMVP certificate number — neither for the bespoke ZK/MPC
  crypto nor for any other code path.
- sparq makes **no** representation that any algorithm it uses runs *inside* a
  validated cryptographic boundary.

This is an honest negative claim. It is not a deficiency to be remediated into a
positive FIPS claim within this certification epic; it is the accurate posture
for a research-grade RDF/SPARQL data engine whose bespoke cryptography is
deliberately built on ZK-friendly (non-FIPS-approved) primitives.

## 2. Why the bespoke primitives are non-FIPS-approved (by design)

The Tier-B cryptography (`sparq-zk`, `sparq-zk-compose`, `sparq-mpc`) is built on
primitives chosen for **in-circuit / zero-knowledge efficiency**, not for
regulatory approval. None of the following is a FIPS-approved algorithm, and none
is intended to be:

| Primitive | Where | What it is | FIPS status |
|---|---|---|---|
| **BN254** (alt-bn128) pairing-friendly curve | `crates/sparq-zk` (`ark-bn254`), the Noir circuits under `zk/` | The proving-system curve; its scalar field is Noir's `Field`. | **Not FIPS-approved.** Not on any NIST-recommended curve list (not P-256/P-384/P-521, not the FIPS 186-5 set). |
| **Poseidon2** hash | `crates/sparq-zk/src/poseidon2.rs` + constants; per-graph commitments; Schnorr challenge hash | An arithmetization-friendly sponge hash over the BN254 scalar field. | **Not FIPS-approved.** Not in FIPS 180-4 (SHA-2) or FIPS 202 (SHA-3); chosen for low in-circuit constraint cost. |
| **Schnorr over Baby-JubJub** (EdDSA-style) | `crates/sparq-zk/src/sig.rs` (`ark-ed-on-bn254`); issuer signatures over commitments | Schnorr signature on the BN254-embedded twisted-Edwards curve, with a Poseidon2 challenge hash. | **Not FIPS-approved.** FIPS 186-5 covers ECDSA/EdDSA over the NIST/Edwards25519/Edwards448 curves only — Baby-JubJub is not among them; the Poseidon2 challenge hash is itself non-approved. |

These are sound engineering choices for a ZK system (the curve and hash must live
in the proving field for an in-circuit verifier to be cheap), and they are the
correct choice *precisely because* the goal is succinct in-circuit verification —
not FIPS compliance. Swapping them for FIPS-approved primitives would defeat the
ZK design; the honest answer is therefore that this estate is **out of FIPS
scope**, not "FIPS-pending".

<!-- [OPUS-4.8] reconciled with post-remediation re-audit (sq-gbp4); see ZK-verdict cross-ref sweep -->
> **Honesty linkage.** This document does **not** assert that the Tier-B crypto
> is sound or production-safe — that is a *separate* and currently *open* question
> gated on `CR-G1` (external accredited-cryptographer audit; see
> [`gap-register.md`](./gap-register.md)) and on `SECURITY.md`'s published posture
> (§"`sparq-zk` and `sparq-zk-compose` — ZK verifier: remediated, but NOT externally
> audited"): the binding layer landed and the internal re-audit found it "sound as
> landed for the assumed threat model," but no production guarantee may be presented
> until the external sign-off (`sq-qhy4`) completes. "Non-FIPS-approved" and "not
> externally verified for soundness" are two independent statements; both are true. A
> FIPS deployment constraint is one reason the estate is out of scope; the externally
> unverified soundness is another, more load-bearing, reason. [OPUS-4.8]

## 3. SHA-256: a FIPS 180-4 algorithm used *outside* any validated module

sparq computes **SHA-256** release digests (`SHA256SUMS` over every release
artifact — the archives, the CycloneDX SBOM, and the VEX; produced in
`.github/workflows/release.yml`). SHA-256 *is* a FIPS 180-4 approved hash
algorithm. **But:**

- It is used here for **artifact integrity**, in the standard way — not as part
  of a cryptographic module under CMVP validation.
- Computing a FIPS 180-4 *algorithm* (e.g. via a Rust crate) is **not** the same
  as running it inside a **FIPS 140-3 validated module**. CMVP validation is a
  property of a *module* (its boundary, key management, self-tests, and operating
  environment), not of the algorithm in isolation.

Therefore: **no module-validation claim attaches** to sparq's SHA-256 usage. An
operator who requires FIPS-validated *module* execution of SHA-256 must supply
that themselves (a FIPS-validated crypto provider / OpenSSL FIPS module / OS
module in their deployment environment) — sparq does not provide it and does not
claim it.

## 4. Containment — why a default consumer never reaches the non-FIPS crypto

The bespoke (non-FIPS-approved) crypto is **opt-in research-only** and is *not*
in the default dependency graph of any published artifact:

- All three crypto crates declare `publish = false` in their manifests
  (`crates/sparq-zk/Cargo.toml`, `crates/sparq-zk-compose/Cargo.toml`,
  `crates/sparq-mpc/Cargo.toml`) — they are **never** shipped to crates.io / npm /
  PyPI.
- None is a dependency of `sparq-wasm`, so they **never** enter the browser
  bundle.
- `sparq-zk-compose` and `sparq-mpc` are non-default workspace members that
  nothing else depends on; default builds and the wasm artifact are
  byte-identical with or without them.

A downstream consumer of the *published* sparq artifacts therefore cannot reach
the non-FIPS-approved primitives unless they deliberately pull the crate by git
path and opt in. This is a containment fact (cited as evidence in CR-13 of
[`controls.md`](./controls.md)), not a substitute for the negative FIPS claim
itself.

## 5. Guidance for a FIPS-constrained operator

If your deployment is subject to a FIPS 140-2 / 140-3 requirement:

1. **Treat the Tier-B crypto (`sparq-zk`, `sparq-zk-compose`, `sparq-mpc`) as out
   of FIPS scope.** Do not enable, link, or rely on it in a FIPS-constrained
   boundary. Because it is `publish = false` and non-default, the simplest posture
   is to **not opt in** — the default build does not include it.
2. **Provide your own FIPS-validated module** for any standard cryptographic
   operation you require in-environment (e.g. TLS termination via a FIPS-validated
   provider in your gateway / OS). sparq is a data engine and an HTTP query API; it
   does not terminate TLS or perform key management inside a validated boundary,
   and it expects to be fronted by operator-controlled infrastructure (see the
   no-auth architectural boundary B3 in the ASVS slice and `research/threat-model.md`).
3. **Verify release artifacts** with the published Sigstore build-provenance
   attestation (`gh attestation verify`) and the SHA-256 `SHA256SUMS` — these are
   Tier-A integrity controls (CR-11) and are sound *for integrity*, independent of
   any FIPS-module question.

## 6. What this document is NOT

- It is **not** a FIPS claim, a CMVP claim, or a statement of FIPS-readiness.
- It is **not** a soundness assertion about the Tier-B crypto (that is `CR-G1`,
  external, open; and `SECURITY.md` is the published posture).
- It does **not** override `SECURITY.md`. `SECURITY.md` is governance-owned; a
  cross-reference to this FIPS posture from `SECURITY.md` is deferred to its own
  bead (mirroring how the SECURITY.md cross-ref for the ZK posture was handled),
  not made here.

## 7. Disposition

`CR-G4` is recorded as **AUDIT-READY (honest negative claim)** — the posture is
documented and verifiable; there is no positive FIPS certificate to obtain within
agent scope because none is sought. If a future operator requirement makes FIPS
validation in-scope, that would be a new, organizational, externally-validated
effort (procuring/integrating a FIPS-validated module), tracked as new work — not
a control this slice can close.
