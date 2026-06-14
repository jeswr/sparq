<!-- [OPUS-4.8] Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns). -->
# WatDiv (Waterloo SPARQL Diversity Benchmark)

The canonical SPARQL **query-shape diversity** suite: a synthetic e-commerce/social RDF
generator (the WSDBM schema) plus 20 "Basic Testing" query templates grouped by topology —
**Linear** (L1-L5), **Star** (S1-S7), **Snowflake** (F1-F5), **Complex** (C1-C3). Where
SP2Bench stresses operators on a fixed DBLP corpus, WatDiv stresses *join structure* across a
graph whose entity types and predicate distributions span a deliberately wide structurality
range. Registry entry: `watdiv` in [`bench/benchmarks.toml`](../benchmarks.toml).

> **Attribution.** Generator + query templates are from the University of Waterloo Data
> Systems Group: Güneş Aluç, Olaf Hartig, M. Tamer Özsu, Khuzaima Daudjee,
> *"Diversified Stress Testing of RDF Data Management Systems"*, ISWC 2014 (LNCS 8796,
> pp. 197-212). WatDiv is distributed for research use; we **vendor + attribute** the query
> templates and a vendored wordlist, and we **do NOT redistribute the generator binary** —
> `gen.sh` fetches the upstream source tarball (pinned by sha256) and builds it locally.

## Layout

```
bench/watdiv/
├── gen.sh                build-once-and-cache the real WatDiv generator, emit a FIXED corpus
├── run.sh                self-contained CI runner (gen + count/materialize/json + row diff)
├── expected-rows.tsv     deterministic per-commit solution counts (correctness diff)
├── files/words.txt       vendored ASCII wordlist for STRING literals (hermetic; no apt dep)
├── queries/              16 per-commit Basic-Testing queries (non-empty + sub-ms at SF=1)
└── queries-heavy/        4 queries empty at SF=1 (need scale) → EC2/nightly SF≥10 tier
```

## Generator decision (empirical)

The canonical `dsg-uwaterloo/watdiv` GitHub repo is a **docs-only Jekyll site** — the buildable
source ships as the **v0.6 tarball** linked from its README. That tarball is small C++ with a
**plain Makefile (no cmake)** and, measured on this box (g++ 13.3 / Boost 1.83), builds **clean,
unpatched** (its Makefile already pins `-std=c++0x -w`, and the code uses no bare `<cstdint>`
types, so the classic modern-GCC breakage does not bite). Build + SF=1 generation is ~15 s cold
and **fully cached** afterward. We therefore use the **real generator on the per-commit path**
(not a fallback), so the corpus is the genuine WatDiv WSDBM graph.

**Pinned source** (see `gen.sh` header for full detail):
* `https://dsg.uwaterloo.ca/watdiv/watdiv_v06.tar`
* sha256 `fb8d930b74b3fbc8f948101bfaf658a90d2f74002f1fefda465c45ffd33a71d2`

**Determinism.** Upstream seeds its RNGs from wall-clock time, so two raw runs differ. `gen.sh`
applies two mechanical seed patches (boost `mt19937` and `srand`, both → `1u`) on the data path,
making SF=1 generation **byte-identical across runs** (sha256-verified). Seed `1u` is the **pin**:
the query constants in `queries/*.rq` and the counts in `expected-rows.tsv` were chosen against
the seed-1 SF=1 corpus — changing the seed reshuffles which entities carry which properties and
will break the row diff.

**Hermeticity (wordlist).** Upstream hard-codes `/usr/share/dict/words` (the `wamerican` apt
package). To avoid an apt dependency, `gen.sh` applies a third patch making the dictionary paths
env-overridable and points STRING literals at the vendored `files/words.txt`. The per-query row
counts are **independent of the wordlist content** (they bind to typed/structural entities, not
literal text — verified by regenerating with an alternate wordlist: identical counts), so the
vendored list only needs to be non-empty + deterministic. firstnames/lastnames ship in the
tarball and are used from there.

> Output is **N-Triples** (one `<s>\t<p>\t<o> .` per line), so load with sparq format
> `ntriples`, not `turtle`.

## Query instantiation

WatDiv templates carry placeholders (`%v0%`, …) that the upstream `-q` tool substitutes with
random data-drawn entities (non-deterministic, shuffled). Instead of that tool, we **instantiate
each template once by hand** with a fixed concrete constant drawn from the seed-1 SF=1 corpus, so
each query is plain runnable SPARQL with a **non-empty, stable** result. Each `.rq` documents the
placeholder it replaced. Three Star/Snowflake templates (S3, S4, S5) carry an embedded canonical
constant whose exact combination is empty at SF=1 (e.g. `sorg:publisher`/`sorg:language
Language0` never co-occur on the same subject in ~100k triples); for those we swap **one arm** for
a co-occurring predicate to keep the WatDiv **shape** non-empty at the per-commit scale, noting
the deviation inline — the canonical-constant forms run at the EC2/nightly tier.

## Run it

```sh
cargo build --release -p sparq-cli
bench/watdiv/run.sh 1          # ensures corpus, runs count/materialize/json, checks rows
# or manually:
CORPUS=$(bench/watdiv/gen.sh 1)
target/release/sparq-cli bench "$CORPUS" ntriples bench/watdiv/queries 3 count   # also materialize|json
```

CI wires the per-commit subset through `scripts/ci-bench.sh` (iters=3) as metrics
`watdiv_<query>_<mode>_us` — **trend-only** (NOT in the deterministic hard perf-gate,
`scripts/perf-gate.py`) — plus a hard **expected-rows equality** check (`expected-rows.tsv`) on
count mode, so a correctness regression fails the build even though the latency is trend-only.

## Tiering — per-commit vs EC2/nightly

| tier | scale | queries | gate |
|---|---|---|---|
| **per-commit** | SF=1 (~106k distinct triples, deterministic, hermetic) | the 16 in `queries/` (all non-empty, all sub-ms; slowest ~140 µs) | `watdiv_*_us` trend + HARD row diff |
| **EC2/nightly** | SF=1000 (~100M+) and the IL-generated stress workload | full 20 (incl. `queries-heavy/`: F1/F4/C1/C2) | per-query timeout + result-size assertion |

`queries-heavy/` holds the 4 Basic-Testing templates that are **empty at SF=1 because their
property/join combinations are too rare at ~100k triples** (not because they are slow): **F1**
(`sorg:trailer` has zero triples at SF=1), **F4** (compound homepage/language snowflake), **C1**
and **C2** (4-/10-hop complex joins that do not connect at small scale). They are kept as faithful
WatDiv templates and populate at scale; run them at SF≥10 (`bench/watdiv/gen.sh <SF>`) on the
`bench-ec2.yml`/nightly tier with per-query timeouts and result-size assertions.
