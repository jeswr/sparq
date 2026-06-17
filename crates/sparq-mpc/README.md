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

## Hidden joins — per-pair leakage tiers (sq-jnkm / sq-khf9 / sq-xhaw)

The hidden-value join over PRIVATE keys (`join.rs`) ships in three leakage tiers,
all honest-majority / **semi-honest only** (not malicious; external sign-off still
pending, sq-qhy4):

- **`HiddenValueJoin::join` / `secure_equal`** — the original per-pair equality:
  one masked-product open per candidate (`m = (a−b)·r`, `m == 0 ⇔ equal`). It
  leaks only the match BIT per pair, but the set of true pairs IS the bipartite
  match graph / key fan-out (**leak L2 at the decision**). Output cardinality is
  the true count (**leak L1**).
- **`HiddenValueJoin::batched_join` (sq-khf9)** — ranges over a row COLUMN per
  holder under a `RowBinding`, and routes the OUTPUT through the oblivious
  shuffle + padded-prefix reveal (sq-jnkm): output cardinality is bounded to a
  public `B` and ordering is shuffled (**L1 bounded, L2 of the result set
  destroyed**). It still OPENS the per-pair match bit, so the decision-time L2 is
  unchanged.
- **`HiddenValueJoin::fully_oblivious_batched_join` (sq-xhaw)** — the
  **fully-oblivious** path: the per-pair match bit is computed as a SECRET-SHARED
  0/1 by `compare::secure_equal_to_bit` (a bit-decomposition + AND-tree, never
  opened) and fed as a `MatchBit::SecretShared` selector into the same oblivious
  output transform. **Nothing is opened per pair** — the decision-time L2
  match-graph leak is closed. The cost is `O(COMPARE_BITS)` secure-multiplication
  rounds per pair (vs one masked open), the LAN profile `compare` documents.

The standalone raw-key entry point `oblivious_join::oblivious_set_output_hidden_keys`
also realises this now (it was previously gated on the secure-compare round
sq-rrz4/sq-dvuc, which has since landed). No soundness/security claim beyond the
documented semi-honest model is made.

## Threshold verdict — in-MPC bit-decomposition of the secret-shared sum (sq-g7t5 / sq-nx0s / sq-mnv5)

