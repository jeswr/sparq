//! `<urn:sparq:zk>` registry plumbing (plan §2.1, Q13).
//!
//! One reserved synthesized graph `<urn:sparq:zk>` (vocabulary
//! `zk:` = `https://sparq.dev/ns/zk#`, mirroring `auth:`) holds the registry
//! the prover needs, one entry per credential document:
//!
//! ```turtle
//! <https://dmv.example/vc/lic-123>
//!     zk:commitment      "0x1a2b…"^^zk:field ;
//!     zk:scheme          zk:poseidon2-rdfc10-v1 ;
//!     zk:cryptosuite     zk:poseidon2-schnorr-v1 ;
//!     zk:issuerKey       <did:example:dmv#key-1> ;
//!     zk:signatureGraph  <https://dmv.example/vc/lic-123?proof> ;
//!     zk:statusList      <https://dmv.example/status/3#94> ;
//!     zk:rdfc10Salt      "0x9f3c…"^^zk:field ;
//!     zk:status          zk:active ;
//!     zk:ingested        "2026-06-12T00:00:00Z"^^xsd:dateTime .
//! ```
//!
//! Conventions mirrored from `sparq-solid`'s `<urn:sparq:auth>`
//! (`install_auth_view`): the registry graph lives in `Graph::named` under
//! the reserved name, is rebuilt wholesale by its installer (never merged
//! with pre-existing content — pre-existing copies are an attack, plan
//! §2.1's loader-stripping rule), and reading is fail-closed (malformed
//! entries are dropped, an absent graph yields no entries). The proof graph
//! convention `<D + "?proof">` (Q13) is provided by [`proof_graph_name`].

use crate::field::{field_from_hex_str, field_to_hex, Fr};
use oxrdf::{NamedNode, Term};
use sparq_core::dict::Dict;
use sparq_core::Graph;

/// The reserved registry graph name.
pub const ZK_GRAPH: &str = "urn:sparq:zk";
/// The reserved IRI space (same hardening domain as `sparq-solid`).
pub const RESERVED_PREFIX: &str = "urn:sparq:";
/// The zk vocabulary namespace.
pub const ZK_NS: &str = "https://sparq.dev/ns/zk#";

pub const ZK_COMMITMENT: &str = "https://sparq.dev/ns/zk#commitment";
pub const ZK_SCHEME: &str = "https://sparq.dev/ns/zk#scheme";
pub const ZK_CRYPTOSUITE: &str = "https://sparq.dev/ns/zk#cryptosuite";
pub const ZK_ISSUER_KEY: &str = "https://sparq.dev/ns/zk#issuerKey";
pub const ZK_SIGNATURE_GRAPH: &str = "https://sparq.dev/ns/zk#signatureGraph";
pub const ZK_STATUS_LIST: &str = "https://sparq.dev/ns/zk#statusList";
pub const ZK_RDFC10_SALT: &str = "https://sparq.dev/ns/zk#rdfc10Salt";
pub const ZK_STATUS: &str = "https://sparq.dev/ns/zk#status";
pub const ZK_INGESTED: &str = "https://sparq.dev/ns/zk#ingested";
/// Datatype of commitment/salt literals.
pub const ZK_FIELD: &str = "https://sparq.dev/ns/zk#field";
/// The v1 commitment scheme id (this crate's pipeline).
pub const ZK_SCHEME_POSEIDON2_RDFC10_V1: &str = "https://sparq.dev/ns/zk#poseidon2-rdfc10-v1";
pub const ZK_STATUS_ACTIVE: &str = "https://sparq.dev/ns/zk#active";
pub const ZK_STATUS_REVOKED: &str = "https://sparq.dev/ns/zk#revoked";
const XSD_DATETIME: &str = "http://www.w3.org/2001/XMLSchema#dateTime";

/// Q13 convention: the proof graph of document `<D>` is `<D + "?proof">`.
pub fn proof_graph_name(document: &NamedNode) -> NamedNode {
    NamedNode::new_unchecked(format!("{}?proof", document.as_str()))
}

