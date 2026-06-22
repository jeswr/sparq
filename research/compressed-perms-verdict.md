# Compressed permutation indexes — verdict (T4)

**Adopted.** Block-wise frame-of-reference + varint compression for the on-disk
permutation indexes, behind a magic-header format version, with the old raw format
still loading unchanged. The lazy-block-mmap mode (v2) is **also adopted** — the
measured perm compression ratio is 2.5–2.75x, clearing the >2x bar set for it.

## Format

Written by `Graph::save_compressed` / `TripleStore::save_compressed` (CLI:
`save <data> <format> <dir> compressed`, or `recompress <src> <dst>` to compact an
existing raw dir without re-parsing). The external-memory `build` can also emit this
format DIRECTLY from its sort/merge tail — `SPARQ_BUILD_COMPRESSED=1 sparq-cli build …`
(sq-vkz7) — skipping the separate `open` + `decode_all` + re-encode recompress pass over
the whole index (a full pass over an 84+ GB index at 1B scale). The sibling perms are
written compressed straight from `extsort::external_sort_compressed`'s merge; SPO is kept
raw until the siblings re-sort from it, then re-encoded last. Raw stays the build DEFAULT;
the one-pass output is byte-identical to a raw build followed by `recompress` (guarded by
`sparq-engine/tests/compressed_build_differential.rs`). Per `perm{i}.bin`, all
little-endian:

```text
magic[8] = "SPQCPRM1" | len u64 | n_blocks u64 | blocks_len u64
directory: n_blocks × { first_row [u32;3], byte_off u32 }     (one per 128 rows)
blocks:    blocks_len bytes — per block: count varint, first row varint-encoded,
           then per row the delta from the previous row (shared-prefix skip +
           varint deltas) — exactly the in-memory `compress::CompressedPerm`
           block stream, so a mapped file is served with no transcode
```

**Version / detection.** The magic prefix is the format version byte(s):
`TripleStore::open` auto-detects per file — `SPQCPRM1` ⇒ compressed, anything else
⇒ the original raw `[u32;3]` rows. A raw file cannot collide: its first 4 bytes are
the smallest id in the perm (ids are dense from 1; the magic would require
≈1.13e9). Old directories load untouched (asserted in tests), and the two formats
can be mixed per file within one directory.

**Serving modes.**
- *Lazy (default on open):* only the sparse directory is decoded to the heap
  (16 B / 128 rows ≈ 0.13 B/triple); scans decode blocks off the mapped file —
  the out-of-core mode, resident set ≈ page cache only.
- *Eager:* `Graph::decompress_indexes` (CLI: `bench-mmap … decompress`) decodes
  all perms to raw RAM once; scans are then zero-copy slice borrows, identical to
  a raw store.

All of it is `mmap`-feature-gated: the wasm build (default-features = false) is
unaffected (1,554,189 B vs the 597afc1 baseline 1,554,093 B — +96 B, +0.006%).

## Measured

Fixtures: bench/qlever-olympics (1,781,625 triples) and bench/qlever-synthetic
(9,999,991 triples), Apple Silicon, release build, min-of-N per query
(`bench-mmap … count`), suite totals below.

### Bytes/triple (on disk)

| dataset | raw perms | compressed perms | ratio | raw dir total | compressed dir total | dir ratio |
|---|---|---|---|---|---|---|
| olympics 1.78M | 128.28 MB = **72.0 B/t** | 46.59 MB = **26.2 B/t** | **2.75x** | 159.5 MB (89.5 B/t) | 77.8 MB (43.7 B/t) | 2.05x |
| synthetic 10M | 720.00 MB = **72.0 B/t** | 291.62 MB = **29.2 B/t** | **2.47x** | 840.7 MB (84.1 B/t) | 412.3 MB (41.2 B/t) | 2.04x |

(The dictionary/numerics files are unchanged by this work; the dir-total ratio is
diluted by them.)

### Load time

Open is mmap-bound and noisy (±0.1s); representative numbers:

| dataset | raw open | compressed open (lazy) | eager decode on top |
|---|---|---|---|
| olympics | 0.08–0.27 s | 0.13–0.30 s | +0.15–0.32 s (heap → 0.129 GB) |
| synthetic 10M | 1.07 s | 1.24 s | +2.39 s (heap → 0.720 GB) |

Lazy open ≈ raw open (the only extra work is decoding the ~0.13 B/triple
directory). Eager decode runs at ≈25 M rows/s across the six perms.

### Query latency (suite totals, min-of-N, count mode)

| dataset | raw mmap | compressed lazy | compressed eager |
|---|---|---|---|
| olympics (10 queries, 5 iters) | 73.4 ms | 106.8 ms (**1.46x**) | 75.5 ms (**1.03x**) |
| synthetic 10M (6 queries, 3 iters) | 108.6 ms | 339.4 ms (**3.12x**) | 125.3 ms (**1.15x**) |

Lazy per-query slowdown is 1.5–3x on scan/join-heavy queries (olympics q03–q10:
1.75–3.4x; synthetic q03 2.4x, q04 2.9x, q10 5.9x) and can be much worse on
queries raw answers by pure zero-copy range borrow (synthetic q06 selective
filter: 15 µs → 19.8 ms). Hash-join-dominated queries are barely affected
(olympics q07 ≈ parity). Eager is raw-parity (within noise; worst case synthetic
q04 +22%).

## Adopted vs rejected

- **Adopted: compressed on-disk format (FoR + varint, SPQCPRM1).** 2.5–2.75x
  smaller perms, ~2x smaller directories, no load-time regression, full
  backward compat.
- **Adopted: v2 lazy-block-mmap.** Ratio 2.47–2.75x clears the >2x bar. It is the
  natural serving mode for an opened compressed dir (heap ≈ 0; the block stream
  never leaves the page cache) — the right default for out-of-core use. Cost:
  1.5–3x query latency on scan-heavy work.
- **Adopted: opt-in load-time decompression** (`decompress_indexes`) for the
  RAM-rich case: pay one ~25 M rows/s decode at open, query at raw speed. This,
  not lazy, is what a latency-sensitive resident server should use — disk shrinks
  2.5x either way.
- **Rejected: making compressed the default `save` format.** `save` stays raw:
  raw is byte-for-byte mmap-able with zero decode anywhere on the hot path, and
  the external-memory `build` path writes raw runs it merges in place.
  `recompress` converts a built dir afterwards.
- **Rejected (not pursued): heavier entropy coding (e.g. zstd-on-blocks).** The
  block stream must stay random-accessible per 128-row block for lazy serving;
  varint deltas already capture most of the redundancy (the remaining dir-total
  bytes are dictionary, out of scope here — see dict-compression-measured.md).

## Test/guard status

- `cargo test --workspace`: 407 passed, 0 failed.
- `cargo test -p sparq-core --features mmap`: 27 passed (includes the new
  store-level roundtrip across all 64 pattern shapes × 4 sort orders ×
  {lazy, eager, predstats-fallback} and the graph-level roundtrip incl. a
  delta-overlay folded into the compressed save).
- wasm guard: builds; 1,554,189 B (+96 B vs baseline).
