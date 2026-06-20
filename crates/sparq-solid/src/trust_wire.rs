//! Trust-graph admission → `<urn:sparq:auth>` wiring (the `trust-graph` feature).
//!
//! This is the ONE edge `sparq-solid` gains when the default-OFF `trust-graph`
//! feature is enabled (design `research/solid-trust-graph-authz-design.md` §6.0; epic
//! `sq-pfae`, issue #940). With the feature OFF this module is not compiled at all, so
//! `sparq-solid` behaves **byte-identically** to WAC/ACP today — the strict-additivity
//! property (G6, §2.2).
//!
//! With the feature on, [`PodStore::admit_trust_credential_and_materialize`] runs the
//! `sparq-trust` admission gate over a presented credential, derives the `auth:*`
//! grants by merging the admitted (issuer-tagged) facts with the controller-authored
//! `.acr` ABAC rule via the shipped N3 reasoner, and installs those grants into
//! `<urn:sparq:auth>` **on top of** the unchanged WAC/ACP view — exactly the
//! ODRL-bridge grant-injection precedent. Everything downstream (the session-scoped
//! `∪ allow ∖ ∪ deny` enforcement, the query rewrite) is the existing shipped code.
//!
//! ## Honest scope (load-bearing — RESEARCH, NOT a security guarantee)
//!
//! The gate verifies a CHECKED issuer signature, but it does **NOT** deliver privacy,
//! unlinkability, or anonymity: the credential is admitted in the clear and the holder
//! binding authenticates the WebID in the clear (the non-anonymous degraded path,
//! `sq-wvne` / `sq-xc4y`). The ZK estate it composes with is externally **UNAUDITED**
//! (`sq-qhy4`). Admission MUST be re-run per request for credential-gated resources
//! (`sq-xc4y`) — calling this once does not freeze a per-request decision into the
//! materialise-once view. Full scope: the `sparq-trust` crate docs + README.
//!
//! [OPUS-4.8] sq-pfae PoC. 🤖 SPARQ agent — trust-graph authorisation PoC.

use crate::{PodStore, AUTH_GRAPH};
use oxrdf::{NamedNode, Term, Triple};
use sparq_core::Graph;
use sparq_trust::admit::{admit, AdmittedFact, PresentedCredential, Session as TrustSession};
use sparq_trust::policy::TrustRule;
use sparq_trust::wire::derive_grants;

/// What the trust-graph admission pass decided and installed.
#[derive(Debug, Clone, Default)]
pub struct TrustAdmissionOutcome {
    /// The issuer-tagged facts that passed the admission gate (possibly empty —
    /// default-deny). Each is marked `trust:admitted` in the design's terms.
    pub admitted: Vec<AdmittedFact>,
    /// The `auth:*` grant triples derived by merging the admitted facts with the
    /// `.acr` ABAC rule and installed into `<urn:sparq:auth>` (possibly empty).
    pub installed_grants: Vec<Triple>,
}

impl PodStore {
    /// Run the trust-graph admission gate over a presented `credential`, merge the
    /// admitted facts with the controller-authored `.acr` `abac_rule_n3` via the
    /// shipped N3 reasoner, and install the derived `auth:*` grants into
    /// `<urn:sparq:auth>` on top of the current (WAC/ACP) view, then reindex so the
    /// grant takes effect on the next [`PodStore::accessible`] / [`PodStore::query_as`].
    ///
    /// Call a static `materialize_*` FIRST if you also want the WAC/ACP grants present;
    /// this adds to whatever auth view currently exists. The grants are **appended** —
    /// a trust-graph grant can never drop a WAC/ACP grant (it only unions allows).
    ///
    /// `rules` is the parsed, **Control-gated** trust policy (see
    /// [`sparq_trust::parse_policy`]); `session` is the **per-request** requester
    /// context (agent + `now`); `target` is the requested resource (the `trust:scope`
    /// check). This is a **per-request** decision and MUST be re-run per request for a
    /// credential-gated resource (`sq-xc4y`) — it is NOT a materialise-once operation.
    ///
    /// # Honest scope
    ///
    /// RESEARCH prototype, NOT a security guarantee. No privacy / unlinkability /
    /// anonymity; the holder binding is clear-WebID (`sq-wvne`); the ZK estate is
    /// externally unaudited (`sq-qhy4`). See the module + `sparq-trust` docs.
    ///
    /// # Errors
    ///
    /// Returns `Err` only if the N3 merge of the admitted facts with `abac_rule_n3`
    /// fails to reason (a malformed rule). Admission itself never errors: a credential
    /// that fails any gate simply admits nothing (fail-closed, empty outcome).
    pub fn admit_trust_credential_and_materialize(
        &mut self,
        credential: &PresentedCredential,
        rules: &[TrustRule],
        session: &TrustSession,
        target: &NamedNode,
    ) -> Result<TrustAdmissionOutcome, String> {
        let admitted = admit(credential, rules, session, target);
        let grants = derive_grants(&admitted, abac_rule_n3_for(self))?;
        if !grants.is_empty() {
            install_auth_grants(&mut self.graph, &grants);
            self.reindex();
        }
        Ok(TrustAdmissionOutcome {
            admitted,
            installed_grants: grants,
        })
    }

