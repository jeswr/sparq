//! Solid Pod access control over the sparq engine.
//!
//! [Solid](https://solidproject.org/) is a decentralized-web specification stack in
//! which people keep their data in personal online datastores ("pods") of RDF
//! documents and grant agents and applications selective access through one of two
//! declarative access-control languages: [Web Access Control (WAC)](https://solidproject.org/TR/wac)
//! (`.acl` documents) and [Access Control Policy (ACP)](https://solidproject.org/TR/acp)
//! (`.acr` documents). This crate stores pods inside a sparq dataset and enforces
//! those languages on SPARQL queries — with zero Solid-specific code in the engine.
//! Design + measured baseline: `research/solid-access-control-design.md`; user-facing
//! overview: this crate's `README.md`.
//!
//! # Model
//!
//! - Pods live in the KG as **named graph per document** (graph name = resource IRI);
//!   `.acl` (WAC) / `.acr` (ACP) documents are named graphs too — access-control rules
//!   stay stored as plain, queryable triples (the user's triples-native requirement,
//!   design doc D1).
//! - WAC/ACP semantics are encoded as **N3 rules** (`rules/*.n3`) run by `sparq-reason`,
//!   materializing an authorization view as triples in the named graph
//!   [`AUTH_GRAPH`] (`<urn:sparq:auth>`): `principal auth:read graphName` etc. The view
//!   is itself queryable with ordinary SPARQL.
//! - A session ((WebID, client-id), both optional) expands to ≤6 principals; the
//!   accessible graph set per (session, mode) is `∪ allow ∖ ∪ deny`, cached (a
//!   transient index, design doc D3) and enforced on queries through the engine's
//!   **zero-copy dataset view** (`sparq_engine::query_view` + per-pattern `GRAPH`
//!   wrapping — the default path, see [`PodStore::query_as`]). The v1 `FROM NAMED`
//!   injection rewrite is kept as a portability path ([`rewrite_for`] /
//!   [`PodStore::query_as_rewrite`]) for running the same policy on engines without
//!   the view API.
//!
//! # Fail-closed semantics
//!
//! Absence of a grant means a graph is **invisible** (design doc D4). Concretely:
//!
//! - before the first `materialize_*` call there is no auth view, so every session
//!   (including ones naming a pod owner's WebID) sees the empty graph set;
//! - an anonymous session only matches grants to `auth:Public` (and
//!   `pair(auth:Public, client)`);
//! - session agent/client values inside the reserved `urn:sparq:` IRI space (or
//!   containing the pair-encoding delimiter `&client=`) yield the empty set — they
//!   could otherwise impersonate a minted pair principal;
//! - graphs named under `urn:sparq:` (including a forged `<urn:sparq:auth>`) are
//!   stripped from loaded datasets at the [`PodStore::new`] / materializer boundary.
//!
//! # Quick start
//!
//! ```
//! use sparq_solid::{PodStore, Session, Mode};
//!
//! // A one-document pod: the document graph + the pod root's WAC ACL graph.
//! let nquads = r#"
//! <https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
//! <https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
//! <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
//! <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
//! <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
//! "#;
//!
//! let graph = sparq_core::Graph::load_dataset(nquads, "nquads")?;
//! let mut store = PodStore::new(graph);
//! store.materialize_wac()?; // run the N3 rules, install <urn:sparq:auth>
//!
//! let q = "SELECT ?title WHERE { ?s <https://ex.dev/ns#title> ?title }";
//! let alice = Session { agent: Some("https://alice.ex/card#me"), client: None };
//! assert_eq!(store.query_as(&alice, Mode::Read, q)?.rows.len(), 1);
//! // same query, anonymous session: fail-closed, zero rows
//! assert_eq!(store.query_as(&Session::default(), Mode::Read, q)?.rows.len(), 0);
//! # Ok::<(), String>(())
//! ```
//!
//! A full end-to-end walk-through over the bundled ~1.1k-graph pod fixture is
//! `examples/quickstart.rs` (`cargo run -p sparq-solid --example quickstart --release`).

