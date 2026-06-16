<!-- [OPUS-4.8] -->
# sparq-mpc

Honest-majority Shamir MPC over (federated) SPARQL (research question RQ2). This
crate is the secret-sharing + secure-computation substrate: a prime field
(`field.rs`, `F_p` over the Mersenne prime `p = 2^61 − 1`), Shamir sharing and
reconstruction, authenticated (MAC) shares, the hidden-value join, robust
(Reed–Solomon) reconstruction, and the in-process / networked transports.

The build plan, milestones, and the deferred malicious-security seams live in
[`PLAN.md`](./PLAN.md). This README records the crate's **security posture
notes** that an auditor needs at a glance.

> **No security guarantee.** This is **research-grade and externally
> unaudited** (sq-qhy4). The ZK verifier it composes with is **not** sound
> (`SECURITY.md` / CR-G1). Nothing here is a production security claim.

## Collaborative-proof (coZK) — witness-validation-before-proving (sq-7leq)

The collaborative (multi-prover) zk-proof path is **deferred** — every
`CollaborativeProof::prove` / `verify` returns `NotYetImplemented` naming its
gate (see `proof.rs` and the no-fake-crypto table in `adversarial_tests.rs`). The
coZK re-audit (`research/mpc-cozk-reaudit.md`, bead `sq-9hrn`) against CRYPTO'25
eprint 2025/1026 surfaced a **negative result**: the "honest-majority semi-honest
⇒ malicious-secure for free" folklore holds for collaborative zk-SNARKs **only if <!-- privacy-claims-allow: hedged conditional ("only if … BEFORE …"), a negative-result caveat; sq-toze.35 -->
the extended witness is validated for cross-holder consistency BEFORE the proving
phase opens or commits to any value derived from it** (requirement **R-WV**).
Proving over an inconsistent/maliciously-extended witness can **leak honest
provers' inputs**, even when the verifier rejects the proof.

That obligation is now **ENCODED as a test** (`src/witness_validation_tests.rs`),
not just documented. **Status: OPEN (documented-gap), NOT met** — there is no
built prover to validate a witness against, so the precondition cannot be
satisfied today. The encoding is two-tier:

- A **PASSING** meta-test pins the current honest fail-closed posture: the
  deferred `prove` never proves over any witness (it refuses before touching one),
  so no prove-over-invalid-witness path exists today — vacuously, because no
  prover exists. It is the regression anchor that fires if a future `prove`
  returns `Ok(..)` while R-WV is still unmet.
- The **`#[ignore]`d T1–T4 suite** encodes R-WV concretely against the
  `WitnessValidatingProver` contract the future prover (`sq-f7bu` / `sq-bjl`,
  milestone M-E) MUST satisfy; it is un-ignored and lifted into that prover's
  suite when it lands.

This **makes no soundness claim** for the collaborative path. Per the re-audit,
no production attestation/correctness claim may be made until R-WV is implemented,
T1–T4 + clause C pass un-ignored, the honest-majority malicious-security line
(`sq-km34.*`) lands, AND an **external cryptographer audit covers the MULTI-prover
construction** — `sq-qhy4` audits only the single-prover verifier and does **not**
discharge this.

## Constant-time / side-channel posture

A **source-level** constant-time review of the secret-bearing paths exists at
[`compliance/cryptoreview/side-channel-analysis.md`](../../compliance/cryptoreview/side-channel-analysis.md)
(CR-G5, bead `sq-egx6`). Its finding: present exposure is **LOW by architectural
placement** — every live equality is over PUBLIC / opened values (the masked
product `== 0`, the verdict bit, public threshold bits), and operands are never
reconstructed — **not** by constant-time primitives. It is a code-reading review,
not an instrumented `dudect` / `ctgrind` timing study; arkworks field-op timing
is out of scope (a black box). Three fix recommendations were bead-tracked:

