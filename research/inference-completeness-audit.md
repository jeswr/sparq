# Inference feature-completeness audit (W3C suites)

**Thread:** inference-suites · **Date:** 2026-06 · **Harness:** `crates/sparq-conformance`,
binary `sparq-inference-conformance` (fetch once with `scripts/fetch-inference-suites.sh`,
then fully offline; report regenerated as `inference-conformance-report.md`).

Pinned sources: w3c/rdf-tests `f25dbc092c654d792974848e81bb519d7328f0e8` (shared with the
gating SPARQL harness), w3c/N3 `23ccf3d56b25cb60a68878a04aae0d52493080f0`, OWL 2 WG
test-case export `all.rdf` (Internet Archive snapshot `20160703034201`, sha256-pinned).

## Scoreboard (per-suite Overall lines)

| suite | run | pass | fail | documented divergence | out-of-scope (with reason) | rate of run |
|---|---:|---:|---:|---:|---:|---:|
| RDF Semantics (rdf-mt, 48 active entries) | 48 | **48** | 0 | 0 | 0 | **100%** |
| OWL 2 RL (RL-profile ∧ RDF-based, Approved) | 91 | **78** | 0 | 13 | 36 | **100%** (pass+div) |
| N3 — reasoner manifest (89) | 88 | **52** | 36 | 0 | 1 | 59.1% |
| N3 — parser manifest (230) | 230 | **189** | 41 | 0 | 0 | 82.2% |
| N3 — extended manifest (978) | 978 | **880** | 98 | 0 | 0 | 90.0% |
| N3 — TurtleTests through the N3 parser (297) | 297 | **210** | 87 | 0 | 0 | 70.7% |
| SPARQL 1.1 entailment regimes (70) | 47 | **44** | 3 | 0 | 23 | 93.6% |
| **all suites** | **1779** | **1501** | **265** | **13** | **60** | **85.1%** |

The gating SPARQL harness is untouched and still at 1225 pass + 4 divergences = 1229/1229.

Harness design notes:
- Entailment checks are blank-node **homomorphisms** from the conclusion into the
  materialized closure over GENERALIZED triples (conclusion bnodes may map onto literals —
  the RDF 1.1 lg/gl rules), with D-entailment literal **value** equality
  (integer/decimal exactly via scaled i128; float/double via f32/f64 rounding; langString
  and XMLLiteral value spaces; ill-formedness detection incl. a strict XML scan).
- The conformance regimes need rules the production materializer deliberately omits as
  store-exploding (axiomatic triples, reflexives). The harness adds the FINITE restriction
  (rdf:_n limited to occurring container-membership properties; declared-vocabulary
  reflexives for the SPARQL regimes) *around* `sparq_reason::materialize` — the core rules
  are always exercised through the production code path.

## 1. RDFS entailment patterns (RDF 1.1 Semantics §9.2.1)

| pattern | production materializer (`rdfs.rs`) | conformance result |
|---|---|---|
| rdfs1 (datatype recognition) | omitted by design | harness layer; rdf-mt passes |
| rdfs2 (domain) / rdfs3 (range) | ✅ (incl. literal objects, generalized) | ✅ |
| rdfs4a/4b (everything is a Resource) | omitted by design (O(terms) bloat) | harness layer; rdf-mt passes |
| rdfs5 / rdfs11 (sp/sc transitivity) | ✅ | ✅ |
| rdfs6 / rdfs8 / rdfs10 (reflexives) | omitted by design | harness layer; rdf-mt + sparql-entailment pass |
| rdfs7 (subPropertyOf use) / rdfs9 (subClassOf use) | ✅ | ✅ |
| rdfs12 (ContainerMembershipProperty ⊑ rdfs:member) | omitted | harness layer (finite rdf:_n restriction) |
| rdfs13 (Datatype ⊑ Literal) | omitted | harness layer |
| RDF/RDFS axiomatic triples | omitted by design | harness layer (finite restriction) |
| rdfD1/rdfD2, D-entailment, D-inconsistency | not materialized | harness (`inference/entail.rs`): value spaces, ill-typed literals, intensional datatype-subclass clash — all 48 rdf-mt entries pass incl. every datatype/float/double case |

