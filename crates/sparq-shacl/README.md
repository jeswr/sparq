# sparq-shacl

Opt-in **SHACL Core + SHACL-SPARQL (`sh:sparql`)** validation over
[`sparq_core::Graph`]s.

Parses a shapes graph into a shapes model, evaluates every SHACL Core
constraint component against a data graph by direct, index-backed permutation
scans (no SPARQL round-trip), evaluates `sh:sparql` constraints by routing their
`sh:select` through `sparq-engine`, and produces a `ValidationReport` with:

- the parsed results (`conforms` + `Vec<ValidationResult>` carrying
  focus node / result path / value / source shape / source constraint
  component / severity / messages),
- `to_turtle()` — the report as an RDF graph in the W3C SHACL
  validation-report vocabulary (valid Turtle, round-trips through a parser),
- `to_text()` — a human-readable rendering.

Like `sparq-reason`, this crate is **isolated**: it is not a dependency of any
other sparq crate, so the core engine and the default wasm bundle carry zero SHACL
code, dependencies or runtime cost unless a consumer opts in. The browser/JS
consumer opts in through `sparq-wasm`'s non-default `shacl` feature, which exposes
`validate` as a stateless `Store.validate(data, shapes, format)` wasm binding
returning a JSON report (a drop-in for `rdf-validate-shacl`); on that wasm32 build
`sparq-engine` drops its defaults so rayon never enters the bundle.

## Status: W3C SHACL core test suite

