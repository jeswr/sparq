//! sparq-core: dictionary-encoded RDF storage with six permutation indexes.
//!
//! This is the storage substrate for the query engine: a [`Graph`] holds the
//! term [`Dict`]ionary and the [`TripleStore`] (six sorted permutations), and
//! is built from an RDF document via the bulk loader.

pub mod compress;
pub mod dict;
#[cfg(feature = "mmap")]
pub mod extsort;
mod nt;
pub mod store;

use dict::{Dict, Id};
use oxrdf::vocab::xsd;
use oxrdf::{Literal, NamedNode, Term};
use oxttl::{NQuadsParser, NTriplesParser, TriGParser, TurtleParser};
use store::{Pattern, TripleStore};

/// An immutable, dictionary-encoded RDF graph ready for querying.
pub struct Graph {
    pub dict: Dict,
    pub store: TripleStore,
    /// Parallel to the dictionary: the f64 value of each numeric literal (NaN for
    /// non-numeric terms). Lets the engine evaluate numeric filters / comparisons
    /// / ORDER BY without materialising the term and parsing its string each time
    /// — a lightweight, u32-id-preserving stand-in for QLever's inline ValueIds.
    numerics: NumData,
}

/// Backing storage for the numeric-value cache (`numerics[id-1]` = f64 value of term
/// `id`, NaN for non-numeric): owned dense in RAM, mmap'd from disk (out-of-core), or
/// SPARSE — only the numeric terms in a hash map. Most RDF terms are IRIs/strings (NaN),
/// and small integers inline (carrying their own value, never cached), so the dense
/// cache is mostly — often entirely — NaN; the sparse form stores only the few real
/// numeric literals, the right shape for the memory-bound browser store.
enum NumData {
    Owned(Vec<f64>),
    #[cfg(feature = "mmap")]
    Mapped(memmap2::Mmap),
    Sparse(rustc_hash::FxHashMap<Id, f64>),
}

impl NumData {
    /// The cached numeric value of a 1-based dictionary id, or `None` if it is not a
    /// (cached) numeric literal. The engine's O(1) numeric fast path.
    #[inline]
    fn lookup(&self, id: Id) -> Option<f64> {
        match self {
            NumData::Sparse(m) => m.get(&id).copied(),
            _ => {
                let v = *self.as_slice().get((id - 1) as usize)?;
                if v.is_nan() {
                    None
                } else {
                    Some(v)
                }
            }
        }
    }

    /// The dense f64 slice — valid only for the Owned/Mapped backings (the ones `save`
    /// persists); the sparse backing is never written to disk.
    #[inline]
    fn as_slice(&self) -> &[f64] {
        match self {
            NumData::Owned(v) => v,
            #[cfg(feature = "mmap")]
            NumData::Mapped(m) => {
                let n = m.len() / std::mem::size_of::<f64>();
                // SAFETY: numerics.bin is a whole number of f64; the mmap base is
                // page-aligned (>= the 8-byte f64 alignment).
                unsafe { std::slice::from_raw_parts(m.as_ptr().cast::<f64>(), n) }
            }
            NumData::Sparse(_) => unreachable!("as_slice on a sparse numeric cache"),
        }
    }

    /// Resident heap bytes (a memory-mapped cache contributes 0 — it is page cache).
    #[inline]
    fn heap_bytes(&self) -> usize {
        match self {
            NumData::Owned(v) => v.capacity() * std::mem::size_of::<f64>(),
            #[cfg(feature = "mmap")]
            NumData::Mapped(_) => 0,
            // hashbrown: ~(8-byte key + 8-byte f64 + 1 control byte) per slot.
            NumData::Sparse(m) => m.capacity() * 17,
        }
    }

    /// Converts a dense cache into the sparse form when it is mostly NaN (≥ 3/4 of terms
    /// non-numeric — almost always true), keeping only the real numeric literals. A no-op
    /// (kept dense) when numeric values are common enough that the map would not save.
    fn into_sparse_if_worthwhile(self) -> NumData {
        let dense = match &self {
            NumData::Owned(v) => v,
            _ => return self, // mmap'd/already-sparse: leave as is
        };
        let numeric = dense.iter().filter(|x| !x.is_nan()).count();
        if numeric * 4 > dense.len() {
            return self; // numeric-dense: the Vec is the better representation
        }
        let mut m: rustc_hash::FxHashMap<Id, f64> = rustc_hash::FxHashMap::default();
        m.reserve(numeric);
        for (i, &v) in dense.iter().enumerate() {
            if !v.is_nan() {
                m.insert(i as Id + 1, v);
            }
        }
        NumData::Sparse(m)
    }
}

/// The f64 value of a term if it is a numeric XSD literal, else NaN.
fn numeric_of(term: &Term) -> f64 {
    match term {
        Term::Literal(l) if is_numeric_dt(l) => l.value().parse::<f64>().unwrap_or(f64::NAN),
        _ => f64::NAN,
    }
}