**Conclusion:** RDFS materialization is complete for the non-axiomatic fragment; the
axiomatic/reflexive layer exists only in the conformance harness. If a product use-case
ever needs it in the store, `entail::close()` is the reference implementation.

## 2. OWL 2 RL/RDF rules table (Profiles §4.3) — per-rule status in `owl.rs`

| rule | status | where / note |
|---|---|---|
| eq-ref | ◑ partial **by design** | reflexive sameAs emitted only for terms touched by equality (full eq-ref is O(terms)) |
| eq-sym, eq-trans, eq-rep-s/p/o | ✅ | union-find entity rewriting + full expansion at the end (RDFox approach) |
| eq-diff1 | ✅ | `inconsistencies()` |
| eq-diff2, eq-diff3 (AllDifferent) | ✅ **(this thread)** | `inconsistencies()`, members/distinctMembers lists |
| prp-ap (annotation-property axioms) | ✗ N/A | axiomatic table; no suite coverage |
| prp-dom, prp-rng | ✅ | via rdfs2/3 |
| prp-fp, prp-ifp | ✅ | derive sameAs → merge; literal-value clash detected (dt-diff ⊢ eq-diff1, **this thread**) |
| prp-irp, prp-asyp | ✅ **(this thread)** | `inconsistencies()` |
| prp-spo1 | ✅ | rdfs7 |
| prp-spo2 (property chains) | ✅ | |
| prp-eqp1/2, prp-inv1/2, prp-symp, prp-trp | ✅ | |
| prp-pdw, prp-adp | ✅ **(this thread)** | `inconsistencies()` |
| prp-key | ✅ | incl. data properties (literal clash) |
| prp-npa1/2 | ✅ **(this thread)** | `inconsistencies()` |
| cls-thing, cls-nothing1 | ✗ missing | cosmetic axioms (owl:Thing/Nothing typed Class); no test demands them |
| cls-nothing2 | ✅ | `inconsistencies()` |
| cls-int1/2, cls-uni | ✅ | |
| cls-com | ✅ | clash |
| cls-svf1/2, cls-avf, cls-hv1/2 | ✅ | |
| cls-maxc1 | ✅ | clash |
| cls-maxc2, cls-maxqc3/4 | ✅ | derive sameAs |
| cls-maxqc1/2 (qualified cardinality 0 clashes) | ✗ missing | no Approved RL test exercises them; cheap follow-up |
| cls-oo (oneOf typing) | ✗ missing | `owl:oneOf` not decoded; the only RL suite case (owl2-rl-valid-oneof) is consistency-only and passes |
| cax-sco, cax-eqc1/2 | ✅ | |
| cax-dw | ✅ | clash |
| cax-adc | ✅ **(this thread)** | clash, members list |
| dt-type1/2, dt-eq, dt-not-type | ◑ harness-level | literal datatype typing/value equality live in the harness D-machinery, not the store closure |
| dt-diff | ✅ **(this thread)** | as the literal-sameAs clash |
| scm-cls | ◑ partial by design | Thing/Nothing/reflexive edges added by the SPARQL-regime layer only |
| scm-sco, scm-spo | ✅ | rdfs11/5 |
| scm-eqc1, scm-eqp1 | ✅ | equivalence folded into subsumption both ways |
| scm-eqc2, scm-eqp2 | ✅ **(this thread)** | mutual subsumption ⊢ equivalence (post-pass) |
| scm-op, scm-dp | ◑ harness-level | declared-property reflexive subPropertyOf |
| scm-dom1/2, scm-rng1/2 | ✅ | + **(this thread)** inverseOf domain/range transposition (valid RDF-based entailment beyond the table) |
| scm-hv, scm-svf1/2, scm-avf1/2 | ✗ missing | schema-level restriction subsumption; no Approved RL test hits them |
| scm-int, scm-uni | ✅ | schema level |
| XSD datatype hierarchy (numeric tower ⊑) | ✅ **(this thread)** | direct edges for occurring datatype IRIs (WebOnt-I5.8) |

