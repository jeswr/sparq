# Custom parsers — MEASURED BASELINE (gate for the design)

Measured 2026-06-12 on the M1 Air (4P+4E, 16 GB, macOS), `bench/parse/` harness
(standalone cargo project, bench/serve isolation pattern), mimalloc + fat LTO to
match the shipped `sparq-cli ingest` environment. Median of 3 runs, wall clock.
MB/s is over **decompressed input bytes**.

Datasets:

| dataset | bytes | triples | provenance |
|---|---|---|---|
| wikidata-slice.nt | 173,333,785 | 1,500,000 | first 1.5 M lines of the real `latest-truthy.nt.bz2` (2026-06-03 dump) |
| wikidata-slice.ttl | 53,946,257 | 1,500,000 | same triples, oxttl Turtle serializer (wd/wdt/… prefixes, predicate-grouped) |
| synthetic.nt | 169,964,508 | 2,560,000 | deterministic generator (copy of `crates/sparq-bench/src/dataset.rs`) |
| synthetic.ttl | 60,022,305 | 2,560,000 | same generator, prefixed Turtle |

A real ~170 MB slice was used rather than a multi-GB file so each cell is a
median of 3; the per-byte rates are what matter and the slice is real dump bytes
(langtags, datatypes, unicode escapes, 42 blank nodes from "somevalue" snaks).

**Correction to the task premise:** sparq does NOT parse N-Triples with oxttl.
`Graph::parse_to_triples("ntriples")` already uses a custom byte-level parser
(`sparq-core/src/nt.rs`) that interns directly into the Dict with **parallel
newline-split chunking and a sharded dict merge**; oxttl is the NT path only in
the serial streaming `load_reader`. Turtle/TriG/N-Quads are oxttl. So the
baseline below measures oxttl *and* the incumbent custom path.

## N-Triples

| dataset | task | threads | s | MB/s | Mtriples/s |
|---|---|---|---|---|---|
| wikidata | memscan (sum bytes; bandwidth ref) | 1 | 0.003 | 60,718 | — |
| wikidata | oxttl parse-only (discard) | 1 | 0.937 | 185 | 1.60 |
| wikidata | oxttl parse + intern | 1 | 1.174 | 148 | 1.28 |
| wikidata | oxttl → Graph (`load_reader`)* | 1* | 1.260 | 138 | 1.19 |
| wikidata | custom parse + intern (incumbent) | 1 | 0.742 | 234 | 2.02 |
| wikidata | custom → Graph (`load_str`) | 1 | 1.118 | 155 | 1.34 |
| wikidata | custom parse + intern (incumbent) | 8 | 0.193 | 896 | 7.76 |
| wikidata | custom → Graph (`load_str`) | 8 | 0.296 | 585 | 5.06 |
| synthetic | oxttl parse-only (discard) | 1 | 0.956 | 178 | 2.68 |
| synthetic | oxttl parse + intern | 1 | 1.519 | 112 | 1.68 |
| synthetic | custom parse + intern (incumbent) | 1 | 0.871 | 195 | 2.94 |
| synthetic | custom → Graph (`load_str`) | 1 | 1.511 | 112 | 1.69 |
| synthetic | custom parse + intern (incumbent) | 8 | 0.224 | 759 | 11.43 |
| synthetic | custom → Graph (`load_str`) | 8 | 0.418 | 407 | 6.13 |

\* the `load_reader` row's *parse* is serial oxttl but its index build runs on
the global rayon pool (all cores) — its build delta (0.086 s) matches the
8-thread build cost below, not the serial one.

### Where the time goes (wikidata slice)

| stage | serial | 8 threads |
|---|---|---|
| parse scan (custom, est. = parse+intern − intern) | ~0.50 s | — |
| intern/dict (est. from oxttl intern delta: 1.174−0.937) | ~0.24 s | — |
| parse + intern total | 0.742 s (66%) | 0.193 s (65%) |
| index build (`load_str` − parse+intern: sort 6 perms + numerics) | 0.376 s (34%) | 0.103 s (35%) |
| **full ingest** | **1.118 s** | **0.296 s** |

- The incumbent custom serial parse+intern (234 MB/s) is already **1.27× faster
  than oxttl parse-only** (185 MB/s) — i.e. it parses *and* interns in less time
  than oxttl takes to parse and throw the triples away.
- Parallel scaling of parse+intern: 3.84× on 4P+4E (896 vs 234 MB/s).
- Nothing is near memory bandwidth (memscan 60 GB/s; parse is 0.4% of that), but
  the residual cost is hashing/interning and sort, not byte scanning.

