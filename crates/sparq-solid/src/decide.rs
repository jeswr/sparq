//! [OPUS-4.8] issue #992 Phase-1 — the per-resource WAC **decision** layer
//! (beads sq-snopa.1 / .2 / .3). Where the rest of this crate filters *graphs* (the
//! authorization *oracle*: `accessible` / `query_as` / `update_as`), this module answers
//! the one question an LDP resource server asks per request: **"may principal X do mode M
//! on resource R?"** — as a typed [`WacDecision`], with the governing ACL + scope and a
//! fail-closed load/error [`AclStatus`].
//!
//! It builds ENTIRELY on the existing machinery — [`crate::AuthIndex::accessible`] for the
//! verdict, the loader's container-chain walk (`loader::parent_iri`) for ACL discovery —
//! and adds no dependency, so it is an always-present public API (no cargo gate), mirroring
//! [`crate::PodStore::wac_allow`].
//!
//! # Stability — **API tier-1 (proposed-stable)**
//!
//! The per-resource decision surface ([`crate::PodStore::decide`] / `decide_batch` /
//! `resolve_acl` / [`crate::PodStore::wac_allow`], with [`WacDecision`] / [`AclStatus`] /
//! [`AclScope`] / [`EffectiveAcl`]) is proposed **tier-1** (semver-stable) in the [API
//! stability & deprecation policy]. The freeze is **pending maintainer ratification**
//! (issue #1346 P0 / #1248); the marker asserts a *proposal*, not an active guarantee.
//! This is the authorization-**decision** surface only — it does not authenticate and makes
//! no cryptographic claim (a `Session` is a caller-asserted claim; see the crate README).
//!
//! [API stability & deprecation policy]: https://github.com/jeswr/sparq/blob/main/docs/api-stability.md
//!
//! # Security posture — fail-CLOSED, never fail-OPEN (FR-6, sq-snopa.2)
//!
//! This is authorization decision logic; the cardinal rule is **any uncertainty ⇒ deny**.
//! Every error/absence path resolves to `allow == false`, with a typed [`AclStatus`] the
//! server maps to an HTTP status — but the deny is independent of the status, so a caller
//! that ignores the status STILL fails closed. The grant verdict reuses
//! [`crate::AuthIndex::accessible`], which is itself fail-closed (an un-materialized view,
//! a grant-less session, or a reserved-encoding session value all degrade to the empty
//! set), so a `decide` allow can never be wider than the oracle would grant.

use crate::authindex::Mode;
use crate::loader::{parent_iri, ACL_SUFFIX, ACR_SUFFIX};
use crate::{AuthIndex, Session, AUTH_GRAPH};
use oxrdf::{NamedNode, Term};
use sparq_core::Graph;

/// Where a governing ACL grant applies relative to the resource it governs — the WAC
/// `acl:accessTo` / `acl:default` distinction (FR-7, sq-snopa.3), surfaced for
/// debuggability ([`AclScope::as_acl_predicate`]) and the `Link: rel="acl"` surface
/// ([`WacDecision::acl_link_header`], FR-5, sq-snopa.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AclScope {
    /// The resource has its **own** access-control document (`<R> + ".acl"`/`".acr"`),
    /// and authorizations there target it directly (WAC `acl:accessTo <R>`).
    AccessTo,
    /// No own ACL; the governing document is the **nearest ancestor container's** ACL,
    /// inherited via WAC `acl:default` (Solid container-`default` inheritance).
    Default,
}

impl AclScope {
    /// The WAC predicate IRI this scope corresponds to — `acl:accessTo` for
    /// [`AclScope::AccessTo`], `acl:default` for [`AclScope::Default`] (FR-5, sq-snopa.4).
    /// Surfaced for debuggability and structured logs: a server can record *why* an ACL
    /// governs a resource (its own vs an inherited container default) by the exact WAC term.
    pub fn as_acl_predicate(self) -> &'static str {
        match self {
            AclScope::AccessTo => "http://www.w3.org/ns/auth/acl#accessTo",
            AclScope::Default => "http://www.w3.org/ns/auth/acl#default",
        }
    }
}

