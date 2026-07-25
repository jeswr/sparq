<!-- [OPUS-4.8] ZK membership/PoK RE-AUDIT (sound-as-landed) run + consolidated by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns. -->
# ZK membership / PoK re-audit — sound-as-landed

Independent adversarial re-audit of the WIRED-but-NOT-yet-sound ZK
**set-membership + holder Proof-of-Possession** members, run for bead **sq-ru0yx**.
It is the membership/PoK analogue of the binding-layer re-audit
(`research/zk-verifier-reaudit.md`, bead sq-gbp4), and covers the four gates the
binding-layer re-audit did NOT touch:

- **hidden-issuer** attestation — `bind_hidden_issuer_attestations`
  (`verifier.rs:3002`) + `hidden_issuer_d{D}` (`issuer.nr`), bead **sq-z9l**.
- **hidden-holder PoK** — `bind_holder_pok` (`verifier.rs:3158`) + `holder_pok`
  (`holder.nr`), beads **sq-xqfg** (circuit) / **sq-i1dt** (verifier gate).
- **hidden-holder SET** — `bind_holder_set` (`verifier.rs:3345`) + `holder_set_d{D}`
  (`holder.nr`), bead **sq-3c00**.

The external accredited-cryptographer sign-off (**sq-qhy4**) stays PENDING and is the
only thing that lifts the standing NOT-yet-sound posture; this is the INTERNAL step
that precedes it, exactly as sq-gbp4 was for the binding layer. Like sq-gbp4 this is
single-model (Opus 4.8) and NOT a substitute for sq-qhy4.

This run is NOT read-only: it found ONE genuine soundness-binding defect (M-1, the
challenge-reduction no-wrap gap) and FIXED it in `issuer.nr` (the `holder_*` members
carry no challenge reduction, so M-1 does not touch them). All other gates are sound
as landed for the stated threat model. Verdicts cite `file:line` on the worktree
branch.

## What the per-finding labels mean (read before quoting one)

This document previously labelled its per-finding dispositions `CLOSED`. In a document
titled a *re-audit*, `CLOSED` reads as a **resolved security conclusion** — which is
not something an internal single-model pass can earn while the external audit is
outstanding. The labels are therefore:

- **INTERNALLY MITIGATED (sq-qhy4 pending)** — this internal pass found the property
  holds with the cited code evidence, and (where a defect was found) fixed it. It is
  **not** an external sign-off and **nothing in this document should be read as
  implying the external accredited-cryptographer audit (`sq-qhy4`) has occurred — it
  has not.** No item here is closed in the audit sense until sq-qhy4 lands.
- **FIXED** — a defect this pass found and fixed in code, with a regression test.
- **CONFIRMED** — a structural fact verified by inspection (e.g. an absent construct).

The standing NOT-yet-sound posture is unchanged by this document.

## Bottom-line verdict

**The three membership/PoK members are SOUND as landed for the threat model
(a prover that fully controls its own private witness, presenting a manifest to a
relying party that supplies the EXTERNAL trust anchors: the trusted `KeySet` K, the
`HolderRegistry`, the authoritative status snapshot), AFTER the M-1 fix.** Every
load-bearing property holds with code evidence:

1. **Public-input reconstruction is exact** for all three members — the verifier
   feeds OUR nonce + OUR recomputed anchor, never the prover's declared bytes, and
   byte-compares against the proof's `public_inputs` before `bb verify` against the
   verifier-recomputed CANONICAL vk. The layouts match each `main` signature
   bit-for-bit (verified below).
2. **The trust anchor is verifier-derived**, never a prover claim: `key_set_root` /
   `holder_set_root` are recomputed from the RP's OWN K / registry; the
   `holder_pk_digest` for `holder_pok` is recovered from the ISSUER signature under
   the external K (not from the proof).
3. **The membership / PoK relations are real cryptography** — in-circuit
   Schnorr-over-Baby-JubJub (issuer), `hpk = hsk·G` possession (holder), with
   on-curve + identity-key + `< L` guards and a Poseidon2 Merkle fold over the
   committed leaf — not structural placeholders.

The ONE finding (M-1) was a soundness-MARGIN defect in the issuer challenge
reduction's *uniqueness* (not a demonstrated universal forgery); it is now closed in
code with a regression suite. See the per-finding dispositions.

## How the three gates anchor (the seam map)

