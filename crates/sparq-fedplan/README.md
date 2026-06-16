# sparq-fedplan

Cost-based **federated source selection** + **bind-vs-hash join planning** over
already-fetched source descriptors — a small, **opt-in** planner that consumes the
statistics a sparq-server already serves, with **no network I/O**.

Given a SPARQL Basic Graph Pattern (BGP) and a set of `SourceDescriptor`s — each
carrying a remote endpoint's W3C VoID property/class partitions plus its mined
**characteristic sets** (served under the `scs:` vocab) — this crate decides which
sources can contribute to each pattern (HiBISCuS-style **recall-safe** pruning +
CostFed-style skew-aware cardinality), then builds a join plan with a per-join
**bind-vs-hash** decision and a greedy join order, using characteristic-set star-join
cardinality to estimate intermediate-result sizes.

The planner is **pure and deterministic**: it plans from descriptors a caller has
already fetched; it never contacts the network. Timing here is **non-canonical**
(work-box EC2) and is therefore not recorded.

> Model: Opus 4.8 (Fable unavailable — flag for re-review when Fable returns).
> Bead **sq-a35t** · epic **sq-3183** (cost-based federated source selection + join planning).

## 🚀 Quickstart

```rust
use sparq_fedplan::{
    Bgp, TriplePattern, Term, Var, SourceDescriptor, SourceId, PredPartition,
    select_sources, plan_bgp, PlanOptions,
};

// A two-arm star: ?s foaf:knows ?o . ?s foaf:name ?n
let bgp = Bgp::new(vec![
    TriplePattern::new(Term::Var(Var::new("s")),
        Term::Iri("http://xmlns.com/foaf/0.1/knows".into()), Term::Var(Var::new("o"))),
    TriplePattern::new(Term::Var(Var::new("s")),
        Term::Iri("http://xmlns.com/foaf/0.1/name".into()), Term::Var(Var::new("n"))),
]);

// A source described by its served VoID partitions (build programmatically or parse
// the served N-Triples with `SourceDescriptor::from_void_nt`).
let src = SourceDescriptor::builder(SourceId::new("https://endpoint.example/sparql"))
    .total_triples(10_000)
    .predicate(PredPartition { predicate: "http://xmlns.com/foaf/0.1/knows".into(),
        triples: 2000, distinct_subjects: 1000, distinct_objects: 1800 })
    .predicate(PredPartition { predicate: "http://xmlns.com/foaf/0.1/name".into(),
        triples: 1000, distinct_subjects: 1000, distinct_objects: 990 })
    .build();
let sources = [src];

// 1. Source selection (recall-safe prune + skew-aware cardinality).
let selection = select_sources(&bgp, &sources);
// 2. Join plan (bind-vs-hash decision + greedy join order).
let plan = plan_bgp(&bgp, &selection, &sources, &PlanOptions::default()).unwrap();

println!("join order: {:?}", plan.join_order());
println!("estimated total cost: {}", plan.total_cost);
```

## ✨ Features

- **HiBISCuS-style source selection** — prunes a source for a pattern only on positive
  evidence it cannot contribute (a bound predicate absent from the source's complete
  predicate partition set; a bound class absent from a *declared* class section; a bound
  subject/object authority absent from a *complete* authority capability set). The
  **recall-safety invariant** holds: a source that could return any binding is never
  pruned — uncertainty always keeps the source. Proven by `recall_safe_*` tests.
- **CostFed-style skew-aware cardinality** — per-(pattern, source) cardinality comes from
  the served per-predicate triple count and average multiplicity, not a uniform guess, so
  per-predicate skew and bound-position selectivity (subject/object) are preserved.
- **Characteristic-set star cardinality** — joining star arms (`?s p₁ ?a . ?s p₂ ?b`) is
  estimated with Neumann & Moerkotte's `Σ_{C⊇Q} count(C)·Π avg_mult`, capturing the
  predicate *correlation* a per-predicate-independence product loses.
