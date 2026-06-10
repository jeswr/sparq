# Engine findings from the W3C SPARQL conformance run (T13)

## Round 3 — F19/F14/F15/F16/F17/F18 fixed (branch `conformance-round3`)

Headline at sparq `454593c`+round-3 / rdf-tests `f25dbc0`, full scope (1229 tests):
**1205 pass / 11 fail / 13 skip — 99.1% of run** (branch point: 1092/73/64, 93.7%).

| group | before (454593c) | after | note |
|---|---|---|---|
| SPARQL 1.0 query eval | 268 / 3 / 12 | 268 / 3 / 12 | unchanged (skips are FROM/FROM NAMED) |
| SPARQL 1.1 query eval | 220 / 4 / 1 | 222 / 2 / 1 | +2: empty named graphs now registered |
| SPARQL 1.1 update eval | 23 / 20 / 51 | **94 / 0 / 0** | F19: suite is 100%, zero skips |
| 1.1 result formats | 6 / 1 / 0 | 6 / 1 / 0 | tsv03 = F21 (lexical preservation) |
| SPARQL 1.2 eval | 25 / 41 / 0 | **65 / 1 / 0** | F14–F18; last fail is upstream-parser F20 |
| syntax (1.0/1.1/1.2) | 550 / 4 / 0 | 550 / 4 / 0 | all F20 (spargebra 0.4.6 posture) |

### What was fixed (round 3)

- **F19 (named-graph UPDATE data loss)** — `update.rs` v2 models the dataset as
  per-graph term-triple sets and implements every `GraphUpdateOperation`: GRAPH-scoped
  INSERT/DELETE DATA (auto-creating targets), DELETE/INSERT…WHERE with GRAPH templates
  (incl. variable graph names), fresh-per-solution blank nodes with request-unique
  labels, triple-term templates, USING/USING NAMED (note: spargebra encodes `WITH g` as
  `using {default:[g], named: None}` — `named: None` must KEEP the store's named graphs,
  `Some(list)` replaces them), CLEAR/DROP/CREATE (ADD/COPY/MOVE arrive pre-desugared as
  DROP + DELETE/INSERT), LOAD for `file://`. `update_in_place` keeps O(batch)
  delta-overlay semantics per target graph. Optional-failure ops (CLEAR/DROP absent,
  CREATE existing) are no-ops — auto-create store semantics, which the suite accepts.
- **F14 (variables in quoted-triple patterns)** — BGP slots holding variable-carrying
  quoted patterns are rewritten to synthetic `#qt#N` variables plus constraint
  relations: enumerate the dictionary's `TermParts::Triple` records (only quoting
  queries pay the scan), structurally unify componentwise (recursing through nesting,
  repeated-variable consistency), then join through the ordinary machinery so inner
  variables join with outer patterns. Wired into BOTH `eval_bgp` and `eval_bgp_binary`
  (the conjunctive-flattening path calls the latter directly).
- **F15 (1.2 builtins)** — TRIPLE/isTRIPLE/SUBJECT/PREDICATE/OBJECT,
  hasLANG/hasLANGDIR/LANGDIR/STRLANGDIR.
- **F16** — `=` on triple terms is componentwise with VALUE equality on objects
  (errors propagate); ORDER BY total order: triple terms sort AFTER literals,
  componentwise within (subject compared through the same total order — raw IRI
  strings, not the `<>`-wrapped Display form).
- **F17** — EBV of rdf:langString/dirLangString is a type error (`!!"a"@en` unbound).
- **F18 + storage** — the CORE dictionary preserves RDF 1.2 base direction by storing
  `lang--dir` in the language slot (hash/equality/layout unchanged; nt.rs types such
  literals rdf:dirLangString); the engine's `str_lit`/`lit_with_lang` carry the same
  combined slot, which yields the 1.2 CONCAT rules (identical lang+dir kept, any
  mismatch drops to a SIMPLE literal) and direction preservation through
  SUBSTR/UCASE/REPLACE/STRBEFORE/STRAFTER for free.
- **Harness** — explicitly-named-but-empty graphs (`qt:`/`ut:graphData` with an empty
  document) are registered after the N-Quads load (N-Quads cannot encode them), fixing
  the two round-2 query failures needing `GRAPH ?g {}` over empty graphs.