/// The typed fail-closed load/error contract for a decision (FR-6, sq-snopa.2).
///
/// Carries over the hard-won fail-OPEN lessons: it lets a resource server distinguish a
/// *definitive* deny (map to **401/403**) from a *retryable* one (map to **503**) —
/// **without ever failing open**. The [`WacDecision::allow`] flag is `false` for every
/// status except [`AclStatus::Resolved`], and `Resolved` only ever carries the verdict
/// the materialized auth view actually supports. A server that ignores `status` entirely
/// still gets the correct allow/deny; `status` only refines the *HTTP code*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AclStatus {
    /// A governing ACL was discovered and the auth view is loaded: the decision is
    /// **authoritative**. `allow` is the real verdict. (An authoritative *deny* — the
    /// principal simply lacks the mode — is also `Resolved`; map it to **403**.)
    Resolved,
    /// No governing ACL exists anywhere up the container chain (the resource is
    /// **un-protected by any discoverable ACL**). Fail-closed ⇒ `allow == false`. A
    /// Solid server treats this as a definitive deny — map to **403** (or **401** for an
    /// anonymous principal that might succeed once authenticated).
    NoAcl,
    /// A governing ACL was discovered, but the authorization view is **not loaded**
    /// (`materialize_wac` / `materialize_acp` has not been run, so no verdict can be
    /// computed). Fail-closed ⇒ `allow == false`. This is a *transient/operational*
    /// condition, not a permission denial — map to a **retryable 503**.
    Unloaded,
    /// A **typed transient error** occurred while deciding (e.g. a malformed resource
    /// IRI the server passed in). Fail-closed ⇒ `allow == false`. Map to a **retryable
    /// 503** — the request is not necessarily forbidden, the decision just could not be
    /// computed *this time*.
    Transient,
}

impl AclStatus {
    /// Whether this status denotes a **retryable** (operational/transient) condition a
    /// server should map to **503**, rather than a definitive permission outcome
    /// (401/403). Convenience for the resource-server status mapping; `Unloaded` and
    /// `Transient` are retryable, `Resolved` and `NoAcl` are definitive. Fail-closed is
    /// orthogonal — every non-`Resolved` status still carries `allow == false`.
    pub fn is_retryable(self) -> bool {
        matches!(self, AclStatus::Unloaded | AclStatus::Transient)
    }
}

/// The effective governing ACL for a resource: the discovered ACL document IRI plus the
/// scope by which it governs (FR-7, sq-snopa.3). Produced by
/// [`crate::PodStore::resolve_acl`] in ONE indexed pass over the dataset's named graphs —
/// no per-ancestor round-trips.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectiveAcl {
    /// The discovered access-control document IRI (`<R>.acl` for an own ACL, or the
    /// nearest ancestor container's `<C>.acl` for an inherited one).
    pub acl: NamedNode,
    /// Whether the ACL governs `R` directly (`acl:accessTo`) or by container inheritance
    /// (`acl:default`).
    pub scope: AclScope,
}

