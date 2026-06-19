<!-- [OPUS-4.8] Design-for-review authored by Opus 4.8 (1M context) (Fable unavailable) — re-review when Fable returns. -->
# Field-native ZK term encoding: the FINAL dual-leaf scheme (value handle AND lexical-identity hash)

Maintainer-review design record. This is the **finalized** encoding design for the
maintainer's `#769` decision, which selected the **dual-leaf** option (value +
lexical) over canonical-on-ingest alone. It **supersedes** the open-question
value-only `VALUE_HOOK` draft (PR #765) — that draft proposed a *single* value-first
leaf and left the dual-leaf question open (#765 §8 Q2); this record closes it with
the decided dual-component leaf and folds in the four adversarial review verdicts
(issuer-desync, term-identity, must-keep survival, gate-reality) that were run
against the draft. His verbatim direction (#769):

> "I guess we need both the value and the hash of the lexical representation then.
> This does create the risk that a malicious issuer could provide a value that does
> not conform to the lexical representation; but I don't know whether there are
> really any attacks that can be done based on that. Could you do the version where
> we include both so we can optimise using the value provided whilst also having the
> lexical representation for sameTerm type queries — and note the performance risk.
> It may be a good reason to force issuers to only be allowed to issue values in a
> canonical form in the long term."

This record builds directly on the gate-count attribution and the **§3.4 must-keep
constraint set** in [`research/zk-age-gatecount-reduction.md`](./zk-age-gatecount-reduction.md)
(read that first). Companion analyses:
[`research/zk-soundness-audit.md`](./zk-soundness-audit.md),
[`research/zk-verifier-reaudit.md`](./zk-verifier-reaudit.md),
[`research/zk-hidden-join-design.md`](./zk-hidden-join-design.md). The adversarial
issuer-desync review that drove §5's correction is in
[`research/zk-dual-leaf-issuer-desync-review.md`](./zk-dual-leaf-issuer-desync-review.md)
(PR #793).

Parent: epic **sq-1s2** ("ZK query-proof build-out + in-circuit privacy upgrades").
This is a **design-for-review**: it changes no `.nr` / `.rs` source, creates no bead
(the orchestrator owns bead structure; recommended children are in §11).

## 1. Honesty framing (load-bearing — read first)

sparq's v1 ZK query-proof verifier (`sparq-zk` / `sparq-zk-compose`) is
**remediated and internally re-audited but NOT externally audited**, and is
documented **NOT-yet-sound** for production reliance (beads **sq-qhy4**, **sq-9hrn**,
**sq-1s2**; `SECURITY.md`; `compliance/cryptoreview/gap-register.md` headline CR-G1).
An external accredited-cryptographer sign-off (**sq-qhy4**, P0) is **REQUIRED before
any ZK soundness / privacy / integrity property may be relied upon in production**.
The MPC estate is semi-honest-only and is not invoked here.

Nothing in this record is a security guarantee. Where this design says a constraint
is "preserved", that means the exact equalities, range-binds and canonical-form
asserts the current circuit makes are **relocated, not removed**, and their
preservation is itself an **audit obligation** — never an established fact. A
relocation is "safe to propose" only if **every** §3.4 must-keep demonstrably
survives with the cited constraint intact (not merely "equivalent"); even then the
framing stays "preserves the constraint set, pending sq-qhy4".

**The central honesty correction this record makes (driven by the adversarial
issuer-desync verdict, §5).** The value-only draft and an earlier dual-leaf draft
framed value↔lexical desync as "within the existing honest-issuer boundary — the
issuer could already lie about the value." **That framing is WRONG against the actual
code and is corrected here.** The current `filter_int` / `filter_signed` /
`filter_decimal` / `filter_float` members enforce, **in-circuit, against arbitrary
committers including a malicious trusted issuer**, the invariant that the compared
value IS the parse of the committed lexical bytes (call it **INV-VL**) — because the
value and the leaf binding are both derived from **one** witnessed digit/byte set
(verified, §2). The dual leaf witnesses the value handle and the lexical hash
**independently**, so it **REMOVES INV-VL**. That is a **trust-model regression for
the value-FILTER lane** (from machine-enforced to issuer-honesty-trusted), a NEW
capability — not the equivalent of a value lie that keeps value and lexical in
agreement. §5 states this plainly; it is not hand-waved away.

The privacy-claims CI gate (`scripts/check-privacy-claims.sh`) path-excludes
`research/**` (it defends the *outward* claim surface, not design records). The
caveat wording proposed for the ZK `SKILL.md`, the `sparq-zk` README, and the
`gap-register.md` CR-G8 row in §9 — which **is** on the scanned surface — is written
negated / obligation-framed (and inline-marked where the gate's predicate regex
over-matches) so it passes the live gate.

All gate counts here are the **measured `bb gates -s ultra_honk` `circuit_size`**
already snapshotted and regression-gated at
`crates/sparq-zk-compose/tests/gate_count_snapshot.json` — a circuit metric, not a
performance-marketing number. Toolchain: `nargo 1.0.0-beta.21`,
`bb 5.0.0-nightly.20260324` (the snapshot's baselined toolchain). Every *projected*
post-change figure is an **estimate bracketed by measured anchors** and is NOT a
claim until re-measured with `bb gates` on the actual changed member. All EC2 /
work-box timings are NON-canonical and none appear here.

## 2. What the leaf is today, what INV-VL is, and what identity ops consult (verified)

The host commitment path is `crates/sparq-zk/src/encode.rs`; the in-circuit mirror is
`zk/compose/compose_core/src/hashes.nr`. The current literal leaf is
(`encode.rs:52-55`):

```text
Enc_literal = h2(TYPE_CODE_LITERAL, blake3_field(literal.to_string()))
            = Poseidon2([2, field_from_hash_bytes(blake3("<lexical>"^^<dt> | @lang))], 2)
```

`oxrdf`'s `Literal::to_string()` passes the lexical form through **verbatim** — it
does not re-canonicalise — so the hashed token carries the **exact ingested bytes**
(`filter_signed.nr:9-12` confirms). `field_from_hash_bytes` (`field.rs`) truncates the
32-byte digest to its low 31 bytes (248 bits, bias-free). `TYPE_CODE_{IRI,LITERAL,
BLANK_NODE} = {1,2,3}` exist in both `encode.rs:33-35` and `hashes.nr:35-37`.

### 2.1 INV-VL: the value↔lexical invariant the current circuit enforces in-circuit

**This is the load-bearing fact the adversarial issuer-desync verdict surfaced, and
it corrects the earlier framing.** In every value-FILTER member today, the compared
numeric value AND the operand binding are derived from **the SAME witnessed digit
array** — there is exactly **one preimage**:

- `filter_int.nr:67-70` builds `value` by accumulating `digits[i]` into a `u64`;
  `filter_int.nr:72-92` rebuilds the canonical N-Triples token **from the same
  `digits`**, blake3-hashes it, and asserts `h2(LITERAL, hs) == operand_enc`. So a
  prover who commits lexical `"5"` literally **cannot** make the FILTER compare it as
  `18`: the circuit re-parses the digits `"5"` for both the value and the binding.
- `filter_signed.nr:150-180` (signed integer) and `filter_signed.nr:229-275`
  (decimal) do the same: `mag` / `mag_scaled` and the rebuilt token both consume the
  same `mag_digits` / `int_digits` / `frac_digits`.
- `filter_float.nr` derives the IEEE bits from witnessed digits the same way, then
  binds the rebuilt token.

Call this invariant **INV-VL**: *the compared value equals the parse of the committed
lexical bytes, enforced in-circuit against an arbitrary (even malicious, even trusted)
committer.* No value↔lexical desync state is reachable today. This is a **prover-side
circuit guarantee independent of issuer honesty** — it is the property the dual leaf
must be honest about (§5).

### 2.2 What identity ops consult (verified) — every one compares the FULL leaf

*Every identity-sensitive in-circuit operation already compares the full leaf,
because the leaf is the only per-term value the circuit holds.*

- **scan row-presence / attribution** (`scan.nr:143-149`, `:164-175`): equality is
  `enc[g][i][s] == rows[j][s]` and `enc[g][i][s] == pattern.const_enc[s]` — over the
  full per-slot term encoding `Enc_t`. The disclosed `rows` (`scan.nr:88`) are PUBLIC
  inputs (so the lexical side can be consumed by any downstream / non-ZK consumer —
  this matters in §5.2).
- **hidden join** (`join.nr:170-177`): `a_val = select_slot(row_a, slot_a)` over a row
  of `enc_a`, then `assert(a_val == b_val)`; the value is bound under a blinder into
  `join_commitment` (`join.nr:179+`). The equality is over the **full leaf** at the
  join slot, and is **cross-credential** (graph A vs graph B). `DISTINCT` / `sameTerm`
  over a ZK column, when added, reduce to the same full-leaf equality.
- **value FILTER** (`filter_int.nr:92`, `filter_signed.nr:114`, `filter_float.nr`):
  the *only* place that looks *inside* the leaf — it re-derives the value from the
  lexical bytes and re-hashes (the in-circuit blake3 the value handle removes).

So the design principle: **put the lexical-identity hash where the full-leaf equality
already lands (identity ops correct by construction, no change), and put the value
handle inside the same leaf as a cheap additional component (so the FILTER member can
bind the value with Poseidon2 instead of blake3).** But — per §2.1 — the value handle
and the lexical hash being *independent witnesses* is exactly what removes INV-VL, so
§5 and §4 must put back, or honestly account for, what the single-preimage construction
gave for free.

## 3. The dual-leaf encoding (mapped to `Poseidon2` / the actual code)

### 3.1 The leaf

```text
Enc_literal = Poseidon2([ value_component, lexical_component, TYPE_CODE_LITERAL ], 3)

  value_component   = Poseidon2([ VALUE_HOOK, DATATYPE_CONST, LANG_CONST ], 3)
                      // the cheap numeric handle (§3.3); a per-datatype field value
                      // + PRECOMPUTED datatype/lang constants for the known FILTER
                      // datatypes — this is what lets the FILTER avoid in-circuit blake3.

  lexical_component = field_from_hash(blake3(canonical N-Triples token))
                      // the identity-bearing part; computed OFF-circuit at ingest,
                      // EXACTLY today's blake3_field(literal.to_string()).
```

`Poseidon2::hash([·,·,·], 3)` is the existing `h3` arity (`hashes.nr:25` — already used
for triple leaves and `sig.rs` 3-input commitments), so no new permutation width is
introduced. The full leaf is one outer Poseidon2 over three fields; the inner
`value_component` is one more Poseidon2 over three fields.

This is a structural generalisation of today's leaf: `lexical_component` **is** today's
`blake3_field(to_string())` (byte-for-byte), so the identity content is unchanged; the
leaf gains a `value_component` sibling and moves `TYPE_CODE_LITERAL` to the
value-first / type-last slot (matching the maintainer's `hashFields([…, termType])`
convention).

### 3.2 Non-literals and string / opaque literals — value_component degenerates

For terms with **no numeric handle**, `value_component` collapses to a fixed
no-value sentinel and the **lexical_component is the sole binding** — in-circuit cost
UNCHANGED from today:

| Term | Leaf | Cost vs today |
| --- | --- | --- |
| IRI | `Poseidon2([NO_VALUE, lexical, TYPE_CODE_IRI], 3)`, `lexical = blake3(iri)` | one extra Poseidon2 layer at commit only; in-circuit unchanged |
| Blank node | `Poseidon2([NO_VALUE, lexical, TYPE_CODE_BLANK_NODE], 3)`, `lexical = Poseidon2([salt_G, blake3(label)], 2)` | salt-scoped inner retained (Q6); one extra layer at commit only |
| `xsd:string`, `rdf:langString`, opaque datatype | `Poseidon2([NO_VALUE, lexical, TYPE_CODE_LITERAL], 3)`, `lexical = blake3("<lexical>"^^<dt> or @lang)` | **string lane, no numeric handle, in-circuit cost UNCHANGED** |

`NO_VALUE` recommendation: a datatype-folded `value_component = Poseidon2([VALUE_NONE,
DATATYPE_CONST, LANG_CONST], 3)` with `VALUE_NONE` a reserved field tag distinct from a
real `VALUE_HOOK = 0`, so a degenerate value_component can never be confused with a real
zero value and `literal_shapes_are_distinguished` (`encode.rs:104-117`) stays true
without leaning on the lexical lane alone. (§10 Q1 confirms this against a single global
`NO_VALUE` alternative.)

### 3.3 VALUE_HOOK per datatype

The handle is **injective on value within a datatype**, range-bound, and is what a
FILTER compares against in-circuit without re-hashing a string. **Note (term-identity
verdict, §5.5): for double/float and decimal the handle is MANY-TO-ONE on the term** —
flagged below and is why reject-list (v) plus the expanded regression guard (§8) are
load-bearing.

| Datatype | VALUE_HOOK | Many-to-one on term? | Canonical-form obligation (B4) |
| --- | --- | --- | --- |
| `xsd:boolean` | `0` / `1` | yes (`"true"`/`"1"` → same) | injective on the two values |
| `xsd:integer` (incl. signed) | signed value in the `u64` magnitude + sign domain `filter_signed.nr` uses (NOT raw wrapping `Field`) | yes (`"05"`/`"5"` → same) | no leading zeros, no `-0` (`filter_int.nr:58-64`, `filter_signed.nr:142-158`) |
| `xsd:decimal` | canonical scaled integer at the member-fixed `FD` scale (matching `filter_decimal_check`) | **yes — `"5.0"`/`"5.00"` collide at a fixed `FD`** | `filter_decimal` digit-canonicality asserts (`filter_signed.nr:215-246`) |
| `xsd:double` / `xsd:float` | IEEE-754 bit pattern as a field | **yes — `-0.0`/`+0.0` compare EQUAL (`tests.nr:379-380`); NaN payloads** | canonical IEEE bits at ingest (may obsolete sq-mslu's in-circuit RNE parser for the *comparison* — §10 Q3) |
| `xsd:dateTime` / `xsd:date` | canonical epoch / component encoding | design-dependent | timezone-normalisation rule open (§10 Q2); NOT in the first slice |
| `xsd:string`, `rdf:langString`, opaque | `VALUE_NONE` (no handle) | — | the §3.2 fallback; lexical lane only |

`DATATYPE_CONST` / `LANG_CONST` are `blake3(datatype IRI)` / `blake3(lang)` off-circuit
at ingest, and **precomputed field constants** in-circuit for the known FILTER
datatypes — the substitution that removes the in-circuit blake3 (§4).

### 3.4 Why the lexical_component cannot be dropped

The value-only draft (#765) committed a single value-first leaf and had to defend
value-collapse (`"05"` vs `"5"` → same leaf) for every operator. The maintainer rejected
canonicalise-on-ingest alone (#769) and chose to carry the lexical hash too, because:

- The engine receives **pre-committed** graphs from external issuers it does not
  canonicalise at ingest. A value-only leaf would silently make
  `join`/`DISTINCT`/`sameTerm` use **value identity** for those — wrong per RDF
  semantics.
- The lexical_component preserves **term identity** for every identity op with **zero
  mechanism change** to scan/join (those ops already compare the full leaf, §2.2, which
  now contains the lexical hash), **conditionally** on reject-list (v) being enforced so
  no identity op ever reads `value_component` (the many-to-one hazard of §3.3 / §5.5).

### 3.5 Leaf-tuple order and the one-time recommit (inherited migration)

Adopting `Poseidon2([value_component, lexical_component, TYPE_CODE], 3)` is value-first
(matches the maintainer's spec) and **re-bases every leaf**: the host
(`encode.rs`/`commit.rs`), the circuit (`hashes.nr` leaf recompute), every checked-in
leaf/commitment vector, every real-bb cross-vector, the `gate_count_snapshot.json`
baselines, and any persisted `<urn:sparq:zk>` commitment must be **recomputed and
re-committed atomically** in the same change, or the verifier byte-compare
(`verifier.rs` `PublicInputMismatch`) diverges silently. Same one-time migration #765
flagged; unchanged here (§10 Q4).

## 4. The gate win — proving a value-FILTER against `value_component`, AND putting back B1/B4

This is the (a) part of the brief. **The gate-reality adversarial verdict CONFIRMED
the win is real** (no in-circuit blake3 hidden in the dual-leaf reconstruction; the
only production in-circuit `std::hash::blake3` call sites are the three FILTER members,
`filter_int.nr:84` / `filter_signed.nr:107` / `filter_float.nr:137`, and scan/join have
**zero**). **But the must-keep adversarial verdict found the §4 pseudocode of an earlier
draft SILENTLY DROPPED the B1/B4 mechanism** — it witnessed VALUE_HOOK as a bare `Field`
and only asserted the leaf binding, with **no range-decomposition and no
canonical-digit asserts**. That is fixed here: the FILTER member MUST *instantiate*, not
merely name, B1 and B4.

```text
// witnesses: VALUE_HOOK (a Field), lexical_component (a Field).
// ---- B1: RANGE-DECOMPOSE the witnessed VALUE_HOOK into the typed comparison domain
//      BEFORE any comparison. A Field witness is NOT a u64; without this a prover can
//      supply VALUE_HOOK congruent to a small value mod the BN254 modulus (modular wrap,
//      reject-list (i)).
//   integer/signed : prove VALUE_HOOK's magnitude < 2^64 (explicit bit/byte decomposition),
//                    sign as a constrained bool; compare in the u64 + sign domain.
//   decimal        : prove scaled magnitude = int_part*10^FD + frac_part with ID+FD<=19
//                    fits u64; compare in the scaled-int + sign domain.
//   double/float   : prove the 64/32-bit IEEE pattern is well-formed; compare via
//                    sparq_ieee754 f64/f32 predicates (filter_f64 shape).
// ---- B4: CANONICAL-FORM bind on the range-decomposed value. Because the digit-string
//      asserts (no leading zero, no -0, canonical scale) cannot exist in a digit-free
//      member, the member MUST EITHER (i) re-introduce a constrained canonicalising
//      re-derivation of VALUE_HOOK from a canonical witness in-circuit (costs gates,
//      must be re-measured), OR (ii) the design must HONESTLY RE-CLASSIFY B4 (and the
//      no-modular-wrap half of B1 not covered by the range-decomposition) to an
//      issuer/ingest-side assumption — a STRICTLY LARGER trust escalation than §5's
//      value↔lexical point, because in-range-ness and single-encoding are TODAY
//      prover-side circuit guarantees independent of issuer honesty. §5.4 records which.
inner = Poseidon2([VALUE_HOOK, DATATYPE_CONST, LANG_CONST], 3)            // 1 perm
leaf  = Poseidon2([inner, lexical_component_witness, TYPE_CODE_LITERAL], 3) // 1 perm
assert_eq(leaf, operand_enc)                                             // the binding
// then: the typed comparison (B2) and assert verdict == expected (B3) — filter_f64 shape.
```

`lexical_component` is supplied to the FILTER member as a **witness** (the
scan-anchored leaf's other component; the FILTER does not need the lexical bytes, only
the field value of its hash, which is enough to reconstruct the full leaf and assert the
binding). The FILTER member **never reconstructs or hashes the lexical string** — that
is the saving — **but it does add the explicit VALUE_HOOK range-decomposition (B1) that
the earlier draft omitted**, whose cost must be measured (§10 Q5; the
noir-optimisation standing warning, PR #37 / `shr_sticky`, applies — inversion/
decomposition surprises intuition on this stack).

Cost accounting against the measured anchors (from `gate_count_snapshot.json`,
reproduced in `zk-age-gatecount-reduction.md` §6):

| Anchor | `circuit_size` | What it isolates |
| --- | ---: | --- |
| `filter_f64` (raw compare, no binding) | 3,113 | comparison verdict + UltraHonk/range-table floor |
| blake3-over-48B probe | 17,416 | the in-circuit string-hash binding **alone** |
| `filter_int_d{1..4}` / `filter_signed_int_d{2,4}` / `filter_decimal_i3_f2` / `filter_f64_d{1..4}` | 17,416 | the full blake3-bound FILTER members |

The blake3 binding (~14,300 gates = 17,416 − 3,113) is replaced by two Poseidon2 perms
(~74 gates each per the `hashes.nr:14` cost note ≈ ~150 gates) **plus** the VALUE_HOOK
range-decomposition and the typed re-derivation.

> **Projected dual-leaf numeric-FILTER member ≈ 3,200 gates** (3,113 comparison floor +
> ~150 Poseidon2 binding + the range-decomposition cost, which must NOT be assumed
> negligible). **This is an ESTIMATE bracketed by the two measured anchors above. It is
> NOT a claim.** It MUST be confirmed with `bb gates -s ultra_honk` on the actual
> dual-leaf FILTER member — *including* the B1 range-decomposition and any B4
> canonicalising re-derivation — and re-baselined into
> `crates/sparq-zk-compose/tests/gate_count_snapshot.json` (and
> `bench/zk-compose/gate_counts_latest.json`) before it is quoted anywhere. The net
> delta of moving the outer binding from arity-2 `h2` to arity-3 `h3` plus one inner
> perm (net +1 Poseidon2 perm over today's single binding perm) plus the range-check is
> a MEASUREMENT obligation, not an assumption.

## 5. The value↔lexical consistency analysis — corrected per the issuer-desync verdict

This is the crux (the maintainer's "malicious issuer could provide a value that does not
conform to the lexical representation" worry). The earlier dual-leaf framing was found
**FALSE against the code** by the issuer-desync verdict; the corrected, honest position
is below.

### 5.1 The structural fact (CONFIRMED by the verdict)

The dual leaf carries two independent components in the same leaf: `value_component`
(from VALUE_HOOK) and `lexical_component` (from blake3(lexical)). They are **independent
preimages**: `Poseidon2([value_component, lexical_component, TYPE_CODE])` binds *that the
leaf contains both*, but does **NOT** bind that `VALUE_HOOK == parse(lexical)`. To bind
them in-circuit, the circuit would have to re-derive the value from the lexical bytes —
witness the lexical string, parse it, assert it equals VALUE_HOOK — which is **exactly
the blake3-over-the-token + digit-parse the value handle exists to avoid** (~14,300
gates). The verdict CONFIRMED this structural claim is correct.

> **Value↔lexical consistency CANNOT be enforced in-circuit without giving back the gate
> win. The loss of INV-VL is therefore the IRREDUCIBLE PRICE of the gate win, not a
> negligible residual.**

### 5.2 The CORRECTED threat model — the dual leaf REMOVES the in-circuit invariant INV-VL

The earlier framing said desync is "not a new value-lie capability (the issuer could
already lie about the value)." **That is wrong, and the issuer-desync verdict refuted it
against the code:**

- **Today (§2.1), INV-VL holds in-circuit against ARBITRARY committers** — including a
  malicious trusted issuer. A malicious issuer who commits lexical `"5"` **cannot** make
  any FILTER compare it as `18`; the circuit re-parses `"5"` for both the value and the
  binding. A value lie today **keeps value and lexical in agreement** (the issuer must
  commit `"18"` lexically to get an `18` comparison, and then `sameTerm`/`join`/`DISTINCT`
  also see `18`).
- **Under the dual leaf, INV-VL is REMOVED.** A malicious trusted issuer can commit a
  leaf with `VALUE_HOOK = 18` but `lexical = "5"`, sign `C(G)`, and a holder proves
  `age ≥ 18` truthfully against `value_component` while the **same** credential answers a
  `sameTerm`/`DISTINCT`/`join` question as `"5"` via `lexical_component`. **That single
  signed credential answering a value question as 18 and an identity question as 5 is
  IMPOSSIBLE today.** It is a NEW capability, a strict trust-model regression for the
  value-FILTER lane: that lane goes from **machine-enforced** to **issuer-honesty-
  trusted**.
- **The exploit surface is wider than "one mixed query."** Disclosed scan rows are
  PUBLIC (`scan.nr:88`; `rows` are public inputs), so the lexical side can be consumed by
  any downstream / non-ZK consumer of the disclosed result, not just an in-proof
  identity op. And `join_eq` (`join.nr:170-177`, full-leaf equality) makes the
  inconsistency **cross-credential**: one malicious issuer's desynced leaf plus one
  honest issuer's `"5"` leaf can be joined on the lexical side while the malicious leaf
  passes a value-FILTER as `18`.

### 5.3 What still holds, honestly

- **No UNTRUSTED party gains anything.** The scan/issuer chain still binds the leaf to a
  trusted Schnorr signature over `C(G)` (`verifier.rs` `bind_issuer_attestations`;
  `issuer.nr`). An untrusted party cannot forge a desynced leaf; the attacker must **be
  or collude with a trusted issuer**.
- **No single-component operation weakens.** A pure value-FILTER over an honest issuer's
  leaf, or a pure identity op, is unaffected. The desync only bites where a value answer
  and an identity answer about the **same** committed term are both consumed.
- **An honest issuer never desyncs**, and §6's host-side same-leaf co-binding makes
  honest sparq ingest *structurally unable* to desync, turning desync into a detectable
  protocol violation for sparq-originated commitments.

### 5.4 Two trust escalations, named separately (do not conflate)

The dual leaf escalates trust in **two** distinct ways, both of which the docs must
state honestly:

1. **Value↔lexical agreement** (§5.2) — removed INV-VL; rests on issuer honesty for the
   value lane.
2. **In-range-ness and single-encoding** (the must-keep verdict, §4) — if the §4 member
   does **not** instantiate the B1 range-decomposition and a B4 canonicalising
   re-derivation in-circuit, then in-range-ness (no modular wrap) and single-encoding (no
   second encoding of a value) **also** migrate from prover-side circuit guarantees to
   issuer/ingest assumptions. This is a **strictly larger** escalation than (1). **The
   design's position: instantiate B1 in-circuit unconditionally (it is cheap relative to
   the comparison floor and a hard reject-list (i) requirement), and instantiate B4
   in-circuit if measurement allows; only if B4's in-circuit re-derivation proves too
   costly may it be downgraded — and then the docs/SKILL/README MUST say in-range single-
   encoding rests on issuer honesty too.** This choice is itself a sq-qhy4 audit
   obligation (§9).

### 5.5 Term-identity is CONDITIONAL, not a property of the encoding (term-identity verdict)

The term-identity verdict found that "preserves term identity end-to-end" rests on an
**unenforced convention** (reject-list (v): "value_component MUST NOT be consulted by any
identity op"), not a circuit invariant — and that VALUE_HOOK is **many-to-one on the
term** for two named datatypes (§3.3): IEEE `-0.0`/`+0.0` compare EQUAL
(`tests.nr:379-380`), NaN has many comparison-unordered patterns, and decimal `"5.0"`/
`"5.00"` collide at a fixed `FD`. So `value_component` is affirmatively a term-identity
**HAZARD**, not a neutral sibling. Term identity is preserved **only because every
current identity op reads the full leaf** (which today carries the lexical hash) — **not
as a property of the encoding**. The moment a DISTINCT/sameTerm/value-keyed member is
added (the design says they are coming), nothing *structural* stops it consulting
`value_component`. Fix: §8 promotes reject-list (v) to a structurally-enforced invariant
and §8/§11-bead-4 expands the regression guard to the many-to-one datatypes.

### 5.6 The long-term close — canonical issuance is a NAMED PRECONDITION, not just a future option

The maintainer's own suggestion — *force issuers to issue only canonical-form values* —
is, per the issuer-desync verdict, **elevated from a documented future option to a NAMED
PRECONDITION for relying on the value lane in any adversarial-issuer setting.** If an
issuer is contractually/technically required to emit canonical lexical forms AND
`VALUE_HOOK = parse(canonical_lexical)`, then for that issuer class INV-VL is restored as
an issuance invariant, value identity ≡ term identity for hooked datatypes, and the dual
leaf could collapse back to a value-first single leaf (§7). Until such a conformance
mechanism exists and is relied upon, **the value-FILTER lane over an adversarial issuer
is sound only under the explicit honest-issuer-for-value assumption named in §5.2.**

## 6. The host-side same-leaf co-binding at ingest (issuer-desync fix)

So that **honest sparq ingest never desyncs** and desync becomes a **detectable protocol
violation** for sparq-originated commitments, `encode.rs` / `commit.rs` MUST compute
`VALUE_HOOK = parse(canonical(lexical))` from the **same bytes** that are hashed into
`lexical_component`, and **fail closed** if the lexical form does not parse to a
canonical value of its datatype:

- For a hookable datatype, ingest parses the lexical bytes once, derives both the
  canonical `VALUE_HOOK` and the `lexical_component` blake3 from the same parse, and
  commits the dual leaf only if the parse succeeds and the lexical form is canonical
  (or it canonicalises and records that it did). A non-parseable / non-canonical lexical
  for a hookable datatype is a fail-closed ingest error, not a silent desync.
- This does **not** prevent a *malicious external issuer* from committing a desynced leaf
  off-sparq and signing it (only canonical-issuance conformance, §5.6/§7, closes that) —
  but it guarantees sparq's own commitments are INV-VL-consistent and makes any desynced
  leaf a deviation from sparq's published ingest behaviour.

This is off-circuit Rust; it is not a gate cost, but it is a real ingest cost and must be
measured, not assumed negligible (§10 Q5).

## 7. Performance honesty — dual-leaf is MORE expensive than value-only

Per the maintainer's "note the performance risk":

- **Commit time (host).** The dual leaf is two Poseidon2 perms per literal (inner
  value_component + outer leaf) **plus** the blake3 over the lexical token, **plus** the
  §6 ingest parse + fail-closed canonical check — versus today's one blake3 + one `h2`.
  For IRIs/strings it adds one extra Poseidon2 layer over today. Off-circuit, but real;
  must be measured.
- **In-circuit (gates).** The reduction applies to **value-comparison FILTER members
  ONLY**: 17,416 → est. ~3,200, to be re-measured **including the B1 range-
  decomposition**. It does **NOT** apply to scan / join / revoke / issuer / holder
  (one `h3` per leaf, unchanged), to identity ops (full-leaf equality — the dual leaf
  buys them **correctness**, conditional on §5.5, not speed), or to IRI/string/opaque
  FILTERs (string lane unchanged).
- **Net.** Same FILTER gate win as value-only, at MORE commit-time cost, while carrying
  the lexical hash forever, **and** at the price of the removed INV-VL on the value lane
  (§5.2). The maintainer accepted the commit-time trade explicitly (#769); §5.6's
  canonical issuance is the path to retire both the carried lexical hash and the INV-VL
  regression.

## 8. §3.4 must-keeps carried onto the DUAL leaf, with the verdict fixes folded in

Each `zk-age-gatecount-reduction.md` §3.4 must-keep, restated for the dual leaf.
File:line citations verified against the current checkout. None is asserted as proven.

**A. Operand bound to the signed/committed credential (not attacker-chosen).**

- **A1 / A4 — operand binding by RELOCATION.** Today `filter_int.nr:92` /
  `filter_signed.nr:114` assert `h2(LITERAL, blake3(token)) == operand_enc`. Under the
  dual leaf: `assert_eq(Poseidon2([Poseidon2([VALUE_HOOK, DT_CONST, LANG_CONST], 3),
  lexical_component_witness, TYPE_CODE_LITERAL], 3), operand_enc)`. The equality assert
  is kept. **CORRECTION (must-keep verdict):** A1 binds VALUE_HOOK as a **field element**,
  not as a range-bound `u64`; A1's survival is real **only if the SAME range-decomposed
  VALUE_HOOK feeds both the binding and the typed comparison** (§4). Without the §4 B1
  decomposition, A1 binds "the same field" but not "the same in-range value the
  comparison uses." **AUDIT OBLIGATION.**
- **A2 — scan anchors the new DUAL leaf.** `verifier.rs` (`scanned != operand` ⇒
  `BindingInconsistent`); `scan.nr:97-108` commit-recompute now carries the dual leaf.
  Mechanism unchanged; what flows through it changes. Must not weaken.
- **A3–A7 — scan binds witnessed graph to public commitment.** `scan.nr:97-108`
  `commit_fold == commitments[g]`; `scan.nr:119-124` strictly-increasing commitments
  (anti duplicate-inclusion / COUNT-forgery); row-present `scan.nr:139-149`;
  completeness + attribution `scan.nr:158-182`. Survives unchanged except the hashed leaf
  is the dual leaf.
- **A8 / A9 — issuer attestation / in-circuit Schnorr.** `verifier.rs`
  `bind_issuer_attestations`; `issuer.nr`. **Untouched** — and (per §5) exactly what makes
  the removed-INV-VL a matter of issuer trust, since the malicious-desync attacker must be
  a trusted issuer.

**B. Comparison `value ⋈ bound` correct over the field — NO modular wrap.**

- **B1 — range-checked integer domain, no signed-Field wrap.** Today the magnitude
  accumulates into `u64` from range-checked ASCII digits (`filter_int.nr:58-70`,
  `filter_signed.nr:142-153`, `filter_signed.nr:229-242`). **CORRECTION (must-keep
  verdict):** the §4 member has no digit arrays, so it MUST add an **explicit in-circuit
  range-decomposition** of the witnessed VALUE_HOOK proving it lies in the typed domain
  (magnitude `< 2^64`; scaled magnitude `ID+FD<=19`; well-formed IEEE pattern) **before**
  the comparison. NOT raw `Field` arithmetic — REJECTED (reject-list (i)). This is
  instantiated in §4, not merely named.
- **B2 — verdict correct and constant-shape.** `filter_int.nr:96-110`, `signed_verdict`
  (`filter_signed.nr:57-87`), `f64_verdict` (`filter_float.nr:29-43`). Unconditional over
  typed `u64`/`f64`; untouched.
- **B3 — verdict asserted equal to public `expected`.** `filter_int.nr:111` /
  `filter_signed.nr:183` / `filter_signed.nr:284` / `filter_float.nr:152`. Unchanged.
- **B4 — canonical-form bind carried onto VALUE_HOOK.** Today on digit arrays
  (`filter_int.nr:58-64`, `filter_signed.nr:142-158`, `filter_signed.nr:215-246`).
  **CORRECTION (must-keep verdict):** with no digit arrays in the member, the no-leading-
  zero / no-`-0` / canonical-scale asserts have nothing to attach to. The member MUST
  EITHER re-introduce a constrained canonicalising re-derivation of VALUE_HOOK from a
  canonical witness in-circuit (costs gates — re-measure), OR the docs MUST honestly
  re-classify B4 to an issuer/ingest assumption (the §5.4 (2) larger escalation). **AUDIT
  OBLIGATION;** §5.4 records which path is taken. *Distinct from §5.2:* B4 stops a
  **prover** binding a second VALUE_HOOK encoding of the same value; §5.2 is about a
  **malicious issuer** setting VALUE_HOOK ≠ parse(lexical).

**C. Public inputs bind correctly — no malleability.**

- **C1 — verifier-side reconstruction byte-matches the proof.** A dual-leaf FILTER member
  changes the `pub`/witness layout (witnessed VALUE_HOOK + lexical_component witness + the
  per-datatype constants), so `reconstruct_public_inputs` **and** the six real-bb
  cross-vectors (`reconstruct_filter_int_matches_real_bb_public_inputs` and the
  f64/signed/decimal siblings, `verifier.rs` ~5010/5041/5111/5146/5184/5222) **must be
  updated in lockstep** (verified against the checkout). MANDATORY CO-CHANGE.
- **C2 — canonical verifier-side vk.** `verifier.rs` `canonical_vk`; `derive_id` must pin
  the new member's parameters. CO-CHANGE.
- **C3 / C4 — nonce + query-correctness binding.** `verifier.rs` field-0 challenge;
  `bind_query_correctness`. Unchanged.

**D. Nullifier / uniqueness / replay.** `record_fresh`; holder PoP / `holder_pok`.
Untouched.

### Reject-list (carried from §3.4 — still REJECTED)

(i) lower the typed comparison to raw `Field` arithmetic, or **omit the VALUE_HOOK
range-decomposition** (modular wrap — B1); (ii) drop the canonical-form binds on
VALUE_HOOK without re-classifying them honestly (second encoding — A1/B4); (iii) take
VALUE_HOOK as a free witness not anchored to the scan-bound commitment (A1/A2); (iv) feed
Baby-JubJub coords to the Grumpkin native MSM black-box (wrong curve — A9). **Specific to
dual-leaf:** (v) consult `value_component` in **any** identity-sensitive operator
(scan-row equality / `join_eq` / DISTINCT / sameTerm) — that re-introduces value-collapse
on the identity lane, and (per §5.5) `value_component` is many-to-one on the term for
double/float/decimal, so this is an affirmative hazard. Identity ops MUST consult the
full leaf (identity carried by `lexical_component`). **(v) must be STRUCTURALLY enforced,
not left as prose** (§8 fix below). REJECTED.

### Structural enforcement of reject-list (v) (term-identity verdict fix)

(a) Any identity-sensitive in-circuit member (scan-row eq, `join_eq`, future
DISTINCT/sameTerm) MUST take the **full leaf `Enc`** as its equality operand and MUST be
**structurally prevented** from receiving `value_component`/`VALUE_HOOK` as an input —
e.g. type-segregate `value_component` so it is only addressable inside FILTER members and
is never a selectable row slot. (b) The §11-bead-4 regression guard MUST be **expanded
beyond the integer `"05"`/`"5"` fixture** to assert non-collision for EACH many-to-one
VALUE_HOOK datatype: IEEE `-0.0` vs `+0.0` (distinct lexical, value-equal), NaN payloads,
and decimal `"5.0"` vs `"5.00"` at a fixed `FD` — proving they do NOT
join/dedup/sameTerm. (c) §9/§12 state term-identity preservation is **conditional** on
(a) and only verified by (b), NOT a property of the dual-leaf encoding itself.

### What binds each component (collision argument)

A prover cannot find a `value_component' ≠ value_component` or `lexical_component' ≠
lexical_component` with the same leaf (Poseidon2 collision-resistance + the
`assert_eq(leaf, operand_enc)` against the scan-anchored leaf force both witnessed
components to be the committed ones). Cross-datatype value collisions (integer `5` vs
decimal `5` vs the double bits for `5.0`) are prevented by `DATATYPE_CONST` inside the
inner value_component. Term identity (`"05"` vs `"5"`) is preserved by the distinct
`lexical_component` **and** by reject-list (v) being structurally enforced. **Standard
collision-resistance arguments under the Poseidon2 / blake3 assumptions — design intent
the external audit must verify, NOT a proven property.**

## 9. Audit-obligation registration (exact wording)

Extends CR-G8 to the FINAL dual leaf, folding in the issuer-desync (removed INV-VL),
term-identity (many-to-one + unenforced reject-list (v)), and must-keep (B1/B4
instantiation) verdicts. Obligation-/negation-framed to pass
`scripts/check-privacy-claims.sh`. The full revised CR-G8 row text is in
`compliance/cryptoreview/gap-register.md` (this PR edits it). SKILL/README wording:

### 9.1 ZK `SKILL.md` (`skills/zk-query-proofs/SKILL.md`) — caveat

> A dual-leaf value+lexical literal encoding (research-grade, design in
> `research/zk-field-native-encoding.md`) is proposed to cut numeric-FILTER gate cost
> while carrying a separate lexical-identity hash for sameTerm/DISTINCT/join; it is NOT
> implemented and NOT audited. It REMOVES the in-circuit invariant that the compared
> value equals the parse of the committed lexical bytes (today enforced against arbitrary
> committers), so the value-FILTER lane becomes sound only under an unverified
> honest-issuer-for-value assumption; term-identity preservation is conditional on an
> enforced rule that no identity operator reads the value handle (the handle is
> many-to-one on the term for double/float/decimal); and in-range single-encoding rests
> on an in-circuit range-decomposition that must be instantiated, not assumed. All of
> this is registered as an open external-audit obligation (CR-G8 / sq-qhy4); it provides
> no soundness or privacy guarantee. <!-- privacy-claims-allow: negative/obligation framing — names the unaudited dual-leaf encoding only to flag removed invariants as open audit obligations; sq-qhy4 -->

### 9.2 `crates/sparq-zk/README.md` — caveat

> A proposed dual-leaf value+lexical term/literal encoding
> (`research/zk-field-native-encoding.md`, research-grade, NOT implemented) would re-base
> the commitment onto value-first leaves carrying both a per-datatype numeric hook and a
> lexical-identity hash; it is unaudited, it removes the in-circuit value-equals-parsed-
> lexical invariant the current FILTER members enforce (value-lane consistency then rests
> on issuer honesty), and it makes no soundness or privacy claim — it is an open external-
> audit obligation (sq-qhy4). <!-- privacy-claims-allow: negative/obligation framing — flags an unimplemented unaudited encoding as an open audit obligation, asserts no guarantee; sq-qhy4 -->

## 10. Open questions (need the maintainer)

1. **`value_component` degeneracy shape (§3.2):** global `NO_VALUE` vs datatype-folded
   `Poseidon2([VALUE_NONE, DT_CONST, LANG_CONST])`? Recommendation: the latter. Confirm.
2. **dateTime/date VALUE_HOOK:** canonical epoch vs component tuple; timezone-
   normalisation rule. Deferred from the first slice.
3. **Double bits vs sq-mslu parser:** confirm against `filter_float.nr` /
   `sparq_ieee754` that the IEEE-bit VALUE_HOOK obsoletes the in-circuit RNE parser for
   the *comparison*, AND decide the `-0.0`/`+0.0`/NaN canonicalisation rule at ingest
   (§3.3 many-to-one), before closing/superseding sq-mslu.
4. **One-time recommit (§3.5):** confirm the value-first re-base of all persisted
   commitments/vectors/snapshot is acceptable now (research grade).
5. **B4 in-circuit vs ingest (§5.4):** is the cost of an in-circuit canonicalising
   re-derivation of VALUE_HOOK acceptable (keeping single-encoding a prover-side
   guarantee), or is the honest downgrade to an issuer/ingest assumption accepted? The
   §4/§6 ingest parse + the §4 B1 range-decomposition cost must be `bb gates`-measured
   first.
6. **Canonical-issuance conformance (§5.6):** is the canonical-issuance conformance
   mechanism — now a NAMED precondition for relying on the value lane against an
   adversarial issuer — wanted on the roadmap now (a §11 bead), or accepted as a stated
   precondition that bounds when the value lane may be relied upon?

## 11. Beads (orchestrator to create — ordered)

Recommended children of epic **sq-1s2**, gated on **sq-qhy4** for any production reliance
(research-grade / opt-in / NOT-yet-sound before sign-off). Existing related beads to
link, not duplicate: **sq-j506** (numeric lane in `encode`/`commit`), **sq-mslu**
(`xsd:double` RNE parser — likely refined). The orchestrator owns bead creation.

1. **Audit-obligation registration (CR-G8 revised + SKILL/README caveats, §9).** Doc-only;
   land FIRST so sq-qhy4 is forced to check the removed INV-VL, the B1/B4 instantiation,
   the structural reject-list (v) enforcement, the many-to-one handle hazard, and the
   value↔lexical issuer-honesty assumption. No code. *(This PR lands the CR-G8 + research
   doc; the SKILL/README edits are this bead's follow-on so they land WITH the impl.)*
2. **Host encoding overhaul + same-leaf co-binding at ingest (`encode.rs` + `commit.rs`).**
   Implement the dual leaf `Poseidon2([value_component, lexical_component, TYPE_CODE], 3)`
   with per-datatype VALUE_HOOK + the degenerate `value_component`; keep
   `lexical_component` = today's `blake3_field(to_string())`; **compute
   `VALUE_HOOK = parse(canonical(lexical))` from the SAME bytes, fail-closed** (§6).
   One-time atomic recommit of all leaf/commitment vectors. Extend/relink sq-j506. Tested.
   Audit-gated.
3. **Circuit + verifier co-change WITH B1/B4 instantiation.** Add dual-leaf FILTER
   member(s) (`filter_int`/`signed`/`decimal`/`float`) binding via the 2-Poseidon2
   constant-datatype path with `lexical_component` as a witness (NO in-circuit blake3),
   **including the explicit VALUE_HOOK range-decomposition (B1) and a B4 canonical bind or
   the honest §5.4 downgrade**; structurally type-segregate `value_component` so no
   identity op can read it (reject-list (v)); update `scan.nr`/`join.nr` leaf recompute to
   the dual leaf; update `reconstruct_public_inputs` + `derive_id`/`canonical_vk` + the six
   real-bb cross-vectors in lockstep (C1/C2); **re-measure with `bb gates` (including the
   range-decomposition)** and re-baseline `gate_count_snapshot.json` +
   `gate_counts_latest.json`. Depends on 2. Audit-gated.
4. **Identity-op + desync regression guard (EXPANDED).** Prove `value_component` is NEVER
   consulted by scan-row equality / `join_eq` / DISTINCT / sameTerm (reject-list (v)),
   with fixtures for EACH many-to-one datatype: integer `"05"`/`"5"`, IEEE `-0.0`/`+0.0`,
   NaN payloads, decimal `"5.0"`/`"5.00"` at fixed `FD` — asserting they do NOT
   join/dedup. Plus a desync-detection test: the §6 fail-closed ingest rejects a
   non-canonical hookable lexical. Depends on 3.
5. **Canonical-issuance conformance (NAMED PRECONDITION, §5.6).** Design + implement the
   conformance mechanism that, for conforming issuers, restores INV-VL as an issuance
   invariant and lets the leaf collapse back to the single value-first leaf (drop the
   carried lexical hash). Second one-time recommit. Audit-gated. This is the named
   precondition for relying on the value lane against an adversarial issuer, not merely a
   roadmap nicety.

Suggested ordering / deps: **1** (register first) → **2** → **3** (depends on 2) → **4**
(depends on 3) → **5** (depends on 4 + this draft's resolution). 2/3 are audit-gated
behind sq-qhy4 for production reliance; they may be implemented at research grade before
sign-off, consistent with the rest of the ZK estate.

## 12. Verdict

**The dual-leaf encoding is the correct realisation of the maintainer's #769 decision,
with two honesty corrections folded in from the adversarial review.** It gives the same
anchor-bracketed FILTER gate reduction as the value-only draft (17,416 → estimated
~3,200, to be re-measured **including the B1 range-decomposition**) by replacing one
in-circuit blake3 with two constant-datatype Poseidon2 permutations over the witnessed
VALUE_HOOK, while keeping the lexical-identity hash in the same leaf so identity ops stay
correct **conditionally** (see below). The cost, stated honestly, is that the lexical
hash is still carried (more commit-time cost than value-only) and that the value lane
loses an invariant it has today.

**Correction 1 (issuer-desync — the maintainer's malicious-issuer question, answered
straight):** binding value↔lexical in-circuit would re-derive the value from the lexical
bytes — exactly the blake3/parse the value handle exists to avoid — so consistency CANNOT
be enforced in-circuit without defeating the gate win. **This means the dual leaf REMOVES
the in-circuit invariant INV-VL (value = parse(committed lexical)) that the current
`filter_int`/`signed`/`decimal`/`float` members enforce against arbitrary committers
including a malicious trusted issuer.** A malicious *trusted* issuer can therefore commit
one signed credential that answers a value question as `18` and a sameTerm/DISTINCT/join
question as `5` — **impossible today**, a NEW capability, a strict trust-model regression
for the value-FILTER lane (not "the issuer could already lie about the value"; that lie
keeps value and lexical in agreement). No untrusted party can exploit it (the
scan/issuer chain still binds to a trusted signature), and a host-side same-leaf
co-binding at ingest (§6, fail-closed) makes honest sparq ingest unable to desync.
Canonical-issuance conformance (§5.6) is the NAMED PRECONDITION that restores INV-VL for
conforming issuers and is the exit path that retires the dual leaf.

**Correction 2 (must-keep + term-identity):** the value-FILTER member MUST *instantiate*,
not merely name, an explicit in-circuit range-decomposition of VALUE_HOOK (B1) and a
canonical-form bind (B4), or in-range-ness and single-encoding silently downgrade to
issuer/ingest assumptions — a larger escalation than the value↔lexical point, which the
docs must state. And term-identity preservation is **conditional** on structurally
enforcing reject-list (v) (no identity op reads `value_component`, which is many-to-one on
the term for double/float/decimal) and is verified only by the expanded regression guard,
NOT a property of the encoding itself.

All of this — the removed INV-VL and its honest-issuer-for-value assumption, the B1/B4
in-circuit instantiation (or its honest downgrade), the structural reject-list (v)
enforcement, the many-to-one handle hazard, and the canonical-issuance precondition — is
**registered as an explicit external-audit obligation (CR-G8 / sq-qhy4)**; none is
presented as a settled property, and no soundness or privacy guarantee is claimed pending
the external sign-off.
