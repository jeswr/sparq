//! External-memory (out-of-core) index BUILD: sort/merge the triples on disk so a
//! dataset whose permutation indexes exceed RAM can be CONSTRUCTED with bounded memory
//! (only one chunk of triples + the dictionary live in RAM at once). Complements the
//! memory-mapped query path (`Graph::open`). Native only (`mmap` feature).

use crate::dict::Id;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

const TRIPLE_BYTES: usize = std::mem::size_of::<[Id; 3]>(); // 12

/// Reinterprets a triple slice as raw little-endian bytes for writing.
fn as_bytes(t: &[[Id; 3]]) -> &[u8] {
    // SAFETY: [u32;3] is plain-old-data; we only read it as bytes.
    unsafe { std::slice::from_raw_parts(t.as_ptr().cast::<u8>(), std::mem::size_of_val(t)) }
}

/// Advise the kernel to RECLAIM the `[offset, offset+len)` page range of a read-only,
/// front-to-back-consumed mmap (drop-behind), keeping merge-input residency bounded.
/// Linux: `MADV_DONTNEED` discards the resident file pages (re-faulted from the clean
/// file if ever touched again — which we never do). macOS: `MADV_DONTNEED` does NOT
/// reduce RSS for file-backed maps, so use `MADV_FREE_REUSABLE`, the Darwin idiom that
/// actually returns the pages. Both are advisory and unmapped-range-safe; failures are
/// ignored. The map is read-only and the range is already fully consumed, so no data
/// can be lost — the `unchecked_advise` contract is satisfied.
// [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns
#[cfg(unix)]
fn drop_behind(map: &memmap2::Mmap, offset: usize, len: usize) {
    if len == 0 {
        return;
    }
    #[cfg(target_vendor = "apple")]
    let advice = memmap2::UncheckedAdvice::FreeReusable;
    #[cfg(not(target_vendor = "apple"))]
    let advice = memmap2::UncheckedAdvice::DontNeed;
    // SAFETY: read-only mapping, range already consumed (never read again); DontNeed/
    // FreeReusable on clean file pages only drops the resident copy.
    unsafe {
        map.unchecked_advise_range(advice, offset, len).ok();
    }
}

#[cfg(not(unix))]
fn drop_behind(_map: &memmap2::Mmap, _offset: usize, _len: usize) {}

/// Sorts a chunk (by its stored column order) and spills it to a fresh run file under
/// `tmp`, clearing the buffer. The caller pushes triples already in the desired key
/// order, so a plain lexicographic sort suffices.
pub fn spill_run(buf: &mut Vec<[Id; 3]>, runs: &mut Vec<PathBuf>, tmp: &Path) -> io::Result<()> {
    if buf.is_empty() {
        return Ok(());
    }
    // Parallel sort when available: a run is up to `chunk` (tens of millions of) triples,
    // and this sort previously sat SERIALLY inside the ingest spill stage and each
    // sibling-perm external sort. `[u32;3]` is total-ordered and equal triples are
    // byte-identical, so the run file is byte-identical to the serial `sort_unstable`.
    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        buf.par_sort_unstable();
    }
    #[cfg(not(feature = "parallel"))]
    buf.sort_unstable();
    let path = tmp.join(format!("run{}.bin", runs.len()));
    std::fs::write(&path, as_bytes(buf))?;
    runs.push(path);
    buf.clear();
    Ok(())
}

