//! [OPUS-4.8] sq-2te — HDT WRITE support: serialise a sparq [`Graph`] to an `.hdt`
//! archive (the mirror of [`crate::load`]).
//!
//! ## The write path (and its cost — read this before optimising)
//!
//! The wrapped `hdt` crate (0.4) can BUILD an archive — but only through
//! [`hdt::Hdt::read_nt`], whose 0.4 signature takes a **file path** (`&Path`) and
//! reads N-Triples from it; the reader-based variant is commented out upstream. It
//! then serialises with [`hdt::Hdt::write`]. Both live behind the crate's
//! experimental `sophia` feature, which this crate's `write` feature turns on.
//!
//! So the only tractable path on hdt 0.4 is a **temporary N-Triples round-trip**:
//!
//! 1. scan the sparq `Graph` and render every triple as an N-Triples line into a
//!    temp file (terms are rendered through oxrdf's canonical N-Triples `Display`,
//!    the same rendering the round-trip tests compare against);
//! 2. `Hdt::read_nt(temp)` re-parses that text and builds the FourSectDict (Plain
//!    Front Coding) + BitmapTriples (SPO) structures — re-interning every term and
//!    re-sorting the triples it already had in `Graph` form;
//! 3. `Hdt::write` serialises that to the output (compressing if the caller asked
//!    for a `.hdt.gz` / `.hdt.zst` / `.hdt.bz2` container).
//!
//! **Cost.** This is NOT free: step 1 materialises the whole graph as N-Triples
//! TEXT on disk (roughly the size of the equivalent `.nt`), and step 2 re-parses
//! and re-interns it — work sparq already did once on ingest. For a graph that
//! came from `.hdt` and is being re-saved this is a full text decode + re-encode.
//! It is correct and uses only the maintained upstream writer; it is the LOW/EASY
//! path from bead sq-2te. The faster path (sq-2te path B / the queued upstream
//! contribution — see `UPSTREAM.md`) is a direct in-memory `FourSectDict` +
//! `BitmapTriples` builder fed from sparq's id space, skipping the text round-trip
//! entirely. That is deliberately deferred: it means vendoring the inverse of
//! `decode.rs` (PFC + BitmapTriples ENCODERS), which is a separate, larger unit.
//!
//! The temp file is created under the system temp dir with a unique name and
//! removed on success and on every error path (RAII guard).

use crate::Error;
use sparq_core::Graph;
use std::io::Write;
use std::path::Path;

/// The compression container to wrap the written `.hdt` bytes in, chosen by the
/// OUTPUT path's extension (the write side, unlike the read side, cannot sniff
/// content that does not exist yet — so it honours the file name the caller chose).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutContainer {
    /// `.hdt.gz`
    Gzip,
    /// `.hdt.zst`
    Zstd,
    /// `.hdt.bz2`
    Bzip2,
    /// bare `.hdt` (any other extension)
    None,
}

/// Picks the output container from the path's final extension. Recognises the same
/// `.gz` / `.zst` / `.bz2` suffixes the reader sniffs by magic bytes; anything else
/// (including a bare `.hdt`) writes an uncompressed archive.
fn out_container(path: &Path) -> OutContainer {
    match path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() {
        Some("gz") => OutContainer::Gzip,
        Some("zst") | Some("zstd") => OutContainer::Zstd,
        Some("bz2") => OutContainer::Bzip2,
        _ => OutContainer::None,
    }
}

/// A temp file removed on drop (success OR error / panic), so a failed write never
/// leaves an N-Triples scratch file behind.
struct TempNt {
    path: std::path::PathBuf,
}

impl TempNt {
    /// Creates a uniquely-named scratch path in the system temp dir. The name mixes
    /// the pid and a process-monotonic counter so concurrent `save`s never collide.
    fn new() -> std::io::Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let name = format!("sparq-hdt-save-{}-{n}.nt", std::process::id());
        Ok(TempNt { path: std::env::temp_dir().join(name) })
    }
}