`disclose_threshold_verdict` (`compare.rs`) answers `sum > threshold` while
disclosing **only the boolean verdict bit**, never the integer total. It does
this by bit-decomposing the *existing* secret-shared sum **in-MPC** — the local
`reconstruct(sum_shares)` shortcut is **gone** (`sq-g7t5`). The decomposition is
the masked-open protocol (Damgård et al. TCC'06): a random mask `[r]` is added to
`[sum]`, only the statistically-hiding `c = sum + r` is opened (gap
`κ = DECOMP_STAT_SECURITY_BITS = 40`), and the sum's bits are recovered by a
secret-shared borrow-subtraction `c ⊖ [r]`. The sum bound is an **in-protocol
range proof** (`sq-nx0s`), so an out-of-range sum aborts fail-closed rather than
returning a silent wrong verdict.

**[OPUS-4.8] `sq-mnv5` — deployment-grade random-bit sub-protocol (this slice).**
The mask `[r]` and its bits previously came from `deal_random_solved_bits`, where
the in-process dealer drew each bit in **cleartext** and shared it — legitimate in
the single-process simulation but **not deployable**, because in a real deployment
no party may know `r` (a party that knew `r` would learn `sum = c − r` from the
opened `c`). That seam is now closed: `secure_bit_decompose` draws its solved-bits
from `deal_random_solved_bits_via_square_protocol`, the **square-protocol**
random-bit generator (`square_protocol_random_bit`). Each bit `[b]` is jointly
generated from a random `[a]` by opening **only** `c = a²` — a quadratic residue
that is independent of the bit, since `a` and `−a` give the same `c` — then setting
`[b] = (a·d⁻¹ + 1)·2⁻¹` for the public root `d = c^{(p+1)/4}` (valid because
`p = 2^61−1 ≡ 3 (mod 4)`). **No party ever knows the mask or the bit.** Cost: one
secure multiplication + one open per bit.

This is **honest-majority, semi-honest only** — like every other operator in this
crate, it is **not** maliciously secure (the `a²` open and the `degree_reduce` <!-- privacy-claims-allow: NEGATIVE usage — explicitly denies malicious security (semi-honest only); sq-qhy4 -->
re-sharings are unauthenticated; `sq-qhy4` external sign-off is still **pending**),
and the magnitude bound is unchanged (`DECOMP_VALUE_BITS = 20`, covers the
four-flatmates `10^6`). The two residual `sq-mnv5` deployment items remain open
follow-ups:

- **Wider magnitude** — `p = 2^61−1` forces the 20-bit bound; lift via a larger
  field or a non-masked-open comparison (Rabbit, eprint 2021/119). Still open.
- **Malicious security** — carry IT-MACs (`sq-km34.*`) through the decomposition +
  comparison chain and MAC-check the verdict before open. **Partly closed by
  `sq-ka8m`** (`auth_compare`): the malicious-with-abort *comparison chain* over
  secret operands now carries an IT-MAC through every gate (`MacSession::auth_mul`,
  design §2.4 route (a)) and MAC-checks the verdict before open
  (`MacSession::mac_check`, §2.5), aborting on any tamper at the minimal `n = 2t+1` <!-- privacy-claims-allow: design-doc-cited construction, scoped + caveated below; sq-toze.35 -->
  (soundness from the secret `α`, not RS redundancy) — `malicious_greater_than` /
  `malicious_threshold`. The residual is the end-to-end `disclose_threshold_verdict`
  decomposition opens (`a²`, `c = sum+r`, the zero-test products), which are not yet
  routed through the MAC-check; and external sign-off (`sq-qhy4`) is still pending.

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
- **`sq-19ej` (ADDRESSED) + `sq-it50` (ADDRESSED — closes the residual).** The
  masking CSPRNG seed is the one secret byte buffer `sparq-mpc` itself controls:
  `SecureRng::from_os` pulls the 32-byte ChaCha seed into a `zeroize::Zeroizing`
  buffer and wipes it on scope exit (sq-19ej). sq-19ej had to leave a documented
  residual — the inner `ChaCha20Rng` *key schedule* (recoverable via `get_seed()`)
  was **not** scrubbable in place because `rand_chacha` exposes **no `zeroize`
  feature** and no `Drop`/`&mut`-key accessor (verified at both the pinned `0.9`
  and the latest published `0.10.0`, Feb 2026). **`sq-it50` closes it:** the
  production `SecureRng` now backs onto a minimal, **sparq-owned** ChaCha20
  keystream generator (`crate::chacha::ChaCha20Csprng`, private to the crate) whose
  full 16-word block state — the expanded key, the block counter, and the cached
  64-byte keystream block — is `#[derive(ZeroizeOnDrop)]`, so the entire key
  schedule is zeroized on drop. No sparq-reachable copy of the key survives drop
  and there is no `get_seed()`-style accessor. The owned cipher is
  correctness-pinned to two independent RFC 8439 known-answer vectors (§2.3.2
  block + §2.4.2 keystream), and the scrub is proven by test. The now-unused direct
  `rand` dependency was dropped, and as of the rand-ecosystem 0.10 upgrade
  (`sq-8xug`) the `rand_chacha` dependency is dropped too: `rand_core` 0.10 removed
  `OsRng`, so `SecureRng::from_os` now draws the 32-byte seed straight from
  `getrandom::fill` (the same OS-entropy source `OsRng` wrapped). HYGIENE only — the
  masking RNG is still an OS-seeded ChaCha20 CSPRNG drawn as uniform `F_p`;
  behaviour is identical and all MPC tests pass unchanged in both feature states.
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
