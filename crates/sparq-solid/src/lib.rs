#![doc = include_str!("../README.md")] // [OPUS-4.8] README is the docs.rs front page
#![forbid(unsafe_code)] // [OPUS-4.8] sq-emay: crate has zero `unsafe`

mod authindex;
// [OPUS-4.8] sq-3jtd.9: library-level ACP conformance harness — a table-driven scenario
// corpus over the ACP engine (materialize_acp + AuthIndex::accessible), asserting
// decision parity against the Solid ACP spec semantics. Always compiled (no feature
// gate): it depends only on the always-present ACP path. The corpus lives in
// tests/conformance_acp.rs.
pub mod conformance;
// [OPUS-4.8] issue #992 Phase-1 (sq-snopa.1/.2/.3): the per-resource WAC DECISION layer —
// `PodStore::decide`/`decide_batch`/`resolve_acl`, the typed fail-closed `AclStatus`, and
// the `accessTo`-vs-`default` `AclScope`. Always compiled (no feature gate): it adds no
// dependency, reusing only `AuthIndex::accessible` + the loader's container-chain walk —
// so it mirrors `wac_allow`'s always-present public-API placement (#1154).
mod decide;
pub mod fixture;
mod loader;
mod materialize;
// [OPUS-4.8] sq-3jtd.5: TRUSTED per-resource creator/owner provenance — the channel by
// which the storage layer (PSS) supplies the `acp:CreatorAgent`/`acp:OwnerAgent` facts.
mod provenance;
// [OPUS-4.8] sq-h3uk: ODRL→AUTH_GRAPH bridge — opt-in (`odrl-bridge` feature), OFF by
// default, so the core sparq-solid build carries zero ODRL/sparq-policy code.
#[cfg(feature = "odrl-bridge")]
pub mod odrl_bridge;
// [OPUS-4.8] sq-pfae PoC (issue #940): trust-graph admission → AUTH_GRAPH wiring —
// opt-in (`trust-graph` feature), OFF by default, so the core sparq-solid build carries
// zero trust-graph/sparq-trust code and is byte-identical to WAC/ACP today (G6).
#[cfg(feature = "trust-graph")]
pub mod trust_wire;
mod rewrite;
mod update; // [OPUS-4.8] sq-xor3: write/update-path enforcement
// [OPUS-4.8] issue #992 FR-3 (sq-snopa.5): the AUTHORITATIVE ACL write-through —
// `PodStore::put_acl`/`delete_acl` replace (or remove) a `.acl`/`.acr` graph AND
// re-materialize `<urn:sparq:auth>` as ONE atomic, fail-closed unit, keeping SPARQ the
// source of truth. Always compiled (no feature gate): it adds no dependency, reusing only
// the always-present materialize + named-graph machinery — mirroring `update_as`/`decide`.
mod write_through;
// [OPUS-4.8] sq-3jtd.8: library-level WAC conformance harness — the table-driven
// scenario sibling of `conformance` (ACP), over the WAC engine (materialize_wac +
// AuthIndex::accessible). Shares the decision/expectation/report vocabulary with
// `conformance`; adds the WAC `.acl`-corpus builder (`AclBuilder`) and runner
// (`WacScenario`). Always compiled (no feature gate). Corpus in tests/conformance_wac.rs.
pub mod wac_conformance;

pub use authindex::{pair_principal, triple_principal, AuthIndex, Mode, Session};
// [OPUS-4.8] sq-3jtd.9: the ACP conformance harness entry types.
pub use conformance::{AcpScenario, AcrBuilder, Decision, Expect, ScenarioReport};
// [OPUS-4.8] issue #992 Phase-1 (sq-snopa.1/.2/.3): the per-resource WAC decision types.
pub use decide::{AclScope, AclStatus, EffectiveAcl, WacDecision};
// [OPUS-4.8] sq-3jtd.8: the WAC conformance harness entry types (the decision/expectation
// /report vocabulary is the shared `conformance::{Decision, Expect, ScenarioReport}`).
pub use wac_conformance::{AclBuilder, AuthBuilder, WacScenario};
pub use fixture::{acp_fixture, wac_fixture};
pub use materialize::{materialize_acp, materialize_acp_with, materialize_wac, MaterializeStats};
pub use provenance::AccessProvenance;
#[cfg(feature = "odrl-bridge")]
pub use odrl_bridge::{
    action_to_mode, materialize_permission, materialize_permission_conditional, materialize_policy,
    materialize_prohibition, materialize_prohibition_conditional, BridgeOutcome,
};
#[cfg(feature = "odrl-bridge")]
pub use odrl_bridge::{BridgeEntry, BridgeKind, BridgeLedger};
// [OPUS-4.8] sq-pfae PoC: the trust-graph admission outcome types (feature-gated).
// `TrustStaticOutcome` is the materialise-time (static/dynamic split, sq-xc4y) result.
#[cfg(feature = "trust-graph")]
pub use trust_wire::{TrustAdmissionOutcome, TrustStaticOutcome};
pub use rewrite::{rewrite_for, wrap_for_view};
// [OPUS-4.8] issue #992 FR-3 (sq-snopa.5): the authoritative ACL write-through outcome type
// (the `put_acl`/`delete_acl` methods live on `PodStore` in `write_through`).
pub use write_through::AclWriteOutcome;

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

