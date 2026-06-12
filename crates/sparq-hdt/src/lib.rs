//! sparq-hdt: load HDT (Header Dictionary Triples) archives into a sparq [`Graph`].
//!
//! HDT (<https://www.rdfhdt.org/>, W3C member submission) is the de-facto binary
//! archive format for RDF: the dictionary is Plain-Front-Coded and the triples are
//! a bitmap-compressed adjacency list, so files are a fraction of the size of
//! (even gzipped) N-Triples and load without text parsing. This crate wraps the
//! maintained [`hdt`] crate's reader — which supports the standard layout written
//! by hdt-cpp / hdt-java: FourSectionDictionary (PFC) + BitmapTriples, SPO order —
//! and streams its dictionary + triples into a sparq graph:
//!
//! ```no_run
//! let graph = sparq_hdt::load("dataset.hdt").unwrap();
//! ```
//!
//! The translation works at the **id level**: each distinct HDT dictionary id is
//! decompressed to its term string ONCE, interned into the sparq [`Dict`], and the
//! mapping memoized in a flat per-section table — so the term set is never
//! materialized twice and the per-triple work is three array lookups.
//!
//! GZipped containers (`.hdt.gz`) are detected by MAGIC BYTES (not file name)
//! and decompressed on the fly by every entry point; [`header`] exposes the
//! archive's metadata triples (the H in HDT) as a queryable sparq [`Graph`].

use sparq_core::dict::{Dict, Id};
use sparq_core::Graph;

use hdt::containers::ControlInfo;
use hdt::header::Header;
use hdt::{Hdt, IdKind};
use std::io::BufRead;
use std::path::Path;