**98 / 98 (100%)** of the `sht:Validate` entries in the core section of the
official [w3c/data-shapes](https://github.com/w3c/data-shapes) test suite
(`data-shapes-test-suite/tests/core/...`, pinned commit `b6e73695`), covering
`node`, `property`, `path`, `targets`, `misc`, `complex` (including the
SHACL-validating-SHACL meta test) and `validation-reports`.

Run it yourself (the suite is fetched into a gitignored directory; the test
self-skips when it is absent):

```sh
crates/sparq-shacl/fetch-shacl-tests.sh
cargo test -p sparq-shacl --test w3c_core -- --nocapture
```

## Differential fuzzing against reference engines

Beyond the fixed W3C suite, a **differential fuzzer** (`tests/diff_fuzz.rs` +
`tests/gen.rs`) generates random-but-valid SHACL shapes + data graphs from a
seed, validates each through `sparq-shacl` **and** through a reference SHACL
engine, and asserts the reports agree on the `sh:conforms` bit and the
per-focus-node violated-constraint set (deduplicated; blank-node and
complex-path tolerant — see the comparison policy in the test's module docs).
The generator is a deterministic SplitMix64 loop (the same idiom as
`sparq-bench`'s engine-vs-Oxigraph differential), so any disagreement reproduces
from its printed seed.

The reference side is a pluggable "report-cli" adapter (bead sq-eifd): a
subprocess reading `{data, shapes}` Turtle and emitting a normalised JSON
report. The first wired reference is **pySHACL**
(`tests/diff_fuzz/pyshacl_adapter.py`); other engines (rdf-validate-shacl /
shacl-engine via Node, Jena-SHACL via a jar) are tracked as follow-up beads and
slot in as alternative adapters producing the same JSON shape.

It is `#[ignore]`d (off the per-PR fast path) and runs as a **nightly** CI lane
(`.github/workflows/shacl-diff-fuzz.yml`). Run it locally against a Python that
has pySHACL + rdflib installed:

```sh
python3 -m venv /tmp/shacl-ref-venv && /tmp/shacl-ref-venv/bin/pip install pyshacl
SHACL_DIFF_PYTHON=/tmp/shacl-ref-venv/bin/python SHACL_DIFF_COUNT=2000 \
  cargo test -p sparq-shacl --test diff_fuzz -- --ignored --nocapture
```

When no reference engine resolves, the test skips cleanly (so a fresh checkout
stays green). Two fast, reference-free self-tests of the generator
(well-formedness + outcome mix, and key-normalisation consistency) DO run in the
per-PR path.

## Supported constraint components

`sh:class`, `sh:datatype` (with XSD lexical-space ill-formedness checks),
`sh:nodeKind`, `sh:minCount`/`sh:maxCount`,
`sh:minInclusive`/`sh:minExclusive`/`sh:maxInclusive`/`sh:maxExclusive`
(numeric, string, boolean and date/time orderings, including the
timezone-presence comparability rule), `sh:minLength`/`sh:maxLength`,
`sh:pattern` (+`sh:flags`), `sh:languageIn`, `sh:uniqueLang`, `sh:equals`,
`sh:disjoint`, `sh:lessThan`, `sh:lessThanOrEquals`, `sh:not`, `sh:and`,
`sh:or`, `sh:xone`, `sh:node`, `sh:property`, `sh:qualifiedValueShape`
(+`sh:qualifiedMinCount`/`sh:qualifiedMaxCount`/
`sh:qualifiedValueShapesDisjoint`), `sh:closed` (+`sh:ignoredProperties`),
`sh:hasValue`, `sh:in`.

**SHACL-SPARQL:** `sh:sparql` (the SPARQL-based constraint component, §5.2) —
the `sh:select` runs per focus node with `$this` pre-bound (and `$PATH` on
property shapes), each solution a violation; honours `sh:prefixes`
(`sh:declare`/`sh:prefix`/`sh:namespace`, with `owl:imports` chasing) and
`sh:message` (`{?var}` templating). Maps `?value`→`sh:value` (defaulting to the
focus node when unprojected), `?path`→`sh:resultPath`, `?message`→
`sh:resultMessage`. Pinned by the W3C `sparql/node` + `sparql/property`
sub-suites.

**SPARQL-based constraint *components* (custom `sh:ConstraintComponent`, §6):**
implemented. A component node — typed `sh:ConstraintComponent` or any
`rdfs:subClassOf*` descendant — declaring `sh:parameter`s and a validator
(`sh:validator`, or the kind-specific `sh:nodeValidator`/`sh:propertyValidator`,
chosen per shape kind, §6.2.2) activates on any shape that carries the
parameter predicates. The validator runs with `$this`, `$value`, each parameter
VALUE (`$paramName`) and (on a property shape) `$PATH` pre-bound; `sh:ask`
validators run per value node (`false` ⇒ violation), `sh:select` validators run
per focus node (each solution a violation, §6.3). `sh:optional true` parameters
need not be present. The component's IRI is the
`sh:sourceConstraintComponent`. The remaining §6 limit is the full
`sparql/pre-binding` semantics (rejecting variable re-binding, `$shapesGraph`) —
see the open beads for this crate (`bd list -l area:sparq-shacl`).

**SHACL Advanced Features rules (`sh:rule` + `sh:values`, SHACL-AF) — opt-in
feature `shacl-af`:** an *inference* step (it produces triples; it is not part of
`validate(..)`). A shape's rules infer triples for that shape's focus nodes.
Three rule types:

- `sh:TripleRule` — `sh:subject`/`sh:predicate`/`sh:object` node expressions; the
  inferred triples are the cartesian product of the three evaluated sets.
- `sh:SPARQLRule` — an `sh:construct` CONSTRUCT run per focus node with `$this`
  pre-bound, reusing the `sh:sparql` engine path; honours `sh:prefixes`.
- `sh:values` value rule — a property shape with a single-predicate `sh:path` and
  an `sh:values` node expression infers `(focus, predicate, v)` for every `v` in
  the evaluated set (the canonical "derive these values" rule). A value rule on a
  `sh:property` child of a targeted node shape fires for the parent's focus nodes.

**Node-expression algebra** (the operand of `sh:subject`/`sh:predicate`/`sh:object`
and `sh:values`): the focus node `sh:this`, a constant IRI/literal, a path
expression `[ sh:path P ; sh:nodes N? ]` (over any SHACL property path; the
optional `sh:nodes` is itself a node expression giving the start nodes, defaulting
to `sh:this`), a filter-shape expression `[ sh:filterShape S ; sh:nodes N ]` (the
nodes of `N` that conform to shape `S`), `[ sh:intersection ( … ) ]`,
`[ sh:union ( … ) ]`, a bare `rdf:list` (a SHACL 1.2 *list expression* — its
members in order, preserving duplicates), and the **function-expression form**
backed by a SHACL-function registry (sq-mk9n). These nest arbitrarily.

The function registry implements the SHACL 1.2 built-in node-expression operators
(`shnex:`/`sh:`): `concat`, `count`, `sum`, `min`, `max`, `distinct`,
`if`/`then`/`else`, `exists`, `limit`, `offset`, `instancesOf`, `nodesMatching`,
`flatMap`, `findFirst`, `matchAll`, `remove`, `orderBy`, and `var` (only the
`"focusNode"` binding is defined outside a SPARQL scope; any other variable is the
empty set). A **custom `sh:SPARQLFunction`** IRI applied to a `sh:list` of
arguments is dispatched through the SPARQL engine (its `sh:select` body run with the
ordered `sh:parameter` variables pre-bound to the evaluated arguments). A function
IRI with no registered implementation is dropped (lenient), so a rule that uses an
unknown function infers nothing rather than misfiring.

**`sh:expression` constraint** (`sh:ExpressionConstraintComponent`, SHACL-AF) — a
value node `v` is a violation when its `sh:expression` node expression does NOT
evaluate to `{ true }` for `v`. On a node shape `v` is the focus node; on a
property shape it is each path value node.

**`sh:nodeByExpression` constraint** (`sh:NodeByExpressionConstraintComponent`,
SHACL-AF) — like `sh:node`, but the node shape is *computed* by a node expression
rather than fixed. For each value node `v`, the expression is evaluated against
`v` as focus to a set of node-shape terms; `v` is a violation when it does NOT
conform to one of them. A constant IRI expression is the `sh:node` special case
(it evaluates to that one shape); a path-values expression locates the shape(s)
dynamically. An expression result that names no parsed shape is skipped (lenient).

Rules honour `sh:condition` (fire only for focus nodes conforming to every
condition shape), `sh:order` (ascending; a rule sees earlier groups' inferences)
and `sh:deactivated`. The schedule is **iterated to a fixpoint** bounded by
`rules::MAX_ITERATIONS` (100; `Inference::capped` flags a non-terminating set).
Entry points: `apply_rules(data, shapes) -> Inference`, `apply_rules_with_model`,
`expand(data, shapes) -> Graph` (data ∪ inferred; the input is never mutated),
plus the node-expression seam `eval_node_expression(data, shapes, expr, focus) ->
Option<Vec<Term>>` and the conformance primitive `conforms(data, shapes,
shape_node)` (a reusable `ConformanceCheck`). With the feature off, none of this is
compiled in. A gated W3C conformance harness
([`tests/w3c_node_expr.rs`](tests/w3c_node_expr.rs)) drives the `sht:EvalNodeExpr`
suite for the implemented forms — all evaluation entries (focus/constant/list/path/
filter/intersection/union plus the registry's function operators) pass; it
self-skips when the suite is not fetched. A companion gated harness
([`tests/w3c_node_expr_constraints.rs`](tests/w3c_node_expr_constraints.rs)) drives
the suite's two `sht:Validate` constraint entries (`expression-001`,
`nodeByExpression-001`) end-to-end through `validate`, comparing the produced
report to the expected one.

Targets: `sh:targetNode`, `sh:targetClass` (with `rdfs:subClassOf*` closure),
implicit class targets (a shape that is itself an `rdfs:Class`),
`sh:targetSubjectsOf`, `sh:targetObjectsOf`.

Property paths: predicate paths plus all SHACL path forms — sequence,
alternative, inverse, `zeroOrMore`/`oneOrMore`/`zeroOrOne` — evaluated by
direct graph walks (BFS closure for the recursive forms).

Also handled: `sh:severity`, `sh:message` (copied to `sh:resultMessage`),
`sh:deactivated`, recursive shape references (re-entrant validation of the
same focus/shape pair counts as conforming — SHACL leaves recursion
undefined).

## Usage

```rust
use sparq_core::Graph;

let data = Graph::load_str(r#"
    @prefix ex: <http://example.org/> .
    ex:alice a ex:Person ; ex:age "thirty" .
"#, "turtle")?;

let shapes = Graph::load_str(r#"
    @prefix sh: <http://www.w3.org/ns/shacl#> .
    @prefix ex: <http://example.org/> .
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    ex:PersonShape a sh:NodeShape ;
      sh:targetClass ex:Person ;
      sh:property [ sh:path ex:age ; sh:datatype xsd:integer ; sh:minCount 1 ] .
"#, "turtle")?;

let report = sparq_shacl::validate(&data, &shapes);

if !report.conforms {
    // Human-readable summary:
    eprintln!("{}", report.to_text());
    // Or the standard report graph:
    println!("{}", report.to_turtle());
}

// Severity-aware gating: `conforms` counts EVERY result (the spec's
// sh:conforms); for CI-style "warnings don't fail the build" use:
if report.conforms_violations_only() { /* only sh:Warning / sh:Info results */ }
```

A CLI-style end-to-end run via the bundled example
([`examples/validate.rs`](examples/validate.rs)):

```sh
cargo run -p sparq-shacl --example validate -- data.ttl shapes.ttl
# or the report graph instead of the text rendering:
cargo run -p sparq-shacl --example validate -- data.ttl shapes.ttl --turtle
```

It prints the report and exits 0 iff the data conforms.

For repeated validation of many data graphs against one shapes graph, parse
the shapes once:

```rust
let model = sparq_shacl::ShapesModel::parse(&shapes);
for data in data_graphs {
    let report = sparq_shacl::validate_with_model(&data, &model);
}
```

## Scope and non-goals

- **SHACL Core + `sh:sparql` + custom §6 components.** SHACL Core, `sh:sparql`
  (§5.2) and the SPARQL-based constraint *component* declaration machinery
  (custom `sh:ConstraintComponent` with `sh:parameter` / `sh:validator`, §6) are
  all implemented (see "Supported constraint components" above). What remains out
  of scope is the full `sparql/pre-binding` semantics (rejecting variable
  re-binding, `$shapesGraph`); see the open beads for this crate
  (`bd list -l area:sparq-shacl`).
- Validation results are **not deduplicated** across traversal routes /
  component occurrences — matching the test suite's expectations (a nested
  shape reached through two parents reports twice).
