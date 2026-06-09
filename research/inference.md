# Inference / reasoning for sparq — architecture + roadmap

Goal (user): **full inference support — RDFS, OWL, and Notation3 (N3) — opt-in at build,
with no impact on the default engine's performance or memory.** For N3, **feature parity
with the EYE reasoner / eye-js**, validated against EYE's test suite.

## Architecture (decided + in place)

Reasoning lives in a **separate crate, `crates/sparq-reason`**, depending on `sparq-core`.
Consequences:
- The core engine and the default build carry **zero reasoning code, deps, or runtime cost**
  — `sparq-reason` is not compiled unless depended upon. The query hot path, the 6-perm
  store, the WASM bundle, and the measured ingest/memory numbers are all unchanged.
- Reasoning is **materialization** (forward-chaining: compute the deductive closure and add
  the entailed triples), the right fit for a dictionary-encoded store — rules are joins over
  fixed-width integer ids, computed **once at load/build time**, after which querying is
  exactly as fast as before. This is RDFox's model.
- The seam: `sparq-core` exposes `Graph::parse_to_triples` → (reason here) →
  `Graph::from_parts`. A caller that wants reasoning (the CLI, or a future build path)
  interposes `sparq_reason::materialize(profile, &mut dict, &mut triples)` between them.
- CLI surface: `--reason <profile>` on `query`, and a `reason <data> <format> <profile>
  [out.nt]` subcommand that writes the materialized closure.

## Status

### ✅ RDFS — DONE (materialization)
`sparq_reason::materialize_rdfs`. Forward-chaining fixpoint over the **useful, non-explosive**
RDFS rules (RDF 1.1 Semantics §9.2.1): rdfs2 (domain), rdfs3 (range), rdfs5 (subPropertyOf
transitivity), rdfs7 (subPropertyOf entailment), rdfs9 (type up subClassOf), rdfs11
(subClassOf transitivity). Deliberately omits the axiomatic / reflexive / `rdfs:Resource`
rules (rdfs4a/4b/6/8/10/13) — true but useless and O(terms)-explosive; this matches the
"RDFS" rule set materialized by GraphDB/RDF4J. Naive fixpoint (re-derive to stability) for
obvious correctness; RDFS closures converge in ≈ hierarchy-depth rounds. Tested: subclass
transitivity, domain/range typing, subproperty entailment, the rdfs7→rdfs2 interaction,
idempotency, and end-to-end via the CLI (`query … --reason rdfs` returns the entailed
answers; 0 without).

### ▢ OWL — NEXT (OWL 2 RL profile, materialization)
OWL 2 **RL** is the materialization-friendly profile (the others — EL, QL — target different
algorithms; full OWL DL is undecidable for forward chaining). RL extends the same fixpoint
engine with property and class axioms, all expressible as datalog-style rules over ids:
- **Property:** `owl:sameAs` (+ the replacement rules eq-rep-s/p/o, eq-trans/sym),
  `owl:inverseOf`, `owl:SymmetricProperty`, `owl:TransitiveProperty`,
  `owl:equivalentProperty`, `owl:propertyChainAxiom`, `owl:FunctionalProperty` /
  `owl:InverseFunctionalProperty` (these derive `sameAs`).
- **Class:** `owl:equivalentClass`, `owl:intersectionOf`, `owl:unionOf` (T-side),
  `owl:someValuesFrom` / `owl:allValuesFrom` / `owl:hasValue`, `owl:Restriction`.
- **Consistency:** `owl:disjointWith`, `owl:differentFrom`, `owl:complementOf` produce
  *clashes* — RL reasoners report inconsistency rather than materializing.
Plan: a `Profile::OwlRl` reusing the `Schema`/fixpoint scaffold; the `sameAs` closure +
canonicalization is the one structurally new piece (union-find over ids, then rewrite). The
W3C OWL 2 RL/RDF rules table (`prp-*`, `cls-*`, `cax-*`, `eq-*`, `scm-*`) is the spec to
implement and the OWL test suite the validation set. Sizeable but mechanical.

### ▢ Notation3 / EYE parity — the large subsystem (separate design)
N3 is a **superset of Turtle** adding: graph-term literals `{ … }`, rules
`{ premise } => { conclusion }` (log:implies), quantification (`@forAll`/`@forSome`,
universals via `?x`), and a large **builtin** library (`math:`, `string:`, `list:`, `log:`,
`time:`, `crypto:`, …). EYE is a mature reasoner (Euler-path / backward-chaining resolution
with forward closure) with **hundreds of builtins**; "parity" is a genuinely large,
multi-stage target. Decomposition:
1. **N3 parser** — oxttl parses Turtle but NOT N3 rules/formulae. Need an N3 parser
   producing quads + rule terms (formulae as first-class graph terms, variables). Either
   extend a Turtle tokenizer or port from `eye-js`/N3.js's grammar.
2. **Rule engine** — N3 rules are Horn-ish with builtins; EYE does backward chaining with a
   forward-closure driver. A semi-naive forward chainer over the rule set covers the common
   `=>` materialization cases; full EYE parity (nested formulae, scoped negation
   `log:notIncludes`, `log:collectAllIn`, proof output) needs the resolution engine.
3. **Builtins** — implement the EYE builtin set incrementally, prioritized by test-suite
   coverage: `math:*`, `string:*`, `list:*`, `log:equalTo/notEqualTo/includes`, `graph:*`.
4. **Validation** — run **EYE's own test suite** (the `eye` repo's `cases/` + the
   `eye-js`/`n3-tests`) differentially: same input N3 → compare derived closure / proof to
   EYE's output. This is the parity gate.
Honest scope: RDFS (done) and OWL-RL (next) are bounded datalog materialization. **N3/EYE
parity is a multi-month subsystem** — its own crate module, parser, engine, and a long tail
of builtins measured against EYE's suite. It will land incrementally, parity-by-test.

## Why materialization (not query-rewriting/backward-chaining) for RDFS+OWL-RL
A dictionary-encoded, permutation-indexed store makes the forward closure cheap (integer
joins) and keeps the query path a plain SPARQL evaluation over a bigger triple set — no
per-query reasoning overhead, no planner changes. The cost is build-time + store growth,
both opt-in. (Backward-chaining / query-rewriting would suit a frequently-updated store or
huge closures; a future `Profile` could add it. N3 inherently needs (backward) chaining,
handled in its own engine.)

## Out-of-core reasoning (future)
RDFS/OWL-RL materialization currently runs on the in-memory `parse_to_triples` path (fits
datasets that fit RAM). Billion-scale materialization (à la RDFox out-of-core) — running the
fixpoint over the external-merge build — is a later step; the TBox is small and fits RAM, so
the data rules can stream, but the `sameAs` closure and dedup need care at scale.