/// A per-request access-control decision for one `(principal, resource, mode)` (FR-1,
/// sq-snopa.1).
///
/// The point-query analogue of the graph-filtering oracle: a Solid resource server hands
/// this straight to its request pipeline. It is **fail-closed by construction** — see the
/// module docs and [`AclStatus`]. The grant verdict reuses
/// [`crate::AuthIndex::accessible`] (the same `∪ allow ∖ ∪ deny` per-mode set), so a
/// `decide` allow is never wider than what `accessible` / `query_as` would grant.
///
/// **API tier-1 (proposed-stable)** — the return type of the proposed semver-stable
/// per-resource decision surface; see the module docs and the [API stability policy].
///
/// [API stability policy]: https://github.com/jeswr/sparq/blob/main/docs/api-stability.md
///
/// # Examples
///
/// ```
/// use sparq_solid::{AclScope, AclStatus, Mode, PodStore, Session};
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
/// let d = store.decide(&alice, "https://pod.ex/notes/n1", Mode::Read);
/// assert!(d.allow);
/// assert_eq!(d.status, AclStatus::Resolved);
/// assert_eq!(d.scope, Some(AclScope::Default)); // inherited from the root `.acl`
/// assert!(d.granted_modes.contains(&Mode::Read));
/// // alice has only Read — Write is an authoritative deny, NOT a transient one.
/// let dw = store.decide(&alice, "https://pod.ex/notes/n1", Mode::Write);
/// assert!(!dw.allow);
/// assert_eq!(dw.status, AclStatus::Resolved);
/// # Ok::<(), String>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WacDecision {
    /// The verdict for the requested mode. **Fail-closed**: `false` for every non-`Resolved`
    /// status, and `false` whenever the principal lacks the mode.
    pub allow: bool,
    /// Every mode the principal holds on the resource (sorted, deduped) — so the server can
    /// build a `WAC-Allow` advertisement or a 403 body from one decision. Empty unless
    /// `status == Resolved`.
    pub granted_modes: Vec<Mode>,
    /// The governing ACL document IRI, when one was discovered (FR-7). `None` when
    /// `status == NoAcl` (no ACL anywhere in the chain), or on a `Transient` error.
    pub governing_acl: Option<NamedNode>,
    /// How the governing ACL applies — `AccessTo` (own ACL) or `Default` (inherited).
    /// `Some` exactly when `governing_acl` is `Some`.
    pub scope: Option<AclScope>,
    /// The typed fail-closed load/error status (FR-6). Drives the server's HTTP mapping;
    /// the allow/deny itself is already correct regardless of it.
    pub status: AclStatus,
}

impl WacDecision {
    /// A fail-closed deny with the given status and no governing ACL — the single
    /// constructor for every uncertainty path, so deny-by-default is impossible to forget.
    fn deny(status: AclStatus) -> WacDecision {
        WacDecision { allow: false, granted_modes: Vec::new(), governing_acl: None, scope: None, status }
    }

    /// The Solid [`Link: rel="acl"`](https://solidproject.org/TR/protocol#acl-resource)
    /// response-header **value** advertising the governing ACL document, as an RFC 8288
    /// link-value: `<acl-iri>; rel="acl"` (FR-5, sq-snopa.4). This is the discoverability
    /// surface a resource server emits alongside [`crate::PodStore::wac_allow`] so a client
    /// can fetch/edit the effective ACL — built straight from the provenance FR-1 already
    /// puts on the decision, so it can never point at a document discovery did not find.
    ///
    /// **`None` when no governing ACL was discovered** (`status == NoAcl`, or a `Transient`
    /// error): there is no ACL to advertise, so nothing is emitted. Fail-closed — the link
    /// surfaces *where* the governing ACL is, never a verdict, so it is safe on a deny.
    /// `Some` exactly when [`WacDecision::governing_acl`] is `Some` (including `Unloaded`,
    /// where the ACL was discovered but the view is not yet materialized).
    ///
    /// # Examples
    ///
    /// ```
    /// use sparq_solid::{Mode, PodStore, Session};
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
    /// let alice = Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None, now: None };
    /// let d = store.decide(&alice, "https://pod.ex/notes/n1", Mode::Read);
    /// assert_eq!(d.acl_link_header().as_deref(), Some(r#"<https://pod.ex/.acl>; rel="acl""#));
    /// // No ACL anywhere ⇒ no link to advertise.
    /// let none = store.decide(&alice, "https://other.ex/x", Mode::Read);
    /// assert!(none.acl_link_header().is_none());
    /// # Ok::<(), String>(())
    /// ```
    pub fn acl_link_header(&self) -> Option<String> {
        // [OPUS-4.8] POSITIONAL format args (CodeQL rust/unused-variable false positive).
        self.governing_acl
            .as_ref()
            .map(|acl| format!("<{}>; rel=\"acl\"", acl.as_str()))
    }
}

/// A read-only structural index of a dataset's named graphs: which IRIs are control
/// documents (`.acl`/`.acr`), and whether the auth view is loaded. Built ONCE per
/// `decide_batch` / `resolve_acl` call so a batch of N decisions is N point lookups, not
/// N full scans (FR-7's "one indexed call, not N round-trips"). [OPUS-4.8]
pub(crate) struct AclIndex {
    /// Every control-document IRI present (both `.acl` and `.acr` — a pod is one system,
    /// but we don't assume which, and an own-ACL lookup is just a set membership test).
    control: rustc_hash::FxHashSet<String>,
    /// Whether the materialized auth view (`<urn:sparq:auth>`) is present in the dataset.
    materialized: bool,
}

