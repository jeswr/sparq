//! [OPUS-4.8] sq-ashy — DIRECT in-memory HDT encoder: serialise a sparq [`Graph`]
//! to the standard HDT v1.0 layout (FourSectionDictionary with Plain Front Coding +
//! BitmapTriples, SPO order) WITHOUT a temporary N-Triples round-trip.
//!
//! ## What this replaces (and why it is the inverse of `decode.rs`)
//!
//! The previous [`crate::save`] path (still available as the differential oracle in
//! the tests) rendered the whole graph to N-Triples TEXT on disk, then re-parsed it
//! through [`hdt::Hdt::read_nt`] — which re-interned every term and re-sorted the
//! triples sparq already held. That is a full text decode + re-encode of data sparq
//! already has in compact id form.
//!
//! This encoder feeds the upstream section builders directly from sparq's in-memory
//! dictionary + triple ids, then writes the sections in the exact order (and via the
//! exact byte-level writers) [`hdt::Hdt::write`] uses:
//!
//!   1. `ControlInfo::global()`           — the `$HDT` global preamble + CRC16.
//!   2. `Header` (ntriples body)          — VoID statistics, length-framed.
//!   3. `FourSectDict`                    — `dictionaryFour` control info + four PFC
//!      sections (`DictSectPFC::compress` does the front-coding; its `write` emits
//!      the 3-vbyte preamble, CRC8 over the metadata, the Log64 offset `Sequence`,
//!      the packed bytes, and the CRC32-C over them — every quirk `decode.rs`
//!      already reads back).
//!   4. `TriplesBitmap` (SPO)             — `triplesBitmap` control info + `bitmap_y`,
//!      `bitmap_z`, `sequence_y`, `sequence_z`, exactly as the decoder consumes them.
//!
//! Because the upstream `Hdt` struct keeps its fields private we cannot build an
//! `Hdt` and call `Hdt::write`; instead we replay `Hdt::write`'s four steps with the
//! public section writers. The bytes are byte-for-byte what `Hdt::write` would have
//! produced for the same dictionary partition + sorted triple-id list — see the
//! round-trip-vs-oracle and byte-equivalence-vs-old-path tests in
//! `tests/write_roundtrip.rs`.
//!
//! ## Dictionary partitioning (mirrors `FourSectDict::read_nt`)
//!
//! HDT splits terms into four PFC sections by the POSITIONS a term appears in:
//!
//!   * `shared`     — terms used as BOTH subject and object (subjects ∩ objects),
//!   * `subjects`   — terms used only as subjects (subjects ∖ objects),
//!   * `objects`    — terms used only as objects (objects ∖ subjects),
//!   * `predicates` — every distinct predicate term (its own numbering).
//!
//! Each section is sorted (a `BTreeSet<&str>`) and `compress`ed; the section a term
//! lands in, plus its 1-based rank within the section's id space, then resolves the
//! triple-id mapping via `FourSectDict::string_to_id` — the same numbering
//! `decode.rs::map_so` inverts on read.
//!
//! ## Term rendering (the inverse of `crate::intern_hdt_term`)
//!
//! HDT stores dictionary terms WITHOUT the N-Triples wrapping `crate::intern_hdt_term`
//! strips: IRIs bare (no angle brackets), blank nodes as `_:label`, and literals as
//! `"lexical"` / `"lexical"@lang` / `"lexical"^^<datatype>` with the lexical form
//! RAW (NOT N-Triples-escaped). [`hdt_term_string`] produces exactly that from a
//! sparq [`Dict`] id, so a term written here re-interns to the identical sparq id.

use crate::Error;
use hdt::containers::rdf::{Id as HId, Literal as HLiteral, Term as HTerm, Triple as HTriple};
use hdt::containers::ControlInfo;
use hdt::dict_sect_pfc::DictSectPFC;
use hdt::four_sect_dict::{FourSectDict, IdKind};
use hdt::header::Header;
use hdt::triples::{TripleId, TriplesBitmap};
use sparq_core::dict::{is_inline, Dict, Id, TermParts, INLINE_BASE};
use sparq_core::Graph;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

/// The PFC block size HDT writers use (hdt-cpp / hdt-java / the `hdt` crate's own
/// `read_nt`): one whole string per block start, the rest front-coded against it.
const BLOCK_SIZE: usize = 16;

