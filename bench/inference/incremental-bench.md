# Incremental inference maintenance — insert/delete delta benchmarks

**Thread:** incremental-inference (2026-06-12) · **Harness:**
`crates/sparq-reason/examples/incremental_olympics_bench.rs`
(`cargo run -p sparq-reason --example incremental_olympics_bench --release [olympics.nt]`)
· **Machine:** Apple M1, 16 GB, rustc 1.89.0, `--release`, default (parallel) features.

The question these benchmarks answer: with a materialized closure already built, what does a
small base mutation cost **incrementally** (counting maintenance: `MaterializedGraph` /
`MaterializedOwlGraph` / `MaterializedN3Graph`) versus the v1 story (full
re-materialization), for **both inserts and deletes**?

## Methodology

* **Dataset (RDFS/OWL):** `bench/qlever-olympics/olympics.nt` — the real QLever olympics
  dataset, **1,781,625 triples** (athletes/teams/results; gitignored, fetch via the QLever
  harness in that directory). The data carries only 765 `rdfs:subClassOf` triples of its
  own, so each workload adds a small synthesized TBox over the dataset's REAL vocabulary
  (printed by the bench, defined in `olympics_tbox`): class chains above `foaf:Person` /
  `dbo:SportsTeam` / `dbo:SportsEvent` / `dbo:Olympics`, domains/ranges for the heavy
  predicates (`olympics:athlete/games/event/medal`, `dbo:team`, `foaf:age/gender`),
  2 subPropertyOf links; the OWL workloads add `dbo:team owl:inverseOf syn:hasMember` +
  `foaf:Person owl:equivalentClass syn:Human` (CountingMono), and the fixpoint variant
  additionally declares a transitive `syn:locatedIn` over a 200-edge synthetic
  venue→city→region→country→continent chain (transitive subgraphs being small relative to
  the dataset is the realistic shape).
* **Deltas:** inserts are FRESH athlete-shaped triples (new athlete: type/team/age + a
  result edge); deletes are uniformly sampled EXISTING ABox triples (TBox triples excluded —
  TBox deltas are the documented full-rebuild path by design). Sizes 1 / 100 / 10,000.
  After each measurement the delta is reverted, and the closure is asserted equal to a
  from-scratch run at the end (the oracle rides along).
* **Baseline:** full `materialize_rdfs` / `materialize_owl_rl` over the same base — what an
  update had to pay before incremental maintenance.
* **Single-shot timings** (not best-of-N): the deltas are sub-millisecond, so relative
  numbers are indicative; the orders of magnitude are stable across runs.

## RDFS (`MaterializedGraph`, counting)

Base 1,781,647 → closure 2,774,595. Initial incremental build 1.17 s;
full `materialize_rdfs` 1.05 s.

| delta | insert | vs full | delete | vs full |
|---:|---:|---:|---:|---:|
| 1 | 3 µs | ~406,000x | 1 µs | ~1,800,000x |
| 100 | 117 µs | ~9,000x | 44 µs | ~24,000x |
| 10,000 | 16.4 ms | 64x | 8.2 ms | 128x |

## OWL 2 RL (`MaterializedOwlGraph`)

**CountingMono** (inverseOf + equivalentClass; mirrors the batch monotone path).
Base 1,781,649 → closure 2,588,876. Initial build 0.79 s; full `materialize_owl_rl` 1.21 s.

| delta | insert | vs full | delete | vs full |
|---:|---:|---:|---:|---:|
| 1 | 3 µs | ~346,000x | 2 µs | ~646,000x |
| 100 | 63 µs | ~19,000x | 52 µs | ~23,000x |
| 10,000 | 2.6 ms | 465x | 3.9 ms | 308x |

**CountingFixpoint** (+ transitive `syn:locatedIn`; exact transitive layer).
Base 1,781,850 → closure 3,049,248. Initial build 0.79 s; full `materialize_owl_rl` 1.73 s.