- CONSTRUCT templates instantiate triple-term objects recursively (4 tests).

Perf guard (vs `454593c`, same machine): `update_in_place` 10-triple INSERT into a 2M
graph 15.2µs → 15.9µs mean-of-3 (+4.6%, within the 5% guard — the per-quad graph-slot
routing); ci-bench query latencies all ≥ parity (noise-dominated, branch faster in
every pairing); store 100 B/triple and dict 84 B/term unchanged; wasm bundle
1,511,742 → 1,546,616 B (+2.3% — the named-graph update + 1.2 matching/builtins code,
not a leaked feature).

### The 11 remaining failures (all pre-diagnosed, none regressions)

- 5× **F20 upstream spargebra 0.4.6** (4 syntax-negative + the 1.2 grouping eval test
  that fails at parse) + *case-insensitive booleans* (F13).
- 3× **lexical-form preservation / suite-convention conflicts**: *xsd:decimal cast*,
  *tsv03* (F21), *SUM DISTINCT with GROUP BY* (canonical-vs-plain double, see round 2).
- *"/" on mixed datatypes* (suites disagree on integer-division scale, round-2 note).
- *dawg-optional-filter-005-not-simplified* (algebra-duality, round-2 note).

The 13 remaining skips are all **FROM / FROM NAMED** on queries — the engine still
drops `Query::Select { dataset }`; wiring it through the same active-dataset builder
`update.rs` now has (`Dataset::build_using`) is the obvious next lever, and would also
fix the latent run-against-wrong-dataset bug for users.

## Scope extension — full SPARQL 1.1 + 1.2 coverage (branch `conformance-12`)

The runner now covers the WHOLE prioritised target: 1.0/1.1 evaluation as before,
plus the **SPARQL 1.2 suites** (`sparql12/`: triple-term query+update evaluation,
expression, lang-basedir, grouping, rdf11, codepoint-escapes), the **1.1 result-format
suites** (`csv-tsv-res` TSV + `json-res`), and ALL **syntax suites** (1.0/1.1/1.2,
positive = spargebra must parse, negative = it must reject). Still not run:
protocol, SERVICE evaluation, entailment (report footer). Scoreboard at sparq
`93cec32` / rdf-tests `f25dbc0`, 1229 tests run:

| group | pass | fail | skip | pass-rate (of run) |
|---|---:|---:|---:|---:|
| SPARQL 1.0 query evaluation | 224 | 42 | 17 | 84.2% |
| SPARQL 1.1 (query / update / result formats) | 214 | 54 | 58 | 79.9% |
| SPARQL 1.2 evaluation | 24 | 35 | 7 | 40.7% |
| Syntax 1.0+1.1+1.2 (spargebra posture) | 550 | 4 | 0 | 99.3% |
| **overall** | **1012** | **135** | **82** | **88.2%** |

Harness extensions behind the new visibility (all in this crate): syntax-test mode;
TSV expected results; RDF 1.2 triple terms decoded in `.srj`/`.srx` and matched with
blank-node bijection inside triple terms; TriG/N-Quads test data loaded as datasets;
UPDATE comparison is now **quad-based over the full dataset** (named graphs included)
instead of default-graph-only — which unmasked F19 below.

### NEW failure categories (round 2)

#### F14. Variables inside triple-term patterns — 14 tests, the dominant 1.2 blocker
`engine error: variable where a term was expected` (13×) and the explicit
`variable inside a triple-term pattern is not yet supported (T6)` (1×). Kills most of
`sparql12/eval-triple-terms`: every `<< ?s ?p ?o >>` / reifier-pattern test with a
variable in any slot, including the GRAPH and annotation-syntax variants. The suite
exercises: var in subject/predicate/object slot, nested triple terms with vars, same
variable repeated, vars under GRAPH, and annotation sugar `:s :p :o {| :q :z |}`.

#### F15. SPARQL 1.2 builtin functions missing — 12 tests
- `TRIPLE()` (5×) and `isTRIPLE()` (1×) → `unsupported SPARQL function: Triple/IsTriple`
  (SUBJECT()/PREDICATE()/OBJECT() are exercised inside the same queries).