mod authindex;
pub mod fixture;
mod loader;
mod materialize;
mod rewrite;

pub use authindex::{AuthIndex, Mode, Session};
pub use fixture::{acp_fixture, wac_fixture};
pub use materialize::{materialize_acp, materialize_wac, MaterializeStats};
pub use rewrite::{rewrite_for, wrap_for_view};

use oxrdf::{NamedNode, Term};
use rustc_hash::FxHashMap;
use sparq_core::Graph;
use sparq_engine::{DatasetView, DefaultGraphMode, FxHashSet, QueryResult};
use std::sync::Arc;

/// The reserved named graph holding the materialized authorization view.
///
/// Its triples are the storage of record for authorization decisions (design doc D1):
/// `principal auth:read|write|append|control graphName`, `auth:deny*` triples, and
/// `auth:ConditionalGrant` nodes (ACP `noneOf`). It is queryable like any graph:
///
/// ```
/// # let nquads = r#"
/// # <https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
/// # <https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
/// # <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
/// # <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
/// # <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
/// # "#;
/// # let graph = sparq_core::Graph::load_dataset(nquads, "nquads")?;
/// # let mut store = sparq_solid::PodStore::new(graph);
/// # store.materialize_wac()?;
/// let who_reads_n1 = sparq_engine::query(
///     &store.graph,
///     "SELECT ?who WHERE { GRAPH <urn:sparq:auth> {
///        ?who <https://sparq.dev/ns/auth#read> <https://pod.ex/notes/n1> } }",
/// )?;
/// assert_eq!(who_reads_n1.rows.len(), 1); // alice
/// # Ok::<(), String>(())
/// ```
///
/// The whole `urn:sparq:` IRI space is reserved: loaded datasets cannot supply graphs
/// under it (they are stripped — a dataset must not smuggle in a forged auth view),
/// and agent/client/origin IRIs inside it are rejected at materialization time.
pub const AUTH_GRAPH: &str = "urn:sparq:auth";

/// Namespace of the materialized auth-view vocabulary
/// (`auth:read`, `auth:denyWrite`, `auth:Public`, `auth:ConditionalGrant`, …).
pub const AUTH_NS: &str = "https://sparq.dev/ns/auth#";
/// Namespace of derivation-internal predicates (kept out of the auth view except for
/// the matcher accept-set facts that conditional grants reference).
pub const SOLIDX_NS: &str = "https://sparq.dev/ns/solidx#";

/// A pod dataset + its materialized auth view + the per-session graph-set cache.
///
/// The central type of this crate: wrap a loaded dataset with [`PodStore::new`],
/// build the authorization view with [`PodStore::materialize_wac`] /
/// [`PodStore::materialize_acp`], then evaluate per-session queries with
/// [`PodStore::query_as`] (or fetch the raw authorized graph set with
/// [`PodStore::accessible`]).
///
/// The cache is a *transient index* over the auth-view triples (design doc D3):
/// it is dropped wholesale whenever the view is re-materialized (epoch bump), so a
/// revoked grant takes effect for all sessions at the next query.
///
/// # Examples
///
/// ```
/// use sparq_solid::{PodStore, Session, Mode};
///
/// let nquads = r#"
/// <https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
/// <https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
/// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
/// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
/// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
/// "#;
/// let mut store = PodStore::new(sparq_core::Graph::load_dataset(nquads, "nquads")?);
/// store.materialize_wac()?;
///
/// let alice = Session { agent: Some("https://alice.ex/card#me"), client: None };
/// // alice was granted acl:Read only — Write fails closed
/// assert_eq!(store.accessible(&alice, Mode::Read).len(), 2); // n1 + the notes/ container
/// assert_eq!(store.accessible(&alice, Mode::Write).len(), 0);
/// # Ok::<(), String>(())
/// ```
pub struct PodStore {
    /// The underlying dataset: pod documents, `.acl`/`.acr` graphs, and (after a
    /// `materialize_*` call) the `<urn:sparq:auth>` view. Mutating it directly does
    /// NOT rebuild the session index — go through the `materialize_*` methods.
    pub graph: Graph,
    auth: Arc<AuthIndex>,
    epoch: u64,
    cache: FxHashMap<(Option<String>, Option<String>, Mode), SessionSets>,
}

