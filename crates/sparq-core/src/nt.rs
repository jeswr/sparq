//! A byte-level N-Triples parser that interns directly into the [`Dict`], skipping the
//! intermediate `oxrdf::Term` allocation that a general parser makes for every term
//! *occurrence*. The common no-escape case interns the raw UTF-8 slice with zero
//! allocation; terms containing `\` take a correct escape-decoding slow path. Used by
//! the parallel bulk loader for the `ntriples` format (it is one statement per line, so
//! a byte buffer can be split at newlines and parsed in parallel).
//!
//! Blank-node labels are kept verbatim (file-scoped, as the loader folds one document).
//!
//! [OPUS-4.8] (sq-hxgb) RDF 1.2 triple terms `<<( s p o )>>` are accepted in the OBJECT
//! position (object-only, may nest), interned structurally to agree with the Turtle loader
//! and the engine's CONSTRUCT N-Triples serializer (which emits this form). The fast plain-
//! triple path is unchanged: a triple term is only looked for when the object byte is `<`
//! AND the next two bytes are `<(`, so a plain `<…>` IRI pays just two extra byte peeks.

use crate::dict::{Dict, Id};
use std::borrow::Cow;

/// [OPUS-4.8] (sq-7d3dj.2) Rough average N-Triples line length, in bytes, used ONLY to pre-size
/// the per-chunk triple `Vec` (here) and the per-chunk partial `Dict` (in `parse_block`) before
/// filling them — killing the reallocation/rehash churn the ingest profile flagged (~5% of the
/// parallel load). It is a pure capacity HINT: `bytes / AVG_NT_LINE_BYTES` estimates the triple
/// (and, loosely, the distinct-term) count of a chunk, but it NEVER changes the parsed triples,
/// their order, or the interned term/id bijection (pinned by
/// `parse_chunk_dict_capacity_is_pure_hint` here and `presizing_is_pure_capacity_hint` for
/// `parse_block`). Crucially, the reservation is derived ENTIRELY from
/// the real chunk length: it is `bytes.len() / AVG_NT_LINE_BYTES`, hence never exceeds
/// `bytes.len()` and cannot grow independently of the input. So — unlike HDT's independently
/// declared `num_strings` (`decode.rs`), which an attacker could set huge behind a tiny file — it
/// needs no HDT-style clamp against an untrusted count. `64` tracks the mean line of the
/// deterministic CI corpus (`sparq-bench dump`); it is NOT a lower bound on line length, so
/// shorter lines make it a slight under-reservation and longer ones a slight over-reservation —
/// either way the miss is absorbed by ordinary `Vec`/`Dict` growth, since this only reserves
/// capacity and never bounds the parsed count.
pub(crate) const AVG_NT_LINE_BYTES: usize = 64;

/// [OPUS-4.8] (sq-53s1, ASVS V5.5.2) Maximum nesting depth of RDF 1.2 triple terms
/// `<<( s p o )>>` accepted by the byte-level N-Triples/N-Quads parser before it returns a
/// clean parse error instead of recursing until the native stack overflows and aborts the
/// process. `object_term`/`triple_term` (and the no-intern `span_object`/`span_triple_term`
/// graph-scan mirror) recurse once per `<<(` nesting level on the OBJECT axis; an adversarial
/// input such as `<<( <<( … )>> )>>` repeated 10k deep would otherwise blow the stack. The cap
/// is far deeper than any real RDF 1.2 data nests (triple-term nesting beyond a handful of
/// levels is unheard of in practice) yet shallow enough to keep recursion within the stack.
/// Kept in step with spargebra's `MAX_RECURSION_DEPTH` (the SPARQL side of the same control).
const MAX_TRIPLE_TERM_DEPTH: usize = 128;

/// [OPUS-4.8] (sq-25r3) The identity of an N-Quads quad's GRAPH field (4th term): an absolute
/// IRI or a blank-node label, decoded (escapes resolved) so two byte-different-but-equal graph
/// references route to the SAME graph. `None` (the default graph) is represented by the absence
/// of a key, not a variant here. Used purely as a routing key by the chunk-parallel N-Quads
/// loader — the graph name itself never enters any per-graph dictionary; it becomes the
/// `oxrdf::Term` stored in the parent graph's `named` vec. `Ord`/`Hash` give a deterministic
/// per-graph grouping independent of parse order.
#[cfg(feature = "parallel")]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GraphKey {
    Iri(String),
    Blank(String),
}

#[cfg(feature = "parallel")]
impl GraphKey {
    /// Rebuilds the `oxrdf::Term` for the parent graph's `named` entry. `new_unchecked` is sound:
    /// these strings came straight out of a successful parse (the same path the serial loader's
    /// `Term::NamedNode`/`Term::BlankNode` take).
    pub fn to_term(&self) -> oxrdf::Term {
        match self {
            GraphKey::Iri(s) => oxrdf::Term::NamedNode(oxrdf::NamedNode::new_unchecked(s)),
            GraphKey::Blank(s) => oxrdf::Term::BlankNode(oxrdf::BlankNode::new_unchecked(s)),
        }
    }
}

/// [OPUS-4.8] (sq-25r3) Relabels an error raised by the SHARED N-Triples term parsers (`term`,
/// `object_term`, `scan_delim`, `decode`, …) so it reads `N-Quads:` when surfaced from the
/// N-Quads fast path — the shared code emits the `N-Triples:` prefix, which is misleading to a
/// caller that handed in N-Quads. Only the leading prefix is rewritten; the byte offset and the
/// rest of the message are preserved.
#[cfg(feature = "parallel")]
fn as_nquads_err(e: String) -> String {
    match e.strip_prefix("N-Triples:") {
        Some(rest) => format!("N-Quads:{rest}"),
        None => e,
    }
}

