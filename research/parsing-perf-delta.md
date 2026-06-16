<!-- [OPUS-4.8] parsing-perf design-delta for epic sq-4wo. Additive, empirically honest:
     verified the spec (research/parsing-optimization-plan.md) against the tree on 2026-06-16.
     Most of the epic is LANDED; this records what genuinely remains. No hard-coded perf numbers. -->

# Parsing/ingest perf — design-delta for sq-4wo (verified state-of-play + genuine remainders)

Epic: **sq-4wo** ("HDT + Turtle parse/ingest performance"). Spec: [`research/parsing-optimization-plan.md`](./parsing-optimization-plan.md).
This is an ADDITIVE delta: it does **not** restate the plan. It records, lever-by-lever, what was
**verified landed in the tree** (file:line) versus what **genuinely remains**, so the orchestrator
can close the epic with confidence and bead only the real residual work. All claims below were
checked against code; "landed" means the code path exists AND is wired AND has a differential gate.
Per repo discipline: no hard-coded perf numbers, no absolute machine paths.

## Headline

The epic is **substantially landed.** Every ranked lever in the plan — HDT H1–H6, Turtle T1–T3 — is
in the tree, wired into the default path, and gated by a differential oracle. All seven `sq-4wo`
children are closed, and additional HDT work the plan did not pre-bead (parallel dict decode `sq-s506`,
in-memory HDT encoder `sq-ashy`, HDT write `sq-2te`, zlib-ng `sq-i3xc`) also landed. What genuinely
remains is a single algorithmic lever the plan filed as measure-first C-tier (**T4** — Turtle's parallel
dict merge is still the *serial* `merge_remap`, not the sharded merge proven on the N-Triples path),
plus two small tooling/honesty gaps (bench per-stage granularity; rdf-turtle suite not explicitly pinned).
**H7** (CRC skip) is absent but the plan itself rates it marginal and measure-first.

---

## A. HDT workstream — verified

| Lever | Plan intent | Verified state | Evidence |
|---|---|---|---|
| **H1+H2** | sparq-side direct SPO decoder; skip `TriplesBitmap::new` (wavelet/OP-index/rank-select); predicates from `sequence_y` not the wavelet | **LANDED + DEFAULT.** `decode::graph_from_reader` walks `bitmap_y/z`+`sequence_y/z` in SPO order, no wavelet/OP-index build; `load_reader` defaults to it. | `crates/sparq-hdt/src/decode.rs:1-60`; wired at `lib.rs:54,234`; upstream path kept as oracle `load_reader_via_upstream` `lib.rs:241` |
| **H3** | PFC block-sequential dict decode into a reusable buffer (O(N) not O(N·block)) | **LANDED.** `decode_section` walks each PFC block once into one reusable buffer. | `decode.rs` (`decode_section`, doc `:28-40`) |
| **H4** | Intern from borrowed `&str`, drop the per-id `String` round-trip | **LANDED.** Each decoded term interned from a borrowed view; only the dict's own `Box<str>` allocates. | `decode.rs:40-46` doc + impl |
| **H5** | `.hdt.gz` already streamed; add zstd + bzip2 sniffing + zlib-ng backend | **LANDED.** Magic-byte sniff for gz/zst/bz2 (streaming, never materialized); `zlib-ng` opt-in feature. | `Cargo.toml` (`zstd`,`bzip2`,`zlib-ng` feature); gate `tests/roundtrip.rs::all_codecs_decode_to_identical_triple_set` |
| **H6** | Parallelize the four dict sections' DECODE (not just CRC); merge via sharded path | **LANDED** (`sq-s506`, PR #100). Four independent PFC blobs decoded concurrently on rayon, each into its own partial `Dict`, merged via `merge_remap` in fixed section order → bit-for-bit identical ids. | `decode.rs:48-60,320,357`; commits `0a17b07`,`6500056` |
| **H7** | **Opt-in** "trusted input" CRC32 skip | **ABSENT.** CRCs always verified; no `verify`/`skip_crc` flag. Plan rates this *small / measure-first* (CRC already partly threaded). | `decode.rs:34` ("CRCs are still verified"), `:149-175` |

**Differential gate (verified present):** `tests/roundtrip.rs` asserts the direct decoder yields an
identical `BTreeSet<[String;3]>` to `graph_from_hdt(&Hdt::read(...))` on the same bytes
(`assert_direct_matches_upstream` `:257`), across a **multi-block-PFC + shared-section** fixture
(`direct_decoder_matches_upstream_generated_multiblock` `:313`, asserts `num_strings > block_size`),
plus `.hdt.gz`/`.hdt.zst`/`.hdt.bz2` == plain (`all_codecs_decode_to_identical_triple_set` `:384`) and
truncation/bit-flip rejection-oracle tests. This is exactly the §4 gate the plan demanded.