- New direction/language builtins: `hasLANG`, `hasLANGDIR`, `LANGDIR` (2×),
  `STRLANGDIR` → `unsupported SPARQL function: HasLang/HasLangDir/LangDir/StrLangDir`
  (5× in `sparql12/lang-basedir`).

#### F16. Triple-term VALUE semantics — 4 tests
Constant triple-term machinery works (12 passes incl. all-graph-triples dumps and
constant matches), but: `=` value-equality between triple terms whose inner literals
are value-equal-but-not-identical (`01` vs `1`) fails; ORDER BY does not implement the
SPARQL 1.2 total order extension (triple terms sort AFTER literals; `Embedded triple -
ORDER BY` / `ordering` put them elsewhere); `Pattern - Nesting 1` loses a doubly-nested
match.

#### F17. EBV type-error propagation (1.2 `expression/not-not`) — 1 test
`!!?v` over `"a"@en`, `"z"^^xsd:boolean`, ill-typed numerics etc. must leave `?ebv`
UNBOUND (EBV type error) for non-EBV-able terms; the engine binds true/false instead.
Same root cause as round-1 F5/F8 (errors-as-values), now with 1.2's expanded vector.

#### F18. `rdf:dirLangString` handling — 1 test (+5 blocked behind F15)
`CONCAT` of two dir-lang strings with the same lang but different/absent base
direction must drop to plain `xsd:string`/`langString` per the 1.2 rules; engine keeps
`@en` where it must not (`CONCAT and rdf:dirLangString`).

#### F19. UPDATE silently DROPS named graphs — ~20 tests (1.1 update + 2× 1.2)
Previously masked by harness skips; the quad-aware comparison exposes it:
`sparq_engine::update` rebuilds the result from `current_triples(graph)` (default
graph only) and returns a `Graph` whose `named` is EMPTY, so any pre-existing named
graph is lost even when the operation itself succeeds (e.g. `CLEAR DEFAULT` returns a
fully empty dataset; `ADD`/`COPY`/`MOVE`/graph-specific `DELETE` lose `:g1`/`:g2`).
This is a *data-loss* class, distinct from the explicit "named graphs not yet
supported" errors (which remain honest skips). The 1.2 `Reified triples - Update`
tests fail the same way (`INSERT { << ?s ?p ?o >> :source ?g } WHERE { GRAPH ?g … }`
— the inserted default-graph triples are right, the named graphs vanish).

#### F20. Upstream spargebra 0.4.6 posture (syntax suites + 1 eval) — 5 tests
The syntax suites measure the parser dependency; sparq inherits these:
- accepts nested aggregate functions (`sparql12/syntax` negative test);
- accepts a literal / triple term in the SUBJECT position of a triple term in
  *expressions* (2× `syntax-triple-terms-negative`);
- accepts `sparql10/syntax-sparql3` `syn-bad-26.rq`;
- REJECTS 1.2's now-legal reuse of a SELECT expression variable in an aggregating
  query (`sparql12/grouping` evaluation test fails at parse).
Everything else parses cleanly: 550/554 — including all 113 positive triple-term
documents, VERSION declarations and codepoint escapes.

#### F21. Lexical forms of data terms not preserved — 1 test (`csv-tsv-res` tsv03)
`"1.0e6"^^xsd:double` from the data comes back as `"1.0E6"` — the store normalises
numeric lexical forms instead of preserving them; SPARQL requires bound values to keep
the original lexical form. (Adjacent to round-1 F4 but the inverse: F4 wants
*canonical* forms for computed values, F21 wants *preserved* forms for data values.)

Other 1.2 notes: `sparql12/rdf11` (singleton bnode graphs, plain-vs-xsd:string) passes
3/3; codepoint-escape evaluation passes 4/4 (+1 CONSTRUCT skip); 1.2 CONSTRUCT tests
(6) skip on the round-1 CONSTRUCT gap; `sparql12/version` is syntax-only and passes.

---

## Round 1 (historical) — original notes below

## Engine round 2 — fixes (branch `conformance-round2`)

Headline after the round-2 engine batch: **509 pass / 7 fail / 86 skip over 602
evaluation tests — 98.6% of executed tests pass** (was 441/75/86 at the branch
point). Fixed in `crates/sparq-engine` (commits on `conformance-round2`):

