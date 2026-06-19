<!-- [OPUS-4.8] Design-for-review authored by Opus 4.8 (1M context) (Fable unavailable) — re-review when Fable returns. -->
# Field-native ZK term encoding: a numeric VALUE_HOOK in the commitment scheme

Maintainer-review design record. The maintainer has greenlit, **at research
grade**, an encoding overhaul: re-base sparq's ZK term/literal commitment onto
the value-first, field-native scheme he used in his original ZK work —
`hashFields([hashString(value), termTypeAsNumber])` for non-literals, and an
inner/outer literal hash carrying a per-datatype `VALUE_HOOK` — *provided* it
gives a gate-count improvement and introduces no obvious soundness issue that can
be identified and reasoned about. He further directed: **introduce the numeric
hook now (research grade) and register it as an explicit item the external audit
must check.**

This record is the **implementation-design follow-up** to
[`research/zk-age-gatecount-reduction.md`](./zk-age-gatecount-reduction.md) — the
gate-count attribution that located the cost (read it first; this record builds
directly on its **§3.4 must-keep constraint set** and **§3.1 numeric value
lane**). Companion analyses: [`research/zk-soundness-audit.md`](./zk-soundness-audit.md),
[`research/zk-verifier-reaudit.md`](./zk-verifier-reaudit.md),
[`research/zk-hidden-join-design.md`](./zk-hidden-join-design.md).

