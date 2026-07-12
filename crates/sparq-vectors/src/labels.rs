//! Label-only embedding — the back-compat special case of the verbalization layer in
//! [`crate::verbalize()`].
//!
//! [`embed_labels`] embeds **one human-readable label per entity** (by predicate
//! priority) and nothing else. That is enough for lexical lookup, but production KG
//! systems embed a richer *passage* per entity (label + type + description + selected
//! literals — see `research/genai-text-embedding-practices.md`); use
//! [`embed_entities`] with an
//! [`EntityTextConfig`] for that. Both helpers share the same
//! engine: predicate index-block scans for candidates (no full-graph scan), batched
//! embedding, vectors keyed by dictionary id.

use crate::embed::Embedder;
use crate::store::VectorStore;
use crate::verbalize::{embed_entities, label_predicates, EntityTextConfig};
use oxrdf::NamedNode;
use sparq_core::Graph;

/// Which predicates carry an entity's label, in **priority order** (an entity's label
/// is taken from the first listed predicate it has). Defaults: `rdfs:label`,
/// `skos:prefLabel`, `foaf:name`, `schema:name`, `dc:title` (see
/// [`label_predicates`]).
#[derive(Clone, Debug)]
pub struct LabelConfig {
    pub predicates: Vec<NamedNode>,
    /// Texts per [`Embedder::embed`] call.
    pub batch: usize,
}

impl Default for LabelConfig {
    fn default() -> Self {
        LabelConfig {
            predicates: label_predicates(),
            batch: 256,
        }
    }
}

/// Embeds one label per labeled entity in `graph` into `store` (build phase) with the
/// default [`LabelConfig`]. Returns the number of entities embedded. The store is left
/// **unfinalized** so callers can add more vectors; call [`VectorStore::finalize`] when
/// done.
///
/// A thin wrapper over [`embed_entities`] with a label-only
/// [`EntityTextConfig`] ([`EntityTextConfig::labels_only`]): no language preference
/// (first label in scan order wins), no description/type enrichment. Prefer
/// `embed_entities` when the graph has descriptions or other text worth embedding —
/// label-only vectors can't tell two "John Smith"s apart.
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
    let cfg = EntityTextConfig::labels_only(cfg.predicates.clone(), cfg.batch);
    embed_entities(graph, store, embedder, &cfg)
}