- **Bind-vs-hash join decision** — each binary join picks a **bind join** (probe the
  right with the left's bindings — cheap when the left is small + right selective) or a
  **hash/symmetric join** (scan both sides once — cheap when the left is large), and the
  decision *flips* at the expected cost threshold (tunable `PlanOptions::request_cost`).
- **ANAPSID-style non-blocking streaming join + bounded spill** (`StreamJoin`) — the
  execution-side operator the planner's `JoinAlgo::Streaming` choice names. It consumes two
  incrementally-arriving tuple streams (federated sub-results at different rates), builds *and*
  probes both sides, and emits matches as soon as both sides have a join key — without first
  materialising either full input. Memory is bounded by `StreamJoinOptions::mem_budget_tuples`:
  over-budget join-key partitions spill to a backing store (a temp file by default — `std`
  only, no new dependency) and are reconciled on probe. **Correctness invariant** (tested): the
  streamed + spilled result is *multiset-equal* to the equivalent blocking hash join
  (`blocking_hash_join`) for any stream interleaving and any budget — no loss, no duplication.
  A large hash-class join (`L + R` past `PlanOptions::stream_threshold`) is marked
  `JoinAlgo::Streaming` so it runs non-blocking + spillable rather than materialising a side.
- **Opt-in, zero core overhead** — a standalone member (like `sparq-canon`/`sparq-prov`),
  gated behind the `fedplan` cargo feature, **off by default**. Nothing in sparq's
  default build or the wasm artifact depends on it; the lean core is byte-identical
  without it. No `sparq-core`/`sparq-engine` dependency — it plans over descriptors only.
- **Pure & deterministic** — no network I/O; same descriptors + same BGP ⇒ same plan.

## Scope — covered vs deferred

| Capability | Status |
|---|---|
| Source selection (HiBISCuS prune + CostFed cardinality) | ✅ covered here |
| Bind-vs-hash decision + greedy join order + CS star cardinality | ✅ covered here |
| ANAPSID-style non-blocking streaming join with operator spill (`StreamJoin`) | ✅ covered here (sq-vf7q) |
| Live adaptive re-planning at stage boundaries (`AdaptiveExecutor`), cardinality- **and** per-source-latency-weighted | ✅ covered here, opt-in `adaptive-replan` (sq-7s4z + sq-b51o) |
| Mid-*operator* swap (tear down an in-flight join) + live source failover | ⏳ deferred (roadmap bead, epic sq-3183) |

`plan_bgp` itself is still a **static** planner — it commits to one order from the estimates
it is given. The opt-in `adaptive-replan` feature adds an `AdaptiveExecutor` on top that
reacts at run time (next section). sq-vf7q added the streaming + spill *operator* the plan
can name; mid-*operator* adaptivity remains a bead under epic sq-3183.

## Adaptive re-planning (opt-in `adaptive-replan` feature, sq-7s4z)

Behind the **off-by-default** `adaptive-replan` cargo feature (which implies `fedplan`), the
crate adds live **mid-execution plan switching** — the reactive half of ANAPSID adaptivity.
A build that does not enable the feature compiles **zero** adaptive code (the module is
`#[cfg]`-gated out), so the lean default build and the `fedplan`-only build are unaffected.

- **Capture actual stats** — `RuntimeStats` records the *observed* per-pattern leaf
  cardinality (real row counts the sources returned) and per-source latency, fed in by the
  executor as each stage completes. Latency is **EWMA-smoothed** per source (below), so the
  cost model reacts to the trend, not the last raw sample.
- **Re-plan trigger** — at each **stage boundary** (between two leaf joins of the left-deep
  plan), if a *not-yet-executed* pattern's observed cardinality `o` diverges from its
  estimate `e` past `ReplanPolicy::divergence_factor` `k` in either direction (`o > k·e` or
  `e > k·o`; default `k = 4`), **or** ([OPUS-4.8] sq-b51o) a not-yet-executed pattern's
  slowest source is observed at more than `k×` the latency baseline, the planner is re-invoked
  on the **remaining** patterns with the observed cardinalities substituted in
  (`corrected_selection`) and the join costs latency-weighted (below). Source *membership* is
  never re-pruned — only the order changes — so recall-safety is preserved.
- **Per-source latency weighting (sq-b51o) — a HEURISTIC, not optimal.** Cardinality is not
  the only thing the estimates get wrong: a source can be *slow* (contended / far /
  rate-limited) even at exactly its predicted row count. Each candidate join's cost is scaled
  by a latency factor derived from the **slowest** observed latency over the pattern's retained
  sources (a union is bottlenecked by its slowest arm):
  `factor = clamp(1 + latency_weight·(s − 1), latency_floor, latency_cap)` where
  `s = ewma_latency / latency_baseline`. A source at baseline — **or with no latency
  observation** — gets `factor = 1.0`, so a measurement-free re-plan is byte-identical to the
  cardinality-only planner; a 2×-slow source costs 1.5× at the default `latency_weight = 0.5`,
  and the `latency_cap` (default 4.0) stops one outlier from dominating. The constants
  (`latency_weight = 0.5`, `latency_baseline = 100.0`, `latency_floor = 0.5`,
  `latency_cap = 4.0`) are **hand-tuned, not derived**: this is a deliberately gentle *bias*
  toward faster sources / deferring a slow one, **not** a claim to find the latency-optimal
  plan. Latency enters only the *cost* term (and the suffix-selection score), **never** the
  output cardinality — so results are unchanged (see the soundness boundary). Set
  `latency_weight = 0` to disable it entirely (pure cardinality planning).
- **Latency EWMA smoothing (sq-b51o follow-up) — a HEURISTIC α, not optimal.** The cost model
  + trigger are fed not the single last latency sample but a per-source **exponentially-weighted
  moving average**: each `RuntimeStats::record_source_latency` folds the new sample in as
  `ewma_new = α·observed + (1−α)·ewma_prev` (the first sample seeds the average). α is
  `RuntimeStats::latency_alpha`, default `RuntimeStats::DEFAULT_LATENCY_ALPHA = 0.3` — a
  **hand-picked** smoothing factor (latest sample 30%, history 70%), **not** a workload-derived
  optimum. This is the cleaner anti-thrash discipline that replaces the bare single-sample value:
  a **single transient spike** moves the average only a fraction of the way and does *not*, by
  itself, clear the re-plan trigger, while a **sustained** shift converges past it within a few
  samples. The absolute `latency_floor`/`latency_cap` clamp is kept as a **final guard** on the
  resulting cost factor. Use `RuntimeStats::with_latency_alpha(α)` to override (`α = 1.0` recovers
  the old un-smoothed "last sample wins" behaviour); higher α tracks faster but is twitchier,
  lower α is calmer but laggier.
- **Hysteresis / anti-thrash** — the re-planned suffix is adopted **only** if its estimated
  remaining cost (cardinality- **and** latency-weighted) beats the current suffix's by more
  than `ReplanPolicy::improvement_margin` (default 10%) and there is a hard `max_replans`
  budget (default 8). Stable-but-noisy stats — **including jittery latency** — therefore never
  cause the plan to flap; `maybe_replan` returns a `ReplanOutcome` (`NoDivergence` /
  `KeptWithinHysteresis` / `Switched` / `BudgetExhausted`).

**Soundness boundary** — re-planning reorders only the not-yet-started **suffix**; it is
**not** a mid-operator swap (a join already producing output is never torn down). Because a
BGP is a conjunction of triple patterns, its answer is the natural join of the per-pattern
solution multisets, which is **commutative and associative** — any order over the same
patterns yields the **same** result multiset. The already-produced prefix is carried across
the switch unchanged, so no binding is lost or duplicated. **The latency weighting does not
move this boundary**: it enters only the cost/ordering, never the output cardinality or the
pattern set, so a latency-driven reorder is the same kind of pure suffix permutation. This
result-equivalence is proven in `adaptive::tests::replan_result_equals_static` (a
cardinality-driven re-plan), `latency_replan_result_equals_static` (a re-plan driven purely by
latency) **and** `ewma_replan_result_equals_static` (a re-plan driven by the EWMA-smoothed
latency) — each genuinely flips the order yet yields the identical multiset to the static plan —
backed by an exhaustive all-permutations order-independence test. The EWMA changes only *when*
the latency path fires (anti-thrash), never the soundness boundary: re-planning is still a
sub-query / **stage-boundary** suffix reorder, never a mid-operator swap. Mid-*operator*
adaptivity and live source failover are deferred (epic sq-3183).

## Streaming join (quick use)

```rust
use sparq_fedplan::{StreamJoin, StreamJoinOptions, Tuple, Var, blocking_hash_join};

let mut join = StreamJoin::new([Var::new("s")], StreamJoinOptions::default());
// Feed tuples from either side as they arrive; each push returns newly-derivable results.
let _ = join.push_left(Tuple::new([(Var::new("s"), "a".into()), (Var::new("o"), "1".into())]));
let out = join.push_right(Tuple::new([(Var::new("s"), "a".into()), (Var::new("n"), "x".into())]));
assert_eq!(out.len(), 1); // emitted as soon as both sides have key `s=a` — non-blocking.
```

Cap memory with `StreamJoinOptions::mem_budget_tuples`; over-budget partitions spill
(`SpillStore::TempFile` by default, `SpillStore::Memory` for tests). The result equals
`blocking_hash_join` regardless of budget or arrival order.

## 📚 Learn more

- Skill: `skills/federated-planning/SKILL.md`
- Served source statistics: `crates/sparq-introspect` (VoID + `scs:` characteristic sets)
- Federation discovery descriptors: `crates/sparq-server/src/descriptors.rs`
- HiBISCuS (Saleem & Ngonga Ngomo, ESWC 2014); CostFed (Saleem et al., SEMANTiCS 2018);
  characteristic sets (Neumann & Moerkotte, ICDE 2011); ANAPSID (Acosta et al., ISWC 2011).

## License

MIT — `publish = false` workspace member. [OPUS-4.8]
