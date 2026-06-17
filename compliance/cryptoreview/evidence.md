<!-- [OPUS-4.8] sq-toze / sq-toze.20 — Cryptographic-review evidence pack.
     Per-claim file/line/test verification. Authored by Opus 4.8 while Fable 5 unavailable. -->

# Cryptographic review — evidence pack

Per-control verification for [`controls.md`](./controls.md). Every claim cites a
repo-relative path (and, where it pins a fact, a line range or test name). Re-verifiable on
`main` at the time of writing; line numbers may drift — the symbol names are the durable
anchor. NON-CANONICAL timing applies (EC2 work box); no timing is baked here.

---

## CR-1 — Crypto-estate inventory

The three bespoke crypto crates and their declared role / non-shipping status:

- `crates/sparq-zk/Cargo.toml` — `publish = false`; deps: `rdf-canon 0.15.3` (RDFC10),
  `ark-bn254`/`ark-ff`/`ark-ec`/`ark-serialize`/`ark-ed-on-bn254`/`ark-std 0.6` (BN254 field
  + Baby-JubJub), `blake3` (off-circuit string hash), `getrandom` (salt CSPRNG). Manifest
  comment: "NOTHING in the workspace depends on this crate."
- `crates/sparq-zk-compose/Cargo.toml` — `publish = false`; deps `sparq-zk`, `libc` (native
  flock for the durable nonce store). Comment: "byte-identical wasm artifact with or without
  it."
- `crates/sparq-mpc/Cargo.toml` — `publish = false`; deps `sparq-core`/`sparq-engine`,
  `getrandom 0.4` for the OS-entropy seed of the sparq-owned ChaCha20 CSPRNG
  (`src/chacha.rs`; `rand_chacha` dropped at the rand-ecosystem 0.10 upgrade, sq-8xug).
  Header: "Native-only research scaffold; crypto deferred"; `insecure-test-rng`
  feature OFF by default.

Standard primitives inventory: see CR-11.

---

## CR-2 — Original soundness audit preserved

<!-- [OPUS-4.8] reconciled with post-remediation re-audit (sq-gbp4); see ZK-verdict cross-ref sweep -->
`research/zk-soundness-audit.md` — the 2026-06-13 adversarial audit (59-agent fan-out).
Assurance statement (verbatim, line 8): *"v1 verifier soundness is BROKEN. verify_manifest
provides NO meaningful soundness guarantee to a relying party."* 12 confirmed issues, 5
CRITICAL. This **original** verdict is preserved as history in the `SECURITY.md` §"`sparq-zk`
and `sparq-zk-compose` — ZK verifier: remediated, but NOT externally audited" caveat block (its
"**Status — what changed**" paragraph records the predated-BROKEN finding, then the landed
`sq-1s2` remediation, then the re-audit verdict). **The original verdict is on the record and
the regression map for its now-closed findings is bead `sq-1gir`.** [OPUS-4.8]

---

## CR-3 — Post-remediation re-audit, honestly calibrated

`research/zk-verifier-reaudit.md` (bead `sq-gbp4`, closed). Re-runs the 12-finding pass
against the verifier **after** all 17 `sq-1s2` remediation commits. Bottom-line verdict
(line 19): *"The v1 verifier is SOUND as landed for the threat model the prior audit
assumed."* Every prior finding marked **CLOSED with code evidence** (per-finding dispositions,
lines 81-291).

**The three honesty caveats this framework attaches to that verdict (and why CR-3 is
AUDIT-READY, not PASS, and CR-4 stays NOT-SOUND):**

1. **Single-model internal self-review.** The re-audit header self-flags: *"run + consolidated
   by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns."* It is the same project /
   model family that authored the remediation. Self-review is **necessary but not sufficient**
   for a production cryptographic-soundness claim. An external accredited cryptographer has
   **not** reviewed it (CR-G1).
