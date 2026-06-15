<!-- [OPUS-4.8] sq-ustq — full-text-search benchmark suite. Design: research/capability-benchmark-program.md §3.4. -->
# Full-text-search suite

The FTS analogue of the LUBM / SHACL template: an **overview** dashboard row, a
**self-asserting deterministic gate** (regression alerts), and a **competitor** comparison
surface. It exercises `sparq-text` — the opt-in BM25 inverted index + `text:` magic
predicates (`text:matches` AND, `text:matchesAny` OR, prefix `foo*`, `text:phrase`,
`text:near` proximity/slop).

## Two axes (design §3.4)

1. **Latency axis (engine surface)** — `crates/sparq-text/examples/bench_text.rs` over N
   synthetic 8-word literals (a ~10k-term Zipf-skewed vocabulary, deterministic seed),
   loaded into a sparq `Graph`, positions-enabled BM25 index, AND / OR / prefix / phrase /
   near query latency. Per-commit corpus N=100000 (sub-5s); the design's **1M**-literal
   axis is the heavy/latency tier (`bench/fts/gen.sh 1000000`).
2. **IR-quality axis (BEIR)** — Recall@100 / nDCG@10 on a small BEIR cut (SciFact /
   TREC-COVID) loaded as RDF literals vs qrels. **Status: GATHER-ONLY, NOT YET WIRED.** The
   BEIR corpus is not redistributable in-repo, so it is a download-step (gather /
   nightly), not a committed per-PR gate. Tracked as a follow-up bead (see below); the
   per-commit gate is the deterministic latency-axis structure below.

## Data substrate

`bench/fts/gen.sh [N] [seed]` is a thin **parameter source** — the corpus is synthetic and
generated **in-process** by `bench_text` from a deterministic seed, so there is no external
generator (no rapper/javac) and nothing to materialise on disk. It echoes the two pinned
corpus parameters (N then seed) the way LUBM's `gen.sh` echoes its two artifact paths.

Per-commit tier: `N=100000, seed=0`. Heavy/latency tier: `N=1000000` (the design's
1M-literal axis; advisory timing only).

## Workloads

The FIXED 200-query set is drawn from an **independent** seed inside `bench_text`, so the
asserted hit counts shift only when search **semantics** change — never as an artefact of
how many RNG draws corpus generation happened to consume. Each query is a two-term pair:

| Workload | Predicate | Count meaning |
|---|---|---|
| `and_terms` | `text:matches "a b"` | docs containing BOTH terms (AND) |
| `or_terms` | `text:matchesAny "a b"` | docs containing AT LEAST ONE term (OR) |
| `prefix4` | `text:matches "abcd*"` | docs matching the 4-char prefix of term a |
| `phrase` | `text:phrase "a b"` | docs with a, b adjacent and in order |
| `near_slop2` | `text:near "a b"` (slop 2) | docs with a, b in order within gap 2 |

Plus the index footprint:

| Metric | Meaning |
|---|---|
| `bytes_per_doc` | `index.heap_bytes() / index.len()` truncated to an integer |

## Deterministic gate (HARD) vs timing (ADVISORY)

`run.sh` is the self-asserting entry point (the LUBM pattern). It runs `bench_text` on the
pinned corpus and **asserts, per workload, the `count` column vs `expected.tsv`, exit 1 on
any drift**:

- **the query hit counts** (`and_terms` / `or_terms` / `prefix4` / `phrase` / `near_slop2`)
  — the TOTAL hits summed over the fixed query set (the FTS `expected-rows.tsv`; the primary
  semantic gate, integer-exact).
- **`bytes_per_doc`** — the integer index footprint per document (the FTS analogue of
  `store_bytes_per_triple`; runner-noise-immune, integer-exact).

The constants in `expected.tsv` were **derived by running `sparq-text`** on the pinned
corpus (not guessed); the header records the monotonicity cross-checks
(`phrase <= near_slop2 <= and_terms`, `or_terms >= and_terms`, `prefix4 >> or_terms`).

These deterministic metrics also have `mode:"auto"` entries in
`bench/perf-baseline.json` (`fts_bytes_per_doc`) so a footprint regression cross-commit is
ratcheted in addition to the per-commit `expected.tsv` diff.

**Timing is ADVISORY** (`mode:noise`, trend-only, **never hard-gated** — and this dev box is
non-canonical, so its timings are advisory only): the ci-bench hook harvests
`text_and_us` / `text_prefix_us` / `text_build_s` into the dashboard; they are **not** in
`scripts/perf-gate.py`. The hard gate lives in `run.sh`'s `expected.tsv` diff.

## Running it

```sh
cargo build --release -p sparq-text --example bench_text
bench/fts/run.sh                       # self-asserting: exit 1 on any count drift
# heavy/latency tier (advisory timing, 1M literals):
FTS_N=1000000 bench/fts/run.sh
```

`bench_text` emits, per workload, the same `name<TAB>count<TAB>us` 3-column contract the
ci-bench hook consumes (`sparq-text` is the *isolated* FTS crate — not a `sparq-cli`
dependency — so the runner is a crate `--example`, not a CLI subcommand).

## Competitors

**Be honest: Solr / Elasticsearch are NOT SPARQL competitors** (no RDF/SPARQL surface; RDF4J
deprecated its Solr backend) — they are **kept OFF the dashboard**. Two legitimate
references are registered in `bench/competitors.json` (with the `engines`/`values` dashboard
seam **empty** in git per AGENTS.md — no hard-coded perf):

| Engine | Lang | License | Adapter kind | Role |
|---|---|---|---|---|
| **Apache Jena Fuseki + `jena-text` (Lucene SAIL)** | JVM | Apache-2.0 | `http-sparql` | The only like-for-like FTS-over-SPARQL: `text:query` ≈ `text:matches`. Surface peer (dashboard). Gather-only docker. |
| **Lucene (embedded) via Anserini** | JVM | Apache-2.0 | `python-lib` (its OWN IR harness) | Kernel BM25 reference on BEIR. **Labelled "sub-component, not an RDF benchmark"** — off the dashboard. |

Scope the kernel comparison to what `sparq-text` implements (token AND/OR, prefix, phrase,
proximity/slop, BM25 k1=1.2/b=0.75) or it is unfair. A real
`scripts/gather-competitors.sh --run --only <id>` writes git-ignored
`bench/competitor-results/`; docker-based competitors are inherently gather-only on a Docker
EC2 box (no Docker on the dev box), so they add zero recurring CI cost.

## Follow-up

The BEIR IR-quality axis (Recall@100 / nDCG@10 as deficits, G4) is captured as a bead — it
needs a download/gather step (corpus not redistributable in-repo) before it can become a
deterministic gate. Until then the per-commit gate is the deterministic latency-axis
structure (hit counts + `bytes_per_doc`) above.
