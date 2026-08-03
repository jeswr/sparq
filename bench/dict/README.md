<!-- [OPUS-4.8] Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns). -->
# bench/dict — term-index (dictionary) BASELINE harness

Establishes the dictionary **bytes-per-term** and **dict-construction/load
throughput** baseline the term-index-compression plan
(`research/term-index-compression.md` §2–§3) is gated on. That design record
notes "sparq has **no measured dictionary bytes/term or load-throughput baseline
yet** … establishing that baseline is itself the first action" — this harness IS
that action (bead **sq-9w0t**). Every later dictionary-compression %-claim — A1
IRI prefix factoring (sq-xhwf), A2 extended inline ValueIds, A3 lang/datatype
interning, the prototype FSST/PFC tier — must be measured against **our** numbers
here, not borrowed prior-art ratios (the empirical-honesty prerequisite).

Standalone cargo project (own `[workspace]` table, same isolation pattern as
`bench/parse` and `bench/serve`): nothing here touches the root workspace or the
wasm build. Runs under fat LTO to match the shipped `sparq-cli ingest` build, and
measures the dictionary via the production `Graph::load_reader_parallel` path —
so the reported bytes/term is the same `B/term` the CLI load summary prints, and
the throughput is the real ingest cost, not an intern-only micro-benchmark.

## What it measures

The dictionary lives in `crates/sparq-core/src/dict.rs`. The harness reads its
footprint from the public `Dict::heap_bytes()` / `Dict::len()` and walks
`Dict::iter()` once for the per-class composition. It reports:

- **bytes/term**, in BOTH storage modes — `arena` (the native in-RAM `Vec<Stored>`)
  and `blob` (the compacted browser/WASM mode, `into_blob`) — against a `naive
  whole-string/term` denominator (un-deduplicated UTF-8 of every term's
  components). `vs naive` < 100% reflects the single-storage interner +
  prefix-factoring already shipped (A0); where it is *above* 100% on tiny-suffix
  vocab, that is the per-`Stored`-slot overhead A0's "remaining gap" calls out —
  exactly the kind of measured nuance this baseline exists to expose.
- **per-class composition** — IRI / literal / language-tagged / blank / triple-term
  counts and naive UTF-8 bytes, so each lever is sized against the measured arena.
- **lever-sizing levers** — distinct IRI namespace prefixes + the prefix/suffix
  byte split (A1), the dictionary-resident literal count (A2's eviction pool),
  distinct datatype IRIs + language-tag bytes (A3).
- **dict-build throughput** — triples/s, terms/s, MB/s over the full production
  load (median of 3). QUIET-BOX-sensitive — trust it only from an idle run; the
  bytes/term + composition figures are deterministic and load-robust.

## Datasets (`data/`, gitignored)

Two deterministic, fixed-seed (SplitMix64) vocabularies — the two shapes the
design record names — generated in-process (no external dump needed):

- **`wikidata.nt`** — Wikidata-shaped: `wd:Q…` entities, `wdt:P…` properties,
  language-tagged `rdfs:label`s, plain `xsd:string` descriptions, and
  `xsd:integer`/`decimal`/`dateTime` statement values. Exercises A1 (the `wd`/`wdt`
  namespaces), A2 (the numeric/date literals) and A3 (the lang tags + datatype IRIs).
- **`uniprot.nt`** — the most prefix-rich class (the design record's lower-bound
  dataset): long `purl.uniprot.org/uniprot/…` IRIs where the namespace dwarfs the
  local suffix — where A1 has the most to gain.

```sh
cargo build --release
./target/release/dict-baseline gen 200000 data       # -> data/wikidata.nt, data/uniprot.nt
```

For a REAL-data cross-check, point `bench` at a slice of an actual dump (e.g. the
Wikidata truthy `.nt` slice `bench/parse` documents) — the harness loads any
N-Triples file.

## Measurements

```sh
./target/release/dict-baseline bench data/wikidata.nt
./target/release/dict-baseline bench data/uniprot.nt
```

Each prints a markdown block; numbers + analysis are recorded in
`research/term-index-compression.md` §3 (per the catalog's "measure first" rule).

## Selftest (CI-runnable, no dataset)

```sh
./target/release/dict-baseline selftest
```

In-process invariant checks — every term is classified exactly once, `into_blob`
preserves the term count, footprints are positive. Not a performance claim; purely
structural, so it runs deterministically in CI without a dataset or a quiet box.

Because this is a standalone cargo project (own `[workspace]` table), the root
workspace's `cargo clippy --workspace` + nextest lanes never reach it. The CI
`clippy + fmt (gate)` job (`.github/workflows/ci.yml`) therefore clippy-gates this
crate and runs `dict-baseline selftest` directly (`--manifest-path
bench/dict/Cargo.toml`), so a harness regression — a composition miscount, an
`into_blob` that drops terms — fails CI rather than passing unobserved (bead
**sq-hqmm**).