Each gate closes the "the proof attests SOMETHING, but to WHAT?" seam the same way
the binding layer does (sq-gbp4 audit #1/#2/#3/#4), specialised to membership/PoK:

| Gate | What the proof hides | Verifier-fed public inputs | External anchor |
|---|---|---|---|
| hidden-issuer | WHICH issuer key signed | `[nonce, m, key_set_root]` | K (`KeySet::hidden_issuer_root`) + the issuer-signed `m` recomputed from the scan-referenced commitment+salt+status |
| holder-pok | the holder KEY (`hsk`,`hpk`) | `[nonce, holder_pk_digest]` | the ISSUER-signed `holder_pk_digest` (`commitment_message_with_holder` under K) |
| holder-set | WHICH holder | `[nonce, holder_set_root]` | the registry (`HolderRegistry::hidden_holder_set_root`) |

`nonce` is the verifier's fresh `VerifierNonce` (audit #4 freshness), identical to
the sub-proof loop's field-0 binding. A proof committed under any other challenge
cannot byte-match. The canonical vk is recomputed per `CircuitId`
(`prover.canonical_vk`, audit #2) — the prover's vk is never trusted.

---

## Per-finding dispositions

### 1. (load-bearing) hidden-issuer public-input reconstruction matches `main` — INTERNALLY MITIGATED (sq-qhy4 pending)

`bind_hidden_issuer_attestations` (`verifier.rs:3066-3069`) builds exactly
`[nonce, m, key_set_root]` (three 32-byte BE words), matching
`hidden_issuer_d4/src/main.nr` `main(challenge, m, key_set_root, …private…)` — only
those three are `pub`; the issuer key, `(R,s)`, the reduction `(e,e_k)`, and the
membership `index`/`siblings` are PRIVATE (confirmed against `main`). The message
`m` is OUR recomputation `commitment_message_with_status(C(G), salt, status_ref)`
(`scan_referenced_messages`, `verifier.rs:3034/3509`) over a commitment a VERIFIED
scan references — never the prover's declared `hi.message`. A PoK over an
unreferenced commitment → `HiddenIssuerUnreferencedCommitment`; a non-authoritative
root → `HiddenIssuerRootMismatch`; a message not bound to a referenced commitment →
`HiddenIssuerMessageMismatch`. Then `bb verify` against the canonical
`hidden_issuer_d{depth}` vk. Tested end-to-end (toolchain): `e2e.rs:4703`
(`hidden_issuer_in_set_verifies_and_key_is_private`), `:4860` (out-of-set
unprovable), `:4900` (forged signature unprovable), `:4939` (forged root rejected).

### 2. (load-bearing) hidden-issuer key-set root is verifier-derived — INTERNALLY MITIGATED (sq-qhy4 pending)

`KeySet::hidden_issuer_root` (`verifier.rs`) → `issuer::key_set_root_sparse`
(sq-r6dq, since PR #3651; previously the dense `issuer::key_set_root`) folds the
RP's OWN trusted keys in canonical `BTreeSet` order, leaf = `key_set_leaf(pk) =
Poseidon2([pk.x, pk.y])` (`sig.rs:1117`), internal = `h2 = Poseidon2([l, r])`
(`issuer.rs:59`) — bit-identical to `issuer.nr::key_leaf` / `h2` (cross-vector
`issuer.rs:359`, `tests.nr` `h2(1,2)`). The prover commits the SAME order
(`ordered_keys`), so the roots agree. A prover that proves membership in its OWN
forged set fails the `key_set_root` byte-compare.

**What the sparse substitution is, stated precisely (do not shorten this to
"scaling-only").** The sparse root is **value-equivalent to the dense root wherever
the dense evaluation completes** — that is the property the tests pin, and it is the
property the trust anchor needs: the anchor a prover's public root must byte-equal is
unchanged at every depth the dense builder can still be run. It is **not** a
behaviour-preserving change. Removing the dense builder's implicit `O(2^depth)` host
cost means a **deep policy depth that previously aborted or exhausted memory before
producing an anchor now produces one and the verifier reaches LATER states** — root
derivation, key parsing, leaf hashing, the 96-byte public-input byte-compare, and the
`canonical_vk` lookup. Those later states are themselves fail-closed (an
uncompiled `hidden_issuer_d{depth}` member yields a `Driver` error from
`canonical_vk`; there is no fallback to `d4` and no attacker-selected vk), but the
reachable state space genuinely grew, and with it the work an unauthenticated
submission can force before the verifier discovers the member is unavailable. That
residual DoS surface is the RP's own `with_hidden_issuer_depth` configuration and is
tracked separately, NOT closed here.

**How far the equivalence is actually evidenced (do not read this as a
cross-check at every depth).** Two DIFFERENT kinds of evidence exist and they must
not be conflated:

- **Dense cross-check (independent oracle), depths ≤ 12 only.** `issuer::tests`
  `sparse_root_matches_dense_for_all_sizes` (every size, depths 0–6) and
  `sparse_witness_matches_dense_for_all_indices` (depths 0–5) compare against the
  independently-computed dense builder; `hidden_issuer_root_uses_sparse_builder_and_scales`
  (`verifier.rs`) does the same for the real `KeySet` anchor at depths 4/8/12. Only
  these are cross-checks.
- **Self-consistency ONLY, deep trees (24, 28, 31).** `sparse_deep_tree_witness_refolds_to_root`
  (depth 28), the depth-24 group in `hidden_issuer_root_uses_sparse_builder_and_scales`,
  and `sparse_valid_depth_31_root_and_path_are_defined` check that a sparse witness
  re-folds to the sparse root. Root and witness both come from `sparse_fold_leaves`,
  so a **correlated common-mode error in that shared fold would pass both**. There is
  no independent oracle at these depths (the dense builder cannot materialise
  `2^24`+ leaves), so "a wrong anchor or a wrong sibling fails it" holds for an
  ISOLATED error, not for a common-mode one. The equivalence claim at deep depths
  rests on the induction argument (at level *k* the sparse prefix is exactly the
  dense nodes intersecting populated subtrees and every omitted node is the
  precomputed all-padding digest for that level) plus the ≤ 12 cross-checks — not on
  a deep-tree oracle.

The one genuinely independent end-to-end oracle is
`e2e.rs::prove_hidden_issuer`, which builds the prover's root/path with the **dense**
builders and attaches the **sparse** `KeySet::hidden_issuer_root` as the public input
the verifier byte-compares, so `hidden_issuer_in_set_verifies_and_key_is_private`
goes red on a single-bit divergence — through a real `bb` proof. Do not "simplify"
that test to a single builder; the cross-builder asymmetry is what makes it an oracle.

Padding leaf `Fr::from(0)`. **Correction to a previously asserted invariant:** this
document (and `issuer.rs`'s module doc) claimed "the membership fold binds the
`index` bits, so a padding slot is never a usable member". That claim is **false as
stated** — index binding identifies WHICH slot, it cannot distinguish two **equal**
leaf values, so a key whose `key_set_leaf(pk)` happened to equal the padding leaf
could authenticate at any padding slot, and `is_prime_order()` does not prove the
digest nonzero. Exploiting it requires a Poseidon2 zero-preimage (infeasible; no such
key is known and none can be exhibited today), but the invariant was asserted as
absolute while being unenforced by code. Tracked as a pre-existing soundness-claim
defect in **sparq-org/sparq#3782** (host guard `key_set_leaf(pk) != padding_leaf()`
in both `key_leaves` and `sparse_key_leaves`, plus the in-circuit
`key_leaf != padding` assertion, which belongs to sq-qhy4 scope). What IS enforced
today: `key_set_leaf` fail-closes on any non-prime-order (identity / torsion /
off-curve) key, so such a key cannot enter `K` at all
(`issuer::tests::unusable_key_is_rejected_by_both_builders`).

### 3. (CRITICAL→MARGIN, NEW finding) hidden-issuer challenge-reduction binding was not UNIQUE — FIXED (M-1)

`challenge_scalar` (`issuer.nr`) binds the witnessed scalar `e` (and quotient `e_k`)
to the Poseidon2 digest `e_base` via `e + e_k·L == e_base`, with `e < L`
(`assert_lt_l`) and `e_k < 8` (3 bits). The module docs CLAIMED this "uniquely pins
e to e_base mod L". **It does not.** `e + e_k·L == e_base` is a FIELD identity (mod
`q_base`), and because `7·L < q_base < 8·L`:

> for `e_k == 7` the identity ALSO admits `e' = e_base + q_base − 7·L`, which has
> `e' < L` yet `e' ≠ e_base mod L` — `e' + 7·L` overflows past `q_base` and lands
> back on `e_base`.

The window `e ∈ [q_base − 7·L, L)` (~2^126 wide) is unreachable by any honest
reduction (the honest `e_k==7` path has `e = e_base − 7·L ∈ [0, q_base − 7·L)`), but
nothing FORBADE it. Verified numerically: for the ~2^-128 fraction of messages with
`e_base < 8·L − q_base ≈ 2^126`, a divergent `(e', 7)` is accepted by the unfixed
constraint; 200k random samples + small-`e_base` brute force confirm exactly one
such alternative per affected `e_base`.

**Severity.** Not a demonstrated universal forgery: the attacker still cannot
freely choose `e` (it is pinned by `e_base = Poseidon2(R, pk, m)`, and getting two
valid `e` does not let it solve `s·G = R + e·pk` without `dlog(pk)`), and the
affected message subset is negligible. BUT the binding is *looser than the docs
claim*, weakening the reduction's guarantee in an adversarial/algebraic analysis —
exactly the "the comment proves less than it claims" class sq-gbp4 prioritised, in
the soundness-critical reduction the `issuer.nr` header itself flags. For a "make
sound" bead it is fixed, not merely documented.

**Fix.** `reduction_range_bind` (`issuer.nr`) adds a NO-WRAP per-bucket upper bound:
`bound = (e_k == 7) ? (q_base − 7·L) : L`, asserted via the canonical
witnessed-difference range bind `assert_lt_to(e, bound)`. For `e_k < 7` the sum
`e + e_k·L ≤ 7·L − 1 < q_base` cannot wrap (so `bound = L`, unchanged); for
`e_k == 7` the honest `e` always lands `< q_base − 7·L`, so no honest witness is
rejected. Verified: 0 honest rejections + 0 wrap-alternatives accepted across the
same 200k+brute-force sweep. Cost: **+14 ACIR/UltraHonk gates** (16932 → 16946,
`bb gates`, +0.08% — well inside the 3% regression tolerance; snapshot + bench JSON
updated). Honest e2e prove/verify still passes (`hidden_issuer_in_set_verifies_…`).
Regression: `tests.nr` `reduction_no_wrap_*` (accepts honest top-bucket boundary +
low-quotient max; rejects the two wrap-window forge values). The `s < L` scalar
needs no reduction binding (a signer output, range-checked only) — unaffected.

### 4. (load-bearing) hidden-issuer on-curve / identity / `< L` guards — INTERNALLY MITIGATED (sq-qhy4 pending)

`schnorr_verify` (`issuer.nr`) asserts `pk` and `R` on-curve (twist/small-subgroup
forgery guard), rejects the neutral `(0,1)` issuer key (the identity-key universal
forgery: `e·pk = neutral`, so `s·G == R` is satisfiable for any `R = s·G`), and
range-binds `s < L`. Tested: `tests.nr` `hidden_issuer_identity_key_rejected`,
`hidden_issuer_off_curve_key_rejected`, `hidden_issuer_tampered_s_rejected`,
`hidden_issuer_forged_challenge_rejected`, `hidden_issuer_wrong_message_rejected`.
The Grumpkin-vs-Baby-JubJub note (`issuer.nr` header) is correct: the std
`embedded_curve_*` black boxes are Grumpkin, NOT Baby-JubJub, so the explicit
`Field`-constraint twisted-Edwards arithmetic here is REQUIRED for soundness (a
silent wrong-curve verify would otherwise pass). Confirmed `scalar_mul` /
`point_add` use the Baby-JubJub `BJJ_*` constants only.

### 5. (load-bearing) holder-pok digest anchor is issuer-signed, not prover-trusted — INTERNALLY MITIGATED (sq-qhy4 pending)

`bind_holder_pok` (`verifier.rs:3158`) reconstructs `[nonce, holder_pk_digest]`
(`verifier.rs:3248-3249`), matching `holder_pok/src/main.nr` `main(challenge,
holder_pk_digest, hsk, hpk_x, hpk_y)` — only the first two are `pub`; `hsk`/`hpk`
PRIVATE (confirmed). Crucially the digest is NOT read from the proof: it is recovered
from the covering attestation's issuer signature via
`verify_holder_attestation_signature` (`verifier.rs:4298`), which (a) requires the
issuer key in the EXTERNAL K (`IssuerKeyNotInKeySet`), (b) recomputes
`commitment_message_with_holder(C, salt, status_ref, holder_pk_digest)` itself
(`sig.rs:981`, ZKSIG_C4 domain tag), and (c) verifies the Schnorr signature over it.
So the proven hidden holder key is bound to the issuer-attested credential: a holder
A lacking `hsk_B` cannot satisfy B's digest (DL-hardness + proof soundness), and
cannot swap in its own digest without breaking the issuer's signature. Fail-closed:
unreferenced PoK → `HolderPokUnreferencedCommitment`; bearer credential (no holder
binding) → `HolderPokBindingMissing`; digest mismatch → `HolderPokDigestMismatch`.
Tested: `holder_pok_binding.rs` (unreferenced/malformed default-lane;
`#[ignore]` toolchain wrong-holder/tampered/valid).

### 6. (load-bearing) holder-pok mandatory-possession sweep — INTERNALLY MITIGATED (sq-qhy4 pending)

Under `HolderBindingPolicy::require_in_circuit_pok` AND a `HolderPop` binding,
`bind_holder_pok` (`verifier.rs:3291-3301`) requires EVERY holder-bound
scan-referenced credential to carry a verified PoK — a holder-bound covering
attestation with no matching PoK → `HolderPokMissing`, fail-closed (the hidden-key
possession proof is never silently waived). A plain `Challenge` binding presents no
holder, so the sweep is correctly scoped out (mirrors the B1 `bind_holder_pop`
`Challenge` early-return). Tested: `holder_pok_binding.rs`
`holder_pok_required_but_absent_rejected` (`#[ignore]`).

### 7. (load-bearing) holder-set root anchor + Merkle membership — INTERNALLY MITIGATED (sq-qhy4 pending)

`bind_holder_set` (`verifier.rs:3345`) reconstructs `[nonce, holder_set_root]`
(`verifier.rs:3414-3415`), matching `holder_set_d4/src/main.nr` `main(challenge,
holder_set_root, hsk, hpk_x, hpk_y, index, siblings)` — only the first two `pub`
(confirmed); `holder_pk_digest` is NOT public here (the hidden-holder upgrade over
`holder_pok`). `holder_set_root` is recomputed from the RP's OWN registry
(`HolderRegistry::hidden_holder_set_root` → `holder::holder_set_root`), leaf =
`holder_set_leaf(hpk) = holder_key_digest(hpk) = Poseidon2([ZKSIG_HK, hpk.x, hpk.y])`
(`holder.nr`, bit-identical to `sig.rs:935`), NOT the issuer `h2(x,y)` shape
(domain-separated; cross-tested `holder.rs:438`, `sig.rs:1862`). A prover proving
membership in its OWN forged set fails the root byte-compare
(`HolderSetRootMismatch`). The in-circuit relation also re-proves `hpk = hsk·G` +
on-curve + non-identity + `hsk < L` (reuses `issuer.nr`'s gadgets verbatim), so a
set member must still possess the holder secret. Fail-closed: not opted in →
`HolderSetNotEnabled`; depth mismatch → `HolderSetDepthMismatch`; unreferenced →
`HolderSetUnreferencedCommitment`. Tested: `holder_set_binding.rs` (not-enabled /
depth / unreferenced / malformed / `registry_root_matches_host_root` default-lane;
`#[ignore]` in-set/forged-root/out-of-set toolchain).

### 8. (load-bearing) holder members carry NO challenge reduction — CONFIRMED (M-1 does not apply)

`holder_pok` and `hidden_holder_set` (`holder.nr`) prove `hpk = hsk·G` with `hsk`
range-bound `< L` directly — there is NO base→scalar field reduction (no `e_base`,
no `(e, e_k)` quotient), because the holder relation has no Fiat-Shamir challenge
scalar. So the M-1 wrap defect is structurally absent from both holder members
(grep-confirmed: `REDUCTION_*` and `e_k` appear only in `issuer.nr`). No fix needed.

### 9. (scope, documented) freshness, replay, vk, depth — INTERNALLY MITIGATED, inherited (sq-qhy4 pending)

All three gates take the verifier nonce as public-input field 0 (audit #4), recompute
the canonical vk per `CircuitId` (audit #2), and require the declared `depth` to
equal the policy depth (`HiddenIssuerDepthMismatch` / `HolderSetDepthMismatch`),
which selects the member whose vk is recomputed — so a depth relabel fails bb verify
against the wrong-depth canonical vk, exactly as the binding-layer audit #11 closes
the n/d/r relabel. No new freshness/replay/vk hole specific to membership/PoK.

---

## NOT-yet-closed (out of scope — sq-qhy4 / privacy residuals)

These are NOT soundness breaks (no forge-and-verify); they are the standing
boundary this re-audit does not move:

- **External sign-off (sq-qhy4)** — the only thing that lifts NOT-yet-sound. This
  re-audit is single-model internal; it does not replace it.
- **Privacy is wired, not proven.** The members are designed to hide the issuer key
  / holder key / which-holder, and the public-input layout discloses only the
  anchors. But the ZK-privacy PROPERTY (witness indistinguishability / the proofs
  leak nothing beyond the relation) rests on Barretenberg's UltraHonk zero-knowledge
  and the circuit having no witness-dependent public output — asserted by the
  layout, NOT independently proven here. No privacy claim is made.
- **Toolchain-gated coverage.** The crypto-chain e2e cases (`#[ignore]`, nargo/bb)
  and the gate-count snapshot run only in the toolchain CI lane; the default lane
  covers the Rust fail-closed paths + the Noir unit/forge tests (`nargo test`).

## Regression map (one forge → reject per finding)

| # | Finding | Forge | Reject path / error | Lane |
|---|---|---|---|---|
| 1 | hidden-issuer PI reconstruction | proof over a different referenced commitment / message | `HiddenIssuerMessageMismatch` / `HiddenIssuerProofRejected` | toolchain |
| 2 | hidden-issuer root anchor | membership in a prover-forged key set | `HiddenIssuerRootMismatch` | default + toolchain (`hidden_issuer_forged_root_rejected`) |
| 3 | **M-1 reduction no-wrap** | `e` in `[q_base−7L, L)` with `e_k==7` | `assert_max_bit_size` (no-wrap bound) | default (`reduction_no_wrap_rejects_*`) |
| 4 | hidden-issuer guards | identity / off-curve key, tampered `s`, forged `e` | `identity issuer key rejected` / `point is not on Baby-JubJub` / equation / reduction | default (`nargo test`) |
| 5 | holder-pok digest anchor | PoK over a digest the issuer did not sign | `HolderPokDigestMismatch` / `InvalidIssuerSignature` | toolchain (`holder_pok_wrong_holder_…`) |
| 6 | holder-pok mandatory sweep | holder-bound credential with no PoK under require | `HolderPokMissing` | toolchain (`holder_pok_required_but_absent_rejected`) |
| 7 | holder-set root anchor | membership in a prover-forged holder set | `HolderSetRootMismatch` | default + toolchain (`hidden_holder_set_forged_set_root_…`) |
| 8 | holder no-reduction | (n/a — structurally absent) | — | grep-confirmed |

## Methodology

Read the four gates on the worktree branch (`verifier.rs`
`bind_hidden_issuer_attestations` / `bind_holder_pok` / `bind_holder_set` /
`verify_holder_attestation_signature` + the `verify_manifest` orchestration), the
host root/leaf mirrors (`issuer.rs`, `holder.rs`, `sig.rs::key_set_leaf` /
`holder_key_digest` / `commitment_message_with_holder` / `in_circuit_witness`), and
the three circuits (`hidden_issuer_d4`, `holder_pok`, `holder_set_d4` + the
`compose_core` relations `issuer.nr` / `holder.nr`). The public-input layouts were
matched line-by-line against each `main` `pub` signature (no omitted `pub`, no
included private param, no `-> pub` return). The reduction binding was analysed
arithmetically over the BN254 scalar field `q_base` and the Baby-JubJub order `L`
(`q_base // L == 7`, `7L < q_base < 8L`); the M-1 wrap alternative + the fix were
verified by exhaustive small-case + 200k random-sample checks AND by in-circuit
`nargo test` (the fix compiles, the honest e2e prove/verify passes, the forge
witnesses reject). Verdicts prioritise empirical honesty over reassurance: the report
states plainly that the members are sound as landed AFTER M-1, that M-1 was a real
(if margin-level) defect now fixed, and that the external sign-off + ZK-privacy
property remain open.