/// One session-cache entry: the same authorized graph set in the two shapes its two
/// consumers need — sorted (the v1 `FROM NAMED` rewrite, [`PodStore::accessible`])
/// and hashed (the engine [`DatasetView`], [`PodStore::accessible_set`]). Both are
/// `Arc`-shared per call; neither is ever copied after construction.
struct SessionSets {
    sorted: Arc<Vec<NamedNode>>,
    set: Arc<FxHashSet<Term>>,
}

impl PodStore {
    /// Wrap a loaded dataset (e.g. `Graph::load_dataset(nquads, "nquads")`). No auth
    /// view yet — every session sees nothing until a `materialize_*` call (fail-closed).
    /// Named graphs under the reserved `urn:sparq:` space are dropped: a loaded dataset
    /// must not smuggle in the rewrite sentinel or a forged auth view.
    ///
    /// # Examples
    ///
    /// A dataset that arrives carrying a forged `<urn:sparq:auth>` grant is stripped
    /// at this boundary, and the un-materialized store fails closed:
    ///
    /// ```
    /// use sparq_solid::{PodStore, Session, Mode};
    ///
    /// let forged = r#"
    /// <https://pod.ex/secret#it> <https://ex.dev/ns#title> "secret" <https://pod.ex/secret> .
    /// <https://mallory.ex/card#me> <https://sparq.dev/ns/auth#read> <https://pod.ex/secret> <urn:sparq:auth> .
    /// "#;
    /// let mut store = PodStore::new(sparq_core::Graph::load_dataset(forged, "nquads")?);
    /// let mallory = Session { agent: Some("https://mallory.ex/card#me"), client: None };
    /// assert!(store.accessible(&mallory, Mode::Read).is_empty());
    /// # Ok::<(), String>(())
    /// ```
    pub fn new(mut graph: Graph) -> PodStore {
        loader::strip_reserved_graphs(&mut graph);
        PodStore { graph, auth: Arc::new(AuthIndex::default()), epoch: 0, cache: FxHashMap::default() }
    }

    /// (Re-)materialize the WAC auth view from the `.acl` graphs. Call again after any
    /// ACL/group-document change (v1 maintenance = full re-run; measured ~1 s on the
    /// ~1.1k-graph fixture in release — see the design doc's baseline section).
    ///
    /// Re-materializing bumps the epoch and drops the whole session cache, so
    /// revocations take effect immediately.
    ///
    /// # Errors
    ///
    /// Returns `Err` if any agent / group-member / origin IRI in the ACL or group
    /// documents collides with the reserved principal encoding (starts with
    /// `urn:sparq:` or contains the literal `&client=`), or if the N3 reasoner fails.
    /// On error the previous auth view (if any) is left in place.
    pub fn materialize_wac(&mut self) -> Result<MaterializeStats, String> {
        let stats = materialize_wac(&mut self.graph)?;
        self.reindex();
        Ok(stats)
    }