- **`sq-u8a8` (LANDED).** Adopt `subtle` / `zeroize` as defensive primitives
  (`Fp::ct_eq`, `SecretKey::ct_eq`; zeroize-on-drop of the MAC key `α`, the
  secret-share vectors, the issuer/holder key). HYGIENE only — no `==` on a live
  protocol path was replaced.
- **`sq-19ej` (ADDRESSED, with documented residual).** The masking CSPRNG seed —
  the one secret byte buffer `sparq-mpc` itself controls. `SecureRng::from_os` now
  pulls the 32-byte ChaCha seed into a `zeroize::Zeroizing` buffer and wipes it on
  scope exit (previously `from_os_rng()` routed the OS seed straight into
  `ChaCha20Rng` with no sparq-side copy to scrub), and `Drop` for `SecureRng` does
  a best-effort scrub of the most-recently-buffered keystream block. **Irreducible
  residual:** the inner `ChaCha20Rng` *key schedule* (recoverable via `get_seed()`)
  is **not** scrubbable in place — `rand_chacha` exposes **no `zeroize` feature**
  and no `Drop`/`&mut`-key accessor, verified at both the pinned `0.9` and the
  latest published `0.10.0` (Feb 2026). We do not claim it is scrubbed. Exposure is
  **LOW** (session-lived, never persisted, `Debug`-redacted + `!Clone`); the
  upstream fix / roll-our-own-ChaCha follow-up is bead-tracked. RNG behaviour is
  identical — all 270 MPC tests pass unchanged.
- **`sq-7ltf` (this README's subject — ADDRESSED).** The latent non-constant-time
  `Fp::pow` (square-and-multiply branches on the exponent bits). See below.
- **`sq-8jv7` (open).** Schnorr issuance signing uses arkworks scalar ops not
  asserted constant-time; issuance-side only.

### `Fp` exponentiation (sq-7ltf)

The original `Fp::pow` was a textbook square-and-multiply: `if exp & 1 == 1 {
acc = acc.mul(base) }`, a data-dependent branch on the **exponent** bits. It was
safe in practice only because every caller passes a **public** exponent — but
nothing at the type level stopped a future caller from passing a secret. That
latent gap is now closed by splitting the surface in two:

- **`Fp::pow_vartime(exp)`** — the variable-time square-and-multiply, renamed with
  a load-bearing `_vartime` suffix and a **public-exponent contract** in its
  doc-comment. Its only caller is the robust Berlekamp–Welch decoder
  (`robust.rs`), which raises a **public** evaluation point to the **public**
  error-correction parameter `e`. It MUST NOT be called with a secret exponent.
- **`Fp::pow_ct(exp)`** — a **fixed 64-iteration, branchless** square-and-multiply
  that selects the conditional multiply with `subtle::ConditionallySelectable`
  instead of an `if`. It is **constant-time in the base value** by construction
  (no data-dependent branch on `self`), so it is safe to exponentiate a
  secret-bearing field element. (It is not asserted constant-time in the
  *exponent* — the bit extraction reads exponent bits — but the trip count is
  fixed and the loop body does not vary with the base.)

**`Fp::inv` now routes through `pow_ct`** (`a^(P−2)`), so field inversion — used
all over Lagrange interpolation and Gaussian elimination — is constant-time in
the inverted value by construction, rather than safe only by current-caller
convention. The exponent `P − 2` is a public constant, so this changes no timing
that was ever secret today; it closes the gap for any future secret-value
inversion. **No arithmetic result changes** — `pow_ct` and `pow_vartime` are
proven to agree over a spread of bases/exponents (`field.rs` tests
`pow_ct_agrees_with_pow_vartime`, `inv_via_pow_ct_is_multiplicative_inverse`), and
all 270 existing MPC tests (disclosure-minimisation, MAC-soundness, hidden-join,
tamper-detection, comparison) pass UNCHANGED, so protocol behaviour is identical.

This is a **defense-in-depth / latent-gap** fix, NOT a soundness fix.
