<!-- [OPUS-5] sq-lsp7k.13 — GraphQL read endpoint auto-generated from SHACL shapes: feasibility + design record. Research only; no implementation. -->
# GraphQL-from-SHACL (`sq-lsp7k.13`): feasibility, substrate audit, and a phased plan

**Bead** `sq-lsp7k.13` (issue #3037), parent `sq-lsp7k` (#2565). Status in the
repo's own planning records: **P3, deliberately deferred** —
[`competitive-feature-analysis-2026-07.md:76`](./competitive-feature-analysis-2026-07.md)
records it as `GAP (defer) — bounded opt-in crate (shapes→schema, read-only) once
forms land; low evidence of demand vs SPARQL/BI`, and
[`feature-research-broad-sparql-vendors.md:344`](./feature-research-broad-sparql-vendors.md)
classifies the same item as **`ambiguous-ask-user`** — *"large surface, debatable
fit for an RDF-first engine. Ask user."*

This record does the work the defer was waiting on: it audits whether the shape
substrate is actually ready, corrects the brief's premise, names the three hard
problems that decide whether this is an L or an XL, and proposes a phased plan.
**It does not implement anything.** The recommendation (§5) is that the bead stays
deferred, but that the *reason* changes from "substrate unproven" to "demand
unproven" — which is a maintainer question, not an engineering one.

## 0. Verdict up front

| Question | Answer |
|---|---|
| Is the stated soft dependency (forms F1) satisfied? | **Yes.** `sparq-forms` is shipped and real, not a stub. |
| Is the shape-model API sufficient to generate a GraphQL schema? | **No — not on its own.** See the premise correction in §1. |
| Is the effort estimate `L` right? | **Only for a schema emitter.** A genuine read *endpoint* with filtering/ordering/pagination + `explain` is XL. |
| Recommendation | Keep deferred. If it proceeds, ship **G1 (SDL emitter) alone** and stop; treat execution as a separate, separately-justified decision. |

## 1. Premise check — one correction, one confirmation

**Confirmed:** the brief's soft dependency ("do after forms F1 lands so the
shape-model API is proven") **is satisfied**. `crates/sparq-forms` is a shipped,
non-stub implementation (~2.3k src lines, ~975 test lines across 6 suites,
3 golden fixtures) that consumes the shape model in anger, and it has a
[README](../crates/sparq-forms/README.md) and a
[`skills/shacl-forms/SKILL.md`](../skills/shacl-forms/SKILL.md) surface. The
shape-model API has a second real consumer, which is exactly the "proven"
signal the defer was waiting for.

**Correction to the brief.** The issue describes the new crate as *"consuming
sparq-shacl shape model (node shapes->types, property shapes->fields)"*. That
description is **incomplete in a way that matters for scoping**: the parsed shape
model deliberately drops the presentation metadata that GraphQL type and field
*naming* depends on.

`Component` ([`model.rs:47`](../crates/sparq-shacl/src/model.rs)) carries the
*constraints* — but `sh:name`, `sh:description`, `sh:order` and `sh:group` are
**not** `Component` variants and are **not** stored on `Shape`
([`model.rs:379`](../crates/sparq-shacl/src/model.rs)). `sparq-forms` gets them by
going back to the shapes graph through `GraphView`
([`view.rs`](../crates/sparq-shacl/src/view.rs)) — e.g. `sh:group` at
[`derive.rs:226`](../crates/sparq-forms/src/derive.rs), the `sh:name` →
`rdfs:label` fallback at `derive.rs:573-574`, `sh:order` at `derive.rs:642`.

So the correct dependency statement is: **a GraphQL generator must consume the
`(ShapesModel, GraphView)` *pair*, not the shape model alone.** This is not a
blocker — both are public (`pub use model::{…}` and `pub mod view` in
[`lib.rs:25-32`](../crates/sparq-shacl/src/lib.rs)) — but it means the generator
inherits `sparq-forms`' metadata-resolution logic rather than being a thin fold
over `Component`. Any estimate built on "just map `Component` variants" is low.

**Also worth recording, to prevent a future misreading:** the only occurrences of
"graphql" in the Rust sources are the `graphql-ws` WebSocket *subprotocol* string
in two bearer-token tests in `crates/sparq-server/src/http.rs` (lines 9216, 9229).
There is **no** existing GraphQL surface, partial or otherwise.

## 2. What the substrate actually provides

Verified by reading the source, not the brief. Constraints relevant to schema
generation, against `Component` in
[`model.rs`](../crates/sparq-shacl/src/model.rs):

| SHACL term | In the parsed model? | Where |
|---|---|---|
| `sh:targetClass` | Yes | `Target::Class` (`model.rs:26`), via `Shape::targets` |
| `sh:path` | Yes | `Shape::path: Option<Path>`, full path expressions |
| `sh:datatype` | Yes | `Component::Datatype(Vec<String>)` (`model.rs:60`) |
| `sh:class` | Yes | `Component::Class` (`model.rs:48`), `ClassIn` |
| `sh:nodeKind` | Yes | `Component::NodeKind(Vec<String>)` (`model.rs:65`) |
| `sh:minCount` / `sh:maxCount` | Yes | `model.rs:66-67` |
| `sh:in` | Yes | `Component::In(Vec<Term>)` (`model.rs:153`) |
| `sh:node` | Yes | `Component::Node(usize)` (`model.rs:120`), index into `ShapesModel::shapes` |
| `sh:or` | Yes | `Component::Or(Vec<usize>)` (`model.rs:118`) |
| `sh:name`, `sh:description`, `sh:order`, `sh:group` | **No** | Read from the shapes `GraphView` (see §1) |

`Path` supports the full expression grammar (`Predicate`, `Inverse`, `Sequence`,
`Alternative`, `ZeroOrMore`, `OneOrMore`, `ZeroOrOne`) and — importantly for
translation — exposes `to_sparql_property_path() -> Option<String>`. The `Option`
is the tell: not every SHACL path round-trips to a SPARQL property path, so the
generator needs a declared, tested behaviour for the `None` case (skip the field,
or fail the schema build) rather than an `unwrap`.

`ShapesModel` and `Shape` carry **no serde derives**. An SDL emitter does not need
them, but an `explain` endpoint that returns a machine-readable translation trace
will need its own serializable projection — do not plan on serializing the shape
model directly.

## 3. The three hard problems

These, not the `Component` fold, are what decide the size of this work.

### 3.1 Naming: IRIs are not GraphQL names (no injective mapping)

GraphQL names must match `/[_A-Za-z][_0-9A-Za-z]*/` and are flat within a type.
IRIs are neither. Every real vocabulary breaks this immediately:
`foaf:name` and `schema:name` are distinct properties that both want the field
`name`; `dcterms:title` contains no illegal characters but `ex:has-title` does;
two shapes in different namespaces both targeting a class called `Person` want
the same type name.

There is no mapping that is simultaneously (a) total, (b) injective, (c) stable
across shape-file edits, and (d) pretty. Something must give, and *which* thing
gives is a user-visible API contract that is painful to change later. The
realistic options are a declared prefix map with deterministic collision
suffixing, or requiring `sh:name` and failing the build on collision — the latter
is stricter but produces a schema an app developer would actually want to use,
and it is the option that makes the feature's value proposition (app-developer
adoption) real rather than nominal.

