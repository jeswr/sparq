# sparq-fedclient

A **streaming federation CLIENT** over heterogeneous remote RDF sources — the query
*consumer* half of federation, **opt-in** and OFF by default.

Given one SPARQL query and a set of heterogeneous remote sources (full SPARQL endpoints,
bindings-restricted brTPF servers, plain TPF servers, and the *local* sparq engine), this
crate — when complete — **discovers** each source's capability, **plans** a federated
execution that pushes the most precise sub-query each source can answer (reusing the
`sparq-fedplan` cost-based planner), and **streams** results back through non-blocking
federation operators. See `research/federation-client-design.md` for the full design (§4
architecture, §6 phased build plan, §7 honest risks).

> Model: Opus 4.8 (Fable unavailable — flag for re-review when Fable returns).
> Epic **sq-dnko** / **sq-3183** (streaming federation client). Beads: skeleton
> **sq-s1uy** · discovery **sq-nfxl** · source abstraction **sq-rsxf** · planner bridge +
> single-source interpreter **sq-j27p** · streaming operators **sq-vtba** · brTPF/TPF
> **sq-2qze**.

## What has landed, and what is still ahead

The crate began as the **Phase-0 skeleton** (design §6) — the public module layout, the
opt-in feature, and the dependency-boundary proof, before any federation logic. Landed
since (each behind the same default-OFF `fedclient` feature):

- **Phase 1 — capability discovery** (`sq-nfxl`): the `discovery` module GETs a source's
  SPARQL Service Description + `/.well-known/void`, parses them to a `Capability` +
  `SourceDescriptor`, with a FedX-style ASK-probe fallback, all behind an SSRF-guarded
  fetch seam.
- **Phase 2 — source-type abstraction + Endpoint adapter** (`sq-rsxf`): the `source` module
  — `SourceType` (`Endpoint | BrTpf | Tpf | Local`), the `FederatedSource` trait, the
  fine-grained `Capability` descriptor, and the real `Endpoint` SRJ adapter over a
  `Transport` seam behind a **default-deny SSRF egress guard**.
- **Phase 3 — planner bridge + materialised single-source interpreter** (`sq-j27p`): the
  `planner` module's `SourceResolver` (index → adapter resolution) + `lower_leaf` per-leaf
  lowering, and the `operators` module's `materialize_single_source` interpreter +
  `parse_srj` + `solutions_equal` result-equivalence (see below).
- **Phase 4 — capability-aware pushdown** (`sq-7byx`): the `pushdown` module — FedX
  exclusive-group decomposition, the maximal pushable sub-algebra per group, the exact
  common-variable FILTER check, and the cross-group bind-join block primitive (see below).
- **Phase 5 — streaming operators** (`sq-vtba`): the `stream` module's `SolutionStream`
  boundary + the `operators` module's bounded blocking thread-pool (`ScatterPool`),
  `StreamJoin`-feeder streaming join (`StreamingJoin`), and the streaming interpreter
  (`stream_single_source`) that EMIT results before the inputs are exhausted (see below).
- **Phase 6 — brTPF + TPF fragment adapters** (`sq-2qze`): the real Triple-Pattern-Fragments
  adapters (see below).
- **Phase 7 — adaptive re-planning** (`sq-ij5x`, the FINAL phase): the opt-in ANAPSID feedback
  loop in the `adaptive` module, behind the extra default-OFF `fedclient-adaptive` feature (see
  below).

With Phase 7 the **8-phase streaming federation client is feature-complete** — Phases 0–7 have
all landed and epic **sq-dnko** is closed. Still ahead as future beads under epic sq-3183:
multi-source UNION-per-leaf fan-out, the streaming bind-join pushed down as VALUES/`maxMpR`
blocks, and the ANAPSID "adaptive operator" refinement (§7).

