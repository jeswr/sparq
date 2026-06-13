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

## How to: hybrid search with text vectors

Structural similarity knows how entities are *connected*; it knows nothing about
what their labels and descriptions *mean* (two differently-modeled descriptions of
the same person share no structure). The opt-in [`sparq-vectors`](../sparq-vectors)
crate covers the text side — `embed_entities` embeds a passage per entity
(label + type + description, configurable) — and ships dependency-free fusion
helpers, so the two signals combine without either crate depending on the other:

```rust
use sparq_sim::Sim;
use sparq_vectors::{fuse_rrf, RRF_K};

let structural: Vec<(Term, f64)> = Sim::new(&graph).most_similar(&query, 50);
let text: Vec<(Term, f64)> = index // sparq_vectors::VectorIndex
    .nearest_term(&query, &graph, &store, 50)
    .into_iter().map(|(t, s)| (t, s as f64)).collect();

// Reciprocal Rank Fusion: rank-based, no score normalization needed
// (weighted Jaccard in [0,1] and cosine in [-1,1] fuse as-is).
let hybrid = fuse_rrf(&[&text, &structural], RRF_K, 10);
```

Over-fetch each signal (k = 50 for a top-10 fusion) so the fusion has overlap to
reward. `fuse_scores(&text, &structural, alpha, k)` is the tunable alternative
(min-max normalized alpha-blend). Full recipe + the research behind it:
[`sparq-vectors` README](../sparq-vectors/README.md) and
[`research/genai-text-embedding-practices.md`](../../research/genai-text-embedding-practices.md).

## Measured results — olympics, 1.78M triples

`bench/qlever-olympics/olympics.nt` (134,730 foaf:Person + SportsTeam/SportsEvent/
Olympics/Sport/City), ground truth = `rdf:type`, **type triples excluded from
signatures** (leakage rule, design doc §5.5). Stratified per-class sampling (40 per
class — the data is 98% Person).

The `olympics_eval` example reports the quality + latency metrics and checks them
against their gates (precision@10 same-class > 0.7; `Predicates`-mode class-separation
AUC > 0.8; ms-level `most_similar(k=10)` latency). Run it for the numbers:

```sh
cargo run -p sparq-sim --example olympics_eval --release
```

The gate it enforces, and the two AUC interpretations below, are the load-bearing
part — the absolute figures print from the example.

**Note on the two AUCs.** Pairwise AUC asks "do two random same-class entities score
higher than two cross-class ones?". In `PredicateNeighbor` mode most same-class pairs
(two arbitrary athletes) share *no concrete neighbor* and tie with cross-class pairs
at 0 — by design: that mode measures shared context, not class membership. Role
similarity is the `Predicates` mode's job, where class separation is perfect (1.000).
The ranking task the crate is built for — `most_similar` retrieving same-class
entities — is measured by precision@10: 0.999. Per-class: Person 1.000, SportsEvent
1.000, Olympics 1.000, SportsTeam 1.000, Sport 1.000, City 0.995.

Sport and City are served by the **neighbor-sparse profile fallback**
(`SimConfig::profile_fallback`, default on): v1 returned Sport 0/0 (no candidates —
every event names exactly one sport, so no two sports share a concrete neighbor) and
City 2/4; the fallback fills starved slots with role-profile matches, taking those
classes to 400/400 and 398/400 at a latency cost of ~0.3 ms on the mean (v1: mean
1.48 ms, p95 8.9 ms). See `TODO.md` for the before/after table.

## Tests

- 13 unit tests (`src/lib.rs`): Jaccard math hand-checks, symmetry, direction,
  IDF ordering, exclusions, mode semantics, hub-cap behaviour (with and without the
  fallback), neighbor-sparse fallback semantics (starved star topology, exact-first
  ranking, no fallback when generation suffices), and a generated-taxonomy
  AUC gate (> 0.9, deterministic).
- 1 integration test (`tests/olympics.rs`): API sanity at 1.78M-triple scale —
  skips (passes with a note) when the fixture is absent; override the path with
  `SPARQ_OLYMPICS_NT`.