impl Drop for TempNt {
    fn drop(&mut self) {
        // Best-effort; the file is in the temp dir and named uniquely, so a stray
        // remove failure (e.g. already gone) is not worth surfacing.
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Writes the sparq `Graph` to `w` as N-Triples (one `S P O .` line per stored
/// triple), rendering each term through oxrdf's canonical N-Triples `Display`.
///
/// This is the sparq-side serialiser the write path feeds to `Hdt::read_nt`; it is
/// `pub(crate)` so it is also unit-testable. The scan walks the store ONCE in its
/// natural order (no sort): HDT's `read_nt` re-sorts into SPO itself.
pub(crate) fn write_ntriples<W: Write>(graph: &Graph, w: &mut W) -> std::io::Result<()> {
    let scan = graph.store.scan(&[None, None, None]);
    for row in scan.rows.iter() {
        let [s, p, o] = scan.to_spo(row);
        // oxrdf `Term`/`NamedNode`/`BlankNode`/`Literal` Display == N-Triples term
        // syntax (the exact rendering `tests/roundtrip.rs::triple_set` compares),
        // so `S P O .` is a valid N-Triples line.
        writeln!(w, "{} {} {} .", graph.dict.term(s), graph.dict.term(p), graph.dict.term(o))?;
    }
    Ok(())
}

/// Saves a sparq [`Graph`] as an HDT archive at `path`.
///
/// The on-disk layout is the standard HDT v1.0 (FourSectionDictionary with Plain
/// Front Coding + BitmapTriples in SPO order) written by the wrapped `hdt` crate —
/// the same layout [`crate::load`] reads, and the one hdt-cpp / hdt-java emit, so
/// the result is interoperable. If `path` ends in `.gz` / `.zst` / `.bz2` the
/// archive bytes are wrapped in that container (matching the reader's magic-byte
/// sniffing); any other extension writes a bare `.hdt`.
///
/// HDT carries a SINGLE default graph, so only the default graph's triples are
/// written; any named graphs on `graph` are ignored (HDT has no place for them).
///
/// COST: this serialises the graph to a temporary N-Triples file and re-parses it
/// through the upstream builder — see the module docs. Requires the `write` feature.
pub fn save(graph: &Graph, path: impl AsRef<Path>) -> Result<(), Error> {
    let path = path.as_ref();

    // 1. Render the graph to a temp N-Triples file (removed on drop, any outcome).
    let tmp = TempNt::new()?;
    {
        let mut nt = std::io::BufWriter::new(std::fs::File::create(&tmp.path)?);
        write_ntriples(graph, &mut nt)?;
        nt.flush()?;
    }

    // 2. Build the HDT structures from that N-Triples text (FourSectDict + BitmapTriples).
    let hdt = hdt::Hdt::read_nt(&tmp.path)?;

    // 3. Serialise to `path`, wrapping in the requested compression container.
    //    `Hdt::write` takes any `impl Write`, so the compressing encoders chain in
    //    directly — the bare `.hdt` bytes are never fully materialised in memory.
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    match out_container(path) {
        OutContainer::None => {
            let mut w = file;
            hdt.write(&mut w)?;
            w.flush()?;
        }
        OutContainer::Gzip => {
            let mut w = flate2::write::GzEncoder::new(file, flate2::Compression::default());
            hdt.write(&mut w)?;
            w.finish()?.flush()?;
        }
        OutContainer::Zstd => {
            let mut w = zstd::stream::write::Encoder::new(file, 3).map_err(Error::Io)?.auto_finish();
            hdt.write(&mut w)?;
            w.flush()?;
        }
        OutContainer::Bzip2 => {
            let mut w = bzip2::write::BzEncoder::new(file, bzip2::Compression::default());
            hdt.write(&mut w)?;
            w.finish()?.flush()?;
        }
    }
    // The temp NT file is removed when `tmp` drops here. (`Hdt::read_nt` /
    // `Hdt::write` return `hdt::hdt::Error`, which `?` converts via the existing
    // `impl From<hdt::hdt::Error> for Error` in lib.rs.)
    Ok(())
}
