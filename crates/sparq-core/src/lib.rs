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
    /// Named graphs (each a self-contained `Graph`), keyed by their name term. Empty for the
    /// usual single-default-graph load; populated by [`load_dataset`](Self::load_dataset) from
    /// N-Quads / TriG so the engine can evaluate `GRAPH <iri> { … }` / `GRAPH ?g { … }`.
    pub named: Vec<(Term, Graph)>,
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
        let (dict, triples) = Self::parse_to_triples(text, format)?;
        Ok(Self::from_parts(dict, triples))
    }

    /// Parses an RDF document into its dictionary + interned triples WITHOUT building the
    /// indexes. The seam that opt-in reasoning hooks into: a caller (e.g. the CLI, which can
    /// depend on `sparq-reason`) parses, materializes the entailed triples, then calls
    /// [`from_parts`](Self::from_parts) — keeping all reasoning out of the core engine.
    pub fn parse_to_triples(text: &str, format: &str) -> Result<(Dict, Vec<[Id; 3]>), String> {
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
                    return parse_ntriples_parallel(bytes);
                }
                #[cfg(not(feature = "parallel"))]
                {
                    let mut d = Dict::new();
                    let t = nt::parse_chunk(bytes, &mut d)?;
                    return Ok((d, t));
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
                // Turtle is not line-oriented, but it splits at top-level statement
                // terminators (with the @prefix preamble shared into each chunk), parsed in
                // parallel with a serial fallback on any mis-split — see parse_turtle_parallel.
                #[cfg(feature = "parallel")]
                {
                    return parse_turtle_parallel(bytes);
                }
                #[cfg(not(feature = "parallel"))]
                {
                    for t in TurtleParser::new().for_slice(bytes) {
                        let t = t.map_err(|e| e.to_string())?;
                        push_triple!(&t.subject, &t.predicate, &t.object);
                    }
                }
            }
        }

        Ok((dict, triples))
    }

    /// Load an RDF DATASET (N-Quads / TriG) preserving NAMED GRAPHS as separate sub-graphs, so the
    /// engine can evaluate `GRAPH <iri> { … }` / `GRAPH ?g { … }`. Default-graph triples form the
    /// main graph; each named graph becomes a [`named`](Self::named) entry. Formats without named
    /// graphs defer to [`load_str`](Self::load_str). In-memory only (the mmap path is triple-only).
    pub fn load_dataset(text: &str, format: &str) -> Result<Graph, String> {
        use oxrdf::GraphName;
        use std::collections::HashMap;
        if !matches!(format, "nquads" | "n-quads" | "trig" | "application/trig") {
            return Self::load_str(text, format);
        }
        let bytes = text.as_bytes();
        let mut groups: HashMap<Option<Term>, Vec<[Term; 3]>> = HashMap::new();
        macro_rules! group {
            ($parser:expr) => {
                for q in $parser.for_slice(bytes) {
                    let q = q.map_err(|e| e.to_string())?;
                    let g = match q.graph_name {
                        GraphName::DefaultGraph => None,
                        GraphName::NamedNode(n) => Some(Term::NamedNode(n)),
                        GraphName::BlankNode(b) => Some(Term::BlankNode(b)),
                    };
                    groups
                        .entry(g)
                        .or_default()
                        .push([subject_term(&q.subject), Term::NamedNode(q.predicate), q.object]);
                }
            };
        }
        match format {
            "nquads" | "n-quads" => group!(NQuadsParser::new()),
            _ => group!(TriGParser::new()),
        }
        let build_terms = |triples: &[[Term; 3]]| -> Graph {
            let mut dict = Dict::new();
            let ids: Vec<[Id; 3]> =
                triples.iter().map(|[s, p, o]| [dict.intern(s), dict.intern(p), dict.intern(o)]).collect();
            Self::build(dict, ids)
        };
        let default = groups.remove(&None).unwrap_or_default();
        let mut g = build_terms(&default);
        for (name, triples) in groups {
            if let Some(name) = name {
                g.named.push((name, build_terms(&triples)));
            }
        }
        Ok(g)
    }

    /// Builds a graph from an already-interned dictionary + triple set (e.g. after opt-in
    /// reasoning materialized additional triples). Public counterpart of the internal
    /// [`build`](Self::build).
    pub fn from_parts(dict: Dict, triples: Vec<[Id; 3]>) -> Graph {
        Self::build(dict, triples)
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

    /// Streaming PARALLEL load: reads the (already-decompressed) `reader` in newline-aligned
    /// ~32 MiB blocks and parses each block in parallel, so the full decompressed document is
    /// NEVER materialised in memory — only one block plus the growing dictionary/triples. (The
    /// store itself must fit in RAM; this removes the redundant full-text copy a read-to-string
    /// load would hold alongside it.) For N-Triples; other formats defer to the serial streaming
    /// [`load_reader`].
    #[cfg(feature = "parallel")]
    pub fn load_reader_parallel<R: std::io::Read>(mut reader: R, format: &str) -> Result<Graph, String> {
        if !matches!(format, "ntriples" | "n-triples") {
            return Self::load_reader(reader, format);
        }
        const BLOCK: usize = 32 << 20;
        let mut global = Dict::new();
        let mut all: Vec<[Id; 3]> = Vec::new();
        let mut carry: Vec<u8> = Vec::new();
        let mut chunk = vec![0u8; BLOCK];
        // Parse one newline-aligned block in parallel and merge its partial dict into the global.
        fn flush(global: &mut Dict, all: &mut Vec<[Id; 3]>, bytes: &[u8]) -> Result<(), String> {
            let (pd, pt) = parse_ntriples_parallel(bytes)?;
            let remap = global.merge_remap(&pd);
            remap_extend(all, pt, &remap);
            Ok(())
        }
        loop {
            let n = reader.read(&mut chunk).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            carry.extend_from_slice(&chunk[..n]);
            if let Some(pos) = carry.iter().rposition(|&b| b == b'\n') {
                flush(&mut global, &mut all, &carry[..=pos])?;
                carry.drain(..=pos);
            }
        }
        if !carry.is_empty() {
            flush(&mut global, &mut all, &carry)?;
        }
        Ok(Self::build(global, all))
    }

    /// Builds the store + numeric cache from interned triples (shared by the
    /// string and streaming loaders).
    fn build(dict: Dict, triples: Vec<[Id; 3]>) -> Graph {
        let store = TripleStore::from_triples(triples);
        let numerics = NumData::Owned(numerics_of(&dict));
        Graph { dict, store, numerics, named: Vec::new() }
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
            named: self.named,
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
        Ok(Graph { dict, store, numerics, named: Vec::new() })
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
        #[cfg(all(feature = "mmap", feature = "parallel"))]
        let _t_build = std::time::Instant::now();
        #[cfg(all(feature = "mmap", feature = "parallel"))]
        build_timing::reset();

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

        // Opt-in (`SPARQ_SHARDED_DICT`) PARALLEL sharded-dict ingest for N-Triples: interns
        // through N hash-shards (no serial `merge_remap`), spilling temporary sharded ids
        // that an order-preserving remap turns into final dense ids after the SPO sort. When
        // not selected (or for other formats / non-parallel builds), the normal path runs.
        #[cfg(all(feature = "mmap", feature = "parallel"))]
        let sharded = matches!(format, "ntriples" | "n-triples") && std::env::var("SPARQ_SHARDED_DICT").is_ok();
        #[cfg(not(all(feature = "mmap", feature = "parallel")))]
        let sharded = false;
        let mut sharded_remap: Option<(Vec<u64>, u32)> = None;

        if sharded {
            #[cfg(all(feature = "mmap", feature = "parallel"))]
            {
                let n_shards = (rayon::current_num_threads() * 2).clamp(4, 64);
                let mut sd = dict::ShardedDict::new(n_shards);
                build_external_ntriples_sharded(reader, &mut sd, &mut buf, &mut runs, &tmp, chunk)?;
                let (merged, base, stride) = sd.into_merged();
                dict = merged;
                sharded_remap = Some((base, stride));
            }
        } else {
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
        }
        extsort::spill_run(&mut buf, &mut runs, &tmp).map_err(|e| e.to_string())?;
        buf.shrink_to_fit();
        #[cfg(all(feature = "mmap", feature = "parallel"))]
        if build_timing::enabled() {
            build_timing::report("parse+intern+spill done", _t_build.elapsed().as_secs_f64());
        }

        // Merge the SPO runs into the SPO permutation file (deduplicating).
        let spo_path = dir.join(format!("perm{}.bin", Perm::Spo as usize));
        extsort::kway_merge(&runs, &spo_path).map_err(|e| e.to_string())?;
        #[cfg(all(feature = "mmap", feature = "parallel"))]
        if build_timing::enabled() {
            eprintln!("[build-timing] kway_merge SPO done | {:.2}s wall to here", _t_build.elapsed().as_secs_f64());
        }
        for r in &runs {
            std::fs::remove_file(r).ok();
        }
        // Sharded build: the SPO perm holds TEMPORARY sharded ids — remap them to final dense
        // ids in place (order-preserving, so it stays sorted+deduped) BEFORE the sibling sorts
        // read it, so the permutations and the merged dictionary agree on ids.
        #[cfg(all(feature = "mmap", feature = "parallel"))]
        if let Some((base, stride)) = &sharded_remap {
            remap_perm_file(&spo_path, base, *stride)?;
        }

        // Build the other BUILT permutations by external-sorting the SPO file into each
        // order (re-reading it memory-mapped, so this stays bounded-memory too).
        let (map, n) = extsort::map_perm(&spo_path).map_err(|e| e.to_string())?;
        // SAFETY: perm0 is a whole number of [u32;3] rows written above; map outlives the loop.
        let spo: &[[Id; 3]] =
            unsafe { std::slice::from_raw_parts(map.as_ptr().cast::<[Id; 3]>(), n) };
        let siblings: Vec<Perm> = BUILT.iter().copied().filter(|&p| p != Perm::Spo).collect();
        let sib_sort = |perm: Perm, sub: &std::path::Path, per: usize| -> Result<(), String> {
            std::fs::create_dir_all(sub).map_err(|e| e.to_string())?;
            let out = dir.join(format!("perm{}.bin", perm as usize));
            extsort::external_sort(spo.iter().copied(), perm.order(), &out, sub, per).map_err(|e| e.to_string())
        };
        // The sibling sorts are independent — run them CONCURRENTLY (each in its own tmp
        // subdir, so run files don't collide), sharing the chunk budget so total resident
        // memory stays ~`chunk`. The shared SPO mmap is read-only (paged, no extra RAM).
        // Persisting the DICTIONARY (save_mmap: the term blob + sorted-hash index) and the
        // numeric cache only needs `dict` — it is INDEPENDENT of the permutation sorts. Run
        // it CONCURRENTLY with the sibling sorts (on its own thread) so the multi-hundred-MB
        // dict write is hidden under the sort time instead of being a serial tail. Output is
        // byte-identical (same files); only the wall-clock ordering overlaps.
        std::thread::scope(|scope| -> Result<(), String> {
            let dict_ref = &dict;
            let finalize = scope.spawn(move || -> Result<(), String> {
                dict_ref.save_mmap(dir).map_err(|e| e.to_string())?;
                write_numerics(&dir.join("numerics.bin"), &numerics_of(dict_ref)).map_err(|e| e.to_string())?;
                Ok(())
            });
            #[cfg(feature = "parallel")]
            {
                use rayon::prelude::*;
                let per = (chunk / siblings.len().max(1)).max(1 << 16);
                siblings
                    .par_iter()
                    .try_for_each(|&perm| sib_sort(perm, &tmp.join(format!("p{}", perm as usize)), per))?;
            }
            #[cfg(not(feature = "parallel"))]
            for &perm in &siblings {
                sib_sort(perm, &tmp, chunk)?;
            }
            finalize.join().map_err(|_| "dict-finalize thread panicked".to_string())??;
            Ok(())
        })?;
        drop(map);
        #[cfg(all(feature = "mmap", feature = "parallel"))]
        if build_timing::enabled() {
            eprintln!("[build-timing] sibling sorts ∥ dict-save done | {:.2}s wall to here", _t_build.elapsed().as_secs_f64());
        }

        // Empty files for the unbuilt permutations so `open` finds all six slots.
        for i in 0..6 {
            let p = dir.join(format!("perm{i}.bin"));
            if !p.exists() {
                std::fs::File::create(&p).map_err(|e| e.to_string())?;
            }
        }
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
/// Per-ISA software prefetch-for-read hint (x86 `prefetcht0`, aarch64 `prfm pldl1keep`, a no-op
/// elsewhere). Correctness-neutral — a prefetch never faults and never changes architectural
/// state; it only asks the CPU to pull a cache line in early.
#[cfg(feature = "parallel")]
#[inline(always)]
fn prefetch_read<T>(p: *const T) {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: _mm_prefetch is defined for any address (the hint is dropped on a bad one).
    unsafe {
        core::arch::x86_64::_mm_prefetch(p as *const i8, core::arch::x86_64::_MM_HINT_T0);
    }
    #[cfg(target_arch = "aarch64")]
    // SAFETY: prfm is a hint — it cannot fault or write memory/registers.
    unsafe {
        core::arch::asm!("prfm pldl1keep, [{0}]", in(reg) p, options(nostack, preserves_flags));
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    let _ = p;
}

/// Compile-time default for the dict-remap software prefetch, chosen per hardware from measured
/// A/B results (the `bench-remap` micro-benchmark, prefetch on vs off; the hint helps or *hurts*
/// depending on the core's hardware prefetcher):
///   x86_64 (Intel/AMD; incl. AWS c7i Sapphire Rapids):   +7.5%  -> ON
///   aarch64 + macOS (Apple M-series):                    +22%   -> ON
///   aarch64 + Linux (AWS Graviton3 / Neoverse-V1, etc.): -10%   -> OFF — the HW prefetcher
///       already saturates the gather, so explicit `prfm` hints only add instruction overhead.
/// Overridable at runtime for re-tuning on new silicon: `SPARQ_PREFETCH=1` forces on,
/// `SPARQ_NO_PREFETCH=1` forces off.
#[cfg(feature = "parallel")]
const PREFETCH_DEFAULT: bool = cfg!(any(
    target_arch = "x86_64",
    all(target_arch = "aarch64", target_os = "macos"),
));

/// Append `triples`, remapped from a partial dict's ids to the merged global ids
/// (`remap[id-1]`; inline-integer ids pass through). The `remap` gather is latency-bound on a
/// large global dictionary (the build-path bottleneck — triple-remap measured at ~3s/50M), so
/// each iteration software-prefetches the gather targets of a triple `DIST` ahead. Hardware-
/// specific (per-ISA prefetch) but correctness-neutral; the prefetch is just a hint.
#[cfg(feature = "parallel")]
fn remap_extend(out: &mut Vec<[Id; 3]>, triples: Vec<[Id; 3]>, remap: &[Id]) {
    const DIST: usize = 32;
    let n = triples.len();
    let base = remap.as_ptr();
    // Per-hardware default (PREFETCH_DEFAULT, measured), with a runtime override. The getenv runs
    // once per merged-dict load — which then processes millions of triples — so it is free.
    let do_prefetch = match (
        std::env::var("SPARQ_PREFETCH").as_deref(),
        std::env::var("SPARQ_NO_PREFETCH").as_deref(),
    ) {
        (Ok("1"), _) => true,
        (_, Ok("1")) => false,
        _ => PREFETCH_DEFAULT,
    };
    let lut = |id: Id| -> Id {
        if id >= dict::INLINE_BASE {
            id
        } else {
            remap[(id - 1) as usize]
        }
    };
    out.reserve(n);
    for i in 0..n {
        if do_prefetch && i + DIST < n {
            for &id in &triples[i + DIST] {
                if id < dict::INLINE_BASE {
                    // SAFETY: id-1 < remap.len() for every dictionary id; prefetch is hint-only.
                    prefetch_read(unsafe { base.add((id - 1) as usize) });
                }
            }
        }
        let [s, p, o] = triples[i];
        out.push([lut(s), lut(p), lut(o)]);
    }
}

/// Isolated micro-benchmark of the latency-bound `remap_extend` gather (the build-path
/// bottleneck the per-ISA prefetch targets). Builds `n` synthetic triples whose ids scatter
/// randomly across a `dict_size`-entry remap table (so the gather misses cache like a real large
/// global dictionary), then times `remap_extend` over `iters` runs and returns the best (ms).
/// Honours `SPARQ_NO_PREFETCH=1`. Used to measure the prefetch's effect per hardware in isolation,
/// undiluted by parsing. Not part of the query/build path.
#[cfg(feature = "parallel")]
pub fn bench_remap(n: usize, dict_size: usize, iters: usize) -> f64 {
    // Cheap deterministic LCG scatter (no rand dep, no Date/Random harness restrictions).
    let mut x: u64 = 0x9E3779B97F4A7C15;
    let mut next = || { x ^= x << 13; x ^= x >> 7; x ^= x << 17; x };
    let ds = dict_size.max(1) as u64;
    let mut triples: Vec<[Id; 3]> = Vec::with_capacity(n);
    for _ in 0..n {
        // ids in [1, dict_size], scattered, so the gather misses cache.
        let s = (1 + (next() % ds)) as Id;
        let p = (1 + (next() % ds)) as Id;
        let o = (1 + (next() % ds)) as Id;
        triples.push([s, p, o]);
    }
    // Identity remap of the right size (values irrelevant to the gather's latency).
    let remap: Vec<Id> = (1..=dict_size as Id).collect();
    let mut best = f64::INFINITY;
    for _ in 0..iters.max(1) {
        let mut out: Vec<[Id; 3]> = Vec::new();
        let t = std::time::Instant::now();
        remap_extend(&mut out, triples.clone(), &remap);
        let ms = t.elapsed().as_secs_f64() * 1e3;
        std::hint::black_box(&out);
        if ms < best { best = ms; }
    }
    best
}

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
        // Dictionary ids are 1-based and below INLINE_BASE; inline-integer ids carry their
        // value and pass through unchanged. Prefetches the remap gather (see remap_extend).
        remap_extend(&mut all, ptriples, &remap);
    }
    Ok((global, all))
}

/// Serial Turtle parse of `bytes` into `dict` — the fallback and the per-chunk worker.
#[cfg(feature = "parallel")]
fn parse_turtle_chunk(bytes: &[u8], dict: &mut Dict) -> Result<Vec<[Id; 3]>, String> {
    let mut triples = Vec::new();
    for t in TurtleParser::new().for_slice(bytes) {
        let t = t.map_err(|e| e.to_string())?;
        let s = dict.intern(&subject_term(&t.subject));
        let p = dict.intern(&Term::NamedNode(t.predicate.clone()));
        let o = dict.intern(&t.object);
        triples.push([s, p, o]);
    }
    Ok(triples)
}

/// Skip whitespace and `#`-comments from `i`, returning the next significant byte offset.
#[cfg(feature = "parallel")]
fn skip_ws_comments(bytes: &[u8], mut i: usize) -> usize {
    let n = bytes.len();
    loop {
        while i < n && matches!(bytes[i], b' ' | b'\t' | b'\n' | b'\r') {
            i += 1;
        }
        if i < n && bytes[i] == b'#' {
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
        } else {
            return i;
        }
    }
}

/// Is the token at `k` a SPARQL-style `PREFIX`/`BASE` directive (keyword + whitespace)?
#[cfg(feature = "parallel")]
fn is_sparql_directive_start(bytes: &[u8], k: usize) -> bool {
    let m = |kw: &[u8]| {
        bytes.len() > k + kw.len()
            && bytes[k..k + kw.len()].eq_ignore_ascii_case(kw)
            && matches!(bytes.get(k + kw.len()), Some(b' ' | b'\t' | b'\n' | b'\r'))
    };
    m(b"prefix") || m(b"base")
}

/// Scan from `start` (in the top-level/normal Turtle state) to the next statement-terminating
/// `.` (one followed by whitespace/EOF/comment, i.e. not a decimal point or PN_LOCAL dot),
/// skipping over IRIs `<...>`, string literals (all four quote forms, with `\` escapes), and
/// `#` comments. Returns the offset just past the `.`, or `None` if EOF is reached first
/// (malformed/incomplete → caller falls back to the serial parser).
///
/// Sets `*saw_bnode` if a blank node syntax (`[`, `(` collection, or `_:`) is seen in the normal
/// state: blank-node identity is document-scoped, so independently-parsed chunks would restart
/// blank-node numbering and collide on merge — the caller must then fall back to serial.
#[cfg(feature = "parallel")]
fn next_terminator(bytes: &[u8], start: usize, saw_bnode: &mut bool) -> Option<usize> {
    let n = bytes.len();
    let mut i = start;
    while i < n {
        match bytes[i] {
            b'[' | b'(' => {
                *saw_bnode = true;
                i += 1;
            }
            b'_' if bytes.get(i + 1) == Some(&b':') => {
                *saw_bnode = true;
                i += 2;
            }
            b'#' => {
                while i < n && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'<' => {
                i += 1;
                while i < n && bytes[i] != b'>' {
                    i += 1;
                }
                if i >= n {
                    return None;
                }
                i += 1;
            }
            q @ (b'"' | b'\'') => {
                let triple = i + 2 < n && bytes[i + 1] == q && bytes[i + 2] == q;
                if triple {
                    i += 3;
                    loop {
                        if i >= n {
                            return None;
                        }
                        if bytes[i] == b'\\' {
                            i += 2;
                        } else if bytes[i] == q && i + 2 < n && bytes[i + 1] == q && bytes[i + 2] == q {
                            i += 3;
                            break;
                        } else {
                            i += 1;
                        }
                    }
                } else {
                    i += 1;
                    while i < n && bytes[i] != q {
                        i += if bytes[i] == b'\\' { 2 } else { 1 };
                    }
                    if i >= n {
                        return None;
                    }
                    i += 1;
                }
            }
            b'.' => {
                match bytes.get(i + 1) {
                    None | Some(b' ' | b'\t' | b'\n' | b'\r' | b'#') => return Some(i + 1),
                    _ => i += 1,
                }
            }
            _ => i += 1,
        }
    }
    None
}

/// Split Turtle `bytes` into independently-parseable chunks for parallel parsing, or `None` to
/// fall back to serial. Fast-paths the overwhelmingly common shape: a leading run of `@prefix`/
/// `@base` directives followed by triple statements. Each chunk is `preamble + a run of
/// statements`, so the prefixes are in scope in every chunk. Bails (→ serial) on SPARQL-style
/// `PREFIX`/`BASE`, directives interspersed among the triples, or anything it cannot scan.
#[cfg(feature = "parallel")]
fn turtle_chunks(bytes: &[u8], target: usize) -> Option<Vec<Vec<u8>>> {
    let n = bytes.len();
    let mut saw_bnode = false;
    // Phase 1: consume the leading @-directive preamble.
    let mut i = skip_ws_comments(bytes, 0);
    while i < n && bytes[i] == b'@' {
        i = next_terminator(bytes, i, &mut saw_bnode)?;
        i = skip_ws_comments(bytes, i);
    }
    if i < n && is_sparql_directive_start(bytes, i) {
        return None; // SPARQL-style preamble — not fast-pathed
    }
    let pre_end = i;

    // Phase 2: collect the body's top-level terminators; bail on any interspersed directive
    // or any blank node (document-scoped identity can't be parsed chunk-independently).
    let mut terms: Vec<usize> = Vec::new();
    let mut j = pre_end;
    loop {
        let k = skip_ws_comments(bytes, j);
        if k >= n {
            break;
        }
        if bytes[k] == b'@' || is_sparql_directive_start(bytes, k) {
            return None;
        }
        let t = next_terminator(bytes, k, &mut saw_bnode)?;
        if saw_bnode {
            return None;
        }
        terms.push(t);
        j = t;
    }
    if terms.len() < 2 {
        return None;
    }

    // Partition the body terminators into ~target contiguous groups.
    let preamble = &bytes[..pre_end];
    let per = (terms.len() / target.max(1)).max(1);
    let mut chunks: Vec<Vec<u8>> = Vec::new();
    let mut body_start = pre_end;
    let mut idx = 0;
    while idx < terms.len() {
        let end_i = (idx + per).min(terms.len());
        let body_end = terms[end_i - 1];
        let mut chunk = Vec::with_capacity(preamble.len() + (body_end - body_start) + 1);
        chunk.extend_from_slice(preamble);
        chunk.push(b'\n');
        chunk.extend_from_slice(&bytes[body_start..body_end]);
        chunks.push(chunk);
        body_start = body_end;
        idx = end_i;
    }
    Some(chunks)
}

/// Parse Turtle in parallel by statement-boundary chunking (see [`turtle_chunks`]). If the input
/// is not safely splittable, OR any chunk fails to parse (an over-eager split), it falls back to
/// the serial parser — so the result is always identical to a plain serial Turtle parse.
#[cfg(feature = "parallel")]
fn parse_turtle_parallel(bytes: &[u8]) -> Result<(Dict, Vec<[Id; 3]>), String> {
    use rayon::prelude::*;
    let serial = || {
        let mut dict = Dict::new();
        parse_turtle_chunk(bytes, &mut dict).map(|t| (dict, t))
    };
    let target = (rayon::current_num_threads().max(1) * 4).min(bytes.len() / 8192 + 1).max(1);
    let chunks = match turtle_chunks(bytes, target) {
        Some(c) if c.len() > 1 => c,
        _ => return serial(),
    };
    let partials: Result<Vec<(Dict, Vec<[Id; 3]>)>, String> = chunks
        .par_iter()
        .map(|chunk| {
            let mut dict = Dict::new();
            parse_turtle_chunk(chunk, &mut dict).map(|t| (dict, t))
        })
        .collect();
    let partials = match partials {
        Ok(p) => p,
        Err(_) => return serial(), // an over-eager split produced invalid Turtle — redo serially
    };
    let total: usize = partials.iter().map(|(_, t)| t.len()).sum();
    let cap = partials.iter().map(|(d, _)| d.len()).max().unwrap_or(0);
    let mut global = Dict::with_capacity(cap);
    let mut all = Vec::with_capacity(total);
    for (pd, ptriples) in partials {
        let remap = global.merge_remap(&pd);
        remap_extend(&mut all, ptriples, &remap);
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
    // Parsed partials flow parse-thread -> merge (this thread). A small bound keeps memory
    // bounded (a couple of blocks' partials in flight) while letting the rayon PARSE of the
    // next block overlap the SERIAL dict-merge of the current one. Profiling showed the
    // merge (merge_remap + triple-remap, ~10.5s/50M) dominates the parallel parse (~5.2s),
    // and they previously ran sequentially per block; this 3-stage pipeline hides the parse.
    type Partials = Vec<(Dict, Vec<[Id; 3]>)>;
    let (ptx, prx) = sync_channel::<Partials>(2);

    std::thread::scope(|scope| -> Result<(), String> {
        // Stage 1 — decompress + read on its own thread, emitting newline-aligned blocks.
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
                    return Ok(()); // a downstream stage errored and dropped the receiver
                }
            }
        });

        // Stage 2 — parse+intern each block in parallel (per-chunk local dicts, no shared
        // state), forwarding the partials to the merge stage. Concurrent with stage 3.
        let parser = scope.spawn(move || -> Result<(), String> {
            for block in rx {
                if block.is_empty() {
                    continue;
                }
                let partials = parse_block(&block)?;
                if ptx.send(partials).is_err() {
                    return Ok(()); // the merge stage errored and dropped the receiver
                }
            }
            Ok(())
        });

        // Stage 3 (this thread) — SERIAL dict-merge + triple-remap + spill; owns dict/buf/
        // runs. The id-assignment order is identical to the old sequential path (blocks
        // arrive in order; partials are in chunk order), so the output is byte-identical.
        for partials in prx {
            for (pd, ptriples) in partials {
                let t_merge = std::time::Instant::now();
                let remap = dict.merge_remap(&pd);
                build_timing::MERGE_NS.fetch_add(t_merge.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
                let t_remap = std::time::Instant::now();
                let map = |id: Id| if id >= dict::INLINE_BASE { id } else { remap[(id - 1) as usize] };
                // Prefetch the remap gather DIST triples ahead — the large-global-dict gather
                // is the build-path bottleneck (per-ISA hint, correctness-neutral).
                let base = remap.as_ptr();
                for i in 0..ptriples.len() {
                    if i + 32 < ptriples.len() {
                        for &id in &ptriples[i + 32] {
                            if id < dict::INLINE_BASE {
                                // SAFETY: id-1 < remap.len(); prefetch is hint-only.
                                prefetch_read(unsafe { base.add((id - 1) as usize) });
                            }
                        }
                    }
                    let [s, p, o] = ptriples[i];
                    buf.push([map(s), map(p), map(o)]);
                    if buf.len() >= chunk {
                        extsort::spill_run(buf, runs, tmp).map_err(|e| e.to_string())?;
                    }
                }
                build_timing::REMAP_NS.fetch_add(t_remap.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
            }
        }
        // Join parse first (it feeds stage 3 — surface a parse error), then the producer.
        parser.join().map_err(|_| "parse thread panicked".to_string())??;
        producer.join().map_err(|_| "decompression thread panicked".to_string())?
    })
}

/// SHARDED variant of the parallel ingest: same decompress→parse stages, but the merge stage
/// interns into a hash-sharded dictionary (`ShardedDict`) so the dominant dict work runs in
/// parallel across shards instead of through one serial `merge_remap`. Triples are spilled
/// with TEMPORARY sharded ids (`shard*STRIDE+local`); `ShardedDict::into_merged` + an
/// order-preserving `remap_perm_file` pass turn them into final dense ids after the sort.
#[cfg(all(feature = "mmap", feature = "parallel"))]
fn build_external_ntriples_sharded<R: std::io::Read + Send>(
    reader: R,
    sharded: &mut dict::ShardedDict,
    buf: &mut Vec<[Id; 3]>,
    runs: &mut Vec<std::path::PathBuf>,
    tmp: &std::path::Path,
    chunk: usize,
) -> Result<(), String> {
    use std::sync::mpsc::sync_channel;
    const BLOCK: usize = 64 << 20;
    let (tx, rx) = sync_channel::<Vec<u8>>(3);
    type Partials = Vec<(Dict, Vec<[Id; 3]>)>;
    let (ptx, prx) = sync_channel::<Partials>(2);

    std::thread::scope(|scope| -> Result<(), String> {
        // Stage 1 — decompress (identical to the non-sharded pipeline).
        let producer = scope.spawn(move || -> Result<(), String> {
            let mut reader = reader;
            let mut readbuf = vec![0u8; BLOCK];
            let mut carry: Vec<u8> = Vec::new();
            loop {
                let mut filled = 0;
                while filled < BLOCK {
                    let n = reader.read(&mut readbuf[filled..]).map_err(|e| e.to_string())?;
                    if n == 0 {
                        break;
                    }
                    filled += n;
                }
                if filled == 0 {
                    if !carry.is_empty() {
                        let _ = tx.send(std::mem::take(&mut carry));
                    }
                    return Ok(());
                }
                let mut block = std::mem::take(&mut carry);
                block.extend_from_slice(&readbuf[..filled]);
                let cut = block.iter().rposition(|&b| b == b'\n').map_or(0, |p| p + 1);
                carry = block[cut..].to_vec();
                block.truncate(cut);
                if tx.send(block).is_err() {
                    return Ok(());
                }
            }
        });
        // Stage 2 — parse (identical).
        let parser = scope.spawn(move || -> Result<(), String> {
            for block in rx {
                if block.is_empty() {
                    continue;
                }
                let partials = parse_block(&block)?;
                if ptx.send(partials).is_err() {
                    return Ok(());
                }
            }
            Ok(())
        });
        // Stage 3 — SHARDED merge: route each partial's (non-inline) terms to shards and
        // intern in parallel (component-based, no Term alloc), then remap triples to temp ids.
        for partials in prx {
            let t_merge = std::time::Instant::now();
            let remaps = sharded.intern_partials(&partials);
            build_timing::MERGE_NS.fetch_add(t_merge.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
            let t_remap = std::time::Instant::now();
            for (pidx, (_, ptriples)) in partials.iter().enumerate() {
                let rm = &remaps[pidx];
                let map = |id: Id| if id >= dict::INLINE_BASE { id } else { rm[id as usize] };
                for &[s, p, o] in ptriples {
                    buf.push([map(s), map(p), map(o)]);
                    if buf.len() >= chunk {
                        extsort::spill_run(buf, runs, tmp).map_err(|e| e.to_string())?;
                    }
                }
            }
            build_timing::REMAP_NS.fetch_add(t_remap.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
        }
        parser.join().map_err(|_| "parse thread panicked".to_string())??;
        producer.join().map_err(|_| "decompression thread panicked".to_string())?
    })
}

/// Rewrite a permutation file in place, remapping every temporary sharded id to its final
/// dense id (`dict::remap_sharded`). Order-preserving (temp ids sort like final ids), so the
/// already-sorted, deduplicated file stays sorted — no re-sort needed.
#[cfg(all(feature = "mmap", feature = "parallel"))]
fn remap_perm_file(path: &std::path::Path, base: &[u64], stride: u32) -> Result<(), String> {
    let f = std::fs::OpenOptions::new().read(true).write(true).open(path).map_err(|e| e.to_string())?;
    let len = f.metadata().map_err(|e| e.to_string())?.len() as usize;
    if len == 0 {
        return Ok(());
    }
    // SAFETY: read-write mapping of a freshly-written perm file of whole [u32;3] rows.
    let mut mmap = unsafe { memmap2::MmapMut::map_mut(&f) }.map_err(|e| e.to_string())?;
    let ids: &mut [u32] = unsafe { std::slice::from_raw_parts_mut(mmap.as_mut_ptr().cast::<u32>(), len / 4) };
    use rayon::prelude::*;
    ids.par_iter_mut().for_each(|id| *id = dict::remap_sharded(*id, base, stride));
    mmap.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// Parses one (complete-line) N-Triples byte block in parallel into per-chunk partial
/// dictionaries + local-id triples (no shared state) — the parallelizable half of ingest.
#[cfg(all(feature = "mmap", feature = "parallel"))]
fn parse_block(bytes: &[u8]) -> Result<Vec<(Dict, Vec<[Id; 3]>)>, String> {
    use rayon::prelude::*;
    let target = (rayon::current_num_threads().max(1) * 4).min(bytes.len() / 4096 + 1);
    let bounds = newline_chunk_bounds(bytes, target);
    let t_parse = std::time::Instant::now();
    let partials: Vec<(Dict, Vec<[Id; 3]>)> = bounds
        .par_iter()
        .map(|&(s, e)| {
            let mut d = Dict::new();
            let t = nt::parse_chunk(&bytes[s..e], &mut d)?;
            Ok::<_, String>((d, t))
        })
        .collect::<Result<Vec<_>, _>>()?;
    build_timing::PARSE_NS.fetch_add(t_parse.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
    Ok(partials)
}

/// Env-gated (`SPARQ_BUILD_TIMING`) phase-time accumulators for the parallel ingest path,
/// to attribute wall-time across parallel-parse vs serial dict-merge vs serial triple-remap.
#[cfg(all(feature = "mmap", feature = "parallel"))]
mod build_timing {
    use std::sync::atomic::AtomicU64;
    pub static PARSE_NS: AtomicU64 = AtomicU64::new(0);
    pub static MERGE_NS: AtomicU64 = AtomicU64::new(0);
    pub static REMAP_NS: AtomicU64 = AtomicU64::new(0);
    pub fn reset() {
        use std::sync::atomic::Ordering::Relaxed;
        PARSE_NS.store(0, Relaxed);
        MERGE_NS.store(0, Relaxed);
        REMAP_NS.store(0, Relaxed);
    }
    pub fn enabled() -> bool {
        std::env::var("SPARQ_BUILD_TIMING").is_ok()
    }
    pub fn report(stage: &str, secs: f64) {
        use std::sync::atomic::Ordering::Relaxed;
        let (p, m, r) = (
            PARSE_NS.load(Relaxed) as f64 / 1e9,
            MERGE_NS.load(Relaxed) as f64 / 1e9,
            REMAP_NS.load(Relaxed) as f64 / 1e9,
        );
        eprintln!(
            "[build-timing] {stage}: parse(parallel) {p:.2}s | merge_remap(serial) {m:.2}s | triple-remap(serial) {r:.2}s | {secs:.2}s wall to here"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "parallel")]
    #[test]
    fn parallel_turtle_matches_serial() {
        let decoded = |d: &Dict, t: &[[Id; 3]]| -> Vec<String> {
            let mut v: Vec<String> =
                t.iter().map(|&[s, p, o]| format!("{}|{}|{}", d.term(s), d.term(p), d.term(o))).collect();
            v.sort();
            v
        };
        // Blank-node-FREE doc (exercises the parallel statement-split path), >8 KiB so it fans
        // out, with edge cases that stress the terminator scan: decimals, dots inside strings &
        // IRIs, escaped quotes, multi-line ;/, statements, triple-quoted strings with `.`+
        // newlines, and trailing comments.
        let mut ttl = String::from(
            "@prefix : <http://ex/> .\n@prefix ex: <http://example.org/foo.bar#> .\n# header . comment\n",
        );
        for i in 0..500 {
            ttl.push_str(&format!(
                ":s{i} :dec {i}.5 ; :s \"a.b.c\" , \"x\\\"y.z\" ; :iri ex:rel{i} .\n\
                 :s{i} ex:m \"\"\"l1 . still\nl2.\"\"\" ; :p <http://x.y/a.b.{i}> . # trailing . c\n",
            ));
        }
        assert!(ttl.len() > 8192);
        assert!(turtle_chunks(ttl.as_bytes(), 32).is_some(), "blank-node-free doc should fan out");
        let (pd, pt) = parse_turtle_parallel(ttl.as_bytes()).unwrap();
        let mut sd = Dict::new();
        let st = parse_turtle_chunk(ttl.as_bytes(), &mut sd).unwrap();
        assert_eq!(decoded(&pd, &pt), decoded(&sd, &st), "parallel split must equal serial");
        assert!(pt.len() >= 1500);

        // Blank nodes ([ ], ( ), _:label) must BAIL to the serial parser (chunk-independent
        // parsing would collide their document-scoped identity). We can't compare anonymous
        // blank-node labels across two parses (oxttl mints fresh ones each run), so we assert
        // the bail itself plus a matching triple count.
        let bn = "@prefix : <http://ex/> .\n:a :p [ :q :r ] .\n:x :y ( :i1 :i2 ) .\n_:b :z :w .\n".repeat(300);
        assert!(turtle_chunks(bn.as_bytes(), 32).is_none(), "blank nodes must bail to serial");
        let (_, bt) = parse_turtle_parallel(bn.as_bytes()).unwrap();
        let mut bsd = Dict::new();
        let bst = parse_turtle_chunk(bn.as_bytes(), &mut bsd).unwrap();
        assert_eq!(bt.len(), bst.len(), "blank-node doc triple count must match serial");
    }

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
