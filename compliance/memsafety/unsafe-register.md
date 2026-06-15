<!-- [OPUS-4.8] sq-toze.6 / cert gap GX-5 (epic sq-toze). Authored while Fable
     unavailable — re-review when Fable returns. -->

# `unsafe` Rust justification register

**Certification gap GX-5** (production-hardening / memsafety framework + ASVS).
Modelled on the `unsafe-rust-attestation` pattern and the prod-solid-server
compliance discipline: every `unsafe` site in the workspace's first-party crates is
enumerated here with the invariant it relies on, why that invariant holds, and how it
is tested or bounded. The register is kept at **100% coverage** mechanically by
`scripts/unsafe-gate.py` (the unsafe-count **ratchet**) — see
[§ Ratchet](#ratchet--how-this-stays-honest).

> This register covers **first-party `unsafe` only** (the `crates/` tree). Third-party
> `unsafe` (e.g. inside `memmap2`, `libc`, `rayon`, the `hdt` dependency) is out of
> scope here and is governed by the supply-chain lane (`cargo-deny`, SBOM/VEX, PR #210)
> and the upstream crates' own audits.

## Scope & threat model

The load-bearing boundary is **B5** in [`research/threat-model.md`](../../research/threat-model.md):
a *hostile on-disk index file → mmap loader → `unsafe` pointer reinterpret*. The
register distinguishes two trust classes of `unsafe`:

- **Trusted-input `unsafe`** — the pointer/slice/byte reinterprets whose backing was
  produced by this process (a `Vec`/slice we just built, or a file we wrote and own for
  the call's lifetime). Soundness rests on type/POD + alignment + length invariants
  that hold by construction.
- **Untrusted-input `unsafe`** (the B5 surface) — the mmap loaders that map a `.spq` /
  `dict-*.bin` / `.spqv` / DiskANN file that may be **hostile or corrupt**. These are
  **validated at open** before any unchecked access: `Dict::open_mmap → MappedDict::validate`
  (sq-znld), `VectorStore::open_validated`, and the DiskANN `open` validator bound every
  offset/length, and the hot read path uses bounds-/UTF-8-**checked** accessors
  (`rd_str_checked`) — the `from_utf8_unchecked` fast paths are reachable **only** for
  records this process built or already validated.

### How each site is bounded (test / lint coverage)

| Mechanism | Covers |
|---|---|
| `miri` lane (`.github/workflows/miri.yml`, sq-fo28) | pure-Rust unsafe in `sparq-core` reachable without `mmap`/`dict-spill`: the parallel scatter writes, POD↔bytes reinterprets, `from_utf8_unchecked` over in-memory buffers, `MaybeUninit`+`set_len`. Runs nightly under Tree-Borrows for aliasing/provenance/data-race UB. |
| `mmap_corruption_oracle` test (`crates/sparq-core/tests/`, run under `--features mmap,dict-spill`) + `fuzz` lane's mmap-loader target | the **B5** mmap sites Miri structurally cannot run (file-backed mappings): hostile/corrupt index → loader must reject or stay in-bounds, never UB. |
| `#![forbid(unsafe_code)]` on 20 crates (sq-emay) | proves the unsafe surface is *confined* to the 5 crates below; a new `unsafe` anywhere else fails to compile. |
| `scripts/unsafe-gate.py --check` (this PR, GX-5) | the count **ratchet** — a PR cannot add `unsafe` without updating this register + re-seeding the snapshot. |
| `// SAFETY:` / adjacent-comment argument on every site (CONTRIBUTING) | the per-site argument lives next to the code. Enforcement is the **register + review + the count ratchet** (a new `unsafe` cannot land without a register row, a source comment, and a re-seed). `clippy::undocumented_unsafe_blocks` (which would mechanically require the literal `// SAFETY:` token) is **recommended but NOT yet enabled on the first-party unsafe crates** — tracked as gap MS-G2 in [`gap-register.md`](./gap-register.md). [OPUS-4.8] |

## Register

**56 `unsafe` sites** across 5 crates (the other 20 crates are `#![forbid(unsafe_code)]`).
Counts and the file:line list are produced by `scripts/unsafe-gate.py --list` and
must equal `bench/unsafe-snapshot.json`.

Recurring invariant shorthands used below:
- **POD-bytes** — `[u32;3]` / `u32` / `u64` / `f64` are plain-old-data with no invalid
  bit patterns; reinterpreting `&[T] ↔ &[u8]` only reads/writes valid bytes; `size_of_val`
  bounds the length exactly.
- **page-align** — an mmap base is page-aligned (≥ any of the 4/8-byte scalar alignments),
  and the file is a whole number of fixed-width records, so `from_raw_parts(base.cast::<T>(), len/size_of::<T>())` is aligned and in-bounds.
- **own-for-lifetime** — the mapped/opened file is owned by the structure for the
  borrow's duration and is not mutated by us; external concurrent mutation is explicitly
  out of contract (documented stance, same as the rest of the mmap surface).

### `sparq-core` — 42 sites (the unsafe core: mmap loaders, zero-copy dict, parallel build)

| File:line | Kind | Invariant relied on | Why sound / how bounded |
|---|---|---|---|
| `src/lib.rs:278` | slice reinterpret (read) | page-align; `numerics.bin` is whole f64 | mmap base ≥ 8-byte f64 align; `n = len/8`. Mapped-file open validates size == `dict.len()*8`. |
| `src/lib.rs:382` | ptr read | page-align; instant section is `n` f64 at offset 0 | `i < n` checked at the call; f64 at `base+i`. |
| `src/lib.rs:449` | slice reinterpret (read) | page-align; instants are `n` f64 at offset 0 | `n = mapped_len`; materialises the cells. |
| `src/lib.rs:570` | slice reinterpret (write) | POD-bytes | reinterpret the f64 column as bytes to write `temporals.bin`. |
| `src/lib.rs:1387` | slice reinterpret (read) | page-align; `temporals.bin` starts with `len` f64 | `len = mapped_len`; flags read separately after. |
| `src/lib.rs:1438` | `Mmap::map` | own-for-lifetime | `numerics.bin` opened only if `size == dict.len()*8` (length pre-validated). |
| `src/lib.rs:1446` | `Mmap::map` | own-for-lifetime | `temporals.bin` opened only if `size == dict.len()*9` (length pre-validated). |
| `src/lib.rs:1723` | slice reinterpret (read) | page-align; perm0 is whole `[u32;3]` rows | `n` from `map_perm`; map outlives the loop. Written by us above. |
| `src/lib.rs:2116` | slice reinterpret (read) | page-align; perm0 is whole `[u32;3]` rows | same as 1723 (external-build path). |
| `src/lib.rs:3444` | slice reinterpret (write) | POD-bytes | reinterpret the f64 numerics cache as bytes to write. |
| `src/lib.rs:3488` | `_mm_prefetch` (x86_64) | hint-only | prefetch is defined for any address; the hint is dropped on a bad one — cannot fault. |
| `src/lib.rs:3493` | `prfm` asm (aarch64) | hint-only | `prfm pldl1keep` is a hint; `nostack, preserves_flags`; cannot fault or write memory/regs. |
| `src/lib.rs:3548` | ptr `add` (prefetch arg) | `id-1 < remap.len()` for every dict id | only computes an address for the hint-only `prefetch_read`; never dereferenced here. |
| `src/lib.rs:4235` | ptr `add` (prefetch arg) | `id-1 < remap.len()` | same as 3548 (other build path). |
| `src/lib.rs:4392` | `MmapMut::map_mut` | own-for-lifetime; freshly-written perm file | read-write map of a perm file of whole `[u32;3]` rows we just wrote. |
| `src/lib.rs:4393` | mut slice reinterpret | page-align; POD-bytes | `len/4` u32; exclusively owned (the `MmapMut`); rayon writes are disjoint by index. |
| `src/lib.rs:6229` | `std::env::set_var` | single-threaded test; var restored before return | TEST-only (`#[test] external_quads_fd_*`): sets `SPARQ_QUADS_SPILL_MAX_OPEN` to exercise the bounded-writer-pool path; no other thread reads the env in the test. Edition-2024 made `set_var` `unsafe`. |
| `src/lib.rs:6231` | `std::env::remove_var` | same test; restores the env | TEST-only: removes the var set at 6229 before returning so the process env is left clean. Edition-2024 `unsafe`. |
| `src/store.rs:106` | slice reinterpret (read) | page-align; whole `[u32;3]` triples | `n = len/12`; mmap base ≥ 4-byte u32 align. |
| `src/store.rs:375` | slice reinterpret (write) | POD-bytes | reinterpret contiguous `[u32;3]` rows as bytes for `std::fs::write`. |
| `src/store.rs:459` | `Mmap::map` | own-for-lifetime | per-permutation file; empty (size 0) skipped; format auto-detected after. **B5**. |
| `src/dict.rs:483` | `from_utf8_unchecked` | bytes came from a `&str` we wrote / a validated dict | TRUSTED path only — the untrusted mmap path uses `rd_str_checked` and is validated at open (sq-znld). |
| `src/dict.rs:734` | slice reinterpret (read) | page-align; LE u64 array | `len/8`; mmap base ≥ 8. **B5** (validated at open). |
| `src/dict.rs:747` | slice reinterpret (read) | page-align; LE u32 array | `len/4`; mmap base ≥ 4. **B5** (validated at open). |
| `src/dict.rs:1675` | `Mmap::map` | own-for-lifetime | read-only map of a dict section file. **B5** (`MappedDict::validate` runs after). |
| `src/dict.rs:1946` | slice reinterpret (write) | POD-bytes (`T: Copy`, u32/u64) | `write_pod_slice` reinterprets the POD array as bytes for `fs::write`. |
| `src/dict.rs:2192` | `unsafe impl Send for SlotPtr` | `SlotPtr` (`*mut Id`) only ever touches its own disjoint slot | the parallel scatter routes each `(shard,id)` slot to exactly one shard — no aliasing, no reads until the scope ends. |
| `src/dict.rs:2193` | `unsafe impl Sync for SlotPtr` | same as 2192 | shared read of `Copy` raw-pointer handles; the writes they perform are disjoint. |
| `src/dict.rs:2211` | ptr `write` (scatter) | `i < len`; slot owned by this shard | bounded by `assert!`/`debug_assert!` on `lid<stride` & `i<len`; covered by `miri`. |
| `src/dict.rs:2410` | `Vec::set_len` | all `total` slots initialised exactly once | the shard slices partition `[0,total)` and each fills its slice fully; covered by `miri`. |
| `src/dictspill.rs:119` | `sysconf` FFI | pure query | `_SC_PHYS_PAGES`/`_SC_PAGE_SIZE`; negative ⇒ "unsupported", handled. |
| `src/dictspill.rs:139` | `statvfs` FFI | zeroed out-param + NUL-terminated path; checked return | `statvfs` is a plain C struct (sound to zero-init); path from a validated `CString`; return checked. |
| `src/dictspill.rs:223` | `from_utf8_unchecked` | bytes written from a `&str` in `serialize_termparts` | round-trips our own serialisation — never untrusted input. |
| `src/dictspill.rs:228` | `from_utf8_unchecked` | same as 223 (the `rest` tail) | our own serialisation. |
| `src/dictspill.rs:720` | `unsafe impl Send for SlotPtr` | `*mut u64` only touches its own hash-routed slot | disjoint writes, no reads until the parallel scope ends. |
| `src/dictspill.rs:721` | `unsafe impl Sync for SlotPtr` | same as 720 | `Copy` handle shared; writes disjoint. |
| `src/dictspill.rs:736` | ptr `write` (scatter) | `i < len`; slot hash-routed to this shard | `debug_assert!(i<len)`; disjoint by hash routing. (`dict-spill` feature ⇒ outside the `miri` lane; covered by the deterministic corruption oracle + fuzz.) |
| `src/extsort.rs:15` | slice reinterpret (write) | POD-bytes | `as_bytes` over `[Id;3]` for writing a run. |
| `src/extsort.rs:38` | `unchecked_advise_range` | read-only map, range already consumed | `DontNeed`/`FreeReusable` only drops the resident copy of clean file pages; never read again. |
| `src/extsort.rs:83` | `Mmap::map` | own-for-lifetime | run files written by us, not mutated during the merge. |
| `src/extsort.rs:98` | slice reinterpret (read) | page-align; whole `[u32;3]` triples | `n = len/12` per run. |
| `src/extsort.rs:180` | `Mmap::map` | own-for-lifetime | `map_perm` read-only map of a perm file we own for the call. |

### `sparq-vectors` — 9 sites (aligned vector blobs + mmap'd `.spqv` / DiskANN)

| File:line | Kind | Invariant relied on | Why sound / how bounded |
|---|---|---|---|
| `src/store.rs:80` | mut slice reinterpret | `words` is u32-aligned, holds ≥ `len` bytes; region exclusively owned | `AlignedBytes` over-allocates to a word boundary; `copy_from_slice` fills it. |
| `src/store.rs:88` | slice reinterpret (read) | u32-aligned, ≥ `len` initialised bytes | reads the bytes copied in above; base ≥ 4-byte aligned by construction. |
| `src/store.rs:190` | `Mmap::map` | own-for-lifetime | read-only map of a `.spqv` file; then `open_validated` bounds every offset. **B5**. |
| `src/store.rs:348` | slice reinterpret (write) | f32 has no invalid bit patterns; `align(f32) ≥ align(u8)` | f32→LE bytes for write; big-endian targets rejected at create/open. |
| `src/store.rs:488` | slice reinterpret (read) | `start` is a multiple of 4 ⇒ f32-aligned; range validated in `open` | the backing is u32-aligned (review 1874 fixed a UB align bug here); f32 accepts any bit pattern. **B5**. |
| `src/store.rs:611` | slice reinterpret (write) | f32 no invalid patterns; align ok | f32→LE bytes for write. |
| `src/diskann.rs:395` | slice reinterpret (read) | f32 no invalid patterns; align ok | f32→LE bytes; LE target asserted; borrows `b.vectors`. |
| `src/diskann.rs:509` | `Mmap::map` | own-for-lifetime | read-only map of a DiskANN file; `open` validates after. **B5**. |
| `src/diskann.rs:596` | slice reinterpret (read) | page-align; `start` a multiple of 4 ⇒ f32-aligned; range validated in `open` | `debug_assert_eq!` checks alignment; f32 accepts any bit pattern; borrows the map. **B5**. |

### `sparq-cli` — 2 sites (the `dump-perm` debug command)

| File:line | Kind | Invariant relied on | Why sound / how bounded |
|---|---|---|---|
| `src/main.rs:425` | `Mmap::map` | own-for-lifetime | read-only map of a perm file held open for the call. |
| `src/main.rs:428` | slice reinterpret (read) | page-align; whole `[u32;3]` rows | `n = len/12`; `n==0` handled. CLI utility over a file the operator named. |

### `sparq-zk-compose` — 2 sites (cross-process advisory file lock)

| File:line | Kind | Invariant relied on | Why sound / how bounded |
|---|---|---|---|
| `src/verifier.rs:871` | `libc::flock(LOCK_EX)` | `fd` is a valid open fd owned by `file` for the call | the `MutexGuard` keeps `file` (hence `fd`) alive; an error fails closed (`return false`). |
| `src/verifier.rs:879` | `libc::flock(LOCK_UN)` | same valid, locked fd | unlock helper run on every return path so the advisory lock is never leaked (a leak would deadlock the next caller). |

### `sparq-bench` — 1 site (peak-RSS measurement; non-shipping bench binary)

| File:line | Kind | Invariant relied on | Why sound / how bounded |
|---|---|---|---|
| `src/main.rs:257` | `getrusage` FFI | `rusage` is a plain C struct (sound to zero-init); `getrusage(RUSAGE_SELF, &mut ru)` writes only into the valid out-param | return checked before any field read. Bench-only binary, not shipped. |

## NEEDS-REVIEW

**None.** Every one of the 56 sites has a clear, sound safety argument in source: 50
via the literal `// SAFETY:` token, and 6 via an adjacent justification block comment
(the `from_utf8_unchecked` TRUSTED fast path `dict.rs:483`, and the two `unsafe impl
Send`/`Sync for SlotPtr` pairs `dict.rs:2192-93` + `dictspill.rs:720-21`). Normalising
those 6 to the literal token — and enabling `clippy::undocumented_unsafe_blocks` to keep
them there — is gap MS-G2. Should a future site lack any argument, mark it
`NEEDS-REVIEW` here and open a bead (`bd create`) rather than fabricating a
justification — do **not** re-seed the snapshot over an unjustified `unsafe`.

## Ratchet — how this stays honest

`scripts/unsafe-gate.py` counts the first-party `unsafe` sites per crate (the same
comment-/string-stripped keyword scan used to build this table) and compares against the
committed snapshot `bench/unsafe-snapshot.json`:

```sh
scripts/unsafe-gate.py --check        # FAIL if any crate rose above its snapshot
scripts/unsafe-gate.py --seed         # re-seed AFTER a reviewed register update
scripts/unsafe-gate.py --list         # file:line:text of every counted site
```

The **`unsafe-register (count ratchet)`** CI lane (`.github/workflows/ci.yml`) runs
`--check` on every PR and merge-queue ref. Because it does **not** contain the word
"advisory"/"informational", the `ci-summary / gate` aggregator treats it as a
**required** (gating) check — distinct from the pre-existing non-gating
`unsafe report (cargo-geiger, informational)` lane, which stays as a visibility-only
report. A PR that adds an `unsafe` site therefore fails CI until the author adds a
register row here, a `// SAFETY:` comment in source, and re-seeds the snapshot — all
three changes land in the same reviewable diff.
