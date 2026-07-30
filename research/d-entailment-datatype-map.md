# D-entailment: datatype-map broadening + shared value-space comparator adoption (sq-pbz04.6) [FABLE-5]

> **Status: DESIGN + DECOMPOSITION RECORD.** Authored by Claude Fable 5 as the
> architecture pass for epic **sq-pbz04.6** (parent sq-pbz04, program sq-6tykl,
> program record `research/reasoner-federation-program.md`). Design and
> decomposition only — the implementation is fleet work tracked in the child
> beads listed in §5. 🤖 SPARQ agent.
>
> Builds on: `research/reasoner-federation-program.md` (seam 2 / F6),
> `research/shared-eval-substrate.md` §2.1 (the value-space carve-out that became
> sq-v5evr), and the in-code design notes of `crates/sparq-reason/src/dtype.rs`.

## 1. Verified current state (what is BUILT, checked against the code)

`Profile::D` ships in `sparq-reason` behind the opt-in `d-entail` feature
(`crates/sparq-reason/src/dtype.rs`, wired through
`crates/sparq-conformance/src/inference/entail.rs` and the ratcheted lane
`crates/sparq-conformance/tests/d_entail_suite.rs`, floor `D_ENTAIL_FLOOR`
mirrored in `tests/scoreboard_floors.rs`). Verified inventory:

> **[GPT-5] Historical snapshot:** the 23-datatype inventory below records the
> PRE-broadening state verified when this design was written. After D2,
> `Recognized::standard()` is table-driven and contains 30 datatypes; do not
> re-propagate 23 as the current count.

