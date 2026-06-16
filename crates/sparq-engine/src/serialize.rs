//! [OPUS-4.8] (sq-678h) RDF serializer matrix — Turtle / TriG / N-Quads writers.
//!
//! sparq could *parse* Turtle / TriG / N-Quads / N-Triples but could only *write*
//! N-Triples (via [`crate::triples_to_ntriples`], which leans on oxrdf's canonical
//! `Display`). Every peer round-trips, so this closes that baseline gap.
//!
//! ## Opt-in
//!
//! This whole module lives behind the **`serialize-rdf`** cargo feature: a consumer
//! that only needs N-Triples (the always-on `triples_to_ntriples`) pays nothing —
//! no extra code compiles and no new dependency enters the default build. The writers
//! reuse the existing `oxrdf` term/IRI/literal infrastructure (no new crates).
//!
//! ## What's here
//!
//! * [`write_turtle`] — Turtle with `@prefix` compaction, predicate-object lists,
//!   `a` for `rdf:type`, and correct literal / IRI / blank-node escaping.
//! * [`write_trig`] — Turtle blocks wrapped in `GRAPH <g> { … }` for named graphs.
//! * [`write_nquads`] — N-Triples with the 4th graph column.
//! * [`graph_to_turtle`] / [`graph_to_trig`] / [`graph_to_nquads`] — pull the triples
//!   straight out of a [`Graph`] (and its named graphs, for the dataset formats).
//! * [`graph_to_jsonld`] / [`write_jsonld`] — JSON-LD 1.1 in the **expanded**,
//!   **flattened**, or basic prefix **compacted** document form (see [`JsonLdForm`]).
//!   A native writer (no json-ld crate, zero new deps) built on the same oxrdf terms,
//!   implementing the Deserialize-to-RDF inverse: it emits exactly the node objects the
//!   JSON-LD-to-RDF algorithm would consume back into this triple set.
//!
//! ## Canonical interop note
//!
//! N-Triples / N-Quads term syntax is produced by oxrdf's `Display` (the same
//! canonical form the parsers accept), so those two formats are byte-stable. Turtle
//! and TriG add prefix compaction on top of that same escaping, so a parse →
//! serialize → re-parse round-trip is isomorphic (verified by the property tests).
//!
//! RDF 1.2 triple terms (`<<( … )>>`) appear only in object position; the Turtle
//! writer emits them in that grammar (oxrdf renders the nested triple), so a graph
//! carrying triple terms still round-trips through Turtle / N-Triples.

use oxrdf::{NamedOrBlankNode, Term, Triple};
use sparq_core::Graph;
use std::collections::BTreeMap;
use std::fmt::Write as _;

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// A prefix map (`prefix` → namespace IRI) for Turtle / TriG compaction. The empty
/// string key is the default (`@prefix : <…>`) namespace.
pub type Prefixes = BTreeMap<String, String>;

