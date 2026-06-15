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
| **query** | engine compute + differential correctness; aux-index harnesses; well-known suites | `sparq-bench-compare`, `sparq-bench-fuzz`, `sparq-bench-diff`, `cli-bench-suite`, `cli-bench-mmap`, `operator-coverage`, `sp2b`, `dbpsb`, `watdiv`, `bsbm`, `lubm`, `shacl-validate-bench`, `selective-bindjoin`, `u64-valueids`, `qlever-olympics`, `qlever-synthetic-10m`, `qlever-synthetic-100m`, `text-index-bench`, `geo-index-bench`, `rsp-throughput`, `vectors-throughput`, `gpu-bench`, `sim-olympics-eval`, `introspect-olympics` |
| **parse** | text-format parse throughput (MB/s) | `parse-baseline` |
| **ingest** | load + dict + external-memory build throughput | `cli-ingest`, `cli-save-build`, `cli-bench-remap`, `hdt-load-bench`, `wikidata-8b` |
| **compression** | index / result-serialization footprint tradeoffs | `cli-probe-compress`, `cli-compare-compress`, `compress-bench` |
| **scaling** | parallel thread sweep + cross-commit/hardware tracking | `cli-scaling`, `ci-bench`, `ci-bench-ec2`, `hw-bench` |
| **inference** | N3 / RDFS / OWL closure + incremental maintenance | `inference-eye-comparison`, `inference-owl-bench`, `inference-incremental`, `deep-taxonomy`, `solid-wac-bench` |
| **zk** | commitment pipeline, trace seam, circuit gates, prove/verify | `zk-commit-throughput`, `zk-trace-overhead`, `zk-compose-gates`, `zk-compose-prove-verify` |
| **serve** | concurrent-serving + memory-tiering research spikes | `serve-spikes`, `memtier-spikes` |
| **conformance** | W3C SPARQL + reasoning suites (correctness, not perf) | `sparql-conformance`, `inference-conformance` |
| **competitors** | versioned external-engine comparison (Oxigraph / QLever / eye) + version+env capture | `competitor-gather` (registry: [`competitors.json`](./competitors.json)) |

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
- **`dbpsb` (DBPSB/FEASIBLE) is tiered + fetch-and-cache (real DBpedia)** — see
  [`bench/dbpsb/README.md`](./dbpsb/README.md). Per-commit, `fetch.sh` downloads ONE
  sha256-pinned DBpedia Databus slice (CC-BY-SA; `mappingbased-objects_lang=en` 2019.09.01,
  N-Triples despite the `.ttl` ext) and emits a DETERMINISTIC head cut of 750k triples, then
  runs 13 sub-second curated FEASIBLE/DBPSB queries emitting `dbpsb_<query>_<mode>_us`
  (trend-only) plus a HARD expected-rows correctness diff. Three unselective queries
  (`queries-heavy/`) and the **full ~11.8M artifact** (the `.bz2` ingested directly via the
  fused-decompress path) — up to DBpedia 'latest-core' ~1B triples — belong to the
  EC2/nightly tier.
- **`watdiv` (WatDiv) is tiered** — see [`bench/watdiv/README.md`](./watdiv/README.md). The
  per-commit path builds+caches the real Waterloo v0.6 generator (research-use; sha256-pinned,
  g++ + Boost, RNG seed-pinned to `1u`) and runs the 16 sub-ms Basic-Testing queries on a fixed
  SF=1 corpus (~106k triples), emitting `watdiv_<query>_<mode>_us` (trend-only) plus a HARD
  expected-rows correctness diff (count mode). Four templates empty at SF=1 (F1/F4/C1/C2) sit in
  `queries-heavy/` for the **EC2/nightly SF≥10 tier** (`bench/watdiv/gen.sh <SF>`).
- **`bsbm` (Berlin SPARQL Benchmark, Explore mix) is tiered + fetch-and-cache** — see
  [`bench/bsbm/README.md`](./bsbm/README.md). Per-commit, `gen.sh` fetches the PREBUILT bsbmtools
  v0.2 distribution (JRE-only; sha256-pinned zip) and emits a deterministic `-fc -pc 300` corpus
  (~116k triples), then runs the 11 Explore queries emitting `bsbm_<query>_<mode>_us` (trend-only)
  plus a HARD expected-rows correctness diff against **MATERIALIZE** mode (the mix has a CONSTRUCT
  + a DESCRIBE — graph-valued forms report produced-triple counts). `query06` (unanchored regex,
  omitted from the official mix) + the **full ~100M+ scale** and the Explore-and-Update / Business
  Intelligence mixes belong to the EC2/nightly tier (`bench/bsbm/gen.sh <product_count>`).
