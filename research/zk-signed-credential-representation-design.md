<!-- [OPUS-4.8] Design-for-review authored by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns. -->
# Signed typed-value representations: removing in-circuit literal parsing from SPARQL-over-credentials proofs

<!-- [OPUS-4.8] sq-5reoy (#1599): the in-tree `zk/ieee754` library was externalized to the `sparq-org/noir_IEEE754` (v0.10.0) face repo and REMOVED from this repo; `zk/compose` now consumes the released `sparq_ieee754` as a pinned Nargo git dependency. Any `zk/ieee754/…` path below is a HISTORICAL in-tree reference — the live source is the face repo. -->

Design-for-maintainer-review record. NO production code lands here. This proposes
a change to **what the issuer signs / what the commitment binds** so that the
numeric-FILTER, relational-comparison, equality, and join relations in
`zk/compose/` can verify a predicate over a hidden typed value **without
re-parsing a lexical literal in-circuit and without re-hashing its canonical
N-Triples token with the `blake3` BLACKBOX**.

This record matches the rigour of `research/zk-soundness-audit.md`,
`research/zk-verifier-reaudit.md`, `research/zk-hidden-join-design.md`, and
`research/zk-holder-pop-design.md`. It is a remediation *direction* under the open
soundness epic, not a hardened scheme.

> **Honesty / scope (read first).** The sparq ZK/MPC estate is **research-stage and
> NOT externally audited** (open beads `sq-qhy4` / `sq-9hrn`; remediation epic
> `sq-1s2`). `verify_manifest` is the full-binding path and an internal re-audit
> finds it sound-as-landed under its stated threat model, but it is **pending
> external cryptographer sign-off and must be treated as NOT-yet-sound for
> production**. **Nothing in this document is a soundness claim.** Where it argues
> a property is preserved or a new risk is closed, that is a *design argument to be
> reviewed*, predicated on external sign-off. No gate-count or latency numbers are
> measured here — the only figures cited are the repo's own checked-in baselines
> (`crates/sparq-zk-compose/tests/gate_count_snapshot.json`); they were recorded on
> a non-canonical work-box and are the repo's *recorded* baselines, not freshly
> re-measured. Any number in a future implementation must be re-measured with
> `bb gates` (the regression gate `sq-c5f` / `sq-mj2z`). All ZK mentions are
> caveated for the live privacy-claims gate.

Parent context: epic `sq-1s2` ("ZK query-proof build-out + in-circuit privacy
upgrades"). The two flagged precedents this design extends are the repo's own
`crates/sparq-zk-compose/README.md:255-273` ("sparq-zk API gaps noted — No numeric
value lane in the encoding") and `zk/compose/compose_core/src/filter_int.nr:1-15`
(the module doc that calls the in-circuit blake3 re-derivation "a deliberate,
measured exception to the never-hash-strings-in-circuit house rule").

---

## 1. The problem, precisely, with cited overhead evidence

### 1.1 The host-vs-circuit split and the gap it opens

`sparq-zk` commits every term **off-circuit** (`crates/sparq-zk/src/encode.rs:46-62`):

```text
Enc_t(term) = h2(type_code, h_s(value))
```

where `h_s` = `blake3` over the canonical byte serialization and `h2` = Poseidon2.
For a literal (`encode.rs:52-55`) the hashed bytes are the **full lexical N-Triples
token** — `l.to_string()`, i.e. the string `"5"^^<http://www.w3.org/2001/XMLSchema#integer>`
including the quotes and the `^^<datatype>` suffix. The triple leaf is
`h3(Enc(s), Enc(p), Enc(o))` (`encode.rs:65-73`) and the per-graph commitment is
`commit_fold` over leaves (`zk/compose/compose_core/src/hashes.nr:30-32`).

The BGP **scan** circuit treats every encoding as an **opaque Field** and only
recomputes Poseidon2 layers (`scan.nr` step 1; `hashes.nr:12-15` notes each
`h2`/`h3` is one Poseidon2 permutation, ~74 gates). All string hashing is on the
host. That is exactly the right design for scan/join/revoke/issuer/holder — and it
is why those circuits are cheap.

The gap: **a committed numeric literal has no field-arithmetic binding** — it is
hidden behind a one-way `blake3` hash of a *string*. So to prove a numeric
predicate (a FILTER) about the hidden value, the circuit must somehow recover the
arithmetic value and bind it to the committed term. The current code does this the
hard way: it re-derives the exact host binding **in-circuit**.

### 1.2 The in-circuit literal-parse-and-rehash site

`zk/compose/compose_core/src/filter_int.nr:46-92` is the primary site. `filter_int_check<D>`:

1. takes the hidden integer's decimal digit **bytes** as a private witness `digits: [u8; D]`;
2. validates them as ASCII digits and rejects a leading zero (`:58-64`) — a partial canonicaliser;
3. folds them into a `u64`: `value = value*10 + (digit-48)` (`:67-70`) — **this is the lexical string→field conversion, done in-circuit**;
4. **rebuilds the canonical N-Triples token** `"<digits>"^^<…#integer>` (`:74-83`);
5. runs the `std::hash::blake3` BLACKBOX over that token (`:84`);
6. truncates the digest to the low 31 bytes big-endian (`:88-91`, mirroring `sparq_zk::field::field_from_hash_bytes`);
7. asserts `h2(TYPE_CODE_LITERAL, hs) == operand_enc` (`:92`), where `operand_enc` is the PUBLIC term encoding the scan proof bound to the committed triple slot;
8. *only then* compares `value` against `bound` (`:94-111`).

The same pattern repeats across the literal-binding FILTER family:

- `filter_float.nr:94-152` — `filter_f64_composable_check<D>`: digit-bytes→`u64`, canonical `xsd:double` token rebuild + in-circuit blake3 + truncation + `operand_enc` bind, then IEEE bits via `f64::from(value)` (integer-valued double fragment only; a general decimal→IEEE parser is explicitly deferred, module doc `:1-12`).
- `filter_signed.nr:128-184` (`filter_signed_int_check<MD>`) and `:200-285` (`filter_decimal_check<ID,FD>`): parse sign + magnitude (+ fraction) digit bytes to `u64` (`:150-153` / `:230-242`), rebuild the exact lexical token (with optional `-` / `.`) and bind via the shared `assert_literal_binding<LEN,SLEN>` helper (`:93-115`) — itself the in-circuit blake3 token hash + low-31-byte truncation + `h2(LITERAL,hs)==operand_enc`.

The host side (`crates/sparq-zk-compose/src/build.rs:299-306`, `:388-413`) does the
*easy* half: `digits = value.to_string()` then ships the digit bytes as the witness;
`encode_signed_int_literal` / `encode_decimal_literal` build the lexical form on the
host. `derive_filter_int_id` (`:300`) pins `D` to the exact decimal digit count — a
wrong-`D` member is unprovable (sq-wto).

### 1.3 The overhead is unmistakable in the repo's own checked-in baselines

`crates/sparq-zk-compose/tests/gate_count_snapshot.json` (tool `bb gates -s ultra_honk`,
`circuit_size`; bb `5.0.0-nightly.20260324` / nargo `1.0.0-beta.21`) records:

| member | `circuit_size` | note |
|---|---|---|
| `filter_int_d1` … `filter_int_d4` | **17416** (all) | literal-bound integer FILTER |
| `filter_f64_d1` … `filter_f64_d4` | **17416** (all) | literal-bound double FILTER |
| `filter_signed_int_d2`, `_d4` | **17416** | literal-bound signed-int FILTER |
| `filter_decimal_i3_f2` | **17416** | literal-bound decimal FILTER |
| **`filter_f64`** (raw IEEE bits, **no** operand binding, no token rebuild, no in-circuit blake3) | **3113** | the contrast member |
| `scan_k1_n16_r4` | 5991 | Poseidon-only, scales with `N` |
| `join_eq_na16_nb16` | 7025 | two `commit_fold` re-commitments |

Two observations the snapshot makes for us:

- **The in-circuit literal-parse-and-bind layer is ≈ `17416 − 3113 ≈ 14303`** circuit-size units versus the raw-bits float filter that does the comparison but binds nothing. (Stated as the repo's recorded delta, not a fresh measurement.)
- **It is constant across `D` (1–4) and across int / double / signed / decimal.** This constancy is the tell that the cost is the **blake3 compression of a ≤64-byte canonical token** — one compression block — which dominates (~82% of a bound filter per `README.md:261-266`); the digit-fold + token rebuild + comparison arithmetic are negligible against it.

The project attributes this exactly: `README.md:261-266` ("re-deriving the blake3
token binding in-circuit is 'a deliberate, measured exception to never hash strings
in-circuit, ~17.4k gates'"); `README.md:76` ("bb gates: 17416 each
(blake3-block-bound, identical to filter_int)"); `filter_int.nr:1-15` ("blake3
BLACKBOX … ~3 orders below the standard-cryptosuite cliff").

**Problem statement.** The numeric/comparison/decimal FILTER relations pay a fixed
~14.3k-unit in-circuit `blake3`-of-a-string layer **solely to recover an arithmetic
binding that the commitment threw away** by hashing a lexical token. The cost is
intrinsic to the *representation choice* (commit a string hash, not a field value),
not to the comparison logic. Additional structural costs of the representation
choice: a **per-digit-count circuit family** (`filter_int_d{1..4}`, exact-`D`
witness `[u8;D]`) that **leaks `ceil(log10(value))`** of the hidden operand
(`filter_int.nr:26-28`); and a `filter_f64` composable path that needs in-circuit
canonical-decimal token printing (deferred for general floats,
`filter_float.nr:1-12`).

---

## 2. Proposed representation(s): sign the canonical typed value

### 2.0 One-line statement

**Add a field-arithmetic numeric value into what the issuer signs and what the
commitment binds, alongside (not replacing) the existing string-hash encoding.**
Then a FILTER/comparison/join binds to the *numeric field* by a Poseidon2
re-combination — **pure field arithmetic, no in-circuit string hashing, no
per-digit-count family.** This is the central technique of the reference repo
(`jeswr/sparql_noir`, §4) adapted to sparq's Poseidon2 commitment and Schnorr
signature.

### 2.1 What the issuer signs / commits — the value lane

Introduce a **dual-lane term encoding** for typed literals. The string lane is
unchanged (preserves the existing scan/join/issuer/revoke binding byte-for-byte);
a new **value lane** is added for the typed-value-bearing literals.

For a typed literal `l` with datatype IRI `dt` and a canonical typed value `v`:

```text
Enc_str(l) = h2(TYPE_CODE_LITERAL, blake3(canonical_token))           // UNCHANGED — encode.rs:52-55
Enc_val(l) = h2(TYPE_CODE_TYPED_VALUE, h3(VALUE_DOMAIN, v_field, dt_field))
```

where:

- `v_field` is the **canonical typed value as a field element** — the load-bearing change. For `xsd:integer`/`xsd:byte`/`xsd:long`/etc., `v_field = Fr::from(value)` directly (no lexical bytes). Sign handled by §2.4. For `xsd:double`/`xsd:float`, `v_field` carries the **IEEE-754 bit pattern** as an integer field (u64/u32 bits), reusing the project's `zk/ieee754` library convention — numeric, not lexical (this is the reference's `noir_IEEE754` choice, §4). For `xsd:decimal`, `v_field` is a fixed-point scaled integer with the scale folded into `dt_field` (§2.5). `xsd:boolean` is `0`/`1`. `xsd:dateTime`/`xsd:date` are a canonical integer (e.g. a fixed-epoch offset; §2.5).
- `dt_field` is a **datatype tag** — NOT the lexical IRI bytes, but a small enumerated code under `VALUE_DOMAIN` domain separation so that `xsd:integer` `5` and `xsd:byte` `5` and `xsd:long` `5` **never collide** (the datatype-confusion risk, §6.3). The mapping `dt_field ← canonical_datatype_iri` is fixed, total, and shared host↔circuit (a single source-of-truth table).
- `VALUE_DOMAIN` / `TYPE_CODE_TYPED_VALUE` are fresh domain separators distinct from every existing `SIG_DOMAIN_*` / `TYPE_CODE_*` (`sig.rs`, `encode.rs:33-35`, `hashes.nr:35-37`) so a value-lane encoding can never be cross-substituted for a string-lane one or any other commitment artefact.

The triple leaf and per-graph commitment are unchanged structurally. The cleanest
binding (preferred): make the **leaf carry both lanes** for the object position when
it is a typed-value literal — `leaf = h3(Enc(s), Enc(p), h2(BIND_DOMAIN, Enc_str(o), Enc_val(o)))`
— so a single `commit_fold` still produces `C(G)` and the issuer signs `C(G)`
exactly as today (`sig.rs:409-411`, `:262-274`). The value lane is thereby
**issuer-attested transitively through the existing signature** with **no change to
the signature scheme**. (Alternative leaf shapes in §3.4.)

> **Adopt vs adapt (grounding in `jeswr/sparql_noir`, §4):** we **adopt** the
> reference's "put the canonical numeric value directly in the committed hash"
> technique and its "unpacked-value witness + recompute-and-assert" binding. We
> **adapt** the hash shape (reference uses `hash4([value, special_encoding, langtag,
> datatype])` wrapped in `hash2([Literal, …])`; sparq uses Poseidon2 `h2`/`h3` to
> stay byte-compatible with the existing commitment in `hashes.nr`), and we **keep
> sparq's signature scheme** (Schnorr/Baby-JubJub/Poseidon2, `Poseidon2SchnorrV1`)
> rather than the reference's pluggable suite axis — the value lane is signature-
> agnostic because it binds through `C(G)`, which the existing signature already
> covers.

### 2.2 The credential / leaf format (selective-disclosure compatible)

The unit committed and signed stays the **per-named-graph** commitment `C(G)`
(sparq's granularity), so existing issuer attestation, salt binding
(`commitment_message_with_salt`, `sig.rs:434-436`), status-ref binding
(`commitment_message_with_status`, `sig.rs:262-274`), and holder binding
(`commitment_message_with_holder`, `sig.rs:288-297`) are untouched. The reference
signs a per-triple Merkle root and proves inclusion (§4); sparq's named-graph fold
is the existing equivalent — we keep it (this is the **commitment-granularity
divergence**, §4, retained deliberately to avoid a rewrite).

A literal leaf-position now logically carries `{ Enc_str, Enc_val }` for typed-value
literals; for IRIs, strings, bnodes, and untyped/lang-tagged literals only the
string lane exists and the value lane is absent (the leaf falls back to today's
`Enc_str`-only shape — §6.5).

### 2.3 How the circuit binds + compares WITHOUT re-parsing

The FILTER relation becomes:

1. The prover supplies the **unpacked numeric value `v` as a hidden field witness** (no digit bytes).
2. The circuit recomputes `Enc_val = h2(TYPE_CODE_TYPED_VALUE, h3(VALUE_DOMAIN, v, dt))` — **two Poseidon2 permutations** (`hashes.nr:12-15`, ~74 gates each) — and asserts it equals the PUBLIC `operand_enc_val` that the scan proof bound to the committed triple slot. **No `blake3`, no token rebuild, no digit fold.** This replaces `filter_int.nr:58-92` wholesale.
3. The comparison runs as **native field/integer arithmetic on `v`** exactly as the current `filter_int.nr:94-111` already does (`value < bound`, etc.) — that arithmetic is unchanged and was always cheap.

So the relation collapses from "validate digits → fold → rebuild token → blake3 →
truncate → bind → compare" to "**recompute one Poseidon2 chain → bind → compare**".

- **Relational comparison** (`<`, `<=`, `>`, `>=`, `=`, `≠`): same as above; `v` enters the existing operator-mux. For signed integers the comparison uses the **offset form** the codebase already prefers (noir-optimisation S15.1: "avoid i64; use offset `value + offset`") and which `filter_signed.nr` documents ("Why two relations, not signed i64") — §2.4.
- **Equality / JOIN** (`join.nr:177`, `assert(a_val == b_val)`): the join already operates on *opaque term encodings* `Enc(o)`, so it works on the value lane unchanged — equate `Enc_val(a) == Enc_val(b)` instead of (or in addition to) `Enc_str`. Because `Enc_val` is a deterministic function of `(v, dt)`, two literals join iff they have the **same datatype tag and the same canonical value** — which is precisely SPARQL value-equality for the numeric datatypes (a strict *improvement*: today's string-hash join is term-equality, so `"5"^^xsd:integer` and `"05"^^xsd:integer` would not join even though they are the same value — but note §6.2 makes `"05"` non-canonical/uncommittable anyway). The hiding join-value commitment (`join_value_commitment`, `join.nr:83-85`) is unchanged.

### 2.4 Signed integers (offset form, no `i64`)

Noir's `i64` will not cast to `Field` and the repo already rejects it
(noir-optimisation S15.1; `filter_signed.nr` "Why two relations, not signed i64").
Two clean options, in preference order:

- **(A) Offset value lane.** Define `v_field = value + OFFSET` for a fixed
  `OFFSET = 2^63` so every `xsd:integer` in `[-2^63, 2^63)` maps to a non-negative
  field in `[0, 2^64)`. The datatype tag is `xsd:integer` for both signs (no
  separate signed member). The circuit recovers `value = v_field − OFFSET` for the
  comparison (a single subtraction; comparison stays in the shifted domain to avoid
  underflow, as `filter_signed.nr`'s `signed_verdict` already does). This **collapses
  `filter_int` + `filter_signed_int` into one relation** and removes the sign-branch
  token rebuild (`filter_signed.nr:160-180`).
- **(B) Sign-flag + magnitude value lane.** `v_field = h3(VALUE_DOMAIN, neg?1:0, magnitude)`.
  Mirrors the current `filter_signed` decomposition more literally but keeps a
  branch; (A) is preferred for fewer members and no `-0` ambiguity (the offset map
  is injective, so `−0` simply cannot arise — closes `filter_signed.nr:154-158`).

### 2.5 Non-integer terms — canonical typed encodings

| term kind | value lane `v_field` | datatype tag `dt_field` | comparison |
|---|---|---|---|
| `xsd:integer` (and `byte`/`short`/`int`/`long`/`unsignedInt`…) | offset integer (§2.4) | one tag **per concrete datatype** (anti-confusion, §6.3) | native, shifted domain |
| `xsd:decimal` | scaled integer `round(value · 10^FD)` | tag **includes the scale `FD`** | fixed-point integer compare; host pre-scales the bound (as `filter_decimal_check` already does, `filter_signed.nr:186-199`) |
| `xsd:double` / `xsd:float` | IEEE-754 **bit pattern** as a u64/u32 field | `double` vs `float` tag (distinct widths) | `zk/ieee754` bit-level NaN-aware compare (reference's `noir_IEEE754`, §4) — **no in-circuit decimal→IEEE printing**, which is what `filter_float.nr` currently defers |
| `xsd:boolean` | `0` / `1` | `boolean` tag | equality only |
| `xsd:dateTime` / `xsd:date` / `xsd:time` | canonical integer (e.g. seconds from a fixed epoch; timezone-normalised to UTC at sign time) | one tag per temporal datatype | native integer order = chronological order |
| **IRI** | — (no value lane) | — | term-equality via `Enc_str` only (unchanged) |
| **plain / `xsd:string` / lang-tagged literal** | — (no value lane) | — | term-equality via `Enc_str`; `STRLEN`/`CONTAINS` stay byte-ops out of this design's scope (reference, §4) |
| **blank node** | — (no value lane) | — | unchanged (salted `Enc_str`, `encode.rs:56-59`) |

The **canonicalisation is performed once, by the issuer, at sign time** (host
`build.rs` analogue), not in the circuit. The circuit only re-runs the Poseidon2
recombination and asserts. This is the host-does-the-hard-part / circuit-verifies
discipline the codebase already uses for the string lane (`encode.rs:17-26`), moved
to the value lane.

---

## 3. Migration path onto the existing `sparq-zk-compose` pipeline (NOT a rewrite)

This is **additive and opt-in**, matching the repo's opt-in-feature architecture and
the way `filter_signed`/`filter_decimal`/`hidden_issuer`/`join_eq` were each added as
new compose-core relations + new bin members without disturbing the others. Phased so
each phase is independently gateable (`cargo test -p sparq-zk-compose`, the
`gate_count` regression suite, both feature states).

**Phase 0 — `sparq-zk` value lane (host).** Add `Enc_val` to
`crates/sparq-zk/src/encode.rs` behind a new additive `pub fn encode_typed_value(term, …) -> Option<Fr>`
and the datatype-tag table; add the dual-lane leaf option to `commit.rs`. This is the
"future additive `sparq-zk` API exposing a numeric value lane" the README already
calls for (`README.md:264-266`). The string-lane encoding and existing commitments
are byte-unchanged for non-value terms, so **all existing scan/join/issuer/revoke
proofs continue to verify**. New `IngestedDataset` graphs opt into dual-lane leaves;
old graphs keep single-lane (§6.5 versioning).

**Phase 1 — new compose-core relation.** Add `filter_value.nr` to
`zk/compose/compose_core/src/` implementing §2.3 (recompute `Enc_val` + native
compare). It is **generic only over the comparison op and datatype tag — NOT over a
digit count** (the per-`D` family disappears for the value path). Add the bin
package(s) `filter_value_<dt>/src/main.nr` calling `filter_value_check`, exactly the
thin-bin pattern of `filter_int_d1/src/main.nr`. Update `tests/gate_count_snapshot.json`
+ `bench/zk-compose/gate_counts_latest.json` with measured baselines (the
`bench_json_matches_snapshot` parity test forces both, `gate_count.rs:297`).

**Phase 2 — host builder + manifest.** Add `build_filter_value(operand_enc_val, value, dt, op, bound, expected)`
to `build.rs` (mirrors `build_filter_int`, `:292`, minus the digit-byte witness).
Add `ProofInputs::FilterValue` + `CircuitId::FilterValue` to `manifest.rs` and the
`derive_*_id` resolver (the new id has **no `d` field** — a structural simplification).
The scan proof must now also disclose the **value-lane** encoding for an operand
column (`operand_enc_val`) alongside the existing `operand_enc`; the binding edge
(`BindingEdge`, scan→filter) carries `operand_enc_val`. `prover_toml_for` learns the
new inputs.

**Phase 3 — verifier.** `verify_manifest` (`verifier.rs`) gains a `FilterValue`
sub-proof arm that binds `operand_enc_val` across the scan and filter public inputs
(the `binding_consistency` edge, same shape as today's `filter_int.nr:17-23` note).
**No new trust anchor** — issuer key-set, status policy, nonce all unchanged.

**Phase 4 — deprecate, don't delete.** Keep `filter_int`/`filter_f64`/`filter_signed`/`filter_decimal`
compiled and composable for backward compatibility and for credentials issued before
the value lane (single-lane graphs can *only* use the blake3 path). Route new issuance
+ new proofs to `filter_value`. The `gate_count` regression gate documents the saving
directly: the new member's baseline sits next to the `17416` legacy members.

Each phase carries the `[OPUS-4.8]` marker on new code and the Opus 4.8
`Co-Authored-By` trailer, per repo policy.

### 3.4 Leaf-shape alternatives (for reviewer choice)

- **(Preferred) Combined-leaf:** object position binds `h2(BIND_DOMAIN, Enc_str, Enc_val)`. One `commit_fold`, signature unchanged, both lanes attested. Scan must recompute the combine for value-bearing objects (one extra `h2`).
- **(Simpler, more leakage) Sibling-leaf:** add a second leaf `h3(Enc(s), Enc(p), Enc_val(o))` to the graph. No scan change, but it doubles a value-bearing triple's leaf count and changes `C(G)` arithmetic; the scan `N` buckets would need re-checking.
- **(Most reference-faithful) Inner hash4:** `Enc_val = h2(TYPE_CODE_TYPED_VALUE, h4(value, special, lang, dt))` to mirror `jeswr/sparql_noir` exactly. Adds an `h4`; only worth it if a `langtag`/`special` axis is wanted later. Combined-leaf is recommended for v1.

---

## 4. Grounding in `jeswr/sparql_noir` — adopt vs adapt

The reference scheme (per the scout study of `jeswr/sparql_noir` README /
ARCHITECTURE.md / IEEE754_MIGRATION.md / spec/{proofs,algebra}.md / src/encode.ts /
transform/src/expr.rs, plus the `jeswr/queryable-credentials` design proposal and the
zksparql.org ISWC 2025 paper) is the **inverse representation choice** to sparq's.

**ADOPTED (direct technique transfer):**

1. **Direct canonical-value-in-commitment.** Reference: for `xsd:integer`,
   `special_encoding = parseInt(term.value, 10)` placed **directly as a field** inside
   `hash4([value, special_encoding, langtag, datatype])` (`src/encode.ts`
   `specialLiteralHandling`; `spec/algebra.md`). We adopt placing the canonical numeric
   value as a field in the committed/signed encoding (§2.1).
2. **Unpacked-value witness + recompute-and-assert binding.** Reference: the prover
   supplies the unpacked numeric value as a hidden input and the circuit re-runs the
   *same* hash recombination and asserts equality to the encoded term
   (`spec/algebra.md`: "hidden inputs provide the unpacked numeric value and assertions
   verify the unpacking is correct relative to the encoded term"). We adopt this exactly
   (§2.3) — it is **field/hash arithmetic, not an in-circuit string hash**, which is the
   whole point.
3. **IEEE-754 bit-encoded floats.** Reference: `xsd:double`/`float` use `noir_IEEE754`
   bit-encoded value fields (u32/u64 bit patterns), comparison via NaN-aware bit ops,
   "integer↔float via u32/u64, no lexical parsing." We adopt this for the double/float
   value lane (§2.5), reusing the in-repo `zk/ieee754` library.
4. **Native-arithmetic comparison lowering.** Reference: integer/decimal FILTER →
   `(a as i64) cmp (b as i64)` / xpath int ops; doubles → IEEE compares
   (`transform/src/expr.rs`). sparq already does native compares (`filter_int.nr:94-111`);
   we keep them and only delete the parse/rehash that precedes them.
5. **Disclose-and-verify-externally discipline.** Reference keeps DISTINCT/ORDER/LIMIT
   and string ops out of circuit. We keep `STRLEN`/`CONTAINS`/post-processing out of this
   design's numeric scope.

**ADAPTED (changed to fit sparq, with reason):**

- **Hash shape.** Reference `hash4(...)` wrapped in `hash2(...)`; sparq uses Poseidon2
  `h2`/`h3` to stay bit-compatible with the existing `hashes.nr` commitment so the
  signature and scan recompute are untouched (we do not import the reference's `h4`
  unless the sibling-axes are wanted, §3.4).
- **Commitment granularity.** Reference signs **one Merkle root over per-triple leaves**
  and proves per-triple inclusion; sparq signs a **per-named-graph** Poseidon2 fold
  `C(G)`. We **retain sparq's granularity** (the value lane rides through the existing
  `C(G)` signature) rather than migrate to per-triple Merkle — that migration is out of
  scope and would be a rewrite.
- **Signature suite.** Reference treats the suite (Schnorr/BBS+/SD-JWT-VC/ed25519/ECDSA,
  ML-DSA/lattice-BBS PQ) as a pluggable axis; sparq keeps the single
  `Poseidon2SchnorrV1` suite. The value lane is **suite-agnostic** because it binds
  through `C(G)`.
- **Datatype tag, not lexical IRI.** Reference folds a datatype-IRI hash; we fold a
  small enumerated **tag** to make datatype-confusion a *type error in the table* rather
  than an IRI-string-equality question (§6.3).

**NOT adopted (out of scope):** the reference's far broader SPARQL-1.1 algebra fragment
(OPTIONAL/UNION/property-paths/EXISTS/MINUS) and its Lean-4 mechanized soundness
(Pérez–Arenas–Gutiérrez, zksparql.org). sparq's composable fragment stays BGP scan +
FILTER + single hidden JOIN; the **per-element salted typed-value hashing** from
`jeswr/queryable-credentials` (separate datatype/value/langtag hashes + Merkle rollup
for independent disclosure) is noted as a *future* alternative if per-triple selective
disclosure of a single typed value is later required (§7).

---

## 5. Expected gate savings (QUALITATIVE — no fabricated numbers)

> No new circuit was compiled here; the figures below are **the repo's own recorded
> baselines** (`gate_count_snapshot.json`), used only to bound expectations. Any
> realised number MUST be re-measured with `bb gates` via the `sq-c5f`/`sq-mj2z`
> regression gate before being quoted as canonical.

- **The blake3 layer is removed from the FILTER path.** Per `README.md:261-266`, the
  in-circuit blake3 token re-hash is ~82% of a bound filter's circuit size; the snapshot
  shows every bound filter at `17416` versus the no-binding `filter_f64` at `3113`. The
  value-lane binding replaces blake3 with **two Poseidon2 permutations** (`hashes.nr:12-15`:
  one each, ~74 gates) plus the *unchanged* comparison arithmetic. **Direction:** the new
  `filter_value` member should land **far below the `17416` legacy members** and in the
  neighbourhood of the comparison-only cost — i.e. close to the contrast `filter_f64`
  (3113) order, not the bound-filter order. (Stated as a direction; the regression gate
  will record the actual value.)
- **The per-digit-count family collapses.** `filter_int_d{1..4}` (4 members),
  `filter_f64_d{1..4}` (4), `filter_signed_int_d{2,4}` (2), `filter_decimal_i3_f2` (1) —
  the value path needs **one relation per datatype-tag class**, not per digit count.
  Fewer compiled members, smaller proving-key surface, simpler `derive_*_id`.
- **A leakage channel closes.** No digit-count witness `[u8;D]` means
  `ceil(log10(value))` is no longer leaked by member selection (`filter_int.nr:26-28`).
- **The float fragment generalises.** Carrying IEEE bits directly removes the in-circuit
  canonical-decimal printing that `filter_float.nr:1-12` deferred — the general
  fractional/scientific double FILTER becomes reachable (a *capability* gain, separate
  from the gate saving).

How to demonstrate: add the `filter_value` baseline to `gate_count_snapshot.json` and
`bench/zk-compose/gate_counts_latest.json`; the existing `gate_count_regression` /
`bench_json_matches_snapshot` tests (`tests/gate_count.rs`) then make the saving a
checked-in, regression-gated fact sitting beside the `17416` legacy rows.

---

## 6. Soundness (design argument — NOT an audit claim)

> Predicate-form caveat applies to every clause: these are properties to be reviewed
> under external sign-off (`sq-qhy4`/`sq-9hrn`), not guarantees. The estate is
> NOT-yet-sound.

### 6.1 Why signing the value preserves binding

The value lane is bound to the issuer through the **same** `C(G)` signature that
already binds the string lane (§2.2; `sig.rs:262-274`, `:409-411`). The combined leaf
`h2(BIND_DOMAIN, Enc_str, Enc_val)` means a prover cannot present a `v` whose `Enc_val`
the issuer did not commit: the in-circuit `commit_fold` recompute (the scan/join
present-in-graph discipline, `scan.nr` step 1 / `join.nr:145-158`) would not reproduce
the signed `C(G)`. So the chain is unchanged in *kind* — "in-circuit Poseidon2
commitment recompute → public commitment → host Schnorr verify over Poseidon2(domain,
commitment, …)" — only the leaf now also fixes `(v, dt)`. The binding argument is
**identical structure** to the existing one; the difference is that the bound quantity
is a numeric field instead of a re-hashed lexical token. (This is exactly the
reference's argument, §4: a hash/field recombination, not a string hash, binds the
operand.)

### 6.2 NEW risk: two lexical forms → same integer

A signed numeric value is value-equal across lexical variants (`5`, `05`, `+5`,
`5.0`-as-integer). If two distinct credentials could be issued whose value lanes both
encode `5`, value-equality joins them — which is *correct* SPARQL value semantics, but
it also means a malicious **issuer-side** canonicalisation bug could mint a value the
holder's *string* lane disagrees with.
**Closure.** (a) The canonicalisation is **single-sourced**: the host `encode_typed_value`
computes `v_field` from oxrdf's already-canonicalised typed value, and the **string lane
must agree** — add a host invariant (and a circuit-optional cross-check for the
combined leaf) that `Enc_str` and `Enc_val` describe the same datatype. (b) Forbid
non-canonical lexicals at *ingest*: `"05"^^xsd:integer` is non-canonical and the
existing `filter_int.nr:61-63` leading-zero rejection already encodes that intent — lift
it to the issuer/ingest boundary so a non-canonical literal is **uncommittable** in the
value lane. (c) The value lane is a *deterministic function of `(v, dt)`*, so two
distinct field values cannot be the same integer — the map is injective by construction
(offset form, §2.4, makes this exhaustive).

### 6.3 NEW risk: datatype confusion (`xsd:integer` / `decimal` / `byte`)

If the value lane folded only the bare integer, `"5"^^xsd:byte` and `"5"^^xsd:integer`
and `"5.0"^^xsd:decimal`-scaled-to-5 would collide and could be cross-substituted.
**Closure.** `dt_field` is folded **inside** `Enc_val` under `VALUE_DOMAIN` (§2.1) with a
**distinct tag per concrete datatype**, and for `xsd:decimal` the **scale `FD` is part of
the tag** (§2.5). So a `byte 5` and an `integer 5` have different `Enc_val`; the verifier
(and the join equality) treat them as different terms — matching SPARQL, where
value-equality across the numeric type hierarchy is a deliberate semantic choice the
*query layer* opts into, not a commitment-layer accident. A FILTER member is
parameterised by the datatype tag, so a proof for an `xsd:integer` FILTER cannot verify
against an `xsd:byte` operand (wrong tag → `Enc_val` mismatch → unsatisfiable).

### 6.4 NEW risk: field-boundary sign / overflow

A naive integer-as-field admits `value` and `value + p` (the field modulus) as the same
element, and an offset/scale could overflow.
**Closure.** (a) **Range-bind the witness.** The circuit must `assert` `v` (or
`magnitude`) is within the declared width (`< 2^64` for the integer lane, `< 2^(ID+FD)·…`
for decimal as `filter_signed.nr` already bounds `MD<=19`, `ID+FD<=19`). Without this,
a prover could choose a field element that aliases a different integer — a genuine
soundness hole (the analogue of the noir-optimisation S3.4 `pow2`-not-bound-to-`shift`
pitfall: a hint must be bound back). (b) **Offset must not overflow the comparison
domain:** with `OFFSET = 2^63` and `value ∈ [−2^63, 2^63)`, `v_field ∈ [0, 2^64) < p`
(`p ≈ 2^254`), so no wrap; the comparison stays in the shifted unsigned domain
(`signed_verdict`, `filter_signed.nr:70-87`) so no subtraction underflows. (c) **Decimal
scale:** host pre-scales the bound and the circuit range-checks `int·10^FD + frac < 2^64`
(as `filter_decimal_check` does today).

### 6.5 NEW risk: issuer/verifier encoding disagreement (versioning)

If the issuer's `encode_typed_value` and the circuit's `filter_value_check` recombination
diverge by one byte/order/domain-tag, every proof silently fails (fail-closed, not a
forgery) — but a *partial* migration (some graphs single-lane, some dual-lane) risks a
verifier accepting the wrong lane.
**Closure.** (a) **Single source of truth** for the datatype-tag table, the `VALUE_DOMAIN`
constant, the `OFFSET`, and the field-truncation contract — exactly as `hashes.nr`
already pins "MUST stay bit-compatible with `crates/sparq-zk`" (`hashes.nr:1-3`) and
`join.nr:73-79` pins the join domain tag identical host↔circuit. Add a Rust↔Noir
constant-parity test (the repo's convention). (b) **Lane is explicit in the manifest:**
`CircuitId::FilterValue` vs the legacy `CircuitId::FilterInt` — the verifier knows which
lane a sub-proof binds; a single-lane graph cannot be filtered with a value-lane proof
(its leaves carry no `Enc_val`, so the recompute is unsatisfiable). (c) **Toolchain pin**
unchanged (nargo `1.0.0-beta.21` / bb `5.0.0-nightly.20260324`); a `blake3`-vs-Poseidon2
truncation contract change is the kind of thing the bb public-input byte layout is
sensitive to.

### 6.6 Residual / unchanged risks

The value lane changes nothing about the *unaudited* status of the composition verifier
itself (`sq-qhy4`/`sq-9hrn`): scan completeness, attribution, replay/freshness, issuer
set-membership, and revocation are all unchanged and carry their existing caveats. The
join's value-equality semantics (§2.3) are a *behavioural* change (value-equal vs
term-equal) that the query layer and the `recheck` path must agree with — flag for the
formal-semantics review (`sparql-formal-semantics`).

---

## 7. Open questions for the maintainer

1. **Leaf shape** (§3.4): combined-leaf (preferred), sibling-leaf, or reference-faithful inner `h4`? Affects scan recompute and `C(G)` arithmetic.
2. **Value-equality join semantics** (§2.3, §6.6): is moving the numeric join from term-equality to value-equality desired now, or should the value lane bind FILTER only and leave JOIN on the string lane for v1?
3. **Datatype tag table scope:** which datatypes are in v1 (proposal: `integer`+derived, `decimal`, `double`/`float`, `boolean`, `dateTime`/`date`)? Each is a compiled member.
4. **Decimal scale negotiation:** fixed per-member `FD` (as today) vs a per-credential declared scale folded into `dt_field` — the latter generalises but widens the tag space.
5. **Per-triple selective disclosure:** keep named-graph granularity (this design), or is the `jeswr/queryable-credentials` per-element salted typed-value hashing + Merkle rollup wanted so a *single* typed value can be disclosed/range-proved independently of its graph (§4)? That is the larger, reference-faithful path.
6. **Dual-write cost at ingest:** the value lane adds host hashing at issuance — acceptable, or should it be lazy (computed only for value-bearing literals, which §2.2 already does)?
