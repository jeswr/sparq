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

// [FABLE-5] sq-sqtk2.1 (epic sq-sqtk2; research/mechanized-proof-program.md §3.1 properties
// A-1 + A-2, §5 bead 1) — Kani bounded-model-checking proofs of the decision layer's
// FAIL-CLOSED structure and of the container-walk termination lemma. `#[cfg(kani)]`:
// compiled only under `cargo kani -p sparq-solid`, stripped from every normal build
// (proof-only diff — the runtime decision logic is byte-unchanged).
//
// CLAIM TIER — **PROVED (bounded)**, per the program's assurance ladder. Each harness
// states its exact bounds; nothing here claims "proved for all inputs". Kani string costs
// make symbolic IRIs/ACL-graphs intractable, so the strategy is: concrete strings from a
// small fixed universe, SYMBOLIC structure (which control docs exist, materialization,
// session shape, grant vector, modes) — complete over that product domain. The
// nearest-ancestor SELECTION semantics over a richer resource/control-doc domain are
// EXHAUSTIVELY TESTED (bounded domain) in `tests/container_walk_exhaustive.rs`; full
// WAC/ACP spec parity stays with the conformance corpora + the differential oracle (A-4).
#[cfg(kani)]
mod kani_proofs {
    use super::*;
    use crate::authindex::kani_support::index_with_grants;
    use crate::authindex::PUBLIC;

    // The concrete bounded universe. RES_NESTED's candidate governors, nearest-first:
    // its own OWN_ACL (accessTo), then MID_ACL (/a/), then ROOT_ACL (/). RES_TOP's only
    // candidate governor is ROOT_ACL. RES_BAD is not an IRI (typed-transient path).
    const ROOT_ACL: &str = "https://pod.ex/.acl";
    const MID_ACL: &str = "https://pod.ex/a/.acl";
    const OWN_ACL: &str = "https://pod.ex/a/doc.acl";
    const RES_NESTED: &str = "https://pod.ex/a/doc";
    const RES_TOP: &str = "https://pod.ex/other";
    const RES_BAD: &str = "not a valid iri";
    /// Short WebID for the harness session — 5 bytes avoids memcmp iterations near the
    /// unwind bound. The logical role (a concrete authenticated agent) is unchanged.
    /// [SONNET-4.6] sq-sqtk2.1 string-shortening for solver tractability.
    const AGENT: &str = "alice";
    /// A short reserved-principal encoding — must start with "urn:sparq:" so the
    /// impersonation guard fires, but need not be longer (11 bytes fits unwind(40)).
    /// [SONNET-4.6] sq-sqtk2.1 string-shortening.
    const RESERVED_AGENT: &str = "urn:sparq:x";

    fn any_mode() -> Mode {
        match kani::any::<u8>() % 4 {
            0 => Mode::Read,
            1 => Mode::Write,
            2 => Mode::Append,
            _ => Mode::Control,
        }
    }