**HDT conclusion:** H1–H6 done and gated. Only **H7** is open, and it is a weak candidate (see §D).

---

## B. Turtle workstream — verified

| Lever | Plan intent | Verified state | Evidence |
|---|---|---|---|
| Statement-boundary chunking + prefix-snapshot + terminator pre-scan | shipped at baseline | **LANDED.** `turtle_chunks`/`parse_turtle_chunk`/`parse_turtle_chunked`/`next_terminator`. | `crates/sparq-core/src/lib.rs:3758,3865,3997,4101` |
| **T1** | Drop the per-triple owned `oxrdf::Term` materialization; intern from borrowed slices (measure-first, ≤1.9× serial ceiling) | **EFFECTIVELY LANDED.** `parse_turtle_chunk` uses `TurtleParser::for_slice` and interns directly via `intern_subject_ref`/`intern_object_ref` from `&str` views — no owned `Term`, no `.clone()`, no `dict.intern(&Term)` round-trip. The deeper "own lexer, drop oxrdf term types entirely" was the risky tail the plan flagged as abandonable; not done, and correctly so. | `lib.rs:3758-3767`, `intern_subject_ref` `:3723`, `intern_object_ref` `:3737` |
| **T2** | memchr/SIMD the terminator pre-scan | **LANDED** (`sq-bhd`). `next_terminator` uses `memchr3`; `memchr` rides the `parallel` feature. | `lib.rs:3877-3894`; `Cargo.toml` (`parallel = [...,"dep:memchr"]`) |
| **T3** | Per-chunk prefix-snapshot for interspersed/SPARQL-style directives instead of bailing to serial | **LANDED** (`sq-ouq`). Records both `@`-form (via `next_terminator`) and SPARQL keyword form (via `next_sparql_directive_end`, delimits at the IRIREF), prepends in-order snapshot; invalid trailing `.` deliberately left for oxttl to reject. | `lib.rs:3818-3847,3966-4080` |
| **T4** | Switch Turtle's per-chunk dict merge to the **sharded** merge (the NT path's proven win) | **OPEN — the one genuine algorithmic remainder.** `parse_turtle_chunked` still does the **serial** `global.merge_remap(&pd)` in a loop — the exact merge the NT loader replaced with `ShardedDict`+`sharded_extend`+`finish_sharded` because serial `merge_remap` capped scaling. | `lib.rs:4101-4130` (serial loop) vs NT path `lib.rs:1068-1148` (`ShardedDict`, `sharded_extend`, `finish_sharded`) |

**Conformance gate (verified present):** W3C rdf-turtle is wired into `sparq-conformance`: positive
syntax + `TestTurtleEval` (oxttl-differential, blank-node isomorphism) + `TestTurtleNegativeSyntax` +
`TestTurtleNegativeEval`, all through the sparq parser (`parse_to_triples_with_base(..,"turtle",..)`)
(`crates/sparq-conformance/src/turtle_suite.rs:94-131,112,160-211`). In-tree differential tests
`parallel_turtle_bnodes_match_serial` (`lib.rs:4860`), `parallel_turtle_interspersed_directives_match_serial`
(`:4991`), `parallel_turtle_rejects_malformed` (`:5282`) all present. This satisfies the plan's
"REQUIRED before T1/T3" rejection-oracle precondition — and since T1/T3 already landed, the precondition
is met retroactively and correctly.

**Turtle conclusion:** T1–T3 landed and gated. **T4** is the single real remaining algorithmic milestone.

---

## C. Genuine remaining milestones (to bead)

### M1 — Turtle parallel path: sharded dict merge (lever T4)
- **Goal:** Replace the serial `merge_remap`-in-a-loop in `parse_turtle_chunked` with the same
  `ShardedDict` + `sharded_extend` + `finish_sharded` path the N-Triples loader uses on ≥2 threads,
  so chunked Turtle scales past the ~serial-`merge_remap` ceiling toward the NT path's scaling.
- **Crate:** `sparq-core` (`crates/sparq-core/src/lib.rs`, `parse_turtle_chunked` ~`:4101`; reuse
  `dict::ShardedDict`, `sharded_extend`, `finish_sharded` already in the file).
- **Algorithmic approach:** On `rayon::current_num_threads() > 1`, build one `ShardedDict` spanning all
  chunks; each chunk-worker interns into it (component-level, no per-chunk owned `Dict`); one parallel
  final remap via `finish_sharded`. Single-thread keeps the proven serial path. The earlier blocker —
  the sharded interner rejected RDF 1.2 triple terms — is **gone**: `sq-87bq` made the sharded merge
  consolidate triple terms (`lib.rs:3634-3641`), so Turtle (which can carry RDF-star) is now eligible.
- **Opt-in/lean note:** Already under the existing `parallel` feature; no new dep, no core-bloat.
- **How measured (canonical runner, NOT this EC2 box):** `bench/parse` `bench-ttl` 1T-vs-8T scaling
  ratio on the predicate-grouped Wikidata-shaped Turtle slice, before/after. The durable claim is the
  **scaling ratio** moving toward the NT path's, not absolute MB/s.