**This is the single largest design decision in the bead, and it is not
technical-taste — it is a compatibility commitment.** It deserves the maintainer's
sign-off before any code.

### 3.2 SHACL shapes are constraints, not a schema

This is the deep mismatch, and it is easy to under-weight because the
*syntactic* mapping (node shape → type, property shape → field) looks so clean.

- **SHACL is open-world and non-exhaustive.** A shape says "if you are a
  `Person`, you must have a name". It does *not* say a `Person` has *only* the
  declared properties, nor that every declared property shape is the complete
  field set. GraphQL types are closed. Generating a closed type from an open
  constraint silently tells the client "these are the fields" when the data may
  hold arbitrarily many more. `sh:closed` is the only construct that licenses the
  closed reading, and most real shape files do not set it.
- **A node can conform to many shapes.** GraphQL objects have one concrete type.
  Mapping N conforming shapes onto one node needs interfaces/unions and a
  tie-break rule; `sparq-forms` sidesteps this by exposing a *shape switcher*
  (`shapes: Vec<ShapeChoice>`) and letting the UI pick — a luxury a schema
  generator does not have.
- **Recursion.** `Component::Node(usize)` and `property_children` are indices into
  a graph that is explicitly documented as cycle-safe — i.e. shape cycles are
  expected. GraphQL types recurse happily, so this is benign for *schema* emission,
  but it makes unbounded query depth a live denial-of-service surface for any
  *execution* layer (§3.3).