    /// PROPERTY (A-1, FAIL-CLOSED): over the WHOLE bounded domain below, every
    /// `decide_one` outcome satisfies
    ///   • `status != Resolved  ⇒  allow == false && granted_modes is empty`;
    ///   • on `Resolved`, `allow == granted_modes.contains(mode)` — so `allow == true`
    ///     is impossible without `Resolved` + the mode actually held;
    ///   • provenance coherence: `scope` is `Some` iff `governing_acl` is `Some`;
    ///     `Resolved`/`Unloaded` carry the discovered ACL, `NoAcl`/`Transient` carry none;
    ///   • a reserved-encoding session (pair-principal impersonation) is granted NOTHING.
    ///
    /// BOUNDS (complete product domain, symbolic): resource ∈ {RES_NESTED, RES_TOP,
    /// RES_BAD} × control-doc universe = all 8 subsets of {OWN_ACL, MID_ACL, ROOT_ACL} ×
    /// materialized ∈ {t, f} × session ∈ {anonymous, alice, reserved-impostor} (client/
    /// issuer/now = None) × auth ∈ {empty, one alice allow-grant, alice allow-grant +
    /// public deny-grant on the same graph} with a symbolic grant mode × all 4 query
    /// modes. NOT covered here: arbitrary IRIs/sessions/ACL contents — see the module
    /// header for where those live.
    ///
    /// UNWIND BOUND — `#[kani::unwind(40)]` (increased from 24 to 40 by [SONNET-4.6]
    /// 2026-07-04). The grant index for `auth_kind = 2` includes a `PUBLIC` principal
    /// entry (key = "https://sparq.dev/ns/auth#Public", 32 bytes). The session principal
    /// list built by `accessible()` also includes PUBLIC (32B) and AUTHENTICATED (39B).
    /// With unwind(24) those self-comparisons were pruned at byte 24, making the PUBLIC
    /// deny-grant invisible and leaving the deny-wins check vacuous. unwind(40) ensures all
    /// 39-byte AUTHENTICATED comparisons complete. Harness-controlled strings are kept short
    /// (AGENT = "alice" 5B, RESERVED_AGENT = "urn:sparq:x" 11B) to reduce comparison cost;
    /// IRI constants (ROOT_ACL 19B, MID_ACL 21B, OWN_ACL 24B, RES_* 20B) all fit within
    /// unwind(40). The `resolve_acl` loop iterates ≤3 times; FxHashSet probing on a
    /// ≤3-entry set is ≤8 probes; 40 covers all in-bounds loops with a margin of ≥1.
    // [SONNET-4.6] unwind increase for soundness + string-shortening for tractability.
    #[kani::proof]
    #[kani::unwind(40)]
    fn decide_one_is_fail_closed() {
        let mut control = rustc_hash::FxHashSet::default();
        if kani::any() {
            control.insert(OWN_ACL.to_owned());
        }
        if kani::any() {
            control.insert(MID_ACL.to_owned());
        }
        if kani::any() {
            control.insert(ROOT_ACL.to_owned());
        }
        let index = AclIndex { control, materialized: kani::any() };

        let resource = match kani::any::<u8>() % 3 {
            0 => RES_NESTED,
            1 => RES_TOP,
            _ => RES_BAD,
        };
        let session = match kani::any::<u8>() % 3 {
            0 => Session::default(),
            1 => Session { agent: Some(AGENT), client: None, issuer: None, now: None },
            _ => Session { agent: Some(RESERVED_AGENT), client: None, issuer: None, now: None },
        };
        let grant_mode = any_mode();
        let auth_kind = kani::any::<u8>() % 3;
        let auth = match auth_kind {
            0 => AuthIndex::default(),
            1 => index_with_grants(&[(AGENT, grant_mode, true, RES_NESTED)]),
            _ => index_with_grants(&[
                (AGENT, grant_mode, true, RES_NESTED),
                (PUBLIC, grant_mode, false, RES_NESTED),
            ]),
        };

        let mode = any_mode();
        let d = decide_one(&index, &auth, &session, resource, mode);

        // The fail-closed core: uncertainty ⇒ deny with nothing granted.
        if d.status != AclStatus::Resolved {
            assert!(!d.allow);
            assert!(d.granted_modes.is_empty());
        }
        // The authoritative arm: allow is exactly "the mode is held" — never wider.
        if d.status == AclStatus::Resolved {
            assert!(d.allow == d.granted_modes.contains(&mode));
        }
        // Provenance coherence (the doc contract on WacDecision).
        assert!(d.scope.is_some() == d.governing_acl.is_some());
        if matches!(d.status, AclStatus::Resolved | AclStatus::Unloaded) {
            assert!(d.governing_acl.is_some());
        } else {
            assert!(d.governing_acl.is_none());
        }
        // Reserved-encoding impersonation fails closed to zero grants.
        if session.agent == Some(RESERVED_AGENT) {
            assert!(d.granted_modes.is_empty());
        }
        // Deny-wins at the decision level: the public deny covers the alice allow, so
        // the allow+deny auth state can never grant (alice's principals include Public).
        if auth_kind == 2 {
            assert!(d.granted_modes.is_empty());
        }

        // Non-vacuity: every status — and a REAL allow — is reachable in-bounds.
        kani::cover!(d.allow);
        kani::cover!(d.status == AclStatus::Resolved && !d.allow);
        kani::cover!(d.status == AclStatus::Unloaded);
        kani::cover!(d.status == AclStatus::NoAcl);
        kani::cover!(d.status == AclStatus::Transient);
        kani::cover!(d.scope == Some(AclScope::AccessTo));
        kani::cover!(d.scope == Some(AclScope::Default));
    }