Parent: epic **sq-1s2** ("ZK query-proof build-out + in-circuit privacy
upgrades"). This is a **design-for-review**: it changes no `.nr` / `.rs` source,
creates no bead (the orchestrator owns bead structure; recommended children are
listed in the final section).

## 1. Honesty framing (load-bearing — read first)

sparq's v1 ZK query-proof verifier (`sparq-zk` / `sparq-zk-compose`) is
**remediated and internally re-audited but NOT externally audited**, and is
documented **NOT-yet-sound** for production reliance (beads **sq-qhy4**,
**sq-9hrn**, **sq-1s2**; `SECURITY.md`; `compliance/cryptoreview/gap-register.md`
headline CR-G1). An external accredited-cryptographer sign-off (**sq-qhy4**, P0)
is **REQUIRED before any ZK soundness / privacy / integrity property may be
relied upon in production**.

Nothing in this record is a security guarantee. The encoding change proposed here
is described as **preserving the §3.4 soundness-enforcing constraint set pending
the external audit** — that phrase means the exact equalities, range-binds and
canonical-form asserts the current circuit makes are *relocated, not removed*, and
their preservation is itself an **audit obligation**, never an established fact.
A relocation is only "safe to propose" if **every** §3.4 must-keep demonstrably
survives with the cited constraint intact (not merely "equivalent"); even then the
framing stays "preserves the constraint set, pending sq-qhy4". The new
value-collapse issue in §4 is flagged as an **OPEN audit obligation**, not a
resolved property.

The privacy-claims CI gate (`scripts/check-privacy-claims.sh`) path-excludes
`research/**` (it defends the *outward* claim surface, not design records). The
caveat wording this record proposes for the ZK `SKILL.md` and the `sparq-zk`
README in §5 — which **is** on that scanned surface — is written negated /
obligation-framed so it passes the live gate.

All gate counts here are the **measured `bb gates -s ultra_honk` `circuit_size`**
already snapshotted and regression-gated at
`crates/sparq-zk-compose/tests/gate_count_snapshot.json` — a circuit metric, not a
performance-marketing number. Toolchain: `nargo 1.0.0-beta.21`,
`bb 5.0.0-nightly.20260324` (the snapshot's baselined toolchain). Every
*projected* post-change figure is an **estimate bracketed by measured anchors**
and is NOT a claim until re-measured with `bb gates` on the actual changed member
(§3.4).

## 2. The scheme, mapped to the actual code

### 2.1 What is encoded today (verified against the checkout)

The host commitment path is `crates/sparq-zk/src/encode.rs`; the in-circuit mirror
is `zk/compose/compose_core/src/hashes.nr`. The current term encoding is
`Enc_t(term) = h2(type_code, h_s(value))` with `h2 = Poseidon2([·, ·], 2)` and
`h_s = blake3` computed **off-circuit** (`encode.rs:46-62`, `hashes.nr:20-22`):

- IRI: `Poseidon2([TYPE_CODE_IRI, blake3_field(iri_bytes)])` (`encode.rs:48-51`).
- Literal: `Poseidon2([TYPE_CODE_LITERAL, blake3_field(to_string())])`
  (`encode.rs:52-55`) — `oxrdf`'s `Literal::to_string()` passes the lexical form
  through **verbatim** (it does NOT re-canonicalise); the hashed token is
  `"<lexical>"^^<datatype>` / `"<lexical>"@<lang>` with `<lexical>` exactly the
  bytes ingested (`filter_signed.nr:9-12` confirms this).
- Blank node: `Poseidon2([TYPE_CODE_BLANK_NODE, Poseidon2([salt_G,
  blake3_field(label)])])` (`encode.rs:56-59`) — graph-scoped per-graph salt
  (the Q6 cross-graph correlation close; out of scope for this change and
  retained unchanged).

`termTypeAsNumber` already exists as the static map **`TYPE_CODE_IRI = 1`,
`TYPE_CODE_LITERAL = 2`, `TYPE_CODE_BLANK_NODE = 3`** in BOTH places
(`encode.rs:33-35`, `hashes.nr:35-37`). The maintainer's `termTypeAsNumber` is
this exact map; no new constant is invented.

### 2.2 The one structural difference from the maintainer's spec: leaf-tuple order

The maintainer's spec is **value-first / type-last**:
`hashFields([hashString(value), termType])`. The current code is **type-first /
value-last**: `Poseidon2([type_code, h_s])`. (Confirmed against `encode.rs:48-58`
and `hashes.nr:20`.) Tuple order in a Poseidon2 sponge is a **convention** — both
are collision-resistant — but the two are **not interchangeable**: Poseidon2 is
order-sensitive, so flipping the order **re-bases every leaf value** in the
commitment.

**Recommendation:** adopt the maintainer's order (`Poseidon2([value, termType])`)
to make sparq's encoding match his original ZK work and the spec he wrote, **and
accept the one-time consequence**: every checked-in commitment/leaf vector, every
real-bb cross-vector, the `gate_count_snapshot.json` baselines, and any persisted
`<urn:sparq:zk>` commitment must be **recomputed and re-committed in lockstep** in
the same change. This is mechanical but must be done atomically (host + circuit +
all vectors), or the verifier's byte-compare (`verifier.rs:4653-4655`,
`PublicInputMismatch`) diverges silently. The order flip is **not free of
soundness consequence by itself** beyond the recommit — domain separation is
unchanged (the type field still separates IRI/literal/bnode leaves) — but it is a
breaking, one-time, must-recommit migration and must be sequenced as such.

### 2.3 Non-literal terms (IRI, blank node)

Under the maintainer's order:

- IRI: `Poseidon2([hashString(iri), TYPE_CODE_IRI])`, `hashString = blake3`
  off-circuit (**unchanged in cost**). IRIs keep the string lane: there is no
  numeric handle to give them, and in-circuit blake3 over an IRI only ever appears
  when a FILTER must re-derive an IRI (e.g. `STR()` / IRI-equality predicates),
  which the string lane already covers. No change to the IRI cost profile.
- Blank node: `Poseidon2([Poseidon2([salt_G, hashString(label)]), TYPE_CODE_BLANK_NODE])`
  — the salt-scoped inner is retained (Q6), only the outer tuple order flips.

### 2.4 Literal terms — the value-first inner/outer scheme

The maintainer's literal scheme, mapped onto sparq's `h2`/`Poseidon2`:

```text
inner = Poseidon2([VALUE_HOOK, hashString(datatype), hashString(lang)])
Enc   = Poseidon2([inner, TYPE_CODE_LITERAL])
```

- `hashString(datatype)` = `blake3` over the datatype IRI string (off-circuit for
  ingest; an in-circuit **constant** for the known FILTER datatypes — see §2.6).
- `hashString(lang)` = `blake3` over the language tag, or a fixed sentinel
  (e.g. the field 0 / `blake3("")`) for non-`rdf:langString` literals. The
  inner tuple folds datatype **and** language in (matching today's single
  `blake3(to_string())` which already folds both, `encode.rs:52-54` /
  `hashes.nr` comment), so no information is lost relative to today.
- `VALUE_HOOK` = a per-datatype field encoding of the literal's **value** (§2.5).

This is a strict generalisation of today's single `blake3(token)`: today the
entire datatype+lang+lexical is folded into one off-circuit blake3; under the new
scheme the datatype and lang are still hashed (so distinct-shape literals stay
distinct, as `encode.rs` test `literal_shapes_are_distinguished:104-117`
requires), but the **value** gets a field-arithmetic handle instead of being
buried inside an opaque string hash.

### 2.5 VALUE_HOOK per datatype — which get the cheap field handle, which keep the string

The handle is **injective on value within a datatype** and is what a FILTER
compares against in-circuit without re-hashing a string. Proposed canonical,
range-bound, injective encodings:

| Datatype | VALUE_HOOK | Handle kind | Notes / canonical-form obligation |
| --- | --- | --- | --- |
| `xsd:boolean` | `0` (false) / `1` (true) | cheap field | injective on the two values; `"true"`/`"1"` and `"false"`/`"0"` lexical forms map to the same hook (see §4 collapse). |
| `xsd:integer` (incl. signed) | signed value as a field, **range-bound** (kept in the `u64` magnitude + sign domain `filter_signed.nr` uses; NOT a raw wrapping `Field`) | cheap field | canonical: no leading zeros, no `-0` (carry `filter_int.nr:58-64`, `filter_signed.nr:142-158`). |
| `xsd:decimal` | canonical `(sign, unscaledValue, scale)` OR a single fixed-scale scaled integer | cheap field | **see decision below.** Carry the digit-canonicality asserts of `filter_decimal_check` (`filter_signed.nr:210-246`: integer-part no-leading-zero, `ID+FD` magnitude fits `u64`, `-0` reject). |
| `xsd:double` / `xsd:float` | the **IEEE-754 bit pattern** as a field (64 bits for double, 32 for float) | cheap field | sign the bits directly — see the sq-mslu note below. |
| `xsd:dateTime` / `xsd:date` | a **canonical epoch encoding** (e.g. signed seconds-since-epoch + a fractional-seconds sub-field for `xsd:dateTime`, with timezone normalised to UTC) OR a canonical `(year, month, day, …)` component tuple | cheap field (DESIGN-ONLY) | the canonical-component vs epoch decision and the timezone-normalisation rule are an **open design item** — see §8 open questions. NOT in the first implementation slice. |
| `rdf:langString`, `xsd:string`, **any unknown / opaque datatype** | `VALUE_HOOK = hashString(lexical)` (the string lane) | **string hash — no numeric handle, cost unchanged** | the FALLBACK. These have no value-FILTER comparison the circuit performs numerically, so they keep the blake3 string hash exactly as today. |

**Decimal decision — scaled-int at a member-fixed scale, NOT free
`(unscaledValue, scale)`.** `filter_decimal_check` already compares decimals as a
**host-prescaled fixed-point integer** at a member-comptime `FD` fraction places
(`filter_signed.nr:200-246`, `mag_scaled = int_part * 10^FD + frac_part`, with
`ID+FD` bounded so the scaled magnitude fits `u64`). A free `(unscaledValue,
scale)` pair would let `5.0` (unscaled 50, scale 1) and `5.00` (unscaled 500,
scale 2) be **different** hooks for the same value, multiplying the §4 collapse
surface and breaking the single-comparison domain. Therefore VALUE_HOOK for
decimal should be the **canonical scaled integer at the member's fixed scale**
(matching the existing comparison lane), with the existing `filter_decimal`
canonical-digit asserts carried onto it. This keeps one comparison domain and one
injective value→hook map per member. (Cross-member: a decimal compared at two
different `FD` scales is two different members, exactly as today; the scan anchor
pins which member's encoding the literal was committed under — an audit obligation,
§4.)

**Double/float — sign the IEEE bits directly; this may obsolete sq-mslu's RNE
parser.** `filter_float.nr:1-12` documents the deferred work as an *in-circuit
decimal→IEEE round-to-nearest-even parser* needed because the literal was committed
as a **string** (`"1.5E3"`), so the circuit had to re-derive the bits from the
lexical form. If VALUE_HOOK **is** the IEEE-754 bit pattern (the value, not the
lexical), the commitment binds the bits directly and the in-circuit decimal→IEEE
parser is **not needed for the comparison** — the circuit witnesses the bits, the
comparison is `sparq_ieee754`'s `f64` predicates (`filter_float.nr:29-43`;
`filter_f64` raw-compare floor = **3,113 gates** measured), and the binding is one
Poseidon2 over the bits (§2.6). **Confirm against `filter_float.nr`:** the present
`filter_f64_composable_check` derives bits from *digits* via `f64::from(u64)`
(`filter_float.nr:118-148`) precisely because there was no numeric handle; with a
bit-pattern VALUE_HOOK that derivation path collapses to "witness the committed
bits, assert the Poseidon2 binding." **Caveat (sq-mslu):** the *ingest-side*
canonicalisation of an arbitrary lexical double to its canonical IEEE bit pattern
still happens **off-circuit** at commit time, and "which lexical forms map to which
bits" is exactly the §4 value-collapse question for doubles (`"1.0"`, `"1.0E0"`,
`"1"^^double` all → the same bits). So sq-mslu's RNE *parser-in-circuit* may be
obsoleted for the comparison, but a canonical IEEE-bit ingest rule (and its
collapse consequence) replaces it — this must be confirmed against
`filter_float.nr` and `sparq_ieee754` before the bead closes, not assumed.