- **`sh:or` / `sh:xone` do not map cleanly.** A union of a scalar and an object
  type is not expressible in GraphQL (unions are over object types only).

None of this is fatal; all of it means the honest output is a *lossy, documented
projection* of the shapes, and the `explain` endpoint the brief calls for is not a
nicety but the thing that makes the lossiness auditable.

### 3.3 Execution: translation, N+1, and depth

Schema emission is a pure function of the shapes. *Serving* a query is a
different program: it must translate nested selection sets into SPARQL, and the
naive resolver-per-field shape is the classic N+1 — one query per parent per
field, which on a graph store is exactly the access pattern the engine is worst at
amortising. Doing it well means compiling a whole selection set into one SPARQL
query with the nesting expressed as optional/sub-select structure, plus
filter/order/pagination pushdown — and pagination over an unordered RDF result
set needs a declared stable sort or cursors are meaningless.

Plus the standard GraphQL hardening that is not optional on a public endpoint:
depth limiting, complexity budgeting, and (given §3.2's cycles) cycle-aware
bounds. **This is the XL half of the bead.** The `L` estimate in the issue is
defensible for §3.1 + schema emission; it is not defensible for this.

## 4. Options

| # | Option | Scope | Honest trade-off |
|---|---|---|---|
| A | **Do nothing** (status quo) | — | Zero cost. Leaves the competitor-parity gap open, but the repo's own vendor research rates demand as low vs SPARQL/BI and flags the fit as debatable. |
| B | **SDL emitter only** | `shapes → GraphQL SDL` (string out), plus a translation report | Bounded, testable as pure golden files, no server surface, no DoS surface, no execution semantics to get wrong. Delivers the "app developer sees a familiar schema" hook and forces §3.1 to be settled. Does **not** answer a query. |
| C | **Full read endpoint** | B + resolver + filter/order/paginate + `explain` | What the issue describes. Genuine parity. XL, not L (§3.3); adds a public execution surface to harden; commits sparq to GraphQL semantics long-term. |
| D | **Adopt an existing spec/impl** | Map to a published RDF-GraphQL convention rather than inventing one | Avoids inventing a naming contract; but adopting someone else's projection means inheriting their lossiness decisions, and I could not verify the current state of any such spec (see §7). |

Option B is a strict prefix of C, so choosing B does not foreclose C — provided
B's naming contract (§3.1) is designed as the permanent one.

## 5. Recommendation

**Keep `sq-lsp7k.13` deferred, and re-label the reason.** The defer was
predicated on "do after forms F1 lands so the shape-model API is proven". That
condition is now **met** — so leaving the bead in the same state without saying
why would be stale. The remaining objection is *not* substrate readiness; it is
that the repo's own two research records independently rate demand as low and fit
as `ambiguous-ask-user`. That is a maintainer call, and §7 asks it directly.

**If it proceeds:** take **Option B**, ship G1 alone, and require a demand signal
(a real user asking, not parity-table symmetry) before funding G2+. Rationale:
B is where the whole of the *irreversible* design work lives (the naming
contract), it is genuinely `L`, it is verifiable with golden-file tests and no
new attack surface, and it produces the artifact that would let a prospective
user tell us whether the projection is useful at all — which is precisely the
evidence currently missing.

Do **not** ship a half-execution layer. A GraphQL endpoint that answers simple
queries and N+1s on nested ones is worse than none: it sets a performance
expectation the implementation does not meet, on a project whose stated posture is
performance dominance.

## 6. Phased plan (proposed future beads)

Each phase is independently landable and independently abandonable. `bd` is not
available in this checkout, so these are **proposed** children of `sq-lsp7k.13`,
not created beads.

1. **G0 — settle the naming contract** (S, design, needs:maintainer).
   Decide §3.1 and write it down as a spec with a collision test matrix. Blocks
   everything else. No code. Output: an addendum to this record.
2. **G1 — `shapes → SDL` emitter** (L, new opt-in crate).
   Pure function `(ShapesModel, GraphView) → SDL + TranslationReport`. Covers
   types, fields, scalars from `sh:datatype`, enums from `sh:in`, nullability and
   lists from `sh:minCount`/`sh:maxCount`, `sh:node` for object fields,
   `sh:name`/`sh:description` → names and docstrings. Explicitly and *loudly*
   reports what it dropped (unmapped `sh:or`, non-round-trippable `Path`,
   open-world caveat per type). Golden-file tests; no server wiring.
   **This is the recommended stopping point.**
3. **G2 — `explain` surface** (M). Machine-readable per-field provenance:
   field → originating property shape → SPARQL property path. Needs its own
   serializable projection (the shape model has no serde). Valuable
   independently of execution, as it is what makes G1's lossiness auditable.
4. **G3 — single-query translation + execution** (XL). Selection set → one
   SPARQL query; filter/order/pagination pushdown with a declared stable sort.
   Gate on a benchmark showing nested selections do **not** degrade to N+1.
5. **G4 — endpoint hardening + server wiring** (M, strictly after G3). Depth and
   complexity limits, cycle-aware bounds, opt-in feature flag off by default.

Mutations stay out of scope throughout, per the issue.

## 7. Open questions for the maintainer

1. **Does this ship at all?** Both prior records rate demand low and fit
   debatable; `feature-research-broad-sparql-vendors.md:344` literally routes it
   to `ambiguous-ask-user`. This record does not resolve that — it only removes
   the substrate objection. A yes/no here is worth more than any further design.
2. **§3.1 naming contract** — strict (`sh:name` required, fail on collision) or
   lenient (derive from IRI with deterministic suffixing)? Strict produces a
   better schema and a worse out-of-the-box experience.
3. **Open-world honesty** — should a type generated from a non-`sh:closed` shape
   be marked as non-exhaustive in its docstring (my recommendation), or is the
   closed reading acceptable silently, as is conventional in this space?
4. **Stopping point** — is G1-alone acceptable as a shipped deliverable, or is a
   schema-without-an-endpoint considered not worth having?

## 8. Uncertainties and limits of this record

- **Competitor mechanics are unverified.** The issue cites GraphDB (GraphQL from
  OWL/SHACL), Stardog, and TopBraid EDG as parity drivers. Web access was not
  available in this session, so I could **not** verify how any of them resolve
  §3.1 or §3.2. The competitor claims here are carried from the repo's own
  records ([`feature-research-broad-sparql-vendors.md`](./feature-research-broad-sparql-vendors.md),
  lines 178-195, 293, 344), *not* independently confirmed against vendor docs.
  Anyone doing G0 should verify them first — the naming decision is exactly where
  prior art is most valuable, and Option D depends entirely on it.
- **No performance numbers** appear in this record, by design; the N+1 claim in
  §3.3 is an architectural argument about access patterns, not a measurement, and
  G3's gate is written to require a real one.
- **Effort labels** (`S`/`M`/`L`/`XL`) are my estimates from the substrate audit,
  not calibrated against this repo's historical bead throughput.
- The `sparq-forms` line/test counts in §1 are as reported by a sub-agent survey;
  the API facts (§1-§2) — public types, absent metadata, `GraphView` fallback
  paths, file:line references — I verified directly against the source.
