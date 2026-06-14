<!-- [OPUS-4.8] Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns). -->
# BSBM — Berlin SPARQL Benchmark (Explore mix)

The classic e-commerce SPARQL benchmark: a generator that emits a synthetic product /
vendor / offer / review graph, plus three official query *mixes* (Explore, Explore-and-Update,
Business Intelligence). This directory wires the **Explore** mix at CI scale. The Explore mix
exercises navigational lookups, DISTINCT + ORDER BY + LIMIT/OFFSET, UNION, OPTIONAL-heavy left
joins, negation-by-`OPTIONAL`+`!bound`, numeric-range and `langMatches` FILTERs, a self-join
similarity query, a `DESCRIBE`, and a `CONSTRUCT` export. Registry entry: `bsbm` in
[`bench/benchmarks.toml`](../benchmarks.toml).

> **Attribution.** BSBM and the `bsbmtools` generator are by Christian Bizer and Andreas
> Schultz (Freie Universität Berlin), *"The Berlin SPARQL Benchmark"* (IJSWIS 2009). The
> generator + query templates are distributed by the BSBM project on SourceForge
> (`sourceforge.net/projects/bsbmtools`; GPL-family license — see the files inside the
> distribution that `gen.sh` fetches). A maintained mirror is `github.com/afs/BSBM`. The
> `.rq` files here are the Explore query *templates* (`queries/explore/queryN.txt` in the
> distribution) **instantiated** with constants pinned to entities present at `-pc 300`; each
> file's header records exactly which template it came from and which constants were filled in.

## Layout

```
bench/bsbm/
├── gen.sh                 fetch-once-and-cache the PREBUILT bsbmtools, emit a FIXED corpus
├── run.sh                 self-contained per-commit runner (gen + bench + expected-rows diff)
├── expected-rows.tsv      deterministic per-query result sizes (correctness diff)
├── queries/               11 per-commit Explore queries (the official mix: 1-5, 7-12)
└── queries-heavy/         query06 (regex; excluded from the official mix) → EC2/nightly tier
```

## Generator decision (empirical)

The canonical `bsbmtools` v0.2 SourceForge distribution ships a **prebuilt `lib/bsbm.jar`**
plus its dependency jars (`ssj`, `log4j`, `jdom`), so generation needs only a **JRE** — **no
Maven, no Ant, no compile step** (the JDK-21 `java` already on the box suffices). `gen.sh`
fetches the 2.4 MB zip once (sha256-pinned `40f5e59b…`), caches the unpacked tools, and runs
`benchmark.generator.Generator -fc -pc 300 -s nt`. The generator is **deterministic** — at a
given `-pc` it emits byte-identical N-Triples every run (seeded RNG; sha256-verified in
`gen.sh`'s idempotent path), so the corpus is reproducible per-commit and the instantiated
Explore queries have stable result sizes. At `-pc 300` it produces **~115,987 triples** (the
design's ~100k per-commit target) in a couple of seconds.

`-fc` (forward chaining) materialises the `rdfs:subClassOf` product-type closure **into** the
data, so the Explore queries' `?p a <leaf-type>` patterns match without running a reasoner —
matching how the instantiated queries here are written. Output is **N-Triples**, so load with
sparq format `ntriples`.

### Instantiated constants (pinned to `-pc 300` entities)

| placeholder | value | why |
|---|---|---|
| `%ProductType%` | `bsbm-inst:ProductType2` | 112 products under it — selective but non-empty |
| `%ProductFeature1/2/3%` | `ProductFeature23 / 36 / 26` | co-occur on enough products of that type |
| `%ProductXYZ%` | `…/dataFromProducer4/Product167` | well-connected: 26 features, 43 offers, 24 reviews |
| `%ReviewXYZ%` | `…/dataFromRatingSite1/Review201` | a review of Product167, has a reviewer |
| `%OfferXYZ%` | `…/dataFromVendor1/Offer8` | an offer of Product167, all export properties present |
| `%currentDate%` | `2008-01-01T00:00:00`^^xsd:dateTime | before the corpus's offer-validity window |
| `%x%` / `%y%` | per-query numeric thresholds | chosen from the matching products' numeric ranges |

Instance URIs whose local part contains `/` (Product/Offer/Review) are written as full
angle-bracket IRIs, because `/` is illegal in a SPARQL prefixed-name local part (this is also
how the upstream testdriver substitutes them). One template-faithful quirk at this scale: Q7
and Q10 hardcode vendor countries DE / US, but `-pc 300` generates only 3 vendors (GB/US/CN) —
so Q10 (US) binds while Q7's DE offer-`OPTIONAL` binds nothing; Q7 stays non-empty/stable via
the product label + its review `OPTIONAL`.

## Run it

```sh
cargo build --release -p sparq-cli
bench/bsbm/run.sh                                  # ensure corpus, run Explore mix, diff sizes
# or, manually:
CORPUS=$(bench/bsbm/gen.sh 300)                    # fetch+cache gen, emit fixed corpus
target/release/sparq-cli bench "$CORPUS" ntriples bench/bsbm/queries 3 materialize
```

`materialize` mode is used because the Explore mix includes a CONSTRUCT (`query12`) and a
DESCRIBE (`query09`) — the graph-valued forms report produced-triple counts through the bench
runner. `run.sh` then diffs every query against `expected-rows.tsv` (counts are mode-
independent for this mix, so `count` and `materialize` agree) and **exits 1 on any mismatch or
query error**, so a correctness regression fails the build.

## Tiering — per-commit vs EC2/nightly

- **Per-commit (this dir, `-pc 300`, ~115k triples):** the 11 Explore queries in `queries/`.
  All are sub-millisecond at this scale (the slowest are the similarity self-join Q5 and the
  US-offer Q10), so the whole mix is per-commit-safe.
- **EC2/nightly (full scale):** `bench/bsbm/gen.sh <product_count>` at the BSBM reference scales
  (`-pc ~284,826` ≈ 100M triples, and larger), plus the **Explore-and-Update** and **Business
  Intelligence** mixes and **multi-client concurrency** driven through the serve harness. Query
  `query06` (the unanchored-regex query that the official Explore `querymix.txt` deliberately
  omits) lives in `queries-heavy/` for that tier. Full-scale runs belong to `bench-ec2.yml` /
  nightly with per-query timeouts and result-size assertions vs the published reference numbers.