## Turtle

| dataset | task | threads | s | MB/s | Mtriples/s |
|---|---|---|---|---|---|
| wikidata | oxttl parse-only | 1 | 0.732 | 74 | 2.05 |
| wikidata | oxttl parse + intern | 1 | 0.980 | 55 | 1.53 |
| wikidata | incumbent chunk-parallel parse+intern | 1 | 0.998 | 54 | 1.50 |
| wikidata | → Graph (`load_str`) | 1 | 1.375 | 39 | 1.09 |
| wikidata | incumbent chunk-parallel parse+intern | **8** | **1.004** | **54** | **1.49** |
| wikidata | → Graph (`load_str`) | 8 | 1.207 | 45 | 1.24 |
| synthetic | oxttl parse-only | 1 | 1.972 | 30 | 1.30 |
| synthetic | oxttl parse + intern | 1 | 3.338 | 18 | 0.77 |
| synthetic | incumbent chunk-parallel parse+intern | 1 | 3.669 | 16 | 0.70 |
| synthetic | → Graph (`load_str`) | 1 | 4.895 | 12 | 0.52 |
| synthetic | incumbent chunk-parallel parse+intern | 8 | 1.264 | 47 | 2.03 |
| synthetic | → Graph (`load_str`) | 8 | 1.343 | 45 | 1.91 |

**Finding (real-data parallel fallback):** `turtle_chunks` bails to serial on
*any* blank node (document-scoped identity). The real Wikidata slice contains
42 `_:` somevalue nodes out of 1.5 M statements → **0% parallel speedup on real
data** (1.004 s @ 8T vs 0.998 s @ 1T), while bnode-free synthetic gets **2.9×**
(3.669 → 1.264 s). One bnode per 35K statements forfeits the whole win.

Per-byte, Turtle parses 2.5–6× slower than N-Triples (74 vs 185 MB/s oxttl
serial), but Turtle files are ~3.2× smaller, so per-*triple* serial Turtle is
roughly NT-competitive (2.05 vs 1.60 Mt/s parse-only on the real slice). The
gap is parallelism: NT full ingest 5.06 Mt/s vs Turtle 1.24 Mt/s @ 8T.

Byte-range splitting is trivially correct for N-Triples (newline-delimited).
It is **NOT** for Turtle: prefixes, multi-line literals, `;`/`,` continuation,
and document-scoped blank-node labels all cross arbitrary byte boundaries. Any
parallel Turtle scheme needs statement-terminator chunking (the incumbent's
approach) plus a story for bnode identity — not naive byte ranges.

## Compressed ingest (gzip / zstd)

Ratios on the real slice: gzip −6 = 11.7×, zstd −3 = 12.6× (synthetic 12.1× /
13.4×). zstd compresses *smaller and decodes 2.8× faster*.