/// xsd:string — the datatype sparq stores for a plain literal; HDT writes plain
/// literals WITHOUT a datatype suffix, so this one is elided when rendering.
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

/// xsd:integer — the datatype of sparq's INLINE-integer ids (ids `>= INLINE_BASE`
/// encode the integer value directly rather than storing a term, so `term_parts`
/// does not resolve them; [`hdt_term_string`] decodes them as `xsd:integer` literals,
/// matching `Dict::term`'s canonical reconstruction).
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";

/// Renders a sparq dictionary term as its HDT dictionary string — the exact inverse
/// of [`crate::intern_hdt_term`], so the round trip is the identity on sparq ids.
///
/// Returns `Err` for an RDF 1.2 triple term (`<< s p o >>`): standard HDT's term
/// model has no quoted-triple representation, so such a graph cannot be written to a
/// spec-conformant `.hdt` (this matches the read side, which never produces one).
fn hdt_term_string(dict: &Dict, id: Id) -> Result<String, Error> {
    // Inline-integer ids encode the value directly and are NOT in the record store, so
    // `term_parts` cannot resolve them. They decode to the canonical `xsd:integer`
    // literal (the same term `Dict::term` reconstructs), whose HDT string the reader's
    // `intern_hdt_term` re-interns to the identical inline id.
    if is_inline(id) {
        let value = id - INLINE_BASE;
        return Ok(format!("\"{value}\"^^<{XSD_INTEGER}>"));
    }
    match dict.term_parts(id) {
        TermParts::Iri { prefix, suffix } => {
            // HDT stores IRIs bare (no angle brackets).
            let mut s = String::with_capacity(prefix.len() + suffix.len());
            s.push_str(prefix);
            s.push_str(suffix);
            Ok(s)
        }
        TermParts::Blank(label) => Ok(format!("_:{label}")),
        TermParts::Lit {
            value,
            datatype,
            lang,
        } => {
            // Lexical form is stored RAW (the reader does NOT unescape), so we emit it
            // verbatim between quotes — never N-Triples-escaped.
            let mut s = String::with_capacity(value.len() + 16);
            s.push('"');
            s.push_str(value);
            s.push('"');
            if let Some(slot) = lang {
                // A language-tagged literal: `"lex"@tag` (the stored slot may carry an
                // RDF 1.2 `tag--dir` base direction; the reader lowercases the tag and
                // re-interns to the same slot, so emit it verbatim).
                s.push('@');
                s.push_str(slot);
            } else if datatype != XSD_STRING {
                // A datatyped (non-string) literal: `"lex"^^<datatype>`.
                s.push_str("^^<");
                s.push_str(datatype);
                s.push('>');
            }
            // else: a plain xsd:string literal — no suffix.
            Ok(s)
        }
        TermParts::Triple(_) => Err(Error::Term(
            "cannot write an RDF 1.2 triple term to HDT: the standard HDT dictionary has no \
             quoted-triple representation"
                .to_owned(),
        )),
    }
}

/// One distinct sparq term, rendered to its HDT string, tagged with the positions it
/// occupies across the graph — so the four PFC sections can be partitioned.
#[derive(Default)]
struct Roles {
    subject: bool,
    object: bool,
}

