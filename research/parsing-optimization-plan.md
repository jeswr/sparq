<!-- [OPUS-4.8] parsing-opt plan (HDT decompress/parse + Turtle->NT), Opus 4.8 (Fable unavailable). -->

# Parsing / ingest optimization plan — HDT decode+parse and Turtle→N-Triples parity

Scope: task #13 (HDT decompress+parse, incl. compressed HDT) and task #14 (Turtle parse → match N-Triples throughput). All file refs are to `/home/ubuntu/sparq`. Grounded in `research/custom-parsers-baseline.md`, `research/fast-ingestion.md`, `research/wikidata-ingestion-benchmark.md`, `crates/sparq-hdt/README.md` (plus the crate's then-current `TODO.md`, deleted in the 2026-06-13 hygiene cleanup — its deferred-item rationale now lives in beads and `crates/sparq-hdt/UPSTREAM.md`). Empirical-honesty rule applies: no claimed win without a before/after measurement on a quiet box.

## 0. Goals (and the honest framing of each)

- **(a) HDT decode+parse optimal, incl. compressed HDT.** "Optimal" = a single sequential pass: each distinct dictionary term PFC-decoded exactly once into the sparq `Dict`, each triple emitted once in SPO order, **zero construction of query-only indexes** (wavelet matrix, OP-index, rank/select). Lower bound ≈ `read + CRC + O(dict bytes) + O(triples)` + sparq's normal permutation build. Today HDT loads at 416 k triples/s — ~12× *below* the NT custom path's 5.06 Mt/s — so this is the weakest baseline and has real headroom.
- **(b) Turtle parse == N-Triples throughput.** HONEST UP FRONT: **exact NT parity is infeasible and the reason is intrinsic, not an engineering gap.** Turtle must (i) expand prefixed names (`wd:Q42`) into full IRIs — string concatenation NT never does — and (ii) the format's value *is* its smaller byte count, trading bytes-on-disk for parse-time work. Per-*triple* serial Turtle is already roughly NT-competitive (2.05 vs 1.60 Mt/s parse-only). The realistic target is to **close the parallel-scaling and per-chunk-allocation gap on the common predicate-grouped file**, not byte-for-byte NT parity. The bnode cliff that previously gave 0× scaling on real data is already fixed (now ~2.0–2.35× at 8T).

---

## 1. HDT workstream

`sparq-hdt` is a lean (~300 LOC) wrapper (`crates/sparq-hdt/src/lib.rs`); all real decode happens in vendored upstream `hdt 0.4.0`. The wrapper is correct and already good — the cost is in *how it drives upstream* and the query-only structures upstream builds during a load that sparq uses for a one-shot ordered scan.

### Ranked optimization steps

**H1 — Sparq-side SPO decoder: skip upstream's `TriplesBitmap::new` (wavelet + OP-index + rank/select). BIGGEST WIN.**
- Lever: upstream `TriplesBitmap::new` (`hdt-0.4.0/src/triples.rs:263-305`) + `build_wavelet` build a full `WaveletMatrix`, a `Vec<Vec<u32>>` mega-allocation (one Vec per object, `vec![Vec::with_capacity(4); max_object]`), a per-object `sort_by_cached_key` over cache-hostile wavelet `access`, and an OP-index `CompactVector`+`Rank9Sel`. **sparq iterates triples exactly once in SPO order and never does a pattern/object/predicate query, so all of this is built and immediately thrown away.** Reimplement just the triples-section read in sparq-hdt: read `bitmap_y/z` + `sequence_y/z` (containers are small/self-contained: `bitmap.rs` 256 LOC, `sequence.rs` 310 LOC) and iterate SPO directly, stopping before `new`.
- Expected impact: large — eliminates the single biggest upstream load cost. Folds in H2.
- Risk: medium-high. Must replicate HDT's on-disk quirks (3-vbyte PFC preamble `dict_sect_pfc.rs:242-243`; deliberate vbyte off-by-one `vbyte.rs:26-30`; website-format deviations, not the outdated 2011 W3C submission). Note `hdt 0.6` adds *more* query structure (a `qwt` wavelet) — so the gap widens with upgrades, reinforcing not pinning our scan to upstream's query structures.
- Gate: per-stage timing (`Hdt::read` decode vs `graph_from_hdt` id-translation vs `Graph::build`) before/after, plus the differential oracle (§oracle) must produce an identical `BTreeSet<[String;3]>` to `graph_from_hdt(&Hdt::read(...))` on the same bytes.

