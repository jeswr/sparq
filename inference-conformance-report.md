# sparq inference conformance report

- rdf-tests commit: `f25dbc092c654d792974848e81bb519d7328f0e8`
- w3c/N3 commit: `23ccf3d56b25cb60a68878a04aae0d52493080f0`
- sparq commit: `231872e5672e5d8663b5824fa69818eb574e2ee4`

Every manifest entry lands in exactly one bucket — pass, fail, documented divergence, or out-of-scope WITH its reason (no silent skips). Pass rate is `(pass + divergence) / run`; out-of-scope entries are excluded from the rate but counted in coverage.

## RDF Semantics entailment (rdf-tests rdf/rdf11/rdf-mt)

Premise → `sparq-reason` RDFS/RDF materialization (plus the harness-side finite axiomatic/reflexive augmentation the production materializer deliberately omits) → blank-node-homomorphism entailment check; `mf:result false` = (in)consistency check with the per-test recognized-datatype set.

| suite | pass | fail | divergence | out-of-scope | pass-rate (of run) |
|---|---:|---:|---:|---:|---:|
| rdf-mt | 48 | 0 | 0 | 0 | 100.0% |
| **total** | **48** | **0** | **0** | **0** | **100.0%** |

**Overall (RDF Semantics entailment (rdf-tests rdf/rdf11/rdf-mt)): 48 pass / 0 fail / 0 documented divergence / 0 out-of-scope — pass+divergence 100.0% of run, 100.0% of all in-scope tests.**

## OWL 2 RL (W3C OWL WG test cases, RDF-based semantics)

| suite | pass | fail | divergence | out-of-scope | pass-rate (of run) |
|---|---:|---:|---:|---:|---:|
| **total** | **0** | **0** | **0** | **0** | **—** |

**Overall (OWL 2 RL (W3C OWL WG test cases, RDF-based semantics)): 0 pass / 0 fail / 0 documented divergence / 0 out-of-scope — pass+divergence — of run, — of all in-scope tests.**

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

**48 pass / 0 fail / 0 documented divergence / 0 out-of-scope — pass+divergence 100.0% of run, 100.0% of all in-scope tests.**