    /// (Re-)materialize the ACP auth view from the `.acr` graphs.
    ///
    /// Same contract as [`PodStore::materialize_wac`] (three reasoning strata instead
    /// of one; measured ~1.1 s on the fixture in release).
    ///
    /// # Errors
    ///
    /// As [`PodStore::materialize_wac`] (reserved-encoding collisions in
    /// agent/client IRIs; reasoner failure).
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
    ///
    /// Fail-closed: empty before the first `materialize_*` call, empty for sessions
    /// with no matching grants, and empty for session values inside the reserved
    /// `urn:sparq:` encoding (see [`AuthIndex::accessible`]).
    ///
    /// Measured: ~0.3 ms cold, ~0.6 µs cached on the ~1.1k-graph fixture. The same
    /// cache entry also holds this set in the hash-set shape the engine dataset view
    /// takes — [`PodStore::accessible_set`].
    pub fn accessible(&mut self, s: &Session, mode: Mode) -> Arc<Vec<NamedNode>> {
        Arc::clone(&self.session_sets(s, mode).sorted)
    }

    /// [`PodStore::accessible`] as the hash-set shape the engine [`DatasetView`]
    /// takes (`Arc<FxHashSet<Term>>`, shared per call — design doc §5.3: the engine
    /// holds no session state, visibility is one O(1) hash lookup per graph name).
    ///
    /// Same cache, same fail-closed semantics. Use it with
    /// [`sparq_engine::with_view`] when an entry point [`PodStore::query_as`] /
    /// [`PodStore::query_json_as`] / [`PodStore::ask_as`] doesn't cover (e.g.
    /// `construct`) needs to run under the session's view — or just take
    /// [`PodStore::view_for`].
    pub fn accessible_set(&mut self, s: &Session, mode: Mode) -> Arc<FxHashSet<Term>> {
        Arc::clone(&self.session_sets(s, mode).set)
    }

    fn session_sets(&mut self, s: &Session, mode: Mode) -> &SessionSets {
        let key = (s.agent.map(str::to_owned), s.client.map(str::to_owned), mode);
        let auth = Arc::clone(&self.auth);
        self.cache.entry(key).or_insert_with(|| {
            let sorted = auth.accessible(s, mode);
            let set = sorted.iter().map(|n| Term::NamedNode(n.clone())).collect();
            SessionSets { sorted: Arc::new(sorted), set: Arc::new(set) }
        })
    }

