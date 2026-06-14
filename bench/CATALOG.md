<!-- [OPUS-4.8] Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns). -->
# sparq benchmark catalog

**What benchmarks exist + how to run them.** The machine-readable registry is
[`benchmarks.toml`](./benchmarks.toml) (one `[[benchmark]]` per entry, with the
exact invocation, dataset, and pinning). This file is the human guide:
conventions first, then a per-category map that points at the registry, then a
"replicate everything" quickstart.

> Companion: [`research/BENCHMARKS.md`](../research/BENCHMARKS.md) is **what we
> measured** (results, findings, honesty notes). This catalog is **what exists +
> how to run it**. Per-area numbers also live next to each harness (e.g.
> `bench/zk/README.md`, `bench/qlever-baselines.md`, `bench/inference/eye-comparison.md`).

## Conventions (the methodology, codified)

- **Measure first.** Probes like `probe-compress`, `compare-compress`,
  `bench-remap`, and the `bench/parse` baseline exist to *gate a design on
  numbers before building it*. No optimisation lands without a before/after.
- **Differential correctness is a gate, not an afterthought.** Every query
  result is cross-checked against an independent implementation:
  - **Oxigraph** (embedded Rust crate) — `sparq-bench` compare/fuzz/diff, the
    selective harness, the continuous fuzzer.
  - **QLever** (external, Docker or native) — the `bench/qlever-*` harnesses
    compare COUNT *values* and result *sizes*, not just "1 row == 1 row".
  - **oxttl / EYE / nargo** — parser, N3 reasoner, and ZK circuit semantics are
    each pinned against a reference (oxttl for parsing, EYE for N3 closures,
    `nargo`/`bb` for circuits). A harness fails hard on disagreement.
