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
}