### 2.6 The reduction mechanism — why this reaches the ~4k band

For the **known FILTER datatypes**, `hashString(datatype)` and `hashString(lang)`
are **fixed compile-time strings** (e.g. `^^<…#integer>`, lang = the empty
sentinel). So they can be supplied to the circuit as **precomputed field
constants**, and the in-circuit operand binding becomes:

```text
inner = Poseidon2([VALUE_HOOK, DATATYPE_CONST, LANG_CONST])   // 1 perm
Enc   = Poseidon2([inner, TYPE_CODE_LITERAL])                  // 1 perm
assert_eq(Enc, operand_enc)
```

— **two Poseidon2 permutations over the witnessed VALUE_HOOK** (~74 gates each per
the `hashes.nr:12-15` cost note, ~150 gates total), replacing **one in-circuit
blake3 compression** over the ~46–90-byte canonical N-Triples token
(`filter_int.nr:74-92`).

Measured anchors from `zk-age-gatecount-reduction.md` §6 (re-measured on this box,
reproducing the snapshot byte-for-byte):

| Anchor | `circuit_size` | What it isolates |
| --- | ---: | --- |
| `filter_f64` (raw compare, no binding) | 3,113 | comparison verdict + UltraHonk/range-table floor |
| blake3-over-48B probe | 17,416 | the in-circuit string-hash binding **alone** |
| `filter_int_d{1..4}` / `filter_signed_int_d{2,4}` / `filter_decimal_i3_f2` | 17,416 | the full blake3-bound FILTER members |

The blake3 binding (~14,300 gates = 17,416 − 3,113) is replaced by ~150 gates of
Poseidon2. **Projected numeric-FILTER member ≈ 3,200 gates** (the 3,113
comparison floor + ~150 binding), a **~5.4× reduction from 17,416**.

> **This ≈3,200 figure is an ESTIMATE bracketed by the two measured anchors above.
> It is NOT a claim.** It MUST be confirmed with `bb gates -s ultra_honk` on the
> actual numeric-lane member and re-baselined into
> `crates/sparq-zk-compose/tests/gate_count_snapshot.json` (and
> `bench/zk-compose/gate_counts_latest.json`) before it is quoted anywhere. The
> noir-optimisation skill's standing warning (PR #37 / `shr_sticky` spike) is that
> inversion/decomposition costs surprise intuition on this stack — measure first.

**Cost asymmetry to note honestly:** the win applies to the **numeric FILTER
datatypes** (integer / signed / decimal / double-float / boolean) and to
`xsd:dateTime`/`date` *if* implemented. IRIs, blank nodes, plain strings, and
opaque datatypes **keep the string lane** and see **no change** in cost. The
overhaul is an additive value-handle for hookable datatypes, not a blanket re-hash.

## 3. Soundness analysis — §3.4 must-keeps carried onto the VALUE_HOOK lane

The maintainer asked me to identify and reason about "obvious soundness issues."
Each §3.4 must-keep is restated with how it survives the value-first VALUE_HOOK
encoding, or is flagged as an **AUDIT OBLIGATION** (sq-qhy4). File:line citations
verified against the current checkout. None of the below is asserted as proven —
each is "preserved by construction *if* the relocation is done as described, which
the external audit must verify."

**A. Operand bound to the signed/committed credential (not attacker-chosen).**

- **A1 / A4 — operand binding by RELOCATION.** Today: `filter_int.nr:92` /
  `filter_signed.nr:114` `assert_eq(h2(LITERAL, hs), operand_enc)` where `hs` is a
  blake3 over the rebuilt string token. Under the new scheme the **preimage
  changes** from the string token to the field-native tuple — the assert becomes
  `assert_eq(Poseidon2([Poseidon2([VALUE_HOOK, DT_CONST, LANG_CONST]),
  TYPE_CODE_LITERAL]), operand_enc)`. The equality assert is **kept**; the
  "compared value is a constrained function of the committed encoding, not a free
  witness" property is **kept** (VALUE_HOOK is derived from the same witnessed
  digits/bits the comparison uses). **AUDIT OBLIGATION:** the relocation must be
  shown to bind the SAME committed triple the scan proof exposes (sq-qhy4).