    /// As [`PodStore::admit_trust_credential_and_materialize`] but with an explicit
    /// `abac_rule_n3` (the controller-authored `.acr` ABAC rule, e.g.
    /// `{ ?x schema:age ?y . ?y math:greaterThan 18 } => { ?x auth:read <r> }`),
    /// rather than reading it from the store. This is the v1 PoC entry point: the rule
    /// is the trusted root carried in the Control-gated `.acr` channel.
    pub fn admit_trust_credential_with_rule(
        &mut self,
        credential: &PresentedCredential,
        rules: &[TrustRule],
        session: &TrustSession,
        target: &NamedNode,
        abac_rule_n3: &str,
    ) -> Result<TrustAdmissionOutcome, String> {
        let admitted = admit(credential, rules, session, target);
        let grants = derive_grants(&admitted, abac_rule_n3)?;
        if !grants.is_empty() {
            install_auth_grants(&mut self.graph, &grants);
            self.reindex();
        }
        Ok(TrustAdmissionOutcome {
            admitted,
            installed_grants: grants,
        })
    }
}

/// v1 PoC: there is no in-store ABAC-rule discovery yet (that is P4 — trust-document
/// storage). The default rule-less merge derives nothing; callers pass the `.acr`
/// rule explicitly via [`PodStore::admit_trust_credential_with_rule`]. Kept as a seam
/// so a future P4 can resolve the rule from the `.acr` graph.
fn abac_rule_n3_for(_store: &PodStore) -> &'static str {
    ""
}

/// Append `auth:*` grant triples into `<urn:sparq:auth>` (creating it if absent),
/// de-duplicated — the same install discipline the ODRL bridge uses, kept local to
/// the `trust-graph` feature so it composes with the unchanged materialiser without
/// touching the ODRL path.
fn install_auth_grants(graph: &mut Graph, grants: &[Triple]) {
    let g_name = Term::NamedNode(NamedNode::new_unchecked(AUTH_GRAPH));
    let mut terms: Vec<[Term; 3]> = match graph.named.iter().find(|(n, _)| *n == g_name) {
        Some((_, sub)) => crate::loader::graph_triples(sub),
        None => Vec::new(),
    };
    for t in grants {
        let subj = match &t.subject {
            oxrdf::NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n.clone()),
            oxrdf::NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b.clone()),
        };
        let triple = [subj, Term::NamedNode(t.predicate.clone()), t.object.clone()];
        if !terms.contains(&triple) {
            terms.push(triple);
        }
    }
    // Re-intern into a fresh sub-graph dictionary (mirrors install_auth_view).
    let mut dict = sparq_core::dict::Dict::new();
    let ids: Vec<[sparq_core::dict::Id; 3]> = terms
        .iter()
        .map(|t| [dict.intern(&t[0]), dict.intern(&t[1]), dict.intern(&t[2])])
        .collect();
    let sub = Graph::from_parts(dict, ids);
    if let Some(slot) = graph.named.iter_mut().find(|(n, _)| *n == g_name) {
        slot.1 = sub;
    } else {
        graph.named.push((g_name, sub));
    }
}
