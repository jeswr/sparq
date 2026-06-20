# SPARQL-callable graph-analytics surface for `sparq-algos` (opt-in) + the `af`-vs-own ontology question

<!-- [OPUS-4.8] sq-b2uq (gh-914): DESIGN-FOR-REVIEW. No production code. Authored by the SPARQ agent.
     Reviewed-by jeswr is a HARD precondition before any implementation bead is opened. -->

> Status: **design for review — NOT approved, NO impl bead is ready.** This record exists so
> @jeswr can rule on the surface shape and the ontology question (his explicit ask) *before*
> any code is written or any crate/registry is reserved. Every implementation bead listed in
> §8 is **blocked-on-design-review**.

Bead: **sq-b2uq** (GitHub #914) · Area: `sparq-algos`, `sparq-engine` · Type: design / enhancement

## 0. jeswr's verdict (the thing this design must satisfy)

His real-voice greenlight on #914 (his words, not an agent's):

> "Fine with you looking into this if there are no performance degradations. It should also be
> opt in (as a general rule, it should be possible to build the engine as pure SPARQL without
> any of the extensions). Can you please do a design and explain why/why not use existing
> ontologies like `af` vs. building your own."

So the deliverable is a design plus a reasoned ontology recommendation, under three hard
constraints carried verbatim into §7: **opt-in / pure-SPARQL-buildable**, **no perf
degradation** (measured on a canonical host, which this work box is **not**), and
**review-before-impl**.

## 1. What is actually in the tree today (verified, not assumed)

I read the code rather than trusting the bead framing. The facts that shape the design:

- **`sparq-algos` is Rust-API-only and engine-free.** `crates/sparq-algos/Cargo.toml`
  depends on `sparq-core` + `oxrdf` + `rustc-hash` and **nothing else** — notably **not**
  `sparq-engine`. Nothing in the workspace depends on `sparq-algos`; the default engine build
  does not even compile it. It projects the RDF graph onto a directed adjacency view
  (`NodeGraph`, `graph.rs`) keyed by dictionary ids, and runs `pagerank` (`pagerank.rs`),
  degree/in/out centrality (`centrality.rs`), and weakly-connected-components +
  label-propagation community detection (`community.rs`). All of it is `#![forbid(unsafe_code)]`
  and holds no graph state of its own. There is **today no SPARQL surface at all** — these are
  plain Rust functions over `&NodeGraph`.
- **The engine has THREE distinct extension seams, not one.** The bead names only the
  `FunctionRegistry`; faithfully, there are three, and the surface choice in §3 turns on
  picking the *right existing one per algorithm shape* — none of them is new:
  1. **`FunctionRegistry` (expression seam)** — `crates/sparq-engine/src/lib.rs:141`. Maps a
     function IRI to an `ExtFn = Arc<dyn Fn(&[Term]) -> Result<Term, String>>`: **N concrete
     terms in, exactly ONE concrete term out**, evaluated **per row** inside `FILTER` / `BIND` /
     `SELECT`-expression. Installed per query via `with_functions` / `query_with_functions`.
     This is what `sparq-geo`'s `geof:` functions and SHACL-AF's `sh:SPARQLFunction` custom
     functions ride (`crates/sparq-geo/src/registry.rs`, `crates/sparq-shacl/src/func.rs`).
  2. **The magic-predicate rewrite seam** — `PreparedQuery: From<spargebra::Query>`
     (`crates/sparq-engine/src/lib.rs:460-496`). An extension crate walks the parsed spargebra
     algebra, replaces its own magic triple patterns with ordinary algebra, and hands the
     rewritten `PreparedQuery` to a standard `*_prepared` engine entry point. **This is how
     `sparq-text`'s `text:` predicates and `sparq-vectors`'s `vec:` predicates already work**
     (`crates/sparq-text/src/rewrite.rs`, `crates/sparq-vectors/src/rewrite.rs`). The engine —
     planner, executor, wasm bundle — stays completely unaware. This seam **can bind new
     variables** and produce **set-valued / relational** output; the expression seam cannot.
  3. **The thread-local planner-provider seam** — `with_spatial_index` / `SpatialProvider` /
     `SpatialQuery` (`crates/sparq-engine/src/lib.rs:233+`, used by
     `crates/sparq-geo/src/provider.rs`). The engine recognises a pushable pattern, asks an
     installed provider for a candidate id-set, and pre-restricts rows before the exact
     refinement. This is the "compute-an-index-backed-answer-once, the engine does no
     geometry/analytics itself" precedent.
- **sparq already mints its own extension IRIs and there is a precedent namespace.**
  `sparq-text` uses `http://sparq.dev/text#…` (`text:matches`, `text:score`, …) and
  `sparq-vectors` uses `http://sparq.dev/vec#…` (`vec:nearest`, `vec:search`). These are
  *sparq-invented, non-standard* IRIs, already shipped and documented as such.

### 1.1 Correction to the bead's premise (honest scoping)

Two premises in the brief need correcting before the design is sound:

- **"built on the EXISTING `FunctionRegistry` … NOT a new mechanism."** The intent — *reuse an
  existing seam, do not invent one* — is right and is honoured. But the `FunctionRegistry` **on
  its own cannot express the headline surfaces the bead itself lists**: `?a sparq:shortestPath
  ?b` (binds/enumerates path nodes — set-valued, variable-binding) and `?x sparq:pagerank
  ?score` over *all* nodes (one score per node, a relation, not a per-row scalar of an
  already-bound argument). Those are **magic predicates**, and the matching existing seam is
  #2 (the `text:`/`vec:` rewrite), **not** the expression registry. The `FunctionRegistry` is
  the right seam only for the *scalar-of-a-bound-node* shape (e.g. `BIND(sparq:degree(?n) AS
  ?d)`). So the design reuses **existing** seams throughout — it just maps each algorithm to
  the *correct* existing seam. No new engine mechanism is introduced.
- **"use existing ontologies like `af`."** There is no single ontology called `af`. Jena's
  function/property vocabularies are `afn:` (`http://jena.apache.org/ARQ/function#`, expression
  functions) and `apf:` (`http://jena.apache.org/ARQ/property#`, property functions), plus
  `list:` (`http://jena.apache.org/ARQ/list#`). **I checked Jena's published property-function
  and function libraries: they define list/sequence/container/string/text-match helpers and
  *zero* graph-analytics IRIs** — no `pagerank`, no `shortestPath`, no centrality, no community
  detection (sources in §11). So "adopt `af`" cannot mean "reuse standard analytics IRIs",
  because none exist there to reuse. This reframes the ontology question — see §4.

## 2. Problem framing

Vendor parity gap (epic sq-3183): Neptune Analytics ships 25+ graph algorithms; sparq's
`sparq-algos` implements four, **reachable only from Rust**. A SPARQL user cannot today write
"rank these nodes" or "find the shortest path". We want to expose the algorithms *through
SPARQL*, while keeping the iron constraint that the engine still builds and runs as **pure
SPARQL with every extension off**. The design space is (a) which SPARQL surface shape, (b)
which IRIs/vocabulary, (c) which algorithms, (d) how a model-backed UDF reuses the same wiring.

## 3. SPARQL surface choice — property/magic-predicates as the spine, expression-functions for scalars

### 3.1 The three candidate surface shapes

| Shape | Looks like | Existing seam | Can bind new vars? | Set/relation-valued? |
|---|---|---|---|---|
| **A. Expression function** | `BIND(sparq:degree(?n) AS ?d)` · `FILTER(sparq:pagerank(?n) > 0.01)` | #1 `FunctionRegistry` | No | No — one term out |
| **B. Magic / property predicate** | `?n sparq:pagerank ?score` · `?a sparq:shortestPath ?b` | #2 rewrite (`text:`/`vec:`) | **Yes** | **Yes** |
| **C. Procedure / `CALL` table** | `CALL sparq:louvain() YIELD node, community` | none — **new front-end** | Yes | Yes |

### 3.2 Why magic predicates (B) as the primary surface, with A for scalars

**Recommendation: surface B (magic property predicates) as the spine, surface A (expression
functions) only for the genuinely scalar-of-an-already-bound-node cases. Reject surface C.**

Reasons, ordered by weight:

1. **B and A both reuse *existing, shipped* seams; C does not.** `CALL … YIELD` is not SPARQL
   1.1 grammar. spargebra (the vendored, conformance-tracking parser) does not parse it, and the
   AGENTS rule is explicit that the conformance parser must not be forked for extensions (the
   `OVER(…)` window syntax went through a *separate* `query_over` entry point precisely to avoid
   touching it). A `CALL` table form would mean a bespoke parser front-end — exactly the "new
   mechanism" the bead says to avoid — for no expressive gain over B. **Reject C.**
2. **The set-valued / variable-binding algorithms *require* B.** `shortestPath` enumerates path
   nodes; whole-graph `pagerank`/centrality/community produce one value *per node* (a relation
   `?node ?value`). The `FunctionRegistry` returns exactly one `Term` and binds no variables
   (`ExtFn = Fn(&[Term]) -> Result<Term, String>`), so it physically cannot express these. The
   `text:`/`vec:` predicates already solve this exact "bind new vars, set-valued, compute-once"
   problem via the rewrite seam — graph analytics is the same shape, so it should ride the same
   rail.
3. **B already has a working, reviewed template in-tree.** `vec:`'s rewrite parses a magic BGP,
   computes a result against an index/store once at rewrite time, and splices ordinary algebra
   (`VALUES` / joins) back in, then runs `query_prepared`. Mirroring it for `algo:` is a
   low-novelty, low-risk port, not a research effort.
4. **A is still the right tool for the scalar shape.** `BIND(sparq:degree(?n) AS ?d)` and
   `FILTER(sparq:closeness(?n) > x)` — where `?n` is *already bound* and we want one scalar — fit
   the `FunctionRegistry` exactly, the way `geof:distance` does. Offering A for these keeps the
   common "annotate a row" case idiomatic without forcing a magic-predicate join.

**Honest cost of B (must be designed around):** the headline algorithms are **whole-graph,
not per-row**. Computing pagerank/Louvain once per query and caching it for the duration is
acceptable; recomputing per candidate binding is not. The rewrite must therefore (i) build the
`NodeGraph` projection + run the algorithm **once** at rewrite/prepare time (as `vec:` freezes
its hits at rewrite time — already documented behaviour, including the "re-prepare after the
graph changes" caveat), and (ii) materialise the result as a `VALUES` block or a provider-fed
candidate relation. This is the seam-3 "compute-once provider" pattern applied to analytics.
A naive expression-function pagerank that recomputes the whole graph per row would be a
catastrophic, and easily-made, performance mistake — calling it out here is the point.

### 3.3 Surface ↔ algorithm mapping (proposed)

| Algorithm | Surface | Sketch |
|---|---|---|
| degree / in / out (have it) | **A** scalar | `BIND(sparq:degree(?n) AS ?d)` |
| PageRank (have it) | **B** relation | `?n sparq:pagerank ?score` (one row per node) |
| weighted PageRank (new) | **B** + config | `?n sparq:pagerank (?score) [ sparq:weight ?w ]` *(config-term shape, see §5)* |
| shortest path (new) | **B** path | `(?a ?b) sparq:shortestPath (?len ?via)` |
| closeness / betweenness (new) | **A** scalar **and** **B** relation | scalar for one bound node; relation for all nodes |
| WCC (have it) | **B** relation | `?n sparq:component ?cid` |
| label propagation (have it) | **B** relation | `?n sparq:labelPropagation ?cid` |
| Louvain / Leiden (new) | **B** relation | `?n sparq:louvain ?cid` |

(All IRIs above are **placeholders pending §4's namespace decision** and are **sparq-invented,
non-standard** — flagged as such wherever they appear.)

## 4. The ontology decision (jeswr's explicit ask): why **not** adopt `af`/`apf`, why mint a sparq namespace anchored to W3C-standard primitives

### 4.1 What "adopt `af`" could actually mean

Because Jena's `afn:`/`apf:`/`list:` vocabularies contain **no analytics IRIs** (§1.1, §11),
"adopt `af`" can only mean one of:

- **(i) Adopt the *mechanism convention*** — i.e. model analytics as ARQ-style *property
  functions* (the `apf:`/`list:` magic-predicate idiom). We effectively *do* this already: our
  surface B **is** the property-function idiom, and it is also exactly what `text:`/`vec:`
  do. So the useful, portable part of Jena's design is the *shape*, and we are keeping it.
- **(ii) Mint our analytics IRIs *inside Jena's namespaces*** — e.g. `apf:pagerank` under
  `http://jena.apache.org/ARQ/property#`. This is the only sense in which we would "reuse the
  `af` ontology's IRIs", and it is the option to reject (§4.2).
- **(iii) Adopt some *other* third-party analytics vocabulary.** None is a cross-vendor
  standard: Neptune Analytics exposes its 25+ algorithms as **openCypher `CALL` procedures**,
  not SPARQL IRIs; GraphDB uses SPARQL 1.1 property paths plus a proprietary path-search
  extension; there is **no W3C or community-standard SPARQL graph-analytics vocabulary** to
  adopt (§11). So there is nothing to converge *on*.

### 4.2 Recommendation: **mint sparq's own namespace** (`http://sparq.dev/algo#`), do **not** squat on `apf:`, and **anchor names to standard primitives where they exist**

**Decision: do not put analytics IRIs in Jena's `apf:`/`afn:` namespaces. Mint a sparq
extension namespace — `http://sparq.dev/algo#` — consistent with the already-shipped
`http://sparq.dev/text#` and `http://sparq.dev/vec#`, and label every IRI as
sparq-invented / non-standard in the vocab module and SKILL docs.** Where a primitive genuinely
*is* standardised (RDF list membership, container helpers), continue to honour the standard IRI
rather than re-minting it.

Reasoning (the part jeswr asked to be reasoned out):

1. **Minting an `apf:pagerank` would be namespace-squatting on Apache's IRI space.** Owning the
   IRI `http://jena.apache.org/ARQ/property#pagerank` is Apache's prerogative, not ours; if Jena
   ever defines that IRI with *different* semantics our graphs/queries silently diverge from
   theirs. Coining IRIs under a namespace we do not control is an honesty and
   interoperability hazard. **This alone rules out (ii).**
2. **There is no standard to converge on, so "reuse" buys no interop (§4.1-iii).** The benefit of
   adopting an existing vocabulary is portability — a query that runs on engine X runs on engine
   Y. Since no engine exposes these algorithms under SPARQL IRIs at all (they use openCypher
   `CALL` or property paths), adopting `apf:` names gains **zero** real interoperability while
   incurring the squatting cost of point 1. The trade-off is strictly negative.
3. **Consistency with sparq's own, already-shipped convention.** `text:`/`vec:` already
   established `http://sparq.dev/<surface>#` for sparq extensions, each documented as
   non-standard. A new `algo:` namespace is the *house style*; a one-off Jena-namespaced
   analytics surface would be the inconsistent special case.
4. **Clear honesty boundary.** A distinct `sparq.dev/algo#` namespace makes it unambiguous to a
   reader (and to the privacy/honesty posture this repo enforces) that these are sparq
   extensions with sparq-defined semantics, not W3C-blessed or Jena-blessed behaviour. We will
   state that explicitly in the vocab module doc-comment and the SKILL surface, exactly as
   `vec:`/`text:` do ("the sparq extension namespace").
5. **Where a standard primitive exists, use it.** RDF-list/container access already has W3C and
   Jena `list:`/`rdfs:member` semantics; if an analytics surface ever needs "members of an RDF
   list", reuse `list:`/`rdfs:member` rather than re-minting — i.e. mint-our-own is the rule for
   *novel analytics* IRIs, **not** a blanket "ignore standards" stance. This keeps the
   recommendation defensible: we coin only what genuinely has no standard.

**Net recommendation:** keep the Jena property-function *idiom* (we already use it via the
rewrite seam), but put the analytics IRIs in **`http://sparq.dev/algo#`**, every one labelled
non-standard. Reuse standard IRIs only where the primitive is genuinely standardised.

> Open question for jeswr (genuine): is `http://sparq.dev/algo#` the namespace you want, or
> would you prefer to fold these under the existing `vec:`/`text:` style as a third sibling
> (e.g. `graph:` → `http://sparq.dev/graph#`)? And do you want the scalar (surface-A) forms to
> share that same namespace or sit under a distinct `afn:`-style `func` namespace? This is a
> naming call I should not make for you.

## 5. Algorithm scope to add to `sparq-algos`

Each new algorithm stays a **plain Rust function over `NodeGraph`** in `sparq-algos` (no engine
dep there — see §6), surfaced separately by the binding crate. Scope, with honest difficulty:

1. **Weighted edges.** `NodeGraph` today collapses parallel predicates to one unweighted edge
   (`parallel_edges_collapse` test). Add an *optional* per-edge weight (derived from a
   chosen numeric predicate, or a multiplicity count) as a parallel `Vec<f32>`, leaving the
   unweighted path byte-identical when no weighting is requested. Feeds weighted PageRank +
   weighted shortest path. *Difficulty: low-medium; the index already exists, this is an
   additive field.*
2. **Shortest path.** Single-source BFS (unweighted) and Dijkstra (weighted, non-negative).
   Surface as B `(?a ?b) sparq:shortestPath (?len ?via)`. K-shortest / all-pairs are explicitly
   **out of scope** for v1 (cost). *Difficulty: medium.*
3. **Closeness centrality.** Per-node mean inverse distance; reuses the BFS/Dijkstra core from
   (2). Both surface A (one bound node) and surface B (all nodes). *Difficulty: medium; honest
   note — exact closeness is O(V·(V+E)), so it needs a budget guard and a documented size
   ceiling.*
4. **Betweenness centrality.** Brandes' algorithm. **Most expensive** — O(V·E) unweighted, worse
   weighted. Ship with an explicit working-set/`QueryBudget` guard and a documented "not for
   large graphs without sampling" caveat; approximate/sampled betweenness is a **follow-up**, not
   v1. *Difficulty: high.*
5. **Louvain / Leiden community detection.** Louvain (modularity-greedy) first; Leiden (the
   refinement that fixes Louvain's disconnected-community defect) as a **follow-up**, since Leiden
   is materially more complex and the existing label-propagation already covers the cheap case.
   Surface B relation `?n sparq:louvain ?cid`. *Difficulty: high (Louvain), higher (Leiden).*

Determinism caveat (load-bearing, learned from the existing tests): community/label algorithms
in this crate are **only** reproducible because `NodeGraph` assigns indices in canonical
ascending-term order, independent of the thread-count-dependent dictionary-id order (see the
`node_indices_are_canonical_term_order` and `label_propagation_two_communities_exact_membership`
tests). Louvain/Leiden tie-breaking and any RNG seed **must** key off that canonical order, or
results flip between hosts — the same bug that bit label-propagation in CI. Any new algorithm
gets cross-host-deterministic tests as a hard acceptance criterion.

## 6. Architecture — where the binding lives (keeping core lean + opt-in)

`sparq-algos` must **not** gain a `sparq-engine` dependency — that would entangle the analytics
math with the query engine and is unnecessary. Mirror precisely how `vec:` is structured:
`sparq-vectors`' *math* (the index) carries no engine dep, and only the **`vec-predicate`
feature** pulls `sparq-engine` + `spargebra` in to add the rewrite. So:

- **`sparq-algos`** keeps its pure-math functions (existing + the §5 additions), still
  `sparq-core`-only, still `forbid(unsafe_code)`, default features `[]`.
- **A new opt-in feature** (proposal: `algo-predicate` on `sparq-algos`, gated like
  `vec-predicate`) — **or** a thin sibling crate `sparq-algos-sparql` — adds the rewrite that
  recognises the `algo:` magic predicates and the `FunctionRegistry` scalar functions, pulling
  `sparq-engine`/`spargebra` **only under that feature**. *Decision deferred to review: a gated
  feature keeps the surface in one crate (like `vec:`); a sibling crate keeps `sparq-algos`'
  dependency graph pristine. I lean toward the gated feature for consistency with `vec:`, but
  flag it as a reviewer call.*
- **The whole-graph compute-once discipline** rides seam #3's pattern: build `NodeGraph` + run
  the algorithm once per prepare, materialise as `VALUES`/provider relation, never per-row.

## 7. CONSTRAINTS (carried verbatim from the bead — the implementer must not lose these)

1. **OPT-IN / pure-SPARQL-buildable.** The engine must build and run as **pure SPARQL with the
   extension OFF**. The analytics SPARQL surface ships as an **opt-in cargo feature (or opt-in
   crate), default-OFF**. The default `sparq-engine`/`sparq-core` build, the wasm bundle, and the
   `sparq-algos` default build must carry **zero** analytics-SPARQL code and gain **no** new
   required dependency — exactly the bar `vec-predicate`/`text:` meet. **Gated, tested, and
   clippy-clean in BOTH feature states** (off and on) — the standard sparq opt-in bar.
2. **NO perf degradation — and the canonical-host caveat.** Adding the surface must not regress
   the core query path or the default build. Two non-negotiables: (a) when the feature is OFF the
   default build is byte-identical / dependency-unchanged (the structural guarantee); (b) any
   performance claim ("no regression", "fast enough") **must be measured on a canonical host
   before this is armed**. **This work box is an AWS EC2 instance and is NON-CANONICAL** — its
   timings cannot be baked into docs/tests or used to declare no-regression. This design
   deliberately states **no** performance numbers; producing canonical numbers is itself a gated
   step in §8, and **cannot be done in-session**.
3. **Review-before-impl.** This design and the ontology justification **must be reviewed by
   jeswr before any implementation bead is opened.** No open PR reserves `sparq-algos` or the
   engine registry. Every §8 bead is **blocked-on-design-review**.

## 8. Proposed implementation beads (decomposed) — ALL blocked-on-design-review

Ordered; each becomes a future bead **only after** jeswr signs off on §3-4 (surface + ontology).
None is ready-to-go; the orchestrator must **not** create these as `open`/launchable.

1. **Ratify surface + namespace.** Land jeswr's decisions from §3 (B-spine + A-scalar; reject C)
   and §4 (`http://sparq.dev/algo#` vs alternative; scalar-namespace question) into this doc.
   *Gate for everything below.* (depends on: review)
2. **Vocab + feature scaffold.** Add the `algo:` vocab module (IRIs labelled non-standard) and
   the **default-OFF** opt-in feature/crate skeleton; prove the default build is unchanged and
   the feature compiles + clippy-clean in BOTH states. No algorithms wired yet. (depends on: 1)
3. **Weighted-edge `NodeGraph`.** Additive optional weights in `sparq-algos`; unweighted path
   provably unchanged; cross-host-deterministic tests. (depends on: 1)
4. **Surface-A scalar functions via `FunctionRegistry`.** `sparq:degree`/`closeness` scalar form
   for an already-bound node; mirrors `geof:` registration. (depends on: 2, 3)
5. **Surface-B rewrite for the existing relations.** `pagerank`/`component`/`labelPropagation`
   as magic predicates via the `vec:`-style rewrite, compute-once-per-prepare. (depends on: 2)
6. **Shortest path** (BFS + Dijkstra) in `sparq-algos` + surface B. (depends on: 3, 5)
7. **Closeness + betweenness** (Brandes) in `sparq-algos` + surfaces A/B, with budget guards and
   documented size ceilings. (depends on: 6)
8. **Louvain** community detection + surface B, with canonical-order-deterministic tie-breaks.
   (depends on: 5)
9. **Leiden** refinement (follow-up; only if Louvain's defects matter in practice). (depends on: 8)
10. **Model-backed-UDF recipe** (§9) — wire the existing `FunctionRegistry` to
    `sparq-vectors`' embedding API; **doc/recipe + example, no new trust/crypto surface.**
    (depends on: 2)
11. **Canonical-host no-regression measurement.** Run the default-build benchmark catalog on a
    **canonical** host (NOT this work box) and confirm no regression vs `bench/perf-baseline.json`
    **before** any of the above is armed for merge. *This is the perf-gate the bead demands and
    it cannot be satisfied in-session.* (depends on: 2-10 as they land)
12. **SKILL/README maintenance.** Document the surface, every non-standard IRI, the opt-in
    feature, and the perf/size caveats on the matching `skills/<surface>/SKILL.md` + crate README
    (the maintenance rule). (depends on: 4-10)

## 9. The blessed model-backed-UDF recipe (existing registry → `sparq-vectors`, no new trust surface)

The bead asks for the "blessed model-backed-UDF recipe wiring the existing registry to
`sparq-vectors`." Stated honestly and minimally — **this is a documented recipe, not a new
capability or a new trust/crypto surface**:

- `sparq-vectors` already exposes an embedding seam: an OpenAI-compatible `/v1/embeddings`
  provider behind the opt-in `provider`/`embeddings` features, where **the HTTP transport is
  supplied by the caller** (the crate never opens a socket by default; `embeddings` adds a
  concrete reqwest transport, off by default).
- The recipe: a caller builds a `FunctionRegistry` (seam #1) and `register`s an IRI — e.g.
  `sparq:embed` (sparq-invented, non-standard) — whose `ExtFn` takes a literal/IRI term, calls
  the **caller-provided** `sparq-vectors` embedding provider, and returns the embedding (e.g. a
  packed literal). Then `query_with_functions` runs SPARQL that calls `sparq:embed(?text)` in a
  `BIND`. This is the *exact* `geof_registry` pattern (build a registry, hand it to
  `query_with_functions`), reused unchanged.
- **No new trust/crypto surface, stated plainly.** This is application glue over two existing
  opt-in seams. It does **not** touch the ZK/MPC estate. There is **no** privacy or soundness
  guarantee implied or claimed here — it is an ordinary network-calling extension function, and
  it inherits the same SSRF/egress posture as any outbound call (the operator controls the
  transport and endpoint). The recipe ships as **documentation + an example**, gated behind the
  caller opting into `sparq-vectors`' `provider`/`embeddings` features; the default build calls
  no model and opens no socket.

## 10. Open questions for jeswr (genuine — block §8)

1. **Namespace** (§4.2): `http://sparq.dev/algo#`, or a `graph:`/`http://sparq.dev/graph#`
   sibling, or split scalar vs relation namespaces? Your call, not mine.
2. **Feature vs sibling crate** (§6): gated `algo-predicate` feature on `sparq-algos` (like
   `vec-predicate`), or a separate `sparq-algos-sparql` crate? I lean to the gated feature for
   `vec:`-consistency; confirm.
3. **v1 algorithm cut**: is the §5 v1 set (weighted edges, shortest path, closeness, betweenness,
   Louvain) the right scope, or should expensive ones (betweenness, Leiden) be deferred to a
   second wave behind a sampling/approximation design?
4. **Canonical perf host**: which host counts as canonical for the §8.11 no-regression gate, so
   the implementer doesn't (wrongly) cite this EC2 work box?

## 11. Sources

- Apache Jena — Property Functions in ARQ (`apf:` = `http://jena.apache.org/ARQ/property#`,
  `list:` = `http://jena.apache.org/ARQ/list#`; list/container/string/text-match helpers only,
  no analytics): <https://jena.apache.org/documentation/query/library-propfunc.html>
- Apache Jena — Functions in ARQ (`afn:` = `http://jena.apache.org/ARQ/function#`; expression
  functions, no analytics): <https://jena.apache.org/documentation/query/library-function.html>
- Apache Jena — Writing Property Functions / Extensions:
  <https://jena.apache.org/documentation/query/writing_propfuncs.html>,
  <https://jena.apache.org/documentation/query/extension.html>
- AWS Neptune Analytics — algorithms exposed as **openCypher procedures** (25+; PageRank,
  closeness, WCC/SCC, label propagation, Louvain; shortest-path variants):
  <https://docs.aws.amazon.com/neptune-analytics/latest/userguide/algorithms.html>
- GraphDB — graph path search (SPARQL 1.1 property paths + proprietary path extension; not a
  shared analytics vocabulary):
  <https://graphdb.ontotext.com/documentation/9.9/enterprise/graph-path-search.html>
- In-tree precedent (verified): `crates/sparq-engine/src/lib.rs` (`FunctionRegistry`:141,
  `PreparedQuery`:460, `SpatialProvider`:233+), `crates/sparq-vectors/src/rewrite.rs` +
  `Cargo.toml` (`vec:` rewrite, `vec-predicate` feature, `http://sparq.dev/vec#`),
  `crates/sparq-text/src/lib.rs` + `rewrite.rs` (`text:`, `http://sparq.dev/text#`),
  `crates/sparq-geo/src/registry.rs` + `provider.rs` (`geof:` registry + spatial provider),
  `crates/sparq-shacl/src/func.rs` (SHACL-AF custom functions), `crates/sparq-algos/` (the
  Rust-only algorithms + canonical-order determinism tests).