- **A2 — scan anchors the new literal leaf.** `verifier.rs:2078-2087`
  (`scanned != operand` ⇒ `BindingInconsistent`) is the plain public-input
  equality edge over `FilterInt`/`FilterF64`/`FilterSignedInt`/`FilterDecimal`
  `operand_enc`. The scan proof's `enc[g][i][·]` slots (`scan.nr:99-108`
  commit-recompute) must now carry the **new value-first leaf** `Enc_num`, and the
  edge must equate `Enc_num` (not the old string-hashed `Enc`). The edge MECHANISM
  is unchanged; what flows through it changes. Removing/weakening this edge would
  let a FILTER prove `≥18` about a literal from no credential — **must not weaken**.
- **A3–A7 — scan binds the witnessed graph to the public commitment.**
  `scan.nr:103-108` `commit_fold(leaves, counts[g]) == commitments[g]`;
  `scan.nr:119-124` strictly-increasing commitments (anti duplicate-inclusion /
  COUNT-forgery), mirrored `verifier.rs:2043-2047`; row-present `scan.nr:139-149`;
  completeness + attribution `scan.nr:158-182`. **Survives unchanged** except the
  leaf each slot hashes is the new value-first encoding; all asserts that make
  `Enc_num` *come from* the committed graph stay byte-for-byte.
- **A8 / A9 — issuer attestation / in-circuit Schnorr.** `verifier.rs:2108+`
  `bind_issuer_attestations` (external `trusted_key_set`); `issuer.nr` on-curve /
  identity-reject / `s < L` / challenge-reduction / verify / Merkle root. The
  FILTER-encoding change **does not touch** the issuer path. Survives unchanged.

**B. Comparison `value ⋈ bound` correct over the field — NO modular wrap.**

- **B1 — operands in a range-checked integer domain, no signed-Field wrap.**
  Magnitudes accumulate into `u64` (`filter_int.nr:67-70`,
  `filter_signed.nr:150-153`, `filter_decimal:230-242`); the static overflow guards
  `D ≤ 19` / `MD ≤ 19` / `ID+FD ≤ 19` are must-keep. **Carried onto VALUE_HOOK:**
  the value handle is a field element, but the comparison must still re-derive it
  into the `u64` (magnitude+sign / scaled-int / IEEE-bits) domain and compare
  there. The reduction must **NOT** collapse the comparison into raw `Field`
  arithmetic — that reintroduces modular wrap and is **REJECTED** (see reject-list).
- **B2 — verdict correct and constant-shape.** `filter_int.nr:96-110`,
  `signed_verdict` (`filter_signed.nr:57-87`), `f64_verdict`
  (`filter_float.nr:29-43`) — all six predicates evaluated **unconditionally** over
  **typed `u64`/`f64`**, no secret-dependent branch. The comparison is exactly
  `filter_f64`'s shape, **untouched** by the encoding change.
- **B3 — verdict asserted equal to public `expected`.** `filter_int.nr:111` /
  `filter_signed.nr:183` / `filter_decimal:284` / `filter_float.nr:152`.
  Survives unchanged.
- **B4 — canonical-form bind carried onto VALUE_HOOK (no second encoding, no `-0`,
  no leading zeros baked into the value).** Today (`filter_int.nr:58-64`,
  `filter_signed.nr:142-158`, `filter_decimal:215-246`) these protect A1 by
  forbidding a non-canonical token that re-encodes the same value. Under the
  numeric lane the canonical-form bind must be **carried onto the VALUE_HOOK
  derivation** — a canonical scale for decimal, canonical IEEE bits for double, no
  leading zeros / no `-0` baked into the integer value field — so a prover cannot
  bind a second encoding of the same number. **AUDIT OBLIGATION:** the audit must
  verify this is preserved, not merely "equivalent". This is also the hinge of the
  §4 value-collapse issue.

**C. Public inputs bind correctly — no malleability.**

- **C1 — verifier-side reconstruction byte-matches the proof.**
  `verifier.rs:4653-4655` (`reconstructed != art.public_inputs` ⇒
  `PublicInputMismatch`). A value-first member changes the `pub` parameter list
  (the witnessed VALUE_HOOK and the per-datatype constants), so
  `reconstruct_public_inputs` (`verifier.rs:4754`) **and** the real-bb cross-vectors
  (`reconstruct_filter_int_matches_real_bb_public_inputs` and the f64/signed/decimal
  siblings, `verifier.rs:5010-5208`) **must be updated in lockstep**, or the
  byte-compare silently diverges. This is the load-bearing tie between the JSON
  statement and the detached proof — **MANDATORY CO-CHANGE.**
- **C2 — canonical verifier-side vk, never the prover's.**
  `verifier.rs:4665-4666` `canonical_vk(id, …)`; `derive_id` must pin the new
  member's parameters. CO-CHANGE.
- **C3 / C4 — nonce binding + query-correctness binding.** `verifier.rs` field-0
  challenge / `NonceBindingMismatch`; `bind_query_correctness`
  (`verifier.rs:2096`). Survive unchanged.

**D. Nullifier / uniqueness / replay.** `record_fresh` (`verifier.rs:827/949`);
holder PoP / `holder_pok`. The FILTER-encoding change does not touch these.
Survive unchanged.

### Reject-list (the four soundness-for-gates trades that remain REJECTED, §3.4)

