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

/// Sorts a chunk (by its stored column order) and spills it to a fresh run file under
/// `tmp`, clearing the buffer. The caller pushes triples already in the desired key
/// order, so a plain lexicographic sort suffices.
pub fn spill_run(buf: &mut Vec<[Id; 3]>, runs: &mut Vec<PathBuf>, tmp: &Path) -> io::Result<()> {
    if buf.is_empty() {
        return Ok(());
    }
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
            unsafe { memmap2::Mmap::map(&f) }
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
        }
    }
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
    let n = m.len() / TRIPLE_BYTES;
    Ok((m, n))
}