- **Done-definition:** chunked Turtle uses the sharded merge on ≥2 threads; all existing Turtle
  differential oracles stay green (`parallel_turtle_*_match_serial`, W3C TurtleTests, the SPARQL fuzzer);
  `bench-ttl` 8T scaling improves over the serial-merge baseline on a quiet box; single-thread path
  unchanged. From sq-4wo research.

### M2 — `bench-hdt` per-stage granularity + NT-vs-HDT A/B (measurement honesty, not algorithmic)
- **Goal:** The plan's §4 asks the HDT bench for a **3-way** per-stage split (decode vs id-translation
  vs `Graph::build`) **and** an A/B against the fast NT `load_reader_parallel` path. Today `bench-hdt`
  does a 2-way upstream split (`Hdt::read` vs fused `graph_from_hdt`) and compares direct-vs-upstream
  HDT only — no NT comparison, and id-translation/`Graph::build` are fused.
- **Crate:** `bench/parse` (`bench/parse/src/main.rs`, `bench-hdt` ~`:632`) — standalone bench, not core.
- **Approach:** instrument the direct decoder's stages (dict decode / triple scan / `Graph::build`)
  behind a timing hook, and add an NT-load A/B row (load the same data's `.nt` via
  `load_reader_parallel`) so the "HDT vs fast NT" gap the plan headlines is actually measurable.
- **How measured:** the bench prints the split; this IS the measurement tool, validated by the bench
  running and the existing roundtrip differential staying green.
- **Done-definition:** `bench-hdt` emits a 3-way stage split for the direct path and an NT-load A/B row;
  no behaviour change to the decoder. Lower priority than M1 (tooling, enables future tuning, not a
  shipped speedup). From sq-4wo research.

### (Considered and NOT beaded — see §D)
- **H7** (opt-in CRC skip), **rdf-turtle explicit pin**, **deeper T1 own-lexer**, **HDT 0.4→0.6**,
  parallel decompression, radix-sort permutations, HDT write — each rejected/deferred with reason below.

---

## D. Deliberately NOT beaded (empirical honesty)

| Candidate | Verdict |
|---|---|
| **H7 — opt-in CRC32 skip** | NOT beaded. Plan itself rates it *small + measure-first*; CRC is already partly threaded and is a fraction of multi-GB load. Trusted-input flag also widens the silent-corruption surface. No measurement yet justifies it; do not add speculative knobs. Revisit only if a canonical profile shows CRC is a top stage. |
| **rdf-turtle suite explicit pin** | NOT beaded as perf. The suite IS fetched (rides the whole-repo `w3c/rdf-tests` clone, `scripts/fetch-conformance.sh`) and IS run; it just lacks a named fetch line. Cosmetic CI hygiene, not parse-perf, and out of sq-4wo scope. |
| **Deeper T1** (own lexer, drop `oxrdf` term types entirely) | NOT beaded. The plan caps the serial leg at ≤1.9× and that erodes under 8-thread fan-out; the borrowed-`&str` interning (the cheap 80%) already landed. The remaining tail is a large/treacherous grammar surface (the `\#`-in-local review-1398 bug proves even the splitter is subtle) for sub-2×-on-the-serial-leg upside. The plan's own conclusion: abandon if it doesn't clear the ceiling. Do not write a from-scratch Turtle grammar parser. |
| **HDT 0.4→0.6 upgrade** | Deferred (plan + `Cargo.toml`): 0.6 pulls `qwt` (needs nightly on aarch64) and adds *more* unused query structure — widening, not closing, the gap our direct decoder already bypasses. |
| Parallel decompression / parallel bzip2 | REJECTED with evidence in the plan (single-thread zstd decode already outruns 8T parse+build; recompress `.bz2`→`.zst`). Holds. |
| Radix-sort permutations | Parked — bandwidth-bound, hidden under parse+I/O. Holds. |
| HDT write support | Already shipped opt-in (`sq-2te`, `write` feature) — not a perf remainder. |

---

## E. Summary for the orchestrator

- **HDT:** H1–H6 landed, wired as default, gated by the `BTreeSet` differential oracle across
  multi-block/shared-section/all-codec fixtures. Only **H7** open — and it's a marginal, measure-first
  knob, NOT beaded.
- **Turtle:** T1–T3 landed and gated (including the W3C TurtleTests positive+negative oracle the plan
  required before T1/T3). **T4 (sharded merge) is the one genuine algorithmic remainder** → **M1**.
- **Tooling:** `bench-hdt` lacks the plan's 3-way stage split + NT A/B → **M2** (lower priority).
- Net: epic sq-4wo is substantially complete; two real beads (M1 algorithmic, M2 tooling) remain.