- **`lubm` (Lehigh University Benchmark) is the REASONING suite** — see
  [`bench/lubm/README.md`](./lubm/README.md). `run.sh` is self-asserting: it builds the LUBM(1)
  corpus + Univ-Bench TBox (UBA generator, Apache-2.0 pinned commit, javac-only; `rapper` for
  RDF/XML→NT), materializes the **OWL-RL** closure (`sparq-cli reason … owl` — RDFS is incomplete
  here), then runs the **extensional** tier (Q1/Q2/Q3/Q14 on the raw ABox) + the **entailed** tier
  (Q4-Q13 on the closure; each returns 0 on raw data and its correct count only after reasoning),
  asserting BOTH tiers' counts vs `expected-rows.tsv`. CI emits `lubm_<query>_count_us`
  (trend-only) and fails on any count mismatch (reasoner OR engine regression). Full-scale
  **LUBM(1000)** (~133M triples) is the EC2/nightly tier (`bench/lubm/gen.sh 1000 0`).
- **`shacl-validate-bench` is the SHACL VALIDATION suite** — see
  [`bench/shacl/README.md`](./shacl/README.md). It REUSES the LUBM(1) ABox as its data substrate
  (so it shares the `javac`/`rapper` guard) × the **5 committed shape graphs** under
  `bench/shacl/shapes/` (`cardinality`, `datatype_range`, `class_nodekind`, `node_paths`,
  `sparql_constraint`). `run.sh` is self-asserting: it validates with the `bench_shacl` example and
  asserts each workload's **violations / conforms / focus_nodes** vs `expected.tsv` (deterministic
  at the pinned corpus + shapes; exit 1 on drift). The W3C core pass-count ratchet (98/98) lives in
  `crates/sparq-shacl/tests/w3c_core.rs` (`BASELINE_PASS`, only-tightens). CI emits
  `shacl_<workload>_validate_us` (trend-only, advisory). Heavy tiers: `univ=5`/`univ=10`. The
  cleanest competitor surface — Jena-SHACL / pySHACL / rdf-validate-shacl run the *identical*
  `(data, shapes)` pair (see the competitor map below + `scripts/bench-adapters/`).
- **`text-index-bench` is the FULL-TEXT-SEARCH suite** — see
  [`bench/fts/README.md`](./fts/README.md). It exercises `sparq-text` (BM25 inverted index +
  `text:` magic predicates) over a **synthetic** corpus generated **in-process** (no external
  generator — no `javac`/`rapper`): N seeded 8-word literals over a ~10k-term Zipf vocab. `run.sh`
  is self-asserting: it runs the `bench_text` example and asserts each workload's **total hit
  count** (`and_terms` / `or_terms` / `prefix4` / `phrase` / `near_slop2`, summed over a FIXED
  200-query set drawn from an independent seed) and the integer **index bytes-per-doc** vs
  `expected.tsv` (deterministic at the pinned `N=100000 seed=0` corpus; exit 1 on drift).
  `fts_bytes_per_doc` also has a `mode:auto` ratchet in `bench/perf-baseline.json`. CI emits
  `text_<workload>_us` + `text_build_s` (trend-only, advisory). Heavy/latency tier: `N=1000000`.
  An IR-quality BEIR axis (Recall@100 / nDCG@10) is gather-only and not yet wired (follow-up bead).
  Competitor honesty: **Solr/ES are NOT SPARQL competitors and stay off the dashboard**; the
  surface peer is Fuseki + `jena-text` (`http-sparql`), the kernel ref is Lucene/Anserini (labelled
  *sub-component, not an RDF benchmark*).