/// Builds the [`FourSectDict`] for `graph` (the dictionary partition + PFC sections)
/// and returns it alongside the per-triple HDT id list (already SPO, unsorted).
///
/// The HDT string of every term is computed ONCE and cached, so each distinct term
/// is rendered a single time even when it appears in many triples / both positions.
fn build_dict_and_triples(graph: &Graph) -> Result<(FourSectDict, Vec<TripleId>), Error> {
    // Scan the default graph once; record each term's HDT string and the positions it
    // appears in. Predicates are tracked separately (their own id space).
    let scan = graph.store.scan(&[None, None, None]);

    // sparq Id -> (cached HDT string, roles). A BTreeMap keeps the per-id render
    // memoised; the *section* sets below dedupe by string (HDT dedupes by term).
    let mut s_cache: BTreeMap<Id, String> = BTreeMap::new();
    let mut p_cache: BTreeMap<Id, String> = BTreeMap::new();
    let mut o_cache: BTreeMap<Id, String> = BTreeMap::new();
    let mut roles: BTreeMap<String, Roles> = BTreeMap::new();
    let mut predicate_terms: BTreeSet<String> = BTreeSet::new();

    // Resolve (and cache) the HDT string for a term id in a given position.
    let render = |dict: &Dict, cache: &mut BTreeMap<Id, String>, id: Id| -> Result<String, Error> {
        if let Some(s) = cache.get(&id) {
            return Ok(s.clone());
        }
        let s = hdt_term_string(dict, id)?;
        cache.insert(id, s.clone());
        Ok(s)
    };

    // First pass: collect the term strings + roles. We hold the rendered (s,p,o)
    // strings per row so the second pass needs no re-render.
    let mut rows: Vec<(String, String, String)> = Vec::with_capacity(scan.rows.len());
    for row in scan.rows.iter() {
        let [s, p, o] = scan.to_spo(row);
        let ss = render(&graph.dict, &mut s_cache, s)?;
        let ps = render(&graph.dict, &mut p_cache, p)?;
        let os = render(&graph.dict, &mut o_cache, o)?;
        roles.entry(ss.clone()).or_default().subject = true;
        roles.entry(os.clone()).or_default().object = true;
        predicate_terms.insert(ps.clone());
        rows.push((ss, ps, os));
    }

    // Partition by role: shared (S∧O), subject-only (S∖O), object-only (O∖S).
    let mut shared: BTreeSet<&str> = BTreeSet::new();
    let mut subjects_only: BTreeSet<&str> = BTreeSet::new();
    let mut objects_only: BTreeSet<&str> = BTreeSet::new();
    for (term, r) in &roles {
        match (r.subject, r.object) {
            (true, true) => shared.insert(term.as_str()),
            (true, false) => subjects_only.insert(term.as_str()),
            (false, true) => objects_only.insert(term.as_str()),
            // A term recorded with neither role cannot occur (every entry is set by a
            // subject or object insert above); skip defensively.
            (false, false) => false,
        };
    }
    let predicates: BTreeSet<&str> = predicate_terms.iter().map(String::as_str).collect();

    let dict = FourSectDict {
        shared: DictSectPFC::compress(&shared, BLOCK_SIZE),
        subjects: DictSectPFC::compress(&subjects_only, BLOCK_SIZE),
        predicates: DictSectPFC::compress(&predicates, BLOCK_SIZE),
        objects: DictSectPFC::compress(&objects_only, BLOCK_SIZE),
    };

    // Map each rendered row to its HDT triple ids via the freshly-built dictionary —
    // the same `string_to_id` numbering `decode.rs::map_so` inverts on read.
    let mut encoded: Vec<TripleId> = Vec::with_capacity(rows.len());
    for (s, p, o) in &rows {
        let sid = dict.string_to_id(s, IdKind::Subject);
        let pid = dict.string_to_id(p, IdKind::Predicate);
        let oid = dict.string_to_id(o, IdKind::Object);
        if sid == 0 || pid == 0 || oid == 0 {
            // Should be unreachable: every term was just compressed into the dict.
            return Err(Error::Term(format!(
                "term not found in freshly-built HDT dictionary ({s}, {p}, {o})",
            )));
        }
        encoded.push([sid, pid, oid]);
    }

    Ok((dict, encoded))
}

