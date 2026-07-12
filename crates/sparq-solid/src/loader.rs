//! Reasoning-input assembly: reflect the named-graph structure into facts and serialize
//! the ACCESS-CONTROL graphs (only!) as N3 source for `sparq_reason::reason_n3`.
//!
//! Security boundary (design doc §2.4): pod *content* graphs are never fed to the
//! reasoner — otherwise any agent who can write a document could embed `acl:`/`acp:`
//! triples granting themselves access. The inputs are: `.acl`/`.acr` graphs, group
//! documents referenced via `acl:agentGroup` (fragment stripped), and the synthesized
//! structural facts below.
//!
//! [OPUS-4.8] sq-3jtd.5: the `.acl`/`.acr`/group graphs ARE emitted verbatim, so they are a
//! second smuggling surface. The reasoner's derivation-internal vocabulary — the `solidx:`
//! namespace (`solidx:creator|owner|appliesToResource|isResource|isWebId|…`) — must only be
//! produced by THIS loader (from trusted structural metadata + the caller-supplied
//! `AccessProvenance`) or derived by the rules; a forged `solidx:` fact inside a control
//! document would otherwise grant access (cross-resource privilege escalation / policy
//! redirection). So [`is_reserved_derivation_predicate`] hard-rejects any control-graph or
//! group-document triple whose predicate is in `solidx:` space — the analogue of the
//! `urn:sparq:` reserved-principal guard ([`validate_principal_iri`]).

use crate::{AccessProvenance, AUTH_GRAPH, SOLIDX_NS};
use oxrdf::Term;
use rustc_hash::FxHashSet;
use sparq_core::Graph;
use std::fmt::Write;

pub(crate) const ACL_SUFFIX: &str = ".acl";
pub(crate) const ACR_SUFFIX: &str = ".acr";

const ACL_AGENT: &str = "http://www.w3.org/ns/auth/acl#agent";
const ACL_AGENT_GROUP: &str = "http://www.w3.org/ns/auth/acl#agentGroup";
const ACL_ORIGIN: &str = "http://www.w3.org/ns/auth/acl#origin";
const ACP_AGENT: &str = "http://www.w3.org/ns/solid/acp#agent";
const ACP_CLIENT: &str = "http://www.w3.org/ns/solid/acp#client";
// [OPUS-4.8] sq-3jtd.6: acp:issuer is a pair/triple-principal ingredient too, so its
// values go through the same reserved-encoding validation as agents/clients/origins.
const ACP_ISSUER: &str = "http://www.w3.org/ns/solid/acp#issuer";
const VCARD_MEMBER: &str = "http://www.w3.org/2006/vcard/ns#hasMember";

/// Reserved IRI space: the auth view, the rewrite sentinel, minted pair/candidate/grant
/// principals. Graphs named under it are stripped at PodStore/materializer boundaries,
/// and agent/client/origin values inside it (or containing the pair-IRI delimiter) are
/// REJECTED (roborev 1723). Pair/candidate minting now percent-encodes its components
/// (`string:encodeForUri` / [`sparq_reason::n3::encode_for_uri`]), so a crafted WebID
/// like `…&client=…` can no longer collide with a minted pair at the encoding level —
/// this validation stays as defense in depth, and it remains LOAD-BEARING for raw
/// reserved-space values: a session or ACL agent equal to a full minted IRI
/// (`urn:sparq:pair?…`) would otherwise match that principal's grants directly.
pub(crate) const RESERVED_PREFIX: &str = "urn:sparq:";
const PAIR_DELIMITER: &str = "&client=";

fn validate_principal_iri(iri: &str) -> Result<(), String> {
    if iri.starts_with(RESERVED_PREFIX) || iri.contains(PAIR_DELIMITER) {
        return Err(format!(
            "agent/client/origin IRI <{iri}> is not allowed: \
             `{RESERVED_PREFIX}` and the literal `{PAIR_DELIMITER}` are reserved by the \
             pair-principal encoding"
        ));
    }
    Ok(())
}

/// [OPUS-4.8] sq-3jtd.5: Predicates in the `solidx:` namespace are the reasoner's
/// DERIVATION-INTERNAL vocabulary (`solidx:creator`, `solidx:owner`,
/// `solidx:appliesToResource`, `solidx:isResource`, `solidx:inDoc`, `solidx:isWebId`,
/// `solidx:provForResource`, …). Those facts are synthesized by THIS loader from
/// trusted inputs (structural metadata + the caller-supplied [`AccessProvenance`]) or
/// derived by the N3 rules — they must NEVER originate from access-control-document
/// content. A writer who can place a triple inside an `.acr`/`.acl` they control could
/// otherwise smuggle a forged `<r> solidx:creator <self>` (cross-resource privilege
/// escalation) or `<pol> solidx:appliesToResource <secret>` (policy redirection) that
/// the rules cannot distinguish from a loader-synthesized trusted fact.
///
/// So any control-graph (or group-document) triple whose PREDICATE is in `solidx:`
/// space is DROPPED before it reaches the reasoner — the direct analogue of the
/// `urn:sparq:` reserved-principal guard in [`validate_principal_iri`]. The trusted
/// channel for creator/owner facts is [`AccessProvenance`] and nothing else.
fn is_reserved_derivation_predicate(t: &[Term; 3]) -> bool {
    matches!(&t[1], Term::NamedNode(n) if n.as_str().starts_with(SOLIDX_NS))
}