/// True for `xsd:integer` and its derived integer subtypes (NOT decimal/double/float) —
/// the datatypes whose values are exact integers (parseable as `i128`).
pub fn is_integer_datatype(dt: &str) -> bool {
    dt == xsd::INTEGER.as_str()
        || dt == xsd::LONG.as_str()
        || dt == xsd::INT.as_str()
        || dt == xsd::SHORT.as_str()
        || dt == xsd::BYTE.as_str()
        || dt == xsd::NON_NEGATIVE_INTEGER.as_str()
        || dt == xsd::POSITIVE_INTEGER.as_str()
        || dt == xsd::NON_POSITIVE_INTEGER.as_str()
        || dt == xsd::NEGATIVE_INTEGER.as_str()
        || dt == xsd::UNSIGNED_INT.as_str()
        || dt == xsd::UNSIGNED_LONG.as_str()
        || dt == xsd::UNSIGNED_SHORT.as_str()
        || dt == xsd::UNSIGNED_BYTE.as_str()
}

fn is_numeric_dt(l: &Literal) -> bool {
    let dt = l.datatype().as_str();
    dt == xsd::INTEGER.as_str()
        || dt == xsd::DECIMAL.as_str()
        || dt == xsd::DOUBLE.as_str()
        || dt == xsd::FLOAT.as_str()
        || dt == xsd::LONG.as_str()
        || dt == xsd::INT.as_str()
        || dt == xsd::SHORT.as_str()
        || dt == xsd::BYTE.as_str()
        || dt == xsd::NON_NEGATIVE_INTEGER.as_str()
        || dt == xsd::POSITIVE_INTEGER.as_str()
        || dt == xsd::NON_POSITIVE_INTEGER.as_str()
        || dt == xsd::NEGATIVE_INTEGER.as_str()
        || dt == xsd::UNSIGNED_INT.as_str()
        || dt == xsd::UNSIGNED_LONG.as_str()
        || dt == xsd::UNSIGNED_SHORT.as_str()
        || dt == xsd::UNSIGNED_BYTE.as_str()
}

impl Graph {
    /// Loads triples from an RDF document (default graph only for M1; named
    /// graphs from TriG/N-Quads are folded into the default graph). Returns the
    /// built graph. `format`: "turtle" | "ntriples" | "nquads" | "trig".
    pub fn load_str(text: &str, format: &str) -> Result<Graph, String> {
        let mut dict = Dict::new();
        let mut triples: Vec<[Id; 3]> = Vec::new();
        let bytes = text.as_bytes();

        macro_rules! push_triple {
            ($s:expr, $p:expr, $o:expr) => {{
                let s = dict.intern(&subject_term($s));
                let p = dict.intern(&Term::NamedNode($p.clone()));
                let o = dict.intern($o);
                triples.push([s, p, o]);
            }};
        }

        match format {
            "ntriples" | "n-triples" => {
                // N-Triples is one statement per line, so the input can be split at
                // newline boundaries and parsed + interned in parallel (each thread
                // builds a partial dictionary, then the partials are merged).
                #[cfg(feature = "parallel")]
                {
                    let (d, t) = parse_ntriples_parallel(bytes)?;
                    return Ok(Self::build(d, t));
                }
                #[cfg(not(feature = "parallel"))]
                {
                    let mut d = Dict::new();
                    let t = nt::parse_chunk(bytes, &mut d)?;
                    return Ok(Self::build(d, t));
                }
            }
            "nquads" | "n-quads" => {
                for q in NQuadsParser::new().for_slice(bytes) {
                    let q = q.map_err(|e| e.to_string())?;
                    push_triple!(&q.subject, &q.predicate, &q.object);
                }
            }
            "trig" | "application/trig" => {
                for q in TriGParser::new().for_slice(bytes) {
                    let q = q.map_err(|e| e.to_string())?;
                    push_triple!(&q.subject, &q.predicate, &q.object);
                }
            }
            _ => {
                for t in TurtleParser::new().for_slice(bytes) {
                    let t = t.map_err(|e| e.to_string())?;
                    push_triple!(&t.subject, &t.predicate, &t.object);
                }
            }
        }

        Ok(Self::build(dict, triples))
    }

