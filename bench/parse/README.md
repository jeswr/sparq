# bench/parse — custom-parsers BASELINE harness

Measures the incumbent ingestion stack (oxttl, sparq-core's byte-level N-Triples
parser, flate2/zstd streaming decompression) so the custom-parsers design is
gated on numbers, not vibes. Results + analysis: `research/custom-parsers-baseline.md`.

Standalone cargo project (own `[workspace]` table, same isolation pattern as
`bench/serve`): the codec deps never touch the root workspace or the wasm build.
Runs under mimalloc + fat LTO to match the shipped `sparq-cli ingest` environment.

## Datasets (`data/`, gitignored)

```sh
cargo build --release

# Real data: first 1.5 M lines (~173 MB) of the Wikidata truthy dump.
bzcat /path/to/truthy.nt.bz2 | head -n 1500000 > data/wikidata-slice.nt

# Synthetic: deterministic generator (copy of crates/sparq-bench/src/dataset.rs),
# ~162 MB N-Triples + ~57 MB prefixed Turtle.
./target/release/parse-baseline gen 320000 data/synthetic.nt data/synthetic.ttl

# Turtle version of the real slice (oxttl serializer, wd/wdt/... prefixes).
./target/release/parse-baseline to-ttl data/wikidata-slice.nt data/wikidata-slice.ttl

# Compressed variants (flate2 -6 gzip, zstd level 3).
./target/release/parse-baseline compress data/wikidata-slice.nt
./target/release/parse-baseline compress data/synthetic.nt
```

## Measurements

```sh
./target/release/parse-baseline bench-nt  data/wikidata-slice.nt   # + synthetic.nt
./target/release/parse-baseline bench-ttl data/wikidata-slice.ttl  # + synthetic.ttl
./target/release/parse-baseline bench-zip data/wikidata-slice.nt   # + synthetic.nt
```

- `bench-nt`: memscan ceiling; oxttl parse-only / parse+intern / full Graph;
  incumbent custom NT parser (serial + all-cores) parse+intern and full Graph.
- `bench-ttl`: same for Turtle (oxttl serial; incumbent chunk-parallel path).
- `bench-zip`: gzip/zstd decode-only (bounds any fused unzip+parse win);
  two-stage (decompress fully, then parse) vs streaming decompress-into-parser
  (= today's `sparq-cli ingest` path).

Each number is the median of 3 runs, wall clock. MB/s is over the
*decompressed* input bytes. Run on an otherwise idle machine.