- **F4 numeric type promotion + canonical lexical forms** — `Value::Num` is now a
  typed tower `Num::{Int(i64), Dec(exact fixed-point), Float(f32), Double(f64)}`
  with the XPath promotion table, exact integer/decimal arithmetic and division
  (int/int → decimal; ÷0 on exact types is a type error), datatype-preserving
  ABS/CEIL/FLOOR/ROUND, SECONDS() → xsd:decimal, typed SUM/AVG/MIN/MAX (SUM/AVG
  error → unbound on a non-numeric/errored member). Comparison fast paths
  (numeric_value f64 cache, sargable scans) untouched.
- **F2 XSD constructor casts** — full cast table for xsd:{integer,decimal,float,
  double,string,boolean,dateTime}; plus NOW() and RAND() (native-only, like UUID).
- **F8 open-world `=`** — family-based equality (sameTerm decides positively;
  unknown datatypes / ill-formed lexicals / cross-family pairs are TYPE ERRORS;
  lang-tagged vs non-lang is decided-false), timeline-valued xsd:date/xsd:dateTime
  comparison (offset normalisation, 24:00:00 rollover, XSD ±14h indeterminacy);
  EBV is genuinely 3-valued (`!?w` over an unknown type stays an error).
- **F7 GRAPH semantics** — `GRAPH <absent> {}` yields 0 solutions; a graph
  variable bound INSIDE the pattern joins instead of duplicating the column.
- **F9–F12 stragglers** — FILTER group scoping in the conjunctive flattening
  (filter-nested-2), REGEX `q`/`iq` flags, zero-length property paths on absent
  constants, SPARQL total order for ORDER BY (unbound < bnode < IRI < literal),
  IRI()/URI() relative-reference resolution against the query BASE.

### The 7 remaining failures are blocked outside `sparq-engine`:

1. *case-insensitive booleans* — upstream `spargebra` 0.4.6 parse error (F13).
2. */ operator on number mixed datatypes* — expects `3/3 = "1"^^xsd:decimal`
   while sparql11/functions COALESCE expects `0/2 = "0.0"` and aggregates AVG
   expects `6/3 = "2.0"`: the expected files disagree on integer-division
   scale; the engine follows the (larger) sparql11 convention.
3. *dawg-optional-filter-005-not-simplified* — zero-sum with its `-simplified`
   twin: spargebra hoists the nested filter into the LeftJoin expression, which
   matches the simplified reading; satisfying both needs algebra-level control.
4. *COUNT: no GROUP BY inside of GRAPH* and 5. *VALUES inside GRAPH binding the
   same variable as the graph name* — both need EMPTY named graphs to exist in
   the dataset; the harness loads test data via N-Quads (`Graph::load_dataset`),
   which cannot represent a named graph with zero triples. Needs a core/harness
   path that registers `qt:graphData` graph NAMES explicitly.
6. *xsd:decimal cast* — the expected file contains parser-NORMALISED data terms
   (`"0E1"^^xsd:double` in data appears as `"0.0"^^xsd:double` in results), so
   it cannot pass under term-equality without a normalising loader.
7. *SUM DISTINCT with GROUP BY* — expects `"2100"^^xsd:double` (plain) while
   *SUM with GROUP BY* expects `"3.21E4"^^xsd:double` (canonical scientific)
   for the same construct; the engine follows the canonical convention.


First full run of `sparq-conformance` against w3c/rdf-tests @ `f25dbc092c654d792974848e81bb519d7328f0e8`
(sparq @ `d555096`). Headline: **344 pass / 110 fail / 148 skip over 602 evaluation
tests — 75.8% of executed tests pass** (data-r2 query 85.3%, data-sparql11 query 62.4%,
update 100% of the executed subset). No engine panics or timeouts were observed.

This crate must not touch other crates' source, so everything below is RECORDED here
as follow-up work rather than fixed. Reproduce any item with
`cargo run -p sparq-conformance -- --filter <suite-or-test-name> --verbose`
(after `scripts/fetch-conformance.sh`). The per-test diff samples quoted come from
`conformance-report.md`.

## Coverage gaps (cause SKIPs, not failures)

