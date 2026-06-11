//! Solid Pod access control over the sparq engine — design:
//! `research/solid-access-control-design.md`.
//!
//! - Pods live in the KG as **named graph per document** (graph name = resource IRI);
//!   `.acl` (WAC) / `.acr` (ACP) documents are named graphs too — access-control rules
//!   stay stored as plain, queryable triples (the user's triples-native requirement).
//! - WAC/ACP semantics are encoded as **N3 rules** (`rules/*.n3`) run by `sparq-reason`,
//!   materializing an authorization view as triples in the named graph
//!   [`AUTH_GRAPH`] (`<urn:sparq:auth>`): `principal auth:read graphName` etc.
//! - A session ((WebID, client-id), both optional) expands to ≤6 principals; the
//!   accessible graph set per (session, mode) is cached and injected into queries as a
//!   `FROM NAMED` dataset clause + per-pattern `GRAPH` wrapping (v1 path on today's
//!   public APIs; the zero-copy engine dataset view is the specified L1 follow-up).

mod authindex;
pub mod fixture;
mod loader;
mod materialize;
mod rewrite;

pub use authindex::{AuthIndex, Mode, Session};
pub use fixture::{acp_fixture, wac_fixture};
pub use materialize::{materialize_acp, materialize_wac, MaterializeStats};
pub use rewrite::rewrite_for;

use oxrdf::NamedNode;
use rustc_hash::FxHashMap;
use sparq_core::Graph;
use sparq_engine::QueryResult;
use std::sync::Arc;

/// The reserved named graph holding the materialized authorization view.
pub const AUTH_GRAPH: &str = "urn:sparq:auth";

/// Namespace of the materialized auth-view vocabulary.
pub const AUTH_NS: &str = "https://sparq.dev/ns/auth#";
/// Namespace of derivation-internal predicates (kept out of the auth view except for
/// the matcher accept-set facts that conditional grants reference).
pub const SOLIDX_NS: &str = "https://sparq.dev/ns/solidx#";

/// A pod dataset + its materialized auth view + the per-session graph-set cache.
///
/// The cache is a *transient index* over the auth-view triples (user clarification D3):
/// it is dropped wholesale whenever the view is re-materialized (epoch bump).
pub struct PodStore {
    pub graph: Graph,
    auth: Arc<AuthIndex>,
    epoch: u64,
    cache: FxHashMap<(Option<String>, Option<String>, Mode), Arc<Vec<NamedNode>>>,
}

impl PodStore {
    /// Wrap a loaded dataset (e.g. `Graph::load_dataset(nquads, "nquads")`). No auth
    /// view yet — every session sees nothing until a `materialize_*` call (fail-closed).
    pub fn new(graph: Graph) -> PodStore {
        PodStore { graph, auth: Arc::new(AuthIndex::default()), epoch: 0, cache: FxHashMap::default() }
    }

    /// (Re-)materialize the WAC auth view from the `.acl` graphs. Call again after any
    /// ACL/group-document change (v1 maintenance = full re-run; measured in the design
    /// doc's baseline section).
    pub fn materialize_wac(&mut self) -> Result<MaterializeStats, String> {
        let stats = materialize_wac(&mut self.graph)?;
        self.reindex();
        Ok(stats)
    }

    /// (Re-)materialize the ACP auth view from the `.acr` graphs.
    pub fn materialize_acp(&mut self) -> Result<MaterializeStats, String> {
        let stats = materialize_acp(&mut self.graph)?;
        self.reindex();
        Ok(stats)
    }

    fn reindex(&mut self) {
        self.auth = Arc::new(AuthIndex::from_graph(&self.graph));
        self.epoch += 1;
        self.cache.clear(); // D3: the cache is transient; the triples are the storage
    }

    /// The sorted named-graph set this session may access in `mode`
    /// (`∪ allow(principals) ∖ ∪ deny(principals)`, conditional grants applied) —
    /// cached per (agent, client, mode) until the next re-materialization.
    pub fn accessible(&mut self, s: &Session, mode: Mode) -> Arc<Vec<NamedNode>> {
        let key = (s.agent.map(str::to_owned), s.client.map(str::to_owned), mode);
        if let Some(set) = self.cache.get(&key) {
            return Arc::clone(set);
        }
        let set = Arc::new(self.auth.accessible(s, mode));
        self.cache.insert(key, Arc::clone(&set));
        set
    }

    /// Evaluate `sparql` as `session`: rewrite to the authorized graph set
    /// ([`rewrite_for`]) and run through `sparq_engine::query`. Two sessions running the
    /// same query see different results — the end-to-end contract.
    pub fn query_as(&mut self, s: &Session, mode: Mode, sparql: &str) -> Result<QueryResult, String> {
        let allowed = self.accessible(s, mode);
        let rewritten = rewrite_for(sparql, &allowed)?;
        sparq_engine::query(&self.graph, &rewritten)
    }

    /// The current auth index (for direct inspection/tests).
    pub fn auth(&self) -> &AuthIndex {
        &self.auth
    }
}
