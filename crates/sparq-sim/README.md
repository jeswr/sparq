# sparq-sim

**Training-free structural entity similarity** for the sparq RDF engine — an opt-in
crate (GenAI phase 1, see [`research/genai-design.md`](../../research/genai-design.md))
that computes similarity straight from the store's existing permutation indexes. No
embeddings, no models, no network, no extra state: the indexes ARE the feature store,
so similarity stays correct under incremental updates for free.

## How it works

- **Signature**: an entity's structural signature is its set of
  `(direction, predicate, neighbor)` pairs — outgoing from one SPO range scan,
  incoming from one OSP/OPS range scan. Cost `O(log n + degree)`.
- **Similarity**: predicate-IDF-weighted Jaccard over two signatures
  (`w(e) = 1 + ln(|G| / freq(pred))`, frequencies from the store's existing planner
  stats) — sharing a rare predicate counts for more than sharing `rdf:type`.
- **`most_similar(a, k)`**: candidate generation through the indexes, **not a full
  scan** — each signature element's co-owners are one contiguous index range (POS for
  outgoing `(p, n)`, SPO for incoming). Candidates accumulate shared-element weight
  (an intersection upper bound); the top `max(4k, 64)` are re-scored exactly.
  Complexity `O(Σ_e min(freq(e), F))` for generation plus `O(M·(log n + d̄))` for
  re-scoring. Elements matched by more than `F = max_pair_frequency` triples (hubs —
  the lowest-IDF, least informative) are skipped during *generation only*; re-scoring
  is always exact. Set the cap to `usize::MAX` for exact-but-slower generation.

## API

```rust
use sparq_sim::{Sim, SimConfig, SignatureMode, weighted_jaccard};

let sim = Sim::new(&graph);                       // defaults: PredicateNeighbor + IDF
let sim = Sim::with_config(&graph, SimConfig {    // or configure:
    mode: SignatureMode::PredicateNeighbor,       //   | Predicates (role/profile sim)
    idf: true,
    exclude_predicates: vec![rdf_type],           // e.g. when rdf:type is ground truth
    max_pair_frequency: 10_000,                   // hub cap (the approximation knob)
});

sim.similarity(&a, &b);                  // weighted Jaccard in [0, 1]
sim.most_similar(&a, 10);                // Vec<(Term, f64)>, best first, self excluded
let sig = sim.signature(&a).unwrap();    // build once, reuse
sim.similar_by_signature(&sig, 10);      // probe with an arbitrary signature
weighted_jaccard(&sig_a, &sig_b);        // for callers that cache signatures
```

Two signature modes, two notions of similarity:

- `PredicateNeighbor` (default): similar = **shares concrete context** (same team,
  same games, same birthplace). This is the mode `most_similar`'s index-driven
  candidate generation is built around.
- `Predicates`: similar = **used the same way** (predicate profile / role similarity —
  two Sports share no concrete neighbor, but their profiles are near-identical).

## Measured results — olympics, 1.78M triples

`bench/qlever-olympics/olympics.nt` (134,730 foaf:Person + SportsTeam/SportsEvent/
Olympics/Sport/City), ground truth = `rdf:type`, **type triples excluded from
signatures** (leakage rule, design doc §5.5). Stratified per-class sampling (40 per
class — the data is 98% Person). Apple M1, 16 GB, rustc 1.89, `--release`.
Reproduce: `cargo run -p sparq-sim --example olympics_eval --release`.

| Metric | Result | Gate |
|---|---|---|
| precision@10 (`most_similar`, same-class) | **0.999** (1498/1500) | > 0.7 ✅ |
| AUC, `Predicates` mode (class separation) | **1.000** | > 0.8 ✅ |
| AUC, `PredicateNeighbor` mode (pairwise) | 0.610 | (see note) |
| `most_similar(k=10)` latency, 240 calls | mean **0.77 ms**, p50 **0.13 ms**, p95 **4.9 ms**, max 7.1 ms | ms-level ✅ |
| Load 1.78M triples + `Sim::new` | 0.9 s | — |

**Note on the two AUCs.** Pairwise AUC asks "do two random same-class entities score
higher than two cross-class ones?". In `PredicateNeighbor` mode most same-class pairs
(two arbitrary athletes) share *no concrete neighbor* and tie with cross-class pairs
at 0 — by design: that mode measures shared context, not class membership. Role
similarity is the `Predicates` mode's job, where class separation is perfect (1.000).
The ranking task the crate is built for — `most_similar` retrieving same-class
entities — is measured by precision@10: 0.999. Per-class: Person 1.000, SportsEvent
1.000, Olympics 1.000, SportsTeam 1.000, City 0.500 (2/4), Sport — (0 candidates);
see [`TODO.md`](TODO.md) for the neighbor-sparse candidate-generation gap behind the
last two.

## Tests

- 11 unit tests (`src/lib.rs`): Jaccard math hand-checks, symmetry, direction,
  IDF ordering, exclusions, mode semantics, hub-cap behaviour, and a generated-taxonomy
  AUC gate (> 0.9, deterministic).
- 1 integration test (`tests/olympics.rs`): API sanity at 1.78M-triple scale —
  skips (passes with a note) when the fixture is absent; override the path with
  `SPARQ_OLYMPICS_NT`.