The Phase-3 planner bridge + interpreter is detailed under
[Phase 3 — lower a plan + interpret it against one source](#phase-3--lower-a-plan--interpret-it-against-one-source);
the Phase-5 streaming operators follow it, then the Phase-6 fragment adapters.

## brTPF + TPF fragment adapters (Phase 6, `source` module)

A single triple pattern is the access unit of a Triple Pattern Fragments server. Both
adapters wrap a `FragmentTransport` seam (fetch one fragment page for a pattern, optionally
with an attached binding block → matched triples + the page's `hydra:totalItems` count + an
optional `hydra:next` token) and return a **complete** answer for one pattern as typed
`FragBinding`s:

- **`TpfSource` (plain TPF)** — fetches the fragment to exhaustion (follows `hydra:next`),
  binds every matched triple into the pattern's variables, and returns the whole (selective)
  fragment. There is no bind-join: a plain-TPF source shifts every join client-side, so the
  planner hash-joins the materialised fragments locally, driven by the count metadata.
- **`BrTpfSource` (bindings-restricted TPF)** — additionally pushes a block of *at most
  `maxMpR`* upstream bindings with each request (the standardised brTPF bind-join). It
  chunks the upstream bindings into `maxMpR`-sized blocks, issues one paginated request per
  block, and concatenates the per-block matches — complete by construction, and the block
  size never exceeds `maxMpR`.
- **Count-metadata cardinality** — both expose `cardinality(pattern)` and a one-pattern
  `SourceDescriptor` from `discover()` seeded with the fragment's `hydra:totalItems`, so the
  `sparq-fedplan` CostFed estimate keys on the *served* count. For brTPF the descriptor uses
  the unbound-pattern count (a recall-safe upper bound the bound block only narrows).

A fragment server speaks triples, not SPARQL-Results-JSON, so the adapters answer through
the typed `solutions(...)` methods; their `FederatedSource::execute` (the SRJ entry point)
is a deliberate `Unsupported` that points the caller at `solutions` — no lossy SRJ
re-serialisation, no overclaim. The adapters are tested against an in-memory fixture
fragment server (real fetch → parse → bind → paginate → bind-join, zero network); the
native HTTP `FragmentTransport` (ureq + the default-deny SSRF resolver, Hydra URI-template
serialisation, Turtle/TriG fragment parsing) lands with the streaming phase.

## Capability-aware pushdown (Phase 4, `pushdown` module)

The planner decides *which* source answers *which* pattern and in *what* join order; the
`pushdown` module decides the **most precise sub-query** each source is asked, so a source
answers a whole sub-pattern in one request rather than one single-pattern `SELECT` per leaf.

- **`exclusive_groups(selection, bgp)`** — the FedX **exclusive-group decomposition**. From
  the planner's per-pattern source selection it forms the maximal sets of patterns that each
  have **exactly one** retained source, the **same** source, and are **connected** by a
  shared variable (transitively, via union-find). These are the sub-BGPs pushable as one
  sub-query. A pattern matched by two-or-more sources (a cross-source join) or zero sources
  is **excluded** from every group — it is handled by the operators phase, never folded into
  a single-source sub-query.
- **`push_group(group, bgp, cap, output_vars, filters, order_keys, limit)`** — the **maximal
  sub-algebra builder**. For one group it builds the most precise `SubQuery` the source's
  `Capability` can answer: the **projection** trimmed to exactly the join + output variables;
  the FILTER conjuncts the source's `FilterClass` covers AND that pass the common-variable
  check; `ORDER BY` / `LIMIT` only when the capability's `order_limit` allows. A full endpoint
  gets the whole group as one multi-pattern `SELECT`; a fragment source (brTPF / plain TPF)
  answers **one** triple pattern (its access unit), so a multi-pattern group is *not* collapsed
  and no filter / order / limit is pushed — honest, not an overclaim that a fragment server
  evaluates a multi-pattern BGP. The result records which conjuncts were **pushed** and which
  were kept **local** (the residual the local engine still evaluates).
- **`common_variable_check(filter, group_vars)`** — the **exact** check Comunica is documented
  to omit (issues #834/#609): a FILTER conjunct is pushed **only when every variable it
  references is bound by the group's patterns**. A conjunct over a variable a sibling group
  binds would evaluate against an *unbound* value remote-side and could drop a solution the
  local plan keeps, so it is kept local. This is the load-bearing safety invariant — push only
  the provably-identical sub-algebra.
- **`render_values_block` / `bind_block_size`** — the cross-group **bind-join block**
  primitive: a `VALUES` block for a full endpoint (block size `DEFAULT_BIND_BLOCK` ≈ FedX's
  bound-join batch), a `maxMpR`-bounded block for brTPF, no block for plain TPF. The rendering
  mirrors `sparq-engine`'s `service.rs::render_values_block` byte-for-byte (that function is
  `pub(crate)`, so it is re-declared here, exactly as Phase 2 re-declares the `Transport`
  seam). Phase 4 owns the block *construction*; the operator that gathers upstream bindings,
  slices them into blocks, and streams the per-block matches is Phase 5.

Pushdown only ever **narrows** what a source returns (a pushed FILTER removes solutions, a
trimmed projection removes columns, a `VALUES`/`maxMpR` block bounds the rows requested) — it
never adds an answer the residual local join would not reattach, so it is correctness-preserving
by construction. The FILTER conjuncts are a **light, pre-parsed model** (`Filter`: variable set +
rendered expression + required `FilterClass`); wiring the real parsed-query FILTER algebra and
the disjunctive-filter decomposition is Phase 5's job when the operators consume a whole query
rather than a bare BGP.

## Public module layout (design §4)

| Module        | Design  | Status / what it holds                                              |
|---------------|---------|----------------------------------------------------------------------|
| `source`      | §4.1    | **Phase 2** — `SourceType` (Endpoint \| BrTpf \| Tpf \| Local) + `FederatedSource` trait + `Endpoint` adapter (SSRF-guarded) |
| `discovery`   | §4.1    | **Phase 1** — VoID/SD discovery → `Capability`; reuses `from_void_nt`; ASK fallback |
| `planner`     | §4.2    | **Phase 3** — `SourceResolver` index→adapter resolution + `lower_leaf` per-leaf lowering |
| `pushdown`    | §4.3    | **Phase 4** — FedX exclusive groups + maximal pushable sub-algebra per group + exact common-variable FILTER check + bind-join block primitive |
| `operators`   | §4.4    | **Phase 3 + 5** — `materialize_single_source` (blocking) + `stream_single_source` (streaming) interpreters, `ScatterPool`, `StreamingJoin`, `parse_srj`, `solutions_equal` |
| `stream`      | §4.4    | **Phase 5** — the bounded, backpressured `SolutionStream` boundary the client owns (engine stays materialised) |
| `adaptive`    | §4.5    | **Phase 7** — `execute_adaptive_single_source` + `AdaptiveOutcome`/`ReplanEvent`: observed-cardinality feedback → re-plan the unjoined remainder, at most once per boundary (behind `fedclient-adaptive`) |

## Opt-in (hard constraint)

The whole client is behind the **`fedclient` cargo feature, OFF by default**, and the
crate is a standalone workspace member with `publish = false`. A build that does not
enable `fedclient` compiles an empty crate (mirrors `sparq-fedplan`'s `fedplan` feature).

```toml
[dependencies]
sparq-fedclient = { path = "crates/sparq-fedclient", features = ["fedclient"] }
```

Enabling `fedclient` pulls in `sparq-fedplan` (`fedplan` planner + `StreamJoin`) and
`sparq-engine` (`service` SRJ transport + VALUES bind-join + SSRF egress guard + local
eval) — the two reuse seams §4 names.

The Phase-7 **adaptive re-planning** path is behind a *second* default-OFF feature,
`fedclient-adaptive`, which implies `fedclient` and additionally pulls
`sparq-fedplan/adaptive-replan` (the planner's `AdaptiveExecutor` re-plan engine). A build
that enables only `fedclient` compiles **zero** adaptive code, and a default build pulls
neither feature:

```toml
[dependencies]
sparq-fedclient = { path = "crates/sparq-fedclient", features = ["fedclient-adaptive"] }
```

## The dependency boundary (load-bearing, enforced)

`sparq-core` and `sparq-engine` **never** depend on `sparq-fedclient`. The dependency
arrow points one-way *into* the engine — the client reuses the engine, the engine never
reuses the client — so the default engine build and the WASM artifact are byte-identical
with or without this crate. That invariant is enforced two ways, both of which **fail if
a future edit introduces such an edge**, in both feature states:

- **`scripts/fedclient-boundary-guard.sh`** — a CI step (wired into `feature-matrix.yml`)
  that inverts the dependency graph with `cargo tree -i sparq-fedclient --all-features`
  and fails if `sparq-core` or `sparq-engine` appears as a dependent (any such edge forms
  a dependency cycle, which the guard detects and reports with the cycle path).
- **`tests/boundary.rs`** — a hermetic `cargo test` that reads `cargo metadata`'s resolve
  graph and asserts neither lean-core member transitively reaches `sparq-fedclient`, plus
  the positive check that the client *does* reach its reuse seams under `--all-features`.

Run the guard locally:

```sh
scripts/fedclient-boundary-guard.sh        # exit 0 = boundary intact
cargo test -p sparq-fedclient --features fedclient --test boundary
```

## Phase 3 — lower a plan + interpret it against one source

`sparq-fedplan` plans a BGP into a `JoinTree` of **pattern indices** and **source
indices** — it speaks indices only, with no endpoint-URL or adapter mapping (the Phase-0
finding). Phase 3 supplies that resolution layer and the materialised interpreter:

```rust,ignore
use sparq_fedclient::{SourceResolver, materialize_single_source, solutions_equal};
use sparq_fedplan::{select_sources, plan_bgp, PlanOptions};

// `descriptors[i]` describes source-adapter `adapters[i]` (SAME order — the resolver's
// single source of truth; every lookup is range-checked, so a mismatch fails closed).
let sel  = select_sources(&bgp, &descriptors);
let tree = plan_bgp(&bgp, &sel, &descriptors, &PlanOptions::default()).unwrap();

let resolver = SourceResolver::new(&bgp, &adapters);    // index → pattern / adapter
let rel = materialize_single_source(&resolver, &sel, source, &tree)?;
// `rel` is the federated answer; for one source over graph `G` it carries the SAME
// solution multiset as `sparq_engine::query(&G, <whole BGP>)` — the load-bearing
// correctness property, asserted by `solutions_equal`.
```

The interpreter is **single-source and blocking** (every leaf relation is fetched in full
before joining). The Phase-5 streaming interpreter below removes the blocking; a multi-source
leaf is still rejected (`InterpError::MultiSource`) rather than under-answered.

## Phase 5 — stream the plan (emit before inputs are exhausted)

`stream_single_source` is the streaming counterpart of `materialize_single_source`. It walks
the **same** `JoinTree` and enforces the **same** single-source guard, but instead of
fetching every leaf in full and blocking-joining, it fans each leaf's blocking fetch out
across a bounded thread-pool and chains the leaves through non-blocking joins:

```rust,ignore
use sparq_fedclient::{stream_single_source, StreamOptions, SolutionStream};
use std::sync::Arc;

let opts = StreamOptions::default();              // workers, channel cap, StreamJoin tuning
let stream: SolutionStream =
    stream_single_source(&resolver, &sel, Arc::clone(&source), &tree, &opts)?;
for item in stream {                              // pulls solutions AS they become derivable
    let sol = item?;                              // …before every leaf has finished fetching
}
```

The pieces:

- **`SolutionStream`** (`stream` module) — a bounded, backpressured `Iterator` over
  `Result<Solution, FedError>`, built on a `std::sync::mpsc::sync_channel`. The channel bound
  IS the backpressure: a producer `emit` blocks once the bound is reached, so a fast source
  cannot grow an unbounded in-flight buffer. No GC heap, no async runtime.
- **`ScatterPool`** (`operators` module) — a bounded blocking thread-pool over the blocking
  transport. The **async/runtime decision** (design §7) lands here: NO async runtime is pulled
  into the dependency tree; all concurrency is `std`-only and confined to this opt-in crate,
  so the lean core is untouched. Each leaf's `source.execute` (a blocking round-trip + parse)
  runs as one pool job; at most `workers` run concurrently.
- **`StreamingJoin`** (`operators` module) — a streaming binary join driven by the planner's
  proven non-blocking `StreamJoin` (the reused symmetric hash join + bounded operator spill).
  It bridges `oxrdf::Term` solutions into `sparq-fedplan`'s lexical `Tuple` model via the
  term's canonical N-Triples form (`Term::Display` ↔ `Term::from_str`, lossless and
  injective), so the streamed join's term equality agrees with the materialised join's.

### The load-bearing invariant (streaming-correctness)

> The streamed solution multiset is **multiset-EQUAL** to the Phase-3
> `materialize_single_source` result for **any** source-arrival interleaving — and both equal
> local `sparq-engine` evaluation of the whole BGP.

`tests/streaming_result_equals_phase3.rs` drives this on the **real** engine path: each leaf
is answered by the real engine over an in-process graph, with a `DelayTransport` injecting
per-leaf sleeps so the fetches complete in different orders. Because each `StreamingJoin`
reuses `StreamJoin` (proven multiset-equal to a blocking hash join for any interleaving and
any spill budget) and computes the same join keys / output header as the Phase-3
`natural_join`, the streamed bag equals the materialised bag — verified across no-delay,
one-leaf-slow, both-slow, chained-join, and forced-spill schedules.

### Honest work-vs-stub split (Phase 5)

REAL here: the `SolutionStream` boundary, the bounded blocking thread-pool, the
`StreamJoin`-feeder streaming join (with spill), the streaming single-source interpreter, the
lossless `Term ↔ Tuple` bridge, and the streaming-correctness test against the real engine.

Still a stub / deferred (a clear slice boundary, not a half-done operator):

- **Multi-source UNION-per-leaf fan-out** — a leaf retained by more than one source is still
  rejected with `InterpError::MultiSource` (the Phase-3 guard), not yet fanned out as a
  per-source union. The thread-pool and `SolutionStream` are the seam that work builds on.
- **The pushed-down streaming bind-join** — `JoinAlgo::Bind` is executed with the same
  streaming symmetric hash join (identical result multiset), not yet as a VALUES / `maxMpR`
  block pushed to the source. That pushdown is Phase 4's `pushdown` module + a follow-up
  bead; the result is correct either way (a bind join and a hash join over the same inputs
  produce the same solutions), only the *request discipline* differs.
- **End-to-end laziness through the engine** — a leaf's `execute` returns a materialised SRJ
  body (the engine stays materialised, §7); the per-leaf stream delivers those parsed rows.
  The streaming win is at the **join** level (a fast leaf feeds the join while a slow leaf is
  still in flight), not solution-level laziness *through* a remote endpoint.

## Adaptive re-planning (Phase 7, `adaptive` module, `fedclient-adaptive` feature)

Phases 3 + 5 execute the **static** plan `sparq-fedplan` commits to from descriptor-derived
estimates. When the descriptors are accurate that plan is simplest and at least as good (§7),
so it stays the default. Phase 7 adds the opt-in feedback loop for when they are **not**:

`adaptive::execute_adaptive_single_source` runs the plan in two phases:

1. **Leaf-scan** — fetch each leaf once through the real adapter and record its REAL observed
   row count into `sparq-fedplan`'s `RuntimeStats`. A leaf scan is order-independent, so this
   reveals the true cardinality of every arm — including the not-yet-joined ones (the ANAPSID
   insight: an arm's divergence is only knowable once its scan completes).
2. **Adaptive join-ordering** — walk the plan stage by stage; at each **operator boundary** ask
   `sparq-fedplan`'s `AdaptiveExecutor` whether to re-plan the **unjoined remainder** given the
   observation-corrected statistics, then join the next leaf with the SAME materialised
   `natural_join` the static interpreter uses. A re-plan fires only when an observation diverges
   past the policy's `divergence_factor`, is adopted only when the cheaper suffix clears the
   hysteresis margin, and is considered **at most once per boundary** — no thrashing.

The re-plan **decision** engine (the divergence trigger, hysteresis, suffix re-ordering, and
the commutativity correctness proof) lives in `sparq-fedplan` (`AdaptiveExecutor`, behind its own
`adaptive-replan` feature); the client does not re-write it — it feeds it real observed
cardinalities and executes the order it returns, exactly as the static phases consume `plan_bgp`.

**Re-planning changes the plan, never the answer.** A BGP's answer is the natural join of its
per-pattern solution multisets, which is commutative + associative, so any join order over the
same pattern set yields the same multiset; a re-plan only reorders the not-yet-executed suffix
and carries the already-joined prefix across unchanged.

- `tests/adaptive_result_equals_static.rs` drives this on the **real engine**: a star whose
  descriptors misestimate `:c` as huge (it is tiny) re-plans the suffix, yet the adaptive result
  is multiset-equal to both the static interpreter and ground-truth `sparq_engine::query`.
- The in-crate `adaptive::tests` assert the same over canned SRJ plus the once-per-boundary
  bound and the inert "accurate descriptors ⇒ no switch" path.

### Honest work-vs-stub split (Phase 7)

REAL: the leaf-scan-then-adaptive-join executor, the observed-cardinality capture, the
`RuntimeStats`/`AdaptiveExecutor` wiring, the once-per-boundary bound, and the correctness
tests against the real engine. The executor is **single-source** (it shares the Phase-3/5
`MultiSource` guard). Because the observed-cardinality boundary is a materialisation point (a
leaf must be drained to be counted), the adaptive path scans each leaf fully once, trading the
within-stage streaming of Phase 5 for the cross-stage re-ordering it buys. Folding the two
together — the ANAPSID "adaptive operator" that estimates a leaf's cardinality from a prefix of
its rows while still streaming it — is a clear next slice, filed under epic sq-3183, not
half-built here.

## Status / roadmap

The **8-phase streaming federation client is feature-complete** — epic **sq-dnko** is closed.
Landed: Phase 0 (skeleton + boundary proof, `sq-s1uy`), Phase 1 (discovery, `sq-nfxl`),
Phase 2 (source abstraction + Endpoint adapter, `sq-rsxf`), Phase 3 (planner bridge +
materialised single-source interpreter, `sq-j27p`), Phase 4 (capability-aware pushdown,
`sq-7byx`), Phase 5 (streaming operators — `SolutionStream` + thread-pool fan-out +
`StreamJoin`-feeder join, `sq-vtba`), Phase 6 (brTPF/TPF adapters, `sq-2qze`), Phase 7
(adaptive re-planning, `sq-ij5x`). Still ahead as future beads under epic **sq-3183**:
multi-source UNION-per-leaf fan-out + the pushed-down streaming bind-join, and the ANAPSID
"adaptive operator" refinement. No performance numbers appear here: any "better than Comunica"
claim in the design record is an *architectural prediction* to be validated head-to-head before
being asserted as fact.

[OPUS-4.8] sq-7byx, sq-vtba, sq-ij5x — flagged for Fable re-review.
