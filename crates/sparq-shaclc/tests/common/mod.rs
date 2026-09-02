//! Shared test helpers: fixture loading + RDF graph isomorphism (bnode
//! relabelling allowed, RDF 1.2 triple terms handled recursively) — a port
//! of rdf-shuttle's conformance `iso.js` (backtracking with ground-signature
//! pruning; fine for the small, automorphism-free fixture graphs).

// Shared by several integration-test binaries; each uses a subset, so the
// others see the rest as dead code.
#![allow(dead_code)]

use oxrdf::{NamedOrBlankNode, Term, Triple};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

pub const BASE: &str = "urn:x-base:default";

pub fn fixture_dir(sub: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(sub)
}

/// Sorted fixture stems (`*.shaclc`) in a fixtures subdirectory.
pub fn fixture_names(sub: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(fixture_dir(sub))
        .expect("fixtures dir")
        .filter_map(|e| {
            let n = e.ok()?.file_name().into_string().ok()?;
            Some(n.strip_suffix(".shaclc")?.to_string())
        })
        .collect();
    names.sort();
    names
}

pub fn read_fixture(sub: &str, name: &str, ext: &str) -> String {
    let path = fixture_dir(sub).join(format!("{name}.{ext}"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Parses a `.ttl` oracle with oxttl (the independent reference parser).
pub fn ttl_oracle(sub: &str, name: &str) -> Vec<Triple> {
    let ttl = read_fixture(sub, name, "ttl");
    oxttl::TurtleParser::new()
        .with_base_iri(BASE)
        .expect("base")
        .for_reader(ttl.as_bytes())
        .map(|r| r.unwrap_or_else(|e| panic!("{sub}/{name}.ttl oracle parse: {e}")))
        .collect()
}

fn term_key(t: &Term, bmap: Option<&BTreeMap<String, String>>) -> String {
    match t {
        Term::NamedNode(n) => format!("<{}>", n.as_str()),
        Term::Literal(l) => format!(
            "\"{}\"@{}--{}^^{}",
            l.value(),
            l.language().unwrap_or(""),
            l.direction().map(|d| format!("{d:?}")).unwrap_or_default(),
            l.datatype().as_str()
        ),
        Term::BlankNode(b) => match bmap {
            Some(m) => match m.get(b.as_str()) {
                Some(v) => format!("_:{v}"),
                None => "_:?".to_string(),
            },
            None => "_:?".to_string(),
        },
        Term::Triple(q) => format!(
            "<<({} <{}> {})>>",
            term_key(&subj(q), bmap),
            q.predicate.as_str(),
            term_key(&q.object, bmap)
        ),
    }
}

fn subj(t: &Triple) -> Term {
    match &t.subject {
        NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n.clone()),
        NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b.clone()),
    }
}

fn triple_key(t: &Triple, bmap: Option<&BTreeMap<String, String>>) -> String {
    format!(
        "{} <{}> {}",
        term_key(&subj(t), bmap),
        t.predicate.as_str(),
        term_key(&t.object, bmap)
    )
}

fn collect_bnodes_term(t: &Term, out: &mut BTreeSet<String>) {
    match t {
        Term::BlankNode(b) => {
            out.insert(b.as_str().to_string());
        }
        Term::Triple(q) => {
            collect_bnodes_term(&subj(q), out);
            collect_bnodes_term(&q.object, out);
        }
        _ => {}
    }
}

fn bnodes_of(triples: &[Triple]) -> Vec<String> {
    let mut out = BTreeSet::new();
    for t in triples {
        collect_bnodes_term(&subj(t), &mut out);
        collect_bnodes_term(&t.object, &mut out);
    }
    out.into_iter().collect()
}

/// Ground signature of a bnode: sorted multiset of its triple shapes with
/// every bnode wildcarded.
fn signature(triples: &[Triple], b: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    for t in triples {
        let mut ids = BTreeSet::new();
        collect_bnodes_term(&subj(t), &mut ids);
        collect_bnodes_term(&t.object, &mut ids);
        if ids.contains(b) {
            parts.push(triple_key(t, None));
        }
    }
    parts.sort();
    parts.join("\n")
}

fn same_multiset(
    a: &[Triple],
    b: &[Triple],
    ma: Option<&BTreeMap<String, String>>,
    mb: Option<&BTreeMap<String, String>>,
) -> bool {
    let mut ka: Vec<String> = a.iter().map(|t| triple_key(t, ma)).collect();
    let mut kb: Vec<String> = b.iter().map(|t| triple_key(t, mb)).collect();
    ka.sort();
    kb.sort();
    ka == kb
}

/// RDF graph isomorphism (bnode bijection) between two triple lists.
pub fn is_isomorphic(a: &[Triple], b: &[Triple]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let ba = bnodes_of(a);
    let bb = bnodes_of(b);
    if ba.len() != bb.len() {
        return false;
    }
    if ba.is_empty() {
        return same_multiset(a, b, None, None);
    }
    let mut cands: Vec<(String, Vec<String>)> = Vec::new();
    for x in &ba {
        let sx = signature(a, x);
        let c: Vec<String> = bb
            .iter()
            .filter(|y| signature(b, y) == sx)
            .cloned()
            .collect();
        if c.is_empty() {
            return false;
        }
        cands.push((x.clone(), c));
    }
    cands.sort_by_key(|(_, c)| c.len());
    let id_b: BTreeMap<String, String> = bb.iter().map(|x| (x.clone(), x.clone())).collect();
    fn try_assign(
        i: usize,
        cands: &[(String, Vec<String>)],
        used: &mut BTreeSet<String>,
        mapping: &mut BTreeMap<String, String>,
        a: &[Triple],
        b: &[Triple],
        id_b: &BTreeMap<String, String>,
    ) -> bool {
        if i == cands.len() {
            return same_multiset(a, b, Some(mapping), Some(id_b));
        }
        let (x, cs) = &cands[i];
        for y in cs {
            if used.contains(y) {
                continue;
            }
            used.insert(y.clone());
            mapping.insert(x.clone(), y.clone());
            if try_assign(i + 1, cands, used, mapping, a, b, id_b) {
                return true;
            }
            used.remove(y);
            mapping.remove(x);
        }
        false
    }
    let mut used = BTreeSet::new();
    let mut mapping = BTreeMap::new();
    try_assign(0, &cands, &mut used, &mut mapping, a, b, &id_b)
}

/// Pretty triple list for assertion messages.
pub fn dump(triples: &[Triple]) -> String {
    let mut lines: Vec<String> = triples.iter().map(|t| triple_key(t, None)).collect();
    lines.sort();
    lines.join("\n")
}
