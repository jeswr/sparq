---
name: shacl-validation
description: "Validate RDF data against SHACL shapes with the sparq engine: SHACL Core constraints (class, datatype incl. the SHACL-1.2 disjunctive list form, cardinality, ranges, paths, logical, node/property, qualified, closed incl. sh:ByTypes, in/hasValue, and the SHACL-1.2 list constraints sh:memberShape / sh:uniqueMembers / sh:min+maxListLength / sh:uniqueValuesFor), SHACL-SPARQL sh:sparql constraints (§5.2), and custom SPARQL-based constraint components (sh:ConstraintComponent, §6) — then read the conformance/violations validation report (W3C report vocabulary as Turtle or human text). Also runs opt-in SHACL Advanced Features (SHACL-AF) rules — sh:rule (sh:TripleRule + sh:SPARQLRule) — to INFER triples (feature `shacl-af`). Use when an agent needs to check whether a sparq_core::Graph conforms to shapes, run shape validation, produce a SHACL validation report, or apply SHACL rules to infer/expand a graph in Rust."
---

# sparq-shacl-validation

Validate a data `Graph` against a shapes `Graph` and get back a `ValidationReport`
(conformance flag + per-violation results, renderable as W3C-vocabulary Turtle or
plain text). Covers the full SHACL Core component set, SHACL-SPARQL (`sh:sparql`,
§5.2), and custom SPARQL-based constraint components (`sh:ConstraintComponent`, §6).

