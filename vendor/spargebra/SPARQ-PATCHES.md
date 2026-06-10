# sparq patches on top of spargebra 0.4.6

This tree started as a byte-identical copy of the `spargebra-0.4.6` sources from
crates.io (the only non-upstream change in the base commit is a `[workspace]`
table in `Cargo.toml`, required for the `[patch.crates-io]` path patch). Every
change below is a surgical parser fix, each verified against the W3C SPARQL
test suite (w3c/rdf-tests @ `f25dbc0`) and prepared as an upstream PR against
oxigraph/oxigraph — see `docs/upstream-proposals.md` at the repo root. Drop
this vendored tree once the fixes land in a released spargebra.

## 1. Reject nested aggregate functions

- **Spec**: SPARQL 1.2 Query §18.3.4.1 *Grouping and Aggregation* — the
  aggregation step replaces each aggregate `R(args; scalarvals)` found in
  SELECT/HAVING/ORDER BY with a fresh `agg_i`; no derivation in the algorithm
  (or in §11.4's projection conditions) admits an aggregate inside another
  aggregate's argument. The suite states it directly:
  `mf:comment "The expression argument of an aggregate function can not
  contain an aggregate function."`
- **Test**: `sparql12/syntax#nested-aggregate-functions` (NegativeSyntaxTest,
  `SELECT (COUNT(COUNT(*)) AS ?c) WHERE {}`) — accepted before, rejected after.
- **Fix**: `ParserState::new_aggregation` already replaces each parsed
  aggregate with a synthetic random-named variable. A nested aggregate
  therefore surfaces as one of those synthetic variables inside the argument
  of the aggregate being registered: walk the argument expression
  (`mentions_aggregate_variable`) and error out when one is found. Synthetic
  names are 128-bit random hex, so user variables cannot collide; sub-SELECTs
  push their own aggregate scope, so legal aggregates inside `EXISTS
  { { SELECT ... } }` are unaffected (the walker does not descend into
  `Expression::Exists`).

## 2. Reject literals / triple terms in the subject position of an expression triple term

- **Spec**: SPARQL 1.2 Query §19.7 grammar production
  `[138] ExprTripleTermSubject ::= iri | Var` (an RDF 1.2 triple term subject
  is an IRI or blank node, so the grammar excludes literals and nested triple
  terms from the subject slot). spargebra instead reused
  `ExprTripleTermObject` ([139], which allows `RDFLiteral | NumericLiteral |
  BooleanLiteral | ExprTripleTerm`) for the subject, accepting the invalid
  forms. The data (`TripleTermData`, [127]) and pattern paths already
  enforced the restriction — only the expression path was loose.
- **Tests**: `sparql12/syntax-triple-terms-negative#tripleterm-subject-03`
  (*Triple term in the subject position of a triple term (expression)*,
  `BIND( <<( <<(:s :p :o )>> :q :z )>> AS ?X )`) and `#tripleterm-subject-06`
  (*Literal in the subject position of a triple term (expression)*,
  `BIND( <<( "literal" :q :z )>> AS ?X )`) — both NegativeSyntaxTest,
  accepted before, rejected after.
- **Fix**: `ExprTripleTermSubject` now derives exactly `iri | Var` instead of
  delegating to `ExprTripleTermObject`.

## 3. Longest-match tokenization: `<` starting an IRIREF token is not the less-than operator

- **Spec**: SPARQL 1.2 Query §19.7 (same note in SPARQL 1.1 §19.8), note 3:
  *"When tokenizing the input and choosing grammar rules, the longest match is
  chosen."* In `FILTER (?x<?a&&?b>?y)` the characters starting at `<` form a
  valid IRIREF token (`<?a&&?b>` — none of ``<>"{}|^`\`` or #x00–#x20 occur
  inside), so the input tokenizes as `?x`, IRIREF, `?y`, which has no parse —
  the document is a syntax error. The suite's comment in `syn-bad-26.rq`:
  *"longest token rule means this isn't a '<' and '&&'"*. spargebra's
  scannerless grammar happily read `<` as less-than and accepted the document.
- **Test**: `sparql10/syntax-sparql3#syn-bad-26` (NegativeSyntaxTest) —
  accepted before, rejected after. All other syntax suites (1.0/1.1/1.2,
  including every `<`-comparison with whitespace and `?x<<http://iri>` forms,
  where the second `<` terminates the IRIREF scan) are unchanged.
- **Fix**: the `"<=" / "<"` alternative of `RelationalExpression_inner` now
  carries a negative lookahead `!IRIREF_TOKEN()`, where `IRIREF_TOKEN` is the
  IRIREF terminal exactly as tokenized ([172]). The guard also covers `<=`
  (`<=?a>` is likewise the longer IRIREF token), keeping tokenization
  consistent.

## 4. Accept reuse of an earlier SELECT-expression variable in an aggregating query

- **Spec**: SPARQL 1.2 Query §11.4 *Aggregate Projection Restrictions* — every
  variable occurrence in projection/SELECT expressions of a grouping query
  level must satisfy one of three conditions, the third being *"the variable
  is introduced by an earlier SELECT expression in the same SELECT clause"*.
  spargebra validated SELECT expressions in aggregating queries against only
  the WHERE-visible variables, so the legal
  `SELECT (COUNT(?v) AS ?count) (?count + 1 AS ?countPlusOne)` was rejected
  with *"The SELECT contains an expression with a variable that is unbound"*.
- **Test**: `sparql12/grouping#select-variable-reuse` (*Reuse of SELECT
  variable in an aggregating query*, QueryEvaluationTest) — failed at parse
  before, passes end-to-end after.
- **Fix**: `build_select` adds each select-expression alias to the visible set
  after emitting its `Extend`, so later expressions in the same SELECT clause
  see it. The "SELECT overrides an existing variable" check reads the same
  set, so rebinding an alias stays an error (matching §18.3.4.4: *"var must
  not appear in VS nor in PV"*).
