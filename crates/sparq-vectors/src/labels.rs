//! Graph integration: embed each entity's human-readable label so the graph gains
//! semantic (or, with [`HashEmbedder`](crate::HashEmbedder), lexical) vector lookup.
//!
//! [`embed_labels`] scans the configured label predicates through the store's POS/PSO
//! permutation indexes (one contiguous block per predicate — no full graph scan), picks
//! **one** label per subject by predicate priority order, embeds the labels in batches
//! and `put`s each vector under the subject's dictionary id.

use crate::embed::Embedder;
use crate::store::VectorStore;
use oxrdf::{NamedNode, Term};
use rustc_hash::FxHashSet;
use sparq_core::dict::{self, Id, TermParts};
use sparq_core::Graph;

/// Which predicates carry an entity's label, in **priority order** (an entity's label
/// is taken from the first listed predicate it has). Defaults: `rdfs:label`,
/// `skos:prefLabel`, `foaf:name`, `schema:name`, `dc:title`.
#[derive(Clone, Debug)]
pub struct LabelConfig {
    pub predicates: Vec<NamedNode>,
    /// Texts per [`Embedder::embed`] call.
    pub batch: usize,
}

impl Default for LabelConfig {
    fn default() -> Self {
        let iri = |s: &str| NamedNode::new(s).expect("static IRI");
        LabelConfig {
            predicates: vec![
                iri("http://www.w3.org/2000/01/rdf-schema#label"),
                iri("http://www.w3.org/2004/02/skos/core#prefLabel"),
                iri("http://xmlns.com/foaf/0.1/name"),
                iri("http://schema.org/name"),
                iri("http://purl.org/dc/terms/title"),
            ],
            batch: 256,
        }
    }
}

/// Embeds one label per labeled entity in `graph` into `store` (build phase) with the
/// default [`LabelConfig`]. Returns the number of entities embedded. The store is left
/// **unfinalized** so callers can add more vectors; call [`VectorStore::finalize`] when
/// done.
pub fn embed_labels(
    graph: &Graph,
    store: &mut VectorStore,
    embedder: &impl Embedder,
) -> Result<usize, String> {
    embed_labels_with(graph, store, embedder, &LabelConfig::default())
}

/// [`embed_labels`] with an explicit predicate list / batch size.
pub fn embed_labels_with(
    graph: &Graph,
    store: &mut VectorStore,
    embedder: &impl Embedder,
    cfg: &LabelConfig,
) -> Result<usize, String> {
    if embedder.dim() != store.dim() {
        return Err(format!(
            "embedder dim {} != store dim {}",
            embedder.dim(),
            store.dim()
        ));
    }
    // (entity id, label) in deterministic first-seen order; predicates earlier in the
    // config win because their whole block is scanned first.
    let mut seen: FxHashSet<Id> = FxHashSet::default();
    let mut labeled: Vec<(Id, String)> = Vec::new();
    for pred in &cfg.predicates {
        let Some(pid) = graph.id_of(&Term::NamedNode(pred.clone())) else { continue };
        let scan = graph.store.scan(&[None, Some(pid), None]);
        for row in scan.rows.iter() {
            let [s, _, o] = scan.to_spo(row);
            if dict::is_inline(s) || dict::is_inline(o) || seen.contains(&s) {
                continue;
            }
            // Only literal objects are labels; only IRI/blank subjects are entities.
            let TermParts::Lit { value, .. } = graph.dict.term_parts(o) else { continue };
            match graph.dict.term_parts(s) {
                TermParts::Iri { .. } | TermParts::Blank(_) => {}
                _ => continue,
            }
            let value = value.trim();
            if value.is_empty() {
                continue;
            }
            seen.insert(s);
            labeled.push((s, value.to_string()));
        }
    }

    for chunk in labeled.chunks(cfg.batch.max(1)) {
        let texts: Vec<&str> = chunk.iter().map(|(_, t)| t.as_str()).collect();
        let vectors = embedder.embed(&texts)?;
        if vectors.len() != texts.len() {
            return Err(format!(
                "embedder returned {} vectors for {} texts",
                vectors.len(),
                texts.len()
            ));
        }
        for ((id, _), v) in chunk.iter().zip(vectors) {
            store.put(*id, &v)?;
        }
    }
    Ok(labeled.len())
}
