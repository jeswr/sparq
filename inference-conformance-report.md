# sparq inference conformance report

- rdf-tests commit: `f25dbc092c654d792974848e81bb519d7328f0e8`
- w3c/N3 commit: `23ccf3d56b25cb60a68878a04aae0d52493080f0`
- sparq commit: `1e63ac02cf54a180ecdc5f6acd462cfc87369ac2`

Every manifest entry lands in exactly one bucket — pass, fail, documented divergence, or out-of-scope WITH its reason (no silent skips). Pass rate is `(pass + divergence) / run`; out-of-scope entries are excluded from the rate but counted in coverage.

## RDF Semantics entailment (rdf-tests rdf/rdf11/rdf-mt)

Premise → `sparq-reason` RDFS/RDF materialization (plus the harness-side finite axiomatic/reflexive augmentation the production materializer deliberately omits) → blank-node-homomorphism entailment check; `mf:result false` = (in)consistency check with the per-test recognized-datatype set.

| suite | pass | fail | divergence | out-of-scope | pass-rate (of run) |
|---|---:|---:|---:|---:|---:|
| rdf-mt | 48 | 0 | 0 | 0 | 100.0% |
| **total** | **48** | **0** | **0** | **0** | **100.0%** |

**Overall (RDF Semantics entailment (rdf-tests rdf/rdf11/rdf-mt)): 48 pass / 0 fail / 0 documented divergence / 0 out-of-scope — pass+divergence 100.0% of run, 100.0% of all in-scope tests.**

## OWL 2 RL (W3C OWL WG test cases, RDF-based semantics)

Source: the OWL WG test-repository export (all.rdf, pinned snapshot — see scripts/fetch-inference-suites.sh). Selection: `test:profile test:RL` AND `test:semantics test:RDF-BASED`; each selected case yields one row per declared check (consistency / inconsistency / positive / negative entailment). Cases not RL-profiled or direct-semantics-only are outside the RL applicability rule and are counted in the selection-summary row, not as individual rows.

Method: premise → `sparq_reason::materialize_owl_rl` → `sparq_reason::inconsistencies` / bnode-homomorphism entailment check; ontology-header triples stripped on both sides. The RL/RDF rules are by design incomplete for arbitrary (TBox) conclusions under the RDF-based semantics (conformance doc §2.3, theorem PR1) — such fails are listed, not hidden.

| suite | pass | fail | divergence | out-of-scope | pass-rate (of run) |
|---|---:|---:|---:|---:|---:|
| owl2-rl/consistency | 52 | 0 | 0 | 24 | 100.0% |
| owl2-rl/inconsistency | 11 | 0 | 0 | 3 | 100.0% |
| owl2-rl/negative-entailment | 4 | 0 | 0 | 3 | 100.0% |
| owl2-rl/positive-entailment | 11 | 0 | 13 | 5 | 100.0% |
| owl2-rl/selection | 0 | 0 | 0 | 1 | — |
| **total** | **78** | **0** | **13** | **36** | **100.0%** |

**Overall (OWL 2 RL (W3C OWL WG test cases, RDF-based semantics)): 78 pass / 0 fail / 13 documented divergence / 36 out-of-scope — pass+divergence 100.0% of run, 71.7% of all in-scope tests.**

### Documented divergences

- `owl2-rl/positive-entailment` — **chain2trans1**: conclusion is a TBox axiom (owl:TransitiveProperty) that no RL/RDF rule derives — PR1 completeness covers assertions only.
  *observed*: conclusion not entailed by the RL/RDF-rules closure
- `owl2-rl/positive-entailment` — **DisjointClasses-001**: conclusion invents an owl:complementOf class expression; the RL/RDF rules derive no new class expressions (PR1 assertion-only completeness).
  *observed*: conclusion not entailed by the RL/RDF-rules closure
- `owl2-rl/positive-entailment` — **DisjointClasses-003**: conclusion invents an owl:complementOf class expression; the RL/RDF rules derive no new class expressions (PR1 assertion-only completeness).
  *observed*: conclusion not entailed by the RL/RDF-rules closure
- `owl2-rl/positive-entailment` — **New-Feature-DisjointDataProperties-002**: conclusion is a reified owl:AllDifferent structure; the RL/RDF rules derive inconsistency from AllDifferent (eq-diff2/3) but never construct one.
  *observed*: conclusion not entailed by the RL/RDF-rules closure
