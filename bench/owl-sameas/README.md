<!-- [OPUS-4.8] Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns). -->
# OWL `sameAs` equality micro-suite — equality-reasoning closure correctness

A zero-download, fully-deterministic micro-benchmark that gates the OWL-RL **equality**
(`owl:sameAs`) machinery of `sparq-reason`. It is the equality analogue of
[Deep Taxonomy](../deep-taxonomy/README.md) (subclass/transitivity): a structurally-trivial corpus
whose OWL-RL closure size is a **closed form**, so a single deterministic assertion catches any
under- or over-derivation in the `sameAs` path. Registry entry: `owl-sameas` in
[`bench/benchmarks.toml`](../benchmarks.toml).

> **Why a new suite (not a query bolt-on).** No existing reasoning suite exercises `owl:sameAs`
> equality closure: LUBM's entailed tier is subClassOf / restriction / `TransitiveProperty` /
> `inverseOf`; Deep Taxonomy is N3 subclass transitivity. `sparq-reason` materialises `sameAs` by
> **union-find entity rewriting** then expands each class back to the full closure
> ([`crates/sparq-reason/src/owl.rs`](../../crates/sparq-reason/src/owl.rs)) — a distinct code path
> with its own regression surface. This suite is its deterministic gate.

## The workload (per tier `N`)

`gen_sameas.py K N M` emits, in N-Triples, **K independent equivalence classes**, each of **N**
individuals declared equal via a **star** of `N-1` `owl:sameAs` edges to the class anchor, plus
**M** "data" triples on the anchor only:

```turtle
# class c (one of K):
:c{c}_e0 owl:sameAs :c{c}_e1 .   …   :c{c}_e0 owl:sameAs :c{c}_e{N-1} .   # N-1 star edges
:c{c}_e0 :p0 :v0 .   :c{c}_e1 …            # … M anchor data triples (:p0..:p{M-1})
```

`gen.sh` fixes `K=4` and `M=3` so a single numeric tier knob (`N`, the per-class size) drives the
dashboard scaling axis. The star (not a clique) keeps the **input linear in N** while the
**closure is quadratic in N** — exactly the gap a closure-size gate must defend.

## Closed-form closure size (the gate)

After `sparq-cli reason <corpus> ntriples owl`, the materialised closure is **fully determined** by
`(K, N, M)` (these are the engine's actual emitted sizes, verified — not assumed):

| quantity | value | why |
|---|---|---|
| `closure_triples` | `K·N·(N+M)` | `K·N²` full `sameAs` relation (all ordered pairs **incl. reflexive**) + `K·N·M` eq-rep expansion of each anchor data triple onto every member |
| `query_rows` (`query.rq`) | `K·N` | every member inherits the anchor's `:p0 :v0` via `sameAs`; raw corpus returns only `K` |

The committed tiers ([`expected.tsv`](./expected.tsv)):

| `N` | `closure_triples` | `query_rows` | tier |
|----:|------------------:|-------------:|------|
| 8   | 352               | 32           | per-commit |
| 32  | 4 480             | 128          | per-commit |
| 256 | 265 216           | 1 024        | EC2 / nightly (`SAMEAS_TIERS=256`) |

`run.sh` asserts **both** columns; any mismatch is a `sparq-reason` `owl:sameAs` regression and
fails the run loudly. A correct membership query alone does **not** prove the full `N²` `sameAs`
relation was materialised — the `closure_triples` assertion is the load-bearing gate (design A5:
`closure_triples` is the single most valuable reasoner gate, catching silent under-derivation that
no query touches).

## Layout

```text
bench/owl-sameas/
├── gen_sameas.py   pure-Python (stdlib) deterministic generator: K classes × N members × M data
├── gen.sh          thin cache-backed wrapper around gen_sameas.py; emits an N-Triples corpus per tier
├── run.sh          self-asserting runner: materialise the OWL-RL closure per tier, query it, assert expected.tsv
├── query.rq        class-membership probe: SELECT ?x WHERE { ?x :p0 :v0 }  (returns K·N rows)
└── expected.tsv    DETERMINISTIC per-tier closure_triples (= K·N·(N+M)) + query_rows (= K·N)
```

## Running

```sh
cargo build --release -p sparq-cli
bench/owl-sameas/run.sh                 # per-commit tiers N=8, N=32
SAMEAS_TIERS="256" bench/owl-sameas/run.sh   # EC2/nightly tier
```

Honoured env knobs: `SAMEAS_TIERS` (space-separated per-class sizes), `CLI` (sparq-cli path),
`ITERS` (query bench iterations), `SAMEAS_CACHE` (corpus cache dir, default `/tmp/owl-sameas`,
gitignored + regenerable per [`bench/CATALOG.md`](../CATALOG.md) disk discipline).

## Hermeticity

No network, no heavyweight toolchain — `python3` only (already required by the bench harness) plus
the built `sparq-cli`. So, like Deep Taxonomy and unlike LUBM (javac/rapper), this suite runs on the
**per-commit** tier by default (the `sameas` hook in [`scripts/ci-bench.sh`](../../scripts/ci-bench.sh)).
The generator is a pure function of `(K, N, M)` — byte-identical every run — so the corpus, and thus
the closed-form closure size, is reproducible per-commit.
