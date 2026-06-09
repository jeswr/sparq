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

### ◐ OWL 2 RL — core subset DONE (materialization)
`sparq_reason::materialize_owl_rl` (`--reason owl`). Same forward-chaining fixpoint, adding
the W3C OWL 2 RL/RDF rules: **equality** eq-sym/eq-trans/eq-rep-s,p,o (`owl:sameAs` closure +
substitution); **property** prp-inv1/2 (`owl:inverseOf`), prp-symp (`SymmetricProperty`),
prp-trp (`TransitiveProperty`), prp-eqp1/2 (`equivalentProperty`), prp-fp/ifp
(`Functional`/`InverseFunctionalProperty` → `sameAs`); **class** cax-eqc1/2
(`equivalentClass`). Includes the RDFS rules (RL subsumes them, via the shared `rdfs_round`).
Now ALSO covers (via an RDF-list decoder + restriction/class-list extraction): **prp-spo2**
(`owl:propertyChainAxiom` — n-ary property-chain join) and the **class-expression rules**
cls-svf1 (`someValuesFrom`), cls-avf1 (`allValuesFrom`), cls-hv1/hv2 (`hasValue`), cls-int1/2
(`intersectionOf`), scm-uni (`unionOf`). 6 unit tests (incl. property chain uncle = parent∘
brother, hasValue/someValuesFrom restrictions, intersectionOf) + end-to-end CLI. The `sameAs`
rules use spec eq-rep substitution; union-find canonicalization is a future optimization.
**Consistency** ✅ — `sparq_reason::inconsistencies` detects cax-dw (`disjointWith`), cls-com
(`complementOf`), eq-diff (`sameAs`∩`differentFrom`), cls-nothing (`owl:Nothing`) clashes
post-materialization; the CLI reports them under `--reason owl`. **Remaining for full RL:**
cardinality (`maxCardinality` 0/1, `maxQualifiedCardinality`), `hasKey`, and the remaining
schema rules (scm-* for full class/property-hierarchy completeness).

### ▢ OWL — remaining (cardinality, hasKey, scm-*)
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

### ◐ Notation3 / EYE parity — FOUNDATION DONE, builtins expanding
`sparq_reason::n3` (`--reason n3` / `reason <f> n3 n3`). The core subsystem is in place:
- **Parser** (`n3/parser.rs`) — hand-rolled recursive descent (oxttl can't do N3): `@prefix`/
  `@base`, prefixed names, `<iri>`, `a`, literals (string/typed/lang/int/decimal/double/
  bool), `_:blank`, `?var`, `{ … }` formulae, `( … )` collections → rdf:List, `[ … ]` bnode
  property lists, `;`/`,`, and the `=>` (log:implies) / `=` (sameAs) sugar.
- **Forward-chaining engine** (`n3/mod.rs`) — fixpoint applying `{premise} => {conclusion}`
  rules with variable binding (conjunctive premise = nested-loop join over facts), chaining
  rules to closure, then interning the ground result into the dict. **Backward rules `<=`**
  are supported for the closure (`A <= B` ≡ `B => A`, reversed into a forward rule); true
  goal-directed backward chaining + proof output is a later addition. **Path syntax** `!`/`^`
  (`?x!:p` = the `:p`-object of `?x`; `?x^:p` = the `:p`-subject reaching `?x`) is desugared
  into fresh-variable triples that join in the premise.
- **Builtins:** `math:` comparison (`greaterThan`/`lessThan`/`notGreaterThan`/`notLessThan`/
  `equalTo`/`notEqualTo`) + functional (`sum`/`difference`/`product`/`quotient`/`max`/`min`);
  `string:` (`concatenation` + `contains`/`startsWith`/`endsWith`/`greaterThan`/`lessThan`);
  `list:` (`member`/`in` generators, `length`); `log:equalTo`/`notEqualTo` and
  `log:includes`/`notIncludes` (scoped negation-as-failure via recursive sub-formula match).
- Tested: the canonical Man⊢Mortal rule, a recursive transitive rule (fixpoint), a
  `math:greaterThan` age filter, and end-to-end chained rules (`age>17 ⊢ Adult ⊢ canVote`).

**Toward full EYE parity (ongoing, incremental — builtin-by-builtin against EYE's suite):**
1. **Functional builtins** — `math:sum`/`difference`/`product`/`quotient` (list-subject
   arithmetic, compute the object), `string:concatenation`/`string:contains`/etc.,
   `list:*`, `time:*`. These need in-rule list resolution (the `( )` collection is parsed;
   the builtin must read its members from the rule, not the data).
2. **`log:` builtins** beyond equality: `log:includes`/`log:notIncludes` (scoped negation),
   `log:collectAllIn`, `log:semantics`.
3. **Quantification & scope** — explicit `@forAll`/`@forSome`, nested formulae as data,
   `=>` with bound formula bodies; existentials in conclusions → fresh blank nodes.
4. **Backward chaining / proof** — EYE's Euler-path resolution + proof (`--pass`/`--proof`)
   output; needed for goal-directed queries and parity on the proof tests.
5. **Validation = EYE's own test suite.** Run EYE's `cases/` (the `eye`/`eye-js` repos)
   differentially: same N3 input → compare our ground closure to EYE's `--pass` output. Each
   passing case is a parity checkpoint; the failing ones drive the next builtin/feature. This
   is the parity gate the user specified.

### ▢ Notation3 / EYE — internals roadmap (original design notes)
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