/// Builds a minimal-but-valid HDT header (the "H"): VoID/HDT statistics as an
/// N-Triples body, length-framed. Mirrors the fields the `hdt` crate's `build_header`
/// emits, but WITHOUT the file-path dependency (`build_header` canonicalises a path
/// for the base IRI; a direct in-memory encoder has no source file), so the base is a
/// stable opaque IRI. The decoder only needs `format == "ntriples"` + the byte length;
/// the body triples are informational metadata.
fn build_header(dict: &FourSectDict, num_triples: usize) -> Header {
    use hdt::vocab::*;

    // A stable, file-independent base IRI for the dataset description.
    let base = HId::Named("http://purl.org/HDT/sparq#dataset".to_owned());
    let mut body = BTreeSet::<HTriple>::new();

    // One closure, two object shapes (literal / id), so `body` is borrowed once.
    let mut add = |s: &HId, p: &str, o: HTerm| {
        body.insert(HTriple::new(s.clone(), p.to_owned(), o));
    };
    let lit = |o: String| HTerm::Literal(HLiteral::new(o));

    let d_s = dict.subjects.num_strings + dict.shared.num_strings;
    let d_o = dict.objects.num_strings + dict.shared.num_strings;

    add(&base, RDF_TYPE, lit(HDT_CONTAINER.to_owned()));
    add(&base, RDF_TYPE, lit(VOID_DATASET.to_owned()));
    add(&base, VOID_TRIPLES, lit(num_triples.to_string()));
    add(
        &base,
        VOID_PROPERTIES,
        lit(dict.predicates.num_strings.to_string()),
    );
    add(&base, VOID_DISTINCT_SUBJECTS, lit(d_s.to_string()));
    add(&base, VOID_DISTINCT_OBJECTS, lit(d_o.to_string()));

    let format_id = HId::Blank("format".to_owned());
    let dict_id = HId::Blank("dictionary".to_owned());
    let triples_id = HId::Blank("triples".to_owned());
    add(&base, HDT_FORMAT_INFORMATION, HTerm::Id(format_id.clone()));
    add(&format_id, HDT_DICTIONARY, HTerm::Id(dict_id.clone()));
    add(&format_id, HDT_TRIPLES, HTerm::Id(triples_id.clone()));
    add(
        &dict_id,
        HDT_DICT_SHARED_SO,
        lit(dict.shared.num_strings.to_string()),
    );
    add(&dict_id, HDT_DICT_MAPPING, lit("1".to_owned()));
    add(&dict_id, HDT_DICT_BLOCK_SIZE, lit(BLOCK_SIZE.to_string()));
    add(
        &triples_id,
        DC_TERMS_FORMAT,
        lit(HDT_TYPE_BITMAP.to_owned()),
    );
    add(&triples_id, HDT_NUM_TRIPLES, lit(num_triples.to_string()));
    add(&triples_id, HDT_TRIPLES_ORDER, lit("SPO".to_owned()));

    // `length` must equal the byte length of the rendered body (the reader reads
    // exactly that many bytes): render once to measure, then store it.
    let mut buf = Vec::<u8>::new();
    for triple in &body {
        // `writeln!` to a Vec is infallible.
        let _ = writeln!(buf, "{triple}");
    }
    Header {
        format: "ntriples".to_owned(),
        length: buf.len(),
        body,
    }
}

/// Writes `graph` to `w` as a bare (uncompressed) HDT archive, encoding the sections
/// DIRECTLY from sparq's in-memory dictionary + triples — no temporary N-Triples file
/// and no text parse. Section order + byte layout match [`hdt::Hdt::write`] exactly.
///
/// HDT carries a SINGLE default graph, so only the default graph's triples are
/// written; any named graphs are ignored (HDT has no place for them).
pub(crate) fn write_hdt<W: Write>(graph: &Graph, w: &mut W) -> Result<(), Error> {
    let (dict, mut encoded) = build_dict_and_triples(graph)?;

    // BitmapTriples requires SPO-sorted, deduped triple ids (the same shape
    // `Hdt::read_nt` feeds `from_triples`): the id ordering is the section ordering,
    // so sorting by id == sorting by (S,P,O) term order.
    encoded.sort_unstable();
    encoded.dedup();
    let num_triples = encoded.len();
    let triples = TriplesBitmap::from_triples(&encoded);

    let header = build_header(&dict, num_triples);

    // Replay `Hdt::write`'s four steps with the public section writers (the `Hdt`
    // struct's fields are private, so we cannot build one and call `Hdt::write`).
    // Every section writer returns a module-local error that is a `#[from]` variant
    // of `hdt::hdt::Error`, which `crate::Error` converts via its existing `From`.
    ControlInfo::global()
        .write(w)
        .map_err(hdt::hdt::Error::from)?;
    header.write(w).map_err(hdt::hdt::Error::from)?;
    dict.write(w).map_err(hdt::hdt::Error::from)?;
    triples.write(w).map_err(hdt::hdt::Error::from)?;
    w.flush()?;
    Ok(())
}