/// The error type for HDT loading.
#[derive(Debug)]
pub enum Error {
    /// The file could not be opened / read.
    Io(std::io::Error),
    /// The archive could not be decoded (unsupported section type, checksum
    /// mismatch, truncation, …).
    Hdt(hdt::hdt::Error),
    /// A dictionary entry could not be translated to an RDF term.
    Term(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error reading HDT file: {e}"),
            Error::Hdt(e) => write!(f, "error decoding HDT archive: {e}"),
            Error::Term(e) => write!(f, "invalid term in HDT dictionary: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Hdt(e) => Some(e),
            Error::Term(_) => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<hdt::hdt::Error> for Error {
    fn from(e: hdt::hdt::Error) -> Self {
        Error::Hdt(e)
    }
}

/// Whether the stream starts with the gzip magic bytes (0x1f 0x8b). Peeks via
/// the reader's buffer without consuming.
fn is_gzip<R: BufRead>(reader: &mut R) -> std::io::Result<bool> {
    let buf = reader.fill_buf()?;
    Ok(buf.len() >= 2 && buf[0] == 0x1f && buf[1] == 0x8b)
}

/// Runs `f` over the (transparently decompressed) HDT byte stream: a stream
/// starting with the gzip magic is wrapped in a streaming `MultiGzDecoder`
/// (`.hdt.gz` containers as some publishers ship); anything else is passed
/// through. Detection is by CONTENT, not file name.
fn with_hdt_stream<T>(
    mut reader: impl BufRead,
    f: impl FnOnce(&mut dyn BufRead) -> Result<T, Error>,
) -> Result<T, Error> {
    if is_gzip(&mut reader)? {
        let mut decoder = std::io::BufReader::new(flate2::bufread::MultiGzDecoder::new(reader));
        f(&mut decoder)
    } else {
        f(&mut reader)
    }
}

/// Loads an HDT archive from a file path into a sparq [`Graph`].
///
/// Supports the standard HDT v1.0 layout (FourSectionDictionary with Plain Front
/// Coding + BitmapTriples in SPO order) as written by hdt-cpp, hdt-java and the
/// `hdt` crate; GZipped containers (`.hdt.gz`) are detected by magic bytes and
/// decompressed on the fly. HDT carries a single graph, so the result has no
/// named graphs.
pub fn load(path: impl AsRef<Path>) -> Result<Graph, Error> {
    let file = std::fs::File::open(path)?;
    load_reader(std::io::BufReader::new(file))
}

/// Loads an HDT archive from any buffered reader into a sparq [`Graph`].
///
/// The reader must be positioned at the start of the HDT data: the `$HDT`
/// cookie, or the gzip magic of a compressed container (decompressed
/// transparently, as in [`load`]).
pub fn load_reader<R: BufRead>(reader: R) -> Result<Graph, Error> {
    with_hdt_stream(reader, |r| graph_from_hdt(&Hdt::read(r)?))
}

/// Reads ONLY the HDT header — the dataset-metadata triples every archive
/// carries (the "H" in HDT: VoID statistics, format/provenance notes) — as a
/// queryable sparq [`Graph`], without decoding the dictionary or triples
/// sections. GZipped containers are handled as in [`load`].
///
/// (The wrapped `hdt` crate decodes the header during `Hdt::read` but keeps it
/// private, so this re-reads the head of the stream — it is a few KB.)
pub fn header(path: impl AsRef<Path>) -> Result<Graph, Error> {
    let file = std::fs::File::open(path)?;
    header_reader(std::io::BufReader::new(file))
}

/// [`header`] from any buffered reader positioned at the start of the HDT data.
pub fn header_reader<R: BufRead>(reader: R) -> Result<Graph, Error> {
    with_hdt_stream(reader, |mut r| {
        ControlInfo::read(&mut r).map_err(hdt::hdt::Error::from)?;
        let header = Header::read(&mut r).map_err(hdt::hdt::Error::from)?;
        // The in-crate Header stores parsed triples whose Display is their
        // N-Triples line (Header::write round-trips through it): render and
        // hand them to sparq's own N-Triples loader.
        let mut nt = String::new();
        for triple in &header.body {
            nt.push_str(&triple.to_string());
            nt.push('\n');
        }
        Graph::load_str(&nt, "ntriples").map_err(Error::Term)
    })
}

/// Converts an already-decoded [`hdt::Hdt`] into a sparq [`Graph`] — the seam for
/// callers that hold an `Hdt` (e.g. to also query its header metadata).
pub fn graph_from_hdt(hdt: &Hdt) -> Result<Graph, Error> {
    // HDT id spaces (all 1-based):
    //   subject id  s: 1..=n_shared          -> shared section (terms used as S and O)
    //               s: n_shared+1..=n_subj   -> subject-only section
    //   predicate p: 1..=n_pred              -> predicate section (own numbering)
    //   object id  o: 1..=n_shared           -> shared section (same terms as subject ids)
    //              o: n_shared+1..=n_obj     -> object-only section
    let n_shared = hdt.dict.shared.num_strings;
    let n_subj = n_shared + hdt.dict.subjects.num_strings;
    let n_pred = hdt.dict.predicates.num_strings;
    let n_obj_only = hdt.dict.objects.num_strings;

    let mut dict = Dict::new();
    // Memo tables: HDT id -> sparq id, 0 = not yet translated (sparq ids are never 0).
    // Object ids in the shared range reuse `subj` entries, so each shared term is
    // decompressed + interned exactly once even when it appears in both positions.
    let mut subj: Vec<Id> = vec![0; n_subj + 1];
    let mut pred: Vec<Id> = vec![0; n_pred + 1];
    let mut obj: Vec<Id> = vec![0; n_obj_only + 1]; // indexed by (o - n_shared)

    let mut triples: Vec<[Id; 3]> = Vec::with_capacity(hdt.triples.adjlist_z.sequence.entries);
    for [s, p, o] in &hdt.triples {
        let sid = match subj[s] {
            0 => {
                let id = translate(&mut dict, hdt, s, IdKind::Subject)?;
                subj[s] = id;
                id
            }
            id => id,
        };
        let pid = match pred[p] {
            0 => {
                let id = translate(&mut dict, hdt, p, IdKind::Predicate)?;
                pred[p] = id;
                id
            }
            id => id,
        };
        // Shared-section object ids denote the same term as the equal subject id.
        let oid = if o <= n_shared {
            match subj[o] {
                0 => {
                    let id = translate(&mut dict, hdt, o, IdKind::Object)?;
                    subj[o] = id;
                    id
                }
                id => id,
            }
        } else {
            match obj[o - n_shared] {
                0 => {
                    let id = translate(&mut dict, hdt, o, IdKind::Object)?;
                    obj[o - n_shared] = id;
                    id
                }
                id => id,
            }
        };
        triples.push([sid, pid, oid]);
    }

    Ok(Graph::from_parts(dict, triples))
}

/// Decompresses one HDT dictionary entry and interns it into the sparq dictionary.
fn translate(dict: &mut Dict, hdt: &Hdt, id: usize, kind: IdKind) -> Result<Id, Error> {
    let s = hdt.dict.id_to_string(id, kind).map_err(|e| Error::Term(e.to_string()))?;
    intern_hdt_term(dict, &s)
}

const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const RDF_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";

/// Interns an HDT dictionary string as a sparq term.
///
/// HDT dictionary encoding (per the spec's dictionary section and the reference
/// implementations): IRIs are stored bare (no angle brackets), blank nodes as
/// `_:label`, and literals in N-Triples-like shape — `"lexical"`,
/// `"lexical"@lang`, or `"lexical"^^<datatype>` — with the lexical form stored
/// raw (NOT N-Triples-escaped).
fn intern_hdt_term(dict: &mut Dict, s: &str) -> Result<Id, Error> {
    if let Some(rest) = s.strip_prefix('"') {
        // Literal. The lexical form may itself contain '"', so split at the LAST
        // quote (the lang tag / datatype suffix cannot contain one) — same rule as
        // the reference readers.
        let end = rest.rfind('"').ok_or_else(|| Error::Term(format!("unterminated literal: {s}")))?;
        let lex = &rest[..end];
        let suffix = &rest[end + 1..];
        if suffix.is_empty() {
            return Ok(dict.intern_lit(lex, XSD_STRING, None));
        }
        if let Some(tag) = suffix.strip_prefix('@') {
            // BCP47 tags are case-insensitive; oxrdf (and thus every other sparq
            // loader) normalizes to lowercase, so do the same for round-trip equality.
            let tag = tag.to_ascii_lowercase();
            return Ok(dict.intern_lit(lex, RDF_LANG_STRING, Some(&tag)));
        }
        if let Some(dt) = suffix.strip_prefix("^^") {
            // hdt-cpp / hdt-java / the hdt crate store the datatype as <iri>;
            // tolerate a bare IRI too.
            let dt = dt.strip_prefix('<').and_then(|d| d.strip_suffix('>')).unwrap_or(dt);
            if dt.is_empty() {
                return Err(Error::Term(format!("empty datatype in literal: {s}")));
            }
            return Ok(dict.intern_lit(lex, dt, None));
        }
        return Err(Error::Term(format!("malformed literal suffix: {s}")));
    }
    if let Some(label) = s.strip_prefix("_:") {
        return Ok(dict.intern_blank(label));
    }
    if s.is_empty() {
        return Err(Error::Term("empty dictionary entry".into()));
    }
    Ok(dict.intern_iri(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The dictionary-string parser must map every HDT term shape onto the term
    /// sparq's own loaders would produce (checked via the N-Triples rendering).
    #[test]
    fn term_shapes() {
        let mut dict = Dict::new();
        for (hdt_str, term_str) in [
            ("http://e.org/s", "<http://e.org/s>"),
            ("\"plain\"", "\"plain\""),
            ("\"hallo\"@de-DE", "\"hallo\"@de-de"),
            ("\"3.14\"^^<http://www.w3.org/2001/XMLSchema#double>", "\"3.14\"^^<http://www.w3.org/2001/XMLSchema#double>"),
            ("_:b1", "_:b1"),
        ] {
            let id = intern_hdt_term(&mut dict, hdt_str).unwrap();
            assert_eq!(dict.term(id).to_string(), term_str);
        }
        // Lexical forms containing quotes split at the LAST quote.
        let id = intern_hdt_term(&mut dict, "\"a\"b\"@en").unwrap();
        assert_eq!(dict.term(id).to_string(), "\"a\\\"b\"@en");
        // Bare (bracket-less) datatype IRIs are tolerated.
        let a = intern_hdt_term(&mut dict, "\"x\"^^http://e.org/dt").unwrap();
        let b = intern_hdt_term(&mut dict, "\"x\"^^<http://e.org/dt>").unwrap();
        assert_eq!(a, b);
        // Malformed entries are reported, not panicked on.
        assert!(intern_hdt_term(&mut dict, "\"open").is_err());
        assert!(intern_hdt_term(&mut dict, "\"x\"^^<>").is_err());
        assert!(intern_hdt_term(&mut dict, "\"x\"!!").is_err());
        assert!(intern_hdt_term(&mut dict, "").is_err());
    }
}