/// Credential status in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active,
    Revoked,
}

/// One `<urn:sparq:zk>` registry entry.
#[derive(Debug, Clone, PartialEq)]
pub struct RegistryEntry {
    /// Credential document IRI (= its content graph name).
    pub document: NamedNode,
    /// `C(G)`, the per-graph commitment.
    pub commitment: Fr,
    /// Commitment scheme id (`zk:poseidon2-rdfc10-v1` for this pipeline).
    pub scheme: NamedNode,
    /// Issuer cryptosuite id, when known.
    pub cryptosuite: Option<NamedNode>,
    /// Issuer verification-method reference (a `did:` URL).
    pub issuer_key: Option<NamedNode>,
    /// The credential's proof graph (`<D>?proof` by convention).
    pub signature_graph: Option<NamedNode>,
    /// Status-list reference (plan §2.6).
    pub status_list: Option<NamedNode>,
    /// The per-graph RDFC10 bnode salt (Q6).
    pub salt: Fr,
    pub status: Status,
    /// Ingest timestamp (`xsd:dateTime` lexical form).
    pub ingested: Option<String>,
}

impl RegistryEntry {
    /// Minimal entry for a freshly committed graph.
    pub fn new(document: NamedNode, commitment: Fr, salt: Fr) -> Self {
        RegistryEntry {
            document,
            commitment,
            scheme: NamedNode::new_unchecked(ZK_SCHEME_POSEIDON2_RDFC10_V1),
            cryptosuite: None,
            issuer_key: None,
            signature_graph: None,
            status_list: None,
            salt,
            status: Status::Active,
            ingested: None,
        }
    }

    /// Serializes the entry as triples of the registry graph.
    pub fn to_triples(&self) -> Vec<[Term; 3]> {
        let s = Term::NamedNode(self.document.clone());
        let field_dt = NamedNode::new_unchecked(ZK_FIELD);
        let pred = |p: &str| Term::NamedNode(NamedNode::new_unchecked(p));
        let mut out = vec![
            [
                s.clone(),
                pred(ZK_COMMITMENT),
                Term::Literal(oxrdf::Literal::new_typed_literal(
                    field_to_hex(&self.commitment),
                    field_dt.clone(),
                )),
            ],
            [s.clone(), pred(ZK_SCHEME), Term::NamedNode(self.scheme.clone())],
            [
                s.clone(),
                pred(ZK_RDFC10_SALT),
                Term::Literal(oxrdf::Literal::new_typed_literal(
                    field_to_hex(&self.salt),
                    field_dt,
                )),
            ],
            [
                s.clone(),
                pred(ZK_STATUS),
                Term::NamedNode(NamedNode::new_unchecked(match self.status {
                    Status::Active => ZK_STATUS_ACTIVE,
                    Status::Revoked => ZK_STATUS_REVOKED,
                })),
            ],
        ];
        if let Some(cs) = &self.cryptosuite {
            out.push([s.clone(), pred(ZK_CRYPTOSUITE), Term::NamedNode(cs.clone())]);
        }
        if let Some(k) = &self.issuer_key {
            out.push([s.clone(), pred(ZK_ISSUER_KEY), Term::NamedNode(k.clone())]);
        }
        if let Some(g) = &self.signature_graph {
            out.push([s.clone(), pred(ZK_SIGNATURE_GRAPH), Term::NamedNode(g.clone())]);
        }
        if let Some(sl) = &self.status_list {
            out.push([s.clone(), pred(ZK_STATUS_LIST), Term::NamedNode(sl.clone())]);
        }
        if let Some(ts) = &self.ingested {
            out.push([
                s.clone(),
                pred(ZK_INGESTED),
                Term::Literal(oxrdf::Literal::new_typed_literal(
                    ts.clone(),
                    NamedNode::new_unchecked(XSD_DATETIME),
                )),
            ]);
        }
        out
    }
}