2. **Read-only, no independent re-derivation of the empirical anchors.** The re-audit verifies
   the public-input reconstruction two ways — line-by-line match against each circuit's `main`
   signature, and the non-circular byte anchor — but it is a *reading* of the code, not a
   machine-checked proof of the `reconstruct_public_inputs` ↔ circuit-`main` correspondence
   (re-audit Methodology, lines 402-420).
3. **The crypto-chain forge tests are `#[ignore]`d** (re-audit lines 40-44): the
   `forge_pubinput_*` real-bb cases do not run in default CI, so closure of findings #1–#12
   rests on code-reading + the two empirical anchors, not on default-CI execution. (CR-G2.)

**Net:** CR-3 is genuine, material evidence that remediation happened and was internally
adversarially checked — it would be dishonest to ignore it. It is **not** an external
soundness certificate — it would be dishonest to present it as one. The published
`SECURITY.md` posture remains the relying-party-facing truth until CR-G1 closes.

---

## CR-4 / CR-5 — ZK + MPC overall dispositions (NOT-SOUND)

- **ZK (CR-4):** the 1:1 finding→forge→reject-path map is at `research/zk-verifier-reaudit.md`
  lines 385-398 (e.g. #1 → `PublicInputMismatch`; #2 → bb verify vs canonical vk; #3 →
  `UnattestedCommitment`/`IssuerKeyNotInKeySet`; #4 → `NonceReplay`/`NonceBindingMismatch`).
  These are *internal* closure dispositions, not external certification. The `SECURITY.md`
  §"`sparq-zk` and `sparq-zk-compose` — ZK verifier: remediated, but NOT externally audited"
  caveat block governs the relying-party stance: do **not** present a "verified" result as a
  production-grade guarantee until the external cryptographer audit (`sq-qhy4`) completes. [OPUS-4.8]
- **MPC (CR-5):** `crates/sparq-mpc/Cargo.toml` header documents M0–M3 (honest-majority
  semi-honest Shamir t-of-n + secret-shared equality join) as built and M4/Q1 (the
  distributed-signature-over-secret-shared-witness collaborative proof) as **DEFERRED behind
  `NotYetImplemented` stubs**, gated on the ZK foundation. `research/mpc-m4-distributed-sig-feasibility.md`
  + `research/mpc-malicious-security-design.md` are the design-of-record. `SECURITY.md` lines
  94-100: *"no confidentiality, correctness, attestation, or malicious-security guarantee
  today."* `crates/sparq-mpc/src/adversarial_tests.rs` exists (negative-model tests), but the
  malicious-security guarantee is **not** claimed.

---

## CR-6 / CR-7 — Forge suite + differential fuzzer

- `crates/sparq-zk-compose/tests/forge_gates.rs` (bead `sq-ajl`, closed). Module doc
  (lines 1-28): "construct a FORGED proof/manifest that violates EXACTLY one binding, then
  assert the verifier REJECTS it with the right `CheckError`." Structural forges
  (commitment/attestation/attribution/revocation/nonce-binding) run in default CI (reject
  before any bb subprocess); the crypto-dependent forges (`forge_pubinput_*`, single-use
  replay after a genuine accept) are `#[ignore]`d + toolchain-gated.
- Companion forge files: `join_forge.rs`, `holder_pop_forge.rs`, `audit_forge_map.rs`,
  `verifier_errors.rs`, `join_gates.rs`.
- `crates/sparq-zk-compose/tests/differential_fuzz.rs` (bead `sq-61g`, closed) — prove →
  verify → compare-to-cleartext differential fuzzer (CR-7).

---

## CR-8 / CR-9 — Gate-count ratchet + pinned toolchain lane

- `crates/sparq-zk-compose/tests/gate_count.rs` + `tests/gate_count_snapshot.json` (bead
  `sq-c5f`). Reads real `bb gates -s ultra_honk` `circuit_size` per `zk/compose` member;
  fails if it exceeds `baseline * (1 + 3%/100)`. Not `#[ignore]`d; skips cleanly when the
  toolchain is absent.
- `.github/workflows/zk-toolchain.yml` — `zk forge + anchor suite (nargo/bb)` job. Toolchain
  pinned: `NARGO_VERSION: "1.0.0-beta.21"`, `BB_VERSION` pinned (workflow env block,
  lines ~90-120), with an explicit comment rejecting "latest"/`next` floating installs. The
  check-run **always appears** on the merge_group (so a required-check gate can reference it),
  and the real-bb steps run `if: steps.changes.outputs.zk == 'true'`.

---

## CR-10 — Non-circular public-input anchor

`crates/sparq-zk-compose/src/verifier.rs`:
- `reconstruct_public_inputs` (the prior audit's CRITICAL #1) rebuilds the bb `public_inputs`
  byte vector from the DECLARED `ProofInputs` in each member's `main` declaration order, using
  the **verifier's** nonce as field 0, and `verify_manifest` byte-compares it against the
  proof (`PublicInputMismatch`).
- Anchors: `reconstruct_filter_int_matches_real_bb_public_inputs` (160 B) and
  `reconstruct_scan_matches_real_bb_public_inputs` (704 B) compare the reconstruction against
  bytes captured by `e2e.rs` `probe_*_public_inputs_hex` from an actual `prover.prove_in(...)`
  — **non-circular** (pinned to bb's real output, not to itself).
- **Gap:** only `filter_int_d1` + `scan_k1_n16_r4` are anchored; `filter_f64_d*` + the k=2
  scan members rely on (correct, generic) layout reasoning with no captured golden vector
  (re-audit NEW-1). ⇒ CR-G3.

---

## CR-11 — Standard primitives used correctly (Tier A)

- **Sigstore build provenance:** `.github/workflows/release.yml` —
  `actions/attest-build-provenance@a2bbfa2…` (v4.1.0, SHA-pinned) over the packaged archives
  (line ~107) and over the SBOM + VEX (line ~144); buildkit `provenance: mode=max` on the
  image (line ~250). Verifiable with `gh attestation verify <file> --repo <repo>`. **Real and
  sound** — sparq consumes Sigstore (Fulcio/Rekor + Cosign) in the standard keyless way.
- **SHA-256 digests:** `release.yml` line ~176 `sha256sum -- * > SHA256SUMS` over every
  archive + SBOM + VEX. FIPS 180-4 hash, used for integrity.
- **RDFC10:** `crates/sparq-zk/src/canon.rs` (module doc: "W3C RDF Dataset Canonicalization
  via the maintained zkp-ld `rdf-canon` crate, W3C-test-suite-validated"); conformance test
  `crates/sparq-zk/tests/rdf_canon_suite.rs` (incl. SHA-384 parameterized cases, test075).
  Fail-closed on the HNDQ poison-graph limit (`CanonError::Canonicalization`).
- **CSPRNG:** `crates/sparq-zk/src/ingest.rs` `SaltMint::mint` draws 32 `getrandom::fill` bytes
  per graph; `crates/sparq-mpc/src/rng.rs` `SecureRng::from_os` seeds the sparq-owned
  ChaCha20 CSPRNG (`src/chacha.rs`) with 32 OS-entropy bytes from `getrandom::fill`
  (`rand_chacha` dropped at the rand-ecosystem 0.10 upgrade, sq-8xug — `rand_core`
  0.10 removed `OsRng`). Insecure deterministic PRNG only behind `insecure-test-rng` (CR-12).

---

## CR-12 — Insecure-RNG gated off

`crates/sparq-mpc/Cargo.toml` `[features] insecure-test-rng = []` — OFF by default; manifest
comment: *"a normal `sparq-mpc` build CANNOT construct a predictable masking RNG."* Bead
`sq-1vt` (resolved). The real protocol always uses the OS-seeded ChaCha20 CSPRNG
(`shamir.rs` + `rng.rs`).

---

## CR-13 — Containment (not shipped, not in default graph)

- `publish = false` in `crates/sparq-zk/Cargo.toml`, `crates/sparq-zk-compose/Cargo.toml`,
  `crates/sparq-mpc/Cargo.toml` (all three verified) ⇒ none is published to
  crates.io/npm/PyPI.
- `sparq-zk` / `sparq-mpc` are **absent** from `crates/sparq-wasm/Cargo.toml` ⇒ the browser
  bundle is free of the bespoke-crypto surface. The crates' own headers assert the
  `cargo tree -p sparq-wasm` exclusion invariant.
- The `sparq-engine` `zk` feature (the zk-trace seam) only exists inside `sparq-zk`'s own
  dependency graph (`sparq-zk/Cargo.toml` comment) — default builds do not enable it.

**Why this matters for honesty:** the unsound Tier-B code is not reachable by a downstream
consumer of the published sparq artifacts. The soundness gap is real, but it is *contained* to
an opt-in research surface — which is the honest framing (a documented research scaffold), not
a hidden production vulnerability.

---

## CR-14 — Constant-time / side-channel posture

- The Schnorr signature **verify** path (`crates/sparq-zk/src/sig.rs` `verify`) runs
  verifier-side over **public** commitments and a public issuer key; it carries no long-term
  secret in the timing-sensitive path, so a verify-side timing leak does not expose a secret
  key. Identity-key rejection + prime-order-subgroup checks are present (re-audit §3).
- The MPC mask draw is documented as effectively constant-time in expectation
  (`crates/sparq-mpc/src/rng.rs:45`: rejection-sampling failure probability `1/2^61`).
- **Honest limitation:** sparq has performed **no** formal constant-time analysis and **no**
  `dudect`/`ctgrind`-style timing measurement. The arkworks BN254 field arithmetic is **not**
  asserted constant-time by sparq, and the *signing* path (issuer secret key) has not been CT-
  audited. ⇒ CR-G5. This is appropriate to flag because if the in-circuit hidden-key /
  in-circuit signature upgrades land (epic `sq-1s2`), the secret-bearing paths grow and CT
  analysis becomes load-bearing.

---

## CR-15 — FIPS posture (honest negative claim)

sparq makes **no** FIPS claim and uses **no** FIPS 140-3 validated cryptographic module.
- The ZK-friendly primitives — BN254 pairing-friendly curve, Poseidon2 hash, Schnorr over
  Baby-JubJub — are **not** FIPS-approved algorithms (they are chosen for in-circuit
  efficiency, not regulatory approval) and are not intended to be.
- SHA-256 (release digests) is a FIPS 180-4 hash, but it is used here for artifact integrity,
  not inside a CMVP-validated module, so no module-validation claim attaches.
- No CMVP certificate number is or will be claimed for the bespoke crypto. ⇒ CR-G4 records the
  absence honestly for any operator with a FIPS deployment constraint.

The full negative claim, the FIPS-status table for each primitive, the SHA-256
used-outside-a-module reasoning, the `publish = false` containment, and the
FIPS-constrained-operator guidance are stated in the standalone posture statement
[`fips-posture.md`](./fips-posture.md) (sq-cu32). Short FIPS notes are also added to
the crypto crate docs: `crates/sparq-zk-compose/README.md` (existing README),
`crates/sparq-zk/src/lib.rs` and `crates/sparq-mpc/src/lib.rs` (crate-level `//!`
docs — these crates have no README). `SECURITY.md` was **not** edited (governance-owned).

---

## CR-16 — Residual privacy gaps tracked

From `research/zk-verifier-reaudit.md` NEW-1/NEW-2, all on the books as beads:
- `sq-hyhj` — in-circuit salt binding + remaining revocation-disclosure privacy residuals.
- `sq-93h` — per-graph salt disclosed in the clear on the hidden-only attestation path
  (cross-presentation linkability assessment).
- `sq-i1dt` — `bind_holder_pok` hidden-key gate / credential-bound HolderPop (today a trusted
  holder A could present trusted holder B's credential — narrows "who may present at all," not
  "whose credential").

These are **privacy/binding** residuals, **not** soundness breaks (a revoked/forged credential
still fails; an untrusted holder still fails). They are recorded as CR-G6, not as a soundness
re-opening.