`sparq-shacl` is an **opt-in** crate: depending on it is what turns on SHACL. It is
NOT a dependency of any other sparq crate by default, so the core engine and the
default wasm bundle carry zero SHACL code/cost unless you pull it in. The browser/JS
consumer opts in through `sparq-wasm`'s non-default `shacl` feature, which exposes
`validate` as a stateless `Store.validate(data, shapes, format)` wasm binding
returning a JSON report — a drop-in for `rdf-validate-shacl` (sq-yqi1, #162). On that
wasm32 build the `sparq-engine` dep drops its defaults so rayon never enters the
bundle; see the `javascript-wasm` skill for the JS API + report shape.

For the showcase site there is also a **standalone, lazy-loaded** wasm bundle,
`sparq-shacl-wasm` (the tier-b "W-shacl" artifact, sq-lfmf), kept separate from the lean
default bundle so SHACL never ships on the landing page. It exposes a stateless
`Validator` with the FULL report surface — `Validator.validate(data, shapes, format)`
(JSON report), `validateTurtle` (report-RDF in the `sh:ValidationReport` vocabulary),
`validateText` (human-readable), and `conforms(..., violationsOnly)` (the W3C
`sh:conforms` flag, or a violations-only gate). SHACL-AF `sh:rule` validation is behind
its opt-in `shacl-af` feature. See `crates/sparq-shacl-wasm/README.md`.

## Quickstart

`Cargo.toml`:

```toml
[dependencies]
sparq-core  = { path = "../sparq-core" }   # or version = "0.1"
sparq-shacl = { path = "../sparq-shacl" }  # or version = "0.1"
```

```rust
use sparq_core::Graph;

let data = Graph::load_str(r#"
    @prefix ex: <http://example.org/> .
    ex:alice a ex:Person ; ex:age "thirty" .   // age is a string, not an integer
"#, "turtle").unwrap();

let shapes = Graph::load_str(r#"
    @prefix sh:  <http://www.w3.org/ns/shacl#> .
    @prefix ex:  <http://example.org/> .
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    ex:PersonShape a sh:NodeShape ;
      sh:targetClass ex:Person ;
      sh:property [ sh:path ex:age ; sh:datatype xsd:integer ; sh:minCount 1 ] .
"#, "turtle").unwrap();

let report = sparq_shacl::validate(&data, &shapes);
assert!(!report.conforms);                 // sh:conforms = false
assert_eq!(report.results.len(), 1);       // one DatatypeConstraintComponent violation
eprintln!("{}", report.to_text());         // human-readable
println!("{}", report.to_turtle());        // W3C sh:ValidationReport graph
```

CLI-style end-to-end run via the bundled example (exits 0 iff the data conforms):

```sh
cargo run -p sparq-shacl --example validate -- data.ttl shapes.ttl
cargo run -p sparq-shacl --example validate -- data.ttl shapes.ttl --turtle
```

## Key APIs

Top-level functions (`sparq_shacl::…`):

```rust
// Parse shapes + validate in one call.
pub fn validate(data: &Graph, shapes: &Graph) -> ValidationReport;

// Validate against an ALREADY-parsed shapes model (amortise parsing across many graphs).
pub fn validate_with_model(data: &Graph, model: &ShapesModel) -> ValidationReport;

// Load Turtle resolving relative IRIs against a base (Graph::load_str has no base param).
pub fn load_turtle_with_base(text: &str, base: &str) -> Result<Graph, String>;

// Build a Graph from already-parsed oxrdf::Triples.
pub fn graph_from_triples<I: IntoIterator<Item = oxrdf::Triple>>(triples: I) -> Graph;
```

**SHACL Compact Syntax (SCS) parser** *(opt-in feature `scs`)* — the *parse*
direction of the W3C SCS (`sparq_shacl::scs::…`, re-exported at the crate root):

```rust
// Parse SCS text -> SHACL shapes triples (relative IRIs + the owl:Ontology subject
// resolve against `base`; pass DEFAULT_BASE for the no-`BASE` convention).
pub fn parse_scs(text: &str, base: &str) -> Result<Vec<oxrdf::Triple>, ScsError>;

// Same, then build a queryable Graph ready to feed `validate`.
pub fn parse_scs_to_graph(text: &str, base: &str) -> Result<Graph, ScsError>;

pub const DEFAULT_BASE: &str;   // "urn:x-base:default"
pub struct ScsError { pub line: usize, pub message: String }   // typed; never a silent mis-parse
```

It emits the SAME shapes triples `validate` consumes, so an SCS document validates
data identically to the equivalent Turtle. Covers the grammar the W3C `shacl12-cs`
corpus exercises (32/32 fixtures round-trip graph-isomorphically): directives,
`shape`/`shapeClass`, full path expressions, `[min..max]`, `nodeKind`, bare-IRI
`sh:datatype`-vs-`sh:class`, `@`shape-refs (`sh:node`), `param=value`, `!` (`sh:not`),
`|` (`sh:or`), nested `{...}` shapes (`sh:node`), and `[ ... ]` arrays (`sh:in` /
`sh:ignoredProperties`). The browser/JS surface exposes this as the opt-in
`Store.parseShaclCompact(text, base?)` wasm binding (sq-quly) — SCS text → the shapes
graph as a Turtle string, behind `sparq-wasm`'s non-default `scs` feature; see the
`javascript-wasm` skill for the JS API.

`ShapesModel` (`sparq_shacl::ShapesModel`):

```rust
pub fn ShapesModel::parse(shapes_graph: &Graph) -> ShapesModel;   // parse once, reuse
```

`ValidationReport` (`sparq_shacl::ValidationReport`):

```rust
pub conforms: bool;                         // true iff results is empty (spec sh:conforms — counts EVERY result)
pub results: Vec<ValidationResult>;
pub fn conforms_violations_only(&self) -> bool;                       // ignore sh:Warning / sh:Info
pub fn results_with_severity<'a>(&'a self, severity: &'a str)         // full IRI, e.g. ".../shacl#Warning"
    -> impl Iterator<Item = &'a ValidationResult>;
pub fn to_turtle(&self) -> String;          // W3C report vocabulary (valid, round-trippable Turtle)
pub fn to_text(&self) -> String;            // human-readable summary
```

`ValidationResult` (`sparq_shacl::ValidationResult`) — all fields public:

```rust
pub focus_node: oxrdf::Term;
pub path: Option<sparq_shacl::Path>;        // sh:resultPath (property shapes / sh:closed)
pub value: Option<oxrdf::Term>;             // offending value node
pub source_shape: oxrdf::Term;
pub source_component: String;               // constraint-component IRI, e.g. ".../MinCountConstraintComponent"
pub severity: String;                       // severity IRI (default ".../shacl#Violation")
pub messages: Vec<oxrdf::Term>;             // sh:message literals
pub default_message: String;
pub details: Vec<ValidationResult>;         // nested sh:detail sub-results (see below); empty for most components
pub fn effective_messages(&self) -> Vec<oxrdf::Term>;   // messages, or a generated default
```

`details` carries non-normative `sh:detail` sub-results that explain WHY a result
fired: a `sh:memberShape` violation lists one sub-result per non-conforming list
member (the actual results of validating that member against the member shape),
and a `sh:uniqueMembers` violation lists one sub-result per duplicated member
(`sh:value` = the duplicated term). `sh:detail` is non-normative — it never
affects `sh:conforms` and the W3C suite compares only top-level result fields —
so it is empty for every other component. `to_turtle` nests each detail as a
`sh:ValidationResult` blank node under `sh:detail`; `to_text` indents them.

`Path` (`sparq_shacl::Path`) — `Predicate | Inverse | Sequence | Alternative |
ZeroOrMore | OneOrMore | ZeroOrOne`; `path.to_turtle()` gives the Turtle path
expression used in `sh:resultPath`.

## Common recipes

**CI gating — fail on violations, allow warnings.** `report.conforms` counts every
result; use the severity-aware toggle so `sh:Warning`/`sh:Info` don't fail the build:

```rust
let report = sparq_shacl::validate(&data, &shapes);
if !report.conforms_violations_only() {
    eprintln!("{}", report.to_text());
    std::process::exit(1);
}
```

**Validate many data graphs against one shapes graph** — parse the shapes once:

```rust
let model = sparq_shacl::ShapesModel::parse(&shapes);
for data in data_graphs {
    let report = sparq_shacl::validate_with_model(&data, &model);
    // ...
}
```

**Inspect failures programmatically** instead of rendering:

```rust
for r in &report.results {
    let comp = r.source_component.rsplit(['#', '/']).next().unwrap();   // "MinCountConstraintComponent"
    println!("focus={} comp={comp} value={:?}", r.focus_node, r.value);
}
```

**SHACL-SPARQL (`sh:sparql`, §5.2)** — a constraint node carries an `sh:select`; it
runs per focus node with `$this` pre-bound (and `$PATH` on property shapes), and EACH
returned solution is one violation. `?value`→`sh:value` (defaults to the focus node
when unprojected), `?path`→`sh:resultPath`, `?message`→`sh:resultMessage`; `{?var}` /
`{$var}` templating in `sh:message`:

```rust
let shapes = Graph::load_str(r#"
    @prefix sh:  <http://www.w3.org/ns/shacl#> .
    @prefix ex:  <http://example.org/> .
    @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
    ex:PersonShape a sh:NodeShape ;
      sh:targetClass ex:Person ;
      sh:sparql [
        a sh:SPARQLConstraint ;
        sh:prefixes ex:p ;
        sh:message "Age must not be negative" ;
        sh:select """SELECT $this ?value WHERE {
            $this <http://example.org/age> ?value . FILTER (?value < 0) }""" ;
      ] .
    ex:p sh:declare [ sh:prefix "ex" ; sh:namespace "http://example.org/"^^xsd:anyURI ] .
"#, "turtle").unwrap();
let report = sparq_shacl::validate(&data, &shapes);   // source_component ends with "SPARQLConstraintComponent"
```

**SHACL-1.2 core constraints (always on, no feature flag).** The disjunctive
*set* spellings of `sh:datatype` / `sh:nodeKind` — `sh:datatype ( xsd:string
rdf:langString )`, `sh:nodeKind ( sh:BlankNode sh:IRI )` — conform a value node
when it matches ANY listed datatype / kind (the single-IRI form is the singleton
case). `sh:closed sh:ByTypes` is the "close by types" mode: the allowed-predicate
set is recomputed per value node from its `rdf:type`s (transitively through
`rdfs:subClassOf` / inbound `sh:targetClass` / `sh:node`, SHACL §4.8.1), unlike
`sh:closed true` which fixes it to the shape's own `sh:property` paths. The four
SHACL list constraints validate that each value node is a well-formed SHACL list:
`sh:memberShape` (every member conforms to a shape), `sh:uniqueMembers true`
(members pairwise distinct), `sh:min`/`sh:maxListLength` (member-count bounds),
and `sh:uniqueValuesFor` (the listed properties' values are unique across the
shape's target nodes — one IRI, or a SHACL list for a composite key). A value that
is not a well-formed SHACL list violates the list constraints; a node with no
values for any `sh:uniqueValuesFor` property is never reported.

**Custom SPARQL-based constraint component (`sh:ConstraintComponent`, §6).** Declare
the component (parameters + an `sh:ask`/`sh:select` validator) IN THE SHAPES GRAPH; it
activates on any shape that uses all its mandatory parameter predicates. Each parameter
value is pre-bound as `$paramName` alongside `$this`/`$value`:

```rust
let shapes = Graph::load_str(r#"
    @prefix sh:   <http://www.w3.org/ns/shacl#> .
    @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
    @prefix ex:   <http://example.org/> .
    # Component typed via a subclass of sh:ConstraintComponent (discovery follows rdfs:subClassOf*).
    ex:MyCC rdfs:subClassOf sh:ConstraintComponent .
    ex:MaxLenComponent a ex:MyCC ;
      sh:parameter [ sh:path ex:maxLen ] ;
      sh:validator [ a sh:SPARQLAskValidator ;
        sh:message "Value is longer than {$maxLen} characters" ;
        sh:ask "ASK { FILTER (STRLEN(STR($value)) <= $maxLen) }" ] .
    ex:S a sh:NodeShape ;
      sh:targetNode "abcdef", "ab" ;
      ex:maxLen 3 .                # using ex:maxLen activates the component on ex:S
"#, "turtle").unwrap();
// source_component is the component IRI; ASK=false → violation. sh:nodeValidator /
// sh:propertyValidator are preferred over the generic sh:validator by shape kind (§6.2.2).
```

On a PROPERTY shape, a validator that references the `$PATH` variable gets it
pre-bound to the shape's property path (SHACL §6.3). Because `$PATH` is a SPARQL
property PATH (not a term), it is bound — like the §5.2 `sh:sparql` path — by
re-parsing the validator per property shape with the path's property-path form
textually substituted, rather than via the VALUES table the term bindings
(`$this` / `$value` / `$paramName`) use. The re-parsed per-shape validator is
held off the public `Component` enum in a crate-private store; the public
`Component::CustomSparql { component, args, path_validator }` variant carries
only an `Option<usize>` index into it (`path_validator`), present when the
shape is a property shape, the chosen validator references `$PATH`, and the
substituted query re-parses — otherwise `None` and the component's shared
(path-free) validator is used as-is. Each `$paramName` variable is the LOCAL
NAME of the parameter's `sh:path` IRI (not its `sh:name` display label, §6.2.1).

**Relative-IRI test files / a base IRI** — `Graph::load_str` exposes no base, so use:

```rust
let g = sparq_shacl::load_turtle_with_base(&text, &format!("file://{path}")).unwrap();
```

**SHACL Advanced Features rules (`sh:rule` + `sh:values`, SHACL-AF) — INFER
triples** *(opt-in feature `shacl-af`)*. A shape's rules infer new triples for that
shape's focus nodes (its targets). Three rule types: `sh:TripleRule` (`sh:subject`
/ `sh:predicate` / `sh:object` node expressions — the inferred triples are the
cartesian product of the three evaluated sets), `sh:SPARQLRule` (an `sh:construct`
CONSTRUCT run per focus node with `$this` pre-bound), and the `sh:values` value
rule (a property shape with a single-predicate `sh:path` and an `sh:values` node
expression infers `(focus, predicate, v)` per evaluated `v`). Rules honour
`sh:condition` (fire only for focus nodes conforming to every condition shape),
`sh:order` (ascending, a rule sees earlier groups' inferences), and
`sh:deactivated`. The engine **iterates to a fixpoint** (bounded by
`rules::MAX_ITERATIONS = 100`); the input graph is never mutated.

`Cargo.toml`: `sparq-shacl = { path = "...", features = ["shacl-af"] }`

```rust
use sparq_core::Graph;
let data = Graph::load_str(r#"
    @prefix ex: <http://example.org/> .
    ex:alice a ex:Person ; ex:firstName "Alice" .
"#, "turtle").unwrap();
let shapes = Graph::load_str(r#"
    @prefix sh:  <http://www.w3.org/ns/shacl#> .
    @prefix ex:  <http://example.org/> .
    @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
    ex:PersonShape a sh:NodeShape ;
      sh:targetClass ex:Person ;
      sh:rule [ a sh:TripleRule ;                  # infer (this, rdf:type, ex:Agent)
        sh:subject sh:this ; sh:predicate rdf:type ; sh:object ex:Agent ] ;
      sh:rule [ a sh:SPARQLRule ;                  # infer a label from the first name
        sh:construct "CONSTRUCT { $this <http://example.org/label> ?n } WHERE { $this <http://example.org/firstName> ?n }" ] .
"#, "turtle").unwrap();

// The INFERRED triples only (data is not mutated):
let inf = sparq_shacl::apply_rules(&data, &shapes);   // -> sparq_shacl::Inference
//   inf.triples : Vec<oxrdf::Triple>   inf.iterations : usize   inf.capped : bool
// Or get a fresh graph of data ∪ inferred, ready to query/validate:
let expanded: Graph = sparq_shacl::expand(&data, &shapes);
```

**Node-expression algebra** (operand of `sh:subject`/`sh:predicate`/`sh:object`,
the `sh:values` value rule, and `sh:expression`): `sh:this` (focus node); a
constant IRI/literal; a path expression `[ sh:path P ; sh:nodes N? ]` (any SHACL
property path; the optional `sh:nodes` is itself a node expression giving the start
nodes, default `sh:this`); a filter-shape expression `[ sh:filterShape S ; sh:nodes
N ]` (the nodes of `N` conforming to shape `S`); `[ sh:intersection ( … ) ]`; `[
sh:union ( … ) ]`; a bare `rdf:list` (a SHACL 1.2 list expression — its members in
order, preserving duplicates); and the **function-expression form** (sq-mk9n). These
nest.

**Function registry (sq-mk9n):** the SHACL 1.2 built-in node-expression operators
(`shnex:`/`sh:`) — `concat`, `count`, `sum`, `min`, `max`, `distinct`,
`if`/`then`/`else`, `exists`, `limit`, `offset`, `instancesOf`, `nodesMatching`,
`flatMap`, `findFirst`, `matchAll`, `remove`, `orderBy`, `var` (only `"focusNode"`
is bound outside a SPARQL scope) — plus a custom `sh:SPARQLFunction` IRI applied to
a `sh:list` of arguments (dispatched through the SPARQL engine with the ordered
`sh:parameter` variables pre-bound). An unregistered function IRI is dropped
(lenient), inferring nothing.

**`sh:values` value rule:** a property shape with a single-predicate `sh:path` and
an `sh:values` node expression infers `(focus, predicate, v)` for each evaluated
`v`. A value rule on a `sh:property` child of a targeted node shape ranges over the
parent's focus nodes.

**`sh:expression` constraint** (`sh:ExpressionConstraintComponent`): a value node
violates when its `sh:expression` node expression does NOT evaluate to `{ true }`
(value = focus on a node shape; each path value on a property shape).

**`sh:nodeByExpression` constraint** (`sh:NodeByExpressionConstraintComponent`):
like `sh:node`, but the node shape is *computed* by a node expression. For each
value node `v`, the expression is evaluated against `v` as focus to a set of
node-shape terms; `v` violates when it does NOT conform to one of them. A constant
IRI expression is the `sh:node` special case; an expression result naming no parsed
shape is skipped (lenient).

API: `apply_rules(data, shapes)`, `apply_rules_with_model(data, shapes, &model)`
(amortise shape parsing), `expand(data, shapes) -> Graph`, the node-expression
seam `eval_node_expression(data, shapes, expr, focus) -> Option<Vec<Term>>`, and
the conformance primitive `conforms(data, shapes, shape_node) -> ConformanceCheck`
(call `.holds(node)` per focus). A gated W3C harness (`tests/w3c_node_expr.rs`)
drives the `sht:EvalNodeExpr` suite — all evaluation entries pass; a companion
harness (`tests/w3c_node_expr_constraints.rs`) drives the suite's two `sht:Validate`
entries (`sh:expression` / `sh:nodeByExpression`) end-to-end (both self-skip when
the suite is not fetched).

## Gotchas / feature flags / prerequisites

- **Base SHACL is engaged purely by depending on `sparq-shacl`** (no feature
  needed). It transitively pulls in `sparq-engine` (to run `sh:sparql`/§6 queries).
  Neither is in the **default** wasm dependency graph, so the default browser bundle
  stays SHACL-free; they enter the wasm graph ONLY when a consumer opts in via
  `sparq-wasm`'s non-default `shacl` feature, on which build `sparq-engine`'s defaults
  (rayon/regex/digest) are dropped so the bundle stays lean. The native build is
  unaffected (full engine defaults).
- **SHACL-AF rules (`sh:rule`) are OPT-IN behind the `shacl-af` cargo feature.**
  With the feature off, the base validation path carries zero rule code/parse cost
  and the `apply_rules` / `apply_rules_with_model` / `expand` / `Inference` symbols
  are absent. SHACL-AF rules are an INFERENCE step (they produce triples), not a
  validation step — they do not affect `validate(..)`'s report; validate the
  `expand(..)`-ed graph if you want constraints to see inferred triples.
- **The SHACL Compact Syntax parser is OPT-IN behind the `scs` cargo feature.**
  With it off the `scs` module and the `parse_scs` / `parse_scs_to_graph` / `ScsError`
  / `DEFAULT_BASE` symbols are absent (zero parser code compiled in). It adds no new
  dependencies. Coverage is honest: any construct outside the supported grammar
  returns a typed `ScsError` rather than mis-parsing. Both the SCS parse and the
  reference Turtle must resolve relative IRIs against the same `base` to agree, so
  the round-trip test passes the fixture's `BASE` (or `DEFAULT_BASE`) to both sides.
- **Rule fixpoint is bounded.** `apply_rules` iterates the rule schedule until a
  pass infers nothing, capped at `rules::MAX_ITERATIONS` (100); `Inference::capped`
  flags a non-terminating rule set (e.g. a CONSTRUCT minting a fresh blank node each
  pass) whose inferred set may be incomplete.
- **`sh:conforms` counts EVERY result regardless of severity** (matches the W3C suite).
  For "warnings don't fail" gating use `conforms_violations_only()` /
  `results_with_severity(..)`, NOT `conforms`.
- **Ill-formed shapes are skipped, not errored.** A shape never declared, an
  unparsable path, or an `sh:select` that fails to parse (e.g. undeclared prefix)
  contributes no results; the rest of validation still runs. `validate` never returns
  a `Result`/panics on bad shapes — so a silently-empty report can mean "no targets"
  rather than "conforms".
- **Results are NOT deduplicated** across traversal routes / component occurrences — a
  nested shape reached via two parents reports twice (intentional, matches the suite).
- **Recursion is treated as conforming.** Re-entering the same (focus, shape) pair
  counts as conforming (SHACL leaves recursion undefined); cyclic `sh:node`/`sh:property`
  terminate without stack overflow.
- **`sh:sparql` pre-binding:** `$this` (and `$PATH` on property shapes) is injected via
  an algebra-level VALUES on the parsed query — it lands below solution modifiers, so
  `LIMIT`/`ORDER BY`/`DISTINCT`/`SELECT *` behave correctly. `sh:prefixes` chases
  `sh:declare`(`sh:prefix`/`sh:namespace`) transitively through `owl:imports`.
- **§6 limits:** the W3C `sparql/component/*` suite `owl:imports` the external
  `http://datashapes.org/dash` vocabulary; it is run offline (`tests/w3c_sparql_component.rs`)
  by resolving that import against a vendored, minimal pinned excerpt at
  `crates/sparq-shacl/tests/vendor/dash.ttl`. Full `sparql/pre-binding` semantics (rejecting
  variable re-binding, `$shapesGraph`) are out of scope — see the crate's open beads (`bd list -l area:sparq-shacl`).
- **W3C conformance:** 98/98 of the core `sht:Validate` suite passes. Reproduce with
  `crates/sparq-shacl/fetch-shacl-tests.sh` then
  `cargo test -p sparq-shacl --test w3c_core` (self-skips if the gitignored suite is absent).
- §6 SPARQL-based constraint *components* are implemented and tested
  (`tests/sparql_components.rs` plus the W3C `sparql/component` sub-suite in
  `tests/w3c_sparql_component.rs`); the crate README documents them under
  "Supported constraint components".

## See also

- `sparql-query` — running standalone SPARQL through `sparq-engine` (what `sh:sparql`
  routes through).
- `graph-loading` / `compressed-ingest` — building the `sparq_core::Graph` you validate.
- `fused-decompress-parse`, `hdt-format` — alternative ingest paths feeding a `Graph`.