    /// Streaming loader: parses an RDF document incrementally from a reader (so a
    /// gzip / bzip2 decompression stream can be ingested without holding the whole
    /// document in memory). Same formats as [`load_str`](Self::load_str). The
    /// dictionary and triple buffer still grow in memory, so the full store only
    /// fits for datasets that fit in RAM; the streaming ingest *throughput*,
    /// however, is measurable on arbitrarily large inputs (see `sparq-cli ingest`).
    pub fn load_reader<R: std::io::Read>(reader: R, format: &str) -> Result<Graph, String> {
        let mut dict = Dict::new();
        let mut triples: Vec<[Id; 3]> = Vec::new();

        macro_rules! push_triple {
            ($s:expr, $p:expr, $o:expr) => {{
                let s = dict.intern(&subject_term($s));
                let p = dict.intern(&Term::NamedNode($p.clone()));
                let o = dict.intern($o);
                triples.push([s, p, o]);
            }};
        }

        match format {
            "nquads" | "n-quads" => {
                for q in NQuadsParser::new().for_reader(reader) {
                    let q = q.map_err(|e| e.to_string())?;
                    push_triple!(&q.subject, &q.predicate, &q.object);
                }
            }
            "trig" | "application/trig" => {
                for q in TriGParser::new().for_reader(reader) {
                    let q = q.map_err(|e| e.to_string())?;
                    push_triple!(&q.subject, &q.predicate, &q.object);
                }
            }
            "turtle" | "ttl" => {
                for t in TurtleParser::new().for_reader(reader) {
                    let t = t.map_err(|e| e.to_string())?;
                    push_triple!(&t.subject, &t.predicate, &t.object);
                }
            }
            _ => {
                for t in NTriplesParser::new().for_reader(reader) {
                    let t = t.map_err(|e| e.to_string())?;
                    push_triple!(&t.subject, &t.predicate, &t.object);
                }
            }
        }

        Ok(Self::build(dict, triples))
    }

    /// Builds the store + numeric cache from interned triples (shared by the
    /// string and streaming loaders).
    fn build(dict: Dict, triples: Vec<[Id; 3]>) -> Graph {
        let store = TripleStore::from_triples(triples);
        let numerics = NumData::Owned(numerics_of(&dict));
        Graph { dict, store, numerics }
    }

    /// Like [`load_str`](Self::load_str) but stores the permutation indexes
    /// BLOCK-COMPRESSED (~4-6 B/triple vs 12) — the memory-bound build for the browser,
    /// trading a bounded per-scan decode for ~2.5x more triples per byte of RAM. Query
    /// results are identical to the raw build.
    pub fn load_str_compressed(text: &str, format: &str) -> Result<Graph, String> {
        let g = Self::load_str(text, format)?;
        Ok(g.into_compressed())
    }

    /// Re-encodes into the memory-bound storage mode: the permutations BLOCK-COMPRESSED and
    /// the dictionary's id→term storage compacted to a single BLOB (no per-term `Box<str>`).
    /// Keeps the numeric cache and term ids. The browser/RAM-constrained path; identical
    /// query results, a small per-scan decode.
    pub fn into_compressed(self) -> Graph {
        let triples: Vec<[Id; 3]> = {
            let scan = self.store.scan(&[None, None, None]);
            scan.rows.iter().map(|r| scan.to_spo(r)).collect()
        };
        Graph {
            store: TripleStore::from_triples_compressed(triples),
            dict: self.dict.into_blob(),
            // The numeric cache is mostly (often entirely) NaN — keep only the real
            // numeric literals when sparse, freeing the dense f64-per-term Vec.
            numerics: self.numerics.into_sparse_if_worthwhile(),
        }
    }

    /// Persists the graph to `dir` (the permutation indexes + the dictionary) so it can
    /// later be QUERIED with the indexes MEMORY-MAPPED via [`open`](Self::open) — the
    /// out-of-core path for datasets larger than RAM.
    #[cfg(feature = "mmap")]
    pub fn save(&self, dir: &std::path::Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        self.store.save(dir)?;
        self.dict.save_mmap(dir)?;
        write_numerics(&dir.join("numerics.bin"), self.numerics.as_slice())
    }

    /// Opens a graph saved by [`save`](Self::save) with its permutation indexes AND
    /// numeric-value cache MEMORY-MAPPED (paged in on demand) — so a large out-of-core
    /// dataset opens near-instantly without re-parsing every term, and the cache stays
    /// off the heap. The dictionary is loaded into RAM; the big indexes stay on disk. If
    /// `numerics.bin` is absent or stale (a graph saved before this cache existed), the
    /// cache is recomputed, preserving backward compatibility.
    #[cfg(feature = "mmap")]
    pub fn open(dir: &std::path::Path) -> std::io::Result<Graph> {
        let store = TripleStore::open(dir)?;
        let dict = Dict::open_mmap(dir)?;
        let np = dir.join("numerics.bin");
        let numerics = match std::fs::File::open(&np) {
            Ok(f) if f.metadata()?.len() as usize == dict.len() * std::mem::size_of::<f64>() => {
                // SAFETY: the file is owned by this graph for its lifetime and not mutated.
                NumData::Mapped(unsafe { memmap2::Mmap::map(&f)? })
            }
            _ => NumData::Owned(numerics_of(&dict)),
        };
        Ok(Graph { dict, store, numerics })
    }