**H2 — Iterate predicates from `sequence_y`, not the wavelet (folds into H1).**
- Lever: `SubjectIter::next` (`subject_iter.rs:117`) recovers each predicate via `wavelet_y.access` (O(log σ) bit-rank ops) when it is just `sequence_y.get(pos_y)` (one shift/mask, `sequence.rs:105`). The iterator only goes through the wavelet because that's what `new` kept.
- Impact: medium-large per-triple; removes wavelet construction entirely (it's the natural shape once H1 reads the sequences directly).
- Risk: low if done as part of H1. Gate: same differential oracle; per-triple ns in the new bench.

**H3 — PFC block-sequential dictionary decode (cache the block; sparq-side, unblocked).**
- Lever: each upstream `extract(id)` (`dict_sect_pfc.rs:183-211`) re-walks the PFC block from its start (~block_size/2 ≈ 8 redundant vbyte-delta steps) and allocs a fresh `Vec<u8>`+`String` per id. sparq materializes **every** id in each section exactly once (memo tables `lib.rs:168-211`), so walk each section block-by-block, decoding the whole block once into a reusable buffer — front-coding is inherently sequential. Turns O(N·block_size) into O(N), one buffer reuse instead of N allocs. `num_strings`/`block_size`/`sequence`/`packed_data` are all `pub` on `DictSectPFC`.
- Impact: medium — dominant dictionary cost today. Risk: medium (must match `extract` across block boundaries — add a multi-block PFC fixture). Gate: dict-decode stage timing + differential equality across block boundaries.

**H4 — Intern from borrowed slices, drop the `String` round-trip (sparq-side, builds on H3).**
- Lever: today upstream `extract`→`String`→`intern_hdt_term` re-parses→`dict.intern_*` copies. If H3 decodes PFC bytes itself, parse the term shape on the `&str`/`&[u8]` slice and call `intern_iri`/`intern_lit`/`intern_blank` with borrowed slices — sparq's interners already take `&str` and copy into `Box<str>` once (`dict.rs:801-846`). Saves one `String` alloc + one copy per distinct term.
- Impact: medium. Risk: low-medium (literal grammar; reuse the well-tested `intern_hdt_term` logic, `lib.rs:232-267`). Gate: differential equality + alloc count.

**H5 — Compressed-HDT codecs + faster gzip (sparq-side, easy).**
- Lever: `.hdt.gz` is already content-sniffed (magic `0x1f 0x8b`, not filename), streamed and fused via `MultiGzDecoder` — correct, do not redesign. Add: (i) `flate2` `zlib-ng` backend (drop-in, 1.5–3× inflate); (ii) zstd sniffing (magic `0x28 0xB5 0x2F 0xFD`) + bzip2/xz, since publishers ship `.hdt.zst`/`.hdt.bz2`; (iii) remove the minor double-`BufReader` (`lib.rs:93`).
- Impact: small-medium, **only for compressed inputs**. Risk: low. Gate: decompress+parse MB/s on `.hdt.gz`/`.hdt.zst` of the same data == plain-load triple set.

**H6 — Parallelize the four dict sections' DECODE, not just CRC (needs fork/PR; lower priority).**
- Lever: upstream reads sections serially (`four_sect_dict.rs:159-160`) and only threads the CRC32 (`dict_sect_pfc.rs:278`). Slurp packed bytes serially (I/O-bound), decode+intern four sub-dicts in parallel, merge via sparq's existing sharded-dict path (`dict.rs:1608`, used by `load_reader_parallel`). The shared section feeding both subject+object positions complicates merge.
- Impact: medium on multi-core. Risk: medium-high (merge correctness for the shared section). Gate: measure single-thread H1+H3+H4 first; only pursue if dict decode is still a top stage.

**H7 — Opt-in "trusted input" skip CRC32 (small; measure first).**
- Lever: CRC32-ISCSI over the whole packed dict + both sequences is real work on multi-GB files, already partly threaded. A `verify=false` flag skips it for trusted local archives.
- Impact: small (already partly threaded). Risk: low. Gate: HONEST — measure CRC's share of load time before bothering.

**NOT a lever:** `TripleStore::from_triples`'s 6-permutation build (`store.rs:298-324`) is shared with all loaders and HDT already gives SPO order, so the initial `par_sort_unstable` is on nearly-sorted input (fast). The `compact-index` feature already cuts to 3 permutations. Out of scope for HDT.

### Compressed-HDT handling (two distinct layers — do not conflate)
- **(a) Outer `.hdt.gz` wrapper:** already streaming/fused/content-sniffed (`lib.rs:79-98`). Extend codecs only (H5). The decompressed `.hdt` is never fully materialized — keep that.
- **(b) HDT-internal compression** (PFC dict + BitmapTriples Log64 sequences + rank/select): decoded eagerly by `Hdt::read`; there is no laziness. The real gap is *index materialization*, not on-disk compression — addressed by H1–H4. `Sequence::get` (`sequence.rs:105-122`) is already a clean branchless shift/mask — leave it.

### Reference + correctness oracle
- **Primary reference:** `hdt-cpp` (`github.com/rdfhdt/hdt-cpp`) — its `hdt2rdf`/`hdtSearch` bulk decode-to-triples is the de-facto fast baseline. "Optimal" lower bound = I/O + CRC + unavoidable PFC decode of every distinct term + O(triples) to enumerate adjacency. **Secondary:** the wrapped `hdt 0.4` crate itself (its `benches/criterion.rs`+`iai.rs`) as the in-process A/B oracle.
- **Differential oracle (the load-bearing one):** keep wrapped `hdt 0.4`'s `Hdt::read`/`id_to_string`/`triples_all` as an *in-process* oracle. New test: the sparq-side direct decoder (H1) must yield the **identical** `BTreeSet<[String;3]>` as `graph_from_hdt(&Hdt::read(...))` on the same bytes. Verifiable with no external binary.
- **Extend `crates/sparq-hdt/tests/roundtrip.rs`** (current: `snikmeta_hdt_matches_ntriples` — 328 triples, real archive not produced by our code path; `generated_hdt_round_trips`). Add fixtures: a **multi-block PFC dictionary** (>16 strings/section, exercises H3's block-walk across boundaries), a **shared-section-heavy** file (subject==object IRIs, `lib.rs:191-209`), and `.hdt.gz`/`.hdt.zst` of the same data (assert == plain). Optional cross-impl oracle: `hdt-cpp hdt2rdf` on the same archive, compared as a **normalized term set** (bnode labels and lang/xsd:string canonicalization differ across tools — never line-by-line).

---

## 2. Turtle workstream

CORRECTION TO TASK PREMISE: the prior work's "rejected custom Turtle parser / bnode-fallback / prefix-state-makes-chunking-hard" describes the *baseline-day* state. Two of those have **already shipped** and are in the tree today:
- **Statement-boundary chunking + chunk-parallel oxttl-per-chunk shipped** (`parse_turtle_parallel` `lib.rs:2500`; `turtle_chunks` `lib.rs:2442`; `parse_turtle_chunk` `lib.rs:2285`; `parse_turtle_chunked` `lib.rs:2515`). It is NOT a custom byte-level Turtle grammar parser.
- **Prefix-snapshot-broadcast shipped** (as byte-prefix prepend): `turtle_chunks` consumes the leading `@prefix`/`@base` preamble once and prepends it byte-for-byte into every chunk (`lib.rs:2483-2487`), so each chunk is a self-contained valid Turtle document.
- **Statement-terminator pre-scan shipped:** `next_terminator` (`lib.rs:2341-2411`) scalar-scans for a `.` followed by ws/EOF/`#`, skipping `<...>` IRIs, all four string-quote forms with escapes, comments, and `PN_LOCAL_ESC` (the `\#`-in-local fix from review 1398). `turtle_chunks` pre-scans the whole body collecting terminators then partitions into ~`target` groups.

So the **live gap is per-byte serial throughput** (oxttl ~74 MB/s vs custom NT 234 MB/s), driven by oxttl materializing an `oxrdf::Term` per term occurrence — exactly the allocation `nt.rs` exists to avoid.

### Bnode-fallback granularity issue — SOLVED, optimally (do not redo)
The original bail-to-serial on *any* `_:` node anywhere (document granularity → 42 bnodes in 1.5M statements forfeited ALL parallelism → 0% real-data speedup) is **gone** (commits `9006fa2`, `8c5788e`). The fix (`lib.rs:2419-2440`): **the dict merge IS the shared label-intern map.** Labeled `_:x` in distant chunks both route through `intern_blank("x")` (`dict.rs:1049`) at merge time → unify to one global id, identical to serial. Anonymous `[...]`/`(...)` are single-statement → single-chunk, need distinctness not unification (oxttl mints `BlankNode::default()`, 2^128). No per-chunk renaming, no boundary scan, no post-pass. Finer than "a few bnodes don't forfeit parallelism" — it forfeits *nothing*. **No remaining bnode-granularity work.**

### Ranked steps to close the NT gap

**T1 — Replace the per-chunk oxttl worker's `oxrdf::Term` materialization with direct slice interning (only structural lever left; MEASURE FIRST).**
- Lever: `parse_turtle_chunk` (`lib.rs:2285`) does, per triple, `subject_term(&t.subject)` (owned `Term`), `Term::NamedNode(t.predicate.clone())`, then `dict.intern(&Term)` re-borrows `.as_str()`. This `oxrdf::Term` heap object per S/P/O is the 234-vs-74 MB/s structural gap (the baseline measured the `Term` materialization alone costs more than the entire custom NT scan+intern). Cheapest version that does NOT write a new grammar: keep oxttl's tokenization/grammar, intern from borrowed `&str` views — via (a) a low-alloc/borrowing oxttl `for_slice` variant if one exists (RDF 1.2 triple terms + prefixed-name expansion complicate it), or (b) a thin term-interner fed by oxttl's lexer.
- Expected impact / HONEST CEILING: the baseline measured even a fully custom serial Turtle parser caps at ~1.9× because oxttl parse is only 53% of serial ingest — so this lever's end-to-end ceiling is **≤1.9× on the serial leg, eroded further by the existing 8-thread fan-out → expect single-digit-% to ~1.5× full-ingest at best, NOT NT parity.**
- Risk: medium. Gate: a `bench/parse` before/after measurement AND the W3C TurtleTests rejection oracle (§oracle) — a custom interner must reject the same malformed inputs oxttl does. Do NOT do a blind rewrite.

**T2 — SIMD/memchr the terminator pre-scan `next_terminator` (cheap, small, low-risk).**
- Lever: `next_terminator` (`lib.rs:2341`) is a scalar byte loop (no `memchr`/SIMD in the parse path today); the pre-scan walks the entire body once before fan-out → pure critical-path latency. Replace the inner "advance to next interesting byte" with `memchr3(b'.', b'<', b'"', ...)`.
- Impact: HONEST — small. Byte-scanning is ~0.4% of memory bandwidth; the residual cost is hashing/interning/sort, not scanning. Do it because it's cheap and low-risk, not because it's large.
- Risk: low. Gate: pre-scan stage timing; add `memchr` behind the `parallel` feature.

**T3 — Handle interspersed/SPARQL-style directives instead of bailing to full serial (close the cliff). [OPUS-4.8] DONE (sq-ouq), BOTH directive forms.**
- Lever: previously one mid-document `@prefix`/`@base` after the body starts, or any `PREFIX`/`BASE` SPARQL-style keyword, dropped the **whole file** to serial. Now `turtle_chunks` records every directive (BOTH the `.`-terminated `@`-form via `next_terminator` AND the no-`.` SPARQL keyword form via `next_sparql_directive_end`, which delimits at the directive's IRIREF) as an ordered byte-span, and prefixes each chunk with the verbatim in-order snapshot of the directives in scope at that chunk's start. Replays the document's directive prelude exactly → correct for redefinitions, relative `@base`/`BASE`, and mixed `@`/SPARQL documents (oxttl re-parses the snapshot in document order).
- Impact: workload-dependent — **zero** for predicate-grouped serializer output (the common Wikidata case, already fans out), meaningful for hand-written/tool-emitted Turtle that redeclares mid-file. Converts 0× cliff → multi-× (measured 3.73× at 8T on a 25 MB SPARQL-redeclaring doc, oxttl-only; full-ingest lands in the documented ~2.0–2.35× envelope once dict-merge is added).
- Rejection safety: a trailing `.` after a SPARQL directive's `>` (INVALID Turtle, W3C `turtle-syntax-bad-base-03`) is deliberately NOT consumed — it stays in the stream so the per-chunk oxttl parse rejects it; an over-eager swallow would silently accept invalid input (the corruption class the chunked==serial oracle cannot catch). Truncated/malformed directives → `None` → serial fallback → oxttl rejects.
- Gate (all green): W3C rdf-turtle through the sparq Turtle parser 313/313 (incl. `SPARQL_style_prefix`/`SPARQL_style_base` + `turtle-syntax-bad-base-03`); `parallel_turtle_interspersed_directives_match_serial` extended (SPARQL redef + relative BASE, mixed @/SPARQL, comment-between-keyword-and-IRIREF — all fan out & equal serial); `parallel_turtle_rejects_malformed` extended (SPARQL trailing-`.` + truncated IRIREF); clippy -p sparq-core and clippy --workspace --exclude sparq-py both exit 0.

**T4 — Switch Turtle's per-chunk dict merge to the sharded merge (lower priority). [DONE — sq-eq26]**
- Lever: `parse_turtle_chunked` merged per-chunk dicts serially (`merge_remap`+`remap_extend`) — the *plain* merge, not the sharded merge the external NT loader uses. On the NT path this serial merge is the identified residual bottleneck (the `Dict::merge_remap` workstream in `fast-ingestion.md`; ~200 s/1B at full scale). Used the same sharded-dict merge for Turtle.
- Implemented (sq-eq26): the serial `merge_remap` loop in `parse_turtle_chunked` now calls the shared `merge_partials`, which keeps the serial `merge_remap` on a single thread but fans the per-chunk dicts into one `ShardedDict` (`sharded_extend` + `finish_sharded`) on ≥2 threads — the exact path `load_ntriples_pipelined` / `parse_ntriples_parallel` use. Eligible because sq-87bq taught the sharded merge to consolidate RDF-1.2 triple terms. Output is identical to the serial merge (Turtle's per-chunk parser resets prefix/base/blank scope per chunk and emits fully-resolved terms + per-chunk-unique blank labels, so a partial is structurally an NT block). Single-thread path unchanged.
- Correctness oracles (all green): `parallel_turtle_bnodes_match_serial`, `parallel_turtle_quoted_triples_match_serial`, `parallel_turtle_interspersed_directives_match_serial`, `parallel_turtle_matches_serial`, plus the new `parallel_turtle_sharded_merge_matches_serial` (runs `parse_turtle_chunked` under an explicit 4-thread rayon pool — forcing the sharded path — over a fixture mixing prefixes/base + shared labelled/anonymous bnodes + RDF 1.2 triple terms + `{| … |}` annotations, asserting equality with the serial parse under canonical bnode renumbering). W3C rdf-turtle (rdf11+rdf12) 313/313 unchanged.
- Impact: helps on many-core boxes; it's the same separate workstream already scoped for NT. Risk: low-medium. Gate: 8T-vs-1T scaling RATIO on a real slice (bench `parse` / `bench-ttl`) measured on a quiet CANONICAL runner — not measured on the non-canonical EC2 work box.

### Achievable envelope (honest)
- Per-triple: already roughly NT-competitive serially (2.05 vs 1.60 Mt/s parse-only).
- Parallel scaling: bnode cliff gone → clean predicate-grouped Turtle scales ~2.0–2.35× (8T), matching the bnode-free path. It does **NOT** reach NT's 3.84× because (i) per-chunk oxttl allocates `oxrdf::Term`s the custom NT parser skips, and (ii) the terminator pre-scan + per-chunk preamble-prepend are serial/duplicated overhead NT's trivial newline-split avoids.
- **Constructs that CANNOT match NT regardless of engineering:** prefixed-name expansion (`wd:Q42` → full IRI string concatenation NT never does) and the format's intrinsic byte-count-for-parse-work trade. Multi-line constructs (triple-quoted strings, `;`/`,` lists, `[...]`, `(...)`) require the terminator pre-scan that NT's newline split avoids.
- Headline: **"match NT" is mostly already met for the common Wikidata-shaped file** (the bnode fix closed 0×→2×). The residual is the `oxrdf::Term` tax (T1), a measure-first spike with a sub-2× ceiling — not a guaranteed parity win.

### Correctness oracle (vs oxttl)
- **Exists:** `parallel_turtle_bnodes_match_serial` (`lib.rs:3033`) — chunked-vs-serial-oxttl exact equality after first-occurrence bnode renumbering (`canon_bnodes`; subsumes isomorphism since doc order is preserved); covers shared labels crossing every boundary, anonymous nests, collections, all-bnode chains, sparse Wikidata-shaped labels, bnode-free control. On the real slice, 8T-chunked == plain-serial-oxttl triple sets are **byte-identical** (1.5M statements). Plus `parallel_turtle_escaped_hash_in_local` (`lib.rs:3105`). The SPARQL differential fuzzer (`crates/sparq-bench/src/fuzz.rs`) loads via sparq AND Oxigraph and asserts equal solutions — exercises the Turtle parser on every seed.
- **GAP (empirical honesty):** the oracle proves "chunked ≡ serial-oxttl" (parallelism doesn't change the answer), NOT "sparq Turtle ≡ W3C spec" — its reference *is* oxttl. The W3C `rdf-tests/rdf/rdf11/rdf-turtle` syntax + **negative (rejection)** suite is NOT wired (`scripts/fetch-conformance.sh` pins only SPARQL suites; `sparq-conformance` runs SPARQL + N3 `TurtleTests`, not W3C `TestTurtleEval`/`TestTurtleNegativeSyntax`).
- **REQUIRED before T1/T3:** wire the pinned W3C **TurtleTests** (positive eval with oxttl as differential oracle + `TestTurtleNegativeSyntax` *assert-rejection*) into `sparq-conformance`, running `parse_to_triples("turtle")`. The negative cases are load-bearing: a custom interner (T1) or per-chunk prefix-snapshot (T3) must reject the same malformed inputs oxttl does — an over-eager split that accidentally parses invalid Turtle as valid is a silent corruption the chunked-vs-serial oracle cannot catch. Today rejection is delegated to oxttl-per-chunk (a mis-split fails a chunk → serial fallback, `lib.rs:2532-2535`); a custom interner needs its own rejection-parity tests.

---

## 3. Already REJECTED-with-evidence (do NOT re-tread)

| Idea | Verdict + evidence | Status |
|---|---|---|
| Custom single-thread NT parser | REJECT — incumbent already IS one, beats oxttl parse-only 234 vs 185 MB/s; zero-cost scanner caps full serial ingest at ~1.8×, realistic ≤1.2× at 8T | holds |
| Parallel NT parsing | REJECT — already shipped, newline-split + sharded merge, 3.84× / 896 MB/s | holds |
| Custom single-thread Turtle grammar parser | REJECT — oxttl parse is 53% of serial ingest → caps at ~1.9×; large/treacherous grammar surface (the `\#`-in-local review-1398 bug is concrete evidence even the *splitter* is subtle) | holds — do NOT write a from-scratch Turtle grammar parser |
| Bnode document-scope bail | OBSOLETE — better fix shipped (dict-merge unifies labels, no remap/collision detection); the *bail* rejection is now moot | superseded |
| Naive byte-range Turtle splitting | REJECT as incorrect — prefixes/multi-line/`;`/`,`/doc-scoped bnode labels cross boundaries; only statement-terminator chunking is correct | holds |
| Streaming per-`read()` flush | FOUND + FIXED — was flushing a parse+merge per 0.4–1.6 MB read; now pipelined, 6.9–8.5× | done |
| Parallel decompression (multi-frame zstd / multi-member gzip / lbzip2) | REJECT for the parse pipeline — single-thread zstd decode 1,236 MB/s already outruns 8T parse+build 585 MB/s; only bzip2 sources benefit → recompress `.bz2`→`.zst` once | holds |
| Parallel bzip2 decoder | REJECT — recompress to `.zst`, don't build a parallel bzip2 decoder | holds |
| HDT write support | DEFERRED, blocked upstream — no in-memory FourSectDict builder API (re-verified vs 0.6) | blocked |
| HDT crate upgrade 0.4→0.6 | DEFERRED — 0.6 pulls `qwt` whose default `prefetch` needs nightly on aarch64; and 0.6 adds *more* unused query structure | pinned 0.4 |
| Radix-sort permutations | DEPRIORITIZED — marginal on bandwidth-bound M1; sort hidden under parse+I/O until `merge_remap` is fixed | parked |

---

## 4. Benchmark plan

**Datasets** (all gitignored/regenerable; never delete tracked scripts/.gitignore; cap dataset size + clean `/tmp` per disk-space discipline):
- `wikidata-slice.nt` (173 MB, 1.5M triples, 42 real `_:` bnodes — first 1.5M lines of truthy dump via `bzcat … | head -n 1500000`) and `synthetic.nt` (170 MB, 2.56M triples, `crates/sparq-bench/src/dataset.rs`). Turtle slice via `bench/parse` `to-ttl` (wd/wdt prefixes, predicate-grouped). Compressed via `compress` (gz −6, zst −3).
- **MISSING — must build for HDT:** a **real multi-M-triple `.hdt`** (build via `hdt::Hdt::read_nt` from the existing `.nt` slice) so HDT's size win shows over realistic term-reuse; `.hdt.gz`/`.hdt.zst` of it; a multi-block-PFC fixture and a shared-section-heavy fixture for the roundtrip differential.

**Harness:** reuse `bench/parse/` (standalone cargo, mimalloc + fat-LTO to match shipped `sparq-cli ingest`, median-of-3, MB/s over **decompressed** bytes; subcommands `gen`/`to-ttl`/`compress`/`bench-nt`/`bench-ttl`/`bench-zip`/`probe-read`). **Add to it:** a `bench-hdt` entry that times HDT load against the *fast* `load_reader_parallel` NT path with a **per-stage breakdown** (`Hdt::read` decode vs `graph_from_hdt` id-translation vs `Graph::build`), and a `bench-hdt-zip` for `.hdt.gz`/`.hdt.zst` (currently sniffed+streamed but **unmeasured**). `crates/sparq-hdt/examples/bench_load.rs` is best-of-3 single-thread synthetic against the *slow* serial `.nt.gz` path — keep but do not gate on it.

**Metrics:** MB/s parse (decompressed bytes); MB/s decompress+parse (compressed inputs); triples/s; per-stage time split; **peak RSS** (per disk/RAM discipline — the 1B-truthy run peaked 51.5 GB). For Turtle: 1T vs 8T scaling ratio (the durable claim).

**Differential-correctness gates (every perf claim is gated on these):**
- HDT: sparq direct decoder (H1) `BTreeSet<[String;3]>` == `graph_from_hdt(&Hdt::read(...))` on the same bytes; multi-block + shared-section fixtures; `.hdt.gz`/`.hdt.zst` == plain. Extend `crates/sparq-hdt/tests/roundtrip.rs`.
- Turtle: existing chunked-vs-serial-oxttl byte-identity + the SPARQL fuzzer; **plus** wire W3C TurtleTests positive (oxttl differential) + negative (assert-rejection) before T1/T3.

**NOTE — headline numbers must be taken on a QUIET box:** no concurrent builds, no other agents, monitor `df` during tests (clean `/tmp` scratch, cap dataset size). The fanless M1's thermal state moves absolute numbers session-to-session — **ratios/orderings are the durable claims, absolute MB/s is not.**

---

## 5. Implementation-wave split

**Wave A (build first — sequential, the HDT headline):** H1+H2 as one change (sparq-side SPO decoder reading bitmaps/sequences directly, predicates from `sequence_y`, skipping `TriplesBitmap::new`/wavelet/OP-index). Gate: differential `BTreeSet` equality vs `Hdt::read` oracle on snikmeta + a multi-M-triple HDT; per-stage timing shows the index-build stage gone and load time meaningfully below upstream `Hdt::read`+translate.

**Wave B (parallelisable with Wave A — independent files):**
- B1: H3+H4 (PFC block-sequential decode + borrowed-slice interning) in `dict_sect_pfc`-equivalent sparq code. Gate: dict-decode stage time down, differential equality across block boundaries (multi-block fixture).
- B2: H5 (zstd/bzip2 sniffing + `zlib-ng` backend) in `lib.rs:79-98` + `Cargo.toml`. Gate: `.hdt.zst`/`.hdt.gz` decompress+parse MB/s == plain triple set.
- B3 (Turtle, fully independent of HDT): T2 (memchr the pre-scan). Gate: pre-scan stage time down, all existing Turtle differential tests pass.
- B4 (Turtle, prerequisite for T1/T3): wire W3C TurtleTests (positive + negative) into `sparq-conformance`. Gate: suite runs green against current oxttl-per-chunk path (establishes the rejection oracle before any interner change).

**Wave C (gated on Wave A/B measurements — only if the stage is still hot):**
- C1: H6 (parallel dict-section decode + sharded merge) — only if dict decode remains a top stage after B1.
- C2: T1 (drop the `oxrdf::Term` tax) — only behind a `bench/parse` before/after AND the B4 rejection oracle; abandon if it doesn't beat the ≤1.9× serial-leg ceiling enough to matter after fan-out.
- C3: T3 (interspersed-directive prefix snapshot) — only if target workloads include non-predicate-grouped Turtle; gate on the interspersed-`@prefix` fixture + W3C suite.
- C4: T4 (sharded merge for Turtle) and H7 (CRC skip) — measure share-of-time first; pursue only if the share justifies the risk.

**Sequencing rationale:** Wave A is the dominant HDT win and unblocks everything HDT. B1–B4 parallelise across distinct files/crates with no shared edits. Wave C is strictly measurement-gated — each item names the measurement that authorizes it, and each is abandoned if the gate isn't met (empirical honesty: T1 and C-tier are spikes, not commitments).
