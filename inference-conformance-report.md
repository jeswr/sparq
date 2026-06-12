# sparq inference conformance report

- rdf-tests commit: `f25dbc092c654d792974848e81bb519d7328f0e8`
- w3c/N3 commit: `23ccf3d56b25cb60a68878a04aae0d52493080f0`
- sparq commit: `16ca0f982733ce0d33db84895567be2ac2fe558a`

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

Source: w3c/N3 `tests/` (pinned clone). The reasoner manifest measures EYE/cwm parity of the N3 rule engine; the parser/extended manifests measure the N3 parser (positive = must parse, negative = must be rejected); TurtleTests runs the parser in STRICT Turtle mode. Documents parse against the suite's canonical https://w3c.github.io/N3/tests/ base (the .nt expectations bake those IRIs in), and an offline resolver maps those IRIs back into the pinned clone so log:semantics/log:content work without I/O. Reference graphs are compared under blank-node isomorphism (formulae structurally, lists expanded where the expectation is plain triples, same-datatype numerics by value); reason references parse against the ACTION document's base (cwm generated them so). Under test:conclusions both derived-only and store-minus-rules shapings are accepted (the vendored cwm out-files are inconsistent between the two). `rdft:Rejected` entries are out of scope (upstream rejected them); `test:strings` (log:outputString) stays out of scope.

| suite | pass | fail | divergence | out-of-scope | pass-rate (of run) |
|---|---:|---:|---:|---:|---:|
| n3/extended | 871 | 0 | 0 | 107 | 100.0% |
| n3/parser | 213 | 0 | 1 | 16 | 100.0% |
| n3/reasoner | 83 | 0 | 3 | 3 | 100.0% |
| n3/turtle | 297 | 0 | 0 | 0 | 100.0% |
| **total** | **1464** | **0** | **4** | **126** | **100.0%** |

**Overall (N3 (w3c/N3 community-group suite)): 1464 pass / 0 fail / 4 documented divergence / 126 out-of-scope — pass+divergence 100.0% of run, 92.1% of all in-scope tests.**

### Documented divergences

- `n3/reasoner` — **cwm_includes_conclusion**: the engine now derives the `:result :is { … }` conclusion formula, but the vendored conclusion-ref.n3 cannot match the vendored sources: (1) its quoted daml:comment for :Animal reads `…number of\nontological…` while the vendored daml-ex.n3 has `…number of\n\tontological…` (TAB) — the 2003 cwm run used an older daml-ex.n3 revision; (2) the ref formula is not closed under its OWN quoted rules (it holds `d:father daml:range d:Man`, `daml:range = rdfs:range` and the rule `{?x ?p1 ?y. ?p1 = ?p2} => {?x ?p2 ?y}`, yet lacks `d:father rdfs:range d:Man`) — log:conclusion ("all statements which can be deduced") legitimately derives 31 statements the 2003 run did not (instantiated transitivity rules and the consequent type/schema facts).
  *observed*: output not isomorphic to the reference (2 vs 1 statements; expected-only e.g. [Iri("https://w3c.github.io/N3/tests/N3Tests/cwm_includes/foo.n3#result"), Iri("https://w3c.github.io/N3/tests/N3Tests/cwm_includes/foo.n3#is"), Formula([[Iri("https://w3c.github.io/N3/tests/N3Tests/cwm_other/daml-ex.n3"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("http://www.daml.org/2001/03/daml+oil#Ontology")], [Iri("https://w3c.github.io/N3/tests/N3Tests/)
- `n3/reasoner` — **cwm_includes_t11**: with the offline resolver the engine derives the schema-checking conclusions over <t10a.n3> (test_undefined etc.); t11-ref.n3 holds only the two foo.n3 conclusions and even omits t11's own data facts — the cwm run that produced it never resolved t10a.n3 and ran with an unrecorded --purge.
  *observed*: output not isomorphic to the reference (11 vs 2 statements; actual-only e.g. [Iri("https://w3c.github.io/N3/tests/N3Tests/cwm_includes/foo.n3#test_undefined"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("https://w3c.github.io/N3/tests/N3Tests/cwm_includes/foo.n3#UsedProperty")])
- `n3/reasoner` — **cwm_unify_unify1**: the action concludes `:test :a ?x` (predicate <unify1.n3#a>) but unify1-ref.n3 says `:test a :Successful` (rdf:type) — the vendored cwm reference was generated from an older revision of the action.
  *observed*: output not isomorphic to the reference (1 vs 1 statements; expected-only e.g. [Iri("https://w3c.github.io/N3/tests/N3Tests/cwm_unify/unify1.n3#test"), Iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"), Iri("https://w3c.github.io/N3/tests/N3Tests/cwm_unify/unify1.n3#Successful")]; actual-only e.g. [Iri("https://w3c.github.io/N3/tests/N3Tests/cwm_unify/unify1.n3#test"), Iri("https://w3c.github.io/N3/tests/N3Tests/cwm_unify/unify1.n3#a"), Iri("https:/)
- `n3/parser` — **numbers.n3**: the expected output cwm_n3/n3parser.tests_n3_10013.n3 keeps ONE statement under the generating author's local base (file:/home/syosi/...#is) while the rest were rebased — that statement can never match any correct parse.
  *observed*: parsed statements not isomorphic to the reference

### Out-of-scope reasons

| reason | tests |
|---|---:|
| rdft:Rejected upstream | 125 |
| log:outputString (test:strings) not implemented | 1 |

## SPARQL 1.1 entailment regimes (sparql11/entailment)

Source: `sparql11/entailment` from the pinned rdf-tests clone. Each test runs as query-over-materialized-closure through the same evaluation/comparison machinery as the gating SPARQL harness; the regime materialized is the strongest the reasoner supports of the test's `sd:entailmentRegime` set (RDFS ⊃ RDF; OWL-RDF-Based via the OWL RL rules, a sound subset — its incompleteness shows up as listed fails). D-only / OWL-Direct-only / RIF tests are out of scope with their reason.

| suite | pass | fail | divergence | out-of-scope | pass-rate (of run) |
|---|---:|---:|---:|---:|---:|
| sparql11/entailment | 47 | 0 | 0 | 23 | 100.0% |
| **total** | **47** | **0** | **0** | **23** | **100.0%** |

**Overall (SPARQL 1.1 entailment regimes (sparql11/entailment)): 47 pass / 0 fail / 0 documented divergence / 23 out-of-scope — pass+divergence 100.0% of run, 67.1% of all in-scope tests.**

### Out-of-scope reasons

| reason | tests |
|---|---:|
| entailment regime(s) OWL-Direct not supported (no materialization mapping) | 18 |
| entailment regime(s) RIF not supported (no materialization mapping) | 4 |
| entailment regime(s) D not supported (no materialization mapping) | 1 |

## Overall (all inference suites)

**1637 pass / 0 fail / 17 documented divergence / 185 out-of-scope — pass+divergence 100.0% of run, 89.9% of all in-scope tests.**