A change that (i) lowers the `u64`/typed comparison to raw `Field` arithmetic
(modular wrap — violates B1/B2); or (ii) drops the canonical-form binds when moving
to VALUE_HOOK (a prover binds a second encoding of the same value — violates
A1/B4); or (iii) takes VALUE_HOOK as a free witness/public input without anchoring
it to the scan-bound commitment (violates A1/A2); or (iv) feeds Baby-JubJub coords
to the Grumpkin native MSM black-box (verifies on the wrong curve — violates A9) —
each **trades soundness for gates and is REJECTED.**

### What binds VALUE_HOOK (the collision argument)

A prover cannot find `V' ≠ V` with the same `Enc_num` because **Poseidon2 is
collision-resistant**: `inner = Poseidon2([VALUE_HOOK, DT, LANG])` and the
equality assert against the scan-anchored `operand_enc` together force the
witnessed VALUE_HOOK to be the committed one (no second preimage). Cross-datatype
value collisions (integer `5` vs decimal `5` vs boolean vs the double bit-pattern
for `5.0`) are prevented by the **datatype field `DT_CONST` inside the inner
tuple**: the same numeric VALUE_HOOK under two different datatype constants hashes
to two different inner values, so an integer-`5` commitment and a decimal-`5`
commitment are distinct leaves. (This is the field-native analogue of today's
`literal_shapes_are_distinguished` test, `encode.rs:104-117`.) **These are
standard collision-resistance arguments under the Poseidon2 assumption — stated as
the design intent the external audit must verify, NOT as a proven property.**

## 4. The NEW soundness issue — value-collapse (term identity vs value identity)

**This is the prominent new issue the maintainer asked me to reason about.**

A value-based hook collapses **distinct lexical forms of the same value to the SAME
leaf**:

- `"05"^^xsd:integer` and `"5"^^xsd:integer` → same VALUE_HOOK (5).
- `"5.0"^^xsd:decimal` and `"5.00"^^xsd:decimal` → same scaled value.
- `"true"`/`"1"^^xsd:boolean`; `"1.0"`/`"1.0E0"`/`"1"^^xsd:double` → same bits.

These are **different RDF terms** — RDF `sameTerm` (and SPARQL `=` term-equality in
the cases it falls back to term comparison, `DISTINCT`, `GROUP BY` keys, and
sameTerm-level joins) **distinguishes** them. Today's string-hash encoding
(`blake3(to_string())` verbatim, `encode.rs:52-54`) preserves **term identity**:
`"05"` and `"5"` hash to different leaves. A value-first hook **silently changes
term identity to VALUE identity for hooked datatypes.**

**Why this is broader than the value-FILTER it was designed for.** The §2.6
reduction only *needs* value identity for `FILTER(?v ⋈ bound)` (value-semantic
comparison). But the **same leaf `Enc_num` is what the scan proof commits and the
join proof equates** (`scan.nr` rows, `join_eq` shared-variable equality). If the
commitment leaf is value-collapsed, then a `JOIN`/`DISTINCT`/`sameTerm` over that
column silently uses value identity, not term identity — so `"05"` and `"5"`
would join/dedup as equal when RDF semantics say they are distinct terms. That is a
**semantic-correctness change to the whole engine's ZK view of the data**, not a
local FILTER optimisation.

### Resolution — RECOMMENDED: (a) canonicalise on ingest, with (b) as the guard

Two candidate resolutions:

- **(a) Canonicalise literals to their canonical lexical form at ingest**, so
  non-canonical forms (`"05"`, `"5.00"`, `"+5"`, `"1.0E0"`) **never enter the
  commitment** — only the canonical form is committed, and value identity ≡ term
  identity *for the committed graph* because the graph contains only canonical
  forms. RDF 1.1/1.2 value-space canonicalisation per datatype is well-defined
  (XSD canonical lexical mapping), and sparq already RDFC10-canonicalises graph
  structure at commit (`commit.rs:79-99`, `canon::canonicalize_*`) — literal
  lexical canonicalisation is an additive normalisation in the **same ingest
  pass**, not a new pipeline. This makes the value-first hook *sound for term
  identity by construction*, because the only forms that exist post-ingest are the
  canonical ones the hook is injective over.