- **QUIET-BOX requirement for wall-clock numbers.** Any entry with
  `quiet_box_sensitive = true` (throughput / latency) must run on an *otherwise
  idle* machine — this box is frequently busy, so do NOT trust absolute
  wall-clock numbers taken under load. Where a contended run is unavoidable,
  prefer the **engine-internal timer** (the CLI's own `in Xs` line, or
  criterion's in-process timing) and report *ratios*, which survive contention
  (see `bench/inference/eye-comparison.md` for the worked example). Entries with
  `quiet_box_sensitive = false` (gate counts, B/triple, byte-identity, pass-rates)
  are deterministic and load-robust.
- **Min-of-N, cold vs warm.** Wall-clock harnesses report the **min of K**
  iterations. State the regime: `sparq-bench`/`cli bench` are **warm** (load
  once, query K times); QLever comparisons are **cold** (cache cleared each run,
  sparq has no query cache); `bench-mmap` open is **cold** first-touch. Keep the
  regime fixed when comparing.
- **Disk discipline.** `bench/*` data is **git-ignored and regenerable** — never
  commit datasets. Scratch goes in `/tmp` (the inference/owl scripts `mktemp -d`
  + trap-clean). Delete large datasets after a run; the wikidata-8b runbook caps
  dataset size and runs a `df` watchdog (abort < 50 GB free). Tracked scripts /
  `.gitignore` are never deleted.
- **Determinism / pinning.** Synthetic generators use fixed seeds (SplitMix64 in
  `sparq-bench`, `random.seed(7)` in selective, index-derived in u64). Pin the
  knob each entry names: `--scale`, fanout, thread list, w3c-rdf-tests commit,
  nargo/bb versions, QLever version + dataset version.
- **Opt-in features cost zero in the core.** `ci-bench` tracks
  `wasm_bundle_bytes` and `store_bytes_per_triple`/`dict_bytes_per_term` per
  commit precisely so a feature leaking into the browser bundle or growing memory
  shows up as a deterministic regression.

## Categories (point at the registry for exact commands)

| category | what it covers | registry ids |
|---|---|---|
| **query** | engine compute + differential correctness; aux-index harnesses; well-known suites | `sparq-bench-compare`, `sparq-bench-fuzz`, `sparq-bench-diff`, `cli-bench-suite`, `cli-bench-mmap`, `operator-coverage`, `sp2b`, `selective-bindjoin`, `u64-valueids`, `qlever-olympics`, `qlever-synthetic-10m`, `qlever-synthetic-100m`, `text-index-bench`, `geo-index-bench`, `rsp-throughput`, `vectors-throughput`, `gpu-bench`, `sim-olympics-eval`, `introspect-olympics` |
| **parse** | text-format parse throughput (MB/s) | `parse-baseline` |
| **ingest** | load + dict + external-memory build throughput | `cli-ingest`, `cli-save-build`, `cli-bench-remap`, `hdt-load-bench`, `wikidata-8b` |
| **compression** | index / result-serialization footprint tradeoffs | `cli-probe-compress`, `cli-compare-compress`, `compress-bench` |
| **scaling** | parallel thread sweep + cross-commit/hardware tracking | `cli-scaling`, `ci-bench`, `ci-bench-ec2`, `hw-bench` |
| **inference** | N3 / RDFS / OWL closure + incremental maintenance | `inference-eye-comparison`, `inference-owl-bench`, `inference-incremental`, `solid-wac-bench` |
| **zk** | commitment pipeline, trace seam, circuit gates, prove/verify | `zk-commit-throughput`, `zk-trace-overhead`, `zk-compose-gates`, `zk-compose-prove-verify` |
| **serve** | concurrent-serving + memory-tiering research spikes | `serve-spikes`, `memtier-spikes` |
| **conformance** | W3C SPARQL + reasoning suites (correctness, not perf) | `sparql-conformance`, `inference-conformance` |

Notes on a few that need care:

- **`bench/serve` + `bench/memtier` are research SPIKES, not maintained
  regression benchmarks.** Their numbers calibrate research docs; re-run on the
  target hardware before trusting any absolute value.
- **`sp2b` (SP2Bench) is tiered** — see [`bench/sp2b/README.md`](./sp2b/README.md).
  The per-commit path builds+caches the real Freiburg generator (BSD; sha256-pinned, g++
  `-O2` not `-O3`) and runs 14 sub-second queries on a fixed 250k-triple corpus, emitting
  `sp2b_<query>_<mode>_us` (trend-only) plus a HARD expected-rows correctness diff. Three
  intentionally-pathological queries (q05a/q06/q12a, tens of seconds at 250k) sit in
  `queries-heavy/` for the EC2/nightly tier; the **full 5M-100M scale** belongs to
  `bench-ec2.yml`/nightly (`bench/sp2b/gen.sh <triples>` at a larger `-t`).
- **`wikidata-8b` is external-cost and gated.** It builds the full Wikidata
  truthy dump (~8-9.4B triples) on a 16 GB EC2 box (~$5-17). It is **blocked
  until dict-spill merges to public main** — see
  [`bench/wikidata-8b/RUNBOOK.md`](./wikidata-8b/RUNBOOK.md) §0 (hard launch gate)
  and `STATUS.md`. Do not launch without the budget + gate checks.
- **CI tracking**: `bench.yml` runs `ci-bench` on every push to main (free
  runners, trend + large-regression alert, no hard-fail); `bench-ec2.yml` runs
  the heavier version weekly on spot (NOT live until `AWS_BENCH_ROLE_ARN` is set
  — fails harmlessly at OIDC, no cost). Both push to the orphan **`benchmark-data`**
  branch via github-action-benchmark.

## Replicate everything — quickstart

```sh
# --- build once ---
cargo build --release -p sparq-cli -p sparq-bench

# --- query: differential + perf vs Oxigraph (warm, min-of-K) ---
cargo run -p sparq-bench --release -- --scale 50000 --iters 4
# continuous correctness fuzz (deterministic, shardable by category):
cargo run -p sparq-bench --release -- fuzz 0 5000 all

# --- selective bind-join + u64 value-id probes ---
python3 bench/selective/gen.py 500000 > bench/selective/selective.nt
./target/release/sparq-cli bench bench/selective/selective.nt ntriples bench/selective/queries 3 count
python3 bench/u64-valueids/gen.py 1000000 /tmp/t3-literals.nt
./target/release/sparq-cli bench /tmp/t3-literals.nt ntriples bench/u64-valueids/queries 3 materialize

# --- ingest / compression probes ---
./target/release/sparq-cli ingest <truthy-slice.nt.zst> full
./target/release/sparq-cli save  <data.nt> ntriples /tmp/idx
./target/release/sparq-cli probe-compress /tmp/idx/spo.perm   # B/triple schemes
./target/release/sparq-cli bench-mmap /tmp/idx bench/qlever-synthetic/queries 5 count

# --- scaling sweep (OWN the box: idle, all cores) ---
./target/release/sparq-cli scaling <data.nt> ntriples bench/qlever-synthetic/queries 1,2,4,8 3

# --- inference ---
SPARQ_CLI=target/release/sparq-cli bench/inference/owl-bench.sh
EYE=$HOME/.local/bin/eye SPARQ_CLI=target/release/sparq-cli bench/inference/eye-comparison.sh
cargo run -p sparq-reason --example incremental_olympics_bench --release

# --- zk (standalone projects + Noir toolchain) ---
( cd bench/zk        && cargo bench )
( cd bench/zk-trace  && cargo bench )
bench/zk-compose/scripts/gate_counts.sh > bench/zk-compose/gate_counts_latest.json
bench/zk-compose/scripts/prove_verify.sh filter_int_d1

# --- conformance (correctness gates) ---
scripts/fetch-conformance.sh && cargo run -p sparq-conformance
scripts/fetch-inference-suites.sh && cargo run -p sparq-conformance --bin sparq-inference-conformance

# --- vs QLever (needs QLever installed; see each dir's README) ---
( cd bench/qlever-olympics && ../../.qlever-venv/bin/python compare.py 5 compute )

# --- CI emitter locally / per-platform hardware sweep ---
bash scripts/ci-bench.sh 200000 /tmp/bench-results.json
./scripts/hw-bench.sh 500000 /tmp/hw-bench-results.csv
```

QLever-based comparisons reuse stored reference numbers in
[`bench/qlever-baselines.md`](./qlever-baselines.md) so QLever does **not** need
re-running for every sparq iteration — re-measure QLever only when its version or
the dataset changes (record date + commit).
