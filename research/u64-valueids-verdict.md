# T3 verdict — u64 inline ValueIds

*Measure-first evaluation of widening `Id` to `u64` with tag bits that inline
common literal values (integers beyond the current range, xsd:decimal,
xsd:dateTime/date, xsd:boolean, short strings) directly in the id.
Roadmap thread T3 (#23 + #26). Machine: Apple M1 (16 GB), 2026-06-11.*

**VERDICT: (pending measurements — see below)**

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
(min of 5, count mode — no serialisation).

(pending)

### 3b. Loss floor — 64-bit permutation rows

`cargo run -p sparq-core --example bench_id_width --release` — sort / full-scan /
100k-probe kernels over identical row data at both widths, olympics scale
(1.8M) and 10M rows.

(pending)

### 3c. Compressed on-disk size

Same example: SPQCPRM1-style delta+varint size for u32-valued ids vs u64 ids
with tag bit 60.

(pending)

## 4. Decision

(pending)
