---
name: federated-planning
description: "Cost-based federated SPARQL source selection + bind-vs-hash join planning over already-fetched source descriptors, plus an ANAPSID-style non-blocking streaming join with operator spill, via the opt-in sparq-fedplan crate. Use when planning a federated BGP across multiple SPARQL endpoints from their served statistics (VoID property/class partitions + mined scs: characteristic sets): deciding which sources can contribute to each triple pattern (HiBISCuS recall-safe pruning + CostFed skew-aware cardinality), choosing a join order with per-join bind-vs-hash-vs-streaming algorithm selection (characteristic-set star cardinality for intermediate sizes), and executing a memory-bounded non-blocking symmetric hash join over incrementally-arriving sub-results (StreamJoin, spill to a backing store, result multiset-equal to a blocking join). Pure + deterministic planning, no network I/O. Off by default; does not touch sparq-core/sparq-engine's lean build. Also covers live adaptive RE-planning at stage boundaries (mid-execution plan switching when observed cardinalities diverge from estimates) via the further opt-in adaptive-replan feature (AdaptiveExecutor) — sound because BGP join is order-independent, with mid-operator swap + live source failover deferred."
---

# sparq-fedplan — cost-based federated source selection + join planning

`sparq-fedplan` plans a federated SPARQL **Basic Graph Pattern** (BGP) across several
remote endpoints **from statistics already in hand** — it never contacts the network. A
caller fetches each source's descriptor once (the W3C VoID document a `sparq-server`
serves at `/.well-known/void`, including the mined `scs:` characteristic sets), and the
planner decides, deterministically:

1. **which sources can contribute** to each triple pattern, and
2. **a join order + per-join bind-vs-hash algorithm** over the selected sources.

It is the **opt-in public surface** for cost-based federation planning. Add it
explicitly and enable the `fedplan` feature; it is **not** in sparq's default build
(`sparq-core`/`sparq-engine` stay lean, the wasm artifact is unchanged unless you pull it
in). There is no `sparq-core`/`sparq-engine` dependency — it plans over descriptors only.

## Add the dependency

```toml
[dependencies]
sparq-fedplan = { path = "crates/sparq-fedplan", features = ["fedplan"] }
oxrdf = { version = "0.3", features = ["rdf-12"] }
```

The whole planner is behind the `fedplan` feature (off by default), so even a crate that
depends on `sparq-fedplan` pays nothing for it unless the feature is enabled.

## Build source descriptors

Either programmatically via the builder, or by parsing the served N-Triples document.

```rust
use sparq_fedplan::{SourceDescriptor, SourceId, PredPartition, ClassPartition};

// Programmatic (VoID property/class partitions).
let src = SourceDescriptor::builder(SourceId::new("https://a.example/sparql"))
    .total_triples(10_000)
    .predicate(PredPartition { predicate: "http://xmlns.com/foaf/0.1/knows".into(),
        triples: 2000, distinct_subjects: 1000, distinct_objects: 1800 })
    .class(ClassPartition { class: "http://xmlns.com/foaf/0.1/Person".into(), entities: 1000 })
    .build();

// Or parse the served descriptor (the /.well-known/void N-Triples form, with scs: sets):
let parsed = SourceDescriptor::from_void_nt(SourceId::new("https://b.example/sparql"), nt)?;
```

A descriptor **parsed from VoID partitions is authority-incomplete** (it sees only
predicate/class authorities, never subject/object instance authorities), so
subject/object authority-pruning is disabled for it — recall-safe by construction. To
enable authority pruning, build via `.builder(..)` and call `.authorities_complete()`
only when you truly enumerate every authority the source mints (a HiBISCuS-style
capability set / `void:uriSpace` declaration).

## Select sources for a BGP (recall-safe)

```rust
use sparq_fedplan::{Bgp, TriplePattern, Term, Var, select_sources};

let bgp = Bgp::new(vec![
    TriplePattern::new(Term::Var(Var::new("s")),
        Term::Iri("http://xmlns.com/foaf/0.1/knows".into()), Term::Var(Var::new("o"))),
]);
let sources = [src];
let selection = select_sources(&bgp, &sources);
// selection[i].candidates: the sources retained for pattern i, with estimated_cardinality.
```

**Recall-safety invariant:** a source is pruned for a pattern *only when the descriptor
proves it holds no matching triple*. A bound predicate absent from the source's (complete)
predicate-partition set prunes; a bound class absent from a *declared* class section
prunes; a bound subject/object whose authority is absent prunes *only* when the authority
set is complete. On any uncertainty — open predicate, incomplete authority set, absent
class section — the source is **kept**. The cardinality estimate never prunes (a source
with a tiny or zero estimate is still retained). This is HiBISCuS's design goal: maximise
pruning subject to never losing a result.

## Plan the join (bind vs hash)

```rust
use sparq_fedplan::{plan_bgp, PlanOptions, JoinAlgo, JoinNode};

let plan = plan_bgp(&bgp, &selection, &sources, &PlanOptions::default()).unwrap();
let order: Vec<usize> = plan.join_order(); // patterns in join order (left-deep)
let cost: f64 = plan.total_cost;
```

