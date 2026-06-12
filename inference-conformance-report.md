# sparq inference conformance report

- rdf-tests commit: `f25dbc092c654d792974848e81bb519d7328f0e8`
- w3c/N3 commit: `23ccf3d56b25cb60a68878a04aae0d52493080f0`
- sparq commit: `a99405906841a621265f51f0de824b91574806a1`

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

| suite | pass | fail | divergence | out-of-scope | pass-rate (of run) |
|---|---:|---:|---:|---:|---:|
| **total** | **0** | **0** | **0** | **0** | **—** |

**Overall (N3 (w3c/N3 community-group suite)): 0 pass / 0 fail / 0 documented divergence / 0 out-of-scope — pass+divergence — of run, — of all in-scope tests.**

## SPARQL 1.1 entailment regimes (sparql11/entailment)

| suite | pass | fail | divergence | out-of-scope | pass-rate (of run) |
|---|---:|---:|---:|---:|---:|
| **total** | **0** | **0** | **0** | **0** | **—** |

**Overall (SPARQL 1.1 entailment regimes (sparql11/entailment)): 0 pass / 0 fail / 0 documented divergence / 0 out-of-scope — pass+divergence — of run, — of all in-scope tests.**

## Overall (all inference suites)

**126 pass / 0 fail / 13 documented divergence / 36 out-of-scope — pass+divergence 100.0% of run, 79.4% of all in-scope tests.**