- **`deep-taxonomy` (DeepTaxonomy) is the rule-heavy N3 REASONING suite** — see
  [`bench/deep-taxonomy/README.md`](./deep-taxonomy/README.md). `run.sh` is self-asserting and
  REUSES the existing generator `bench/inference/gen_deeptaxonomy.py` (1 instance + a depth-deep
  `:sc` chain + 1 transitivity meta-rule): per depth tier it materializes the **N3 forward
  closure** (`sparq-cli reason … n3`), runs a class-membership `query.rq` over it, and asserts
  BOTH the closure triple-count (`= 2·depth+1`) AND the query rows (`= depth+1`) vs `expected.tsv`
  — a deterministic, load-robust gate that fails LOUDLY on a reasoner regression. Needs only
  `python3` (no g++/javac), so it runs on the per-commit tier at a SMALL depth pair (dt1k+dt10k);
  dt100k is opt-in via `DEEPTAX_DEPTHS` for EC2/nightly. CI emits
  `deeptax_d<DEPTH>_{closure_s,query_us,closure_triples}` (trend-only). The dashboard features it
  as a scaling suite (depth axis) with EYE external-reference baselines (cited from
  `bench/inference/eye-comparison.md`; dt100k = n/a, EYE not run).
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
- **`competitor-gather` is the versioned external-engine comparison** — see the
  registry [`competitors.json`](./competitors.json) (Oxigraph embedded Rust dep /
  QLever Docker image / eye N3 binary: pinned version, install+run recipe, and the
  per-engine map of which sparq suites each is comparable on) and the orchestrator
  [`scripts/gather-competitors.sh`](../scripts/gather-competitors.sh). It is
  **safe-by-default**: a bare invocation DRY-RUNS (prints what it would do + a
  tool/version/env report, runs no benchmark, pulls no image). A real gather is
  guarded behind `--run --only <id>`; it caps the synthetic `--scale`, runs a `df`
  watchdog, cleans `/tmp` scratch, and writes a results file per engine recording
  the competitor's **version + host env** into git-ignored `bench/competitor-results/`.
  The script never edits the tracked JSON; a maintainer maps a reviewed result into
  the dashboard SEAM (`engines`/`values` in `competitors.json`) deliberately — and
  per the *No hard-coded performance numbers* rule, no figures are baked into git.
  Comparable-suite map (registry-driven): **Oxigraph** ↔ `sparq-bench-compare`,
  `sp2b`, `watdiv`, `bsbm`, `dbpsb`, `lubm` (extensional only); **QLever** ↔
  `qlever-olympics`, `qlever-synthetic-10m/100m`, `watdiv`, `bsbm`, `lubm`
  (extensional); **eye** ↔ `inference-eye-comparison` + `deep-taxonomy` (DeepTaxonomy/anc500/grid30);
  **Jena-SHACL / pySHACL / rdf-validate-shacl** ↔ `shacl-validate-bench` (identical data×shapes;
  cross-check `#violations`/`conforms` per-engine before trusting timing — `report-cli`/`js-lib`
  adapter kinds in `scripts/bench-adapters/`, gather-only on a Docker EC2 box).
  HONESTY NOTE (per parent `sq-i0nm`): a gh-runner gather is noisy and not
  comparable to the EC2/quiet-box reference band — the recorded `env.quiet_box`
  flag lets the dashboard label it distinctly (see the QUIET-BOX convention above).

## Replicate everything — quickstart

```sh
# --- build once ---
cargo build --release -p sparq-cli -p sparq-bench

# --- query: differential + perf vs Oxigraph (warm, min-of-K) ---
cargo run -p sparq-bench --release -- --scale 50000 --iters 4
# continuous correctness fuzz (deterministic, shardable by category):
cargo run -p sparq-bench --release -- fuzz 0 5000 all

# --- well-known query suites (self-contained runners: gen/fetch + run + hard row diff) ---
CORPUS=$(bench/sp2b/gen.sh 250000)   && ./target/release/sparq-cli bench "$CORPUS" turtle    bench/sp2b/queries 3 count
CUT=$(bench/dbpsb/fetch.sh 750000)   && ./target/release/sparq-cli bench "$CUT"    ntriples bench/dbpsb/queries 3 count
bench/watdiv/run.sh 1                 # WatDiv SF=1 (g++ + Boost): gen + count/materialize/json + row diff
bench/bsbm/run.sh                     # BSBM Explore -pc 300 (JRE + unzip): gen + materialize + row diff
bench/lubm/run.sh                     # LUBM(1) (javac + rapper): gen + OWL-RL closure + both tiers + row diff
bench/shacl/run.sh                    # SHACL (javac + rapper): LUBM ABox x 5 shapes + violations/conforms/focus_nodes diff
cargo build --release -p sparq-text --example bench_text && bench/fts/run.sh   # Full-text (no external tool): synthetic BM25 corpus + hit-count/bytes-per-doc diff
bench/deep-taxonomy/run.sh            # DeepTaxonomy (python3 only): N3 closure per depth tier + closure-size + query-row gate

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

# --- versioned competitor comparison (Oxigraph / QLever / eye) — registry: bench/competitors.json ---
scripts/gather-competitors.sh                       # dry-run: tool+version+env report (runs nothing)
scripts/gather-competitors.sh --list                # show the pinned registry + comparable-suite map
scripts/gather-competitors.sh --run --only oxigraph --scale 50000 --iters 4   # real gather (records version+env)

# --- CI emitter locally / per-platform hardware sweep ---
bash scripts/ci-bench.sh 200000 /tmp/bench-results.json
./scripts/hw-bench.sh 500000 /tmp/hw-bench-results.csv
```

QLever-based comparisons reuse stored reference numbers in
[`bench/qlever-baselines.md`](./qlever-baselines.md) so QLever does **not** need
re-running for every sparq iteration — re-measure QLever only when its version or
the dataset changes (record date + commit).