impl AclIndex {
    /// Index the dataset's named-graph names in one pass.
    pub(crate) fn build(graph: &Graph) -> AclIndex {
        let mut control = rustc_hash::FxHashSet::default();
        let mut materialized = false;
        for (name, _) in &graph.named {
            let Term::NamedNode(n) = name else { continue };
            let iri = n.as_str();
            if iri == AUTH_GRAPH {
                materialized = true;
            } else if iri.ends_with(ACL_SUFFIX) || iri.ends_with(ACR_SUFFIX) {
                control.insert(iri.to_owned());
            }
        }
        AclIndex { control, materialized }
    }

    /// The control-document IRI governing `resource`, if one exists in the dataset
    /// (`<resource>.acl` / `<resource>.acr`).
    fn own_acl(&self, resource: &str) -> Option<String> {
        for suffix in [ACL_SUFFIX, ACR_SUFFIX] {
            let candidate = format!("{}{}", resource, suffix);
            if self.control.contains(&candidate) {
                return Some(candidate);
            }
        }
        None
    }

    /// FR-7 (sq-snopa.3) — resolve the EFFECTIVE governing ACL for `resource` in ONE
    /// indexed call: the up-the-container-chain `.acl`/`.acr` discovery + `acl:default`
    /// inheritance, returning the governing ACL IRI + `accessTo`-vs-`default` scope.
    ///
    /// Mirrors the materialize-time nearest-ancestor walk in `rules/wac.n3` /
    /// `rules/common.n3`: the resource's own ACL (`acl:accessTo`) wins; otherwise the
    /// nearest ancestor container that HAS an ACL governs by `acl:default`. `None` when no
    /// ACL exists anywhere up to the pod root (the resource is un-protected ⇒ the caller
    /// fails closed). The container-prefix walk is exactly the loader's `parent_iri`
    /// slash-semantics walk, so discovery and materialization can never disagree.
    pub(crate) fn resolve_acl(&self, resource: &str) -> Option<EffectiveAcl> {
        // Own ACL → accessTo scope.
        if let Some(acl) = self.own_acl(resource) {
            return Some(EffectiveAcl { acl: NamedNode::new_unchecked(acl), scope: AclScope::AccessTo });
        }
        // Walk up the container chain; the nearest ancestor with an ACL governs by default.
        let mut cur = resource;
        while let Some(parent) = parent_iri(cur) {
            if let Some(acl) = self.own_acl(parent) {
                return Some(EffectiveAcl { acl: NamedNode::new_unchecked(acl), scope: AclScope::Default });
            }
            cur = parent;
        }
        None
    }
}

/// Compute the decision for one `(session, resource, mode)` against a prebuilt index +
/// auth index. Shared by [`crate::PodStore::decide`] and `decide_batch` (the batch path
/// builds the [`AclIndex`] once). [OPUS-4.8]
pub(crate) fn decide_one(
    index: &AclIndex,
    auth: &AuthIndex,
    session: &Session,
    resource: &str,
    mode: Mode,
) -> WacDecision {
    // A malformed resource IRI is a typed transient error — fail closed, retryable.
    let Ok(res_node) = NamedNode::new(resource) else {
        return WacDecision::deny(AclStatus::Transient);
    };

    // FR-7: discover the governing ACL up the container chain in one indexed pass.
    let Some(effective) = index.resolve_acl(resource) else {
        // No ACL anywhere ⇒ the resource is un-protected by any discoverable ACL ⇒ deny.
        return WacDecision::deny(AclStatus::NoAcl);
    };

    // FR-6: a governing ACL exists but the view is not loaded ⇒ retryable deny.
    if !index.materialized {
        return WacDecision {
            allow: false,
            granted_modes: Vec::new(),
            governing_acl: Some(effective.acl),
            scope: Some(effective.scope),
            status: AclStatus::Unloaded,
        };
    }

    // Authoritative: ask the (fail-closed) oracle for every mode the session holds on R.
    let granted_modes = held_modes(auth, session, &res_node);
    let allow = granted_modes.contains(&mode);
    WacDecision {
        allow,
        granted_modes,
        governing_acl: Some(effective.acl),
        scope: Some(effective.scope),
        status: AclStatus::Resolved,
    }
}