    /// EXTERNAL-MEMORY build: streams an RDF document and writes the on-disk permutation
    /// indexes + dictionary directly, sorting the triples through disk-backed runs so the
    /// dataset's indexes can be CONSTRUCTED without ever holding them all in RAM. Only one
    /// `chunk`-sized buffer of triples (plus the growing dictionary) is resident at a time;
    /// the rest lives in sorted run files that are k-way merged. The result is identical to
    /// `save()` of an in-memory `load`, but bounded-memory — the billion-scale ingest path.
    /// Open it with [`open`](Self::open).
    ///
    /// `chunk` is the number of triples per in-memory run (e.g. 8_000_000 ≈ 96 MB of ids).
    #[cfg(feature = "mmap")]
    pub fn build_external<R: std::io::Read + Send>(
        reader: R,
        format: &str,
        dir: &std::path::Path,
        chunk: usize,
    ) -> Result<(), String> {
        use store::{Perm, BUILT};
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let tmp = dir.join("tmp");
        std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;

        let mut dict = Dict::new();
        let mut buf: Vec<[Id; 3]> = Vec::with_capacity(chunk);
        let mut runs: Vec<std::path::PathBuf> = Vec::new();

        // Stream-parse + intern, spilling SPO-sorted runs to disk whenever the buffer fills.
        macro_rules! push_triple {
            ($s:expr, $p:expr, $o:expr) => {{
                let s = dict.intern(&subject_term($s));
                let p = dict.intern(&Term::NamedNode($p.clone()));
                let o = dict.intern($o);
                buf.push([s, p, o]);
                if buf.len() >= chunk {
                    extsort::spill_run(&mut buf, &mut runs, &tmp).map_err(|e| e.to_string())?;
                }
            }};
        }
        match format {
            "nquads" | "n-quads" => {
                for q in NQuadsParser::new().for_reader(reader) {
                    let q = q.map_err(|e| e.to_string())?;
                    push_triple!(&q.subject, &q.predicate, &q.object);
                }
            }
            "trig" | "application/trig" => {
                for q in TriGParser::new().for_reader(reader) {
                    let q = q.map_err(|e| e.to_string())?;
                    push_triple!(&q.subject, &q.predicate, &q.object);
                }
            }
            "turtle" | "ttl" => {
                for t in TurtleParser::new().for_reader(reader) {
                    let t = t.map_err(|e| e.to_string())?;
                    push_triple!(&t.subject, &t.predicate, &t.object);
                }
            }
            _ => {
                // N-Triples is the billion-scale bulk format: parse it with the custom
                // byte parser over PARALLEL buffers (per-buffer partial dicts merged into
                // the running dict), the user-requested "parallelise parsing of the file".
                // Still bounded-memory: one ~64 MiB buffer + its partials at a time.
                #[cfg(feature = "parallel")]
                build_external_ntriples_parallel(reader, &mut dict, &mut buf, &mut runs, &tmp, chunk)?;
                #[cfg(not(feature = "parallel"))]
                for t in NTriplesParser::new().for_reader(reader) {
                    let t = t.map_err(|e| e.to_string())?;
                    push_triple!(&t.subject, &t.predicate, &t.object);
                }
            }
        }
        extsort::spill_run(&mut buf, &mut runs, &tmp).map_err(|e| e.to_string())?;
        buf.shrink_to_fit();

        // Merge the SPO runs into the SPO permutation file (deduplicating).
        let spo_path = dir.join(format!("perm{}.bin", Perm::Spo as usize));
        extsort::kway_merge(&runs, &spo_path).map_err(|e| e.to_string())?;
        for r in &runs {
            std::fs::remove_file(r).ok();
        }

        // Build the other BUILT permutations by external-sorting the SPO file into each
        // order (re-reading it memory-mapped, so this stays bounded-memory too).
        let (map, n) = extsort::map_perm(&spo_path).map_err(|e| e.to_string())?;
        // SAFETY: perm0 is a whole number of [u32;3] rows written above; map outlives the loop.
        let spo: &[[Id; 3]] =
            unsafe { std::slice::from_raw_parts(map.as_ptr().cast::<[Id; 3]>(), n) };
        for &perm in BUILT.iter().filter(|&&p| p != Perm::Spo) {
            let out = dir.join(format!("perm{}.bin", perm as usize));
            extsort::external_sort(spo.iter().copied(), perm.order(), &out, &tmp, chunk)
                .map_err(|e| e.to_string())?;
        }
        drop(map);

        // Empty files for the unbuilt permutations so `open` finds all six slots.
        for i in 0..6 {
            let p = dir.join(format!("perm{i}.bin"));
            if !p.exists() {
                std::fs::File::create(&p).map_err(|e| e.to_string())?;
            }
        }

        dict.save_mmap(dir).map_err(|e| e.to_string())?;
        // Persist the numeric-value cache so `open` mmaps it instead of recomputing.
        write_numerics(&dir.join("numerics.bin"), &numerics_of(&dict)).map_err(|e| e.to_string())?;
        // Compute per-predicate stats once (a one-time POS/PSO scan) and persist them so
        // query-open never re-scans those indexes — keeping out-of-core open fast + small.
        let store = TripleStore::open(dir).map_err(|e| e.to_string())?;
        store.save_pred_stats(dir).map_err(|e| e.to_string())?;
        std::fs::remove_dir_all(&tmp).ok();
        Ok(())
    }