| delta | insert | vs full | delete | vs full |
|---:|---:|---:|---:|---:|
| 1 | 4 µs | ~488,000x | 4 µs | ~483,000x |
| 100 | 45 µs | ~39,000x | 39 µs | ~44,000x |
| 10,000 | 53.4 ms | 32x | 3.4 ms | 512x |

`full_rebuilds()` stays 0 throughout — no ABox delta touches a fallback path. Deletion costs
the same order as insertion (the counting property: no overdelete/rederive pass).

## N3 / WAC ACL maintenance (`MaterializedN3Graph`, counting fast path)

Rules: the REAL `crates/sparq-solid/rules/common.n3 + wac.n3` (read-only; the
?UNSCOPED-migrated WAC rules qualify for counting — asserted by
`tests/incremental_n3_prop.rs`). Pod: synthetic 1k-doc fixture shaped like the sparq-solid
loader output — 200 containers x 5 docs + root (1,201 resources), root owner ACL
(agent, accessTo+default, Read/Write/Control), every 10th container with its own ACL
(public-read agentClass + group write + an origin-restricted grant), one 8-member group:
**1,578 reasoner input facts → 14,406-fact closure**.

* Full engine re-run (the v1 maintenance story, `reason_n3_terms` over rules + facts):
  **0.84 s**.
* Initial incremental build (counting + the inheritedAcl/ancestor recursive layers):
  **0.49 s** — the incremental evaluator is *faster than the batch engine* on this rule set
  thanks to anchored premise evaluation.

| ACL edit (1 triple) | insert | vs re-run | delete | vs re-run |
|---|---:|---:|---:|---:|
| grant a mode on the root owner auth | 10.9 ms | 78x | 71.9 ms | 12x |
| add an agent to the root owner auth | 27.6 ms | 31x | 161.5 ms | 5x |
| add an `ownAcl` (guard predicate) | 784 ms — documented full rebuild (1 rebuild counted) | | | |

The root-owner edits are the WORST case for this pod: the owner authorization applies to all
1,201 resources, so one triple flips ~1.6–3.2 k derived grants; container-local ACL edits
are proportionally cheaper. `solidx:ownAcl` (and `acl:origin`) are negation-guard
predicates: their truth is baked into every derivation count, so mutating them takes the
documented rebuild path — which costs the same as the initial build, i.e. still less than
the batch engine re-run.

### Qualification matrix (sparq-solid rule sets)

| rules | mode | reason |
|---|---|---|
| common.n3 + wac.n3 | **Counting** | input-stratified NAF (ownAcl/origin), recursive ancestry/inheritance handled by SCC layers |
| common.n3 + acp-a.n3 | **Counting** | NAF over acp:agent/acp:client (input-only within the stratum) |
| common.n3 + acp-b.n3 | **Counting** | NAF over solidx:acceptsAgentP/acceptsClientP (inputs to this stratum) |
| common.n3 + acp-c.n3 | Fallback | simple-grant rules conclude `{ ?p ?pred ?r }` — a VARIABLE predicate (bound from `solidx:allowPred`/`denyPred` data), so the derived-predicate set is not statically known and predicate-level stratification is unsound |

Chaining incremental ACP strata (stratum k's closure diff feeding stratum k+1's
insert/delete) is wired-ready for a/b but pointless until acp-c is restructured to ground
conclusion predicates (4 rule variants instead of `?mode solidx:allowPred ?pred`); left as
follow-up with the rules' owner.

## Caveats

* The olympics TBox is synthesized (documented above) because the dataset ships none; the
  ABox — the part that matters for delta cost — is the real 1.78M-triple dump.
* Insert deltas intern fresh IRIs into the shared `Dict`; that cost is included in the
  incremental timings.
* The WAC pod is synthetic but loader-shaped (sparq-solid's `assemble_input` output schema);
  the real fixture lives behind sparq-solid's private loader. The engine-re-run baseline
  (0.84 s) is consistent with the ~1 s full `materialize_wac` pipeline measured by
  `cargo run -p sparq-solid --example bench --release` on the real 1.1k-graph fixture.