/// [OPUS-4.8] (sq-25r3) Parses a slice of complete N-QUADS lines, ROUTING each quad's triple to a
/// per-graph partial bucket keyed by its 4th (graph) field — the byte-level analogue of
/// [`parse_chunk`] for datasets. N-Quads is one statement per line, so a byte buffer can be split
/// at newline boundaries and each range parsed independently; this is the per-range worker.
///
/// Returns, IN FIRST-OCCURRENCE ORDER within this byte range, `(Option<GraphKey>, Dict, triples)`
/// buckets — `None` is the default graph. Each bucket carries its OWN partial dict (S/P/O interned
/// locally), mirroring the serial loader's one-dict-per-graph construction; the graph NAME is the
/// routing key and is never interned into any of these dicts. Errors carry the byte offset.
///
/// Cross-range correctness: blank-node labels (subject/object AND graph) are kept verbatim, so the
/// dataset-scoped merge (per graph, by label, exactly as [`parse_chunk`]) unifies them across
/// ranges — see the loader (`load_nquads_parallel`).
#[cfg(feature = "parallel")]
#[allow(clippy::type_complexity)]
pub fn parse_quads_chunk(bytes: &[u8]) -> Result<Vec<(Option<GraphKey>, Dict, Vec<[Id; 3]>)>, String> {
    // `buckets` is in first-occurrence order of graph keys (push-on-first-sight); `index` maps a
    // key to its slot so repeated references to the same graph append to the one existing bucket.
    use std::collections::HashMap;
    let mut buckets: Vec<(Option<GraphKey>, Dict, Vec<[Id; 3]>)> = Vec::new();
    let mut index: HashMap<Option<GraphKey>, usize> = HashMap::new();
    let n = bytes.len();
    let mut i = 0;
    while i < n {
        i = skip_ws(bytes, i);
        if i >= n {
            break;
        }
        if bytes[i] == b'#' {
            i = next_line(bytes, i);
            continue;
        }
        // We must know the GRAPH (which bucket dict to intern into) before interning S/P/O, but the
        // graph is the LAST field. So first locate the optional graph term by SPAN-scanning S/P/O
        // (a no-intern walk, the same cost as a parse), resolve the bucket, THEN intern S/P/O into
        // that bucket's dict by re-scanning from their start offsets. The span walk avoids any
        // throwaway interning into a wrong dict.
        let s_start = i;
        let j = span_term(bytes, i)?;
        i = skip_ws(bytes, j);
        let p_start = i;
        let j = span_term(bytes, i)?;
        i = skip_ws(bytes, j);
        let o_start = i;
        let j = span_object(bytes, i)?;
        i = skip_ws(bytes, j);
        // Optional graph term, then the mandatory `.`.
        let graph = if i < n && bytes[i] == b'.' {
            i += 1;
            None
        } else {
            let g = graph_key(bytes, i)?;
            let jg = span_term(bytes, i)?;
            i = skip_ws(bytes, jg);
            if i >= n || bytes[i] != b'.' {
                return Err(format!("N-Quads: expected '.' at byte {i}"));
            }
            i += 1;
            Some(g)
        };
        // Resolve (or create) the bucket for this graph, then intern S/P/O into ITS dict.
        let slot = *index.entry(graph.clone()).or_insert_with(|| {
            buckets.push((graph.clone(), Dict::new(), Vec::new()));
            buckets.len() - 1
        });
        let (_, dict, triples) = &mut buckets[slot];
        // S/P/O were already span-validated above; the only errors the shared (re-scanning)
        // intern path can still raise here are escape/UTF-8 decode failures, which carry the
        // N-Triples prefix from the shared term parser. Relabel them to N-Quads so a caller of
        // the N-Quads fast path never sees a misleading `N-Triples:` message (sq-25r3).
        let (s, _) = term(bytes, s_start, dict).map_err(as_nquads_err)?;
        let (p, _) = term(bytes, p_start, dict).map_err(as_nquads_err)?;
        let (o, _) = object_term(bytes, o_start, dict).map_err(as_nquads_err)?;
        triples.push([s, p, o]);
    }
    Ok(buckets)
}

/// [OPUS-4.8] (sq-25r3) Parses just the graph (4th) field of an N-Quads quad into a [`GraphKey`]
/// (an absolute IRI or a blank-node label), decoding escapes so the key is the graph's true
/// identity. Literals are NOT valid in graph position and are rejected.
#[cfg(feature = "parallel")]
fn graph_key(b: &[u8], i: usize) -> Result<GraphKey, String> {
    match b.get(i) {
        Some(b'<') => {
            let (start, end, esc, _next) = scan_delim(b, i, b'>').map_err(as_nquads_err)?;
            Ok(GraphKey::Iri(decode(&b[start..end], esc).map_err(as_nquads_err)?.into_owned()))
        }
        Some(b'_') => {
            if b.get(i + 1) != Some(&b':') {
                return Err(format!("N-Quads: bad blank node graph at byte {i}"));
            }
            let start = i + 2;
            let j = blank_label_end(b, start);
            Ok(GraphKey::Blank(str_of(&b[start..j])?.to_owned()))
        }
        _ => Err(format!("N-Quads: graph name must be an IRI or blank node at byte {i}")),
    }
}

/// [OPUS-4.8] (sq-25r3) Returns the index just past a term WITHOUT interning — used to locate the
/// optional graph field before deciding which bucket dict to intern S/P/O into (the caller then
/// re-scans from the start offset to intern). Mirrors [`term`]'s grammar.
#[cfg(feature = "parallel")]
fn span_term(b: &[u8], i: usize) -> Result<usize, String> {
    match b.get(i) {
        // [OPUS-4.8] (sq-87bq) Triple terms are OBJECT-ONLY (RDF 1.2); a `<<(` reaching the
        // subject/predicate (or graph) span walk is a non-object triple term — reject with a
        // precise message, mirroring `term`. See the grammar note there (prods. [7]/[8]/[9]).
        Some(b'<') if b.get(i + 1) == Some(&b'<') && b.get(i + 2) == Some(&b'(') => Err(format!(
            "N-Quads: RDF 1.2 triple term `<<( … )>>` is only valid in OBJECT position \
             (subject/predicate/graph must be an IRI or blank node), at byte {i}"
        )),
        Some(b'<') => {
            let (_s, _e, _esc, next) = scan_delim(b, i, b'>')?;
            Ok(next)
        }
        Some(b'_') => {
            if b.get(i + 1) != Some(&b':') {
                return Err(format!("N-Quads: bad blank node at byte {i}"));
            }
            Ok(blank_label_end(b, i + 2))
        }
        Some(b'"') => span_literal(b, i),
        _ => Err(format!("N-Quads: unexpected term start at byte {i}")),
    }
}

/// [OPUS-4.8] (sq-25r3) Index just past an OBJECT-position term (which may be an RDF 1.2 triple
/// term `<<( … )>>`). Mirrors [`object_term`]'s grammar.
#[cfg(feature = "parallel")]
fn span_object(b: &[u8], i: usize) -> Result<usize, String> {
    span_object_depth(b, i, 0)
}