- **rdfD1 typing closure** (`materialize_d`): recognized + well-formed +
  value-mapped literals get `"l"^^d rdf:type d`; ill-formed literals are
  correctly NOT typed (clash is the inconsistency checker's concern);
  idempotent; inline small-integer ids handled; language-tagged literals
  skipped. This part is sound as scoped.
- **Recognized map** (`Recognized::standard()`): 23 datatypes — the string trio
  (`string`/`normalizedString`/`token`), `boolean`, the 13 integer types,
  `decimal`, `double`, `float`, and the temporal trio
  (`dateTime`/`dateTimeStamp`/`date`).
- **Value keys** (`d_value_key` → local `DValue`): canonical-decimal STRING for
  the integer/decimal space (deliberately not an f64 fast path — the module doc
  states why, and a unit test pins the 2^53+1 non-aliasing guard); IEEE bit
  patterns for `double`/`float` (distinct spaces); `Temporal` instant + family
  tag for the temporal trio (keeps `date` and `dateTime` value spaces disjoint).
- **Honest floor**: the W3C `sparql11/entailment` corpus contains a *single*
  D-only test at the pinned revision (`D_ENTAIL_FLOOR = 1`). The real coverage
  lives in the `dtype` unit tests; the lane doc says so explicitly.

**Two facet-soundness gaps found by this pass** (both let rdfD1 type a literal
that is OUTSIDE its datatype's lexical/value space — a soundness bug, since a
D-interpretation gives such a literal no value and rdfD1 must not fire):

1. **Bounded-range facets are not checked.** `integer_subtype_ok` enforces only
   the sign facets (`nonNegativeInteger` etc.). `"200"^^xsd:byte`,
   `"70000"^^xsd:short`, `"4294967296"^^xsd:unsignedInt`, out-of-range
   `xsd:long`/`int` values all parse as `i128`, fall through the
   `_ => true` arm, and are typed. (The in-file test even *comments* the byte
   case without asserting it.)
2. **String-family facets are not checked.** `normalizedString` (no
   `\t`/`\n`/`\r`) and `token` (collapsed: additionally no leading/trailing
   space, no double spaces) are keyed as raw `Str(lex)` unconditionally, so
   `" a"^^xsd:token` is treated as well-formed and typed.

These are wave-1 bug fixes (bead D1 below), independent of everything else.

## 2. The value-space equality trap (the #1 D-entailment unsoundness)

D-entailment's load-bearing semantics is **value-space** equality, not lexical
equality: `"1"^^xsd:integer`, `"01"^^xsd:integer` and `"1.0"^^xsd:decimal`
denote the SAME value; `"1"^^xsd:integer` and `"1.0"^^xsd:double` do NOT
(disjoint primitive value spaces). Getting either direction wrong is unsound —
aliasing distinct values manufactures entailments; splitting equal values
loses them *and* breaks literal interchangeability. The existing code handles
the known traps correctly and the design keeps them pinned:

- **integer ⊂ decimal**: one shared canonical-decimal key (exact at any
  magnitude; the f64 fast path is rejected in the module doc — `2^53 + 1`
  aliases under f64).
- **float/double vs decimal**: distinct primitives, distinct keys, even at
  numerically "equal" lexicals (`"1.0"^^xsd:decimal ≠ "1.0"^^xsd:double`).
- **date vs dateTime**: disjoint value spaces; the key carries a family tag so
  a shared instant never key-equals across families; tz-presence is part of
  the key so floating vs zoned stays distinguishable.
- **Canonical-form edge cases**: `-0` ≡ `0` (decimal), `"true"` ≡ `"1"`
  (boolean), NaN bit-canonicalization for float/double.
- **Well-formedness precedes value**: an ill-formed literal has NO value —
  facet validation (§1 gaps) is therefore part of the equality story, not a
  side quest.

Every datatype added in §3 must state which value space it maps into and which
existing space (if any) it coincides with; "we can't state that yet" routes the
datatype to the deferral ledger, not to the map.

## 3. Datatype ledger — what broadens the map, what is deferred, and why

### 3.1 ADD (sound value mapping available; bead D2)

| Datatype | Value key | Soundness argument |
|---|---|---|
| `xsd:anyURI` | NEW `Uri(String)` tag, DISJOINT from `Str` | Lexical space validated (no facet beyond string shape); value = the character sequence, equality = codepoint equality (`"a b"` ≠ `"a%20b"` — no escaping normalization, per XSD 1.1 §3.3.17). Keyed disjoint from `xsd:string` conservatively: XSD 1.1 defines anyURI as its own primitive; if its value space is later shown identical to string's, disjoint keying is INCOMPLETE (misses cross-datatype equality) but never unsound. |
| `xsd:language`, `xsd:Name`, `xsd:NCName`, `xsd:NMTOKEN` | `Str(lex)` after PATTERN-facet validation | All derived from `xsd:token` by restriction ⇒ their values ARE string values ⇒ sharing the `Str` key with the string trio is exact. Pattern facets per XSD 1.1 (`language`: `[a-zA-Z]{1,8}(-[a-zA-Z0-9]{1,8})*`, case-significant — `"EN"` ≠ `"en"` as VALUES; `Name`/`NCName`/`NMTOKEN` per the XML grammars). Ill-formed ⇒ no key ⇒ not typed. |
| `xsd:hexBinary`, `xsd:base64Binary` | shared `Octets(Vec<u8>)` key | XSD 1.1 §3.3.15/§3.3.16 define BOTH value spaces as "finite-length sequences of binary octets" — i.e. the SAME set — so equal octet sequences are equal D-values across the two datatypes (the binary analogue of integer ⊂ decimal). Decoding follows the XSD lexical grammars (hex case-insensitive digits; base64 with the XSD whitespace allowance); ill-formed ⇒ no key. **Implementer duty**: verify + cite the two spec sentences in the doc comment; if the identical-value-space reading does not survive that check, fall back to per-datatype disjoint tags (incomplete-but-sound) and note it here. |

`Recognized::standard()`, `has_value_mapping` and `d_value_key` currently
maintain three parallel datatype lists; D2 refactors them onto ONE table so map
membership and value mapping cannot drift.

### 3.2 DEFER (no sound value mapping available today; documented, not built)

| Datatype | Reason it is deferred (honest incompleteness, not oversight) |
|---|---|
| `xsd:time` | `sparq-core::temporal` deliberately excludes it (the engine compares `xsd:time` lexically — a doc-comment invariant in `temporal.rs`). A value mapping needs the reference-day model plus the `24:00:00` ≡ `00:00:00` and floating-vs-zoned rules; adding it by value in D while the engine stays lexical would break the §4 parity chain. Revisit only together with an engine/temporal upgrade. |
| `xsd:gYear`/`gMonth`/`gDay`/`gYearMonth`/`gMonthDay` | No parser in `sparq-core`; timezone offsets make ordering partial and equality subtle; negligible corpus incidence. |
| `xsd:duration`, `yearMonthDuration`, `dayTimeDuration` | Equality is definable ((months, seconds) pair) but there is no parser and no consumer; the value space is famously partially ordered (`P1M` vs `P30D`), so a naive relational mapping is unsound. |
| `rdf:XMLLiteral`, `rdf:HTML` | Value mapping requires DOM/C14N canonicalization — a heavyweight dependency for near-zero test coverage. |

Not deferred but EXCLUDED: `xsd:QName`, `xsd:ENTITY`/`ENTITIES`, `xsd:ID`,
`xsd:IDREF`/`IDREFS`, `xsd:NOTATION` and the list types are outside the
RDF-compatible XSD subset (RDF 1.1 Concepts §5.1) — they never enter D.

An unrecognized or unmapped datatype keeps today's fail-closed behaviour: no
typing, no clash, no value claim.

## 4. The substrate seam — coordination with sq-v5evr

`sq-v5evr` (first-wave, `sparq-substrate`, sonnet) hoists the value-space
EQUALITY/relational comparator out of the engine behind a new default-off
substrate feature; its own acceptance is a substrate-vs-engine differential
over the XSD matrix. D-entailment is the second consumer that un-parked it.
The seam this record fixes:

- **What migrates (bead D3, after sq-v5evr lands):** `d_value_key`'s numeric
  (canonical decimal + IEEE), boolean, string and temporal value comparison
  delegates to the substrate comparator; the local `canon_decimal` /
  `parse_xsd_double` duplicates are deleted (the module doc already promises
  exactly this). Behaviour-neutral by construction: the entailment ratchet and
  the `dtype` unit-test matrix must be byte-identical before/after.
- **What STAYS dtype-resident (by design, not omission):** datatype-map
  membership (`Recognized`), facet validation (§1/§3.1), the rdfD1
  well-formedness judgment, and the D-specific key families the engine does
  not value-compare (`Octets`, `Uri`, the pattern-validated string family).
  dtype.rs becomes "datatype map + facets + rdfD1", not a second comparator.
- **Parity is compositional:** the epic's acceptance ("differential parity
  with the engine's value semantics over the XSD matrix") decomposes as
  dtype ≡ substrate (D3's differential test) ∘ substrate ≡ engine (sq-v5evr's
  differential test). No `sparq-engine` dev-dependency enters `sparq-reason`,
  preserving the crate's `sparq-core`-only dependency posture.
- **Do not over-claim the API:** sq-v5evr's exact surface is not yet landed.
  D3 is specced against the CONTRACT (equality + relational verdicts over
  typed values covering the numeric/boolean/string/temporal spaces); anything
  the landed API does not cover stays local and is noted here.

## 5. Decomposition — child beads (waves, tiers, exclusive files)

All beads carry `{crate, model_tier, invariant, acceptance_test, exclusive
file footprint}` in the tracker; the table is the summary. Beads sharing
`dtype.rs` are SERIALIZED by `bd dep` (the epic is one-file-centric; the win
here is sequencing + tiering, not width). D4 runs in parallel with D3
(disjoint files, both behaviour-compatible with the ratchet).

| Bead | Wave | What | Crate / exclusive files | Tier |
|---|---|---|---|---|
| D1 (bug) | 1 | Facet soundness: bounded-range facets for derived integers + string-family facet validation (§1 gaps) — rdfD1 never types an ill-formed literal | `sparq-reason` / `src/dtype.rs` only | sonnet |
| D2 | 2 (dep: D1) | Broaden the map per §3.1 (`anyURI`, `language`/`Name`/`NCName`/`NMTOKEN`, `hexBinary`/`base64Binary`) + single-table refactor + §3.2 deferral ledger in module docs | `sparq-reason` / `src/dtype.rs` only | sonnet |
| D3 | 3 (dep: D2, sq-v5evr) | Adopt the substrate value-space comparator per §4; delete local canonicalization; behaviour-neutral | `sparq-reason` / `src/dtype.rs`, `Cargo.toml` (one feature line) | opus |
| D4 | 3 (dep: D2) | Conformance: crate-local D value-space matrix arm (sparq-extension floor, tallied separately from the W3C count, mirroring the QL-oracle precedent) + broadened-map end-to-end cases | `sparq-conformance` / `tests/d_entail_suite.rs`, `tests/scoreboard_floors.rs` | sonnet |
| D5 (chore) | 4 (dep: D3, D4) | Docs: datatype-map table + deferral ledger + value-vs-lexical distinction in the crate README + `skills/inference/SKILL.md` | docs / `crates/sparq-reason/README.md`, `skills/inference/SKILL.md` | haiku |

Tier rationale: D3 is the value-space-equality soundness piece (subtle
byte-identical-ratchet invariant across a cross-crate seam) → opus. D1/D2/D4
have crisp mechanical oracles (the XSD 1.1 facet/value-space tables, the
ratchet) with the soundness judgments already made in §3 → sonnet. D5
transcribes this record → haiku.

Cross-epic contention note: `crates/sparq-reason/Cargo.toml` is also touched
by sq-pbz04.5.3 (`rif-xml` feature) and `tests/scoreboard_floors.rs` by
sq-pbz04.5.5 — the one-bead-per-crate-in-flight scheduling discipline (or a
trivial rebase) covers both; noted so the scheduler sees it.

## 6. Non-goals and honesty ledger

- **Named-graph × entailment interaction is out of scope** — tracked by
  sq-6qpyf / sq-oy1f.21 (the epic says do not duplicate; this record doesn't).
- **No W3C-floor inflation promise.** The D-only W3C corpus is 1 test; D4's
  new coverage is a sparq-EXTENSION floor, tallied separately per the
  program's honesty rule 4 — the standards count is not padded.
- **Deferrals are documented incompleteness, not support.** §3.2 datatypes
  stay fail-closed (no typing, no equality claim). That is a valid terminal
  outcome, not debt to be silently "fixed" by a lexical-equality shortcut.
- **Relational (`<`/`>`) semantics beyond equality** is exercised only where
  the substrate comparator provides it (post D3); D-entailment itself needs
  equality — no ordering claim is made for the `Octets`/`Uri`/pattern
  families.
- **No performance numbers in this record**; behaviour-neutrality is gated on
  the named ratchets, not on timings.
- Everything in §1 marked BUILT was verified by reading the code, not the
  epic's framing; the two §1 gaps are the corrected premise this pass adds.
