//! Reasoning-input assembly: reflect the named-graph structure into facts and serialize
//! the ACCESS-CONTROL graphs (only!) as N3 source for `sparq_reason::reason_n3`.
//!
//! Security boundary (design doc §2.4): pod *content* graphs are never fed to the
//! reasoner — otherwise any agent who can write a document could embed `acl:`/`acp:`
//! triples granting themselves access. The inputs are: `.acl`/`.acr` graphs, group
//! documents referenced via `acl:agentGroup` (fragment stripped), and the synthesized
//! structural facts below.

use crate::{AUTH_GRAPH, SOLIDX_NS};
use oxrdf::Term;
use rustc_hash::FxHashSet;
use sparq_core::Graph;
use std::fmt::Write;

pub(crate) const ACL_SUFFIX: &str = ".acl";
pub(crate) const ACR_SUFFIX: &str = ".acr";

const ACL_AGENT: &str = "http://www.w3.org/ns/auth/acl#agent";
const ACL_AGENT_GROUP: &str = "http://www.w3.org/ns/auth/acl#agentGroup";
const ACP_AGENT: &str = "http://www.w3.org/ns/solid/acp#agent";
const VCARD_MEMBER: &str = "http://www.w3.org/2006/vcard/ns#hasMember";
/// `acp:agent` objects that are NOT concrete WebIDs.
const SPECIAL_AGENTS: [&str; 5] = [
    "http://www.w3.org/ns/solid/acp#PublicAgent",
    "http://www.w3.org/ns/solid/acp#AuthenticatedAgent",
    "http://www.w3.org/ns/solid/acp#CreatorAgent",
    "http://www.w3.org/ns/solid/acp#OwnerAgent",
    "http://www.w3.org/ns/auth/acl#AuthenticatedAgent",
];

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum System {
    Wac,
    Acp,
}

/// The triples of one named graph as ground term triples.
pub(crate) fn graph_triples(g: &Graph) -> Vec<[Term; 3]> {
    let pat: sparq_core::store::Pattern = [None, None, None];
    let scan = g.store.scan(&pat);
    scan.rows
        .iter()
        .map(|r| {
            let t = scan.to_spo(r);
            [g.dict.term(t[0]), g.dict.term(t[1]), g.dict.term(t[2])]
        })
        .collect()
}

fn graph_iri(name: &Term) -> Option<&str> {
    match name {
        Term::NamedNode(n) => Some(n.as_str()),
        _ => None,
    }
}

/// One term in N3 source form; blank nodes are skolemized per graph (`gix`) so the
/// single-graph merge keeps per-document scoping and `solidx:inDoc` stays sound.
fn write_term(out: &mut String, t: &Term, gix: usize) {
    match t {
        Term::BlankNode(b) => {
            let _ = write!(out, "<urn:skolem:g{gix}:{}>", b.as_str());
        }
        // NamedNode -> `<iri>`, Literal -> `"…"`/`"…"^^<dt>`/`"…"@lang` (N-Triples
        // shapes, all accepted by the N3 parser).
        other => {
            let _ = write!(out, "{other}");
        }
    }
}

/// Skolemized subject IRI of a triple (used for `solidx:inDoc` provenance).
fn subject_repr(t: &Term, gix: usize) -> String {
    let mut s = String::new();
    write_term(&mut s, t, gix);
    s
}

