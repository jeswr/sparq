---
name: federated-planning
description: "Cost-based federated SPARQL source selection + bind-vs-hash join planning over already-fetched source descriptors, via the opt-in sparq-fedplan crate. Use when planning a federated BGP across multiple SPARQL endpoints from their served statistics (VoID property/class partitions + mined scs: characteristic sets): deciding which sources can contribute to each triple pattern (HiBISCuS recall-safe pruning + CostFed skew-aware cardinality), and choosing a join order with per-join bind-vs-hash algorithm selection (characteristic-set star cardinality for intermediate sizes). Pure + deterministic, no network I/O. Off by default; does not touch sparq-core/sparq-engine's lean build. NOT for live streaming/adaptive joins (ANAPSID — deferred)."
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

## Deferred (NOT here)

**ANAPSID-style non-blocking streaming joins with operator spill** and **live adaptive
re-planning** are out of scope — this is a *static* plan computed up front. They are filed
as a roadmap bead under epic **sq-3183**.

[OPUS-4.8] sq-a35t — flag for Fable re-review.