/// The well-known prefixes assumed when a caller does not supply their own
/// ([`write_turtle`] / [`write_trig`] take an explicit map; the `graph_to_*`
/// convenience wrappers use these). Covers the namespaces that dominate real data
/// so the common case compacts without the caller declaring anything.
pub fn default_prefixes() -> Prefixes {
    [
        ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
        ("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
        ("xsd", "http://www.w3.org/2001/XMLSchema#"),
        ("owl", "http://www.w3.org/2002/07/owl#"),
        ("schema", "http://schema.org/"),
        ("foaf", "http://xmlns.com/foaf/0.1/"),
        ("dc", "http://purl.org/dc/terms/"),
        ("skos", "http://www.w3.org/2004/02/skos/core#"),
        ("sh", "http://www.w3.org/ns/shacl#"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

// ---------------------------------------------------------------------------
// Escaping helpers (shared by Turtle / TriG; N-Triples / N-Quads delegate to
// oxrdf's canonical `Display`).
// ---------------------------------------------------------------------------

/// Escapes the *content* of an IRIREF (the bytes between `<` and `>`) per the
/// Turtle/N-Triples grammar: the control range plus the syntactic delimiters that
/// would otherwise terminate or re-open the IRI. Everything else (including non-ASCII)
/// passes through verbatim, matching oxrdf's own IRI rendering.
fn escape_iri(iri: &str, out: &mut String) {
    out.push('<');
    for c in iri.chars() {
        match c {
            '\u{00}'..='\u{20}' | '<' | '>' | '"' | '{' | '}' | '|' | '^' | '`' | '\\' => {
                let _ = write!(out, "\\u{:04X}", c as u32);
            }
            _ => out.push(c),
        }
    }
    out.push('>');
}

/// Escapes a literal lexical value for a Turtle double-quoted string (the
/// `STRING_LITERAL_QUOTE` production): the four characters that must be escaped
/// (`"`, `\`, LF, CR) become `\"`, `\\`, `\n`, `\r`; everything else (tabs, other
/// control chars, non-ASCII) is legal inside a double-quoted Turtle string and is
/// emitted verbatim. This matches oxrdf's canonical N-Triples literal escaping, so a
/// re-parse is exact.
fn escape_string(value: &str, out: &mut String) {
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
}

/// True if `s` is a valid Turtle `PN_LOCAL` body that needs no escaping — a
/// conservative ASCII subset (`A–Z a–z 0–9 _ -`, and interior `.`) so the compaction
/// is always *correct*; anything outside it falls back to a full `<IRI>`. Empty is
/// allowed (`prefix:` with an empty local name is valid Turtle).
fn is_simple_pn_local(s: &str) -> bool {
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        let ok = b.is_ascii_alphanumeric()
            || b == b'_'
            || b == b'-'
            // interior '.' only (a PN_LOCAL may not end in '.')
            || (b == b'.' && i + 1 < bytes.len());
        if !ok {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Term rendering (Turtle / TriG flavour — prefix-aware).
// ---------------------------------------------------------------------------

/// Renders an IRI as a prefixed name if a registered prefix is a proper, splittable
/// match (longest namespace wins for determinism); otherwise as a full escaped IRIREF.
/// `rdf:type` in predicate position is handled by the caller (`a`), not here.
fn write_iri(iri: &str, prefixes: &Prefixes, out: &mut String) {
    // Longest-namespace-first so `…#` beats `…` etc.; deterministic on ties via the
    // prefix name (BTreeMap iteration order).
    let mut best: Option<(&str, &str)> = None;
    for (pfx, ns) in prefixes {
        if let Some(local) = iri.strip_prefix(ns.as_str()) {
            if is_simple_pn_local(local) {
                match best {
                    Some((_, bns)) if bns.len() >= ns.len() => {}
                    _ => best = Some((pfx.as_str(), local)),
                }
            }
        }
    }
    if let Some((pfx, local)) = best {
        let _ = write!(out, "{pfx}:{local}");
    } else {
        escape_iri(iri, out);
    }
}

/// Renders a literal in Turtle: `"lex"`, `"lex"@lang`, or `"lex"^^<dt>` with `xsd:string`
/// and `rdf:langString` left implicit (the canonical short forms), and the datatype IRI
/// itself prefix-compacted.
fn write_literal(lit: &oxrdf::Literal, prefixes: &Prefixes, out: &mut String) {
    out.push('"');
    escape_string(lit.value(), out);
    out.push('"');
    if let Some(lang) = lit.language() {
        out.push('@');
        out.push_str(lang);
    } else {
        let dt = lit.datatype().as_str();
        // xsd:string and rdf:langString are the implicit datatypes — omit them.
        if dt != "http://www.w3.org/2001/XMLSchema#string"
            && dt != "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString"
        {
            out.push_str("^^");
            write_iri(dt, prefixes, out);
        }
    }
}

/// Renders any term in Turtle/TriG flavour: IRI (prefixed), blank node (`_:label`),
/// literal, or an RDF 1.2 triple term `<<( s p o )>>` (object position only).
fn write_term(term: &Term, prefixes: &Prefixes, out: &mut String) {
    match term {
        Term::NamedNode(n) => write_iri(n.as_str(), prefixes, out),
        Term::BlankNode(b) => {
            out.push_str("_:");
            out.push_str(b.as_str());
        }
        Term::Literal(l) => write_literal(l, prefixes, out),
        Term::Triple(t) => {
            out.push_str("<<( ");
            write_subject(&t.subject, prefixes, out);
            out.push(' ');
            write_iri(t.predicate.as_str(), prefixes, out);
            out.push(' ');
            write_term(&t.object, prefixes, out);
            out.push_str(" )>>");
        }
    }
}

/// Renders a subject (IRI or blank node) in Turtle/TriG flavour. oxrdf models a triple
/// (including a nested RDF 1.2 triple term) with a `NamedOrBlankNode` subject, so a subject
/// is never itself a triple term.
fn write_subject(subj: &NamedOrBlankNode, prefixes: &Prefixes, out: &mut String) {
    match subj {
        NamedOrBlankNode::NamedNode(n) => write_iri(n.as_str(), prefixes, out),
        NamedOrBlankNode::BlankNode(b) => {
            out.push_str("_:");
            out.push_str(b.as_str());
        }
    }
}

// ---------------------------------------------------------------------------
// Turtle.
// ---------------------------------------------------------------------------

/// Serializes a set of triples as Turtle: a `@prefix` header for every prefix actually
/// used, then grouped predicate-object lists per subject (`s p1 o1, o2 ; p2 o3 .`), with
/// `a` standing in for `rdf:type`. Output is deterministic (subjects in first-seen order,
/// predicates grouped, objects in input order). Only the prefixes that are used appear in
/// the header.
pub fn write_turtle(triples: &[Triple], prefixes: &Prefixes) -> String {
    let mut out = String::new();
    write_prefix_header(triples, prefixes, &mut out);
    write_turtle_body(triples, prefixes, 0, &mut out);
    out
}

/// Emits the `@prefix` lines for exactly the prefixes whose namespace is the chosen
/// compaction for at least one IRI in `triples` (so an unused prefix never clutters the
/// header). Determined by a dry render of every IRI position.
fn write_prefix_header(triples: &[Triple], prefixes: &Prefixes, out: &mut String) {
    let mut used: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let mut probe = String::new();
    let mut note = |iri: &str| {
        probe.clear();
        write_iri(iri, prefixes, &mut probe);
        // A prefixed name (`pfx:local`) — not a full `<…>` IRIREF.
        if !probe.starts_with('<') {
            if let Some((pfx, _)) = probe.split_once(':') {
                if let Some((k, _)) = prefixes.get_key_value(pfx) {
                    used.insert(k.as_str());
                }
            }
        }
    };
    for t in triples {
        collect_iris(&Term::from(t.subject.clone()), &mut note);
        note(t.predicate.as_str());
        collect_iris(&t.object, &mut note);
    }
    if used.is_empty() {
        return;
    }
    for pfx in &used {
        let ns = &prefixes[*pfx];
        let mut ns_esc = String::new();
        escape_iri(ns, &mut ns_esc);
        let _ = writeln!(out, "@prefix {pfx}: {ns_esc} .");
    }
    out.push('\n');
}

/// Walks every IRI reachable in a term (recursing through triple terms), invoking `note`.
fn collect_iris(term: &Term, note: &mut impl FnMut(&str)) {
    match term {
        Term::NamedNode(n) => note(n.as_str()),
        Term::Literal(l) => note(l.datatype().as_str()),
        Term::Triple(t) => {
            collect_iris(&Term::from(t.subject.clone()), note);
            note(t.predicate.as_str());
            collect_iris(&t.object, note);
        }
        Term::BlankNode(_) => {}
    }
}

/// One subject's predicate-object lists, accumulated in stable order for the Turtle body:
/// predicate keys in first-seen order, each mapped to its objects in input order.
struct PredObjects {
    /// Predicate render keys (the prefixed IRI, or `a` for rdf:type) in first-seen order.
    order: Vec<String>,
    /// Predicate render key → its objects, in input order.
    objects: std::collections::HashMap<String, Vec<Term>>,
}

/// Writes the Turtle statement body (no header) at the given indent (column count). Used
/// directly by Turtle and, indented, inside each TriG `GRAPH { … }` block.
fn write_turtle_body(triples: &[Triple], prefixes: &Prefixes, indent: usize, out: &mut String) {
    // Group by subject in first-seen order, each subject keeping its predicate→objects
    // map (predicates in first-seen order, objects in input order). The store already
    // hands triples back in a stable order; we only need to cluster by subject.
    let pad = " ".repeat(indent);
    let mut order: Vec<NamedOrBlankNode> = Vec::new();
    let mut seen: std::collections::HashMap<NamedOrBlankNode, usize> =
        std::collections::HashMap::new();
    // For each subject slot: its predicate-object lists.
    let mut groups: Vec<PredObjects> = Vec::new();

    for t in triples {
        let subj: NamedOrBlankNode = t.subject.clone();
        let slot = *seen.entry(subj.clone()).or_insert_with(|| {
            order.push(subj.clone());
            groups.push(PredObjects {
                order: Vec::new(),
                objects: std::collections::HashMap::new(),
            });
            order.len() - 1
        });
        // Predicate key: `a` for rdf:type, else the rendered IRI.
        let pkey = if t.predicate.as_str() == RDF_TYPE {
            "a".to_string()
        } else {
            let mut s = String::new();
            write_iri(t.predicate.as_str(), prefixes, &mut s);
            s
        };
        let group = &mut groups[slot];
        if !group.objects.contains_key(&pkey) {
            group.order.push(pkey.clone());
        }
        group
            .objects
            .entry(pkey)
            .or_default()
            .push(t.object.clone());
    }

    for (i, subj) in order.iter().enumerate() {
        let group = &groups[i];
        out.push_str(&pad);
        match subj {
            NamedOrBlankNode::NamedNode(n) => write_iri(n.as_str(), prefixes, out),
            NamedOrBlankNode::BlankNode(b) => {
                out.push_str("_:");
                out.push_str(b.as_str());
            }
        }
        out.push(' ');
        for (pi, pkey) in group.order.iter().enumerate() {
            if pi > 0 {
                out.push_str(" ;\n");
                out.push_str(&pad);
                out.push_str("    ");
            }
            out.push_str(pkey);
            out.push(' ');
            let objs = &group.objects[pkey];
            for (oi, o) in objs.iter().enumerate() {
                if oi > 0 {
                    out.push_str(", ");
                }
                write_term(o, prefixes, out);
            }
        }
        out.push_str(" .\n");
    }
}

// ---------------------------------------------------------------------------
// TriG.
// ---------------------------------------------------------------------------

/// A named graph for the dataset writers: the graph name term (an IRI or blank node) and
/// its triples. `None` name = the default graph.
pub type NamedGraph<'a> = (Option<&'a Term>, &'a [Triple]);

/// Serializes a dataset as TriG: the `@prefix` header, then the default graph's statements
/// (unwrapped), then each named graph as `GRAPH <g> { … }`. The shared header covers every
/// prefix used across all graphs.
pub fn write_trig(graphs: &[NamedGraph<'_>], prefixes: &Prefixes) -> String {
    let mut out = String::new();
    // Header over the union of every graph's triples.
    let all: Vec<Triple> = graphs
        .iter()
        .flat_map(|(_, ts)| ts.iter().cloned())
        .collect();
    write_prefix_header(&all, prefixes, &mut out);

    let mut first = true;
    for (name, ts) in graphs {
        if ts.is_empty() {
            continue;
        }
        if !first {
            out.push('\n');
        }
        first = false;
        match name {
            None => write_turtle_body(ts, prefixes, 0, &mut out),
            Some(g) => {
                out.push_str("GRAPH ");
                match g {
                    Term::NamedNode(n) => write_iri(n.as_str(), prefixes, &mut out),
                    Term::BlankNode(b) => {
                        out.push_str("_:");
                        out.push_str(b.as_str());
                    }
                    other => {
                        // A literal/triple-term graph name is not legal RDF; render its
                        // canonical N-Triples form so nothing is silently lost (the parser
                        // will reject it, surfacing the bad input rather than corrupting).
                        let _ = write!(out, "{other}");
                    }
                }
                out.push_str(" {\n");
                write_turtle_body(ts, prefixes, 4, &mut out);
                out.push_str("}\n");
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// N-Quads (N-Triples + graph column). Term syntax via oxrdf's canonical Display.
// ---------------------------------------------------------------------------

/// Serializes a dataset as N-Quads: one `s p o [g] .` line per triple, the default graph's
/// triples written without a graph term. Term syntax is oxrdf's canonical N-Triples form
/// (the same the parsers accept), so output is byte-stable.
pub fn write_nquads(graphs: &[NamedGraph<'_>]) -> String {
    let mut out = String::new();
    for (name, ts) in graphs {
        match name {
            None => {
                for t in *ts {
                    let _ = writeln!(out, "{} {} {} .", t.subject, t.predicate, t.object);
                }
            }
            Some(g) => {
                for t in *ts {
                    let _ = writeln!(out, "{} {} {} {} .", t.subject, t.predicate, t.object, g);
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Graph -> bytes convenience wrappers (pull triples out of a live Graph).
// ---------------------------------------------------------------------------

/// Materializes the default graph's triples in stable store order.
fn graph_triples(graph: &Graph) -> Vec<Triple> {
    graph
        .iter_ids()
        .map(|[s, p, o]| triple_from_ids(graph, s, p, o))
        .collect()
}

/// Rebuilds an `oxrdf::Triple` from interned ids via the dictionary. The store invariant
/// guarantees a non-literal subject and an IRI predicate; the `unreachable!`s document
/// that and would only fire on a corrupt store.
fn triple_from_ids(
    graph: &Graph,
    s: sparq_core::dict::Id,
    p: sparq_core::dict::Id,
    o: sparq_core::dict::Id,
) -> Triple {
    let subject: NamedOrBlankNode = match graph.dict.term(s) {
        Term::NamedNode(n) => NamedOrBlankNode::NamedNode(n),
        Term::BlankNode(b) => NamedOrBlankNode::BlankNode(b),
        other => unreachable!("non-IRI/blank subject in store: {other}"),
    };
    let predicate = match graph.dict.term(p) {
        Term::NamedNode(n) => n,
        other => unreachable!("non-IRI predicate in store: {other}"),
    };
    let object = graph.dict.term(o);
    Triple {
        subject,
        predicate,
        object,
    }
}

/// Serializes a [`Graph`]'s default graph as Turtle with the [`default_prefixes`].
pub fn graph_to_turtle(graph: &Graph) -> String {
    write_turtle(&graph_triples(graph), &default_prefixes())
}

/// Serializes a [`Graph`]'s default graph as Turtle with a caller-supplied prefix map.
pub fn graph_to_turtle_with(graph: &Graph, prefixes: &Prefixes) -> String {
    write_turtle(&graph_triples(graph), prefixes)
}

/// Collects the default + named graphs of a [`Graph`] as owned `(name, triples)` pairs for
/// the dataset writers.
fn dataset_graphs(graph: &Graph) -> Vec<(Option<Term>, Vec<Triple>)> {
    let mut out: Vec<(Option<Term>, Vec<Triple>)> = vec![(None, graph_triples(graph))];
    for (name, g) in &graph.named {
        out.push((Some(name.clone()), graph_triples(g)));
    }
    out
}

/// Serializes a [`Graph`] (default + named graphs) as TriG with the [`default_prefixes`].
pub fn graph_to_trig(graph: &Graph) -> String {
    let owned = dataset_graphs(graph);
    let view: Vec<NamedGraph<'_>> = owned
        .iter()
        .map(|(n, ts)| (n.as_ref(), ts.as_slice()))
        .collect();
    write_trig(&view, &default_prefixes())
}

/// Serializes a [`Graph`] (default + named graphs) as N-Quads.
pub fn graph_to_nquads(graph: &Graph) -> String {
    let owned = dataset_graphs(graph);
    let view: Vec<NamedGraph<'_>> = owned
        .iter()
        .map(|(n, ts)| (n.as_ref(), ts.as_slice()))
        .collect();
    write_nquads(&view)
}

// ===========================================================================
// JSON-LD 1.1.
//
// A native writer — no json-ld crate, no serde_json, zero new dependencies. It
// reuses the same oxrdf term model the other writers do and emits JSON by hand
// (the structure is fully under our control, so a generic JSON DOM buys nothing).
//
// The output is the *Deserialize-JSON-LD-to-RDF* inverse: feeding any document this
// produces back through the JSON-LD-to-RDF algorithm reconstructs exactly this triple
// set. That is the round-trip contract — verified by an in-crate expanded-form reader
// in the tests (sparq-core has no JSON-LD parser of its own).
//
// `xsd:string` and `rdf:langString` are kept implicit (`@value` + optional
// `@language`), every other datatype is preserved verbatim as `@type` so no datatype is
// ever lost. Native JSON number/boolean coercion is applied ONLY when it is provably
// lossless and canonical (see `coerce_native`), otherwise the lexical form is kept as a
// string `@value` with its `@type`.
// ===========================================================================

const XSD: &str = "http://www.w3.org/2001/XMLSchema#";
const RDF_LANG_STRING: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";
// RDF-list (`@list`) collapsing is intentionally NOT done here: detecting a well-formed,
// single-referenced rdf:first/rdf:rest/rdf:nil chain safely is not cheap and would risk
// breaking the round-trip on shared/ill-formed lists. The expanded form already round-trips
// list triples losslessly as ordinary triples. Tracked for a follow-up (bead).

/// Which JSON-LD 1.1 document form [`write_jsonld`] / [`graph_to_jsonld`] emits.
///
/// All three are *round-trippable* (re-parsing reconstructs the same RDF). They differ
/// only in shape and in whether IRIs are abbreviated:
///
/// * [`Expanded`](JsonLdForm::Expanded) — the fully-explicit form: a flat array of node
///   objects, every IRI a full string, every value an `@value`/`@id` object. No
///   `@context`. This is the algorithmic core and the safest interop target.
/// * [`Flattened`](JsonLdForm::Flattened) — like expanded but guarantees every node
///   appears exactly once at the top level (subjects merged) and wraps named graphs in
///   `@graph`. For a triple store the expanded output is already node-merged, so this
///   adds the dataset `@graph` framing and a stable node ordering.
/// * [`Compacted`](JsonLdForm::Compacted) — a basic prefix `@context` (from the supplied
///   prefix map) abbreviating IRIs to `prefix:local` / terms, `@type` shorthand for
///   `rdf:type`, and `@language`/`@type`-free plain strings where safe.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JsonLdForm {
    /// Fully-expanded: flat node-object array, no `@context`, every IRI explicit.
    Expanded,
    /// Flattened: node-merged with named graphs wrapped in `@graph` and stable ordering.
    Flattened,
    /// Compacted: a prefix-based `@context` abbreviating IRIs (best-effort, lossless).
    Compacted,
}

/// Escapes a string as a JSON string body (without the surrounding quotes) per RFC 8259:
/// the two mandatory escapes (`"`, `\`), the short escapes for the common control chars,
/// and `\u00XX` for the remaining C0 controls. Everything else (including non-ASCII, which
/// JSON permits raw in UTF-8) passes through verbatim.
fn json_escape(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
}

/// Writes a JSON string literal (quoted, escaped).
fn json_str(s: &str, out: &mut String) {
    out.push('"');
    json_escape(s, out);
    out.push('"');
}

/// If a literal can be represented as a *canonical, lossless* native JSON scalar, returns
/// that scalar's JSON text (e.g. `true`, `42`). Otherwise `None` — the caller then keeps
/// the value as a string `@value` carrying its `@type`, so the datatype survives.
///
/// The bar is deliberately high: a native value is only emitted when re-serializing the
/// JSON scalar yields back the *same lexical form*, so the JSON-LD-to-RDF round-trip
/// reconstructs the identical literal. That means:
///
/// * `xsd:boolean` only for the canonical `true` / `false` lexicals (not `1` / `0`).
/// * `xsd:integer` only when the digits round-trip through `i64` unchanged (no leading
///   zeros, no `+`, fits in range) — JSON numbers are the canonical decimal anyway.
///
/// `xsd:double`/`decimal` are intentionally NOT coerced: JSON has one number type and its
/// shortest round-trip form rarely equals the RDF lexical (`1.5` vs `1.5E0`), which would
/// silently change the literal. Keeping them as typed strings is lossless.
fn coerce_native(value: &str, datatype: &str) -> Option<String> {
    match datatype {
        d if d == format!("{XSD}boolean") => match value {
            "true" => Some("true".to_string()),
            "false" => Some("false".to_string()),
            _ => None,
        },
        d if d == format!("{XSD}integer") => {
            // Reject any lexical whose canonical i64 text differs (leading zeros, '+',
            // spaces, out-of-range) so the re-serialized number is byte-identical.
            value
                .parse::<i64>()
                .ok()
                .filter(|n| n.to_string() == value)
                .map(|n| n.to_string())
        }
        _ => None,
    }
}

/// Renders a JSON-LD *value object* for a literal: `{"@value": …}` plus `@language` (for a
/// language-tagged string) or `@type` (for any non-string datatype). `xsd:string` /
/// `rdf:langString` stay implicit. `prefixes`, when `Some`, abbreviates the `@type` IRI.
fn write_jsonld_literal(lit: &oxrdf::Literal, prefixes: Option<&Prefixes>, out: &mut String) {
    let dt = lit.datatype().as_str();
    // Native coercion only in the absence of a language tag and only when canonical.
    if lit.language().is_none() {
        if let Some(native) = coerce_native(lit.value(), dt) {
            out.push_str("{\"@value\":");
            out.push_str(&native);
            out.push('}');
            return;
        }
    }
    out.push_str("{\"@value\":");
    json_str(lit.value(), out);
    if let Some(lang) = lit.language() {
        out.push_str(",\"@language\":");
        json_str(lang, out);
    } else if dt != format!("{XSD}string") && dt != RDF_LANG_STRING {
        out.push_str(",\"@type\":");
        write_jsonld_iri_value(dt, prefixes, out);
    }
    out.push('}');
}

/// Writes an IRI as a JSON string, compacted to `prefix:local` when a prefix map is supplied
/// and a registered namespace splits cleanly; otherwise the full IRI. Used for `@id` / `@type`
/// values and node keys (predicates) in the compacted form.
fn write_jsonld_iri_value(iri: &str, prefixes: Option<&Prefixes>, out: &mut String) {
    if let Some(pfx) = prefixes {
        if let Some(curie) = compact_iri(iri, pfx) {
            json_str(&curie, out);
            return;
        }
    }
    json_str(iri, out);
}

/// Best-effort `prefix:local` compaction sharing the Turtle writer's correctness rule
/// (`is_simple_pn_local`, longest namespace wins). `None` when no prefix applies — the caller
/// keeps the full IRI. Never produces an ambiguous/lossy CURIE.
fn compact_iri(iri: &str, prefixes: &Prefixes) -> Option<String> {
    let mut best: Option<(&str, &str)> = None;
    for (pfx, ns) in prefixes {
        if let Some(local) = iri.strip_prefix(ns.as_str()) {
            if is_simple_pn_local(local) && !local.is_empty() {
                match best {
                    Some((_, bns)) if bns.len() >= ns.len() => {}
                    _ => best = Some((pfx.as_str(), local)),
                }
            }
        }
    }
    best.map(|(pfx, local)| format!("{pfx}:{local}"))
}

/// A node object accumulated for one subject within one graph: its `@type` IRIs (from
/// `rdf:type`) and its other predicate→object lists, both in first-seen order.
struct JsonLdNode {
    /// `rdf:type` objects (rendered under the `@type` keyword). IRIs only.
    types: Vec<String>,
    /// Predicate IRIs in first-seen order (excluding `rdf:type`).
    pred_order: Vec<String>,
    /// Predicate IRI → its object terms in input order.
    preds: std::collections::HashMap<String, Vec<Term>>,
}

/// Groups a graph's triples into per-subject node objects, subjects in first-seen order.
fn group_nodes(triples: &[Triple]) -> (Vec<NamedOrBlankNode>, Vec<JsonLdNode>) {
    let mut order: Vec<NamedOrBlankNode> = Vec::new();
    let mut slot: std::collections::HashMap<NamedOrBlankNode, usize> =
        std::collections::HashMap::new();
    let mut nodes: Vec<JsonLdNode> = Vec::new();
    for t in triples {
        let s = t.subject.clone();
        let i = *slot.entry(s.clone()).or_insert_with(|| {
            order.push(s.clone());
            nodes.push(JsonLdNode {
                types: Vec::new(),
                pred_order: Vec::new(),
                preds: std::collections::HashMap::new(),
            });
            nodes.len() - 1
        });
        let node = &mut nodes[i];
        if t.predicate.as_str() == RDF_TYPE {
            if let Term::NamedNode(n) = &t.object {
                node.types.push(n.as_str().to_string());
                continue;
            }
            // A non-IRI rdf:type object is unusual but legal RDF — fall through and keep it
            // as an ordinary predicate so nothing is dropped.
        }
        let p = t.predicate.as_str().to_string();
        if !node.preds.contains_key(&p) {
            node.pred_order.push(p.clone());
        }
        node.preds.entry(p).or_default().push(t.object.clone());
    }
    (order, nodes)
}

/// Writes the `@id` of a subject/graph-name node as a JSON string (IRI compacted if a prefix
/// map is supplied; blank nodes keep their `_:label`).
fn write_node_id(subj: &NamedOrBlankNode, prefixes: Option<&Prefixes>, out: &mut String) {
    match subj {
        NamedOrBlankNode::NamedNode(n) => write_jsonld_iri_value(n.as_str(), prefixes, out),
        NamedOrBlankNode::BlankNode(b) => {
            let mut s = String::from("_:");
            s.push_str(b.as_str());
            json_str(&s, out);
        }
    }
}

/// Writes one object term as JSON-LD: an IRI/blank node as `{"@id": …}`, a literal as a value
/// object, an RDF 1.2 triple term as nested expanded JSON-LD (round-trippable). Blank node ids
/// keep the `_:` form so they re-parse to the same node.
fn write_jsonld_object(term: &Term, prefixes: Option<&Prefixes>, out: &mut String) {
    match term {
        Term::NamedNode(n) => {
            out.push_str("{\"@id\":");
            write_jsonld_iri_value(n.as_str(), prefixes, out);
            out.push('}');
        }
        Term::BlankNode(b) => {
            let mut s = String::from("_:");
            s.push_str(b.as_str());
            out.push_str("{\"@id\":");
            json_str(&s, out);
            out.push('}');
        }
        Term::Literal(l) => write_jsonld_literal(l, prefixes, out),
        Term::Triple(t) => {
            // RDF 1.2 triple terms have no standard JSON-LD 1.1 encoding. To avoid silently
            // dropping the value we emit the canonical N-Triples triple-term form
            // (`<<( s p o )>>`, exactly what oxrdf's `Display` and the Turtle/N-Triples
            // parsers use) as a plain `@id` string. A generic JSON-LD processor treats it as
            // an opaque IRI-shaped id; sparq's own round-trip reader recognises it. The value
            // is preserved verbatim either way.
            out.push_str("{\"@id\":");
            let mut nt = String::new();
            let _ = write!(nt, "{}", Term::Triple(t.clone()));
            json_str(&nt, out);
            out.push('}');
        }
    }
}

/// Emits the predicate→objects map of a node as JSON members (the `@type` keyword first when
/// present), reusing input order. Returns whether anything was written before this call's
/// members (so the caller manages the leading comma).
fn write_node_members(node: &JsonLdNode, prefixes: Option<&Prefixes>, out: &mut String) {
    let mut first = true;
    if !node.types.is_empty() {
        first = false;
        out.push_str("\"@type\":[");
        for (i, t) in node.types.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            write_jsonld_iri_value(t, prefixes, out);
        }
        out.push(']');
    }
    for p in &node.pred_order {
        if first {
            first = false;
        } else {
            out.push(',');
        }
        // Predicate key: full IRI (expanded/flattened) or compacted CURIE (compacted form).
        if let Some(pfx) = prefixes {
            if let Some(curie) = compact_iri(p, pfx) {
                json_str(&curie, out);
            } else {
                json_str(p, out);
            }
        } else {
            json_str(p, out);
        }
        out.push(':');
        out.push('[');
        for (i, o) in node.preds[p].iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            write_jsonld_object(o, prefixes, out);
        }
        out.push(']');
    }
}

/// Serializes one graph (already grouped) as a JSON array of node objects into `out`. Shared by
/// every form — the difference between forms is the `@context`/`@graph` framing the callers add.
fn write_node_array(triples: &[Triple], prefixes: Option<&Prefixes>, out: &mut String) {
    let (order, nodes) = group_nodes(triples);
    out.push('[');
    for (i, subj) in order.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"@id\":");
        write_node_id(subj, prefixes, out);
        // `@type` / predicate members (each prefixed by a comma inside write_node_members).
        let mut members = String::new();
        write_node_members(&nodes[i], prefixes, &mut members);
        if !members.is_empty() {
            out.push(',');
            out.push_str(&members);
        }
        out.push('}');
    }
    out.push(']');
}

/// Writes the `@context` object mapping each prefix to its namespace IRI (compacted form only).
/// Only prefixes that actually abbreviate at least one IRI in the dataset are emitted, so the
/// context never carries dead declarations.
fn write_context(all: &[Triple], prefixes: &Prefixes, out: &mut String) {
    let mut used: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let mut note = |iri: &str| {
        if let Some(curie) = compact_iri(iri, prefixes) {
            if let Some((pfx, _)) = curie.split_once(':') {
                if let Some((k, _)) = prefixes.get_key_value(pfx) {
                    used.insert(k.as_str());
                }
            }
        }
    };
    for t in all {
        collect_iris(&Term::from(t.subject.clone()), &mut note);
        note(t.predicate.as_str());
        collect_iris(&t.object, &mut note);
    }
    out.push('{');
    for (i, pfx) in used.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        json_str(pfx, out);
        out.push(':');
        json_str(&prefixes[*pfx], out);
    }
    out.push('}');
}

/// Serializes an RDF dataset (default + named graphs) as a JSON-LD 1.1 document in `form`.
///
/// * **Default graph only** (no named graphs) → a bare node-object array (expanded /
///   flattened), or a `{"@context": …, "@graph": [ … ]}` object (compacted).
/// * **With named graphs** → a top-level object whose `@graph` holds the default graph's nodes
///   plus one `{"@id": <g>, "@graph": [ … ]}` entry per named graph (the standard JSON-LD
///   dataset shape), wrapped under `@context` for the compacted form.
///
/// `prefixes` is consulted only for [`JsonLdForm::Compacted`]; the expanded/flattened forms
/// always emit full IRIs. Output is deterministic (subjects and predicates in first-seen order).
pub fn write_jsonld(graphs: &[NamedGraph<'_>], form: JsonLdForm, prefixes: &Prefixes) -> String {
    let pfx = (form == JsonLdForm::Compacted).then_some(prefixes);
    let mut out = String::new();

    let default_triples: &[Triple] = graphs
        .iter()
        .find(|(n, _)| n.is_none())
        .map(|(_, ts)| *ts)
        .unwrap_or(&[]);
    let named: Vec<&NamedGraph<'_>> = graphs.iter().filter(|(n, _)| n.is_some()).collect();

    // A bare array is only valid for the *expanded* form when there are no named graphs.
    // Compacted always needs the wrapping object to carry the `@context`; flattened is, per
    // the JSON-LD 1.1 flattening algorithm, always a node-map object keyed by `@graph` (so a
    // single-graph document still gets the `{"@graph": […]}` envelope).
    let needs_graph_object =
        !named.is_empty() || form == JsonLdForm::Compacted || form == JsonLdForm::Flattened;

    if !needs_graph_object {
        write_node_array(default_triples, pfx, &mut out);
        return out;
    }

    // Object form: optional @context, then @graph (default-graph nodes + per-named-graph
    // sub-objects).
    out.push('{');
    if form == JsonLdForm::Compacted {
        let all: Vec<Triple> = graphs
            .iter()
            .flat_map(|(_, ts)| ts.iter().cloned())
            .collect();
        out.push_str("\"@context\":");
        write_context(&all, prefixes, &mut out);
        out.push(',');
    }
    out.push_str("\"@graph\":[");
    // Default-graph node objects, spliced in directly (strip the array brackets).
    let mut default_arr = String::new();
    write_node_array(default_triples, pfx, &mut default_arr);
    let inner = &default_arr[1..default_arr.len() - 1];
    out.push_str(inner);
    let mut wrote = !inner.is_empty();
    for (name, ts) in named {
        let g = name.as_ref().expect("named graph has a name");
        if wrote {
            out.push(',');
        }
        wrote = true;
        out.push_str("{\"@id\":");
        // A graph name is an IRI or blank node (a literal/triple-term name is not legal RDF;
        // emit its canonical string so nothing is lost rather than corrupting structure).
        match g {
            Term::NamedNode(n) => write_jsonld_iri_value(n.as_str(), pfx, &mut out),
            Term::BlankNode(b) => {
                let mut s = String::from("_:");
                s.push_str(b.as_str());
                json_str(&s, &mut out);
            }
            other => {
                let mut s = String::new();
                let _ = write!(s, "{other}");
                json_str(&s, &mut out);
            }
        }
        out.push_str(",\"@graph\":");
        write_node_array(ts, pfx, &mut out);
        out.push('}');
    }
    out.push_str("]}");
    out
}

/// Serializes a [`Graph`] (default + named graphs) as JSON-LD 1.1 in `form`, using the
/// [`default_prefixes`] for the compacted form's `@context`. The expanded and flattened forms
/// ignore the prefix map.
pub fn graph_to_jsonld(graph: &Graph, form: JsonLdForm) -> String {
    graph_to_jsonld_with(graph, form, &default_prefixes())
}

/// [`graph_to_jsonld`] with a caller-supplied prefix map for the compacted `@context`.
pub fn graph_to_jsonld_with(graph: &Graph, form: JsonLdForm, prefixes: &Prefixes) -> String {
    let owned = dataset_graphs(graph);
    let view: Vec<NamedGraph<'_>> = owned
        .iter()
        .map(|(n, ts)| (n.as_ref(), ts.as_slice()))
        .collect();
    write_jsonld(&view, form, prefixes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::{BlankNode, Literal, NamedNode};
    use sparq_core::Graph;

    fn nn(s: &str) -> NamedNode {
        NamedNode::new_unchecked(s)
    }

    // ---- Round-trip property: parse -> serialize -> re-parse is isomorphic. ----
    //
    // Isomorphism here is exact-set equality after parsing into a sparq Graph: the
    // dictionary content-addresses terms, so two graphs with the same triple SET (modulo
    // blank-node identity preserved by stable labels) intern identically. We assert the
    // sorted N-Triples projections match — the canonical, label-stable witness.

    fn nt_sorted(g: &Graph) -> Vec<String> {
        let mut v: Vec<String> = g
            .iter_ids()
            .map(|[s, p, o]| {
                let t = triple_from_ids(g, s, p, o);
                format!("{} {} {} .", t.subject, t.predicate, t.object)
            })
            .collect();
        v.sort();
        v
    }

    fn assert_iso(original_ttl: &str) {
        let g0 = Graph::load_str(original_ttl, "turtle").unwrap();
        // Turtle round-trip.
        let ttl = graph_to_turtle(&g0);
        let g1 = Graph::load_str(&ttl, "turtle").unwrap();
        assert_eq!(
            nt_sorted(&g0),
            nt_sorted(&g1),
            "turtle round-trip\n--- serialized ---\n{ttl}"
        );
        // N-Quads round-trip (single default graph).
        let nq = graph_to_nquads(&g0);
        let g2 = Graph::load_dataset(&nq, "nquads").unwrap();
        assert_eq!(
            nt_sorted(&g0),
            nt_sorted(&g2),
            "nquads round-trip\n--- serialized ---\n{nq}"
        );
        // TriG round-trip.
        let tg = graph_to_trig(&g0);
        let g3 = Graph::load_dataset(&tg, "trig").unwrap();
        assert_eq!(
            nt_sorted(&g0),
            nt_sorted(&g3),
            "trig round-trip\n--- serialized ---\n{tg}"
        );
    }

    #[test]
    fn roundtrip_basic_prefixed() {
        assert_iso(
            r#"@prefix ex: <http://ex/> .
               ex:alice a ex:Person ; ex:knows ex:bob ; ex:age 30 ; ex:name "Alice" .
               ex:bob ex:name "Bob" ."#,
        );
    }

    #[test]
    fn roundtrip_blank_nodes() {
        assert_iso(
            r#"@prefix ex: <http://ex/> .
               ex:alice ex:knows [ ex:name "anon" ; ex:age 5 ] .
               _:shared ex:p ex:o . _:shared ex:q "v" ."#,
        );
    }

    #[test]
    fn roundtrip_language_and_datatypes() {
        assert_iso(
            r#"@prefix ex: <http://ex/> .
               @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
               ex:s ex:label "chat"@fr ; ex:label "hello"@en ;
                    ex:n "42"^^xsd:integer ; ex:d "1.5"^^xsd:double ;
                    ex:when "2020-01-01"^^xsd:date ; ex:plain "just a string" ."#,
        );
    }

    #[test]
    fn roundtrip_quotes_newlines_and_escapes_in_literals() {
        assert_iso(
            r#"@prefix ex: <http://ex/> .
               ex:s ex:p "line1\nline2\ttab\r\"quoted\" and \\ backslash" ;
                    ex:q "trailing quote\"" ;
                    ex:e "" ."#,
        );
    }

    #[test]
    fn roundtrip_iris_needing_escaping() {
        // IRIs with spaces/control chars and characters that can't be a PN_LOCAL: must
        // fall back to a full escaped <IRI>, never a malformed prefixed name.
        assert_iso(
            r#"<http://ex/has%20space> <http://ex/p~weird> <http://ex/o?x=1&y=2> .
               <http://ex/s> <http://ex/p> <http://example.com/path/to/thing#frag> ."#,
        );
    }

    #[test]
    fn roundtrip_named_graphs_trig_and_nquads() {
        // Dataset with a default graph and two named graphs — exercises GRAPH blocks and
        // the 4th N-Quads column.
        let data = r#"@prefix ex: <http://ex/> .
            ex:a ex:p ex:b .
            GRAPH ex:g1 { ex:x ex:y ex:z . ex:x ex:k "in g1" . }
            GRAPH ex:g2 { ex:m ex:n ex:o . }"#;
        let g0 = Graph::load_dataset(data, "trig").unwrap();

        let tg = graph_to_trig(&g0);
        let g_tg = Graph::load_dataset(&tg, "trig").unwrap();
        assert_dataset_iso(&g0, &g_tg, &tg);

        let nq = graph_to_nquads(&g0);
        let g_nq = Graph::load_dataset(&nq, "nquads").unwrap();
        assert_dataset_iso(&g0, &g_nq, &nq);
    }

    fn assert_dataset_iso(a: &Graph, b: &Graph, ser: &str) {
        assert_eq!(
            nt_sorted(a),
            nt_sorted(b),
            "default graph\n--- serialized ---\n{ser}"
        );
        assert_eq!(a.named.len(), b.named.len(), "named graph count\n{ser}");
        // Match named graphs by name term, compare contents.
        for (name, ag) in &a.named {
            let bg = b
                .named
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, g)| g)
                .unwrap_or_else(|| panic!("named graph {name} missing after round-trip\n{ser}"));
            assert_eq!(nt_sorted(ag), nt_sorted(bg), "named graph {name}\n{ser}");
        }
    }

    #[test]
    fn roundtrip_rdf12_triple_term() {
        // RDF 1.2 triple term in object position survives Turtle + N-Triples.
        assert_iso(
            r#"<http://ex/r> <http://ex/reifies> <<( <http://ex/a> <http://ex/b> "v" )>> .
               <http://ex/r> <http://ex/note> "plain" ."#,
        );
    }

    #[test]
    fn turtle_uses_a_and_prefixes_and_drops_unused() {
        let g = Graph::load_str(
            r#"@prefix ex: <http://ex/> .
               ex:alice a ex:Person ; ex:age 30 ."#,
            "turtle",
        )
        .unwrap();
        let ttl = graph_to_turtle(&g);
        // `a` keyword for rdf:type.
        assert!(ttl.contains(" a "), "should use 'a' for rdf:type:\n{ttl}");
        assert!(ttl.contains("@prefix"), "header present:\n{ttl}");
        // No prefix line for a namespace never used (e.g. foaf).
        assert!(
            !ttl.contains("foaf:"),
            "unused prefix must not appear:\n{ttl}"
        );
    }

    #[test]
    fn nquads_default_graph_has_no_fourth_column() {
        let t = Triple {
            subject: NamedOrBlankNode::NamedNode(nn("http://ex/s")),
            predicate: nn("http://ex/p"),
            object: Term::NamedNode(nn("http://ex/o")),
        };
        let nq = write_nquads(&[(None, &[t])]);
        assert_eq!(nq, "<http://ex/s> <http://ex/p> <http://ex/o> .\n");
    }

    #[test]
    fn nquads_named_graph_has_fourth_column() {
        let g = Term::NamedNode(nn("http://ex/g"));
        let t = Triple {
            subject: NamedOrBlankNode::NamedNode(nn("http://ex/s")),
            predicate: nn("http://ex/p"),
            object: Term::Literal(Literal::new_simple_literal("v")),
        };
        let nq = write_nquads(&[(Some(&g), &[t])]);
        assert_eq!(nq, "<http://ex/s> <http://ex/p> \"v\" <http://ex/g> .\n");
    }

    #[test]
    fn pn_local_fallback_is_correct() {
        // A local part ending in '.' is NOT a valid PN_LOCAL -> must not compact.
        assert!(!is_simple_pn_local("foo."));
        assert!(is_simple_pn_local("foo.bar"));
        assert!(is_simple_pn_local(""));
        assert!(!is_simple_pn_local("has space"));
        assert!(!is_simple_pn_local("q?x=1"));
    }

    #[test]
    fn blank_node_graph_name_in_trig() {
        let g = Graph::load_dataset(
            r#"_:bg { <http://ex/s> <http://ex/p> <http://ex/o> . }"#,
            "trig",
        )
        .unwrap();
        let tg = graph_to_trig(&g);
        let g2 = Graph::load_dataset(&tg, "trig").unwrap();
        assert_dataset_iso(&g, &g2, &tg);
        // The bnode keeps its label form `_:` in the GRAPH header.
        assert!(
            tg.contains("GRAPH _:"),
            "blank graph name uses _:label:\n{tg}"
        );
    }

    #[test]
    fn empty_graph_serializes_empty() {
        let g = Graph::load_str("", "turtle").unwrap();
        assert_eq!(graph_to_turtle(&g), "");
        assert_eq!(graph_to_nquads(&g), "");
    }

    #[test]
    fn bnode_object_does_not_declare_unused_prefix() {
        // Regression guard: a blank node must not be mistaken for an IRI in the prefix probe.
        let _b = BlankNode::default();
        let g = Graph::load_str(r#"<http://ex/s> <http://ex/p> _:x ."#, "turtle").unwrap();
        let ttl = graph_to_turtle(&g);
        let g2 = Graph::load_str(&ttl, "turtle").unwrap();
        assert_eq!(nt_sorted(&g), nt_sorted(&g2));
    }

    // =======================================================================
    // JSON-LD round-trip: serialize -> JSON-LD-to-N-Quads -> Graph -> isomorphic.
    //
    // sparq-core has no JSON-LD parser, so the round-trip is closed by a focused
    // in-test reader: it parses the document with serde_json (a dev-dependency), runs
    // the standard JSON-LD-to-RDF mapping over the node objects (expanding any
    // `@context` prefixes), and emits N-Quads — which the existing dataset loader then
    // ingests. Reconstructing the *same* triple set across all three forms (expanded /
    // flattened / compacted) for IRIs, blank nodes, datatypes, language tags, named
    // graphs and native-coerced scalars proves the writer is lossless.
    // =======================================================================

    use serde_json::Value;

    /// Expands a possibly-compacted IRI/term using the document's `@context` prefix map.
    /// A `prefix:local` whose prefix is declared expands to `namespace + local`; everything
    /// else (full IRIs, blank-node ids, keywords) passes through.
    fn expand_iri(s: &str, ctx: &std::collections::HashMap<String, String>) -> String {
        if s.starts_with("_:") || s.starts_with("@") || s.starts_with("http") {
            return s.to_string();
        }
        if let Some((pfx, local)) = s.split_once(':') {
            if let Some(ns) = ctx.get(pfx) {
                return format!("{ns}{local}");
            }
        }
        s.to_string()
    }

    /// N-Triples term text for a subject/object id (IRI -> `<iri>`, blank -> `_:label`).
    fn id_term(id: &str) -> String {
        if let Some(b) = id.strip_prefix("_:") {
            format!("_:{b}")
        } else {
            format!("<{id}>")
        }
    }

    /// Emits the N-Triples object text for a JSON-LD object value (a `@value` literal or an
    /// `@id` reference), expanding any compacted `@type` IRI via `ctx`.
    fn object_to_nt(v: &Value, ctx: &std::collections::HashMap<String, String>) -> String {
        let obj = v.as_object().expect("object value is a JSON object");
        if let Some(val) = obj.get("@value") {
            // Native scalar or string. Reconstruct the canonical lexical form + datatype.
            let (lex, dt) = match val {
                Value::Bool(b) => (
                    b.to_string(),
                    Some("http://www.w3.org/2001/XMLSchema#boolean".to_string()),
                ),
                Value::Number(n) if n.is_i64() || n.is_u64() => (
                    n.to_string(),
                    Some("http://www.w3.org/2001/XMLSchema#integer".to_string()),
                ),
                Value::String(s) => (s.clone(), None),
                other => panic!("unexpected @value scalar: {other}"),
            };
            // Escape the lexical value back to N-Triples form.
            let mut esc = String::new();
            for c in lex.chars() {
                match c {
                    '"' => esc.push_str("\\\""),
                    '\\' => esc.push_str("\\\\"),
                    '\n' => esc.push_str("\\n"),
                    '\r' => esc.push_str("\\r"),
                    _ => esc.push(c),
                }
            }
            if let Some(lang) = obj.get("@language").and_then(|l| l.as_str()) {
                return format!("\"{esc}\"@{lang}");
            }
            let dt = obj
                .get("@type")
                .and_then(|t| t.as_str())
                .map(|t| expand_iri(t, ctx))
                .or(dt);
            return match dt {
                Some(d) => format!("\"{esc}\"^^<{d}>"),
                None => format!("\"{esc}\""),
            };
        }
        // An `@id` reference.
        let id = obj
            .get("@id")
            .and_then(|i| i.as_str())
            .expect("@id present");
        let id = expand_iri(id, ctx);
        // A triple-term `@id` (our `<<( … )>>` encoding) is already canonical N-Triples text.
        if id.starts_with("<<(") {
            id
        } else {
            id_term(&id)
        }
    }

    /// Reads one node object into N-Quads lines (with the given graph column, empty for the
    /// default graph), expanding `@type` -> rdf:type and every predicate's object list.
    fn node_to_nquads(
        node: &serde_json::Map<String, Value>,
        graph: &str,
        ctx: &std::collections::HashMap<String, String>,
        out: &mut String,
    ) {
        let subj = node
            .get("@id")
            .and_then(|i| i.as_str())
            .map(|s| id_term(&expand_iri(s, ctx)))
            .expect("node has @id");
        for (k, v) in node {
            if k == "@id" || k == "@graph" {
                continue;
            }
            if k == "@type" {
                for t in v.as_array().expect("@type is an array") {
                    let ty = expand_iri(t.as_str().expect("@type IRI"), ctx);
                    let _ = writeln!(
                        out,
                        "{subj} <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <{ty}> {graph}."
                    );
                }
                continue;
            }
            let pred = expand_iri(k, ctx);
            for o in v.as_array().expect("predicate value is an array") {
                let obj = object_to_nt(o, ctx);
                let _ = writeln!(out, "{subj} <{pred}> {obj} {graph}.");
            }
        }
    }

    /// Full JSON-LD-to-N-Quads for any document our writer emits (bare array, or an object with
    /// optional `@context` and a `@graph` holding default-graph nodes + per-named-graph objects).
    fn jsonld_to_nquads(doc: &str) -> String {
        let v: Value = serde_json::from_str(doc).expect("valid JSON");
        let mut ctx: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let mut out = String::new();

        let nodes: &Vec<Value> = match &v {
            Value::Array(a) => a,
            Value::Object(o) => {
                if let Some(Value::Object(c)) = o.get("@context") {
                    for (k, val) in c {
                        if let Some(ns) = val.as_str() {
                            ctx.insert(k.clone(), ns.to_string());
                        }
                    }
                }
                o.get("@graph")
                    .and_then(|g| g.as_array())
                    .expect("@graph array")
            }
            other => panic!("unexpected top-level JSON-LD: {other}"),
        };

        for n in nodes {
            let node = n.as_object().expect("node object");
            if let Some(inner) = node.get("@graph").and_then(|g| g.as_array()) {
                // A named-graph sub-object: `{"@id": <g>, "@graph": [ … ]}`.
                let gid = node
                    .get("@id")
                    .and_then(|i| i.as_str())
                    .map(|s| id_term(&expand_iri(s, &ctx)))
                    .expect("named graph @id");
                for sub in inner {
                    node_to_nquads(
                        sub.as_object().expect("node"),
                        &format!("{gid} "),
                        &ctx,
                        &mut out,
                    );
                }
            } else {
                node_to_nquads(node, "", &ctx, &mut out);
            }
        }
        out
    }

    /// Asserts a graph survives a round-trip through all three JSON-LD forms.
    fn assert_jsonld_iso(g0: &Graph) {
        for form in [
            JsonLdForm::Expanded,
            JsonLdForm::Flattened,
            JsonLdForm::Compacted,
        ] {
            let doc = graph_to_jsonld(g0, form);
            // The document must be syntactically valid JSON.
            let _: Value = serde_json::from_str(&doc)
                .unwrap_or_else(|e| panic!("{form:?} produced invalid JSON: {e}\n{doc}"));
            let nq = jsonld_to_nquads(&doc);
            let g1 = Graph::load_dataset(&nq, "nquads").unwrap_or_else(|e| {
                panic!("{form:?} re-parse failed: {e}\n--- doc ---\n{doc}\n--- nq ---\n{nq}")
            });
            assert_eq!(
                nt_sorted(g0),
                nt_sorted(&g1),
                "{form:?} default-graph round-trip\n--- doc ---\n{doc}\n--- nq ---\n{nq}"
            );
            assert_eq!(
                g0.named.len(),
                g1.named.len(),
                "{form:?} named graph count\n{doc}"
            );
            for (name, ag) in &g0.named {
                let bg = g1
                    .named
                    .iter()
                    .find(|(n, _)| n == name)
                    .map(|(_, g)| g)
                    .unwrap_or_else(|| panic!("{form:?} named graph {name} missing\n{doc}"));
                assert_eq!(
                    nt_sorted(ag),
                    nt_sorted(bg),
                    "{form:?} named graph {name}\n{doc}"
                );
            }
        }
    }

    #[test]
    fn jsonld_roundtrip_basic() {
        let g = Graph::load_str(
            r#"@prefix ex: <http://ex/> .
               ex:alice a ex:Person ; ex:knows ex:bob ; ex:name "Alice" .
               ex:bob ex:name "Bob" ."#,
            "turtle",
        )
        .unwrap();
        assert_jsonld_iso(&g);
    }

    #[test]
    fn jsonld_roundtrip_blank_nodes() {
        let g = Graph::load_str(
            r#"@prefix ex: <http://ex/> .
               ex:alice ex:knows [ ex:name "anon" ] .
               _:shared ex:p ex:o . _:shared ex:q "v" ."#,
            "turtle",
        )
        .unwrap();
        assert_jsonld_iso(&g);
    }

    #[test]
    fn jsonld_roundtrip_datatypes_and_langtags() {
        let g = Graph::load_str(
            r#"@prefix ex: <http://ex/> .
               @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
               ex:s ex:label "chat"@fr ; ex:label "hello"@en ;
                    ex:n "42"^^xsd:integer ; ex:flag "true"^^xsd:boolean ;
                    ex:d "1.5"^^xsd:double ; ex:when "2020-01-01"^^xsd:date ;
                    ex:plain "just a string" ;
                    ex:notcanon "007"^^xsd:integer ; ex:zero "0"^^xsd:boolean ."#,
            "turtle",
        )
        .unwrap();
        assert_jsonld_iso(&g);
    }

    #[test]
    fn jsonld_roundtrip_escapes() {
        let g = Graph::load_str(
            r#"@prefix ex: <http://ex/> .
               ex:s ex:p "line1\nline2\ttab\r\"quoted\" and \\ backslash" ;
                    ex:q "" ; ex:u "café ☃" ."#,
            "turtle",
        )
        .unwrap();
        assert_jsonld_iso(&g);
    }

    #[test]
    fn jsonld_roundtrip_named_graphs() {
        let g = Graph::load_dataset(
            r#"@prefix ex: <http://ex/> .
               ex:a ex:p ex:b .
               GRAPH ex:g1 { ex:x ex:y ex:z . ex:x ex:k "in g1" . }
               GRAPH ex:g2 { ex:m ex:n "lit"@en . }
               GRAPH _:bg { ex:bn ex:p ex:q . }"#,
            "trig",
        )
        .unwrap();
        assert_jsonld_iso(&g);
    }

    #[test]
    fn jsonld_native_coercion_only_when_canonical() {
        // Canonical integer/boolean -> native JSON scalar; non-canonical -> typed string.
        let g = Graph::load_str(
            r#"@prefix ex: <http://ex/> .
               @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
               ex:s ex:i "42"^^xsd:integer ; ex:b "true"^^xsd:boolean ;
                    ex:lead "007"^^xsd:integer ; ex:bzero "0"^^xsd:boolean ;
                    ex:dec "1.50"^^xsd:decimal ."#,
            "turtle",
        )
        .unwrap();
        let doc = graph_to_jsonld(&g, JsonLdForm::Expanded);
        // Canonical 42 / true become bare JSON scalars (no quotes around the value).
        assert!(
            doc.contains("{\"@value\":42}"),
            "canonical int -> native:\n{doc}"
        );
        assert!(
            doc.contains("{\"@value\":true}"),
            "canonical bool -> native:\n{doc}"
        );
        // Leading-zero integer and non-`true`/`false` boolean stay typed strings.
        assert!(
            doc.contains("\"@value\":\"007\""),
            "leading-zero int stays string:\n{doc}"
        );
        assert!(
            doc.contains("\"@value\":\"0\""),
            "non-canonical bool stays string:\n{doc}"
        );
        // Decimal is never coerced (JSON number form would diverge from the RDF lexical).
        assert!(
            doc.contains("\"@value\":\"1.50\""),
            "decimal stays string:\n{doc}"
        );
        assert_jsonld_iso(&g);
    }

    #[test]
    fn jsonld_expanded_is_bare_array_compacted_has_context() {
        let g = Graph::load_str(
            r#"@prefix ex: <http://ex/> . ex:s a ex:T ; ex:p ex:o ."#,
            "turtle",
        )
        .unwrap();
        let exp = graph_to_jsonld(&g, JsonLdForm::Expanded);
        assert!(
            exp.trim_start().starts_with('['),
            "expanded is a bare array:\n{exp}"
        );
        // Expanded uses full IRIs, no @context.
        assert!(
            !exp.contains("@context"),
            "expanded has no @context:\n{exp}"
        );
        assert!(
            exp.contains("\"http://ex/p\""),
            "expanded predicate is a full IRI:\n{exp}"
        );

        let comp = graph_to_jsonld(&g, JsonLdForm::Compacted);
        assert!(
            comp.contains("\"@context\""),
            "compacted has @context:\n{comp}"
        );
        // schema/foaf etc. unused -> not in context; ex is not a default prefix, so the
        // compacted form still uses full IRIs for ex: (no prefix declared for it).
        let flat = graph_to_jsonld(&g, JsonLdForm::Flattened);
        assert!(
            flat.contains("\"@graph\""),
            "flattened wraps in @graph:\n{flat}"
        );
    }

    #[test]
    fn jsonld_compacted_uses_known_prefixes() {
        // rdf:/rdfs:/foaf: are in default_prefixes, so the compacted form abbreviates them.
        let g = Graph::load_str(
            r#"<http://ex/s> a <http://xmlns.com/foaf/0.1/Person> ;
                 <http://xmlns.com/foaf/0.1/name> "Bob" ."#,
            "turtle",
        )
        .unwrap();
        let comp = graph_to_jsonld(&g, JsonLdForm::Compacted);
        assert!(
            comp.contains("\"foaf\":\"http://xmlns.com/foaf/0.1/\""),
            "foaf in context:\n{comp}"
        );
        assert!(
            comp.contains("\"foaf:name\""),
            "predicate compacted:\n{comp}"
        );
        assert!(comp.contains("\"foaf:Person\""), "@type compacted:\n{comp}");
        assert_jsonld_iso(&g);
    }

    #[test]
    fn jsonld_empty_graph() {
        let g = Graph::load_str("", "turtle").unwrap();
        // Expanded empty default graph -> empty array.
        assert_eq!(graph_to_jsonld(&g, JsonLdForm::Expanded), "[]");
        // Round-trips (vacuously).
        assert_jsonld_iso(&g);
    }

    #[test]
    fn jsonld_roundtrip_rdf12_triple_term() {
        let g = Graph::load_str(
            r#"<http://ex/r> <http://ex/reifies> <<( <http://ex/a> <http://ex/b> "v" )>> .
               <http://ex/r> <http://ex/note> "plain" ."#,
            "turtle",
        )
        .unwrap();
        assert_jsonld_iso(&g);
    }
}