    /// The numeric value of a term id, or `None` if it is not a numeric literal.
    /// O(1), no allocation — the engine's fast path for numeric filters. An inline
    /// integer id carries its value directly (no lookup); other ids use the cache.
    #[inline]
    pub fn numeric_value(&self, id: Id) -> Option<f64> {
        if dict::is_inline(id) {
            return Some((id - dict::INLINE_BASE) as f64);
        }
        self.numerics.lookup(id)
    }

    /// The lexical form of a term id IF it is an exact-valued numeric literal (an
    /// `xsd:integer` subtype or `xsd:decimal` — NOT float/double, whose value IS its f64).
    /// Used to disambiguate comparisons that the f64 fast path collapses (integers > 2^53,
    /// high-precision decimals); only reached when the f64 values compared equal, so the
    /// allocation is rare. Inline-integer ids format their value directly.
    pub fn exact_numeric_lexical(&self, id: Id) -> Option<String> {
        if dict::is_inline(id) {
            return Some((id - dict::INLINE_BASE).to_string());
        }
        match self.dict.term_parts(id) {
            dict::TermParts::Lit { value, datatype, lang: None }
                if is_integer_datatype(datatype) || datatype == xsd::DECIMAL.as_str() =>
            {
                Some(value.to_string())
            }
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// A rough estimate of the graph's in-memory footprint in bytes (dictionary +
    /// the six permutation indexes), for benchmarking.
    pub fn heap_bytes(&self) -> usize {
        self.dict.heap_bytes() + self.store.heap_bytes() + self.numerics.heap_bytes()
    }

    /// Resolves a term to its id, or `None` if the term is absent (so a pattern
    /// bound to it cannot match).
    pub fn id_of(&self, term: &Term) -> Option<Id> {
        let id = self.dict.lookup(term);
        if id == dict::NO_ID {
            None
        } else {
            Some(id)
        }
    }

    /// Builds an id pattern from optional terms; returns `None` if any bound
    /// term is absent from the dictionary.
    pub fn pattern(
        &self,
        s: Option<&Term>,
        p: Option<&NamedNode>,
        o: Option<&Term>,
    ) -> Option<Pattern> {
        let resolve = |t: Option<&Term>| -> Option<Option<Id>> {
            match t {
                None => Some(None),
                Some(t) => self.id_of(t).map(Some),
            }
        };
        let s = resolve(s)?;
        let p = match p {
            None => None,
            Some(n) => Some(self.id_of(&Term::NamedNode(n.clone()))?),
        };
        let o = resolve(o)?;
        Some([s, p, o])
    }
}

/// The numeric-value cache for a dictionary: `numerics[id-1]` is the f64 value of term
/// `id` (NaN for non-numeric). Parallel when the `parallel` feature is on.
fn numerics_of(dict: &Dict) -> Vec<f64> {
    let n = dict.len();
    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        (0..n).into_par_iter().map(|i| numeric_of(&dict.term(i as Id + 1))).collect()
    }
    #[cfg(not(feature = "parallel"))]
    {
        (0..n).map(|i| numeric_of(&dict.term(i as Id + 1))).collect()
    }
}

/// Writes the numeric-value cache to disk (raw little-endian f64) so it can be
/// memory-mapped on open instead of recomputed.
#[cfg(feature = "mmap")]
fn write_numerics(path: &std::path::Path, nums: &[f64]) -> std::io::Result<()> {
    // SAFETY: reinterpret the contiguous f64 cache as bytes for writing.
    let bytes = unsafe { std::slice::from_raw_parts(nums.as_ptr().cast::<u8>(), std::mem::size_of_val(nums)) };
    std::fs::write(path, bytes)
}

fn subject_term(s: &oxrdf::NamedOrBlankNode) -> Term {
    match s {
        oxrdf::NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n.clone()),
        oxrdf::NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b.clone()),
    }
}

/// Splits a byte buffer into ~`target` ranges, each ending on a newline so no
/// (single-line) N-Triples statement is split across a boundary.
#[cfg(feature = "parallel")]
fn newline_chunk_bounds(bytes: &[u8], target: usize) -> Vec<(usize, usize)> {
    let mut bounds = Vec::with_capacity(target);
    let chunk = (bytes.len() / target.max(1)).max(1);
    let mut start = 0;
    while start < bytes.len() {
        let mut end = (start + chunk).min(bytes.len());
        if end < bytes.len() {
            match bytes[end..].iter().position(|&b| b == b'\n') {
                Some(p) => end += p + 1,
                None => end = bytes.len(),
            }
        }
        bounds.push((start, end));
        start = end;
    }
    bounds
}