/// [OPUS-4.8] (sq-53s1) `span_object` carrying the triple-term nesting `depth` so the no-intern
/// graph-scan walk is bounded identically to the interning path (`object_term_depth`).
#[cfg(feature = "parallel")]
fn span_object_depth(b: &[u8], i: usize, depth: usize) -> Result<usize, String> {
    if b.get(i) == Some(&b'<') && b.get(i + 1) == Some(&b'<') && b.get(i + 2) == Some(&b'(') {
        span_triple_term(b, i, depth)
    } else {
        span_term(b, i)
    }
}

/// [OPUS-4.8] (sq-25r3) Index just past an RDF 1.2 triple term `<<( s p o )>>`, recursing through
/// its nested object. Mirrors [`triple_term`].
#[cfg(feature = "parallel")]
fn span_triple_term(b: &[u8], i: usize, depth: usize) -> Result<usize, String> {
    // [OPUS-4.8] (sq-53s1, ASVS V5.5.2) Mirror the interning path's depth bound (`triple_term`).
    if depth >= MAX_TRIPLE_TERM_DEPTH {
        return Err(format!(
            "N-Quads: RDF 1.2 triple term nested more than {MAX_TRIPLE_TERM_DEPTH} levels deep at byte {i}"
        ));
    }
    let mut j = skip_ws(b, i + 3);
    let k = span_term(b, j)?;
    j = skip_ws(b, k);
    let k = span_term(b, j)?;
    j = skip_ws(b, k);
    let k = span_object_depth(b, j, depth + 1)?;
    j = skip_ws(b, k);
    if b.get(j) == Some(&b')') && b.get(j + 1) == Some(&b'>') && b.get(j + 2) == Some(&b'>') {
        Ok(j + 3)
    } else {
        Err(format!("N-Quads: expected ')>>' closing triple term at byte {j}"))
    }
}

/// [OPUS-4.8] (sq-25r3) Span of a literal (value + optional `^^<dt>` / `@lang`), returning the
/// index just past it. Mirrors [`literal`]'s grammar without interning.
#[cfg(feature = "parallel")]
fn span_literal(b: &[u8], i: usize) -> Result<usize, String> {
    let (_vs, _ve, _vesc, after) = scan_delim(b, i, b'"')?;
    match b.get(after) {
        Some(b'^') if b.get(after + 1) == Some(&b'^') => {
            let open = after + 2;
            if b.get(open) != Some(&b'<') {
                return Err(format!("N-Quads: bad datatype at byte {open}"));
            }
            let (_ds, _de, _desc, next) = scan_delim(b, open, b'>')?;
            Ok(next)
        }
        Some(b'@') => {
            let mut j = after + 1;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'-') {
                j += 1;
            }
            Ok(j)
        }
        _ => Ok(after),
    }
}

/// Parses a slice of complete N-Triples lines, interning each term and returning the
/// id-triples. Errors carry the byte offset.
pub fn parse_chunk(bytes: &[u8], dict: &mut Dict) -> Result<Vec<[Id; 3]>, String> {
    // [OPUS-4.8] (sq-7d3dj.2) Pre-size the id-triple output from the chunk's byte length so the
    // Vec does not re-grow (and copy) through ~log2(triples) doublings while parsing. Pure
    // capacity hint (see `AVG_NT_LINE_BYTES`); a tiny chunk (`< AVG_NT_LINE_BYTES` bytes) yields
    // `with_capacity(0)` == no allocation, and any slack is dropped with this Vec once the caller
    // remaps it into the final store — so it never reaches the `store_bytes_per_triple` ratchet.
    let mut out = Vec::with_capacity(bytes.len() / AVG_NT_LINE_BYTES);
    let n = bytes.len();
    let mut i = 0;
    while i < n {
        i = skip_ws(bytes, i);
        if i >= n {
            break;
        }
        if bytes[i] == b'#' {
            i = next_line(bytes, i);
            continue;
        }
        let (s, j) = term(bytes, i, dict)?;
        i = skip_ws(bytes, j);
        let (p, j) = term(bytes, i, dict)?;
        i = skip_ws(bytes, j);
        // [OPUS-4.8] (sq-hxgb) Only the OBJECT position may be an RDF 1.2 triple term
        // `<<( s p o )>>` (matching the Turtle loader / RDF 1.2 grammar — triple terms
        // are object-only and may nest through their own object).
        let (o, j) = object_term(bytes, i, dict)?;
        i = skip_ws(bytes, j);
        if i >= n || bytes[i] != b'.' {
            return Err(format!("N-Triples: expected '.' at byte {i}"));
        }
        i += 1;
        out.push([s, p, o]);
    }
    Ok(out)
}

#[inline]
fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && matches!(b[i], b' ' | b'\t' | b'\r' | b'\n') {
        i += 1;
    }
    i
}

#[inline]
fn next_line(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && b[i] != b'\n' {
        i += 1;
    }
    i
}

#[inline]
fn str_of(raw: &[u8]) -> Result<&str, String> {
    std::str::from_utf8(raw).map_err(|_| "N-Triples: invalid UTF-8".to_string())
}

fn term(b: &[u8], i: usize, dict: &mut Dict) -> Result<(Id, usize), String> {
    match b.get(i) {
        // [OPUS-4.8] (sq-87bq) A `<<(` here is an RDF 1.2 triple term in a NON-object
        // position (subject, predicate, or a triple term's own subject/predicate). The RDF
        // 1.2 N-Triples grammar makes triple terms OBJECT-ONLY — `subject ::= IRIREF |
        // BLANK_NODE_LABEL` (prod. [7]) and `predicate ::= IRIREF` (prod. [8]); only
        // `object ::= IRIREF | BLANK_NODE_LABEL | literal | tripleTerm` (prod. [9]) admits a
        // `tripleTerm` (prod. [11]). (This differs from the legacy RDF-star CG `<< s p o >>`
        // QUOTED-triple syntax, which DID allow subject quoting; RDF 1.2 deliberately dropped
        // that.) Reject with a precise message instead of the generic "unexpected term start".
        Some(b'<') if b.get(i + 1) == Some(&b'<') && b.get(i + 2) == Some(&b'(') => Err(format!(
            "N-Triples: RDF 1.2 triple term `<<( … )>>` is only valid in OBJECT position \
             (subject/predicate must be an IRI or blank node), at byte {i}"
        )),
        Some(b'<') => iri(b, i, dict),
        Some(b'_') => blank(b, i, dict),
        Some(b'"') => literal(b, i, dict),
        _ => Err(format!("N-Triples: unexpected term start at byte {i}")),
    }
}

