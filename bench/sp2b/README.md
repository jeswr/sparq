<!-- [OPUS-4.8] Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns). -->
# SP2Bench (SPARQL Performance Benchmark)

The most operator-focused, deterministic of the well-known SPARQL suites: a DBLP-in-RDF
generator plus 17 canonical queries (Q1-Q12 with the Q3/Q5/Q12 a/b/c variants) that exercise
long path joins, OPTIONAL-heavy left joins, UNION, FILTER, negation-by-`OPTIONAL`+`!bound`,
DISTINCT, ORDER BY, LIMIT/OFFSET, and ASK. Registry entry: `sp2b` in
[`bench/benchmarks.toml`](../benchmarks.toml).

> **Attribution.** Generator + queries are from the Freiburg University Database Group
> (DBIS), Michael Schmidt et al., *"SP²Bench: A SPARQL Performance Benchmark"* (ICDE 2009,
> arXiv:0806.4627), distributed under a **BSD license** (see the `COPYING` inside the
> upstream tarball that `gen.sh` fetches). The 17 `.rq` files here are the published query
> texts verbatim. We do not vendor the generator source; `gen.sh` fetches it (pinned by
> sha256) and builds it.

## Layout

```
bench/sp2b/
├── gen.sh                 build-once-and-cache the real sp2b_gen, emit a FIXED corpus
├── expected-rows.tsv      deterministic per-commit solution counts (correctness diff)
├── queries/              14 per-commit-safe queries (sub-second at 250k)
└── queries-heavy/         3 intentionally-expensive queries → EC2/nightly tier
```

## Generator decision (empirical)

The real `sp2b_gen` is small BSD-licensed C++ with a **plain Makefile (no cmake)** and builds
hermetically from source given only `g++`. Measured on this box (aarch64): build ~few-seconds,
`-t 250000` generation **0.36 s** producing a **26 MB** Turtle corpus; the output is
**byte-identical across runs** (sha256-verified) and sparq loads it in **~0.17 s**. Because the
build is hermetic (one sha256-pinned 2.7 MB tarball GET, then fully cached) and the corpus is
deterministic, we use the **real generator on the per-commit path** rather than a fallback —
result sizes match the published reference. See `gen.sh`'s header for the two mechanical
portability patches required (missing `<cstring>`/`<cstdlib>` includes; an
unsigned-`char`-vs-`EOF` infinite-loop fix for aarch64) and why `-O2` (not `-O3`) is kept (the
upstream Makefile warns `-O3` is not semantics-preserving and changes the generated document).

> Output is **Turtle/N3** (the generator emits `@prefix` + prefixed names + `^^xsd:string`),
> so load with sparq format `turtle`, **not** `ntriples`.

## Run it

```sh
cargo build --release -p sparq-cli
CORPUS=$(bench/sp2b/gen.sh 250000)               # build+cache gen, emit fixed corpus
target/release/sparq-cli bench "$CORPUS" turtle bench/sp2b/queries 3 count   # also materialize|json
```

CI wires the per-commit subset through `scripts/ci-bench.sh` (iters=3) as metrics
`sp2b_<query>_<mode>_us` — **trend-only** (NOT in the deterministic hard perf-gate,
`scripts/perf-gate.py`), with a hard **expected-rows equality** check (`expected-rows.tsv`)
on count mode so a correctness regression fails the build even though the latency is trend-only.

## Tiering — per-commit vs EC2/nightly

At the 250k per-commit scale, three queries are intentionally pathological (SP2Bench designed
them to stress unselective joins / negation) and take tens of seconds, so they are **not** in
the per-commit path — they live in `queries-heavy/` for the EC2/nightly tier. Measured at 250k
(1 iter, count): q05a ~58 s, q06 ~52 s, q12a ~59 s; everything in `queries/` is sub-second
(the slowest, q04, is ~0.32 s). The **full-scale SP2Bench tier (5M-100M triples)** belongs to
`bench-ec2.yml` / nightly with per-query timeouts and result-size assertions — run
`bench/sp2b/gen.sh <triples>` at the larger `-t` there.

`bench/sp2b/run-ec2.sh` is the self-asserting EC2/nightly entry point. It runs the three heavy
queries at 250k, then generates 5M, 25M, and 100M corpora and runs the committed reference
subset at each scale. Each query has an independent timeout (set
`SP2B_QUERY_TIMEOUT_SECONDS`), and each count must match `expected-rows-ec2.tsv`. Set
`SP2B_FULL_SCALES` to select scales for a manual diagnostic run.