| dataset | task | threads | s | MB/s (decompressed) |
|---|---|---|---|---|
| wikidata | gzip decode-only (flate2 MultiGzDecoder) | 1 | 0.396 | 438 |
| wikidata | zstd decode-only | 1 | 0.140 | 1,236 |
| wikidata | two-stage: decode fully → `load_str` | 8 | 1.124 | 154 |
| wikidata | streaming: decoder → `load_reader_parallel` (= today's `sparq-cli ingest`) | 8 | **5.589** | **31** |
| wikidata | zstd two-stage | 8 | 1.144 | 151 |
| wikidata | zstd streaming | 8 | **3.947** | **44** |
| synthetic | gzip decode-only | 1 | 0.410 | 414 |
| synthetic | zstd decode-only | 1 | 0.142 | 1,199 |
| synthetic | gzip two-stage / streaming | 8 | 1.637 / 4.905 | 104 / 35 |
| synthetic | zstd two-stage / streaming | 8 | 1.515 / 4.167 | 112 / 41 |

**Defect found (the headline number of this baseline):** the streaming path is
**3.5–5× slower than just decompressing to RAM first** — the opposite of what
streaming should cost. Root cause, confirmed by probe (`parse-baseline
probe-read`): `Graph::load_reader_parallel` flushes a parse+sharded-merge round
on **every `read()` call**, and the decompressors return small reads into its
32 MiB buffer — gzip averages 382,635 B/read (453 rounds), zstd 1,635,224 B
(106 rounds). The parallel parser+merge machinery is amortised for ~32 MiB
blocks, not 0.4 MB blocks. Fix is a read-until-block-full loop (a few lines).

Second observation: two-stage itself carries ~0.43 s of pure materialisation
overhead (decode 0.396 + `load_str` 0.296 = 0.69 s measured separately vs
1.124 s combined): allocating + faulting a fresh 173 MB buffer and re-validating
UTF-8. A fused path that hands newline-aligned blocks straight from the
decompressor to the parser avoids this too.

Fusion bound: ideal pipelined ingest = max(decode, parse+build) =
max(0.396, 0.296) ≈ **0.40 s** for gzip, max(0.140, 0.296) ≈ **0.30 s** for
zstd — vs 5.59 / 3.95 s streaming today (**14× / 13×**) and 1.12 / 1.14 s
two-stage (**2.8× / 3.8×**). Most of that is plumbing, not parsing: the
producer-thread pipeline ALREADY EXISTS in the external build path
(`build_external_ntriples_parallel`, sparq-core lib.rs) and just isn't used by
the in-memory `load_reader_parallel`.

Parallel *decompression* (multi-frame zstd / multi-member gzip / lbzip2-style
bzip2) is NOT needed for this: single-thread zstd decode (1,236 MB/s) already
outruns the full 8-thread parse+build (585 MB/s); gzip decode (438 MB/s) is
roughly at parity. It would only matter for bzip2 sources (the actual Wikidata
dump format, ~15–20 MB/s single-stream) — and the prior verdict stands
(research/wikidata-ingestion-benchmark.md): recompress `.bz2` → `.zst` once
rather than building a parallel bzip2 decoder.

## External reference points

- QLever loads DBLP (390 M triples) at **1.7 Mt/s** on a Ryzen 9 7950X
  (16C/128 GB/NVMe); Oxigraph 0.6 Mt/s, Jena 0.2 Mt/s, Virtuoso 0.7 Mt/s on the
  same box ([QLever wiki](https://github.com/ad-freiburg/qlever/wiki/QLever-performance-evaluation-and-comparison-to-other-SPARQL-engines)).
  sparq's full in-memory build here is 5.06 Mt/s on a fanless M1 Air (small
  dict; prior measured 1.28 Mt/s on 50 M real truthy with the out-of-core
  6-permutation build — already QLever-class).
- serd (C, the fastest commonly-cited NT/Turtle parser) publishes only
  qualitative results ("fastest by a wide margin" vs rapper/riot, constant
  memory) — no MB/s in its README ([serd](https://github.com/drobilla/serd)).
- UNCITED ESTIMATE: hand-tuned SIMD line scanners reach 0.5–2 GB/s/core on
  newline-delimited formats (cf. simdjson-class parsers); applied to NT with
  *zero* validation that bounds a custom scan at ~2–4× the incumbent's ~350
  MB/s scan-only rate. It does NOT remove the intern (~0.24 s) or build
  (0.376 s serial / 0.103 s @ 8T) stages.
- Upstream oxttl/oxrdfio meanwhile grew `split_file_for_parallel_parsing` for
  NT/NQ ([oxigraph CHANGELOG](https://github.com/oxigraph/oxigraph/blob/main/CHANGELOG.md)),
  so even "parallel NT via the stock parser" no longer requires custom code.

## Verdicts (headroom per opportunity)

| opportunity | verdict | the number |
|---|---|---|
| (i) custom single-thread NT parser | **REJECT** | The incumbent IS a custom parser, 1.27× faster than oxttl-parse-only (234 vs 185 MB/s). A *zero-cost* scanner caps full serial ingest at 1.118→0.62 s (1.8×); a realistic 2× scanner gives **≤1.2× end-to-end at 8T** (0.296→~0.25 s) because intern+build are 65%+ of the parallel path. Not worth a second parser to differential-test. |
| (ii) parallel NT parsing | **REJECT (already shipped)** | Newline-split + sharded-dict merge exists and scales 3.84× on 4P+4E (896 MB/s parse+intern, 585 MB/s full ingest). Residual headroom is dict-merge scaling on many-core boxes — a known, separate workstream, not a parser. |
| (iii) Turtle | **PURSUE the bnode fallback fix; REJECT a custom Turtle parser** | Real data gets **0%** of the existing 2.9× chunk-parallel win because 42 bnodes in 1.5 M statements trip the serial fallback (1.004 s @ 8T = 0.998 s @ 1T; synthetic shows 3.669→1.264 s). Chunk-scoped label remapping (or collision detection) recovers it. A custom serial parser caps at 1.9× (oxttl parse is 53% of serial ingest) — poor ROI for the grammar surface. |
| (iv) fused unzip+parse | **PURSUE (two cheap steps, biggest measured win)** | Step 1: fix `load_reader_parallel`'s per-`read()` flush (parse blocks are 0.4–1.6 MB, probed, instead of 32 MiB) — streaming is currently **3.5–5× slower than two-stage** (gz 5.59 vs 1.12 s). Step 2: reuse the existing producer-thread pipeline from the external build → ideal max(decode, parse) ≈ 0.30–0.40 s vs 1.12–1.14 s two-stage (**~3×**, ~13× vs today's streaming). No new decompressor needed: zstd decode (1,236 MB/s) already outruns 8T parse+build (585 MB/s). |
| zipped serialization / zstd-vs-gzip choice | (later deliverable) | Decode side measured here: zstd −3 beats gzip −6 on ratio (12.6× vs 11.7×) AND decode speed (2.8×). Supports the zstd direction in research/custom-parsers-ADDENDUM-zstd-js-clients.md; dictionary + multi-frame benchmarks belong to the serving-wave deliverable. |

**Honest bottom line:** there is no custom-parser goldmine here. sparq's NT
path is already a custom parallel parser that beats oxttl, and oxttl is itself
fast (185 MB/s serial — ~10× Jena-class parsers). The measured headroom is in
*plumbing*: the compressed-ingest streaming path wastes 3.5–5× to a block-fill
bug and another ~3× to missing pipelining (both fixes reuse existing code), and
parallel Turtle silently turns itself off on any real-world file containing a
blank node. Fix those before writing any new parser; whether `sparq-parse`
should exist at all (vs patches to sparq-core) should be revisited after the
fusion fix lands — the crate currently stays an empty scaffold.

Reproduce: `bench/parse/README.md`.

## Post-fix measurements (2026-06-12, streaming-ingest fix)

`Graph::load_reader_parallel` rewritten (sparq-core) to the external build's
3-stage pipeline: a producer thread **fills the full 32 MiB block across
`read()` calls** (was: one parse+merge round per `read()`, i.e. per 0.38–1.6 MB
decompressor return) and runs the decode **concurrently** with the rayon parse
and the dict merge. Same machine/harness; numbers are medians of 3 harness runs
(each itself a median-of-3), with the run-to-run range shown — this session ran
warmer/noisier than the baseline session (today's 8T `load_str` reference:
0.435 s vs 0.296 s baseline; serial parse replicates within 5%, decode-only
measured faster at 0.19–0.36 s gz / 0.07–0.12 s zst).

| task (wikidata slice, 8T) | baseline | post-fix (median) | range | speedup |
|---|---|---|---|---|
| gzip streaming → `load_reader_parallel` | 5.589 s | **0.661 s** | 0.585–0.692 | **8.5×** |
| zstd streaming → `load_reader_parallel` | 3.947 s | **0.576 s** | 0.541–0.739 | **6.9×** |
| gzip two-stage decode → `load_str` (same runs) | 1.124 s | 0.741 s | 0.646–1.079 | — |
| zstd two-stage decode → `load_str` (same runs) | 1.144 s | 0.576 s | 0.568–0.776 | — |

synthetic.nt (one run): gz streaming 0.699 s vs two-stage 0.823 s; zst 0.706 vs
0.697 s (baseline streaming: 4.905 / 4.167 s).

- **Streaming now matches or beats two-stage in every paired run** (gz 1.12×
  faster on medians; zst parity) instead of being 3.5–5× slower — and it never
  materialises the 173 MB decompressed copy.
- **vs the ideal:** ideal = max(decode, parse+build) measured *today* is
  ~0.44 s (parse+build 0.435 s dominates both codecs). Streaming lands at
  0.58–0.66 s, i.e. ~1.3–1.5× over ideal, not at it. The residual gap is the
  parts that cannot overlap the decode: the final `finish_sharded` remap + the
  6-permutation sort/build after the last block, plus per-block carry copies
  and 6 merge rounds vs `load_str`'s single round. The decode itself is fully
  hidden (streaming 0.585 s < decode 0.19 + load_str 0.435 = 0.625 s sequential
  sum on the same run). The baseline's 0.30–0.40 s ideal assumed the cooler
  session's 0.296 s parse+build; the *ratio* conclusion stands, the absolute
  target was not reached in this session.
- A later, cooler run (after the block-size-parameterisation refactor, same
  binary semantics) measured gz streaming **0.429 s** / zst **0.359 s** vs
  two-stage 0.514 / 0.413 s — i.e. streaming beat two-stage by 1.15–1.20× and
  zstd streaming landed inside the baseline's 0.30–0.40 s ideal band. Thermal
  state moves every number on this fanless machine; the paired
  streaming-vs-two-stage ordering was stable across all 4 runs.
- Regression cover: `load_reader_parallel_short_reads_match_sequential`
  (sparq-core) pins short reads, mid-line read boundaries, EOF without trailing
  newline, empty input, and parse-error propagation through the pipeline.
  `cargo test -p sparq-core --release`: 27 passed. Wasm guard:
  `sparq_wasm.wasm` 1,573,895 B (unchanged — the parallel path isn't compiled
  for wasm).
- API note: `load_reader_parallel` now requires `R: Read + Send` (the producer
  thread); `sparq-cli`'s `open_reader` returns `Box<dyn Read + Send>`.

## Post-fix measurements (2026-06-12, Turtle blank-node parallel fix)

`turtle_chunks` (sparq-core) no longer bails to serial on blank nodes. The bail
was unnecessary, not merely expensive:

- **Labeled `_:x`** — oxttl preserves the written label verbatim
  (`BlankNode::new_unchecked(label)`) and the per-chunk partial dicts merge
  into the global `Dict` by term equality (`merge_remap` → `intern_blank`), so
  cross-chunk occurrences of a label unify to ONE id exactly as the serial
  document-scoped parse does. The dict merge *is* the shared label-intern map —
  option (c) of the design sketch with zero pre-scan, no label-count threshold,
  and no per-chunk namespacing or post-pass.
- **Anonymous `[...]`/`(...)`** — confined to a single statement (hence one
  chunk); oxttl mints them as random 128-bit ids (`BlankNode::default()`,
  thread-safe), so cross-chunk distinctness carries the same probabilistic
  guarantee the serial parser already relies on *within* one document.
  Parallelism adds no new collision class.
- The terminator scanner needs no bracket tracking: in valid Turtle a `.`
  followed by whitespace/EOF/`#` cannot occur inside `[...]`/`(...)`/labels
  outside the strings/IRIs/comments it already skips (DECIMAL/DOUBLE require a
  digit/exponent after the dot; PN_LOCAL/BLANK_NODE_LABEL cannot end in a dot).
  Mis-splits on *invalid* input fail chunk parsing → serial fallback, as before.
- Bonus guard: a 1-thread rayon pool now parses serially outright (chunking
  there was pure overhead — measured 1.300 s chunked vs 1.028 s serial at 1T).

Correctness: differential tests (`parallel_turtle_bnodes_match_serial`) compare
chunked vs serial after first-occurrence bnode renumbering — an *exact*
equality check that subsumes isomorphism because document order is preserved
and the label mapping is 1:1 — over shared labels in distant statements
crossing every chunk boundary, anonymous nests + collections, an all-bnode
chained doc, sparse Wikidata-shaped labels, and a bnode-free control. On the
real slice itself: 8T-chunked vs plain-serial-oxttl triple sets are
**byte-identical** (1,500,000 statements, 1,499,123 distinct triples, 41
distinct `_:` labels — all labeled, so no renaming is even needed).

Numbers (same machine/harness; the final run's thermal state matches the
baseline session — oxttl parse-only 0.778 s vs baseline 0.732 s, 1T 1.028 s vs
baseline 0.998 s — an earlier warmer run this session measured everything ~2×
slower, ratios unchanged):

| wikidata-slice.ttl | 1T | 8T | speedup |
|---|---|---|---|
| parse+intern BEFORE (baseline) | 0.998 s | **1.004 s** | **1.00×** |
| parse+intern AFTER | 1.028 s | **0.479 s** | **2.15×** |
| `load_str` BEFORE (baseline) | 1.375 s | 1.207 s | 1.14× |
| `load_str` AFTER | 1.443 s | 0.613 s | 2.35× |

- Real data now scales the same as bnode-free synthetic measured in the same
  run (2.15× vs 2.21×; synthetic 1.926 → 0.870 s) — the bnode penalty is gone
  entirely, not just reduced.
- The baseline's headline "2.9×" synthetic ratio did not reproduce in this
  session for synthetic either (2.21× today; the fanless M1 Air's thermal
  state moves absolute numbers and ratios run to run). The honest claim is
  parity with the bnode-free path, which is the most this fix could deliver:
  wikidata Turtle 8T parse+intern 1.004 → 0.479 s (2.1×), full `load_str`
  1.207 → 0.613 s (2.0×).
- `cargo test -p sparq-core --release`: 28 passed (27 + the new differential).
  Wasm guard: `sparq_wasm.wasm` 1,573,895 B (byte-identical — the parallel
  path isn't compiled for wasm).