/// [OPUS-4.8] (sq-hxgb) Parses a term in the OBJECT position, where (and only where) RDF 1.2
/// N-Triples permits a triple term `<<( s p o )>>`. The `<<(` opener disambiguates a triple
/// term from a plain IRIREF (`<…>`); anything else falls through to the shared [`term`] path.
fn object_term(b: &[u8], i: usize, dict: &mut Dict) -> Result<(Id, usize), String> {
    object_term_depth(b, i, dict, 0)
}

/// [OPUS-4.8] (sq-53s1) `object_term` carrying the current triple-term nesting `depth` so a
/// pathologically nested `<<( … )>>` chain returns a clean parse error at `MAX_TRIPLE_TERM_DEPTH`
/// instead of overflowing the stack. `depth` is the number of `<<(` openers already entered above
/// this call (0 at the top-level object position).
fn object_term_depth(b: &[u8], i: usize, dict: &mut Dict, depth: usize) -> Result<(Id, usize), String> {
    if b.get(i) == Some(&b'<') && b.get(i + 1) == Some(&b'<') && b.get(i + 2) == Some(&b'(') {
        triple_term(b, i, dict, depth)
    } else {
        term(b, i, dict)
    }
}

/// [OPUS-4.8] (sq-hxgb) Parses an RDF 1.2 triple term `<<( s p o )>>` and interns it
/// STRUCTURALLY (subject/predicate/object interned first, then the triple by their ids — the
/// same `Dict::intern_triple_ids` path `intern(&Term::Triple(_))` and the Turtle loader take,
/// so the two loaders content-address triple terms identically). The subject is an IRI or
/// blank node, the predicate an IRI, and the object any term — including a NESTED triple term
/// (recursion through `object_term`). Returns the interned id and the index past the `>>`.
fn triple_term(b: &[u8], i: usize, dict: &mut Dict, depth: usize) -> Result<(Id, usize), String> {
    // [OPUS-4.8] (sq-53s1, ASVS V5.5.2) Bound the triple-term nesting before recursing into the
    // object, so deeply-nested input yields a clean parse error rather than a stack overflow.
    if depth >= MAX_TRIPLE_TERM_DEPTH {
        return Err(format!(
            "N-Triples: RDF 1.2 triple term nested more than {MAX_TRIPLE_TERM_DEPTH} levels deep at byte {i}"
        ));
    }
    // Skip the `<<(` opener.
    let mut j = skip_ws(b, i + 3);
    let (s, k) = term(b, j, dict)?;
    j = skip_ws(b, k);
    let (p, k) = term(b, j, dict)?;
    j = skip_ws(b, k);
    let (o, k) = object_term_depth(b, j, dict, depth + 1)?;
    j = skip_ws(b, k);
    // Closer `)>>`.
    if b.get(j) == Some(&b')') && b.get(j + 1) == Some(&b'>') && b.get(j + 2) == Some(&b'>') {
        Ok((dict.intern_triple_ids([s, p, o]), j + 3))
    } else {
        Err(format!("N-Triples: expected ')>>' closing triple term at byte {j}"))
    }
}

/// Scans `<...>` returning the (escaped?) content range and the index past `>`.
fn scan_delim(b: &[u8], open: usize, close: u8) -> Result<(usize, usize, bool, usize), String> {
    let start = open + 1;
    let mut j = start;
    let mut esc = false;
    while j < b.len() && b[j] != close {
        if b[j] == b'\\' {
            esc = true;
            j += 1;
        }
        j += 1;
    }
    if j >= b.len() {
        return Err(format!("N-Triples: unterminated delimiter from byte {open}"));
    }
    Ok((start, j, esc, j + 1))
}

fn decode<'a>(raw: &'a [u8], esc: bool) -> Result<Cow<'a, str>, String> {
    let s = str_of(raw)?;
    if esc {
        Ok(Cow::Owned(unescape(s)?))
    } else {
        Ok(Cow::Borrowed(s))
    }
}

// [OPUS-4.8] (sq-7d3dj.18) Per-thread `scheme://authority/` prefix memo for the opt-in IRI
// validation fast path. A thread-local is sound under the rayon chunk-parallel loader — each
// worker keeps its own ring — and needs no clearing between parses: every memoized prefix was
// accepted by `oxiri`, and IRI validity is a pure function of the bytes, so a retained (or
// cross-parse) prefix can only change SPEED, never the accept/reject answer.
#[cfg(feature = "iri-fast")]
thread_local! {
    static IRI_MEMO: std::cell::RefCell<crate::iri::IriPrefixMemo> =
        std::cell::RefCell::new(crate::iri::IriPrefixMemo::new());
}

fn iri(b: &[u8], i: usize, dict: &mut Dict) -> Result<(Id, usize), String> {
    let (start, end, esc, next) = scan_delim(b, i, b'>')?;
    let s = decode(&b[start..end], esc)?;
    // [OPUS-4.8] (sq-7d3dj.18) When the opt-in `iri-fast` feature is on, validate every IRIREF
    // against RFC-3987 (fast path in front of `oxiri`) so the byte-level loader rejects the
    // malformed IRIs the serial oxttl path already rejects — reaching conformance parity at
    // fast-path cost. OFF by default: the loader keeps its current unvalidated-fast behaviour.
    #[cfg(feature = "iri-fast")]
    {
        let valid = IRI_MEMO.with(|m| m.borrow_mut().validate(&s));
        if !valid {
            return Err(format!("N-Triples: invalid IRI <{}> at byte {}", s, i));
        }
    }
    Ok((dict.intern_iri(&s), next))
}

