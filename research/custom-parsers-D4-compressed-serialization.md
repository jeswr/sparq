# D4 — compressed (zipped) serialization of query results: MEASURED

Measured 2026-06-12 on the M1 Air (4P+4E, 16 GB, macOS), `bench/parse/`
harness (`compress-bench` bin, same isolation/mimalloc/fat-LTO pattern as the
baseline), real dataset = the 1.5 M-triple Wikidata slice from
`research/custom-parsers-baseline.md`. Median of 3 runs unless noted; this
fanless machine's thermal state moves absolute numbers session to session
(an earlier warmer run measured serialize-alone 0.141 s vs 0.098 s here —
ratios and orderings were identical).

Implementation: `crates/sparq-parse` (its first real module) —
`CompressedSink` (multi-member gzip / multi-frame zstd, serial or
rayon-parallel, members emitted in push order), `SingleMemberGzipSink`
(browser-safe parallel gzip, see §3 — the headline finding), zstd dictionary
train/compress/decode (`train_dictionary`, `PreparedDict`,
`Codec::ZstdDict`). Tests pin: concat-of-members decodes byte-identical to
concat-of-chunks (both codecs, both modes), parallel members byte-identical
to serial, dictionary frames decode only with the dictionary, single-member
gzip decodes with a strict single-member decoder. `cargo test -p sparq-parse
--release`: 10 + 1 doctest green.

The format property everything rests on: a gzip stream MAY contain multiple
members (RFC 1952 §2.2) and a zstd stream MAY contain multiple frames
(RFC 8878 §3) — so independently compressed chunks, concatenated in order,
are a valid stream. Whether *consumers* honour that is §3.

## 1. Large result (302.5 MB SPARQL-JSON, 33 chunks)

`SELECT ?s ?p ?o` over the slice via `query_json_chunks_with_budget` →
302,516,637 B in 33 chunks (the engine's parallel path emits one chunk per
worker fragment, ~9.2 MB each, plus a 54 B head — these are exactly the
chunks the server streams). Serialize-alone: **0.098 s** (3,099 MB/s, 8T).

| codec | comp bytes | ratio | serial s (MB/s) | parallel 8T s (MB/s) | par speedup | single-stream ratio |
|---|---|---|---|---|---|---|
| gzip -1 | 22,649,329 | 13.36x | 0.387 (781) | 0.079 (3,833) | 4.91x | 13.36x |
| gzip -6 | 17,376,304 | 17.41x | 1.661 (182) | 0.403 (750) | 4.12x | 17.42x |
| zstd -1 | 16,999,774 | 17.80x | 0.191 (1,583) | 0.037 (8,130) | 5.13x | 17.83x |
| zstd -3 | 16,347,914 | 18.50x | 0.195 (1,555) | 0.046 (6,537) | 4.20x | 18.62x |
| gzip -1 1-member (pigz) | 22,652,279 | 13.35x | 0.378 (800) | 0.092 (3,289) | 4.11x | — |
| gzip -6 1-member (pigz) | 17,375,872 | 17.41x | 1.642 (184) | 0.373 (811) | 4.40x | — |

- Chunked framing costs essentially **zero ratio** at these chunk sizes
  (18.50x vs 18.62x single-frame zstd -3; gzip identical to 4 digits) — the
  9 MB chunks dwarf the codecs' windows. (At the engine's nominal 64 KiB
  flush this would cost more; not the production shape today.)
- zstd -3 strictly dominates gzip -6: better ratio (18.5x vs 17.4x), 8.5x
  faster serial, 8.7x faster parallel. Consistent with the baseline's
  decode-side finding.

### End-to-end (the user's "less overhead" claim) — verdict with numbers

