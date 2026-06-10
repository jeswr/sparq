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