/// The sorted set of modes `session` holds on `resource`, via the fail-closed oracle.
/// The point-query analogue of `PodStore::modes_held`, over the index directly (no
/// per-session cache here — `decide` is a single index walk).
///
/// [OPUS-4.8] sq-b7k7u (issue #1571): the accessibility check is restricted to
/// `resource`'s OWN origin ([`AuthIndex::accessible_in_origin`]) instead of building the
/// whole-store accessible set four times. Since `resource` has exactly that origin,
/// membership is identical to the full [`AuthIndex::accessible`] check, but a per-request
/// decision now pays only that origin's grants — not every tenant's — so post-write
/// decision latency no longer scales with the whole store.
fn held_modes(auth: &AuthIndex, session: &Session, resource: &NamedNode) -> Vec<Mode> {
    const MODES: [Mode; 4] = [Mode::Read, Mode::Write, Mode::Append, Mode::Control];
    let origin = crate::loader::iri_origin(resource.as_str());
    MODES
        .into_iter()
        .filter(|&m| auth.accessible_in_origin(session, m, origin).iter().any(|g| g == resource))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(nquads: &str) -> Graph {
        Graph::load_dataset(nquads, "nquads").expect("loads")
    }

    // A root `.acl` granting alice Read by `acl:default`, plus a doc under /notes/.
    const ROOT_ACL: &str = r#"
<https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
<https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
<https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
"#;

    #[test]
    fn resolve_inherited_default_scope() {
        let g = graph(ROOT_ACL);
        let ix = AclIndex::build(&g);
        let eff = ix
            .resolve_acl("https://pod.ex/notes/n1")
            .expect("inherited acl");
        assert_eq!(eff.acl.as_str(), "https://pod.ex/.acl");
        assert_eq!(eff.scope, AclScope::Default);
    }

    #[test]
    fn resolve_own_acl_access_to_scope() {
        // A doc with its OWN .acl → accessTo scope, even though the root also has one.
        let nquads = format!(
            "{ROOT_ACL}\
             <https://pod.ex/notes/n1.acl#a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/notes/n1.acl> .\n"
        );
        let g = graph(&nquads);
        let ix = AclIndex::build(&g);
        let eff = ix.resolve_acl("https://pod.ex/notes/n1").expect("own acl");
        assert_eq!(eff.acl.as_str(), "https://pod.ex/notes/n1.acl");
        assert_eq!(eff.scope, AclScope::AccessTo);
    }

    #[test]
    fn resolve_no_acl_is_none() {
        // A pod with NO control document anywhere.
        let nquads = "<https://pod.ex/d#it> <https://ex.dev/ns#k> \"v\" <https://pod.ex/d> .\n";
        let g = graph(nquads);
        let ix = AclIndex::build(&g);
        assert!(ix.resolve_acl("https://pod.ex/d").is_none());
    }

    #[test]
    fn resolve_walks_nearest_ancestor_not_root() {
        // Both a deep container AND the root have ACLs → the NEAREST (deep) one governs.
        let nquads = format!(
            "{ROOT_ACL}\
             <https://pod.ex/notes/.acl#a> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/notes/.acl> .\n"
        );
        let g = graph(&nquads);
        let ix = AclIndex::build(&g);
        let eff = ix
            .resolve_acl("https://pod.ex/notes/n1")
            .expect("nearest acl");
        assert_eq!(
            eff.acl.as_str(),
            "https://pod.ex/notes/.acl",
            "nearest ancestor, not root"
        );
        assert_eq!(eff.scope, AclScope::Default);
    }

    #[test]
    fn unmaterialized_with_acl_is_unloaded_deny() {
        // The view was never materialized: governing ACL is discovered, but we cannot
        // decide → Unloaded, deny, retryable. FR-6 present-but-unloaded path.
        let g = graph(ROOT_ACL);
        let ix = AclIndex::build(&g);
        let empty_auth = AuthIndex::default();
        let alice = Session {
            agent: Some("https://alice.ex/card#me"),
            client: None,
            issuer: None,
            now: None,
        };
        let d = decide_one(
            &ix,
            &empty_auth,
            &alice,
            "https://pod.ex/notes/n1",
            Mode::Read,
        );
        assert!(!d.allow, "fail-closed");
        assert_eq!(d.status, AclStatus::Unloaded);
        assert!(d.status.is_retryable());
        assert!(
            d.governing_acl.is_some(),
            "ACL still discovered for the Link: rel=acl surface"
        );
    }

    #[test]
    fn malformed_resource_iri_is_transient_deny() {
        let g = graph(ROOT_ACL);
        let ix = AclIndex::build(&g);
        let auth = AuthIndex::default();
        let alice = Session {
            agent: Some("https://alice.ex/card#me"),
            client: None,
            issuer: None,
            now: None,
        };
        let d = decide_one(&ix, &auth, &alice, "not a valid iri", Mode::Read);
        assert!(!d.allow);
        assert_eq!(d.status, AclStatus::Transient);
        assert!(d.status.is_retryable());
        assert!(d.governing_acl.is_none());
    }

    #[test]
    fn acl_scope_predicate_iris() {
        // FR-5: the scope maps to the exact WAC predicate term for structured logs.
        assert_eq!(
            AclScope::AccessTo.as_acl_predicate(),
            "http://www.w3.org/ns/auth/acl#accessTo"
        );
        assert_eq!(
            AclScope::Default.as_acl_predicate(),
            "http://www.w3.org/ns/auth/acl#default"
        );
    }

    #[test]
    fn acl_link_header_built_from_governing_acl() {
        // FR-5: the resolved decision advertises the governing ACL as a rel=acl Link value.
        let g = graph(ROOT_ACL);
        let ix = AclIndex::build(&g);
        let auth = AuthIndex::default(); // unmaterialized → Unloaded, but ACL still discovered
        let alice = Session {
            agent: Some("https://alice.ex/card#me"),
            client: None,
            issuer: None,
            now: None,
        };
        let d = decide_one(&ix, &auth, &alice, "https://pod.ex/notes/n1", Mode::Read);
        // Unloaded still discovers the ACL, so the link surfaces (the resource server can
        // still tell the client WHERE the ACL is even on a retryable 503).
        assert_eq!(d.status, AclStatus::Unloaded);
        assert_eq!(
            d.acl_link_header().as_deref(),
            Some(r#"<https://pod.ex/.acl>; rel="acl""#)
        );
    }

    #[test]
    fn acl_link_header_none_when_no_governing_acl() {
        // FR-5 fail-closed: NoAcl and Transient have no governing ACL ⇒ no link to emit.
        let nquads = "<https://pod.ex/d#it> <https://ex.dev/ns#k> \"v\" <https://pod.ex/d> .\n";
        let g = graph(nquads);
        let ix = AclIndex::build(&g);
        let auth = AuthIndex::default();
        let alice = Session {
            agent: Some("https://alice.ex/card#me"),
            client: None,
            issuer: None,
            now: None,
        };
        let no_acl = decide_one(&ix, &auth, &alice, "https://pod.ex/d", Mode::Read);
        assert_eq!(no_acl.status, AclStatus::NoAcl);
        assert!(no_acl.acl_link_header().is_none());

        let transient = decide_one(&ix, &auth, &alice, "not a valid iri", Mode::Read);
        assert_eq!(transient.status, AclStatus::Transient);
        assert!(transient.acl_link_header().is_none());
    }

    #[test]
    fn no_acl_is_definitive_deny_not_retryable() {
        let nquads = "<https://pod.ex/d#it> <https://ex.dev/ns#k> \"v\" <https://pod.ex/d> .\n";
        let g = graph(nquads);
        let ix = AclIndex::build(&g);
        let auth = AuthIndex::default();
        let alice = Session {
            agent: Some("https://alice.ex/card#me"),
            client: None,
            issuer: None,
            now: None,
        };
        let d = decide_one(&ix, &auth, &alice, "https://pod.ex/d", Mode::Read);
        assert!(!d.allow);
        assert_eq!(d.status, AclStatus::NoAcl);
        assert!(
            !d.status.is_retryable(),
            "absent ACL is a definitive (403) deny, not a retry"
        );
    }
}