/// The reserved named graph recording the **provenance** of bridged ODRL grants
/// ([OPUS-4.8] sq-dpk4). Every auth triple the opt-in ODRL bridge writes into
/// [`AUTH_GRAPH`] is mirrored here verbatim, so a bridged grant is structurally
/// distinguishable from a static WAC/ACP grant: a triple is *bridged* iff it appears
/// in this graph, *static* otherwise. This is what lets the refresh/retraction model
/// (a) re-evaluate and retract only the bridged grants whose ODRL policy no longer
/// holds, never touching a static grant, and (b) re-apply still-valid bridged grants
/// after a wholesale static re-materialization rebuilds [`AUTH_GRAPH`].
///
/// It lives in the reserved `urn:sparq:` space, so a loaded dataset cannot smuggle a
/// forged provenance record in (it is stripped by [`PodStore::new`] like the auth view).
pub const AUTH_BRIDGED_GRAPH: &str = "urn:sparq:auth-bridged";

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
/// let alice = Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None, now: None };
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
    // [OPUS-4.8] sq-3jtd.6: the cache key ([`SessionKey`]) spans all three session
    // dimensions — agent, client, AND issuer — so two sessions differing only by issuer
    // (e.g. the same WebID vouched for by a trusted vs an untrusted IdP) never collide on
    // a cached graph set.
    cache: FxHashMap<SessionKey, SessionSets>,
    /// [OPUS-4.8] sq-dpk4 — the bridged-ODRL-grant ledger: what was bridged from which
    /// `(policy, request)`, plus the static-baseline auth view to rebuild from on a
    /// refresh. Only present when the `odrl-bridge` feature is enabled (the core build
    /// carries zero ODRL state).
    #[cfg(feature = "odrl-bridge")]
    bridge_ledger: odrl_bridge::BridgeLedger,
}