- **(b) Restrict the value-hook leaf to value-semantic comparisons and keep the
  string lane where term identity is required.** I.e. commit BOTH a string-lane
  leaf (term identity, today's encoding) and a value-lane handle (for FILTER), and
  use the value lane *only* in the FILTER member, never in scan/join/DISTINCT.

**RECOMMENDATION: adopt (a) canonical-on-ingest as the primary resolution, and use
(b)'s discipline as a defence-in-depth guard — keep the string lane as the
identity-bearing leaf and treat the value hook as an additional bound handle, so
that even if a non-canonical literal slips past ingest, term identity is still
carried by the string lane and only the FILTER comparison consults the value
hook.** Rationale:

1. **(a) alone is the cheapest and cleanest** — it gives the §2.6 gate win on a
   single leaf and makes value identity ≡ term identity hold by an ingest
   invariant. But it depends on the ingest canonicaliser being **total and
   correct** for every hooked datatype (every XSD canonical mapping, every
   timezone normalisation for dateTime) — a real correctness surface, and a
   relying party who receives a *pre-committed* graph from elsewhere cannot assume
   the producer canonicalised.
2. **(b) alone keeps full term identity but forfeits most of the gate win** for
   scan/join (the string lane stays, so those leaves are unchanged) — it only
   helps the FILTER comparison.
3. **(a)+(b) together**: canonical-on-ingest gives the engine a clean value≡term
   invariant for *its own* commitments, while keeping the string lane as the
   authoritative identity leaf means a non-canonical literal from an external
   producer degrades to "term-distinct, still comparable by value in FILTER"
   rather than "silently value-collapsed in a JOIN." This is the conservative
   resolution: never silently change identity semantics for a leaf used in
   identity-sensitive operators, while still getting the cheap value handle for
   value-FILTERs.

**This value-collapse issue is an EXPLICIT item the external audit (sq-qhy4) must
check** — both the soundness of the chosen resolution and that no
identity-sensitive operator (scan-row equality, `join_eq`, `DISTINCT`, `sameTerm`)
ever consults a value-collapsed leaf. It is registered in §5 as an audit
obligation. It is **not** presented here as resolved.

## 5. Audit-obligation registration (exact wording)

So that the external audit (sq-qhy4) is **forced** to check this change, the
following are the exact texts to add. They are obligation-/negation-framed and
written to pass `scripts/check-privacy-claims.sh` on the live claim surface.

### 5.1 `compliance/cryptoreview/gap-register.md` — new gap-table row

Add under the gap table (matching the existing CR-G* row shape; this is the audit
hook the orchestrator's registration bead will land):

> | **CR-G8** | **Field-native value-hook encoding (`VALUE_HOOK`) is unaudited and relocates the operand-binding + canonical-form constraints from the string token onto a per-datatype value handle; it also collapses distinct lexical forms of equal value to one leaf (term-identity → value-identity for hooked datatypes).** | HIGH | **OPEN / EXTERNAL-REQUIRED** | The external pass (CR-G1 / sq-qhy4) must verify: (1) the relocated operand binding (§3 A1/A4) still binds the scan-committed triple; (2) the canonical-form binds (B4) are carried onto VALUE_HOOK so no second encoding of a value is bindable; (3) the value-collapse resolution (canonical-on-ingest + string-lane identity guard) means NO identity-sensitive operator (scan-row equality, `join_eq`, `DISTINCT`, `sameTerm`) ever consults a value-collapsed leaf; (4) `xsd:double`/`float` IEEE-bit canonicalisation at ingest is total and correct. This property is NOT established — it is an open obligation pending the external audit. | `sq-qhy4` / epic `sq-1s2` |

### 5.2 ZK `SKILL.md` (`skills/zk-query-proofs/SKILL.md`) — one-line caveat

Append to the "Honest scope" / NOT-yet-sound block:

> A field-native value-hook literal encoding (research-grade, design in
> `research/zk-field-native-encoding.md`) is proposed to cut numeric-FILTER gate
> cost; it is NOT implemented and NOT audited, it relocates the operand-binding and
> canonical-form constraints onto a per-datatype value handle, and it collapses
> distinct lexical forms of equal value to one leaf — so it is registered as an
> open external-audit obligation (CR-G8 / sq-qhy4) and provides no soundness or
> privacy guarantee. <!-- privacy-claims-allow: negative/obligation framing — names the unaudited value-hook encoding only to flag it as an open audit obligation; sq-qhy4 -->

### 5.3 `crates/sparq-zk/README.md` — one-line caveat

Append to the "no soundness or privacy claim is made for this pipeline today"
note:

> A proposed field-native value-hook term/literal encoding
> (`research/zk-field-native-encoding.md`, research-grade, NOT implemented) would
> re-base the commitment onto value-first leaves with a per-datatype numeric hook;
> it is unaudited, changes term identity to value identity for hooked datatypes,
> and makes no soundness or privacy claim — it is an open external-audit obligation
> (sq-qhy4). <!-- privacy-claims-allow: negative/obligation framing — flags an unimplemented unaudited encoding as an open audit obligation, asserts no guarantee; sq-qhy4 -->

(The two `privacy-claims-allow:` markers are required because the gate's
predicate-form regex matches "soundness … guarantee"; the inline marker records
that each is a legitimate negative/obligation usage, per the gate's own contract.)

## 6. Comprehensive SPARQL gate-benchmark CATALOG (design only)

The maintainer wants "a very comprehensive benchmark which covers all SPARQL
features, and includes complex filters, paths (including with `+`/`*`/`?`
modifiers) to help identify queries which consume an unnecessarily high number of
gates." This section **designs** the benchmark; it implements nothing (the harness
is a follow-on bead, §7).

### 6.1 Purpose — a coverage map AND an optimisation target list

The benchmark doubles as: (i) a **ZK-coverage map** — for each SPARQL feature,
which circuit member(s) (if any) it compiles to today; and (ii) an **optimisation
target list** — per-member measured `bb gates`, with the high-gate members flagged
as reduction targets. It must **never fabricate a gate number** for a feature the
ZK engine cannot prove today: those entries are honestly labelled
`NO ZK CIRCUIT YET (gap)`.

### 6.2 Query catalog (SPARQL 1.1 feature spanning)

Representative queries, grouped by feature, each tagged with its coverage. The
`circuit_size` column is the **measured snapshot** value for the named member
(`gate_count_snapshot.json`); `null` for gaps.

| # | Feature | Representative query shape | Compiles to ZK member(s) | `circuit_size` | Status |
| --- | --- | --- | --- | ---: | --- |
| Q01 | BGP (1 pattern) | `{ ?s :p ?o }` | `scan_k1_n{16,64}_r{4,8}` | 5,991–18,850 | covered |
| Q02 | BGP (multi-pattern, single graph) | `{ ?s :p ?o . ?s :q ?v }` | multiple `scan_*` | (sum) | covered |
| Q03 | FILTER `xsd:integer` (≥, <, =, ≠) | `FILTER(?age >= 18)` | `filter_int_d{1..4}` | 17,416 | covered (**value-hook target**) |
| Q04 | FILTER signed integer | `FILTER(?bal >= -100)` | `filter_signed_int_d{2,4}` | 17,416 | covered (**target**) |
| Q05 | FILTER `xsd:decimal` | `FILTER(?amt <= 199.99)` | `filter_decimal_i3_f2` | 17,416 | covered (**target**) |
| Q06 | FILTER `xsd:double` (int-valued fragment) | `FILTER(?d > 42)` | `filter_f64_d{1..4}` | 17,416 | covered (**target**); raw-compare floor `filter_f64` = 3,113 |
| Q07 | FILTER boolean / `=` term | `FILTER(?flag = true)` | none yet | null | **gap** (boolean VALUE_HOOK proposed §2.5) |
| Q08 | FILTER string ops (`STR`, `REGEX`, `CONTAINS`) | `FILTER(CONTAINS(STR(?x),"a"))` | none | null | **gap** (string lane only; no in-circuit string predicate) |
| Q09 | FILTER `xsd:dateTime` compare | `FILTER(?d >= "2020-01-01T00:00:00Z"^^xsd:dateTime)` | none | null | **gap** (dateTime VALUE_HOOK design-only, §2.5) |
| Q10 | OPTIONAL | `OPTIONAL { ?s :q ?v }` | scan + verifier-side left-join | partial | **partial** (left-join is verifier-side, not a circuit) |
| Q11 | UNION | `{…} UNION {…}` | per-branch `scan_*`, union verifier-side | partial | **partial** |
| Q12 | JOIN on shared variable (hidden) | `{ ?p :worksFor :ACME }{ ?p :hasSalary ?s }` | `join_eq_na{16,64}_nb{16,64}` | 7,025–18,681 | covered |
| Q13 | Property path `/` (sequence) | `?s :p/:q ?o` | desugars to multi-pattern BGP → `scan_*` | (as BGP) | covered (as BGP) |
| Q14 | Property path `^` (inverse) | `?s ^:p ?o` | swap-slot BGP → `scan_*` | (as BGP) | covered (as BGP) |
| Q15 | Property path `\|` (alternative) | `?s (:p\|:q) ?o` | desugars to UNION → per-branch `scan_*` | partial | partial (as UNION) |
| Q16 | Property path `?` (zero-or-one) | `?s :p? ?o` | none (needs optional/identity step) | null | **gap** |
| Q17 | Property path `+` (one-or-more) | `?s :p+ ?o` | none (unbounded transitive closure) | null | **gap** |
| Q18 | Property path `*` (zero-or-more) | `?s :p* ?o` | none (transitive closure + reflexive) | null | **gap** |
| Q19 | Aggregate / GROUP BY (`COUNT`, `SUM`) | `SELECT (COUNT(?x) …) GROUP BY ?g` | none | null | **gap** (the COUNT-forgery guard `scan.nr:119-124` is anti-forgery, not an aggregate proof) |
| Q20 | Subquery | `{ SELECT … WHERE {…} }` | composed sub-proofs (no dedicated member) | partial | **partial** |
| Q21 | BIND | `BIND(?a + ?b AS ?c)` | none | null | **gap** (no in-circuit expression eval) |
| Q22 | VALUES | `VALUES ?x { :a :b }` | constant-slot scan or verifier-side | partial | partial |
| Q23 | Negation `FILTER NOT EXISTS` / `MINUS` | `FILTER NOT EXISTS {…}` | none | null | **gap** (requires proven absence; scan completeness `scan.nr:158-175` is the closest primitive) |
| Q24 | Revocation / status (hidden index) | credential liveness | `revoke_unset_d10` | 899 | covered |
| Q25 | Issuer attestation (hidden tier) | hidden-issuer Schnorr | `hidden_issuer_d4` | 16,932 | covered (scalar-mul target, §3.3 of the gate-count doc) |
| Q26 | Holder possession (hidden) | `holder_pok` / `holder_set_d4` | `holder_pok`, `holder_set_d4` | 10,334 / 10,650 | covered |

**Honesty note baked into the catalog:** the `gap` rows are the larger story — the
ZK engine proves a **fragment** (BGP scans, the numeric-FILTER family, hidden join,
revocation, issuer/holder), and general property-path traversal (`+`/`*`/`?`),
aggregates, expression `BIND`, string predicates, dateTime compare, and negation
have **NO circuit yet**. The benchmark must surface these as gaps, not omit them —
that is what makes it a coverage map rather than a cherry-picked win list.

### 6.3 Canonical JSON shape (driven off the existing snapshot, can't drift)

Mirror `bench/zk-compose/gate_counts_latest.json` conventions so the two cannot
diverge: top-level `tool: "bb gates -s ultra_honk"`, `bb_version`,
`nargo_version`, and a per-entry `circuit_size` for covered members. Proposed
shape (`bench/zk-compose/sparql_feature_catalog.json`):

```json
{
  "tool": "bb gates -s ultra_honk",
  "bb_version": "5.0.0-nightly.20260324",
  "nargo_version": "nargo version = 1.0.0-beta.21",
  "queries": {
    "Q03_filter_integer_ge": {
      "feature": "FILTER xsd:integer",
      "sparql": "SELECT ?p WHERE { ?p :age ?age FILTER(?age >= 18) }",
      "zk_members": ["filter_int_d2"],
      "circuit_size": 17416,
      "flag": "HIGH_GATE_blake3_binding",
      "reduction_target": "value-hook (research/zk-field-native-encoding.md §2.6)",
      "projected_after": "ESTIMATE ~3200 — MUST be re-measured with bb gates"
    },
    "Q17_path_one_or_more": {
      "feature": "property path +",
      "sparql": "SELECT ?o WHERE { :s :p+ ?o }",
      "zk_members": [],
      "circuit_size": null,
      "flag": "NO_ZK_CIRCUIT_YET",
      "reduction_target": null
    }
  }
}
```

**Anti-drift rule (load-bearing):** the `circuit_size` for every covered query
entry MUST be **derived from** `gate_count_snapshot.json` (the regression-gated
source of truth) at harness build/CI time — the catalog stores the *member name*
and the harness joins the live `circuit_size` — so a member's gate count is never
hand-copied into two files that can disagree. The `gate_count.rs` regression test
already fails on silent member growth (`tests/gate_count.rs`); the catalog harness
should additionally fail if a `zk_members` entry names a member absent from the
snapshot, and if a `circuit_size` literal in the catalog disagrees with the
snapshot.

**Flagging high-gate queries:** any covered entry whose joined `circuit_size`
exceeds a threshold (e.g. the blake3-bound 17,416 members) is auto-flagged
`HIGH_GATE_*` with its `reduction_target`. After the §2.6 value-hook lands and is
re-measured, the harness records both the before (17,416) and the re-measured
after (the real `bb gates` figure, NOT the ~3,200 estimate) so the reduction is
visible and verified, never asserted.

### 6.4 Site surface (follow-on bead, NOT in this design)

A `/benchmarks` or `/papers` gate-cost table reading the canonical JSON
(`sparql_feature_catalog.json`), surfacing the coverage map + per-feature gate cost
+ the gap list, mirrors the existing paper-factory pattern that reads
`bench/`-canonical JSON through accessors (so the perf-number gate's accessor-aware
scan applies and no number is hand-typed into prose). The site work is explicitly
**left to a follow-on bead** (§7); this record specifies only the JSON the site
would read.

## 7. Beads (orchestrator to create)

Recommended children of epic **sq-1s2** (this record creates none; the
orchestrator owns bead structure). Existing related beads to link, not duplicate:
**sq-j506** (numeric lane in `encode`/`commit`), **sq-hb75** (scalar-mul
projective coords), **sq-mslu** (`xsd:double` RNE parser — likely refined by §2.5).

1. **Encoding overhaul (host).** Re-base `crates/sparq-zk/src/encode.rs` /
   `commit.rs` onto the value-first order `Poseidon2([value, termType])` + the
   inner/outer literal scheme with per-datatype `VALUE_HOOK` (integer / signed /
   decimal-scaled / IEEE-bits / boolean; string-lane fallback for opaque/string).
   Add the ingest-side canonical-lexical normalisation for hooked datatypes (§4
   resolution (a)). One-time recommit of all leaf/commitment vectors (§2.2).
   Extend/relink sq-j506. **Audit-gated (sq-qhy4).**
2. **Circuit + verifier co-change.** Add value-hook FILTER member(s)
   (`filter_int`/`signed`/`decimal`/`float`) that bind via the 2-Poseidon2
   constant-datatype path instead of in-circuit blake3; update `scan.nr` leaf
   encoding; update `reconstruct_public_inputs` + `derive_id`/`canonical_vk` + the
   real-bb cross-vectors in lockstep (C1/C2); re-measure with `bb gates` and
   re-baseline `gate_count_snapshot.json` + `gate_counts_latest.json`. Verify the
   ~3,200 estimate against reality. **Audit-gated (sq-qhy4).**
3. **Audit-obligation registration.** Land the CR-G8 gap-register row (§5.1) and
   the SKILL.md + sparq-zk README caveats (§5.2/§5.3) so sq-qhy4 is forced to check
   the relocation, the canonical-form carry, and the value-collapse resolution.
   Doc-only; can land **before** 1/2 to register the obligation up front.
4. **SPARQL gate-benchmark harness.** Implement the §6 catalog + JSON, driven off
   `gate_count_snapshot.json` (anti-drift), with the `gap` rows honestly labelled
   and the high-gate flagging + before/after recording.
5. **Site surface (gate-cost / coverage table).** A `/benchmarks` or `/papers`
   table reading `sparql_feature_catalog.json` through accessors. Follow-on; depends
   on 4.

Suggested ordering / deps: **3** (register first) → **1** → **2** (depends on 1) →
**4** (can start in parallel after 2 lands the first re-measured member) → **5**
(depends on 4). 1 and 2 are audit-gated behind sq-qhy4 for any production reliance;
they may be implemented at research grade (opt-in, NOT-yet-sound) before sign-off,
consistent with the rest of the ZK estate.

## 8. Open questions (need the maintainer)

1. **dateTime/date VALUE_HOOK:** canonical epoch encoding vs canonical component
   tuple, and the timezone-normalisation rule (normalise all to UTC at ingest?).
   Deferred from the first slice — confirm the preferred shape before sq-j506's
   dateTime extension.
2. **Value-collapse resolution depth:** is (a) canonical-on-ingest **alone**
   acceptable for sparq-originated commitments (simpler, full gate win on one leaf),
   or is the (a)+(b) dual-leaf defence-in-depth required for externally-produced
   commitments? My recommendation is (a)+(b); confirm the cost/complexity trade is
   wanted.
3. **Leaf-order flip migration:** confirm the one-time recommit of all persisted
   `<urn:sparq:zk>` commitments + vectors is acceptable now (research grade), vs
   keeping type-first order and treating the maintainer's order purely as a
   documented convention divergence. Recommendation: adopt his order (§2.2).
4. **Double bits vs sq-mslu parser:** confirm against `filter_float.nr` /
   `sparq_ieee754` that a bit-pattern VALUE_HOOK fully obsoletes the in-circuit
   RNE parser for the *comparison* (the ingest-side canonical-bits rule still
   exists) before closing/superseding sq-mslu.

## 9. Verdict (the one the brief asks for)

**The value-first VALUE_HOOK encoding is worth pursuing at research grade:** it
gives a large, anchor-bracketed gate reduction (17,416 → estimated ~3,200, to be
re-measured) on the numeric-FILTER family by replacing one in-circuit blake3 with
two constant-datatype Poseidon2 permutations, and it carries every §3.4 must-keep
onto the value lane by relocation. **The one obvious soundness issue I can identify
and reason about is value-collapse** (§4): a value hook silently turns term
identity into value identity for hooked datatypes, which is wrong for
scan-row-equality / `join_eq` / `DISTINCT` / `sameTerm`, not just for the
value-FILTER it was designed for. **My recommended resolution is (a)
canonicalise literals to their canonical lexical form on ingest, backed by (b)
keeping the string lane as the authoritative identity leaf and consulting the value
hook only in value-semantic FILTER comparisons** — never in an identity-sensitive
operator. This resolution, the operand-binding relocation, and the canonical-form
carry are **registered as explicit external-audit obligations (CR-G8 / sq-qhy4)**;
none is presented as a settled property, and no soundness or privacy guarantee is
claimed pending the external sign-off.