/// [OPUS-4.8] (sq-25r3) End index (exclusive) of a `BLANK_NODE_LABEL` body starting at `start`
/// (the byte after `_:`). Per the N-Triples/N-Quads grammar a label may contain INTERIOR `.`
/// (`(PN_CHARS | '.')* PN_CHARS`) but must not END in `.`, so the scan runs to the next
/// whitespace and then backs the cursor over any trailing `.` bytes — which are the statement
/// terminator, not part of the label. This matches the serial oxttl reference exactly
/// (`_:a.b` → label `a.b`; `_:abc.` → label `abc`, `.` is the terminator) and is the SINGLE
/// source of truth shared by [`blank`] (interning), [`span_term`] (no-intern span), and
/// [`graph_key`] (graph field) so the three never diverge on dotted labels.
fn blank_label_end(b: &[u8], start: usize) -> usize {
    let mut j = start;
    while j < b.len() && !matches!(b[j], b' ' | b'\t' | b'\r' | b'\n') {
        j += 1;
    }
    while j > start && b[j - 1] == b'.' {
        j -= 1;
    }
    j
}

fn blank(b: &[u8], i: usize, dict: &mut Dict) -> Result<(Id, usize), String> {
    if b.get(i + 1) != Some(&b':') {
        return Err(format!("N-Triples: bad blank node at byte {i}"));
    }
    let start = i + 2;
    let j = blank_label_end(b, start);
    Ok((dict.intern_blank(str_of(&b[start..j])?), j))
}

const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const RDF_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";
const RDF_DIR_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#dirLangString";

/// [OPUS-4.8] (sq-langcase / #1119) ASCII-lowercases a parsed language tag, BORROWING the input
/// when it is already all-lowercase (the common case) so the no-uppercase fast path stays
/// allocation-free. RDF language tags are ASCII (`(PN_CHARS_BASE)('-' alnum*)*` plus the RDF 1.2
/// `--ltr`/`--rtl` direction), so `make_ascii_lowercase` is the correct, locale-independent fold —
/// the SAME normalisation `oxrdf`/`oxttl` apply, keeping the byte parser's stored language slot
/// byte-identical to the oxttl paths' for the same tag.
fn lowercase_lang(raw: &str) -> Cow<'_, str> {
    if raw.bytes().any(|c| c.is_ascii_uppercase()) {
        Cow::Owned(raw.to_ascii_lowercase())
    } else {
        Cow::Borrowed(raw)
    }
}

fn literal(b: &[u8], i: usize, dict: &mut Dict) -> Result<(Id, usize), String> {
    let (vstart, vend, vesc, after) = scan_delim(b, i, b'"')?;
    let value = decode(&b[vstart..vend], vesc)?;
    match b.get(after) {
        // ^^<datatype>
        Some(b'^') if b.get(after + 1) == Some(&b'^') => {
            let open = after + 2;
            if b.get(open) != Some(&b'<') {
                return Err(format!("N-Triples: bad datatype at byte {open}"));
            }
            let (dstart, dend, desc, next) = scan_delim(b, open, b'>')?;
            let dt = decode(&b[dstart..dend], desc)?;
            Ok((dict.intern_lit(&value, &dt, None), next))
        }
        // @lang
        Some(b'@') => {
            let lstart = after + 1;
            let mut j = lstart;
            while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'-') {
                j += 1;
            }
            // RDF 1.2 `@lang--dir` keeps the combined slot (the dict's stored encoding)
            // but is typed rdf:dirLangString.
            let raw = str_of(&b[lstart..j])?;
            let dt = if raw.contains("--") { RDF_DIR_LANG_STRING } else { RDF_LANG_STRING };
            // [OPUS-4.8] (sq-langcase / KamiQuasi #1119) NORMALISE the language tag to ASCII
            // lowercase, matching the oxttl/oxrdf paths (`Literal::new_language_tagged_literal`
            // lowercases via `make_ascii_lowercase`; oxttl's Turtle/N-Quads/TriG recognisers call
            // `language.to_ascii_lowercase()` before interning). RDF 1.1/1.2 language tags are
            // case-INSENSITIVE, so `"x"@en-US` and `"x"@en-us` are the SAME RDF term and must
            // intern to the SAME dict id (term identity is byte-equality of the stored slot).
            // Without this the byte-level N-Triples/N-Quads fast path preserved the written case
            // while every oxttl path lowercased it — a format-dependent inconsistency (a Turtle
            // doc lowercased, the same triple in N-Triples did not) AND a latent term-identity
            // bug (case-variant tags interned as two distinct terms). The `--dir` suffix is only
            // ever `ltr`/`rtl` (already lowercase) so lowercasing the whole slot is equivalent to
            // lowercasing just the BCP47 language part. The all-lowercase case (the overwhelming
            // majority) borrows with no allocation; only a tag carrying uppercase pays one alloc.
            let lang = lowercase_lang(raw);
            Ok((dict.intern_lit(&value, dt, Some(&lang)), j))
        }
        // simple literal -> xsd:string
        _ => Ok((dict.intern_lit(&value, XSD_STRING, None), after)),
    }
}

/// Decodes N-Triples ECHAR (`\t \b \n \r \f \" \' \\`) and UCHAR (`\uXXXX`, `\UXXXXXXXX`).
fn unescape(s: &str) -> Result<String, String> {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('t') => out.push('\t'),
            Some('b') => out.push('\u{8}'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('f') => out.push('\u{C}'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('\\') => out.push('\\'),
            Some('u') => out.push(hex(&mut it, 4)?),
            Some('U') => out.push(hex(&mut it, 8)?),
            _ => return Err("N-Triples: bad escape".into()),
        }
    }
    Ok(out)
}

fn hex(it: &mut std::str::Chars<'_>, n: usize) -> Result<char, String> {
    let mut cp = 0u32;
    for _ in 0..n {
        let d = it.next().and_then(|c| c.to_digit(16)).ok_or("N-Triples: bad \\u escape")?;
        cp = cp * 16 + d;
    }
    char::from_u32(cp).ok_or_else(|| "N-Triples: invalid code point".into())
}