/// K-way merges sorted run files into `out` (deduplicating consecutive equal triples).
/// The runs are memory-mapped (paged on demand) and merged through a min-heap, so peak
/// RAM is the heap (one triple per run) — independent of the dataset size.
pub fn kway_merge(runs: &[PathBuf], out: &Path) -> io::Result<()> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    let maps: Vec<memmap2::Mmap> = runs
        .iter()
        .map(|p| {
            let f = std::fs::File::open(p)?;
            // SAFETY: the run files are written by us and not mutated during the merge.
            let m = unsafe { memmap2::Mmap::map(&f) }?;
            // Each run is consumed front-to-back exactly once: tell the kernel so it
            // reads ahead and DROPS BEHIND instead of letting every merged byte sit
            // resident — at 100M+ triples the merge inputs are GBs of file pages that
            // otherwise dominate peak RSS (advisory; failures are ignorable).
            #[cfg(unix)]
            m.advise(memmap2::Advice::Sequential).ok();
            Ok(m)
        })
        .collect::<io::Result<_>>()?;
    let slices: Vec<&[[Id; 3]]> = maps
        .iter()
        .map(|m| {
            let n = m.len() / TRIPLE_BYTES;
            // SAFETY: each run is a whole number of page-aligned [u32;3] triples.
            unsafe { std::slice::from_raw_parts(m.as_ptr().cast::<[Id; 3]>(), n) }
        })
        .collect();

    let mut heads = vec![0usize; runs.len()];
    let mut heap: BinaryHeap<Reverse<([Id; 3], usize)>> = BinaryHeap::new();
    for (i, s) in slices.iter().enumerate() {
        if !s.is_empty() {
            heap.push(Reverse((s[0], i)));
        }
    }

    // DROP-BEHIND: each run is consumed front-to-back exactly once, so pages behind the
    // merge cursor are dead — release them in 32 MiB strides (`MADV_DONTNEED`; advisory)
    // instead of letting the whole merge input (GBs at 100M+ triples) sit resident.
    // `Advice::Sequential` alone does NOT shrink residency on macOS, measured: the
    // sibling-merge phase peaked at 5.7 GiB on a 100M-triple build without this.
    const DROP_STRIDE: usize = 32 << 20;
    let mut dropped = vec![0usize; runs.len()];

    let mut w = BufWriter::new(std::fs::File::create(out)?);
    let mut last: Option<[Id; 3]> = None;
    while let Some(Reverse((t, i))) = heap.pop() {
        if last != Some(t) {
            for &id in &t {
                w.write_all(&id.to_le_bytes())?;
            }
            last = Some(t);
        }
        heads[i] += 1;
        if heads[i] < slices[i].len() {
            heap.push(Reverse((slices[i][heads[i]], i)));
            #[cfg(unix)]
            {
                let consumed = heads[i] * TRIPLE_BYTES;
                if consumed - dropped[i] >= 2 * DROP_STRIDE {
                    // Keep one stride of headroom behind the cursor; offsets stay
                    // page-aligned because DROP_STRIDE is a multiple of the page size.
                    let upto = consumed - DROP_STRIDE;
                    drop_behind(&maps[i], dropped[i], upto - dropped[i]);
                    dropped[i] = upto;
                }
            }
        }
    }
    #[cfg(not(unix))]
    let _ = &mut dropped;
    w.flush()
}

/// External-sorts the triples produced by `iter` into key order `order` (a column
/// permutation), writing the sorted result to `out` using at most `chunk` triples of
/// RAM at a time. The runs are written under `tmp`.
pub fn external_sort(
    iter: impl Iterator<Item = [Id; 3]>,
    order: [usize; 3],
    out: &Path,
    tmp: &Path,
    chunk: usize,
) -> io::Result<()> {
    let mut runs: Vec<PathBuf> = Vec::new();
    let mut buf: Vec<[Id; 3]> = Vec::with_capacity(chunk);
    for t in iter {
        // Store each triple already in `order` so the run sort + merge compare directly.
        buf.push([t[order[0]], t[order[1]], t[order[2]]]);
        if buf.len() >= chunk {
            spill_run(&mut buf, &mut runs, tmp)?;
        }
    }
    spill_run(&mut buf, &mut runs, tmp)?;
    kway_merge(&runs, out)?;
    for r in &runs {
        std::fs::remove_file(r).ok();
    }
    Ok(())
}

/// Memory-maps a permutation file as a `[[Id;3]]` slice (for re-sorting into other
/// orders during the build).
pub fn map_perm(path: &Path) -> io::Result<(memmap2::Mmap, usize)> {
    let f = std::fs::File::open(path)?;
    // SAFETY: read-only mapping of a file we own for the call's duration.
    let m = unsafe { memmap2::Mmap::map(&f)? };
    // The sibling re-sorts stream this front-to-back (one pass each): prefer read-ahead
    // + drop-behind over keeping the whole permutation resident (advisory).
    #[cfg(unix)]
    m.advise(memmap2::Advice::Sequential).ok();
    let n = m.len() / TRIPLE_BYTES;
    Ok((m, n))
}