Batch shape (today's API: serialize fully, then compress):

| codec | serialize-alone s | + serial compress s | + parallel 8T s |
|---|---|---|---|
| gzip -1 | 0.098 | 0.485 | 0.181 |
| gzip -6 | 0.098 | 1.744 | 0.461 |
| zstd -1 | 0.098 | 0.309 | 0.128 |
| zstd -3 | 0.098 | 0.322 | 0.141 |

Streaming shape (paced replay: chunk i is pushed at (i+1)/n·t_ser, i.e. at
the measured production rate, into the parallel sink; best of 3):

| codec | wall s | tail past serialize | first compressed member at | first member bytes |
|---|---|---|---|---|
| gzip -1 | 0.115 | +0.017 | 0.007 s | 77 |
| gzip -6 | 0.383 | +0.285 | 0.007 s | 77 |
| zstd -1 | 0.106 | +0.008 | 0.007 s | 63 |
| zstd -3 | 0.106 | +0.008 | 0.007 s | 63 |
| gzip -1 1-member (pigz) | 0.113 | +0.015 | 0.007 s | 74 |
| gzip -6 1-member (pigz) | 0.374 | +0.276 | 0.007 s | 74 |

**Verdict: TRUE for zstd and gzip -1, FALSE for gzip -6.**

- **zstd -1/-3**: parallel compression overlapping production adds **+8 ms to
  a 98 ms serialization (+8%)** and turns 302 MB into 16.3 MB. Even as a pure
  batch tail it's +43 ms (+44%). The mechanism: 8T zstd -3 compresses at
  6.5 GB/s — 2.1x faster than the serializer produces (3.1 GB/s), so
  compression hides entirely behind production with cores to spare.
- **gzip -6 cannot keep up at any thread count measured**: 750–811 MB/s at 8T
  vs 3,099 MB/s production. Tail = +0.28 s ≈ 3.9x serialize-alone. If a
  browser-gzip body must be cheap, serve gzip -1 (+15–17 ms, 13.4x) and let
  zstd carry the clients that accept it.
- **TTFB proxy**: first compressed member is on the wire at **7 ms** (one
  64 B head member) vs 98 ms for the full uncompressed serialization — and a
  client on a real link starts receiving 18x fewer bytes. Both numbers favour
  compressed streaming strongly.
- Honesty caveats: (i) the replay *simulates* a streaming producer —
  `query_json_chunks_with_budget` returns `Vec<String>`, so true overlap
  needs the Wave D streaming server to push chunks as produced (the sink API
  is shaped for exactly that: `push`/`try_drain`/`finish`); (ii) during the
  replay the serializer is idle, so compression had all 8 cores — when both
  run hot the total CPU-seconds still add, and only zstd's >2x throughput
  margin over the serializer makes the wall-clock overlap near-free; (iii)
  33 chunks bound parallelism at 33 tasks — speedups (4.1–5.1x on 4P+4E)
  match the house parse-scaling numbers, not ideal 8x.

## 2. Small responses: the dictionary benchmark (the addendum's table)

2,400 point queries (`SELECT ?p ?o WHERE { <s> ?p ?o }`, one per distinct
subject) → first 1,200 train a zstd dictionary (`zstd::dict::from_samples`),
last 1,200 are held-out eval (total 27.9 MB; median 634 B — but heavy-tailed:
589 responses >4 KiB carry 98.6% of the bytes, so the bucket table is the
real result). Compression is per-response one-shot (one frame per HTTP
response, prepared dictionary digested once); decode via `zstd::bulk` /
`MultiGzDecoder`.

| codec | comp bytes | ratio | avg comp B/resp | comp MB/s | dec MB/s |
|---|---|---|---|---|---|
| gzip -1 | 3,101,821 | 9.01x | 2,585 | 489 | 586 |
| gzip -6 | 2,468,365 | 11.32x | 2,057 | 180 | 686 |
| zstd -1 | 2,703,228 | 10.34x | 2,253 | 918 | 2,517 |
| zstd -2 | 2,688,720 | 10.39x | 2,241 | 848 | 2,014 |
| zstd -3 | 2,680,839 | 10.42x | 2,234 | 702 | 2,139 |
| zstd -1 +dict(64K) | 2,293,406 | 12.18x | 1,911 | 821 | 2,186 |
| zstd -2 +dict(64K) | 2,276,861 | 12.27x | 1,897 | 767 | 2,223 |
| zstd -3 +dict(64K) | 2,256,533 | 12.38x | 1,880 | 609 | 2,050 |
| zstd -3 +dict(16K) | 2,344,773 | 11.92x | 1,954 | 638 | 2,283 |

Size-bucketed (ratio, avg compressed bytes/response):

| bucket | n | gzip -6 | zstd -3 | zstd -3 +dict(64K) | zstd -3 +dict(16K) |
|---|---|---|---|---|---|
| ≤1 KiB | 606 | 2.29x (277 B) | 2.28x (278 B) | **11.37x (56 B)** | **11.67x (54 B)** |
| 1–4 KiB | 5 | 6.36x (536 B) | 5.77x (591 B) | 10.33x (330 B) | 10.58x (322 B) |
| >4 KiB | 589 | 11.98x (3,901 B) | 10.98x (4,260 B) | 12.40x (3,771 B) | 11.92x (3,922 B) |

**This is the number that justifies the dictionary path: on genuinely small
responses (≤1 KiB, the case dictionaries exist for) the vocabulary dictionary
takes zstd from 2.3x to 11.4x — 278 B → 56 B per response, 5.0x smaller** —
at zstd -1..-3 speeds (3.4–4.5x faster compression than gzip -6, which only
manages 2.3x ratio there). A 16 KiB dictionary is as good as 64 KiB on the
small bucket (slightly better, 54 B — less dilution) and only loses on the
big tail where the dictionary barely matters anyway. Levels 1–3 are within
1% of each other with the dict: use level 1 (or 2) and bank the CPU.
Dictionary training wall was not measured (offline/async concern, not on the
request path).

## 3. Decodability: measured consumer matrix (the headline finding)

In-process roundtrips are asserted in the crate's tests. External reference
decoders and real clients, on the 302 MB artifacts (`compress-bench
artifacts`; browsers driven via Playwright against a local server setting
real `Content-Encoding` headers):

| consumer | multi-member gzip | 1-member pigz gzip | multi-frame zstd | zstd +dict |
|---|---|---|---|---|
| gzip CLI (`gzip -dc`) | **OK full** | **OK full** | — | — |
| Python 3 `gzip` stdlib | **OK full** (member count 33/1 confirmed via zlib) | **OK full** | — | — |
| zstd CLI 1.5.7 | — | — | **OK full** (`zstd -l`: 33 frames) | **OK full** with `-D`; refuses without (exit 1) |
| flate2 `MultiGzDecoder` / `GzDecoder` | OK / first member only | OK / **OK full** | — | — |
| zstd crate stream decoder | — | — | **OK** (multi-frame default) | **OK** (`with_dictionary`, dict spans frames) |
| Node 25.1 `zlib.gunzipSync`, `DecompressionStream('gzip')` | **OK full** | OK (single member is the trivial case) | — | — |
| Node 25.1 `zlib.zstdDecompressSync` / `createZstdDecompress` | — | — | **FIRST FRAME ONLY** (54 B of 302 MB, silent) | — |
| Node 25.1 `fetch` (undici) | C-E gzip **OK full** | OK | C-E zstd **FIRST FRAME ONLY** (silent) | — |
| curl 8.7.1 `--compressed` | **FIRST MEMBER ONLY** (54 B, exit 0, silent) | **OK full** | (no zstd in Apple's build) | — |
| Chromium 148 `fetch` | **FIRST MEMBER ONLY** (54 B, silent) | **OK full** | **OK full** | n/a (no native dict) |
| Firefox 150 `fetch` | **FIRST MEMBER ONLY** (54 B, silent) | **OK full** | **OK full** | n/a |
| WebKit 26.4 `fetch` | **FIRST MEMBER ONLY** (54 B, silent) | **OK full** | raw passthrough (no zstd support) | n/a |
| `DecompressionStream('gzip')` in all 3 engines | **ERROR at member 2** ("Extra bytes past the end" / abort) | **OK full** | ('zstd' not in the Compression spec) | — |
| fzstd 0.1 (pure-JS, what a Solid app would bundle) | — | — | **OK full, 410 MB/s decode on M1** | **not supported** (no dict API — use zstd-wasm) |

**The task premise inverted for gzip: browsers do NOT decode multi-member
`Content-Encoding: gzip` — all three engines silently truncate to the first
member** (RFC 1952-valid input, 0.018% of the body delivered, HTTP 200, no
error — the worst failure mode available). curl does the same; Node's gzip
does not (it loops members), and the browsers' own zstd does the opposite
(multi-frame decodes fully in Chromium/Firefox). Node's *zstd* (25.1) is
first-frame-only — the truncation trap exists somewhere on every format, so
the bindings wave must add decode-equality tests against every client it
ships.

Consequences, folded back into the deliverable:

- **Multi-frame zstd is the parallel wire format** — valid for Chromium- and
  Firefox-native `Content-Encoding: zstd`, the zstd CLI, the Rust crate, and
  JS decoders (fzstd measured). WebKit/Safari never advertises zstd in
  `Accept-Encoding`, so correct negotiation excludes it there naturally.
- **For gzip consumers, `SingleMemberGzipSink` replaces multi-member**: pigz's
  technique — each chunk is an independent raw-deflate segment ended by
  `Z_FULL_FLUSH` (byte-aligned, dictionary reset, parallelizable), one gzip
  header, one trailer with `flate2::Crc::combine`d CRC32. Measured identical
  ratio (17.41x) and parallel wall (0.37 vs 0.40 s) to multi-member, and it
  decodes **fully in all three browser engines, curl, gzip CLI, Python, and
  `DecompressionStream`** (validated on the 302 MB artifact). `Codec::Gzip`
  multi-member remains for CLI/Python/Rust/Node-zlib consumers and decode
  tooling, but MUST NOT be served as browser-facing `Content-Encoding: gzip`.
- Client decode matrix for the bindings wave: browser-native zstd covers
  Chromium/Firefox; Safari needs either pigz-gzip (native, free) or a bundled
  JS/WASM zstd. fzstd decodes our streams at 410 MB/s but has **no dictionary
  support** — dictionary-compressed small responses need zstd-wasm (or
  equivalent) in the client library; decode-side dict cost measured native at
  2.0–2.5 GB/s (bulk, pre-digested).

## 4. Dictionary-fetch protocol (design only — serving wave implements)

Refines the addendum's sketch against the merged sparq-serve model
(`Generation { number, snapshot, epochs: PodEpochs }`, monotonic per-pod
epochs, bounded retention ring).

- **Identity.** A dictionary is immutable, content-addressed:
  `dict-id = base64url(truncated SHA-256 of the dictionary bytes)`. zstd
  dictionaries also embed a 4-byte zstd `dictID`, and every frame compressed
  with one references it in its frame header — so a client can detect *that*
  a frame needs a dictionary, and *which one*, before asking. The server maps
  zstd dictID ↔ dict-id.
- **Endpoint.** `GET /dictionary/{dict-id}` → raw dictionary bytes,
  `Cache-Control: public, max-age=31536000, immutable`, `ETag: "{dict-id}"`.
  Content-addressing makes infinite caching safe (CDN-friendly; a Solid app
  fetches each dictionary once per browser profile).
- **Negotiation.** Dictionary compression is only applied when the client
  proves it already holds the dictionary: request header
  `Sparq-Dictionary: {dict-id}` (multi-valued). Response carries
  `Sparq-Dictionary: {dict-id}` echoing the one actually used (absent = plain
  frames), plus `Sparq-Dictionary-Current: {dict-id}` advertising the newest
  one so clients warm up for the *next* request. First request is therefore
  plain zstd — no mid-response fetch dependency, no extra RTT ever. (This is
  a deliberate API-client simplification of RFC 9842 compression dictionary
  transport — `Available-Dictionary`/`Use-As-Dictionary` with `dcz` — which
  browsers apply to *navigations*; sparq clients are programmatic, so the
  custom header pair avoids RFC 9842's URL-matching machinery. If
  browser-native dictionary transport matures for `fetch`, the mapping is
  mechanical.)
- **Generation/epoch coupling.** A dictionary is a *compression aid, not a
  correctness artifact*: a frame decodes with the dictionary it names
  forever, regardless of how many generations have passed — so dictionaries
  need NO invalidation on publish, unlike Wave B's result cache (which keys
  on `PodEpochs`). Vocabulary drift only degrades *ratio*. Policy: train per
  pod (pods shard the vocabulary), keyed `(pod-id, training-epoch)`;
  retrain asynchronously when the pod's epoch has advanced by a configured
  delta AND the observed dict ratio on small responses regresses past a
  threshold (e.g. 20% worse than at training time); keep the last K
  dictionaries fetchable (mirror of the ring's retention bound) so in-flight
  clients never 404, and expire older ones lazily. The training corpus is
  exactly what `compress-bench small` samples: recent small-response bodies.
- **Scope guard.** Dictionaries apply to responses below a size threshold
  (the ≤1 KiB bucket is the 5x win; >4 KiB gains 13%) — suggested: use the
  dictionary for bodies ≤ 16 KiB, plain multi-frame zstd above; one frame per
  response either way.
- Sizing from §2: default 16 KiB cap (best on the small bucket, cheapest to
  fetch), 64 KiB if mixed traffic skews larger.

## 5. Scope notes

- HTTP wiring (`Accept-Encoding` negotiation, streaming the drained members)
  is Wave D's; the sink API (`push`/`try_drain`/`finish`) is shaped for its
  chunk loop. JS decoder integration is the bindings wave's (matrix in §3).
- RDF serializers: results-JSON only here. Turtle/N-Triples serialization is
  line/statement-oriented and would feed the same sinks unchanged (chunk =
  statement run) — no codec work needed, only a chunked serializer entry
  point; noted as future work, no easy win left on the codec side.
- Wasm guard: `sparq_wasm.wasm` 1,573,895 B (byte-identical to the worktree
  baseline); `cargo tree -p sparq-wasm -e normal` crate set contains no
  sparq-parse/zstd/flate2/rayon. (A naive `grep -cE "parse|zstd|flate2"` over
  the tree output reports 5 — all false positives from this *worktree's
  directory name* `sparq-parse` appearing in path suffixes; the crate-name
  check is the real guard and reports 0.)

Reproduce: `bench/parse/README.md` (`compress-bench big|small|artifacts`),
browser matrix via Playwright (chromium 148 / firefox 150 / webkit 26.4)
against a local server setting real `Content-Encoding` headers.