    /// The session's zero-copy [`DatasetView`] over this store (mode-checked graph
    /// set, empty default graph — pod data never lives in the default graph). This
    /// is what [`PodStore::query_as`] evaluates under; take it directly to drive any
    /// other engine entry point through [`sparq_engine::with_view`].
    ///
    /// Fail-closed like [`PodStore::accessible`]: an un-materialized store or a
    /// grant-less session yields a view over the empty graph set, under which every
    /// graph is indistinguishable from absent.
    pub fn view_for(&mut self, s: &Session, mode: Mode) -> DatasetView<'_> {
        let named = self.accessible_set(s, mode);
        DatasetView { base: &self.graph, named, default: DefaultGraphMode::Empty }
    }

    /// Evaluate `sparql` as `session`: wrap default-graph patterns to range over
    /// named graphs ([`wrap_for_view`]) and run through the engine's **zero-copy
    /// dataset view** ([`sparq_engine::query_view`]) restricted to the session's
    /// authorized graph set. Two sessions running the same query see different
    /// results — the end-to-end contract.
    ///
    /// This is the default (v2) path: graph visibility is one O(1) hash check per
    /// graph name, evaluation runs in place on the existing sub-graphs (zero
    /// decode/rebuild/copy), the view's default graph is empty (pod data never
    /// lives in the default graph), and a non-authorized graph is
    /// *indistinguishable* from an absent one — explicit `GRAPH <g>` patterns and
    /// caller-supplied `FROM (NAMED)` clauses can only restrict the view, never
    /// widen it. The v1 `FROM NAMED`-injection path is kept as
    /// [`PodStore::query_as_rewrite`] (portability: same policy on engines without
    /// a view API), at the measured cost of copying every authorized graph per
    /// query. Measured before/after: see "Measured" in this crate's README.
    ///
    /// # Errors
    ///
    /// Returns `Err` if `sparql` does not parse, or if the engine fails on the
    /// wrapped query. Authorization itself never errors: a session without grants
    /// gets a view over the empty graph set and zero rows.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sparq_solid::{PodStore, Session, Mode};
    /// # let nquads = r#"
    /// # <https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
    /// # <https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
    /// # <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
    /// # <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
    /// # <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
    /// # "#;
    /// # let mut store = PodStore::new(sparq_core::Graph::load_dataset(nquads, "nquads")?);
    /// # store.materialize_wac()?;
    /// let q = "SELECT ?title WHERE { ?s <https://ex.dev/ns#title> ?title }";
    /// let alice = Session { agent: Some("https://alice.ex/card#me"), client: None };
    /// assert_eq!(store.query_as(&alice, Mode::Read, q)?.rows.len(), 1);
    /// assert_eq!(store.query_as(&Session::default(), Mode::Read, q)?.rows.len(), 0);
    /// # Ok::<(), String>(())
    /// ```
    pub fn query_as(&mut self, s: &Session, mode: Mode, sparql: &str) -> Result<QueryResult, String> {
        let wrapped = wrap_for_view(sparql)?;
        sparq_engine::query_view(&self.view_for(s, mode), &wrapped)
    }

    /// [`PodStore::query_as`], returning the SPARQL 1.1 JSON results serialization
    /// (via [`sparq_engine::query_json_view`]) instead of a materialized
    /// [`QueryResult`]. Same view path, same fail-closed semantics.
    pub fn query_json_as(&mut self, s: &Session, mode: Mode, sparql: &str) -> Result<String, String> {
        let wrapped = wrap_for_view(sparql)?;
        sparq_engine::query_json_view(&self.view_for(s, mode), &wrapped)
    }

    /// ASK as `session` through the view path ([`sparq_engine::ask_view`]): `true`
    /// iff the pattern is satisfiable inside the session's authorized graph set.
    /// Fail-closed: a grant-less session always gets `false` (empty view).
    pub fn ask_as(&mut self, s: &Session, mode: Mode, sparql: &str) -> Result<bool, String> {
        let wrapped = wrap_for_view(sparql)?;
        sparq_engine::ask_view(&self.view_for(s, mode), &wrapped)
    }

    /// [`PodStore::query_as`] over the **v1 rewrite path**: inject a `FROM NAMED`
    /// dataset clause for the authorized graph set ([`rewrite_for`]) and run through
    /// plain [`sparq_engine::query`] — no view API needed.
    ///
    /// Kept for portability (the rewritten query enforces the same policy on any
    /// SPARQL 1.1 engine with standard dataset-clause semantics) and as the
    /// differential oracle for the view path (`tests/e2e.rs` asserts both paths
    /// return byte-identical JSON). The honest cost: the engine's `FROM NAMED`
    /// handling decodes + rebuilds every authorized graph **per query** (measured
    /// 12 ms/query at 3k quads, 59 ms at 46k — linear in authorized data), which is
    /// exactly the copy the default view path deletes.
    ///
    /// One deliberate semantic difference: a caller-supplied `FROM <g>` (default
    /// graph) clause is **dropped** here (pod data never lives in the default
    /// graph), while the view path intersects it with the authorized set like any
    /// other dataset clause. Neither path lets it widen visibility.
    ///
    /// # Errors
    ///
    /// As [`PodStore::query_as`].
    pub fn query_as_rewrite(&mut self, s: &Session, mode: Mode, sparql: &str) -> Result<QueryResult, String> {
        let allowed = self.accessible(s, mode);
        let rewritten = rewrite_for(sparql, &allowed)?;
        sparq_engine::query(&self.graph, &rewritten)
    }

    /// The current auth index (for direct inspection/tests).
    ///
    /// Rebuilt (and the session cache cleared) on every re-materialization; an
    /// un-materialized store exposes an empty index.
    pub fn auth(&self) -> &AuthIndex {
        &self.auth
    }
}