/// Drop ALL named graphs in the reserved IRI space — including a pre-existing
/// `<urn:sparq:auth>`: a loaded dataset must not be able to smuggle in the rewrite
/// sentinel (`urn:sparq:nothing`) or a FORGED auth view; only `install_auth_view`
/// creates `<urn:sparq:auth>`, after a successful materialization (roborev 1727).
pub(crate) fn strip_reserved_graphs(graph: &mut Graph) {
    graph.named.retain(|(name, _)| match name {
        Term::NamedNode(n) => !n.as_str().starts_with(RESERVED_PREFIX),
        _ => true,
    });
}

/// Whether a session-supplied agent/client value may participate in principal
/// expansion: anything in the reserved space or containing the pair delimiter could
/// IMPERSONATE a minted pair principal — fail closed.
pub(crate) fn session_value_allowed(v: &str) -> bool {
    !v.starts_with(RESERVED_PREFIX) && !v.contains(PAIR_DELIMITER)
}
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

/// Assemble the full facts source: structural facts + the access-control graphs +
/// (ACP only) the TRUSTED per-resource creator/owner facts from `provenance`.
/// Errors if any agent/client/origin/creator/owner value collides with the reserved
/// principal encoding (see [`validate_principal_iri`]).
///
/// [OPUS-4.8] sq-3jtd.5: `provenance` is the trusted channel for `acp:CreatorAgent` /
/// `acp:OwnerAgent`. Its `<r> solidx:creator|owner <webid>` facts are synthesized HERE,
/// from the caller-supplied map ONLY — never read from the resource graphs (design doc
/// §2.4). For WAC (no creator/owner vocabulary) `provenance` is ignored.
pub(crate) fn assemble_input(
    graph: &Graph,
    system: System,
    provenance: &AccessProvenance,
) -> Result<String, String> {
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
    let mut principal_iris: FxHashSet<String> = FxHashSet::default();
    for (gix, iri, sub) in &control_graphs {
        let mut in_doc: FxHashSet<String> = FxHashSet::default();
        for t in graph_triples(sub) {
            // [OPUS-4.8] sq-3jtd.5: hard-reject any forged derivation-internal fact
            // (`solidx:creator|owner|appliesToResource|…`) smuggled into the control
            // document — only the loader/rules may produce `solidx:` facts.
            if is_reserved_derivation_predicate(&t) {
                continue;
            }
            write_term(&mut out, &t[0], *gix);
            out.push(' ');
            write_term(&mut out, &t[1], *gix);
            out.push(' ');
            write_term(&mut out, &t[2], *gix);
            out.push_str(" .\n");
            in_doc.insert(subject_repr(&t[0], *gix));
            collect_agents(&t, &mut webids, &mut group_docs, &mut principal_iris);
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
                // [OPUS-4.8] sq-3jtd.5: same derivation-internal guard for group
                // documents — a forged `solidx:` fact here would feed the reasoner too.
                if is_reserved_derivation_predicate(&t) {
                    continue;
                }
                write_term(&mut out, &t[0], gix);
                out.push(' ');
                write_term(&mut out, &t[1], gix);
                out.push(' ');
                write_term(&mut out, &t[2], gix);
                out.push_str(" .\n");
                if let (Term::NamedNode(p), Term::NamedNode(o)) = (&t[1], &t[2]) {
                    if p.as_str() == VCARD_MEMBER {
                        webids.insert(o.as_str().to_owned());
                        principal_iris.insert(o.as_str().to_owned());
                    }
                }
            }
        }
    }
    for iri in &principal_iris {
        validate_principal_iri(iri)?;
    }
    // [OPUS-4.8] sq-3jtd.5: TRUSTED creator/owner facts (ACP only). Emitted ONLY from the
    // caller-supplied provenance map — never from pod content (design doc §2.4). The
    // WebIDs go through the SAME reserved-encoding validation as agents/clients/issuers
    // (they become candidate agents in the rules) and are marked isWebId so the
    // candidate-generation lattice picks them up.
    if system == System::Acp {
        for (resource, creator, owner) in provenance.iter() {
            if let Some(c) = creator {
                validate_principal_iri(c)?;
                webids.insert(c.to_owned());
                let _ = writeln!(out, "<{resource}> <{SOLIDX_NS}creator> <{c}> .");
            }
            if let Some(o) = owner {
                validate_principal_iri(o)?;
                webids.insert(o.to_owned());
                let _ = writeln!(out, "<{resource}> <{SOLIDX_NS}owner> <{o}> .");
            }
        }
    }
    for a in &webids {
        let _ = writeln!(out, "<{a}> <{SOLIDX_NS}isWebId> true .");
    }
    Ok(out)
}