13 documented divergences = expected conclusions PROVABLY outside the RL/RDF rules
(OWL 2 Conformance §2.3 / theorem PR1 scopes completeness to assertion-style conclusions):
TBox-axiom conclusions (chain2trans1), invented class expressions (DisjointClasses-001/003,
ObjectQCR-002, I5.5-005), reified AllDifferent structures, prp-pdw/fp/ifp contrapositives,
datatype-range intersections (I5.8-008/009), and one export-side mis-tag
(ReflexiveObjectProperty is not in the RL grammar). Full rationale list is in the report
and `owl_suite.rs::DOCUMENTED_DIVERGENCES`.

Out-of-scope: 30 non-Approved cases (22 status-absent + 8 Proposed), 3 functional-syntax-only,
2 owl:imports, 1 selection-summary row. Export totals: 124 RL∧RDF-based reasoning cases of
~441 reasoning cases (251 not RL-profiled, 66 direct-semantics-only).

## 3. N3 — engine and builtin inventory vs EYE/cwm

Reasoner manifest: **52/88** (59.1%). Fixed this thread (suite-driven):
`is EXPR of`/`has EXPR` syntax, undeclared `:` prefix = `<#>`, RFC 3986 base resolution,
premise blank nodes as existentials, EXACT integer/decimal builtin arithmetic
(scaled i128), numeric comparison of ground objects across numeric types, string-coerced
numerics, INF/NaN, round-half-UP + decimal result typing, integer floor/ceiling, unary ops
rejecting list args, `math:negation` reverse mode, data-list walking for
`list:length/first/last/member/in`, `rdf:nil` = empty list,
`string:equalIgnoringCase/notEqualIgnoringCase/notGreaterThan/notLessThan`, parser
nesting-depth guard (a stress file previously ABORTED the process by stack overflow).

### Remaining reasoner-manifest gaps (36 fails, by root cause)

| root cause | tests | notes |
|---|---|---|
| `@forAll` / `@forSome` quantifiers not parsed | cwm_includes_quantifiers_limited, log_parsedAsN3, cwm_includes_t2, (+6 extended-suite files) | parser feature; moderate |
| log:includes / log:notIncludes scoping subtleties (non-ground scopes, nested formulae, `=>` inside includes) | cwm_includes_t6/t10/t11/builtins/conclusion*/quant-implies, cwm_supports_simple | the engine's scoped-negation handles the common cases; full cwm semantics needs formula quantification |
| first-class list values (`list:append`, list unification in join atoms, lists as conclusions) | cwm_list_unify2-5, cwm_list_append, cwm_list_r1, cwm_list_bug1, cwm_list_builtin_generated_match, list_iterate, cwm_includes_concat | needs `Term::List`; known documented gap (module doc) |
| log:content / log:semantics / log:outputString (document access / output builtins) | log_content, (strings: 1 out-of-scope) | deliberately excluded (pure-function reasoner); decide policy before implementing |
| bnode-conclusion instantiation (fresh bnode per binding) | cwm_includes_bnode, cwm_includes_conjunction | conclusion bnodes are currently shared constants |
| lang-tagged literal handling in builtins | log_langlit | cheap |
| double formatting / e-notation output, INF corner cases | math_inf, math_corners, math_trig, math_exponentiation (1 case), math_remainder (negative operands), string_concatenation (1 IRI-resolution case) | mostly output-lexical-form mismatches vs cwm refs |
| `string:tokenize`-era cwm builtins (uriEncode, roughly), `time:` full formats | cwm_string_uriEncode, cwm_string_roughly, cwm_time_t1 | small builtins, check EYE's table before adding |
| unification through quoting | cwm_unify_unify1/2, cwm_reason_t9 (passes? see report) | formula-term unification in joins |

