<!-- [OPUS-4.8] Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns). -->
# DBPSB / FEASIBLE (DBpedia SPARQL Benchmark)

A real-data SPARQL suite over a pinned slice of **DBpedia**: query-log-shaped queries (the
DBPSB / FEASIBLE family) over a fixed, sha256-pinned DBpedia Databus file. Unlike SP2Bench
(synthetic generator) this suite is **fetch-and-cache** — there is no generator; the point of
DBPSB is to run realistic queries over *real* DBpedia. Registry entry: `dbpsb` in
[`bench/benchmarks.toml`](../benchmarks.toml).

> **Attribution / license.** Data is **DBpedia**, published on the
> [DBpedia Databus](https://databus.dbpedia.org/), licensed **CC-BY-SA 4.0**
> (© DBpedia Association / Wikipedia contributors). We do not vendor the data; `fetch.sh`
> downloads ONE pinned file (verified by sha256) and caches it. The DBPSB benchmark design is
> from Morsey, Lehmann, Auer, Ngonga Ngomo, *"DBpedia SPARQL Benchmark – Performance Assessment
> with Real Queries on Real Data"* (ISWC 2011); FEASIBLE (Saleem et al., 2015) is the
> feature-based query-selection method over real query logs. The `.rq` files here are a curated
> subset adapted to the predicates present in the pinned slice.

## The pinned slice

| field | value |
|---|---|
| **URL** | `https://databus.dbpedia.org/dbpedia/mappings/mappingbased-objects/2019.09.01/mappingbased-objects_lang=en.ttl.bz2` |
| **sha256** | `1cd51b2f3673196764f356943627e087f949eb24d7f7494a391a7f8154f7ad7b` |
| **bytes (compressed)** | `119919822` (~120 MB bzip2) |
| **account / group / artifact / version** | `dbpedia` / `mappings` / `mappingbased-objects` / `2019.09.01` (English) |
| **full triple count** | ~11.8M object-property triples (EC2/nightly tier) |
| **per-commit cut** | deterministic **head of 750,000 triples** (`fetch.sh 750000`) |

**Format note (load-bearing).** Despite the `.ttl.bz2` extension the file is plain
**N-Triples** (one full-IRI `s p o .` per line, no `@prefix`). So:

- sparq loads it with format **`ntriples`** (NOT `turtle`) — the streamed parallel parser, no
  full decompressed copy in RAM;
- a deterministic head-cut by **line** is a deterministic cut by **triple**. The artifact is
  grouped by subject and its serialization order is fixed by the pinned sha256, so
  `bzcat | head -n 750000` yields the **same** 750k triples every run — reproducible without
  re-pinning.

The fused-decompress path means the CLI also accepts the `.bz2` directly
(`sparq-cli bench slice.ttl.bz2 ntriples …`) for the **full** artifact at the EC2 tier; the
per-commit path uses the smaller `.nt` cut so the run is hermetic and fast.

### Why this artifact (vs `instance-types`)

`mappingbased-objects` is a SINGLE file with rich predicate variety — `dbo:birthPlace`,
`deathPlace`, `country`, `occupation`, `genre`, `team`, `award`, `almaMater`, `spouse`,
`child`, `starring`, `homepage`, … (563 distinct predicates in the 750k cut) — so the curated
subset can exercise BGP / star / chain joins, OPTIONAL, UNION, FILTER, DISTINCT, ORDER+LIMIT,
aggregates and a subquery against **one** pinned download. `instance-types` is `rdf:type`-only
and would not support that breadth.

## Layout

```
bench/dbpsb/
├── fetch.sh             download + sha256-verify ONE pinned slice; emit a deterministic cut
├── expected-rows.tsv    deterministic per-commit solution counts (correctness diff)
├── queries/            13 per-commit-safe FEASIBLE/DBPSB queries (sub-second at 750k)
└── queries-heavy/       3 intentionally-unselective queries → EC2/nightly tier
```

### Per-commit queries (all confirmed non-empty + deterministic on the 750k cut)

| query | shape | rows (count) |
|---|---|---|
| q01 | BGP, 1 pattern, bound object (born in NYC) | 873 |
| q02 | chain join (person→birthPlace→country) | 9913 |
| q03 | star join (birthPlace ∧ deathPlace) | 48064 |
| q04 | OPTIONAL left-join (occupation, opt spouse) | 23607 |
| q05 | UNION (birthPlace ∪ deathPlace) | 76077 |
| q06 | DISTINCT + FILTER regex on an IRI | 43 |
| q07 | FILTER `!=` on a bound IRI | 49555 |
| q08 | DISTINCT + ORDER BY + LIMIT | 100 |
| q09 | GROUP BY + COUNT + HAVING + ORDER DESC + LIMIT | 50 |
| q10 | single scalar aggregate `COUNT(*)` | 1 |
| q11 | negation-by-OPTIONAL + `!bound` | 22763 |
| q12 | multi-pattern place-join anchored on a constant (London) | 367650 |
| q13 | subquery + `COUNT(DISTINCT)` + HAVING + ORDER + LIMIT | 25 |

(`rows` is the count-mode **solution count** — q10 is `1` because it is a single scalar row;
its `COUNT(*)` value is 50394.)

## Run it

```sh
cargo build --release -p sparq-cli
CUT=$(bench/dbpsb/fetch.sh 750000)                                  # fetch+verify+cut, cached
target/release/sparq-cli bench "$CUT" ntriples bench/dbpsb/queries 3 count   # also materialize|json
```

CI wires the per-commit subset through `scripts/ci-bench.sh` (iters=3) as metrics
`dbpsb_<query>_<mode>_us` — **trend-only** (NOT in the deterministic hard perf-gate,
`scripts/perf-gate.py`), with a hard **expected-rows equality** check (`expected-rows.tsv`) on
count mode so a correctness regression fails the build even though latency is trend-only.

## Tiering — per-commit vs EC2/nightly

- **Per-commit** (`ubuntu-latest`): the 750k-triple cut + the 13 sub-second `queries/` (slowest
  ~55 ms, the regex-FILTER q06). The compressed slice is fetched once and cached.
- **EC2 / nightly** (`queries-heavy/` + the FULL ~11.8M artifact): three intentionally
  unselective queries (`h01` co-birthplace pairs, `h02` cross-predicate 3-chain, `h03`
  unbounded place self-join — `h03` alone is ~3M solutions / ~700 MB json on the *cut*) and the
  full artifact (`sparq-cli bench .../mappingbased-objects_lang=en.ttl.bz2 ntriples …`, the
  `.bz2` ingested directly via the fused-decompress path). Run with per-query timeouts +
  result-size assertions there. The whole DBpedia **'latest-core' (~1B triples)** belongs to
  the nightly/EC2 tier only.

### CI wiring (per-commit, not applied in this branch — assembled centrally)

The GitHub Actions job caches the pinned slice keyed by URL+sha so steady-state runs do no
network, then builds + runs `scripts/ci-bench.sh` (which contains the guarded `dbpsb` hook):

```yaml
  - name: Cache pinned DBpedia slice
    uses: actions/cache@v4
    with:
      path: /tmp/dbpsb
      # key is the pinned URL+sha256 — a re-pin (new slice) busts the cache automatically.
      key: dbpsb-mappingbased-objects-en-2019.09.01-1cd51b2f
  - run: cargo build --release -p sparq-cli -p sparq-bench
  - run: bash scripts/ci-bench.sh   # emits dbpsb_<query>_<mode>_us + the hard expected-rows diff
```

## Disk discipline

`fetch.sh` scratches under `/tmp/dbpsb` (override with `DBPSB_CACHE`). It keeps the ~120 MB
compressed slice (so re-cutting at another size needs no re-download) and the ~100 MB cut, but
**never** materialises the full decompressed text (~1.6 GB) — the cut is produced by a streaming
`bzcat | head`. Set `DBPSB_KEEP_SLICE=0` to delete the compressed slice after cutting. All
`bench/*` scratch is gitignored + regenerable.