/// Concrete agents + group documents mentioned by an access-control triple; every
/// pair/triple-principal ingredient (agents, group members, origins, clients, issuers)
/// is recorded for reserved-encoding validation.
fn collect_agents(
    t: &[Term; 3],
    webids: &mut FxHashSet<String>,
    groups: &mut FxHashSet<String>,
    principal_iris: &mut FxHashSet<String>,
) {
    let (Term::NamedNode(p), Term::NamedNode(o)) = (&t[1], &t[2]) else { return };
    match p.as_str() {
        ACL_AGENT | ACP_AGENT | VCARD_MEMBER => {
            if !SPECIAL_AGENTS.contains(&o.as_str()) {
                webids.insert(o.as_str().to_owned());
            }
            principal_iris.insert(o.as_str().to_owned());
        }
        ACL_ORIGIN | ACP_CLIENT | ACP_ISSUER => {
            principal_iris.insert(o.as_str().to_owned());
        }
        ACL_AGENT_GROUP => {
            // the group document = the group IRI without its fragment
            let doc = o.as_str().split('#').next().unwrap_or(o.as_str());
            groups.insert(doc.to_owned());
        }
        _ => {}
    }
}

/// The origin (`scheme://authority`) of an IRI — the coarse pod boundary used to bucket
/// the auth index + session cache for an ACL-write invalidation ([OPUS-4.8] sq-b7k7u).
/// `https://pod.ex/a/b` → `https://pod.ex`; an authority with no path (`https://pod.ex`)
/// is its own origin. An IRI with no `://` (e.g. `urn:…`) returns the whole IRI — a
/// self-contained fallback bucket only ever invalidated by a FULL re-materialization.
///
/// Soundness of scoping on this key: `reindex_with`'s **diff-based** invalidation
/// ([SONNET-4.6] sq-b7k7u fix) diffs old vs new `AuthIndex` per-origin and invalidates
/// exactly the origins whose buckets changed — so a cross-origin dependency (WAC
/// agentGroup membership, foreign-subject grant, ACP cross-document indirection) is
/// caught automatically, without relying on any confinement argument about where grants
/// can originate.
pub(crate) fn iri_origin(iri: &str) -> &str {
    match iri.find("://") {
        Some(i) => {
            let after = i + 3;
            match iri[after..].find('/') {
                Some(j) => &iri[..after + j],
                None => iri,
            }
        }
        None => iri,
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
    use super::{iri_origin, parent_iri};

    #[test]
    fn parent_walks_to_root_and_stops() {
        assert_eq!(parent_iri("https://pod.ex/a/b/doc.ttl"), Some("https://pod.ex/a/b/"));
        assert_eq!(parent_iri("https://pod.ex/a/b/"), Some("https://pod.ex/a/"));
        assert_eq!(parent_iri("https://pod.ex/a/"), Some("https://pod.ex/"));
        assert_eq!(parent_iri("https://pod.ex/"), None);
    }

    #[test]
    fn iri_origin_is_scheme_authority() {
        // [OPUS-4.8] sq-b7k7u: origin = scheme://authority, stable across the whole subtree.
        assert_eq!(iri_origin("https://pod.ex/a/b/doc.ttl"), "https://pod.ex");
        assert_eq!(iri_origin("https://pod.ex/.acl"), "https://pod.ex");
        assert_eq!(iri_origin("https://pod.ex/"), "https://pod.ex");
        assert_eq!(iri_origin("https://pod.ex"), "https://pod.ex"); // authority, no path
        assert_eq!(iri_origin("http://host:8080/x"), "http://host:8080"); // port kept
        // Every graph a `.acl` at origin O governs shares O — the scoping soundness key.
        let acl = "https://pod.ex/notes/.acl";
        assert_eq!(iri_origin(acl), iri_origin("https://pod.ex/notes/n1"));
        assert_ne!(iri_origin(acl), iri_origin("https://other.ex/notes/n1"));
        // No scheme → the whole IRI is its own fallback bucket (no slash `.acl` governs it).
        assert_eq!(iri_origin("urn:sparq:auth"), "urn:sparq:auth");
    }
}