/// Assemble the full facts source: structural facts + the access-control graphs.
pub(crate) fn assemble_input(graph: &Graph, system: System) -> String {
    let mut out = String::new();
    let suffix = if system == System::Wac { ACL_SUFFIX } else { ACR_SUFFIX };
    let own_pred = if system == System::Wac { "ownAcl" } else { "ownAcr" };

    // 1) resources: every non-control, non-auth graph + every structural container
    //    prefix (containers exist as inheritance anchors even without their own graph).
    let mut resources: FxHashSet<String> = FxHashSet::default();
    let mut control_graphs: Vec<(usize, &str, &Graph)> = Vec::new(); // (idx, name, sub)
    for (gix, (name, sub)) in graph.named.iter().enumerate() {
        let Some(iri) = graph_iri(name) else { continue };
        if iri == AUTH_GRAPH {
            continue;
        }
        if iri.ends_with(ACL_SUFFIX) || iri.ends_with(ACR_SUFFIX) {
            if iri.ends_with(suffix) {
                control_graphs.push((gix, iri, sub));
            }
            continue;
        }
        resources.insert(iri.to_owned());
        // structural container chain: https://host/a/b/doc -> /a/b/ -> /a/ -> /
        let mut cur = iri;
        while let Some(parent) = parent_iri(cur) {
            if !resources.insert(parent.to_owned()) {
                break;
            }
            cur = parent;
        }
    }
    for r in &resources {
        let _ = writeln!(out, "<{r}> <{SOLIDX_NS}isResource> true .");
    }

    // 2) control-document linkage by naming convention: <R + ".acl"> controls <R>.
    //    The .acl/.acr graphs are themselves resources too (Control gates them).
    for (_, iri, _) in &control_graphs {
        let r = &iri[..iri.len() - suffix.len()];
        let _ = writeln!(out, "<{r}> <{SOLIDX_NS}{own_pred}> <{iri}> .");
    }

    // 3) the access-control graphs' triples (skolemized) + inDoc provenance + WebIDs,
    //    and (WAC) group documents referenced via acl:agentGroup.
    let mut group_docs: FxHashSet<String> = FxHashSet::default();
    let mut webids: FxHashSet<String> = FxHashSet::default();
    for (gix, iri, sub) in &control_graphs {
        let mut in_doc: FxHashSet<String> = FxHashSet::default();
        for t in graph_triples(sub) {
            write_term(&mut out, &t[0], *gix);
            out.push(' ');
            write_term(&mut out, &t[1], *gix);
            out.push(' ');
            write_term(&mut out, &t[2], *gix);
            out.push_str(" .\n");
            in_doc.insert(subject_repr(&t[0], *gix));
            collect_agents(&t, &mut webids, &mut group_docs);
        }
        for s in in_doc {
            let _ = writeln!(out, "{s} <{SOLIDX_NS}inDoc> <{iri}> .");
        }
    }
    if system == System::Wac {
        for (gix, (name, sub)) in graph.named.iter().enumerate() {
            let Some(iri) = graph_iri(name) else { continue };
            if !group_docs.contains(iri) {
                continue;
            }
            for t in graph_triples(sub) {
                write_term(&mut out, &t[0], gix);
                out.push(' ');
                write_term(&mut out, &t[1], gix);
                out.push(' ');
                write_term(&mut out, &t[2], gix);
                out.push_str(" .\n");
                if let (Term::NamedNode(p), Term::NamedNode(o)) = (&t[1], &t[2]) {
                    if p.as_str() == VCARD_MEMBER {
                        webids.insert(o.as_str().to_owned());
                    }
                }
            }
        }
    }
    for a in &webids {
        let _ = writeln!(out, "<{a}> <{SOLIDX_NS}isWebId> true .");
    }
    out
}

/// Concrete agents + group documents mentioned by an access-control triple.
fn collect_agents(t: &[Term; 3], webids: &mut FxHashSet<String>, groups: &mut FxHashSet<String>) {
    let (Term::NamedNode(p), Term::NamedNode(o)) = (&t[1], &t[2]) else { return };
    match p.as_str() {
        ACL_AGENT | ACP_AGENT | VCARD_MEMBER => {
            if !SPECIAL_AGENTS.contains(&o.as_str()) {
                webids.insert(o.as_str().to_owned());
            }
        }
        ACL_AGENT_GROUP => {
            // the group document = the group IRI without its fragment
            let doc = o.as_str().split('#').next().unwrap_or(o.as_str());
            groups.insert(doc.to_owned());
        }
        _ => {}
    }
}

/// Solid slash-semantics parent of an IRI (None at/above the authority root).
pub(crate) fn parent_iri(iri: &str) -> Option<&str> {
    let scheme_end = iri.find("://").map(|i| i + 3)?;
    let path = &iri[scheme_end..];
    let host_end = scheme_end + path.find('/')?;
    let trimmed = iri.strip_suffix('/').unwrap_or(iri);
    if trimmed.len() <= host_end {
        return None; // already the root container
    }
    let cut = trimmed.rfind('/')?;
    if cut < host_end {
        return None;
    }
    Some(&iri[..cut + 1])
}

#[cfg(test)]
mod tests {
    use super::parent_iri;

    #[test]
    fn parent_walks_to_root_and_stops() {
        assert_eq!(parent_iri("https://pod.ex/a/b/doc.ttl"), Some("https://pod.ex/a/b/"));
        assert_eq!(parent_iri("https://pod.ex/a/b/"), Some("https://pod.ex/a/"));
        assert_eq!(parent_iri("https://pod.ex/a/"), Some("https://pod.ex/"));
        assert_eq!(parent_iri("https://pod.ex/"), None);
    }
}