// [OPUS-4.8] (sq-hxgb) Triple-term parsing in the OBJECT position: structural interning,
// nesting, blank-node components, fast-path-unchanged, and the rejection cases.
#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::{BlankNode, Literal, NamedNode, Term, Triple};

    fn iri(s: &str) -> Term {
        Term::NamedNode(NamedNode::new_unchecked(s))
    }

    #[test]
    fn triple_term_object_parses_and_interns_structurally() {
        // The exact byte form the CONSTRUCT serializer emits.
        let nt = b"<http://ex/r1> <http://ex/reifies> <<( <http://ex/alice> <http://ex/age> \"30\"^^<http://www.w3.org/2001/XMLSchema#integer> )>> .\n";
        let mut d = Dict::new();
        let t = parse_chunk(nt, &mut d).unwrap();
        assert_eq!(t.len(), 1);
        let [s, p, o] = t[0];
        assert_eq!(d.term(s), iri("http://ex/r1"));
        assert_eq!(d.term(p), iri("http://ex/reifies"));
        // Object is a structural Term::Triple — not a literal or opaque IRI.
        let want = Term::Triple(Box::new(Triple::new(
            NamedNode::new_unchecked("http://ex/alice"),
            NamedNode::new_unchecked("http://ex/age"),
            Term::Literal(Literal::new_typed_literal("30", NamedNode::new_unchecked("http://www.w3.org/2001/XMLSchema#integer"))),
        )));
        assert_eq!(d.term(o), want);
        // Content-addressed: re-interning the identical term shares the id.
        assert_eq!(d.lookup(&want), o);
    }

    #[test]
    fn nested_and_blank_node_triple_terms_parse() {
        let nt = b"<http://ex/r> <http://ex/p> <<( _:b0 <http://ex/q> <<( <http://ex/a> <http://ex/b> <http://ex/c> )>> )>> .\n";
        let mut d = Dict::new();
        let t = parse_chunk(nt, &mut d).unwrap();
        assert_eq!(t.len(), 1);
        let inner = Term::Triple(Box::new(Triple::new(
            NamedNode::new_unchecked("http://ex/a"),
            NamedNode::new_unchecked("http://ex/b"),
            iri("http://ex/c"),
        )));
        let outer = Term::Triple(Box::new(Triple::new(
            BlankNode::new_unchecked("b0"),
            NamedNode::new_unchecked("http://ex/q"),
            inner,
        )));
        assert_eq!(d.term(t[0][2]), outer);
    }

    #[test]
    fn plain_triples_unaffected() {
        // A plain IRI object (`<…>`) must NOT be mistaken for a triple term, and a literal
        // object beginning with `<`-free bytes parses as before.
        let nt = b"<http://ex/s> <http://ex/p> <http://ex/o> .\n<http://ex/s> <http://ex/p> \"v\" .\n";
        let mut d = Dict::new();
        let t = parse_chunk(nt, &mut d).unwrap();
        assert_eq!(t.len(), 2);
        assert_eq!(d.term(t[0][2]), iri("http://ex/o"));
        assert_eq!(d.term(t[1][2]), Term::Literal(Literal::new_simple_literal("v")));
    }

    // [OPUS-4.8] (sq-langcase / KamiQuasi #1119) The byte-level N-Triples/N-Quads parser
    // NORMALISES language tags to ASCII lowercase, matching every oxttl path (the Turtle /
    // N-Quads / TriG recognisers + `oxrdf::Literal::new_language_tagged_literal`). Previously the
    // byte path PRESERVED the written case (`en-US`) while oxttl lowercased it (`en-us`), giving a
    // format-dependent result for the SAME triple. RDF 1.1/1.2 language tags are case-insensitive,
    // so the lowercased slot is the canonical RDF term — and the regression target is twofold:
    // (a) cross-format consistency, (b) `"x"@en-US` and `"x"@en-us` interning to the SAME id.
    #[test]
    fn language_tag_lowercased_to_match_oxttl() {
        let nt = b"<http://ex/s> <http://ex/p> \"hi\"@en-US .\n";
        let mut d = Dict::new();
        let t = parse_chunk(nt, &mut d).unwrap();
        assert_eq!(t.len(), 1);
        // The stored term carries the lowercased tag — byte-identical to what oxttl produces.
        let want = Term::Literal(Literal::new_language_tagged_literal_unchecked("hi", "en-us"));
        assert_eq!(d.term(t[0][2]), want);
        // And it equals what the checked oxrdf constructor (which lowercases) yields from the
        // mixed-case tag, i.e. the byte parser and the oxttl/oxrdf paths agree.
        let oxttl_like =
            Term::Literal(Literal::new_language_tagged_literal("hi", "en-US").unwrap());
        assert_eq!(d.term(t[0][2]), oxttl_like);
    }

    #[test]
    fn case_variant_language_tags_intern_to_same_id() {
        // Two lines whose ONLY difference is the language-tag case must content-address to one id
        // (term identity is case-insensitive on the tag) — they did NOT before this fix.
        let nt = b"<http://ex/s> <http://ex/p> \"hi\"@en-US .\n<http://ex/s2> <http://ex/p> \"hi\"@en-us .\n";
        let mut d = Dict::new();
        let t = parse_chunk(nt, &mut d).unwrap();
        assert_eq!(t.len(), 2);
        assert_eq!(t[0][2], t[1][2], "case-variant language tags must share one dict id");
    }

    #[test]
    fn directional_language_tag_lowercases_lang_part_only() {
        // RDF 1.2 `@lang--dir`: the BCP47 language part lowercases; the direction (`ltr`/`rtl`,
        // already lowercase) is unchanged. The stored slot must match the oxttl encoding exactly.
        let nt = b"<http://ex/s> <http://ex/p> \"hi\"@en-US--ltr .\n";
        let mut d = Dict::new();
        let t = parse_chunk(nt, &mut d).unwrap();
        assert_eq!(t.len(), 1);
        let want = Term::Literal(
            Literal::new_directional_language_tagged_literal_unchecked(
                "hi",
                "en-us",
                oxrdf::BaseDirection::Ltr,
            ),
        );
        assert_eq!(d.term(t[0][2]), want);
    }

    // The N-Quads byte fast path interns S/P/O through the SAME `literal` path, so it lowercases
    // too — guarding against a future divergence between the two byte loaders.
    #[cfg(feature = "parallel")]
    #[test]
    fn nquads_language_tag_lowercased() {
        let nq = b"<http://ex/s> <http://ex/p> \"hi\"@en-US <http://ex/g> .\n";
        let buckets = parse_quads_chunk(nq).unwrap();
        assert_eq!(buckets.len(), 1);
        let (_g, dict, triples) = &buckets[0];
        let want = Term::Literal(Literal::new_language_tagged_literal_unchecked("hi", "en-us"));
        assert_eq!(dict.term(triples[0][2]), want);
    }

    #[test]
    fn already_lowercase_language_tag_unchanged() {
        // The common all-lowercase tag is untouched (and the fast borrow path is exercised).
        let nt = b"<http://ex/s> <http://ex/p> \"hi\"@fr .\n";
        let mut d = Dict::new();
        let t = parse_chunk(nt, &mut d).unwrap();
        let want = Term::Literal(Literal::new_language_tagged_literal_unchecked("hi", "fr"));
        assert_eq!(d.term(t[0][2]), want);
    }

    #[test]
    fn unterminated_triple_term_errors() {
        let nt = b"<http://ex/s> <http://ex/p> <<( <http://ex/a> <http://ex/b> <http://ex/c> .\n";
        let mut d = Dict::new();
        assert!(parse_chunk(nt, &mut d).is_err(), "missing )>> must error, not silently accept");
    }

    #[test]
    fn triple_term_only_in_object_position() {
        // [OPUS-4.8] (sq-87bq) RDF 1.2 makes triple terms OBJECT-ONLY (N-Triples grammar:
        // `subject ::= IRIREF | BLANK_NODE_LABEL` [7], `predicate ::= IRIREF` [8], only
        // `object` [9] admits a `tripleTerm` [11]). The parser must REJECT a `<<( … )>>` in
        // subject OR predicate position with a precise, position-aware message — not misparse
        // it, and not silently accept the legacy RDF-star CG `<< … >>` subject-quoting that
        // RDF 1.2 dropped.

        // Subject position.
        let subj = b"<<( <http://ex/a> <http://ex/b> <http://ex/c> )>> <http://ex/p> <http://ex/o> .\n";
        let mut d = Dict::new();
        let e = parse_chunk(subj, &mut d).expect_err("triple term in subject position must be rejected");
        assert!(e.contains("only valid in OBJECT position"), "message must explain the object-only rule, got: {e}");

        // Predicate position.
        let pred = b"<http://ex/s> <<( <http://ex/a> <http://ex/b> <http://ex/c> )>> <http://ex/o> .\n";
        let mut d = Dict::new();
        let e = parse_chunk(pred, &mut d).expect_err("triple term in predicate position must be rejected");
        assert!(e.contains("only valid in OBJECT position"), "message must explain the object-only rule, got: {e}");

        // A triple term's OWN subject must also be a plain term (recursion goes through
        // `term`, not `object_term`): a nested `<<( … )>>` in the inner subject is rejected.
        let inner_subj = b"<http://ex/s> <http://ex/p> <<( <<( <http://ex/a> <http://ex/b> <http://ex/c> )>> <http://ex/q> <http://ex/r> )>> .\n";
        let mut d = Dict::new();
        assert!(
            parse_chunk(inner_subj, &mut d).is_err(),
            "a triple term in a triple term's SUBJECT slot must be rejected (object-only nesting)"
        );

        // Sanity: the same triple term in OBJECT position parses fine (the rejection is
        // strictly position-driven, not a blanket ban).
        let obj = b"<http://ex/s> <http://ex/p> <<( <http://ex/a> <http://ex/b> <http://ex/c> )>> .\n";
        let mut d = Dict::new();
        assert!(parse_chunk(obj, &mut d).is_ok(), "the same triple term in object position must parse");
    }

    // [OPUS-4.8] (sq-53s1, ASVS V5.5.2) Builds one N-Triples line whose OBJECT is `depth` levels
    // of nested RDF 1.2 triple terms `<<( s p <<( … )>> )>>`. Each level wraps the previous object
    // in a fresh triple term; the innermost object is a plain IRI.
    fn nested_triple_term_line(depth: usize) -> Vec<u8> {
        let mut line = b"<http://ex/s> <http://ex/p> ".to_vec();
        for _ in 0..depth {
            line.extend_from_slice(b"<<( <http://ex/a> <http://ex/b> ");
        }
        line.extend_from_slice(b"<http://ex/leaf>");
        for _ in 0..depth {
            line.extend_from_slice(b" )>>");
        }
        line.extend_from_slice(b" .\n");
        line
    }

    #[test]
    fn deeply_nested_triple_term_returns_clean_error_not_stack_overflow() {
        // 10k levels of `<<( … )>>` nesting must NOT recurse until the native stack overflows
        // (which would abort the whole process); it must return a normal parse `Err`.
        let line = nested_triple_term_line(10_000);
        let mut d = Dict::new();
        let e = parse_chunk(&line, &mut d).expect_err("deeply-nested triple terms must be rejected, not overflow the stack");
        assert!(
            e.contains("nested more than") && e.contains("levels deep"),
            "must be the depth-limit error, got: {e}"
        );
        // Sanitized per the #241 posture: the error reports the limit and a byte offset, never
        // echoes the (attacker-controlled) offending input.
        assert!(!e.contains("http://ex/leaf"), "error must not echo the offending input");
    }

    #[test]
    fn nested_triple_term_at_limit_parses() {
        // One level below the cap must still parse cleanly — the bound is high enough that no
        // real RDF 1.2 data is rejected. `MAX_TRIPLE_TERM_DEPTH` is the number of `<<(` levels,
        // so a line nested exactly `MAX_TRIPLE_TERM_DEPTH` deep is accepted.
        let line = nested_triple_term_line(MAX_TRIPLE_TERM_DEPTH);
        let mut d = Dict::new();
        assert!(
            parse_chunk(&line, &mut d).is_ok(),
            "nesting up to the cap ({MAX_TRIPLE_TERM_DEPTH}) must parse"
        );

        // One level beyond the cap is the first rejected depth.
        let over = nested_triple_term_line(MAX_TRIPLE_TERM_DEPTH + 1);
        let mut d = Dict::new();
        assert!(
            parse_chunk(&over, &mut d).is_err(),
            "nesting one beyond the cap must be rejected"
        );
    }

    // [OPUS-4.8] (sq-53s1) The N-Quads fast path uses a separate no-intern span walk
    // (`span_object`/`span_triple_term`) to locate the graph field; it must be bounded too.
    #[cfg(feature = "parallel")]
    #[test]
    fn deeply_nested_triple_term_in_nquads_span_returns_clean_error() {
        let mut line = b"<http://ex/s> <http://ex/p> ".to_vec();
        for _ in 0..10_000 {
            line.extend_from_slice(b"<<( <http://ex/a> <http://ex/b> ");
        }
        line.extend_from_slice(b"<http://ex/leaf>");
        for _ in 0..10_000 {
            line.extend_from_slice(b" )>>");
        }
        // N-Quads: explicit graph field after the object.
        line.extend_from_slice(b" <http://ex/g> .\n");
        let e = match parse_quads_chunk(&line) {
            Err(e) => e,
            Ok(_) => panic!("deeply-nested triple terms in N-Quads must be rejected, not overflow"),
        };
        assert!(
            e.contains("nested more than") && e.contains("levels deep"),
            "must be the depth-limit error, got: {e}"
        );
    }

    // [OPUS-4.8] (sq-cpm) The N-Quads 4th field is an OPTIONAL `graphLabel`, and the RDF 1.2
    // N-Quads grammar restricts it to `IRIREF | BLANK_NODE_LABEL` (prod. `graphLabel`) — a
    // literal or a triple term in graph position is a syntax error, not a fourth quad component.
    // `parse_quads_chunk` resolves the graph bucket via `graph_key` (IRI/blank only) and
    // `span_term` (object-only triple-term rule), so these cases must be rejected; an absent 4th
    // field routes to the default (`None`) graph.
    // `parse_quads_chunk` returns `Dict` (no `Debug`/`PartialEq`), so these helpers pull just the
    // graph-key column and the per-graph triple counts out of the bucket vec for assertions.
    #[cfg(feature = "parallel")]
    fn quad_graph_keys(line: &[u8]) -> Result<Vec<(Option<GraphKey>, usize)>, String> {
        parse_quads_chunk(line).map(|buckets| {
            buckets.into_iter().map(|(g, _dict, triples)| (g, triples.len())).collect()
        })
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn nquads_graph_label_accepts_iri_and_blank_only() {
        // IRI graph label: routes the triple into a named-graph bucket keyed by that IRI.
        let iri_g = b"<http://ex/s> <http://ex/p> <http://ex/o> <http://ex/g> .\n";
        assert_eq!(
            quad_graph_keys(iri_g).expect("IRI graph label must parse"),
            vec![(Some(GraphKey::Iri("http://ex/g".to_owned())), 1)],
            "the triple lands in the IRI-keyed graph"
        );

        // Blank-node graph label.
        let blank_g = b"<http://ex/s> <http://ex/p> <http://ex/o> _:g0 .\n";
        assert_eq!(
            quad_graph_keys(blank_g).expect("blank-node graph label must parse"),
            vec![(Some(GraphKey::Blank("g0".to_owned())), 1)],
            "blank-node graph label key"
        );

        // No 4th field: the triple is in the default graph (`None`).
        let default_g = b"<http://ex/s> <http://ex/p> <http://ex/o> .\n";
        assert_eq!(
            quad_graph_keys(default_g).expect("triple with no graph label must parse"),
            vec![(None, 1)],
            "absent graph label is the default graph"
        );
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn nquads_graph_label_rejects_literal() {
        // A literal in the 4th position is not a valid `graphLabel`.
        let lit_g = b"<http://ex/s> <http://ex/p> <http://ex/o> \"g\" .\n";
        let e = quad_graph_keys(lit_g).expect_err("a literal graph label must be rejected");
        assert!(
            e.contains("graph name must be an IRI or blank node"),
            "message must explain the IRI/blank-only rule, got: {e}"
        );

        // A typed literal likewise.
        let typed_g = b"<http://ex/s> <http://ex/p> <http://ex/o> \"5\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n";
        assert!(
            quad_graph_keys(typed_g).is_err(),
            "a typed-literal graph label must be rejected"
        );
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn nquads_graph_label_rejects_triple_term() {
        // A triple term `<<( … )>>` is OBJECT-ONLY; it is not a valid `graphLabel`. The span walk
        // (`span_term`) reaches the `<<(` opener in graph position and rejects it with the
        // position-aware message.
        let tt_g = b"<http://ex/s> <http://ex/p> <http://ex/o> <<( <http://ex/a> <http://ex/b> <http://ex/c> )>> .\n";
        let e = quad_graph_keys(tt_g).expect_err("a triple-term graph label must be rejected");
        assert!(
            e.contains("only valid in OBJECT position"),
            "message must explain the object-only rule, got: {e}"
        );
    }

    /// [OPUS-4.8] (sq-7d3dj.2) The `Dict` capacity is a PURE hint: parsing the same bytes into a
    /// `Dict::new()` vs a `Dict::with_capacity(BIG)` must yield byte-identical triples AND an
    /// identical term↔id bijection (same distinct-term count, same `term(id)` for every id) —
    /// the reserved capacity changes the allocation, never the parsed result or id-assignment
    /// order. (The out-`Vec` reservation is a fixed `bytes.len() / AVG_NT_LINE_BYTES` and is not
    /// varied here; its own purity — being derived solely from the input length — is argued at
    /// `AVG_NT_LINE_BYTES`.)
    #[test]
    fn parse_chunk_dict_capacity_is_pure_hint() {
        // Repeated predicate + type IRIs, distinct subjects/objects, a typed literal, a lang
        // literal, a blank node, and a nested triple term — a mix that grows both the term arena
        // and the lookup table across many interns.
        let mut nt = String::new();
        for i in 0..200 {
            nt.push_str(&format!("<http://ex/n{}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://ex/Person> .\n", i));
            nt.push_str(&format!("<http://ex/n{}> <http://ex/name> \"name{}\"@en .\n", i, i));
            nt.push_str(&format!("<http://ex/n{}> <http://ex/age> \"{}\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n", i, i % 37));
            nt.push_str(&format!("_:b{} <http://ex/reifies> <<( <http://ex/n{}> <http://ex/knows> <http://ex/n{}> )>> .\n", i, i, (i + 1) % 200));
        }
        let bytes = nt.as_bytes();

        let mut d_plain = Dict::new();
        let t_plain = parse_chunk(bytes, &mut d_plain).expect("plain parse");

        // A deliberately OVER-sized reservation — the hint must still not perturb the result.
        let mut d_big = Dict::with_capacity(10_000);
        let t_big = parse_chunk(bytes, &mut d_big).expect("presized parse");

        assert_eq!(t_plain, t_big, "id-triples (and their order) diverge under a capacity hint");
        assert_eq!(d_plain.len(), d_big.len(), "distinct-term count diverges under a capacity hint");
        for id in 1..=d_plain.len() as Id {
            assert_eq!(
                d_plain.term(id),
                d_big.term(id),
                "term for id {} diverges under a capacity hint (the term↔id bijection is not identical)",
                id
            );
        }
    }
}

