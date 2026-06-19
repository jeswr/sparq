<!-- [OPUS-4.8] Design-for-review authored by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns. -->
# Age-proof gate-count reduction: why the FILTER members are ~17k, and how to reach ~4k

Maintainer-review design record. Question answered: *the "age ≥ 18 over a hidden
DOB/credential literal" proof in sparq measures **17,416 gates**, while comparable
range/comparison circuits elsewhere reportedly land around **~4,000 gates** — why
the ~4× gap, what do the ~4k examples do differently, and is there a reduction
that reaches ~4k WITHOUT trading away any soundness-enforcing constraint?*

Parent context: epic **sq-1s2** ("ZK query-proof build-out + in-circuit privacy
upgrades"). Companion analyses: `research/zk-soundness-audit.md`,
`research/zk-verifier-reaudit.md`, `research/zk-hidden-join-design.md`. NO
production circuit change is made by this record (see §4 for the decision and the
impl beads); it is a design-for-review.

## Honesty framing (load-bearing — read first)

The sparq v1 ZK query-proof verifier is **NOT externally audited and is documented
NOT-yet-sound** (beads **sq-qhy4**, **sq-9hrn**, **sq-1s2**). Nothing in this
record is a security guarantee. Where a reduction is described as "safe", that
means it *preserves the soundness-enforcing constraint set* — the exact equalities,
range-binds and canonical-form asserts the current circuit makes — **pending the
external audit**. It is never a claim that soundness or security is proven or
preserved as fact. A reduction is only "safe to propose" if **every** must-keep
constraint in §3.4 demonstrably survives, with the cited constraint intact (not
merely "equivalent"); and even then the framing is "preserves the constraint set,
pending the external audit (sq-qhy4)".

All gate counts here are the **measured `bb gates -s ultra_honk` `circuit_size`** —
a circuit metric the repo already snapshots and regression-gates, not a performance
marketing number. Toolchain: `nargo 1.0.0-beta.21`, `bb 5.0.0-nightly.20260324`
(the snapshot's baselined toolchain). The `circuit_size` figures below were
**re-measured on this box** and reproduce the checked-in snapshot
(`crates/sparq-zk-compose/tests/gate_count_snapshot.json`) byte-for-byte.

## 0. What "the age proof" actually is in this repo

There is no `age.nr`. The "age ≥ 18 over a hidden committed credential literal"
proof is the **hidden-operand numeric FILTER family** — a SPARQL
`FILTER(?value >= bound)` over a *hidden, committed* literal — composed with the
scan/issuer proofs that bind that literal to a signed credential. The relevant
members:

- **Comparison**: `zk/compose/compose_core/src/filter_int.nr`
  (`filter_int_check<D>`, non-negative `xsd:integer`) and
  `zk/compose/compose_core/src/filter_signed.nr` (`filter_signed_int_check<MD>` /
  `filter_decimal_check<ID,FD>`). Deployed bins: `zk/compose/filter_int_d{1..4}`,
  `filter_signed_int_d{2,4}`, `filter_decimal_i3_f2`.
- **DOB-bound-to-credential**: `zk/compose/compose_core/src/scan.nr` exposes
  `operand_enc`; `zk/compose/compose_core/src/issuer.nr` ties the commitment to a
  trusted issuer signature.
- **The binding glue**: `crates/sparq-zk-compose/src/verifier.rs`
  (`reconstruct_public_inputs`, the `scanned != operand` binding edge,
  `canonical_vk`, `record_fresh`).

The "age" cost question is therefore a question about **one FILTER member's
17,416 gates**.

## 1. WHY the age proof is ~17k gates — the attribution table

The dominant cost is **not** the range/comparison. It is the in-circuit hash that
binds the compared number to the committed credential literal. Measured on this box
(`bb gates -s ultra_honk`, `circuit_size`):

| Probe | Measured `circuit_size` | What it isolates |
| --- | ---: | --- |
| empty `main` (one trivial assert) | **35** | UltraHonk/ACIR fixed floor |
| `blake3` over a 48-byte token + 31-byte truncation, only | **17,416** | the string-hash binding **alone** |
| `filter_f64` (`filter_f64_check`, raw f64 compare, **no** binding) | **3,113** | comparison verdict + table floor, no hash |
| `filter_int_d2` (`filter_int_check`, compare + blake3 binding) | **17,416** | the full age-style FILTER |
| `filter_signed_int_d4` (signed, compare + blake3 binding) | **17,416** | signed variant, same binding |

Two facts fall straight out of the measurements:

1. **blake3 over the canonical token IS the circuit.** The blake3-only probe
   (17,416) equals the full FILTER member (17,416). The comparison verdict, the
   digit-canonicality asserts, the value accumulation, and the Poseidon2
   public-input binding all fit *inside* the gate envelope blake3 alone already
   establishes — they add zero marginal `circuit_size`.
2. **The comparison is cheap and already lookup-backed.** The only structural
   difference between the 3,113-gate `filter_f64` and the 17,416-gate
   `filter_f64_d{D}` / `filter_int_d{D}` members is the in-circuit
   `std::hash::blake3(token)` over a ~46–90-byte canonical N-Triples token
   (`filter_int.nr:74-84`, `filter_signed.nr:98-107`,
   `filter_float.nr:127-137`). That isolates **17,416 − 3,113 ≈ 14,300 gates
   (~82%) as the blake3 string-hash binding**, and leaves the comparison verdict at
   ~3,113 — most of *which* is the UltraHonk padding floor + range-check table
   setup, not the comparison ops.

Why blake3 is in-circuit at all (`filter_int.nr:1-15`): `sparq_zk::encode` commits
a literal as `Enc = h2(LITERAL, blake3(canonical_token))` — a **string** hash, so
the committed numeric value has *no field-arithmetic handle*. To compare it, the
circuit re-witnesses the digit bytes, rebuilds the exact N-Triples token, re-hashes
with blake3, and asserts `h2(2, digest) == operand_enc` (`filter_int.nr:92`). That
blake3 re-derivation is the **soundness-critical binding** between the disclosed
`operand_enc` (which the scan proof anchors to a committed triple) and the number
actually compared. The `filter_int.nr` module docstring flags it as "a measured,
deliberate exception to the never-hash-strings-in-circuit house rule".

There is **no ECDSA/EdDSA/Schnorr signature verification in the FILTER circuit** —
credential authenticity is anchored upstream by the scan/issuer proof via the
public `operand_enc`, not re-verified here. The only in-circuit hashing in a FILTER
member is one blake3 compression + one Poseidon2 `h2`.

The figure is **invariant to digit count D (1→4) and to signedness** (all
`filter_*_d{D}` = 17,416 in the snapshot, lines 21-32), the first tell that one
fixed-cost primitive (a single blake3 compression over a ≤64-byte block, which is
length-invariant) dominates — not the per-digit comparison.

## 2. What the ~4k examples do differently — the specific technique

A "compare two values" range circuit that lands at ~4k gates is doing exactly the
part `filter_f64` already does at 3,113 gates: a typed-integer comparison over
Plookup-backed range checks, with **no in-circuit string hash**. The ~4k examples
differ from sparq's FILTER members in **one** structural way:

- **They bind the operand to a commitment via a field-arithmetic-friendly hash of
  the NUMERIC VALUE** (e.g. a Poseidon/Pedersen commitment to the integer), or they
  take the value as a public input and do not bind it to a string-hashed credential
  commitment at all. Either way **they never reconstruct and hash a string token
  in-circuit.** A single Poseidon2 permutation over the value-as-field is ~74 gates
  (`hashes.nr:12-15` cost note); a blake3 compression over the ~46–90-byte string
  token is ~14,300 gates. That one substitution is the whole gap.

What the ~4k examples do **not** do better:

- **Range/comparison.** sparq's comparison is already optimal for this stack. The
  integer FILTER compares with native `u64` operators (`value < bound`,
  `value == bound`; `filter_int.nr:96-97`), and on UltraHonk every typed-`uN`
  comparison and every `assert_max_bit_size::<N>()` is **already Plookup-backed**
  via the `RANGE` black-box (there is no user-callable general lookup-table gadget
  in this Noir/bb version). A `u64 <` is ~13 gates and a `u64 ==` ~11 gates against
  a one-time ~2,800-gate shared range-table setup. Migrating the comparison to
  `Field`-arith would be a **regression** (and reintroduce modular wrap — see
  §3.4-B). The lone comparison anti-pattern to avoid is `Field::lt` (~183
  gates/call, ~14× the `u64 <` cost), used in `scan.nr:121` only because
  commitments are full BN254 field elements that genuinely don't fit `u64`.
- **The commitment hash primitive.** sparq already uses **Poseidon2 exclusively**
  (`hashes.nr:17`), the cheapest in-circuit hash on this stack (~74 gates/
  permutation). Pedersen is scalar-mul-based and would be *more* expensive, not
  less. So "use a cheaper commitment hash" is a non-lever; the lever is "don't hash
  a *string*".

**Conclusion for the diagnosis:** any survey that promises a big win "via Plookup
range proofs" on these members has mis-located the cost. The comparison is ~13
gates; the gap is the ~14,300-gate string-hash. The ~4k examples are cheaper
**because they bind the operand numerically, not via an in-circuit string hash.**

## 3. Concrete reduction plan: technique → dominant cost → expected gate count

### 3.1 The candidate that reaches ~4k: a numeric value lane in the commitment

**Technique.** The in-circuit blake3 exists *only* because the commitment scheme
encodes a numeric literal as `Enc = h2(LITERAL, blake3(token))` — a string hash with
no field handle on the number. If the commitment scheme **additionally** bound a
numeric encoding of canonical-typed numeric literals — for example
`Enc_num = h2(NUM_TYPE, Poseidon2(value_as_field, scale, sign))` — then the FILTER
circuit would prove its predicate against the **Poseidon2-bound field value
directly** and never reconstruct or hash the string token.

**Dominant cost it removes.** The ~14,300-gate blake3 string-hash + its 31-byte
truncation loop (`filter_int.nr:84-91`).

**Expected post-reduction gate count (estimate — must be measured).** The blake3
binding (~14,300) is replaced by **one Poseidon2 permutation ≈ 74 gates**. The
comparison floor is unchanged at ~3,113 (the `filter_f64` measurement). New member:

```text
~3,113 (comparison + UltraHonk/range floor)  +  ~74 (one Poseidon2 over the value)
≈ ~3,200 gates  — squarely in the ~4k band, a ~5.4× reduction from 17,416.
```

This estimate is bracketed by **two measured anchors on this box**: the
no-binding comparison floor is **3,113** (`filter_f64`), and one Poseidon2
permutation is the `hashes.nr` ~74-gate cost note (the same `h2` already inside the
17,416-gate member at zero marginal cost). It is **not** a claim until re-measured
with `bb gates` on the actual numeric-lane member — see the standing warning in §5.

**Why this is NOT a circuit-only, low-risk change (the decision driver).** The
numeric lane is a **commitment-scheme change**: it must be added to
`crates/sparq-zk/src/encode.rs` and `crates/sparq-zk/src/commit.rs` (the host
commit path), the scan proof must expose/anchor the numeric-lane encoding, and the
verifier must check that the scan anchors `Enc_num` exactly as it anchors
`operand_enc` today. It **relocates** the soundness-enforcing canonical-form binding
(see A1/A4 in §3.4) from the string token onto the numeric lane — it does not delete
it, but it changes *what is bound and where*. That is precisely the kind of change
that must wait on the external audit (sq-qhy4), and it touches the host commitment
across crate boundaries, not just one `.nr` file. The `filter_float.nr:9-12`
docstring already names this exact gap ("a numeric value lane in the sparq-zk
encoding").

### 3.2 The dominated alternative (rejected): swap blake3 → Poseidon2 over packed token bytes

Keep the string token but hash it with Poseidon2 over the bytes packed into field
elements (~3 fields of 31 bytes each → one Poseidon2 permutation, ~74 gates). This
*also* removes the ~14,300-gate blake3. **Rejected** because it is strictly
dominated by §3.1: it still pays the in-circuit token reconstruction and
byte-packing, and it *still* changes the commitment hash for literals (so the host
`h_s` must change to Poseidon2 too — the same audit-gated commitment-scheme change
as §3.1, with none of the §3.1 simplification). §3.1 avoids the string entirely.
Recorded only to show it was considered and is the worse form.

### 3.3 The signature-member cost class (separate from the ~4k FILTER question)

The `hidden_issuer_d4` (16,932) and `holder_pok` (10,334) members are a **different**
dominant cost: in-circuit Baby-JubJub scalar-multiplication. `issuer.nr:117-135`
(`scalar_mul`) is a 251-bit double-and-add; each `point_add` (`issuer.nr:99-108`)
does **two Field divisions** (`/(1+dxxyy)`, `/(1-dxxyy)`) — ~251 × 2 doubles/adds ×
2 scalar-muls, i.e. thousands of in-loop Field inversions, is the bulk of the cost.
Two curve-preserving / curve-changing levers:

- **(a) Projective / extended-twisted-Edwards coordinates** defer all the in-loop
  inversions to a single normalisation per scalar-mul (one inversion instead of
  ~2000). This is algebraically equivalent (the on-curve, identity-reject, `< L`,
  and challenge-reduction binds — A9 in §3.4 — all stay), so it does **not** touch
  the soundness-enforcing equalities, only the coordinate system. Plausibly 3–5× on
  the scalar-mul body — **must be measured**.
- **(b) Re-key issuer signatures to Grumpkin** so Noir's native
  `multi_scalar_mul`/`embedded_curve_add` black-boxes apply (~10× on the scalar-mul).
  **Blocked as a drop-in**: `sig.rs` signs over **Baby-JubJub**, and those black-boxes
  operate on **Grumpkin** (`issuer.nr:35-43`) — feeding Baby-JubJub coords to the
  Grumpkin black-box would verify a signature on the *wrong curve*, a silent
  soundness break. So (b) is a host **signature-scheme change** in
  `crates/sparq-zk/src/sig.rs`, audit-gated.

Neither is the ~4k FILTER lever, but both are recorded because the user's question
("comparable ops at ~4k") generalises to the issuer/holder members. Lever (a) is the
highest-value curve-preserving optimisation and does not change any soundness
constraint; (b) is the larger, audit-gated scheme change.

### 3.4 Must-keep soundness constraint set, and exactly how each survives §3.1

A reduction is "safe to propose" only if **every** constraint below survives, with
the cited constraint intact. For the §3.1 numeric-lane change, each line states how
it survives (or what it relocates). File:line citations were verified against the
current checkout.

**A. DOB bound to the signed/committed credential (not attacker-chosen).**

- **A1 — operand binding.** `filter_int.nr:92` / `filter_signed.nr:114`
  `assert_eq(h2(LITERAL, hs), operand_enc, "operand encoding mismatch")`. The
  compared value is a constrained function of the committed literal bytes
  (`filter_int.nr:67-70`), not a free witness. *Survives §3.1 by RELOCATION:* the
  binding moves from `h2(LITERAL, blake3(token))` to `h2(NUM_TYPE,
  Poseidon2(value_field))` against a new `Enc_num`. The equality assert and the
  "value is a constrained function of the committed encoding" property are kept; the
  preimage changes from the string token to the numeric field. The host
  (`encode.rs:53-54`, `to_string()` verbatim) and `field_from_hash_bytes`
  (`field.rs`) string path is **retained for the string lane**; the numeric lane is
  additive. *This relocation is the audit-gated step (sq-qhy4): it must be shown the
  numeric lane binds the SAME committed triple the scan proof exposes.*
- **A2 — `operand_enc` (or `Enc_num`) is the field the scan proof bound to a
  committed triple.** `verifier.rs:2085` `if scanned != operand { return
  Err(BindingInconsistent) }` (the `binding_consistency` edge, covering
  `FilterInt`/`FilterF64`/`FilterSignedInt`/`FilterDecimal`,
  `verifier.rs:2078-2087`). *Survives §3.1 unchanged in MECHANISM:* the plain
  field-equality edge is kept; the scan proof must expose the numeric-lane slot so
  `scanned` is the `Enc_num` the FILTER proves against. Removing this edge would let
  a FILTER prove `≥18` about a literal from no credential — must not weaken.
- **A3–A7 — scan binds the witnessed graph to the public commitment.**
  `scan.nr:104` `commit_fold(leaves, counts[g]) == commitments[g]` (re-commit);
  `scan.nr:146,149` disclosed-row-present; `scan.nr` completeness; `scan.nr:121`
  `commitments[g-1].lt(commitments[g])` strictly-increasing (anti
  duplicate-inclusion / COUNT forgery), mirrored at `verifier.rs:2043-2047`;
  `scan.nr:181` `attribution[g] == graph_matches`. *Survives §3.1 unchanged:* the
  scan proof is untouched except to additionally expose the numeric-lane encoding;
  all A3–A7 asserts that make `operand_enc`/`Enc_num` actually *come from* the
  committed graph stay byte-for-byte.
- **A8 — commitment signed by a trusted issuer (clear tier).**
  `verifier.rs:2128-2129` `bind_issuer_attestations(manifest, trusted_key_set,
  hidden_covered)` against the **external** `trusted_key_set` (never
  `manifest.key_set`); fail-closed "neither clear nor hidden ⇒ reject". *Survives
  §3.1 unchanged:* the FILTER change does not touch issuer attestation.
- **A9 — hidden-issuer tier in-circuit Schnorr.** `issuer.nr` on-curve
  (`:145`), identity-key rejection (`:198`-region), `s < L` (`assert_lt_l`,
  `:173-179`), challenge-reduction bind (`:163` `assert(e + e_k*BJJ_L == e_base)`),
  verify equation, Merkle root. *Survives §3.1 unchanged:* the FILTER change does not
  touch issuer. (For the §3.3 levers: (a) keeps every A9 bind; (b) is curve-changing
  and audit-gated.)

**B. The comparison `value >= bound` is correct over the field — no wraparound.**

- **B1 — operands in a range-checked integer domain; no signed-field wrap.**
  Magnitudes accumulate into `u64` (`filter_int.nr:67-70`, `filter_signed.nr:150-153`,
  `filter_decimal_check:230-242`); the static overflow guards `D<=19`
  (`filter_int.nr:53-54`), `MD<=19` (`filter_signed.nr:137-138`), `ID+FD<=19`
  (`filter_signed.nr:210-212`) are must-keep. *Survives §3.1 unchanged:* the
  numeric lane supplies a field value, but the value is still re-derived into the
  `u64` domain from canonical digits for the comparison, keeping the no-wrap
  property. The reduction must **not** collapse this into a single signed `Field`.
- **B2 — verdict correct and constant-shape.** `filter_int.nr:96-110`,
  `signed_verdict` (`filter_signed.nr:57-87`) — all six predicates evaluated
  **unconditionally** over **typed `u64`**, no secret-dependent branch. *Survives
  §3.1 unchanged:* the comparison is exactly `filter_f64`'s shape, untouched. A
  reduction that lowered it to `Field` arithmetic reintroduces modular wrap and is
  **rejected** (see §2, §3.4-B1).
- **B3 — verdict asserted equal to public `expected`.** `filter_int.nr:111` /
  `filter_signed.nr:183` / `filter_decimal:284`. *Survives unchanged.*
- **B4 — canonical-lexical discipline (no second encoding, no `-0`).** digit-range
  (`filter_int.nr:58-59`), no-leading-zero (`filter_int.nr:61-64`), `-0` rejected
  (`filter_signed.nr:156-158`), op-in-range (`filter_int.nr:55`). *Survives §3.1 by
  RELOCATION:* these protect A1 by forbidding a non-canonical token that re-encodes
  the same value. Under the numeric lane the canonical-form bind must be **carried
  onto the numeric encoding** (e.g. canonical scale, no leading zeros baked into
  `value_field`), so a prover cannot bind a second encoding of the same number. This
  is the part the audit must verify is preserved, not merely "equivalent".

**C. Public inputs bind correctly — no malleability.**

- **C1 — verifier-side public-input reconstruction byte-matches the proof.**
  `verifier.rs:4654` `if reconstructed != art.public_inputs { return
  Err(PublicInputMismatch) }`; per-variant serialization order must match `main`'s
  `pub` order. *Survives §3.1 by MANDATORY CO-CHANGE:* a numeric-lane FILTER member
  changes the `pub` parameter list (a new lane field), so `reconstruct_public_inputs`
  AND the real-bb cross-vectors (e.g. `reconstruct_filter_int_…`) **must be updated
  in lockstep**, or the byte-compare silently diverges. This is the load-bearing tie
  between the JSON statement and the detached proof — listed here so the reduction
  cannot forget it.
- **C2 — canonical verifier-side vk, never the prover's.** `verifier.rs:4666`
  `canonical_vk(id, …)` recomputed from the re-derived `CircuitId`. *Survives §3.1
  by CO-CHANGE:* `derive_id` must pin the new member's parameters.
- **C3 — field 0 is the verifier nonce, and the declared binding equals it.**
  `reconstruct_public_inputs` pushes the challenge as field 0; the manifest's declared
  challenge must equal the verifier nonce (`NonceBindingMismatch` otherwise). *Survives
  unchanged.*
- **C4 — query-correctness binding.** `verifier.rs:2096`
  `bind_query_correctness(manifest)` + `bind_attributions` — `op`/`bound`/`expected`/
  pattern constants match the relying party's query. *Survives unchanged.*

**D. Nullifier / uniqueness / replay.**

- **D1 — single-use nonce, burn-on-present.** `verifier.rs` `record_fresh` (declared
  `:827`, file-backed `:949`) before the crypto gate; a rejection is never a free
  retry. *Survives unchanged.*
- **D2 — holder possession.** clear tier `bind_holder_pop`; in-circuit hidden tier
  `holder.nr:95-109` `holder_pok` (on-curve, identity rejection, `hsk < L`,
  `hpk == hsk*G`, digest binding) — **NOT-yet-sound / opt-in (sq-qhy4)**. *Survives
  §3.1 unchanged:* the FILTER change does not touch the holder path.

**Constraints that would be TRADED if mis-implemented — the reject list.** A
reduction that (i) lowers the `u64` comparison to `Field`-arith (re-introduces
modular wrap — violates B1/B2); or (ii) drops the canonical-form binds when moving to
the numeric lane (lets a prover bind a second encoding of the same value — violates
A1/B4); or (iii) takes the operand as a free witness/public input without anchoring
it to the scan-bound commitment (violates A1/A2); or (iv) feeds Baby-JubJub coords to
the Grumpkin native MSM black-box (verifies on the wrong curve — violates A9) — each
**trades soundness for gates and is REJECTED.** The numeric-lane change is only
acceptable if it carries A1/B4's canonical binding onto the numeric encoding and the
scan anchors `Enc_num` (A2).

## 4. Decision: design-only — implement no circuit change now

The only path to the ~4k band for the FILTER members is §3.1 (the numeric value
lane). By the §3.4 analysis it is **not** a circuit-only, clearly-low-risk change:
it relocates the soundness-enforcing canonical-form binding (A1/A4), changes the host
commitment scheme across `crates/sparq-zk` (`encode.rs`, `commit.rs`, the scan
anchor, the verifier reconstruction + cross-vectors), and is exactly the kind of
commitment-scheme change that is audit-gated under sq-qhy4. The §3.3 scalar-mul
levers are likewise either algebraically non-trivial (must be measured) or a host
signature-scheme change.

**Per the task's gate, no candidate is "clearly low-risk AND provably preserves all
constraints", so this record implements no `.nr` change.** The implementation work is
captured as beads, children of the ZK epic **sq-1s2**, linking this record:

- **sq-j506** — add a numeric encoding lane to `sparq_zk::encode`/`commit`, a
  numeric-lane FILTER member, scan anchoring, and the verifier reconstruction +
  real-bb cross-vectors; re-measure `bb gates`, re-baseline the snapshot. Audit-gated
  (sq-qhy4). Expected member ~3,200 gates (estimate; must be measured).
- **sq-hb75** — rewrite `issuer.nr` `point_add`/`scalar_mul` in projective/extended
  TE coordinates (one normalisation per scalar-mul); re-measure. Curve-preserving
  (keeps all A9 binds); the issuer/holder members' ~4k path.

## 5. Standing measurement warning

Every projected post-reduction figure (~3,200 for the numeric-lane FILTER; "3–5×" on
the scalar-mul body) is an **estimate bracketed by measured anchors** — it must be
confirmed with `bb gates -s ultra_honk` on the actual changed member before it is
claimed. The noir-optimisation skill's entire thesis (the PR #37 / `shr_sticky`
spike) is that inversion/decomposition costs surprise intuition on this codebase. The
gate-count regression test (`crates/sparq-zk-compose/tests/gate_count.rs`) will trip
if a constraint set is silently dropped (a removed `assert` typically moves the count
tens of percent), so any landed reduction must re-run
`bench/zk-compose/scripts/gate_counts.sh` and re-baseline
`tests/gate_count_snapshot.json` deliberately.

## 6. Measured baselines and impl beads

Measured on this box (`bb gates -s ultra_honk`, `circuit_size`), reproducing
`crates/sparq-zk-compose/tests/gate_count_snapshot.json`:

| Member | `circuit_size` | Note |
| --- | ---: | --- |
| empty `main` (probe) | 35 | UltraHonk/ACIR floor |
| `filter_f64` (no binding) | 3,113 | comparison + floor only |
| blake3-over-48B probe | 17,416 | the string-hash binding alone |
| `filter_int_d{1..4}` | 17,416 | age-style FILTER (blake3 binding) |
| `filter_signed_int_d{2,4}` | 17,416 | signed variant |
| `filter_decimal_i3_f2` | 17,416 | decimal variant |
| `hidden_issuer_d4` | 16,932 | two Baby-JubJub scalar-muls (§3.3) |
| `holder_pok` | 10,334 | one scalar-mul (§3.3) |

Expected after §3.1 (estimate, must be measured): a numeric-lane FILTER member
**≈ 3,200 gates** (~5.4× reduction from 17,416), preserving the §3.4 constraint set
pending the external audit (sq-qhy4).

**Key files.** `zk/compose/compose_core/src/filter_int.nr` (blake3 binding, lines
74-92), `filter_signed.nr` (`assert_literal_binding`, 93-115), `filter_float.nr`
(3,113 vs 17,416 comparison, 127-152), `hashes.nr` (Poseidon2 ~74-gate cost note,
12-15), `issuer.nr` (`point_add`/`scalar_mul` inversions, 99-135; A9 binds),
`scan.nr` (A3–A7 binds), `crates/sparq-zk-compose/src/verifier.rs`
(`scanned != operand` 2085, public-input reconstruction 4654, `canonical_vk` 4666);
measured snapshot `crates/sparq-zk-compose/tests/gate_count_snapshot.json`; cost
model `.claude/skills/noir-optimisation/SKILL.md`.
