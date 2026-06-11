# T3 verdict — u64 inline ValueIds

*Measure-first evaluation of widening `Id` to `u64` with tag bits that inline
common literal values (integers beyond the current range, xsd:decimal,
xsd:dateTime/date, xsd:boolean, short strings) directly in the id.
Roadmap thread T3 (#23 + #26). Machine: Apple M1 (16 GB), 2026-06-11.*

**VERDICT: REJECT (measured).** Widening to u64 costs 2x permutation memory and
~1.6x on bandwidth-bound scan kernels at both olympics and 10M scale, while the
only hot path still paying a dict round-trip — xsd:dateTime/date evaluation
(measured 15.6–16.3x slower than the cached-numeric path) — is fully
recoverable *within u32* by a `temporal_of` side-cache symmetrical to the
existing `numerics_of`. Integer/decimal FILTER and aggregation already bypass
the dict today (≈10 ns/row). On-disk compressed size is unaffected (+0%), so
disk is not a tiebreaker either way. Billion-scale (>2^31 distinct terms)
headroom remains the one motivation u32 cannot answer; deferred until a real
workload approaches the cap (see §4).

## 1. What exists today (recon)

sparq already implements the QLever tagged-ValueId idea *inside* `u32`, plus a
side-table that covers most of what u64 inlining would buy:

- **Id space partition** (`dict.rs`): `0` = NO_ID; dictionary ids `[1, 2^31)`
  (≈2.1 B distinct non-integer terms); **inline integers** `[2^31, 2^31 + 2^30)`
  — canonical non-negative `xsd:integer` values `0..2^30-1` carried directly in
  the id, sorting by value in the permutations (range pruning in FILTER);
  engine **local vocab** `[3·2^30, 2^32)` for query-computed terms (`exec.rs`
  `LOCAL_BASE`).
- **Numerics cache** (`lib.rs` `numerics_of`): `numerics[id-1] = f64` for every
  dictionary term (NaN if non-numeric), dense or sparse, persisted/mmap-able
  (`numerics.bin`). `Graph::numeric_value(id)` is O(1) and allocation-free for
  *all* numeric literals — negative/big integers, decimals, floats — not just
  the inline range. Numeric FILTER/comparison/SUM never round-trips the dict.
- **Exact-decimal path** (`exec.rs` `Dec`, `cmp_decimal_str`): exact fixed-point
  arithmetic/comparison re-parses the lexical only when the f64 fast path ties —
  rare, and bounded by `exact_numeric_lexical` (inline ids format directly).
- **Not covered**: `xsd:dateTime`/`xsd:date` FILTER/ORDER/MIN/MAX materialise an
  `oxrdf::Term` from the dict (allocation) and parse the lexical (`Timeline`)
  *per row, per evaluation*. Booleans similarly resolve through terms. This is
  the only remaining dict round-trip on literal-heavy hot paths.

## 2. Id-width leak inventory (what a u64 `Id` touches)

| Site | File | Interaction |
| --- | --- | --- |
| `pub type Id = u32` + partition constants | `dict.rs:20-66` | INLINE_BASE = 2^31, INLINE_MAX = 2^30-1, capacity assert at `push` (`dict.rs:567`) |
| Permutation arrays `Vec<[Id;3]>` / mmap `[u32;3]` | `store.rs` (PermData, overlay added/deleted, pred_stats) | 12 B/triple/perm → 24 B; ×6 native = 72→144 B/triple; ×3 wasm = 36→72 B/triple |
| Raw on-disk perm format | `store.rs:318` (`perm{i}.bin` little-endian `[u32;3]`), `lib.rs:1636` `remap_perm_file` casts mmap to `&mut [u32]` | format change; magic-detection heuristic in `compress.rs:20` assumes first id < 0x43515053 |
| Compressed perm format SPQCPRM1 | `compress.rs` | varints already widen transparently (`put_varint(u64)`), directory entries are `([Id;3], u32)`; **high tag bits make first-row/reset varints 9–10 bytes** (measured below) |
| External sort kernels | `extsort.rs` (TRIPLE_BYTES=12, run files, k-way merge) | row width doubles; run files double |
| Sharded dict merge | `dict.rs:1278-1295` ShardedDict stride = INLINE_BASE / n; `lib.rs:608` `sharded_remap: (Vec<u64>, u32)` | stride scheme tied to the u32 partition |
| Engine id spaces | `exec.rs:451-460` LOCAL_BASE = 3·2^30, `is_local`, `inline_pass_values` range arithmetic, JSON `inline_int_json(u32)` | every range-prune and id-class test |
| Join keys / hash tables | `exec.rs` `Key = SmallVec<[Id;2]>`, FxHashMap keyed by Id rows | hash + key memory doubles |
| Numerics cache indexing | `lib.rs` `numerics[id-1]` dense | stays dense (dict ids stay dense) — unaffected except index type |
| WAL / delta overlay | `lib.rs:908` (N-Triples lines — id-free, unaffected); overlay `Vec<[Id;3]>` | overlay memory doubles |
| sparq-hdt bridge | `crates/sparq-hdt/src/lib.rs:107` `Vec<Id>` per HDT section | width doubles |
| wasm target | `sparq-wasm` (compact-index 3 perms) | perm memory 36→72 B/triple in the browser; baseline bundle 1,560,961 B |

## 2b. The candidate u64 design (what was evaluated)

QLever-style 4-bit tag in the top nibble of a 64-bit id:

| tag | payload (60 bits) | notes |
| --- | --- | --- |
| 0 | dictionary id | 2^60 terms — billion-scale headroom |
| 1 | xsd:integer, signed 60-bit | covers ±5.8e17; negative + large ints inline |
| 2 | xsd:decimal, 4-bit scale + 56-bit signed mantissa | exact fixed-point within range |
| 3 | xsd:dateTime, epoch-seconds + tz-presence | epoch-encodable range |
| 4 | xsd:date | as above, midnight |
| 5 | xsd:boolean | 2 values |
| 6 | local-vocab id | engine-computed terms |
| 7 | short string ≤ 7 bytes | order-preserving big-endian pack |

Ordering coherence: today inline integers sort *by value* above all dict ids,
which `inline_pass_values` (exec.rs) exploits for binary-searched range FILTER.
A multi-datatype tag scheme must define a total order: per-tag blocks keep the
current semantics (each datatype's block sorts by value; cross-type comparison
stays a type error), but integers and decimals then live in *separate* blocks,
so a mixed-numeric column loses single-range pruning unless the numeric tags
share an order-preserving encoding (QLever solves this by encoding all numerics
as order-preserving doubles — surrendering sparq's exact-decimal semantics).

## 3. Measurements

### 3a. Win ceiling — dict-resolve cost today (literal-heavy 5M triples)

Fixture: `bench/u64-valueids/gen.py` — 1M subjects × {inline int, big int
(>2^30, cache path), decimal (cache path), dateTime (dict path), group key}.
`sparq-cli bench /tmp/t3-literals.nt ntriples bench/u64-valueids/queries 5 count`
(min of 5, count mode — no serialisation). Load: 5M triples in 1.67 s,
store ~0.48 GB, dict 1,202,145 terms.

| query | rows | best (ms) | path exercised |
| --- | ---: | ---: | --- |
| q01 filter int (inline id) | 499,990 | **0.0032** | index range prune — effectively free |
| q02 filter big int (cache) | 499,990 | 5.04 | `numerics_of` f64 cache, ≈10 ns/row, **no dict** |
| q03 filter decimal (cache) | 499,990 | 5.04 | same — already dict-free |
| q04 filter dateTime (dict) | 503,331 | 81.9 | term materialise + lexical parse/row — **16.3x q03** |
| q05 BIND int +1 | 500,000 | 72.4 | Term construction + `id_of` per computed value |
| q06 BIND dec +0.25 | 499,990 | 220.0 | + local-vocab intern (new lexicals) — **3.0x q05** |
| q07 SUM(int) GROUP BY | 40 | 88.7 | numerics cache — dict-free |
| q08 SUM(dec) GROUP BY | 40 | 104.2 | numerics cache — dict-free |
| q09 ORDER BY dateTime | 10 | 1,316.7 | dict round-trip per comparison — **15.6x q10** |
| q10 ORDER BY decimal | 10 | 84.6 | cached f64 sort key |
| q11 MAX(dateTime) GROUP BY | 40 | 144.4 | dict round-trip per row |

Decomposition of the ceiling:

- **Integers + decimals in FILTER / aggregation: there is ~nothing left to win.**
  The numerics cache already removes the dict from these paths (q02/q03/q07/q08
  ≈10 ns/row). u64 inlining could at best convert q02/q03's 5 ms into q01-style
  index pruning — a 5 ms/1M-row absolute win on paths that are not bottlenecks.
- **dateTime/date is the real cost (~16x)** — q04 82 ms vs 5 ms, q09 1.32 s vs
  85 ms, q11 144 ms. But this is a *cache gap*, not an id-width gap: an epoch
  side-table (`temporal_of: id → i64`, exactly analogous to `numerics_of`)
  recovers the same ~16x inside u32, with zero cost to perm width, wasm memory,
  or disk format.
- **BIND/computed values** pay Term construction (alloc + hash) even for ints,
  and local-vocab interning for new decimal lexicals (q06−q05 ≈ 148 ms/1M
  values). u64 inline decimals would skip the intern — but a `Value::Num` fast
  path in `value_to_id`/`value_to_id_readonly` (exec.rs:6174) that encodes
  inline-range integers *before* constructing a Term captures the integer half
  of this inside u32 today, and the decimal half is bounded by q06's 220 ms/1M,
  not a 2x-memory problem.

### 3b. Loss floor — 64-bit permutation rows

`cargo run -p sparq-core --example bench_id_width --release` — sort / full-scan /
100k-probe kernels over identical row data at both widths, olympics scale
(1.8M) and 10M rows. Apple M1, min of 3 (sort/probe) / 5 (scan).

| kernel | 1.8M u32 | 1.8M u64 | ratio | 10M u32 | 10M u64 | ratio |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| memory (one perm) | 21 MB | 43 MB | **2.00x** | 120 MB | 240 MB | **2.00x** |
| sort (build kernel) | 83.8 ms | 72.3 ms | 0.86x | 502.2 ms | 484.6 ms | 0.96x |
| full scan (sum col) | 0.4 ms | 0.7 ms | **1.60x** | 2.6 ms | 4.1 ms | **1.58x** |
| 100k range probes | 5.0 ms | 5.2 ms | 1.04x | 6.9 ms | 7.1 ms | 1.03x |

Reading: sort is compute-bound and even slightly *faster* at u64 (aligned 24 B
rows, single-word lexicographic compare), and pointer-chasing probes are
latency-bound and indifferent — but the bandwidth-bound scan kernel, which is
the shape of every pattern scan and merge join inner loop, pays the full ~1.6x.
Resident memory doubles unconditionally: native 72 → 144 B/triple across 6
perms, wasm 36 → 72 B/triple against a 4 GB linear-memory ceiling (halves
browser capacity; the u32-alias feature mitigation would mean maintaining both
widths everywhere).

### 3c. Compressed on-disk size

Same example: SPQCPRM1-style delta+varint size (128-row blocks, lexicographic
delta) for u32-valued ids vs u64 ids with inline values moved to tag bit 60.

| | raw u32 | varint u32-ids | varint u64 tag-bit60 |
| --- | ---: | ---: | ---: |
| 1.8M rows | 21 MB | 9 MB | 9 MB (**+0%**) |
| 10M rows | 120 MB | 52 MB | 52 MB (**+0%**) |

The just-merged SPQCPRM1 varint format absorbs the widening entirely: within a
tag-block, deltas are small regardless of where the tag bit sits; only
block-first rows and tag-boundary resets pay 9–10-byte varints, amortised over
128-row blocks to noise. **Disk is neutral — neither a cost of adopting nor a
benefit of staying.**

## 4. Decision

**REJECT u64 inline ValueIds.** The loss is structural and unconditional —
2x resident permutation memory (native and, critically, wasm) and ~1.6x on the
bandwidth-bound scan kernel that underlies every pattern scan — while the
measured win decomposes into pieces that are each cheaper to capture inside
u32:

1. Integer/decimal FILTER + aggregation already bypass the dict (numerics
   cache, ≈10 ns/row). Nothing material left.
2. The one genuine dict round-trip — dateTime/date at 15.6–16.3x — is a
   missing *side-cache*, not a missing id width. Follow-up (recommended, new
   thread): `temporal_of` epoch-seconds cache mirroring `numerics_of`,
   expected to collapse q04/q09/q11 to the q03/q10 envelope.
3. BIND-path Term construction for inline-range integers can short-circuit in
   `value_to_id`/`value_to_id_readonly` before allocating a Term — a localised
   u32 optimisation.
4. Ordering coherence is a real design risk for u64: per-tag blocks would
   split today's single by-value inline-integer ordering that
   `inline_pass_values` exploits for binary-searched range FILTER, unless all
   numerics share an order-preserving encoding (QLever's doubles — which would
   surrender sparq's exact-decimal semantics).

**Deferred, not dead:** >2^31 distinct terms (billion-scale dictionaries) is
the one motivation u32 cannot answer. When a workload approaches the cap, the
path is a parametrised `Id` type (feature-gated, wasm pinned to u32) — and the
measurements above say to take *plain* u64 dict ids at that point, not
inline-tagged ones: the inlining half of the idea is dominated by side-caches
either way.

No engine code was changed on this branch (measure-first; measurements said
stop). `cargo test --workspace`: 449 passed, 0 failed (65 suites). wasm bundle
(unchanged code = baseline): 1,173,584 bytes (`wasm-pack build --target web
--release`, wasm-pack 0.13.1).