- `owl2-rl/positive-entailment` — **New-Feature-DisjointObjectProperties-001**: conclusion needs the CONTRAPOSITIVE of prp-pdw (property disjointness ⊢ owl:differentFrom of the fillers), which is not an RL/RDF rule.
  *observed*: conclusion not entailed by the RL/RDF-rules closure
- `owl2-rl/positive-entailment` — **New-Feature-DisjointObjectProperties-002**: conclusion needs the CONTRAPOSITIVE of prp-pdw (property disjointness ⊢ owl:differentFrom of the fillers), which is not an RL/RDF rule.
  *observed*: conclusion not entailed by the RL/RDF-rules closure
- `owl2-rl/positive-entailment` — **New-Feature-ObjectQCR-002**: conclusion invents an owl:complementOf class expression; the RL/RDF rules derive no new class expressions (PR1 assertion-only completeness).
  *observed*: conclusion not entailed by the RL/RDF-rules closure
- `owl2-rl/positive-entailment` — **New-Feature-ReflexiveProperty-001**: premise uses ReflexiveObjectProperty, which the OWL 2 RL profile grammar excludes — the export's RL tag contradicts the profile, and prp-rfx is accordingly absent from the RL/RDF rules table.
  *observed*: conclusion not entailed by the RL/RDF-rules closure
- `owl2-rl/positive-entailment` — **owl2-rl-rules-fp-differentFrom**: conclusion needs the CONTRAPOSITIVE of prp-fp (functionality + differentFrom fillers ⊢ differentFrom subjects), which is not an RL/RDF rule.
  *observed*: conclusion not entailed by the RL/RDF-rules closure
- `owl2-rl/positive-entailment` — **owl2-rl-rules-ifp-differentFrom**: conclusion needs the CONTRAPOSITIVE of prp-ifp, which is not an RL/RDF rule.
  *observed*: conclusion not entailed by the RL/RDF-rules closure
- `owl2-rl/positive-entailment` — **WebOnt-I5.5-005**: conclusion invents an owl:unionOf class expression; the RL/RDF rules derive no new class expressions (PR1 assertion-only completeness).
  *observed*: conclusion not entailed by the RL/RDF-rules closure
- `owl2-rl/positive-entailment` — **WebOnt-I5.8-008**: needs datatype-range INTERSECTION reasoning (xsd:short ∩ xsd:unsignedInt ⊑ xsd:unsignedShort), beyond the RL/RDF rules' datatype support.
  *observed*: conclusion not entailed by the RL/RDF-rules closure
- `owl2-rl/positive-entailment` — **WebOnt-I5.8-009**: needs datatype-range INTERSECTION reasoning (xsd:nonNegativeInteger ∩ xsd:nonPositiveInteger = {0} ⊑ xsd:short), beyond the RL/RDF rules' datatype support.
  *observed*: conclusion not entailed by the RL/RDF-rules closure

### Out-of-scope reasons

| reason | tests |
|---|---:|
| status absent (only Approved cases are conformance tests) | 22 |
| status Proposed (only Approved cases are conformance tests) | 8 |
| premise only available in functional syntax | 3 |
| uses owl:imports (no dereferencing in the harness) | 2 |
| selection summary (informational row) | 1 |

## N3 (w3c/N3 community-group suite)

Source: w3c/N3 `tests/` (pinned clone). The reasoner manifest measures EYE/cwm parity of the N3 rule engine; the parser/extended/Turtle manifests measure the N3 parser subset (positive = must parse, negative = must be rejected). Reference graphs are compared under blank-node isomorphism (formulae structurally); `test:strings` (log:outputString) tests are out of scope.

| suite | pass | fail | divergence | out-of-scope | pass-rate (of run) |
|---|---:|---:|---:|---:|---:|
| n3/extended | 880 | 98 | 0 | 0 | 90.0% |
| n3/parser | 189 | 41 | 0 | 0 | 82.2% |
| n3/reasoner | 52 | 36 | 0 | 1 | 59.1% |
| n3/turtle | 210 | 87 | 0 | 0 | 70.7% |
| **total** | **1331** | **262** | **0** | **1** | **83.6%** |

**Overall (N3 (w3c/N3 community-group suite)): 1331 pass / 262 fail / 0 documented divergence / 1 out-of-scope — pass+divergence 83.6% of run, 83.5% of all in-scope tests.**

### Out-of-scope reasons

| reason | tests |
|---|---:|
| log:outputString (test:strings) not implemented | 1 |

<details><summary>All failures (262)</summary>