/// The owned session-cache key: the request dimensions ([`Session`] is borrowed, so the
/// cache owns `String` copies) plus the [`Mode`]. [OPUS-4.8] sq-3jtd.6 added the issuer
/// slot; [OPUS-4.8] sq-0q7n added the clock (`now`) slot so a time-windowed conditional
/// grant re-checked at two different instants never returns a stale cached graph set
/// (two requests differing only by `now` — one inside a window, one after it — must not
/// collide).
type SessionKey = (Option<String>, Option<String>, Option<String>, Option<String>, Mode);

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
    /// let mallory = Session { agent: Some("https://mallory.ex/card#me"), client: None, issuer: None, now: None };
    /// assert!(store.accessible(&mallory, Mode::Read).is_empty());
    /// # Ok::<(), String>(())
    /// ```
    pub fn new(mut graph: Graph) -> PodStore {
        loader::strip_reserved_graphs(&mut graph);
        PodStore {
            graph,
            auth: Arc::new(AuthIndex::default()),
            epoch: 0,
            cache: FxHashMap::default(),
            #[cfg(feature = "odrl-bridge")]
            bridge_ledger: odrl_bridge::BridgeLedger::new(),
        }
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
        self.reconcile_bridged_after_static();
        self.reindex();
        Ok(stats)
    }

    /// [OPUS-4.8] sq-dpk4 — after a wholesale static (WAC/ACP) re-materialization has
    /// rebuilt `<urn:sparq:auth>` (dropping any bridged grants that were on top),
    /// capture the fresh static view as the refresh baseline and REPLAY the bridge
    /// ledger so still-valid bridged grants are re-applied (and now-invalid ones stay
    /// retracted). This reconciles the two grant sources: a static re-run can never drop
    /// a valid bridged grant, and the replay can never widen or drop a static grant
    /// (the baseline is the static view verbatim). No-op without the `odrl-bridge`
    /// feature, or when no grants were ever bridged.
    #[cfg(feature = "odrl-bridge")]
    fn reconcile_bridged_after_static(&mut self) {
        self.bridge_ledger.capture_static_baseline(&self.graph);
        // refresh() rebuilds from the just-captured baseline and replays the ledger.
        self.bridge_ledger.refresh(&mut self.graph);
    }

    /// No-op stub when the bridge is compiled out — the core build has no bridged state.
    #[cfg(not(feature = "odrl-bridge"))]
    #[inline]
    fn reconcile_bridged_after_static(&mut self) {}

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
        self.materialize_acp_with(&AccessProvenance::new())
    }

    /// (Re-)materialize the ACP auth view using TRUSTED per-resource creator/owner facts,
    /// resolving `acp:CreatorAgent` / `acp:OwnerAgent` matchers ([OPUS-4.8] sq-3jtd.5).
    ///
    /// Identical to [`PodStore::materialize_acp`], but `provenance` supplies "who
    /// created/owns `<r>`" for the resources whose ACP policies use a `CreatorAgent` /
    /// `OwnerAgent` matcher. `provenance` is the **trusted channel** for those facts: they
    /// are asserted by the caller (the storage layer that minted the resource) and are
    /// **never** read from the resource graph — a writer who embeds `<r> solidx:creator
    /// <self>` in a document they control cannot thereby grant themselves access (design
    /// doc §2.4). `materialize_acp()` is exactly `materialize_acp_with(&AccessProvenance::new())`.
    ///
    /// # Errors
    ///
    /// As [`PodStore::materialize_acp`], plus an `Err` if a creator/owner WebID collides
    /// with the reserved principal encoding (starts with `urn:sparq:` or contains the
    /// literal `&client=`).
    ///
    /// # Examples
    ///
    /// ```
    /// use sparq_solid::{AccessProvenance, Mode, PodStore, Session};
    ///
    /// // A policy on /docs/ grants Read to the resource's CREATOR.
    /// let acp = "http://www.w3.org/ns/solid/acp#";
    /// let nquads = format!(
    ///     "<https://pod.ex/docs/d0.ttl#it> <https://ex.dev/ns#k> \"v\" <https://pod.ex/docs/d0.ttl> .\n\
    ///      <https://pod.ex/docs/.acr> <{acp}memberAccessControl> <https://pod.ex/docs/.acr#c> <https://pod.ex/docs/.acr> .\n\
    ///      <https://pod.ex/docs/.acr#c> <{acp}apply> <https://pod.ex/docs/.acr#pol> <https://pod.ex/docs/.acr> .\n\
    ///      <https://pod.ex/docs/.acr#pol> <{acp}allow> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/docs/.acr> .\n\
    ///      <https://pod.ex/docs/.acr#pol> <{acp}allOf> <https://pod.ex/docs/.acr#m> <https://pod.ex/docs/.acr> .\n\
    ///      <https://pod.ex/docs/.acr#m> <{acp}agent> <{acp}CreatorAgent> <https://pod.ex/docs/.acr> .\n");
    /// let mut store = PodStore::new(sparq_core::Graph::load_dataset(&nquads, "nquads")?);
    ///
    /// // The trusted storage layer says alice created d0.
    /// let mut prov = AccessProvenance::new();
    /// prov.set_creator("https://pod.ex/docs/d0.ttl", "https://alice.ex/card#me");
    /// store.materialize_acp_with(&prov)?;
    ///
    /// let alice = Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None, now: None };
    /// let bob = Session { agent: Some("https://bob.ex/card#me"), client: None, issuer: None, now: None };
    /// assert_eq!(store.accessible(&alice, Mode::Read).len(), 1); // d0 (her creation)
    /// assert_eq!(store.accessible(&bob, Mode::Read).len(), 0);   // not the creator
    /// # Ok::<(), String>(())
    /// ```
    pub fn materialize_acp_with(
        &mut self,
        provenance: &AccessProvenance,
    ) -> Result<MaterializeStats, String> {
        let stats = materialize_acp_with(&mut self.graph, provenance)?;
        self.reconcile_bridged_after_static();
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

    /// Build the value of a [`WAC-Allow`](https://solidproject.org/TR/wac#wac-allow)
    /// response header for `resource` as seen by `session` — the RFC-style
    /// `user="…",public="…"` permission advertisement a Solid server returns on a
    /// `GET`/`HEAD`. [OPUS-4.8] sq-i7k08.
    ///
    /// `user` lists the modes the **authenticated** agent (`session`) holds on the
    /// named graph backing `resource`; `public` lists the modes an **anonymous**
    /// caller ([`Session::default`]) holds — so an unauthenticated request still learns
    /// what the public can do. Each list is the space-separated lower-case mode names
    /// (`read write append control`, in that fixed order), containing **only** the modes
    /// actually granted, and empty (`user=""`) when none are. The four
    /// [`PodStore::accessible`] sweeps share the per-session cache, so this is an O(1)
    /// hash check per mode over the materialized index.
    ///
    /// **Fail-closed**, exactly like [`PodStore::accessible`]: before the first
    /// `materialize_*` call, for a grant-less session, or for a `resource` outside the
    /// session's accessible set, the corresponding list is empty — never a wider one.
    ///
    /// **Scope (honest):** `sparq-solid` is a *library-level* authoriser with **no HTTP
    /// surface** — mapping a request **path to its named graph** (`resource`) and
    /// authenticating the WebID into a `Session` are the **server's** job (see
    /// `research/sparq-solid-scope.md` §4). This builds the header *value* from the
    /// authorization verdict; it does not parse requests, discover ACLs, or set headers.
    ///
    /// # Examples
    ///
    /// ```
    /// use sparq_solid::{Mode, PodStore, Session};
    /// use oxrdf::NamedNode;
    ///
    /// let nquads = r#"
    /// <https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
    /// <https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
    /// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
    /// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
    /// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
    /// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Write> <https://pod.ex/.acl> .
    /// <https://pod.ex/.acl#pub> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
    /// <https://pod.ex/.acl#pub> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
    /// <https://pod.ex/.acl#pub> <http://www.w3.org/ns/auth/acl#agentClass> <http://xmlns.com/foaf/0.1/Agent> <https://pod.ex/.acl> .
    /// <https://pod.ex/.acl#pub> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
    /// "#;
    /// let mut store = PodStore::new(sparq_core::Graph::load_dataset(nquads, "nquads")?);
    /// store.materialize_wac()?;
    ///
    /// let resource = NamedNode::new("https://pod.ex/notes/n1").map_err(|e| e.to_string())?;
    /// let alice = Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None, now: None };
    /// // alice holds read+write; everyone (public) holds read.
    /// assert_eq!(store.wac_allow(&alice, &resource), r#"user="read write",public="read""#);
    /// // an anonymous request still learns the public modes (user == public here).
    /// assert_eq!(store.wac_allow(&Session::default(), &resource), r#"user="read",public="read""#);
    /// # Ok::<(), String>(())
    /// ```
    pub fn wac_allow(&mut self, session: &Session, resource: &NamedNode) -> String {
        let user = self.modes_held(session, resource);
        // The `public` field is what an anonymous caller (no WebID) holds — `acl:Read`
        // granted to `foaf:Agent` etc. — independent of the authenticated session.
        let public = self.modes_held(&Session::default(), resource);
        format!(r#"user="{}",public="{}""#, user, public)
    }

    /// The space-separated WAC mode names `session` holds on `resource`'s graph, in the
    /// fixed `read write append control` order — the per-field body of [`Self::wac_allow`].
    /// Empty string when no mode is held (fail-closed). [OPUS-4.8] sq-i7k08.
    fn modes_held(&mut self, session: &Session, resource: &NamedNode) -> String {
        const MODES: [(Mode, &str); 4] = [
            (Mode::Read, "read"),
            (Mode::Write, "write"),
            (Mode::Append, "append"),
            (Mode::Control, "control"),
        ];
        let mut held = Vec::with_capacity(MODES.len());
        for (mode, name) in MODES {
            if self.accessible(session, mode).iter().any(|g| g == resource) {
                held.push(name);
            }
        }
        held.join(" ")
    }

    /// [OPUS-4.8] issue #992 FR-1 (sq-snopa.1) — the per-REQUEST access-control decision:
    /// **may `principal` do `mode` on `resource`?** Returns a typed [`WacDecision`]
    /// (`allow`, `granted_modes`, `governing_acl`, `scope`, fail-closed `status`).
    ///
    /// This is the point-query an LDP resource server asks per request, NOT graph
    /// filtering. Where [`PodStore::query_as`] / [`PodStore::accessible`] return the *set*
    /// of authorized graphs, `decide` answers the single `(principal, resource, mode)`
    /// question and pairs the verdict with the governing-ACL provenance the server needs
    /// for its `Link: rel="acl"` / `WAC-Allow` surfaces.
    ///
    /// It reuses the SAME machinery — the verdict is the fail-closed
    /// [`AuthIndex::accessible`] oracle (the `∪ allow ∖ ∪ deny` per-mode set), the ACL
    /// discovery is the loader's container-chain walk ([`PodStore::resolve_acl`]) — so a
    /// `decide` allow can never be wider than `query_as` would grant.
    ///
    /// # Fail-closed (FR-6, sq-snopa.2 — never fail OPEN)
    ///
    /// Every uncertainty path denies. The typed [`WacDecision::status`] lets a server map
    /// the deny to the right HTTP code — **401/403** for a definitive deny
    /// ([`AclStatus::Resolved`] without the mode, or [`AclStatus::NoAcl`]); a **retryable
    /// 503** for an operational one ([`AclStatus::Unloaded`] — the view was never
    /// materialized; [`AclStatus::Transient`] — e.g. a malformed resource IRI) — but the
    /// `allow == false` is already correct regardless of the status. See [`AclStatus`].
    ///
    /// `resource` is the named-graph IRI backing the LDP resource (mapping the request
    /// PATH to it is the server's job — `sparq-solid` is a library-level authoriser with
    /// no HTTP surface; see `research/sparq-solid-scope.md` §4).
    ///
    /// # Examples
    ///
    /// ```
    /// use sparq_solid::{AclScope, AclStatus, Mode, PodStore, Session};
    ///
    /// let nquads = r#"
    /// <https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hi" <https://pod.ex/notes/n1> .
    /// <https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
    /// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
    /// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
    /// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
    /// "#;
    /// let mut store = PodStore::new(sparq_core::Graph::load_dataset(nquads, "nquads")?);
    /// store.materialize_wac()?;
    ///
    /// let alice = Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None, now: None };
    /// let d = store.decide(&alice, "https://pod.ex/notes/n1", Mode::Read);
    /// assert!(d.allow && d.status == AclStatus::Resolved);
    /// assert_eq!(d.scope, Some(AclScope::Default)); // inherited from the root .acl
    /// // anonymous has no grant → an authoritative deny (403), not a transient one.
    /// let anon = store.decide(&Session::default(), "https://pod.ex/notes/n1", Mode::Read);
    /// assert!(!anon.allow && anon.status == AclStatus::Resolved);
    /// # Ok::<(), String>(())
    /// ```
    pub fn decide(&self, session: &Session, resource: &str, mode: Mode) -> WacDecision {
        let index = decide::AclIndex::build(&self.graph);
        decide::decide_one(&index, &self.auth, session, resource, mode)
    }

    /// [OPUS-4.8] issue #992 FR-1 (sq-snopa.1) — [`PodStore::decide`] for a BATCH of
    /// `(resource, mode)` requests as one `principal`, building the structural ACL index
    /// ONCE and reusing it for every request (the LDP server's "decide a page of
    /// resources" call). Each element's [`WacDecision`] is independent and fail-closed
    /// exactly as [`PodStore::decide`]; the result vector is parallel to the input.
    pub fn decide_batch(&self, session: &Session, requests: &[(&str, Mode)]) -> Vec<WacDecision> {
        let index = decide::AclIndex::build(&self.graph);
        requests
            .iter()
            .map(|(resource, mode)| decide::decide_one(&index, &self.auth, session, resource, *mode))
            .collect()
    }

    /// [OPUS-4.8] issue #992 FR-7 (sq-snopa.3) — resolve the EFFECTIVE governing ACL for a
    /// resource in ONE indexed call: the up-the-container-chain `.acl`/`.acr` discovery +
    /// `acl:default` inheritance, returning the governing ACL IRI + `accessTo`-vs-`default`
    /// scope ([`EffectiveAcl`]) — instead of N HTTP round-trips up the chain.
    ///
    /// The resource's OWN ACL (`<resource>.acl`/`.acr`) wins with [`AclScope::AccessTo`];
    /// otherwise the NEAREST ancestor container that has an ACL governs by
    /// [`AclScope::Default`] (Solid container-`default` inheritance). `None` when no ACL
    /// exists anywhere up to the pod root (an un-protected resource — the caller should
    /// fail closed). The walk is exactly the loader's slash-semantics `parent_iri` chain,
    /// so this per-resource discovery and the materialize-time inheritance can never
    /// disagree. Feeds [`PodStore::decide`]'s `governing_acl`/`scope`, which
    /// [`WacDecision::acl_link_header`] turns into the FR-5 `Link: rel="acl"` surface.
    ///
    /// # Examples
    ///
    /// ```
    /// use sparq_solid::{AclScope, PodStore};
    ///
    /// let nquads = r#"
    /// <https://pod.ex/notes/n1#it> <https://ex.dev/ns#k> "v" <https://pod.ex/notes/n1> .
    /// <https://pod.ex/.acl#a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
    /// "#;
    /// let store = PodStore::new(sparq_core::Graph::load_dataset(nquads, "nquads")?);
    /// let eff = store.resolve_acl("https://pod.ex/notes/n1").expect("inherited acl");
    /// assert_eq!(eff.acl.as_str(), "https://pod.ex/.acl");
    /// assert_eq!(eff.scope, AclScope::Default);
    /// assert!(store.resolve_acl("https://other.ex/x").is_none()); // no ACL anywhere
    /// # Ok::<(), String>(())
    /// ```
    pub fn resolve_acl(&self, resource: &str) -> Option<EffectiveAcl> {
        decide::AclIndex::build(&self.graph).resolve_acl(resource)
    }

    fn session_sets(&mut self, s: &Session, mode: Mode) -> &SessionSets {
        let key = (
            s.agent.map(str::to_owned),
            s.client.map(str::to_owned),
            s.issuer.map(str::to_owned),
            s.now.map(str::to_owned),
            mode,
        );
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
    /// let alice = Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None, now: None };
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

    /// Apply a SPARQL Update as `session`, **enforcing write access control** — the
    /// write-path mirror of [`PodStore::query_as`] (design doc §4.4 / §7 item 6).
    /// [OPUS-4.8] sq-xor3.
    ///
    /// Every graph the update could mutate is checked against the session's WAC/ACP
    /// **write** permission BEFORE anything is applied: `acl:Write` ([`Mode::Write`])
    /// for a delete/clear, and `acl:Write` OR `acl:Append` ([`Mode::Write`] /
    /// [`Mode::Append`]) for a pure `INSERT`. The permission model is identical to the
    /// read path — the same `∪ allow ∖ ∪ deny` per-mode graph sets from the materialized
    /// auth view. Writing an `.acl`/`.acr` access-control document needs `Write` on that
    /// graph, which the rules grant only to `acl:Control` holders — so Control gates the
    /// rules through exactly the same auth view, with no Solid-specific branch.
    ///
    /// Fail-closed:
    ///
    /// - if any target is not writable, the update is **denied** (`Err`) and the store
    ///   is left untouched (the whole check runs before [`sparq_engine::update_in_place`]);
    /// - writes to the **default graph** are always denied (pod data lives in named
    ///   graphs only);
    /// - a `DELETE`/`INSERT … WHERE` with a `GRAPH ?var` slot is resolved **precisely**
    ///   ([OPUS-4.8] sq-biss): the operation's WHERE is evaluated to enumerate the
    ///   concrete graphs `?var` binds to (exactly the set the apply will write), and
    ///   write is required only on those — not on every store graph. Still fail-closed:
    ///   any bound graph the actor cannot write denies the whole update, and a binding
    ///   that cannot be reduced to a writable named graph (a blank-node graph name, a
    ///   `USING`/`WITH`-re-scoped op, or a WHERE the check cannot evaluate) falls back to
    ///   the conservative all-graphs check below;
    /// - a target that spans all graphs (`CLEAR`/`DROP` `ALL`/`NAMED`), or a `GRAPH ?var`
    ///   slot that fell back, requires write on **every** graph in the store — sound
    ///   (never permissive), at the cost of denying some updates a per-solution check
    ///   might allow.
    ///
    /// On a permitted update that touched an `.acl`/`.acr`/group document — any static
    /// control-doc write, any precisely-resolved variable-graph update (its targets could
    /// include a group document, which has no naming convention), or any graph-wildcard
    /// update — the auth view is **re-materialized** automatically (WAC by default; pass
    /// [`PodStore::update_as_acp`] for ACP pods), so a changed rule takes effect on the
    /// next call.
    ///
    /// # Errors
    ///
    /// `Err` if `sparql` does not parse as a SPARQL Update, if the session lacks write
    /// permission on any target (the deny path), or if the engine fails to apply the
    /// (already-authorized) update.
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
    /// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Write> <https://pod.ex/.acl> .
    /// "#;
    /// let mut store = PodStore::new(sparq_core::Graph::load_dataset(nquads, "nquads")?);
    /// store.materialize_wac()?;
    ///
    /// let alice = Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None, now: None };
    /// let ins = "INSERT DATA { GRAPH <https://pod.ex/notes/n1> { \
    ///     <https://pod.ex/notes/n1#it> <https://ex.dev/ns#tag> \"x\" } }";
    /// // alice has acl:Write on n1 -> permitted
    /// store.update_as(&alice, ins)?;
    /// // anonymous has no write grant -> denied, store untouched
    /// assert!(store.update_as(&Session::default(), ins).is_err());
    /// # Ok::<(), String>(())
    /// ```
    pub fn update_as(&mut self, s: &Session, sparql: &str) -> Result<(), String> {
        self.update_inner(s, sparql, false)
    }

    /// [`PodStore::update_as`] for **ACP** pods: identical write-access enforcement, but
    /// a permitted control-doc/group write re-materializes the ACP view
    /// ([`PodStore::materialize_acp`]) instead of the WAC one. [OPUS-4.8] sq-xor3.
    pub fn update_as_acp(&mut self, s: &Session, sparql: &str) -> Result<(), String> {
        self.update_inner(s, sparql, true)
    }

    fn update_inner(&mut self, s: &Session, sparql: &str, acp: bool) -> Result<(), String> {
        // Authorize against the CURRENT auth view before mutating anything (fail-closed).
        let auth = Arc::clone(&self.auth);
        let permit = update::check(&self.graph, &auth, s, sparql)?;
        // Authorized: apply through the engine's in-place delta path.
        sparq_engine::update_in_place(&mut self.graph, sparql)?;
        // A change to the access-control rules invalidates the auth view.
        if permit.rematerialize {
            if acp {
                self.materialize_acp()?;
            } else {
                self.materialize_wac()?;
            }
        }
        Ok(())
    }

    /// [OPUS-4.8] sq-h3uk — evaluate an ODRL `policy` against `request` and, on a
    /// definite Permit, materialize the equivalent WAC/ACP grant into this store's
    /// `<urn:sparq:auth>` view, then rebuild the session index so the grant takes
    /// effect on the next [`PodStore::accessible`] / [`PodStore::query_as`] call.
    ///
    /// This is the [`PodStore`] entry point for the opt-in ODRL bridge
    /// ([`odrl_bridge::materialize_permission`]): a matched ODRL `Permission`
    /// (action + constraints satisfied + duties discharged) becomes a concrete
    /// `principal auth:<mode> graph` triple honoured by the existing graph-level
    /// enforcement — **no new enforcement engine**. The grant is APPENDED to the
    /// current auth view (any WAC/ACP grants already materialized are preserved).
    ///
    /// **Fail-closed:** a Deny, an ambiguous evaluation, an unmapped ODRL action, or
    /// a request without a concrete party/target materializes NOTHING and leaves the
    /// auth view (and so the session index) untouched — see
    /// [`odrl_bridge::materialize_permission`] for the exact predicates and the
    /// [action→mode mapping](odrl_bridge#action--mode-mapping).
    ///
    /// Call [`PodStore::materialize_wac`] / [`materialize_acp`](PodStore::materialize_acp)
    /// FIRST if you also want the static WAC/ACP grants present; the bridge adds to
    /// whatever auth view currently exists (or creates a fresh one holding just the
    /// bridged grant).
    #[cfg(feature = "odrl-bridge")]
    pub fn materialize_odrl_permission(
        &mut self,
        policy: &sparq_policy::Policy,
        request: &sparq_policy::Request,
    ) -> odrl_bridge::BridgeOutcome {
        let outcome = odrl_bridge::materialize_permission(&mut self.graph, policy, request);
        if outcome.granted {
            // Track for refresh/retraction (sq-dpk4), then rebuild index + drop cache.
            self.bridge_ledger.record(policy, request, odrl_bridge::BridgeKind::Permission);
            self.reindex();
        }
        outcome
    }

    /// [OPUS-4.8] sq-w693 — evaluate an ODRL `policy`'s prohibitions against `request`
    /// and, on a matched Prohibition that carves the request out, materialize the
    /// equivalent `principal auth:deny<Mode> graph` DENY into this store's
    /// `<urn:sparq:auth>` view, then rebuild the session index so the deny takes effect
    /// on the next [`PodStore::accessible`] / [`PodStore::query_as`] call.
    ///
    /// The materialized deny is honoured by the EXISTING enforcement under
    /// **deny-overrides**: the session layer computes `∪ allow ∖ ∪ deny`, so the deny
    /// beats any allow grant for the same principal+target+mode — see
    /// [`odrl_bridge::materialize_prohibition`].
    ///
    /// **Fail-closed:** no matching prohibition, an unmapped action, or a request
    /// without a concrete party/target materializes NOTHING and leaves the auth view
    /// (and session index) untouched.
    #[cfg(feature = "odrl-bridge")]
    pub fn materialize_odrl_prohibition(
        &mut self,
        policy: &sparq_policy::Policy,
        request: &sparq_policy::Request,
    ) -> odrl_bridge::BridgeOutcome {
        let outcome = odrl_bridge::materialize_prohibition(&mut self.graph, policy, request);
        if outcome.prohibited {
            self.bridge_ledger.record(policy, request, odrl_bridge::BridgeKind::Prohibition);
            self.reindex();
        }
        outcome
    }

    /// [OPUS-4.8] sq-w693 — materialize **both** sides of an ODRL `policy` for
    /// `request` into this store's `<urn:sparq:auth>` view: the Permit allow grant AND
    /// the matched-Prohibition deny ([`odrl_bridge::materialize_policy`]), then rebuild
    /// the session index if either materialized.
    ///
    /// A policy with both a permission and a prohibition for the same
    /// principal+target+mode materializes both triples; the deny **wins** at
    /// enforcement time (deny-overrides — the session layer subtracts `∪ deny` from
    /// `∪ allow`). Each side is independently fail-closed.
    #[cfg(feature = "odrl-bridge")]
    pub fn materialize_odrl_policy(
        &mut self,
        policy: &sparq_policy::Policy,
        request: &sparq_policy::Request,
    ) -> odrl_bridge::BridgeOutcome {
        let outcome = odrl_bridge::materialize_policy(&mut self.graph, policy, request);
        if outcome.granted || outcome.prohibited {
            self.bridge_ledger.record(policy, request, odrl_bridge::BridgeKind::Policy);
            self.reindex();
        }
        outcome
    }

    /// [OPUS-4.8] sq-hiz4 — like [`PodStore::materialize_odrl_permission`], but persists a
    /// FAITHFULLY-mappable ODRL constraint (recipient/assignee) as a re-checked ACP
    /// `auth:ConditionalGrant` rather than freezing it into a one-shot allow: the
    /// granted agent is re-verified per session through the SAME enforcement path
    /// ([`PodStore::accessible`] / [`PodStore::query_as`]), not re-running the ODRL
    /// evaluator.
    ///
    /// A constraint with **no** faithful ACP-condition analogue (`odrl:purpose`,
    /// `odrl:dateTime`/time windows, `odrl:count`, a `neq`/order recipient) keeps the
    /// one-shot materialization-time behaviour (checked once, frozen) — see
    /// [`odrl_bridge::materialize_permission_conditional`] for the full mapping table
    /// and the fail-closed rationale.
    #[cfg(feature = "odrl-bridge")]
    pub fn materialize_odrl_permission_conditional(
        &mut self,
        policy: &sparq_policy::Policy,
        request: &sparq_policy::Request,
    ) -> odrl_bridge::BridgeOutcome {
        let outcome =
            odrl_bridge::materialize_permission_conditional(&mut self.graph, policy, request);
        if outcome.granted {
            self.bridge_ledger.record(
                policy,
                request,
                odrl_bridge::BridgeKind::PermissionConditional,
            );
            self.reindex();
        }
        outcome
    }

    /// [OPUS-4.8] sq-4r70 — the deny dual of
    /// [`PodStore::materialize_odrl_permission_conditional`]: persist a matched ODRL
    /// **prohibition** whose recipient/assignee constraints map faithfully as a
    /// re-checked ACP **conditional deny** (`auth:effect auth:Deny`) rather than freezing
    /// it one-shot. The carve-out is re-verified per session through the SAME enforcement
    /// path ([`PodStore::accessible`] / [`PodStore::query_as`]); the deny **overrides**
    /// any allow for the same principal+target+mode (deny-overrides).
    ///
    /// A prohibition carrying a constraint with **no** faithful ACP-condition analogue
    /// (`odrl:purpose`, `odrl:dateTime`, `odrl:count`) falls back to the one-shot deny
    /// ([`odrl_bridge::materialize_prohibition`], frozen at materialization) — see
    /// [`odrl_bridge::materialize_prohibition_conditional`] for the full rationale.
    #[cfg(feature = "odrl-bridge")]
    pub fn materialize_odrl_prohibition_conditional(
        &mut self,
        policy: &sparq_policy::Policy,
        request: &sparq_policy::Request,
    ) -> odrl_bridge::BridgeOutcome {
        let outcome =
            odrl_bridge::materialize_prohibition_conditional(&mut self.graph, policy, request);
        if outcome.prohibited {
            self.bridge_ledger.record(
                policy,
                request,
                odrl_bridge::BridgeKind::ProhibitionConditional,
            );
            self.reindex();
        }
        outcome
    }

    /// [OPUS-4.8] sq-58mh — STATEFUL `odrl:count` enforcement THROUGH the bridge: evaluate
    /// `policy` against `request`, **atomically consume** one unit of any applicable
    /// `odrl:count` budget from `store`, and on a grant materialize the equivalent
    /// `principal auth:<mode> graph` allow into this store's `<urn:sparq:auth>` view — so
    /// the existing graph-level WAC/ACP enforcement honours it. The grant then
    /// **self-retracts on exhaustion**: it is tracked as
    /// [`odrl_bridge::BridgeKind::PermissionCounted`], and the next
    /// [`PodStore::refresh_odrl_grants`] re-checks the budget READ-ONLY (never consuming)
    /// and RETRACTS the grant once the budget is spent — access is GONE through
    /// [`PodStore::accessible`] / [`PodStore::query_as`].
    ///
    /// This closes the gap the [`PodStore::materialize_odrl_permission_conditional`]
    /// mapping table left open: ACP is stateless (no per-session usage counter), so the
    /// count cannot be a re-checked ACP *condition*; instead it is enforced via the
    /// EXISTING refresh/retraction ledger (sq-dpk4). The decision routes through
    /// sparq-policy's [`sparq_policy::evaluate_and_exercise`] — the first *N* exercises of
    /// an "at most *N*" permission grant; the *(N+1)*th denies; a denied / exhausted /
    /// store-unavailable exercise burns no budget and materializes nothing (fail-closed).
    ///
    /// `store` is the injected [`sparq_policy::UsageCounterStore`] (shared via `Arc` so
    /// the same budgets back both exercise-time consumption and refresh-time re-checks);
    /// [`sparq_policy::InMemoryCounterStore`] is the in-process reference impl. A
    /// distributed deployment supplies a shared-atomic backend honouring the same
    /// `try_consume` contract. The budget key is `(rule_id, party, target)`
    /// ([`sparq_policy::CountKey`]) — per-assignee, per-asset, per-rule.
    ///
    /// **Fail-closed:** a base Deny, an exhausted/unavailable count, an unmapped action,
    /// or a request without a concrete party/target materializes NOTHING. Only present
    /// under the `count-enforcement` feature.
    #[cfg(feature = "count-enforcement")]
    pub fn materialize_odrl_permission_counted(
        &mut self,
        policy: &sparq_policy::Policy,
        request: &sparq_policy::Request,
        store: &Arc<dyn sparq_policy::UsageCounterStore + Send + Sync>,
    ) -> odrl_bridge::BridgeOutcome {
        let outcome = odrl_bridge::count::materialize_permission_counted(
            &mut self.graph,
            policy,
            request,
            store.as_ref(),
        );
        if outcome.granted {
            // Remember the store so refresh can re-check the budget (read-only), then
            // track the counted grant for refresh/retraction, then rebuild the index.
            self.bridge_ledger
                .set_count_store(&odrl_bridge::count::CounterHandle(Arc::clone(store)));
            self.bridge_ledger.record(
                policy,
                request,
                odrl_bridge::BridgeKind::PermissionCounted,
            );
            self.reindex();
        }
        outcome
    }

    /// [OPUS-4.8] sq-dpk4 — re-evaluate **every bridged ODRL grant** this store is
    /// tracking against its (possibly changed) policy, and **retract** the ones that no
    /// longer hold, while preserving static WAC/ACP grants and still-valid bridged
    /// grants. Call this after an ODRL policy changes — a permission is withdrawn, a
    /// time window lapses, or a re-evaluation now Denies — so a STALE bridged grant
    /// loses access (the gap from sq-h3uk/#280: the bridge only ever appended, never
    /// retracted).
    ///
    /// # How it works
    ///
    /// The auth view is rebuilt as `static_baseline ∪ replay(still-valid bridged
    /// entries)`: the `<urn:sparq:auth>` view is reset to the static (WAC/ACP) baseline
    /// captured at the last `materialize_wac`/`materialize_acp`, the bridged-provenance
    /// graph is cleared, and each tracked `(policy, request)` is re-evaluated through its
    /// original bridge entry point. An entry that still yields a grant is re-applied; an
    /// entry that now yields nothing is dropped (retracted). The session index is rebuilt
    /// and the cache dropped, so the change takes effect on the next
    /// [`PodStore::accessible`] / [`PodStore::query_as`].
    ///
    /// Returns the number of bridged grants RETRACTED.
    ///
    /// # Fail-closed (security-sensitive — access retraction)
    ///
    /// - A withdrawn permission, a lapsed time window, or a re-evaluation that now Denies
    ///   (including a now-matching prohibition) emits nothing on replay → the grant is
    ///   removed → access is GONE. On any ambiguous re-evaluation the underlying ODRL
    ///   evaluator denies, so the entry is retracted rather than left stale.
    /// - A static WAC/ACP grant is never in the ledger, never re-evaluated, and always in
    ///   the baseline — refresh can neither widen nor drop it.
    /// - If the policy supplied to refresh differs from what was bridged, pass the new
    ///   policy by re-running the relevant `materialize_odrl_*` (which re-records) before
    ///   refreshing, OR rely on the policy embedded in the tracked entry; this method
    ///   replays the policy AS TRACKED — to refresh against a mutated policy, call the
    ///   bridge entry point again with the new policy first.
    #[cfg(feature = "odrl-bridge")]
    pub fn refresh_odrl_grants(&mut self) -> usize {
        let retracted = self.bridge_ledger.refresh(&mut self.graph);
        self.reindex();
        retracted
    }

    /// [OPUS-4.8] sq-dpk4 — refresh the bridged grant for one slot against an **updated**
    /// policy / request, then re-evaluate the whole ledger.
    ///
    /// This is the entry point for the real revocation cases: the ODRL `policy` changed
    /// (a permission withdrawn, a prohibition added) or the request context moved on (a
    /// time window lapsed — pass a `request` carrying the current `odrl:dateTime`). It
    /// replaces the tracked `(policy, request)` for the grant slot matching
    /// `(kind, request.target, request.party)` with the supplied ones, then runs
    /// [`PodStore::refresh_odrl_grants`]: the updated entry is re-evaluated and, if it no
    /// longer holds, RETRACTED — access is gone.
    ///
    /// Returns `(matched, retracted)`: `matched` is whether a tracked grant slot was
    /// updated (`false` ⇒ nothing tracked for that `(kind, target, party)`, nothing
    /// changed), and `retracted` is the total number of bridged grants retracted by the
    /// ensuing refresh.
    ///
    /// Fail-closed exactly as [`PodStore::refresh_odrl_grants`]: a now-Deny / lapsed /
    /// withdrawn / now-prohibited re-evaluation loses access, static grants are untouched.
    #[cfg(feature = "odrl-bridge")]
    pub fn refresh_odrl_grant(
        &mut self,
        policy: &sparq_policy::Policy,
        request: &sparq_policy::Request,
        kind: odrl_bridge::BridgeKind,
    ) -> (bool, usize) {
        let matched = self.bridge_ledger.update(policy, request, kind);
        let retracted = self.bridge_ledger.refresh(&mut self.graph);
        self.reindex();
        (matched, retracted)
    }

    /// [OPUS-4.8] sq-dpk4 — the bridge ledger (tracked bridged grants + static
    /// baseline), for inspection/audit. Only present under the `odrl-bridge` feature.
    #[cfg(feature = "odrl-bridge")]
    pub fn bridge_ledger(&self) -> &odrl_bridge::BridgeLedger {
        &self.bridge_ledger
    }

    /// The current auth index (for direct inspection/tests).
    ///
    /// Rebuilt (and the session cache cleared) on every re-materialization; an
    /// un-materialized store exposes an empty index.
    pub fn auth(&self) -> &AuthIndex {
        &self.auth
    }
}
