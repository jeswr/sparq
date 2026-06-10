# Engine findings from the W3C SPARQL conformance run (T13)

## Round-2 update (conformance-round2 branch)

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