/// Registry installation / parse errors.
#[derive(Debug)]
pub enum RegistryError {
    /// A document IRI may not live in the reserved `urn:sparq:` space.
    ReservedDocumentIri(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::ReservedDocumentIri(iri) => {
                write!(f, "registry entry document IRI is in the reserved space: {iri}")
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// Installs (replaces wholesale) the `<urn:sparq:zk>` registry graph on a
/// store, mirroring `sparq-solid::install_auth_view`: a fresh sub-graph
/// dictionary, `Graph::from_parts`, and replacement of any existing slot —
/// pre-existing registry content never survives (only the installer writes
/// this graph).
pub fn install_registry(graph: &mut Graph, entries: &[RegistryEntry]) -> Result<usize, RegistryError> {
    for e in entries {
        if e.document.as_str().starts_with(RESERVED_PREFIX) {
            return Err(RegistryError::ReservedDocumentIri(e.document.as_str().to_string()));
        }
    }
    let mut dict = Dict::new();
    let mut ids: Vec<[sparq_core::dict::Id; 3]> = Vec::new();
    for e in entries {
        for [s, p, o] in e.to_triples() {
            ids.push([dict.intern(&s), dict.intern(&p), dict.intern(&o)]);
        }
    }
    let n = ids.len();
    let reg = Graph::from_parts(dict, ids);
    let name = Term::NamedNode(NamedNode::new_unchecked(ZK_GRAPH));
    if let Some(slot) = graph.named.iter_mut().find(|(gname, _)| *gname == name) {
        slot.1 = reg;
    } else {
        graph.named.push((name, reg));
    }
    Ok(n)
}

/// Reads the registry back from a store. Fail-closed: an absent registry
/// graph yields no entries; entries missing a parseable commitment or salt
/// are dropped.
pub fn read_registry(graph: &Graph) -> Vec<RegistryEntry> {
    let name = Term::NamedNode(NamedNode::new_unchecked(ZK_GRAPH));
    let Some((_, reg)) = graph.named.iter().find(|(gname, _)| *gname == name) else {
        return Vec::new();
    };

    // Group predicate/object pairs by subject, preserving first-seen order.
    let mut order: Vec<NamedNode> = Vec::new();
    let mut by_subject: std::collections::HashMap<String, Vec<(String, Term)>> =
        std::collections::HashMap::new();
    for t in reg.iter_ids() {
        let s = reg.dict.term(t[0]);
        let p = reg.dict.term(t[1]);
        let o = reg.dict.term(t[2]);
        let (Term::NamedNode(s), Term::NamedNode(p)) = (s, p) else { continue };
        if !by_subject.contains_key(s.as_str()) {
            order.push(s.clone());
        }
        by_subject
            .entry(s.as_str().to_string())
            .or_default()
            .push((p.as_str().to_string(), o));
    }

    let mut out = Vec::new();
    for doc in order {
        let props = &by_subject[doc.as_str()];
        let field_of = |pred: &str| -> Option<Fr> {
            props.iter().find_map(|(p, o)| {
                if p != pred {
                    return None;
                }
                match o {
                    Term::Literal(l) if l.datatype().as_str() == ZK_FIELD => {
                        field_from_hex_str(l.value())
                    }
                    _ => None,
                }
            })
        };
        let iri_of = |pred: &str| -> Option<NamedNode> {
            props.iter().find_map(|(p, o)| {
                if p != pred {
                    return None;
                }
                match o {
                    Term::NamedNode(n) => Some(n.clone()),
                    _ => None,
                }
            })
        };
        let (Some(commitment), Some(salt)) = (field_of(ZK_COMMITMENT), field_of(ZK_RDFC10_SALT))
        else {
            continue; // fail closed on malformed entries
        };
        let Some(scheme) = iri_of(ZK_SCHEME) else { continue };
        let status = match iri_of(ZK_STATUS) {
            Some(s) if s.as_str() == ZK_STATUS_REVOKED => Status::Revoked,
            Some(s) if s.as_str() == ZK_STATUS_ACTIVE => Status::Active,
            _ => continue, // unknown status: fail closed
        };
        let ingested = props.iter().find_map(|(p, o)| match o {
            Term::Literal(l) if p == ZK_INGESTED => Some(l.value().to_string()),
            _ => None,
        });
        out.push(RegistryEntry {
            document: doc,
            commitment,
            scheme,
            cryptosuite: iri_of(ZK_CRYPTOSUITE),
            issuer_key: iri_of(ZK_ISSUER_KEY),
            signature_graph: iri_of(ZK_SIGNATURE_GRAPH),
            status_list: iri_of(ZK_STATUS_LIST),
            salt,
            status,
            ingested,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::salt_from_bytes;

    fn entry(doc: &str) -> RegistryEntry {
        let mut e = RegistryEntry::new(
            NamedNode::new(doc).unwrap(),
            Fr::from(42u64),
            salt_from_bytes(&[9u8; 32]),
        );
        e.cryptosuite = Some(NamedNode::new_unchecked("https://sparq.dev/ns/zk#poseidon2-schnorr-v1"));
        e.issuer_key = Some(NamedNode::new_unchecked("did:example:dmv#key-1"));
        e.signature_graph = Some(proof_graph_name(&e.document));
        e.status_list = Some(NamedNode::new_unchecked("https://dmv.example/status/3#94"));
        e.ingested = Some("2026-06-12T00:00:00Z".to_string());
        e
    }

    #[test]
    fn round_trip_through_store() {
        let mut g = Graph::load_str("", "turtle").unwrap();
        let entries = vec![entry("https://dmv.example/vc/lic-123"), entry("https://rail.example/vc/t-9")];
        let n = install_registry(&mut g, &entries).unwrap();
        assert!(n >= entries.len() * 4);
        let back = read_registry(&g);
        assert_eq!(back, entries);
    }

    #[test]
    fn reinstall_replaces_wholesale() {
        let mut g = Graph::load_str("", "turtle").unwrap();
        install_registry(&mut g, &[entry("https://dmv.example/vc/a")]).unwrap();
        install_registry(&mut g, &[entry("https://dmv.example/vc/b")]).unwrap();
        let back = read_registry(&g);
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].document.as_str(), "https://dmv.example/vc/b");
    }

    #[test]
    fn reserved_document_iri_rejected() {
        let mut g = Graph::load_str("", "turtle").unwrap();
        let e = RegistryEntry::new(
            NamedNode::new_unchecked("urn:sparq:zk-fake"),
            Fr::from(1u64),
            Fr::from(2u64),
        );
        assert!(matches!(
            install_registry(&mut g, &[e]),
            Err(RegistryError::ReservedDocumentIri(_))
        ));
    }

    #[test]
    fn read_is_fail_closed() {
        // Absent graph.
        let g = Graph::load_str("", "turtle").unwrap();
        assert!(read_registry(&g).is_empty());
        // Malformed entry (no commitment): dropped.
        let mut g = Graph::load_str("", "turtle").unwrap();
        let mut dict = Dict::new();
        let s = Term::NamedNode(NamedNode::new_unchecked("https://x.example/vc/1"));
        let p = Term::NamedNode(NamedNode::new_unchecked(ZK_STATUS));
        let o = Term::NamedNode(NamedNode::new_unchecked(ZK_STATUS_ACTIVE));
        let ids = vec![[dict.intern(&s), dict.intern(&p), dict.intern(&o)]];
        let reg = Graph::from_parts(dict, ids);
        g.named.push((Term::NamedNode(NamedNode::new_unchecked(ZK_GRAPH)), reg));
        assert!(read_registry(&g).is_empty());
    }

    #[test]
    fn proof_graph_convention() {
        let d = NamedNode::new("https://dmv.example/vc/lic-123").unwrap();
        assert_eq!(proof_graph_name(&d).as_str(), "https://dmv.example/vc/lic-123?proof");
    }
}