1. **Query forms: only SELECT is supported** (`sparq_engine::query` rejects the rest).
   ASK = 51 skipped tests, CONSTRUCT = 12, DESCRIBE present in data-r2 too. The single
   biggest conformance lever: ASK alone is `exec::count_select(..) > 0`, and the whole
   `sparql10/type-promotion` (30 tests) + `sparql10/ask` + half of open-world are ASK.
2. **FROM / FROM NAMED is silently dropped**: `query()` matches
   `Query::Select { pattern, .. }` and discards `dataset`. 12 tests skipped by the
   harness — but note this is a *latent correctness bug* for users: a query with FROM
   runs against the wrong dataset instead of erroring.
3. **Updates touching named graphs** are unsupported (67 skips): `GRAPH` in
   INSERT/DELETE templates, `WITH`, `USING`, and ADD/MOVE/COPY/DROP/LOAD/CLEAR of named
   graphs (engine returns "not yet supported", harness records a skip). All 21
   executable default-graph update tests PASS — default-graph update semantics look solid.
4. Engine API takes no base IRI for query/data parsing; the harness compensates by
   prepending `BASE <file://…>`. A `with_base_iri` style option on
   `sparq_engine::query` / `Graph::load_str` would remove the workaround.

## Failure categories (110 FAILs)

### F1. EXISTS / NOT EXISTS in FILTER unimplemented — 15 tests
`M2: unsupported expression: Exists(..)`. Kills `sparql11/exists` (0/6),
most of `sparql11/negation` (8 fails, e.g. *Subsets by exclusion (NOT EXISTS)*), and
`sparql11/subquery` *sq10*.

### F2. XSD constructor casts unimplemented — 16 tests
`xsd:integer(?x)`, `xsd:decimal`, `xsd:float`, `xsd:double`, `xsd:string`,
`xsd:boolean`, `xsd:dateTime` as cast functions → `unsupported SPARQL function:
Custom(xsd:*)`. Wipes out `sparql10/cast` (0/7) and `sparql11/cast` (0/6), plus
*Function sort*, *GROUP BY with a function*, *Protect from error in AVG*.

### F3. Missing builtins — 15 tests
MD5 / SHA1 / SHA256 / SHA384 / SHA512 (10 tests), TIMEZONE(), TZ(), BNODE(),
BNODE(str), UUID(), STRUUID().

### F4. Numeric type promotion & canonical lexical forms — ~25 tests
The engine computes arithmetic/aggregates in f64 and stamps the result `xsd:double`
(or `xsd:integer`), instead of SPARQL's operand type promotion
(integer→decimal→float→double) and XSD canonical lexical forms. Examples:
- *SUM*: expected `"11.1"^^xsd:decimal`, got `"11.100000000000001"^^xsd:double`.
- *AVG*: expected `"2.22"^^xsd:decimal`, got `"2.22"^^xsd:double`.
- *MIN with GROUP BY*: expected `"2.0E-1"^^xsd:double`, got `"2E-1"^^xsd:double`
  (canonical double form requires a digit after the point).
- *CEIL()/FLOOR()/ROUND()* must preserve the argument datatype: expected
  `"3"^^xsd:decimal` for `CEIL("2.5"^^xsd:decimal)`, got `"3"^^xsd:integer`.
- `sparql10/expr-ops` *+,-,*,/ on mixed datatypes* (4 tests), *Unary Minus*,
  *plus-1-corrected* (`1.0 + 2` must be `"3.0"^^xsd:decimal`, got `"3"^^xsd:integer`),
  *COALESCE()* (`/0` should be type error → `"0.0"^^xsd:decimal` case, got
  `"NaN"^^xsd:double` / `"0"^^xsd:integer`), *SECONDS()* (must return `xsd:decimal`).

### F5. Expression type errors not propagated — ~6 tests
SPARQL type errors must make the expression error (binding left unbound / row
filtered), not produce a value:
- *IF() error propogation*: expected `{}` (unbound), got `?error="false"^^xsd:boolean`.
- *Error in AVG* / *Protect from error in AVG*: AVG over a group containing a
  non-numeric must error → group row with unbound aggregate.
- Integer division by zero must be an error, not NaN (see COALESCE above).