Each binary join is a **bind join** (cost ≈ `L·(req + fan_out)` — probe the right with the
left's bindings; cheap when the left is small and the right selective) or a **hash /
symmetric join** (cost ≈ `R + L` — scan both sides once; cheap when the left is large or
the right unselective). The decision flips as the left intermediate grows past the point
where per-row requests overtake a full scan; tune the round-trip penalty with
`PlanOptions::request_cost`. Star arms (`?s p₁ ?a . ?s p₂ ?b`) use characteristic-set
cardinality (`Σ_{C⊇Q} count(C)·Π avg_mult`) for intermediate sizes, capturing the
predicate correlation an independence product loses.

## Non-blocking streaming join + spill (`StreamJoin`, sq-vf7q)

The planner's `JoinAlgo::Streaming` choice corresponds to an execution-side operator:
`StreamJoin`, an ANAPSID/XJoin-style **symmetric hash join** over two
incrementally-arriving tuple streams. Feed tuples from either side as federated sub-results
arrive; each `push` returns the results that arrival newly completes — it never blocks on
either input finishing.

```rust
use sparq_fedplan::{StreamJoin, StreamJoinOptions, SpillStore, Tuple, Var, blocking_hash_join};

let opts = StreamJoinOptions { mem_budget_tuples: 100_000, spill_store: SpillStore::TempFile };
let mut join = StreamJoin::new([Var::new("s")], opts);
let _ = join.push_left(Tuple::new([(Var::new("s"), "a".into()), (Var::new("o"), "1".into())]));
let out = join.push_right(Tuple::new([(Var::new("s"), "a".into()), (Var::new("n"), "x".into())]));
assert_eq!(out.len(), 1); // emitted the moment both sides hold key s=a — non-blocking.
```

**Bounded spill.** Memory is capped at `mem_budget_tuples`; when an insert would exceed it,
the largest in-memory join-key partition is spilled to a backing run (a temp file under
`std::env::temp_dir` by default — `std` only, no new dependency; `SpillStore::Memory` is an
in-process simulation for tests). Spilling only relocates tuples; the probe consults both
the live bucket and every spilled run for the key.

**Correctness invariant (load-bearing).** The streamed + spilled result is *multiset-equal*
to `blocking_hash_join(left, right, join_vars)` — same tuples, no loss, no duplication — for
**any** stream interleaving and **any** budget (including one so low every partition spills).
Each matching pair `(l, r)` is emitted exactly once, when the second of the two arrives and
probes the other side (found in memory or a spill run). Proven by the `streamed_equals_*`,
`spill_path_equals_*`, `duplicate_keys_*`, and `emits_before_inputs_exhausted` tests.

The planner picks `Streaming` over plain `Hash` when a hash-class join's combined estimated
inputs `L + R` exceed `PlanOptions::stream_threshold` (default 100 000 rows; set to
`f64::INFINITY` to always use plain hash) — large joins run non-blocking + spillable rather
than materialising a side up front.

## Live adaptive re-planning (opt-in `adaptive-replan` feature, sq-7s4z)

Behind the **off-by-default** `adaptive-replan` cargo feature (which implies `fedplan`), the
crate adds the reactive half of ANAPSID adaptivity: **mid-execution plan switching**. A build
that does not enable the feature compiles **zero** adaptive code (`#[cfg]`-gated out), so the
lean default build and the `fedplan`-only build are byte-unchanged.

`AdaptiveExecutor` models execution as a sequence of **stages** (the left-deep join order)
and holds, at all times, the patterns already joined (the **prefix**) and the patterns still
to join (the **suffix**).

- **Capture** — `RuntimeStats` records the *observed* per-pattern leaf cardinality (real row
  counts the sources returned) and per-source latency, fed in as each stage completes.
- **Trigger** — at each **stage boundary**, `maybe_replan(&stats)` checks whether a
  *not-yet-executed* pattern's observed cardinality `o` diverges from its estimate `e` past
  `ReplanPolicy::divergence_factor` `k` either way (`o > k·e` or `e > k·o`; default `k = 4`).
  If so it re-invokes the cost model on the **remaining** patterns with the observed
  cardinalities substituted in (`corrected_selection`). Source *membership* is never
  re-pruned — only the order changes — so HiBISCuS recall-safety is preserved.
- **Hysteresis** — the re-planned suffix is adopted **only** if its estimated remaining cost
  beats the current suffix's by more than `ReplanPolicy::improvement_margin` (default 10%),
  with a hard `max_replans` budget (default 8). Stable-but-noisy stats never thrash;
  `maybe_replan` returns `ReplanOutcome::{NoDivergence, KeptWithinHysteresis, Switched,
  BudgetExhausted}`.

**Soundness boundary (load-bearing).** Re-planning reorders only the not-yet-started
**suffix** — it is **NOT** a mid-operator swap (an in-flight join is never torn down). A BGP
answer is the natural join of the per-pattern solution multisets, which is **commutative and
associative**: any order over the same patterns yields the **same** result multiset, and the
already-produced prefix is carried across the switch unchanged (no binding lost or
duplicated). Proven by `adaptive::tests::replan_result_equals_static` (a re-plan that
genuinely flips the order yields the identical multiset to the static plan) plus an
exhaustive all-permutations order-independence test.

## Deferred (NOT here)

**Mid-*operator* adaptivity** (tearing down a join while it is producing output and resuming
its half-built hash tables under a new algorithm) and **live source failover** (switching to
a replica mid-stage when a source goes dark — the latency capture exists but the live
multi-source execution layer does not live in this pure crate) are out of scope. Filed as
roadmap beads under epic **sq-3183**.

[OPUS-4.8] sq-a35t / sq-vf7q / sq-7s4z — flag for Fable re-review.