/// Parses + interns N-Triples in parallel: each chunk builds a partial dictionary +
/// local-id triples, then the partials are merged into one global dictionary with the
/// local ids remapped. Interning is per-thread (no shared lock); the merge is linear.
#[cfg(feature = "parallel")]
fn parse_ntriples_parallel(bytes: &[u8]) -> Result<(Dict, Vec<[Id; 3]>), String> {
    use rayon::prelude::*;
    // A few chunks per thread for load balancing (terms are not uniformly dense).
    let target = (rayon::current_num_threads().max(1) * 4).min(bytes.len() / 4096 + 1);
    let bounds = newline_chunk_bounds(bytes, target);

    let partials: Vec<(Dict, Vec<[Id; 3]>)> = bounds
        .par_iter()
        .map(|&(s, e)| {
            let mut dict = Dict::new();
            let triples = nt::parse_chunk(&bytes[s..e], &mut dict)?;
            Ok::<_, String>((dict, triples))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let total: usize = partials.iter().map(|(_, t)| t.len()).sum();
    let cap = partials.iter().map(|(d, _)| d.len()).max().unwrap_or(0);
    let mut global = Dict::with_capacity(cap);
    let mut all = Vec::with_capacity(total);
    for (pd, ptriples) in partials {
        let remap = global.merge_remap(&pd);
        // Dictionary ids are 1-based and below INLINE_BASE; inline-integer ids carry
        // their value and pass through unchanged.
        let map = |id: Id| {
            if id >= dict::INLINE_BASE {
                id
            } else {
                remap[(id - 1) as usize]
            }
        };
        all.extend(ptriples.into_iter().map(|[s, p, o]| [map(s), map(p), map(o)]));
    }
    Ok((global, all))
}

/// Streams N-Triples from `reader` in newline-aligned ~64 MiB blocks, parsing+interning
/// each block IN PARALLEL (the custom byte parser, per-block partial dicts merged into
/// the running `dict`), and spilling SPO runs — the parallel-parse path for the external
/// (billion-scale, bounded-memory) build.
///
/// DECOMPRESSION is PIPELINED onto its own thread feeding a bounded channel, so it OVERLAPS
/// parsing+spilling instead of running additively. For a `.bz2` ingest — where the (slow,
/// single-stream) decompress dominates wall-time — this hides the parse cost under the
/// decompress, the largest measured ingest win. At most a few 64 MiB blocks are in flight,
/// so memory stays bounded.
#[cfg(all(feature = "mmap", feature = "parallel"))]
fn build_external_ntriples_parallel<R: std::io::Read + Send>(
    reader: R,
    dict: &mut Dict,
    buf: &mut Vec<[Id; 3]>,
    runs: &mut Vec<std::path::PathBuf>,
    tmp: &std::path::Path,
    chunk: usize,
) -> Result<(), String> {
    use std::sync::mpsc::sync_channel;
    const BLOCK: usize = 64 << 20; // 64 MiB
    let (tx, rx) = sync_channel::<Vec<u8>>(3);

    std::thread::scope(|scope| -> Result<(), String> {
        // Producer: decompress + read on its own thread, emitting newline-aligned blocks.
        let producer = scope.spawn(move || -> Result<(), String> {
            let mut reader = reader;
            let mut readbuf = vec![0u8; BLOCK];
            let mut carry: Vec<u8> = Vec::new();
            loop {
                // Fill the read buffer (a single read may return less than requested).
                let mut filled = 0;
                while filled < BLOCK {
                    let n = reader.read(&mut readbuf[filled..]).map_err(|e| e.to_string())?;
                    if n == 0 {
                        break;
                    }
                    filled += n;
                }
                if filled == 0 {
                    // EOF: a final line without a trailing newline lives in `carry`.
                    if !carry.is_empty() {
                        let _ = tx.send(std::mem::take(&mut carry));
                    }
                    return Ok(());
                }
                // Emit `carry + readbuf[..filled]` up to the last newline; carry the
                // remainder (a partial line split across the read boundary) to the next.
                let mut block = std::mem::take(&mut carry);
                block.extend_from_slice(&readbuf[..filled]);
                let cut = block.iter().rposition(|&b| b == b'\n').map_or(0, |p| p + 1);
                carry = block[cut..].to_vec();
                block.truncate(cut);
                if tx.send(block).is_err() {
                    return Ok(()); // the consumer hit an error and dropped the receiver
                }
            }
        });

        // Consumer: parse+intern+spill each block (itself rayon-parallel) as it arrives.
        for block in rx {
            parse_block_into(&block, dict, buf, runs, tmp, chunk)?;
        }
        // Surface a decompression error (the producer ends → channel closes → loop above
        // exits cleanly, and the error is here).
        producer.join().map_err(|_| "decompression thread panicked".to_string())?
    })
}

/// Parses one (complete-line) N-Triples byte block in parallel, merges each chunk's
/// partial dictionary into `dict`, and spills SPO runs as `buf` fills.
#[cfg(all(feature = "mmap", feature = "parallel"))]
fn parse_block_into(
    bytes: &[u8],
    dict: &mut Dict,
    buf: &mut Vec<[Id; 3]>,
    runs: &mut Vec<std::path::PathBuf>,
    tmp: &std::path::Path,
    chunk: usize,
) -> Result<(), String> {
    use rayon::prelude::*;
    if bytes.is_empty() {
        return Ok(());
    }
    let target = (rayon::current_num_threads().max(1) * 4).min(bytes.len() / 4096 + 1);
    let bounds = newline_chunk_bounds(bytes, target);
    let partials: Vec<(Dict, Vec<[Id; 3]>)> = bounds
        .par_iter()
        .map(|&(s, e)| {
            let mut d = Dict::new();
            let t = nt::parse_chunk(&bytes[s..e], &mut d)?;
            Ok::<_, String>((d, t))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for (pd, ptriples) in partials {
        let remap = dict.merge_remap(&pd);
        let map = |id: Id| if id >= dict::INLINE_BASE { id } else { remap[(id - 1) as usize] };
        for [s, p, o] in ptriples {
            buf.push([map(s), map(p), map(o)]);
            if buf.len() >= chunk {
                extsort::spill_run(buf, runs, tmp).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "mmap")]
    #[test]
    fn save_open_mmap_roundtrip() {
        // Build a graph, persist it, re-open with the indexes memory-mapped, and check
        // the store + dictionary are structurally identical (every triple round-trips).
        let mut nt = String::new();
        for i in 0..3000u32 {
            nt.push_str(&format!(
                "<http://ex/n{}> <http://ex/p{}> \"{}\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n",
                i % 211,
                i % 13,
                i % 500
            ));
        }
        nt.push_str("<http://ex/n0> <http://ex/name> \"caf\\u00e9\"@fr .\n");
        let g = Graph::load_str(&nt, "ntriples").unwrap();
        let dir = std::env::temp_dir().join(format!("sparq_mmap_test_{}", std::process::id()));
        g.save(&dir).unwrap();
        let g2 = Graph::open(&dir).unwrap();
        assert_eq!(g.len(), g2.len());
        assert_eq!(g.dict.len(), g2.dict.len());
        let dump = |gg: &Graph| {
            let scan = gg.store.scan(&[None, None, None]);
            let mut v: Vec<(String, String, String)> = scan
                .rows
                .iter()
                .map(|r| {
                    let spo = scan.to_spo(r);
                    (gg.dict.term(spo[0]).to_string(), gg.dict.term(spo[1]).to_string(), gg.dict.term(spo[2]).to_string())
                })
                .collect();
            v.sort();
            v
        };
        assert_eq!(dump(&g), dump(&g2), "mmap-reopened store differs");

        // The numeric-value cache must round-trip through its memory-mapped form: every
        // numeric literal resolves to the same f64 (and non-numerics to None) as before.
        assert!(dir.join("numerics.bin").exists(), "numerics cache not persisted");
        assert!(matches!(g2.numerics, NumData::Mapped(_)), "numerics not mmap'd on open");
        for v in [0u32, 1, 42, 250, 499] {
            let lit = Term::Literal(Literal::new_typed_literal(v.to_string(), xsd::INTEGER));
            if let Some(id) = g.id_of(&lit) {
                assert_eq!(g.numeric_value(id), Some(v as f64));
                assert_eq!(g2.numeric_value(id), Some(v as f64), "mmap'd numeric differs for {v}");
            }
        }
        // A non-numeric term (the language-tagged literal) must be None in both.
        if let Some(id) = g.id_of(&Term::Literal(Literal::new_language_tagged_literal("café", "fr").unwrap())) {
            assert_eq!(g2.numeric_value(id), None);
        }
        // The dictionary is memory-mapped (zero resident term storage) and lookup still
        // round-trips: every term resolves to the same id and back to the same term.
        assert!(dir.join("dict-terms.bin").exists(), "mmap dict not persisted");
        for s in 0..50u32 {
            let t = Term::NamedNode(NamedNode::new_unchecked(format!("http://ex/n{}", s % 211)));
            assert_eq!(g.id_of(&t), g2.id_of(&t), "mmap dict lookup differs for {t}");
        }
        // Per-predicate stats are persisted (no POS/PSO re-scan on open) and identical.
        assert!(dir.join("predstats.bin").exists(), "pred stats not persisted");
        for p in 0..13u32 {
            let pred = NamedNode::new_unchecked(format!("http://ex/p{p}"));
            if let Some(pid) = g.id_of(&Term::NamedNode(pred)) {
                assert_eq!(g.store.pred_stat(pid), g2.store.pred_stat(pid), "pred_stat differs for p{p}");
            }
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(feature = "mmap")]
    #[test]
    fn build_external_matches_in_memory() {
        // External-memory build with a TINY chunk so the triples spill across many runs
        // and exercise the k-way merge + per-permutation re-sort. The on-disk result must
        // be byte-for-byte identical to an in-memory load → save (same dedup, same order).
        let mut nt = String::new();
        for i in 0..5000u32 {
            nt.push_str(&format!(
                "<http://ex/n{}> <http://ex/p{}> \"{}\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n",
                i % 173,
                i % 7,
                i % 400
            ));
        }
        // A duplicate line (must be deduped) + a non-integer literal with a language tag.
        nt.push_str("<http://ex/n0> <http://ex/p0> \"0\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n");
        nt.push_str("<http://ex/n0> <http://ex/name> \"caf\\u00e9\"@fr .\n");

        let base = std::env::temp_dir().join(format!("sparq_ext_{}", std::process::id()));
        let mem_dir = base.join("mem");
        let ext_dir = base.join("ext");

        // In-memory load → save (reference), vs streaming external build (chunk = 256).
        let g = Graph::load_str(&nt, "ntriples").unwrap();
        g.save(&mem_dir).unwrap();
        Graph::build_external(nt.as_bytes(), "ntriples", &ext_dir, 256).unwrap();

        let mem = Graph::open(&mem_dir).unwrap();
        let ext = Graph::open(&ext_dir).unwrap();
        assert_eq!(mem.len(), ext.len(), "triple count differs");
        assert_eq!(mem.dict.len(), ext.dict.len(), "dict size differs");

        // Every BUILT permutation file must be byte-identical (same sort, same dedup).
        for &perm in store::BUILT {
            let f = format!("perm{}.bin", perm as usize);
            let a = std::fs::read(mem_dir.join(&f)).unwrap();
            let b = std::fs::read(ext_dir.join(&f)).unwrap();
            assert_eq!(a, b, "permutation {f} differs between in-memory and external build");
        }

        // And the data round-trips through terms.
        let dump = |gg: &Graph| {
            let scan = gg.store.scan(&[None, None, None]);
            let mut v: Vec<(String, String, String)> = scan
                .rows
                .iter()
                .map(|r| {
                    let spo = scan.to_spo(r);
                    (gg.dict.term(spo[0]).to_string(), gg.dict.term(spo[1]).to_string(), gg.dict.term(spo[2]).to_string())
                })
                .collect();
            v.sort();
            v
        };
        assert_eq!(dump(&mem), dump(&ext), "external-built store differs");
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn ntriples_parallel_matches_sequential() {
        // >4KB so the input spans multiple parallel chunks; subjects/predicates repeat
        // across chunks (exercising the partial-dict merge) and the objects are inline
        // integers (exercising the inline-id passthrough in the remap).
        let mut nt = String::new();
        for i in 0..2000u32 {
            nt.push_str(&format!(
                "<http://ex/n{}> <http://ex/p{}> \"{}\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n",
                i % 137,
                i % 11,
                i % 500
            ));
        }
        // The byte-level parser's risky paths, cross-checked against oxttl: escapes
        // (quote / backslash / newline / \\u), language tags, typed + simple literals,
        // a comment, and a different IRI namespace.
        nt.push_str("# a comment line\n");
        nt.push_str("<http://ex/s> <http://other.org/p> \"a \\\"q\\\" b\\nc \\\\ d \\u00e9\" .\n");
        nt.push_str("<http://ex/s> <http://ex/name> \"caf\\u00e9\"@fr .\n");
        nt.push_str("<http://ex/s> <http://ex/v> \"1.5\"^^<http://www.w3.org/2001/XMLSchema#decimal> .\n");
        nt.push_str("<http://ex/s> <http://ex/plain> \"just a string\" .\n");
        nt.push_str("<http://ex/s> <http://ex/big> \"\\U0001F600 grin\" .\n");
        let par = Graph::load_str(&nt, "ntriples").unwrap(); // parallel (when feature on)
        let seq = Graph::load_reader(nt.as_bytes(), "ntriples").unwrap(); // sequential
        assert_eq!(par.len(), seq.len());
        assert_eq!(par.dict.len(), seq.dict.len());
        // Full structural equality independent of id-assignment order: map every stored
        // triple back to its terms, sort, compare.
        let dump = |g: &Graph| {
            let scan = g.store.scan(&[None, None, None]);
            let mut v: Vec<(String, String, String)> = scan
                .rows
                .iter()
                .map(|r| {
                    let spo = scan.to_spo(r);
                    (g.dict.term(spo[0]).to_string(), g.dict.term(spo[1]).to_string(), g.dict.term(spo[2]).to_string())
                })
                .collect();
            v.sort();
            v
        };
        assert_eq!(dump(&par), dump(&seq));
    }

    #[test]
    fn load_and_scan() {
        let ttl = "@prefix ex: <http://ex/> . ex:a ex:p ex:b, ex:c . ex:d ex:p ex:b .";
        let g = Graph::load_str(ttl, "turtle").unwrap();
        assert_eq!(g.len(), 3);

        let p = NamedNode::new("http://ex/p").unwrap();
        let b = Term::NamedNode(NamedNode::new("http://ex/b").unwrap());
        // ?s ex:p ex:b  -> ex:a and ex:d
        let pat = g.pattern(None, Some(&p), Some(&b)).unwrap();
        let scan = g.store.scan(&pat);
        assert_eq!(scan.rows.len(), 2);
    }
}