### F6. String functions drop language tags / argument typing — ~14 tests
- UCASE/LCASE/SUBSTR(2&3-arg)/CONCAT/STRBEFORE/STRAFTER must preserve the language
  tag (expected `"BAR"@en`, got `"BAR"`) and follow the argument-compatibility rules
  (STRBEFORE/STRAFTER datatyping tests).
- *STRDT()* / *STRLANG()*: must error unless the first argument is a simple literal
  (engine returned a value for `STRDT("bar"@en, …)`-style inputs; 4 tests incl. the
  16-row TypeErrors matrices).

### F7. GRAPH evaluation semantics — 7 tests
- *graph-not-exist*: `SELECT * { GRAPH ex:unknown {} }` must give **0** solutions
  (graph absent), engine returns the unit row.
- *graph-variable-join* (expected 1, got 3), *graph-optional* (expected 1, got 4),
  *sq02 subquery in GRAPH* (expected 1, got 2), *VALUES inside GRAPH binding the graph
  variable* (expected 3, got 2): variables bound inside `GRAPH ?g {…}` (including `?g`
  itself) don't join correctly with the surrounding pattern.
- *COUNT: no GROUP BY inside of GRAPH* (expected 2, got 1): implicit grouping
  interacts wrongly with GRAPH.

### F8. Open-world `=` / `!=` on unknown datatypes, date/dateTime comparison — 10 tests
- open-eq-04/06/08/10/11: `=` between literals of *unknown* datatypes with different
  lexical forms must be a type error (row filtered), engine answers true/false.
- open-eq-12: `!=` cases — expected 10 rows, got 0.
- *date-2* / *Equality with dateTime*: timezone-aware xsd:date/dateTime value
  comparison missing (e.g. `2006-08-23Z` vs `2006-08-23+00:00` are equal values).
- *boolean effective value - unknown types*: EBV of an unknown-datatype literal must
  be a type error (filter out), engine treats it as true.

### F9. FILTER scope — 2 tests
- *Filter-nested - 2*: `{ FILTER(?v = 1) }` in an inner group must not see `?v` from
  the outer group (expected 0 rows, got 1).
- *dawg-optional-filter-005-not-simplified*: FILTER inside OPTIONAL referencing an
  outer variable — engine keeps the OPTIONAL binding it should reject.

### F10. REGEX — 3 tests
- `q` / `iq` flags (literal-pattern mode, XPath F&O) unsupported by the `regex` crate
  backend — needs pattern escaping when `q` is present.
- *regex-query-003*: REGEX over an IRI must be a type error (string literals only);
  engine matched `?val=<http://example.com/uri>`.

### F11. Property paths: zero-length paths on absent terms — 4 tests
`<s> p* ?x` / `<s> p? ?x` with a constant endpoint must yield the zero-length
solution even when the term does not occur in the data (and on an empty graph).
Engine returns 0 rows (likely because the constant isn't in the dictionary —
`pattern()` returning `None` short-circuits the zero-length case).

### F12. ORDER BY total order — 3 tests
sort-3 / sort-6 / sort-8: same multiset, wrong sequence — the SPARQL ordering
(unbound < blank < IRI < literal, plus numeric-value ordering within compatible
types) isn't fully implemented for the mixed-type sort keys these tests use.

### F13. Upstream (spargebra 0.4.6) — 1 test
*case-insensitive booleans* (`SELECT (TRUE as ?t) (False as ?f) {}`) fails to parse:
`error at 1:23: expected ENCODE_FOR_URI`. Boolean keywords are case-insensitive in
the grammar; this is an upstream parser limitation, not a sparq bug.

## Harness notes (for whoever picks these up)

- Comparison is **term equality** under blank-node bijection, BAG semantics
  (SEQUENCE when the query has a top-level ORDER BY and the expected encoding
  preserves order). The suites expect canonical lexical forms, so F4's
  "same value, different lexical form" diffs are real conformance failures.
- Expected-result formats handled: `.srx`, `.srj`, `.ttl`/`.rdf` result-set graphs.
  `rdf:XMLLiteral` with nested markup inside `.srx` would be mis-read by the SRX
  parser (text-only accumulation) — no current failure traces back to this.
- Skips are listed per reason in `conformance-report.md`; nothing is silently dropped
  except non-evaluation entry types (syntax/protocol/CSV tests: 166 entries).