### Parser posture (syntax suites)

- parser manifest 189/230: 12 eval mismatches (formula/quoting round-trips), 8
  negative-syntax leniency, the rest single-feature gaps (`@keywords`, string escapes).
- extended 880/978: dominated by `@forAll/@forSome` (and files needing full N3 paths /
  `@keywords`); 1 non-UTF8 file.
- TurtleTests 210/297 through the N3 parser: 46 negative-syntax tests ACCEPTED (the
  hand-rolled parser is deliberately permissive — e.g. bare `\uXXXX` validation, datatype
  position checks), 21 eval mismatches (mostly `\u` escape decoding and numeric-literal
  canonicalization), a handful of `PreFIX`-case / exotic-token errors. NOTE: sparq's
  product Turtle path is oxttl (covered by the gating SPARQL harness); this measures the
  N3 subsystem's parser only.

## 4. SPARQL 1.1 entailment regimes

44/47 run pass (RDF + RDFS + OWL-RDF-Based mappings); 23 out-of-scope with reasons
(18 OWL-Direct-only, 4 RIF, 1 D-only). Regime layer added by the runner: rdfD2 predicate
typing, declared-class/property reflexives (rdfs6/10, scm-cls/op/dp), Thing/Nothing edges,
Thing-typing of NamedIndividuals. Engine fixes this thread: inverseOf domain/range
transposition; reflexive sameAs pairs within merged equivalence classes (eq-ref on touched
terms).

Remaining 3 fails (annotated in the report):
- `sparqldl-10` — non-distinguished (blank-node) query variables.
- `sparqldl-11/12` — the regime's answer restriction (skolemization) excludes bnode
  bindings; the harness doesn't filter engine results yet.

## 5. Prioritized gap map for follow-up fix threads

1. **N3 `Term::List`** (first-class list values): unlocks ~9 reasoner tests
   (list unification, `list:append`, lists in conclusions, backward goals over lists —
   EYE's fibonacci/collatz class of programs). The single biggest N3 step. (structural)
2. **`@forAll`/`@forSome`** parsing (+ mapping to Var/fresh-bnode): ~3 reasoner + ~8
   syntax tests. (parser, moderate)
3. **Conclusion-bnode instantiation** (fresh bnode per rule firing): 2 reasoner tests,
   semantic correctness issue worth fixing regardless. (small-medium)
4. **log:includes full scoping** (quantified formula containment): ~7 reasoner tests.
   (semantics, medium)
5. **N3-parser Turtle strictness pass** (\u escapes, negative-syntax rejections,
   `@keywords`): ~60 syntax tests, mechanical. (parser, low risk)
6. **SPARQL-regime answer filtering** (drop bnode bindings per skolemization condition):
   2 tests; needs a result post-filter hook in the runner or engine. (small)
7. **OWL leftovers**: cls-oo typing, cls-maxqc1/2 clashes, scm-hv/svf/avf — no current
   test pressure; do alongside any RL hardening. (small each)
8. **Decide policy** on document-access builtins (log:semantics/content/outputString) —
   excluded today by design; an opt-in resolver could lift 2+ tests and real EYE use-cases.

## 6. Reproduction

```sh
./scripts/fetch-inference-suites.sh         # one-time, pinned
cargo run --release -p sparq-conformance --bin sparq-inference-conformance
# report: inference-conformance-report.md  (--verbose for per-test lines,
# --filter SUBSTR to narrow, --strict to exit 1 on fails)
```

CI: the `inference-conformance` job (informational, non-gating) uploads the report and
prints it to the step summary; ratchet to follow once the N3 score stabilizes.