    /// PROPERTY (A-2, termination lemma): for EVERY ASCII string of length ≤ `MAX_LEN`,
    /// `parent_iri` either returns `None` or a STRICT PREFIX that is strictly shorter
    /// and ends in `/` — and never panics. Since the parent is itself an ASCII string
    /// of length < `MAX_LEN`, the domain is closed under the step: by induction, the
    /// `resolve_acl` container walk (`while let Some(parent) = parent_iri(cur)`)
    /// terminates in at most `MAX_LEN` steps for every input in the domain.
    ///
    /// BOUNDS (honest) — `MAX_LEN = 8`, `#[kani::unwind(16)]` [SONNET-4.6] 2026-07-04:
    ///   all ASCII byte strings of length ≤ 8 over the range 1..=127. Tightened from the
    ///   original Fable-designed MAX_LEN=24/unwind(32) after confirming that both the
    ///   `vec![0u8; len]` symbolic-heap version AND the fixed-`[u8; 24]` version caused
    ///   CBMC exit-144 (killed at `run_utf8_validation` iteration 12–14 on an arm64
    ///   machine already under heavy CBMC load). Root cause: Rust's `str::find("://")`
    ///   uses `TwoWaySearcher` whose internal state machine scales with string length.
    ///   With MAX_LEN=8 the minimum exercisable IRI ("a://b/" = 6 bytes) and all
    ///   None-paths are fully covered; the structural invariant — `rfind('/')` returns a
    ///   position strictly before the trimmed end, so `cut+1 < len` — is length-independent
    ///   and holds for longer IRIs by the same argument. Longer/non-ASCII IRIs are outside
    ///   this proof's bounded domain; the exhaustive walk test covers multi-segment IRIs.
    #[kani::proof]
    #[kani::unwind(16)]
    fn parent_iri_strictly_shortens() {
        const MAX_LEN: usize = 8;
        // Fixed-size stack array avoids the symbolic-heap allocation that caused
        // `run_utf8_validation` path explosion at larger sizes. [SONNET-4.6]
        let bytes: [u8; MAX_LEN] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= MAX_LEN);
        // Constrain ALL 8 bytes to ASCII (1..=127) — over-constraining the unused tail
        // is SOUND (shrinks input space, never widens it). Constant-size loop (8
        // iterations) that CBMC unrolls statically; no symbolic-length iteration.
        for b in bytes.iter() {
            kani::assume(*b >= 1 && *b <= 127); // ASCII ⇒ valid single-byte UTF-8
        }
        // `from_utf8` on a 8-byte buffer: all bytes ASCII, validation loop ≤ 8
        // iterations; Err is unreachable under the ASCII assume constraints.
        let s = match core::str::from_utf8(&bytes[..len]) {
            Ok(s) => s,
            Err(_) => return, // unreachable under the ASCII kani::assume constraints
        };
        let parent = parent_iri(s);
        if let Some(p) = parent {
            assert!(p.len() < s.len());
            assert!(s.as_bytes()[..p.len()] == *p.as_bytes());
            assert!(p.as_bytes().last() == Some(&b'/'));
        }
        kani::cover!(parent.is_some()); // the Some arm is genuinely exercised
    }
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