- `n3/reasoner` — **cwm_includes_bnode**: output not isomorphic to the reference (0 vs 2 statements; expected-only e.g. [Blank("_b1"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/bnodeConclude.n3#Result")])
- `n3/reasoner` — **cwm_includes_concat**: output not isomorphic to the reference (9 vs 16 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/concat.n3#TEST13"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/concat.n3#success")])
- `n3/reasoner` — **cwm_includes_conclusion_simple**: expected parse error: unknown prefix 'log:'
- `n3/reasoner` — **cwm_includes_conclusion**: expected parse error: unknown prefix 'rdfs:'
- `n3/reasoner` — **cwm_includes_conjunction**: output not isomorphic to the reference (0 vs 1 statements; expected-only e.g. [Formula([[Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/foo.n3#sky"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/foo.n3#color"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/foo.n3#blue")], [Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/foo.n3#sky"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/foo.n3#color"), Iri("file://tests/w3c/n3/tests/)
- `n3/reasoner` — **cwm_includes_t2**: reasoner error: bad token '' at 877
- `n3/reasoner` — **cwm_includes_t6**: output not isomorphic to the reference (0 vs 1 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/t6-ref.n3#test6"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/t6-ref.n3#success")])
- `n3/reasoner` — **cwm_includes_builtins**: output not isomorphic to the reference (3 vs 4 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/builtins.n3#test1"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/builtins.n3#Success")]; actual-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/builtins.n3#test1n"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/)
- `n3/reasoner` — **cwm_includes_t10**: output not isomorphic to the reference (0 vs 3 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/foo.n3#test10a"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/foo.n3#success")])
- `n3/reasoner` — **cwm_includes_t11**: output not isomorphic to the reference (1 vs 2 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/foo.n3#includesTest2"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_includes/foo.n3#success")]; actual-only e.g. [Iri("http://www.w3.org/2000/10/swap/log#implies"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("http://www.w3.org/2000/10/swap/log#Chaff")])
- `n3/reasoner` — **cwm_includes_quantifiers_limited**: reasoner error: bad token '@forSome' at 585
- `n3/reasoner` — **cwm_includes_quant-implies**: output not isomorphic to the reference (6 vs 8 statements; expected-only e.g. [Blank("_b1"), Lit("", "http://www.w3.org/2001/XMLSchema#integer", None), Lit("", "http://www.w3.org/2001/XMLSchema#integer", None)])
- `n3/reasoner` — **log_content**: output not isomorphic to the reference (0 vs 1 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/log/content.n3#test1"), Iri("file://tests/w3c/n3/tests/N3Tests/log/content.n3#is"), Lit("@prefix log:  <http://www.w3.org/2000/10/swap/log#>.\n\n{<> log:content ?x} => {:test1 :is ?x} .\n\n{_:bn log:content ?x} => {:test2 a :FAILURE} .\n\n{1 log:content ?x} => {:test3 a :FAILURE} .\n\n{\"foo\" log:content ?x} => {:test4 a :FAILURE} .\n",)
- `n3/reasoner` — **log_langlit**: output not isomorphic to the reference (0 vs 1 statements; expected-only e.g. [Lit("hello", "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString", Some("en")), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/log/langlit.n3#Answer")])
- `n3/reasoner` — **log_parsedAsN3**: reasoner error: bad token '@forAll' at 136
- `n3/reasoner` — **math_exponentiation**: output not isomorphic to the reference (12 vs 13 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/math/exponentiation.n3#test2a"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/math/exponentiation.n3#SUCCESS")])
- `n3/reasoner` — **math_remainder**: reasoner panicked
- `n3/reasoner` — **math_inf**: output not isomorphic to the reference (18 vs 29 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/math/inf.n3#test3e"), Iri("file://tests/w3c/n3/tests/N3Tests/math/inf.n3#is"), Lit("NaN", "http://www.w3.org/2001/XMLSchema#double", None)])
- `n3/reasoner` — **math_corners**: output not isomorphic to the reference (0 vs 2 statements; expected-only e.g. [Lit("0", "http://www.w3.org/2001/XMLSchema#integer", None), Iri("file://tests/w3c/n3/tests/N3Tests/math/corners.n3#valueOf"), Lit(" () math:sum ?x  --- should be 0", "http://www.w3.org/2001/XMLSchema#string", None)])
- `n3/reasoner` — **math_trig**: output not isomorphic to the reference (29 vs 35 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/math/trig.n3#test1a"), Iri("file://tests/w3c/n3/tests/N3Tests/math/trig.n3#SIN"), Lit("0.0e0", "http://www.w3.org/2001/XMLSchema#double", None)]; actual-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/math/trig.n3#test1h"), Iri("file://tests/w3c/n3/tests/N3Tests/math/trig.n3#TANH"), Lit("0", "http://www.w3.org/2001/XMLSchema#integer", )
- `n3/reasoner` — **cwm_list_bug1**: output not isomorphic to the reference (5 vs 6 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/cwm_list/a"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_list/list-bug1.n3#RESULT"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_list/b")])
- `n3/reasoner` — **list_iterate**: output not isomorphic to the reference (0 vs 60 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/list/iterate.n3#test1a"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/list/iterate.n3#SUCCESS")])
- `n3/reasoner` — **cwm_list_r1**: output not isomorphic to the reference (5 vs 7 statements; expected-only e.g. [Lit("one", "http://www.w3.org/2001/XMLSchema#string", None), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_list/v#SUCCESS")])
- `n3/reasoner` — **cwm_list_unify2**: output not isomorphic to the reference (3 vs 4 statements; expected-only e.g. [Lit("17", "http://www.w3.org/2001/XMLSchema#integer", None), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_list/unify2.n3#RESULT")])
- `n3/reasoner` — **cwm_list_unify3**: output not isomorphic to the reference (13 vs 15 statements; expected-only e.g. [Lit("17", "http://www.w3.org/2001/XMLSchema#integer", None), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_list/unify3.n3#RESULT")])
- `n3/reasoner` — **cwm_list_unify4**: output not isomorphic to the reference (36 vs 38 statements; expected-only e.g. [Lit("2", "http://www.w3.org/2001/XMLSchema#integer", None), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_list/unify4.n3#RESULT")]; actual-only e.g. [Blank("_b9"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#first"), Lit("13", "http://www.w3.org/2001/XMLSchema#integer", None)])
- `n3/reasoner` — **cwm_list_unify5**: output not isomorphic to the reference (20 vs 26 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/cwm_list/unify5.n3#Aphrodite"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_list/unify5.n3#ShouldBeAphrodite")]; actual-only e.g. [Blank("_b4"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#first"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_list/unify5.n3#Bob")])
- `n3/reasoner` — **cwm_list_append**: output not isomorphic to the reference (0 vs 27 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/cwm_list/append.n3#test1"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_list/append.n3#success")])
- `n3/reasoner` — **cwm_list_builtin_generated_match**: output not isomorphic to the reference (6 vs 7 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/cwm_list/builtin_generated_match.n3#q"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_list/builtin_generated_match.n3#GreatThing")])
- `n3/reasoner` — **string_concatenation**: output not isomorphic to the reference (31 vs 32 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/string/concatenation.n3#s01"), Iri("file://tests/w3c/n3/tests/N3Tests/string/concatenation.n3#is"), Lit("https://w3c.github.io/N3/tests/N3Tests/string/concatenation.n3#z", "http://www.w3.org/2001/XMLSchema#string", None)]; actual-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/string/concatenation.n3#n10"), Iri("file://tests/w3c/n3/tes)
- `n3/reasoner` — **cwm_string_roughly**: output not isomorphic to the reference (0 vs 12 statements; expected-only e.g. [Blank("_b1"), Iri("http://www.w3.org/2000/10/swap/pim/contact#fullName"), Lit("Tim berners-Lee", "http://www.w3.org/2001/XMLSchema#string", None)])
- `n3/reasoner` — **cwm_string_uriEncode**: output not isomorphic to the reference (0 vs 23 statements; expected-only e.g. [Lit("asd#jkl", "http://www.w3.org/2001/XMLSchema#string", None), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_string/uriEncode.n3#AS_FragID"), Lit("asd%23jkl", "http://www.w3.org/2001/XMLSchema#string", None)])
- `n3/reasoner` — **cwm_supports_simple**: output not isomorphic to the reference (0 vs 1 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/cwm_supports/simple.n3#Q"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_supports/simple.n3#Q"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_supports/simple.n3#Q")])
- `n3/reasoner` — **cwm_time_t1**: output not isomorphic to the reference (27 vs 54 statements; expected-only e.g. [Lit("1999-12-31T23:59:59.99Z", "http://www.w3.org/2001/XMLSchema#string", None), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_time/TEST#dayOfWeek"), Lit("5", "http://www.w3.org/2001/XMLSchema#integer", None)])
- `n3/reasoner` — **cwm_unify_unify1**: output not isomorphic to the reference (0 vs 1 statements; expected-only e.g. [Iri("file://tests/w3c/n3/tests/N3Tests/cwm_unify/unify1.n3#test"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("file://tests/w3c/n3/tests/N3Tests/cwm_unify/unify1.n3#Successful")])
- `n3/reasoner` — **cwm_unify_unify2**: reasoner error: bad token '@forAll' at 60
- `n3/parser` — **quantifiers**: parse error: bad token '@forSome' at 491
- `n3/parser` — **quantifiers_limited**: parse error: bad token '@forSome' at 585
- `n3/parser` — **t2**: parse error: bad token '' at 877
- `n3/parser` — **classes**: parse error: bad token '@forSome' at 55
- `n3/parser` — **strquot.n3**: negative syntax: parser accepted an invalid document
- `n3/parser` — **underbarscope**: parse error: bad token '@forSome' at 71
- `n3/parser` — **single_gen**: parse error: bad token '' at 62
- `n3/parser` — **djb1**: negative syntax: parser accepted an invalid document
- `n3/parser` — **Empty Graph Eval**: parsed statements not isomorphic to the reference
- `n3/parser` — **Empty Graph Implies Eval**: parsed statements not isomorphic to the reference
- `n3/parser` — **isImpliedBy Eval**: parsed statements not isomorphic to the reference
- `n3/parser` — **isImpliedBy Eval**: parsed statements not isomorphic to the reference
- `n3/parser` — **isImpliedBy Eval**: parsed statements not isomorphic to the reference
- `n3/parser` — **embedded-dot-in-qname**: parse error: bad token 'food' at 94
- `n3/parser` — **equals1.n3**: parsed statements not isomorphic to the reference
- `n3/parser` — **equals2.n3**: parsed statements not isomorphic to the reference
- `n3/parser` — **lstring.n3**: parsed statements not isomorphic to the reference
- `n3/parser` — **no-last-nl**: parsed statements not isomorphic to the reference
- `n3/parser` — **numbers.n3**: parsed statements not isomorphic to the reference
- `n3/parser` — **path1.n3**: parsed statements not isomorphic to the reference
- `n3/parser` — **path2.n3**: action parse error: bad token '' at 611
- `n3/parser` — **qvars3**: parse error: bad token '@forAll' at 85
- `n3/parser` — **space-in-uri**: negative syntax: parser accepted an invalid document
- `n3/parser` — **newline-in-uri**: negative syntax: parser accepted an invalid document
- `n3/parser` — **this-quantifiers-ref**: parse error: bad token '@forSome' at 422
- `n3/parser` — **this-rules-ref**: parse error: bad token '@forAll' at 154
- `n3/parser` — **too-nested.n3**: parse error: nesting deeper than 1024
- `n3/parser` — **trailing-dot-in-qname**: negative syntax: parser accepted an invalid document
- `n3/parser` — **trailing-semicolon.n3**: parsed statements not isomorphic to the reference
- `n3/parser` — **zero-objects.n3**: negative syntax: parser accepted an invalid document
- `n3/parser` — **zero-predicates**: parse error: bad token '' at 8
- `n3/parser` — **unify2**: parse error: bad token '@forAll' at 60
- `n3/parser` — **bad_prefix2**: negative syntax: parser accepted an invalid document
- `n3/parser` — **manifest-parser**: timeout (10s) in parser
- `n3/parser` — **caret_neg**: negative syntax: parser accepted an invalid document
- `n3/parser` — **inverted_properties**: parse error: unexpected end of input
- `n3/parser` — **not_embedded.n3**: parse error: bad token 'id' at 36
- `n3/parser` — **with_whitespace.n3**: parse error: bad token 'id' at 37
- `n3/parser` — **with_newline.n3**: parse error: bad token 'id' at 39
- `n3/parser` — **single_object.n3**: parse error: bad token 'id' at 46
- `n3/parser` — **nested_resources.n3**: parse error: bad token 'id' at 42
- `n3/extended` — **10tt_proof**: parse error: bad token '@forAll' at 3109630
- `n3/extended` — **4color_proof**: parse error: bad token '@forAll' at 14083
- `n3/extended` — **agent1-proof**: parse error: bad token '@forAll' at 3272
- `n3/extended` — **agent2-proof**: parse error: bad token '@forAll' at 3602
- `n3/extended` — **answer**: parse error: bad token '@forAll' at 3721
- `n3/extended` — **biE**: parse error: bad token '@forAll' at 98401
- `n3/extended` — **blueproof001**: parse error: bad token '@forAll' at 1424
- `n3/extended` — **blueproof002**: parse error: bad token '@forAll' at 1440
- `n3/extended` — **blueproof003**: parse error: bad token '@forSome' at 1331
- `n3/extended` — **bmi_proof**: parse error: bad token '@forAll' at 19368
- `n3/extended` — **crypto-proof**: parse error: bad token '@forAll' at 2725
- `n3/extended` — **deE**: parse error: bad token '@forAll' at 1196
- `n3/extended` — **dpE**: parse error: bad token '@forAll' at 3878
- `n3/extended` — **dpe_proof**: parse error: bad token '@forAll' at 1572
- `n3/extended` — **easter-proof**: parse error: bad token '@forAll' at 41556
- `n3/extended` — **easterE**: parse error: bad token '@forAll' at 11018
- `n3/extended` — **einsteinE**: parse error: bad token '@forAll' at 14793
- `n3/extended` — **fcm_proof**: parse error: bad token '@forAll' at 8554
- `n3/extended` — **fgcm_proof**: parse error: bad token '@forAll' at 15292
- `n3/extended` — **fibE**: parse error: bad token '@forAll' at 1700
- `n3/extended` — **floatingwoman-proof**: parse error: bad token '@forAll' at 1085
- `n3/extended` — **food-proof**: parse error: bad token '@forAll' at 10465
- `n3/extended` — **food2-proof**: parse error: bad token '@forAll' at 10592
- `n3/extended` — **forAllIn_proof**: parse error: bad token '@forAll' at 3376
- `n3/extended` — **gedcom-proof**: parse error: bad token '@forAll' at 7670
- `n3/extended` — **gps-proof1**: parse error: bad token '@forAll' at 3359
- `n3/extended` — **gps-proof2**: parse error: bad token '@forAll' at 16166
- `n3/extended` — **graph.proof**: parse error: bad token '@forAll' at 3599
- `n3/extended` — **hanoiE**: parse error: bad token '@forAll' at 1845
- `n3/extended` — **iq_proof**: parse error: bad token '@forAll' at 7075
- `n3/extended` — **lldmE**: parse error: bad token '@forAll' at 1740
- `n3/extended` — **medicE**: parse error: bad token '@forAll' at 3383
- `n3/extended` — **mmln-gv-mln**: parse error: bad token '' at 28996
- `n3/extended` — **mmln-gv-proof**: parse error: bad token '@forAll' at 39460
- `n3/extended` — **mq_proof**: parse error: bad token '@forAll' at 3761
- `n3/extended` — **nbbn_proof**: parse error: bad token '@forAll' at 2528
- `n3/extended` — **notIn_proof**: parse error: bad token '@forAll' at 2427
- `n3/extended` — **numeral_proof**: parse error: bad token '@forAll' at 3390
- `n3/extended` — **palindrome-proof**: parse error: bad token '@forAll' at 1768
- `n3/extended` — **palindrome2-proof**: parse error: bad token '@forAll' at 1741
- `n3/extended` — **path-9-3-proof**: parse error: bad token '@forAll' at 4562
- `n3/extended` — **pi-proof**: parse error: bad token '@forAll' at 1451
- `n3/extended` — **proof-001**: parse error: bad token '@forAll' at 2320
- `n3/extended` — **proof-10**: parse error: bad token '@forAll' at 2410
- `n3/extended` — **proof-100**: parse error: bad token '@forAll' at 5841
- `n3/extended` — **proof-1000**: parse error: bad token '@forAll' at 80551
- `n3/extended` — **proof-10000**: parse error: bad token '@forAll' at 857559
- `n3/extended` — **proof-2-10**: parse error: bad token '@forAll' at 3847
- `n3/extended` — **proof-2-100**: parse error: bad token '@forAll' at 7308
- `n3/extended` — **proof-2-1000**: parse error: bad token '@forAll' at 82050
- `n3/extended` — **proof-2-10000**: parse error: bad token '@forAll' at 859090
- `n3/extended` — **proof**: parse error: bad token '@forAll' at 3722
- `n3/extended` — **randomsample-proof**: parse error: bad token '@forAll' at 162252
- `n3/extended` — **resto-proof**: parse error: bad token '@forAll' at 3514
- `n3/extended` — **rifE**: parse error: bad token '@forAll' at 238844
- `n3/extended` — **sdcoding-a-proof**: parse error: bad token '@forAll' at 5759
- `n3/extended` — **sdcoding-proof**: parse error: bad token '@forAll' at 4148
- `n3/extended` — **select-proof-extra**: parse error: bad token '@forAll' at 6372
- `n3/extended` — **select-proof**: parse error: bad token '@forAll' at 19669
- `n3/extended` — **skos-extra-rules**: parse error: bad token '@false' at 1399
- `n3/extended` — **skos-rules**: parse error: bad token '@false' at 3654
- `n3/extended` — **skos_mv_proof**: parse error: bad token '@forAll' at 5784
- `n3/extended` — **socrates_proof**: parse error: bad token '@forAll' at 1700
- `n3/extended` — **swet_proof**: parse error: bad token '@forAll' at 122991
- `n3/extended` — **takE**: parse error: bad token '@forAll' at 1969
- `n3/extended` — **test-proof-1000**: parse error: bad token '@forAll' at 1285
- `n3/extended` — **testE**: parse error: bad token '@forAll' at 3681
- `n3/extended` — **test_proof**: parse error: bad token '@forAll' at 25910
- `n3/extended` — **train_model_proof**: parse error: bad token '@forAll' at 34394
- `n3/extended` — **turing_proof**: parse error: bad token '@forAll' at 1387
- `n3/extended` — **usmE**: parse error: bad token '@forAll' at 1620
- `n3/extended` — **utf8_proof**: parse error: bad token '@forAll' at 3280
- `n3/extended` — **witch-proof**: parse error: bad token '@forAll' at 988
- `n3/extended` — **D1Q**: parse error: bad token '@forAll' at 403
- `n3/extended` — **D2Q**: parse error: bad token '@forAll' at 403
- `n3/extended` — **LanguageQ**: parse error: bad token '@de' at 514
- `n3/extended` — **bcE**: parse error: bad token '@forAll' at 1914
- `n3/extended` — **exonQ**: parse error: bad token '@forAll' at 405
- `n3/extended` — **icalQ001**: parse error: bad token '' at 546
- `n3/extended` — **icalQ002**: parse error: bad token '' at 564
- `n3/extended` — **icalR**: parse error: bad token '' at 216
- `n3/extended` — **metastaticE**: parse error: bad token '@forAll' at 2251
- `n3/extended` — **metastaticR**: parse error: unknown prefix 'owl:'
- `n3/extended` — **michaelE**: parse error: bad token '@prefix' at 379
- `n3/extended` — **minsuE**: parse error: bad token '@prefix' at 301
- `n3/extended` — **query-survey-10**: negative syntax: parser accepted an invalid document
- `n3/extended` — **query-survey-11**: parse error: bad token '@de' at 499
- `n3/extended` — **query-survey-13**: parse error: bad token '' at 535
- `n3/extended` — **sethE**: parse error: bad token '@prefix' at 300
- `n3/extended` — **socratesR**: parse error: bad token '@forAll' at 1527
- `n3/extended` — **unsaidQ**: parse error: bad token '@forAll' at 316
- `n3/extended` — **cd**: parse error: bad token '@forSome' at 404
- `n3/extended` — **eventTime_queries2**: parse error: bad token '' at 1286
- `n3/extended` — **gmpbnodeE1**: parse error: bad token '@forSome' at 1196
- `n3/extended` — **gv-mln**: parse error: bad token '' at 28997
- `n3/extended` — **resto-cwm-proof**: parse error: bad token '@forSome' at 684
- `n3/extended` — **resto-cwm-proof2**: parse error: bad token '@forSome' at 684
- `n3/extended` — **utf8**: read tests/w3c/n3/tests/N3Tests/07test/utf8.n3: stream did not contain valid UTF-8
- `n3/turtle` — **IRI_with_four_digit_numeric_escape**: parsed graph not isomorphic to the reference
- `n3/turtle` — **IRI_with_eight_digit_numeric_escape**: parsed graph not isomorphic to the reference
- `n3/turtle` — **prefix_with_non_leading_extras**: action parse error: bad token 'a·̀ͯ‿' at 45
- `n3/turtle` — **reserved_escaped_localName**: action parse error: bad token '\-\' at 40
- `n3/turtle` — **localName_with_non_leading_extras**: action parse error: bad token '⁀' at 45
- `n3/turtle` — **labeled_blank_node_with_PN_CHARS_BASE_character_boundaries**: parser panicked
- `n3/turtle` — **labeled_blank_node_with_non_leading_extras**: parser panicked
- `n3/turtle` — **sole_blankNodePropertyList**: parsed graph not isomorphic to the reference
- `n3/turtle` — **nested_blankNodePropertyLists**: parsed graph not isomorphic to the reference
- `n3/turtle` — **blankNodePropertyList_containing_collection**: parsed graph not isomorphic to the reference
- `n3/turtle` — **literal_with_escaped_BACKSPACE**: parsed graph not isomorphic to the reference
- `n3/turtle` — **literal_with_escaped_FORM_FEED**: parsed graph not isomorphic to the reference
- `n3/turtle` — **literal_with_numeric_escape4**: parsed graph not isomorphic to the reference
- `n3/turtle` — **literal_with_numeric_escape8**: parsed graph not isomorphic to the reference
- `n3/turtle` — **repeated_semis_at_end**: action parse error: bad token '' at 65
- `n3/turtle` — **repeated_semis_not_at_end**: action parse error: bad token '' at 65
- `n3/turtle` — **turtle-syntax-base-04**: parse error: bad token 'base' at 0
- `n3/turtle` — **turtle-syntax-prefix-02**: parse error: bad token 'PreFIX' at 0
- `n3/turtle` — **turtle-syntax-pname-esc-01**: parse error: bad token '\-\' at 61
- `n3/turtle` — **turtle-syntax-pname-esc-02**: parse error: bad token '\-\' at 65
- `n3/turtle` — **turtle-syntax-number-11**: parse error: bad token 'E+1' at 12
- `n3/turtle` — **turtle-syntax-struct-04**: parse error: bad token '' at 62
- `n3/turtle` — **turtle-syntax-struct-05**: parse error: bad token '' at 75
- `n3/turtle` — **turtle-syntax-bad-uri-01**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-uri-02**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-uri-03**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-uri-04**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-uri-05**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-prefix-01**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-prefix-05**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-base-03**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-struct-02**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-struct-03**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-struct-04**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-struct-05**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-struct-06**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-struct-07**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-kw-04**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-kw-05**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-n3-extras-01**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-n3-extras-02**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-n3-extras-03**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-n3-extras-04**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-n3-extras-05**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-n3-extras-06**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-n3-extras-09**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-n3-extras-10**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-struct-08**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-struct-09**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-struct-10**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-struct-11**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-struct-14**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-struct-15**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-struct-16**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-struct-17**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-lang-01**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-esc-01**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-esc-02**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-esc-03**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-esc-04**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-pname-01**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-pname-02**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-pname-03**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-num-02**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-num-05**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-subm-01**: parsed graph not isomorphic to the reference
- `n3/turtle` — **turtle-subm-16**: parsed graph not isomorphic to the reference
- `n3/turtle` — **turtle-subm-27**: parsed graph not isomorphic to the reference
- `n3/turtle` — **turtle-eval-bad-01**: eval test without mf:result
- `n3/turtle` — **turtle-eval-bad-02**: eval test without mf:result
- `n3/turtle` — **turtle-eval-bad-03**: eval test without mf:result
- `n3/turtle` — **turtle-eval-bad-04**: eval test without mf:result
- `n3/turtle` — **comment_following_localName**: parsed graph not isomorphic to the reference
- `n3/turtle` — **number_sign_following_localName**: parsed graph not isomorphic to the reference
- `n3/turtle` — **comment_following_PNAME_NS**: parsed graph not isomorphic to the reference
- `n3/turtle` — **number_sign_following_PNAME_NS**: parsed graph not isomorphic to the reference
- `n3/turtle` — **langtagged_LONG_with_subtag**: parsed graph not isomorphic to the reference
- `n3/turtle` — **turtle-syntax-bad-blank-label-dot-end**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-ln-dash-start**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-ln-escape**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-bad-ln-escape-start**: negative syntax: parser accepted an invalid document
- `n3/turtle` — **turtle-syntax-ln-dots**: parse error: bad token 's' at 89
- `n3/turtle` — **turtle-syntax-ns-dots**: parse error: bad token 'e' at 53
- `n3/turtle` — **IRI-resolution-01**: parsed graph not isomorphic to the reference
- `n3/turtle` — **IRI-resolution-02**: parsed graph not isomorphic to the reference
- `n3/turtle` — **IRI-resolution-07**: parsed graph not isomorphic to the reference
- `n3/turtle` — **IRI-resolution-08**: parsed graph not isomorphic to the reference

</details>

## SPARQL 1.1 entailment regimes (sparql11/entailment)

| suite | pass | fail | divergence | out-of-scope | pass-rate (of run) |
|---|---:|---:|---:|---:|---:|
| **total** | **0** | **0** | **0** | **0** | **—** |

**Overall (SPARQL 1.1 entailment regimes (sparql11/entailment)): 0 pass / 0 fail / 0 documented divergence / 0 out-of-scope — pass+divergence — of run, — of all in-scope tests.**

## Overall (all inference suites)

**1457 pass / 262 fail / 13 documented divergence / 37 out-of-scope — pass+divergence 84.9% of run, 83.1% of all in-scope tests.**
