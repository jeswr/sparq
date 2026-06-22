//! # `admit.rs` — the admission gate (default-deny, short-circuit)
//!
//! For a presented credential graph `G`, the parsed `Vec<TrustRule>`, and the live
//! [`Session`], emit the `Vec<AdmittedFact>` that pass admission. The algorithm is
//! the design's §6.0 pseudocode, run exactly (short-circuit on first failure;
//! default-deny):
//!
//! ```text
//! admit(G, rules, session) -> admitted:
//!   cG := canonicalise(G)                              # sparq-canon RDFC-1.0
//!   for r in rules:
//!     if not scope_covers(r.scope, target):     continue
//!     if not verify_sig(cG.commitment, r.key):  continue   # CHECKED issuer sig
//!     if session.now - issued_at(G) > r.fresh:  continue   # per-request Rust check
//!     if revoked(G):                            continue   # input-stratified guard
//!     for t=(s,p,o) in G:
//!       if is_reserved(p):                      continue   # solidx:/urn:sparq: guard
//!       if not shape_admits(r.shape, t, G):     continue   # sparq-shacl
//!       if subject_of(t) != session.agent:      continue   # §3.4 holder binding
//!       emit AdmittedFact{ t, issuer: r.source, mark: trust:admitted }
//! ```
//!
//! `verify_sig`, `revoked`, freshness, and holder-binding are **Rust side-conditions**
//! (per-request, NOT in-reasoner — §3.3 B′); `shape_admits` runs the shipped,
//! terminating `sparq-shacl` validator; `scope_covers` is a containment check.
//!
//! ## The static / dynamic admission split (the `sq-xc4y` resolution)
//!
//! Admission has TWO classes of condition with DIFFERENT lifetimes (design §3.3
//! ADMISSION-VS-MATERIALIZE-ONCE GAP):
//!
//! - **STATIC** — the issuer signature over the RDFC-1.0 commitment, the
//!   statement-type scope (SHACL shape), the reserved-predicate guard, and the
//!   `trust:scope` containment. These depend ONLY on the credential + the policy, NOT
//!   on the request. They are session-independent and can be decided **once, at
//!   materialise-time**, exactly like the WAC/ACP derivation stratum.
//! - **DYNAMIC** — the holder binding (`credentialSubject == Session.agent`) and
//!   freshness (`Session.now - issued_at ≤ freshWithin`). These are **per-request**
//!   facts. They MUST be re-evaluated on every request and must NOT be frozen into the
//!   session-independent materialise-once view.
//!
//! [`admit`] runs BOTH classes against a single live [`Session`] (the snapshot path:
//! the decision is valid for *that* request only). [`admit_static`] runs ONLY the
//! static class and returns each fact carrying the dynamic conditions it still owes
//! ([`StaticAdmittedFact::holder`] and [`StaticAdmittedFact::not_after_unix_secs`]) so
//! the caller can install a **conditional grant** whose holder/freshness are re-checked
//! per request at query time — the shipped sq-0q7n `auth:notBefore`/`auth:notAfter`
//! conditional-grant precedent (`sparq-solid` `AuthIndex::cond_applies`). This is the
//! `sq-xc4y` decision **(a) split static-admission from dynamic-admission**, NOT
//! **(b) per-request re-materialise**: the static stratum composes with the epoch
//! cache, the dynamic conditions ride the existing per-request conditional-grant path.
//!
//! ## Open-problem hooks this gate RESPECTS (does not silently solve)
//!
//! - **`sq-xc4y`** (holder-binding/freshness are per-request) — RESOLVED by the
//!   static/dynamic split above: [`admit_static`] emits the static decision once and
//!   defers holder/freshness to the per-request conditional-grant check, so a stale or
//!   wrong-holder credential is denied at query time WITHOUT a re-materialise and is
//!   never frozen into the materialise-once view. The combined [`admit`] remains the
//!   per-request snapshot path for a single request.
//! - **`sq-wvne` / `sq-xc4y` (holder binding)** — the holder binding is the
//!   **clear-WebID v1 path** (`credentialSubject == Session.agent`), the
//!   **non-anonymous degraded path**. It authenticates the requester's WebID in the
//!   CLEAR; it does **NOT** deliver unlinkable/anonymous presentation. This is the
//!   documented degradation, not a solved problem.
//! - **`sq-tu4e`** — `revoked` is an **input-only** seeded predicate (a per-request
//!   Rust check here), never in-reasoner negation over a derived fact; there is **no
//!   deny-on-disagreement** rule in this PoC.
//!
//! ## Honest scope
//!
//! This gate verifies a CHECKED issuer signature; it does **NOT** provide privacy,
//! unlinkability, anonymity, or any cryptographic guarantee. The ZK estate it
//! composes with is research-grade and **externally UNAUDITED** (`sq-qhy4`). The issuer
//! key is operator-asserted by default (the `issuerKey → verifying-key` binding is the
//! live forgery vector D′ of §3.3); the opt-in `did` feature (`sq-pfae.3`, the `did`
//! module) can bind it from a `trust:issuerDid` instead, narrowing — not eliminating — D′.
//!
//! [OPUS-4.8] sq-pfae PoC (issue #940). 🤖 SPARQ agent — trust-graph authorisation PoC.

use crate::policy::{ShapeRef, TrustRule};
use crate::vocab;
use oxrdf::{NamedNode, NamedOrBlankNode, Term, Triple};
use sparq_zk::commit::commit_triples;
use sparq_zk::encode::salt_from_bytes;
use sparq_zk::field::Fr;
use sparq_zk::sig::{commitment_message, signature_from_hex, verify};

/// The live request context the admission gate binds against (§3.3 / §3.4). This is a
/// **per-request** value — the gate MUST be re-evaluated per request (`sq-xc4y`), not
/// frozen into a session-independent materialise-once view.
#[derive(Debug, Clone)]
pub struct Session {
    /// The authenticated requester's WebID (`Session.agent`). The holder binding
    /// requires the credential subject to equal this (§3.4) — the **clear-WebID**,
    /// **non-anonymous** v1 path (`sq-wvne`).
    pub agent: NamedNode,
    /// The current instant as a Unix timestamp (seconds), consulted against
    /// `trust:freshWithin` (§3.3 B′). A per-request value, never frozen.
    pub now_unix_secs: i64,
}

/// A presented credential: the claim graph `G`, the issuer signature over its
/// RDFC-1.0 commitment, the per-graph salt the commitment was minted under, the
/// issuance instant, and the revocation flag.
///
/// This is the verifiable unit — the same signed-graph unit the ZK estate already
/// produces (`sparq-zk` commits per-named-graph over RDFC-1.0-canonicalised leaves).
/// The signature is over the **checked** RDFC-1.0 commitment, NOT a self-asserted "I
/// am signed" triple (the load-bearing soundness condition, §3.3 item 1).
#[derive(Debug, Clone)]
pub struct PresentedCredential {
    /// The credential claim graph `G` (e.g. `<Jesse> schema:age 25`).
    pub graph: Vec<Triple>,
    /// The issuer Schnorr signature over `commitment_message(C(G))`, hex-encoded
    /// (`compressed(R) ‖ s`), exactly as `sparq_zk::sig::SecretKey::sign_commitment`
    /// produces. An absent/forged signature fails closed.
    pub issuer_signature_hex: String,
    /// The 32-byte per-graph salt the RDFC-1.0 commitment was minted under
    /// (`zk:rdfc10Salt`). The verifier re-commits under the same salt; a mismatched
    /// salt yields a different `C(G)` and fails the signature check.
    pub salt: [u8; 32],
    /// Issuance instant as a Unix timestamp (seconds). Compared against
    /// `Session.now_unix_secs` and the rule's `fresh_within_secs`.
    pub issued_at_unix_secs: i64,
    /// Per-request revocation flag (the **input-only** guard — `sq-tu4e`). A
    /// production wiring would consult a W3C Bitstring Status List (P6); the PoC
    /// takes the boolean directly so the fail-closed path is testable.
    pub revoked: bool,
}

/// A fact that passed admission, issuer-tagged and marked `trust:admitted` (§2.1
/// stratum boundary). It enters the **unchanged** `sparq-solid` materialiser ahead of
/// the derivation stratum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedFact {
    /// The admitted triple (e.g. `<Jesse> schema:age 25`).
    pub triple: Triple,
    /// `trust:source` — the issuer this fact is tagged with.
    pub issuer: NamedNode,
}

/// A fact that passed the **STATIC** admission class (signature + statement-type scope +
/// reserved-predicate guard + `trust:scope`), carrying the **DYNAMIC** conditions it
/// still owes so the caller can defer them to a per-request check (`sq-xc4y`).
///
/// The static decision is session-independent — it depends only on the credential and
/// the policy — so it can be made **once, at materialise-time**. The dynamic conditions
/// ([`holder`](Self::holder) and [`not_after_unix_secs`](Self::not_after_unix_secs))
/// MUST NOT be frozen into the materialise-once view; they ride the shipped sq-0q7n
/// conditional-grant path (`auth:agent` re-checked against `Session.agent`,
/// `auth:notAfter` re-checked against `Session.now`) and are re-evaluated per request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticAdmittedFact {
    /// The admitted triple (e.g. `<Jesse> schema:age 25`).
    pub triple: Triple,
    /// `trust:source` — the issuer this fact is tagged with.
    pub issuer: NamedNode,
    /// The bound holder: the credential subject the fact may only be used on behalf of.
    /// The per-request holder binding is `holder == Session.agent`, deferred to query
    /// time (NEVER frozen into the view — `sq-xc4y` / `sq-wvne` clear-WebID path).
    pub holder: NamedNode,
    /// The freshness deadline as a Unix timestamp (seconds): `issued_at + freshWithin`.
    /// The per-request freshness check is `Session.now ≤ not_after_unix_secs`, deferred
    /// to query time so a credential that lapses denies *that request* without a
    /// re-materialise (`sq-xc4y`; the sq-0q7n `auth:notAfter` precedent).
    pub not_after_unix_secs: i64,
}

/// Run the admission gate over a presented credential. Returns the admitted facts
/// (possibly empty — default-deny). NEVER panics on adversarial input: a malformed
/// signature, an uncanonicalisable graph, or a bad shape all fail closed (no admit).
///
/// `target` is the resource the request is for (the `trust:scope` containment check).
pub fn admit(
    cred: &PresentedCredential,
    rules: &[TrustRule],
    session: &Session,
    target: &NamedNode,
) -> Vec<AdmittedFact> {
    let mut admitted: Vec<AdmittedFact> = Vec::new();

    // Step 1: canonicalise + commit G (sparq-canon RDFC-1.0, the same canonical unit
    // the ZK estate commits over). A graph that cannot be committed admits nothing.
    let salt = salt_from_bytes(&cred.salt);
    let Ok(commitment) = commit_triples(&cred.graph, salt) else {
        return admitted; // fail closed
    };
    let commitment_fr: Fr = commitment.commitment;

    // Parse the issuer signature once (fail closed on malformed hex).
    let Some(signature) = signature_from_hex(&cred.issuer_signature_hex) else {
        return admitted;
    };

    for r in rules {
        // §3.2 scope: the rule must cover the target resource.
        if !scope_covers(&r.scope, target) {
            continue;
        }
        // §3.3 (1): the CHECKED issuer signature over the RDFC-1.0 commitment. A
        // self-asserted "I am signed" triple proves nothing; this verifies the
        // signature against the key the rule names.
        let msg = commitment_message(&commitment_fr);
        if !verify(&r.issuer_key, &msg, &signature) {
            continue;
        }
        // §3.3 (B′) freshness — a per-request Rust check, NOT an in-reasoner predicate.
        let age = session
            .now_unix_secs
            .saturating_sub(cred.issued_at_unix_secs);
        if age < 0 || age > r.fresh_within_secs {
            continue;
        }
        // §3.3 input-stratified revocation guard (input-only; never NAF over a
        // derived predicate — sq-tu4e).
        if cred.revoked {
            continue;
        }
        // Pre-build the shape data once per rule.
        for t in &cred.graph {
            // The reserved-derivation-predicate guard stays in force UNDER admission:
            // a source trusted for schema:age cannot launder a solidx:/urn:sparq:/acl
            // internal triple in (§3.3 item 2). This mirrors sparq-solid's
            // `is_reserved_derivation_predicate` / `validate_principal_iri` guards.
            if is_reserved_predicate(&t.predicate) {
                continue;
            }
            // §2.3.2 statement-type scoping via the SHACL shape (forShape /
            // forPredicate-sugar). A triple whose subject the shape does not target,
            // or that violates the shape, is NOT of the trusted statement-type.
            if !shape_admits(&r.shape, t, &cred.graph) {
                continue;
            }
            // §3.4 holder binding: the credential subject must equal the authenticated
            // requester (the clear-WebID, non-anonymous v1 path — sq-wvne). Presenting
            // a third party's credential without holder binding must NOT admit.
            if !subject_is(t, &session.agent) {
                continue;
            }
            let fact = AdmittedFact {
                triple: t.clone(),
                issuer: r.source.clone(),
            };
            if !admitted.contains(&fact) {
                admitted.push(fact);
            }
        }
    }
    admitted
}

/// Run ONLY the **static** admission class over a presented credential, returning the
/// statically-admitted facts each tagged with the **dynamic** conditions it still owes
/// (the bound holder + the freshness deadline). This is the materialise-time half of
/// the `sq-xc4y` static/dynamic split: it decides everything that is session-
/// independent (signature, statement-type scope, reserved-predicate guard, `trust:scope`)
/// and DEFERS holder-binding + freshness to a per-request conditional-grant check.
///
/// Unlike [`admit`], this takes **no `Session`**: a static decision must not depend on
/// the request. The credential subject becomes the [`StaticAdmittedFact::holder`] the
/// grant is pinned to; the freshness deadline becomes
/// [`StaticAdmittedFact::not_after_unix_secs`]. The caller installs a conditional grant
/// that re-checks both per request — so a stale or wrong-holder credential is denied at
/// query time WITHOUT a re-materialise and is never frozen into the materialise-once
/// view.
///
/// Returns the statically-admitted facts (possibly empty — default-deny). NEVER panics
/// on adversarial input: a malformed signature, an uncanonicalisable graph, or a bad
/// shape all fail closed (no admit). The static class fails closed exactly as [`admit`]
/// does; only the holder/freshness checks move to query time.
pub fn admit_static(
    cred: &PresentedCredential,
    rules: &[TrustRule],
    target: &NamedNode,
) -> Vec<StaticAdmittedFact> {
    let mut admitted: Vec<StaticAdmittedFact> = Vec::new();

    // Step 1: canonicalise + commit G (the same canonical unit the ZK estate commits
    // over). A graph that cannot be committed admits nothing.
    let salt = salt_from_bytes(&cred.salt);
    let Ok(commitment) = commit_triples(&cred.graph, salt) else {
        return admitted; // fail closed
    };
    let commitment_fr: Fr = commitment.commitment;

    let Some(signature) = signature_from_hex(&cred.issuer_signature_hex) else {
        return admitted;
    };

    for r in rules {
        // §3.2 scope (STATIC: policy + credential only).
        if !scope_covers(&r.scope, target) {
            continue;
        }
        // §3.3 (1) CHECKED issuer signature (STATIC).
        let msg = commitment_message(&commitment_fr);
        if !verify(&r.issuer_key, &msg, &signature) {
            continue;
        }
        // §3.3 input-stratified revocation guard. Revocation is a per-credential input
        // fact (not request-dependent), so a credential KNOWN revoked at materialise is
        // dropped here too; a credential revoked LATER is handled by re-materialise
        // (epoch bump on trust-graph/status change — design §3.3 / G5).
        if cred.revoked {
            continue;
        }
        // The freshness deadline this rule imposes: issued_at + freshWithin. Carried as
        // a per-request DYNAMIC condition (NOT checked here against any `now`).
        let not_after_unix_secs = cred.issued_at_unix_secs.saturating_add(r.fresh_within_secs);

        for t in &cred.graph {
            // The reserved-predicate guard stays in force UNDER admission (STATIC).
            if is_reserved_predicate(&t.predicate) {
                continue;
            }
            // §2.3.2 statement-type scoping via the SHACL shape (STATIC).
            if !shape_admits(&r.shape, t, &cred.graph) {
                continue;
            }
            // The bound holder is the credential subject — pinned into the grant's
            // `auth:agent` head, re-checked against `Session.agent` per request (the
            // DYNAMIC holder binding, deferred — §3.4 / sq-xc4y). A triple whose subject
            // is a blank node cannot be holder-bound (no stable WebID), so it is not
            // statically admissible for a credential-gated grant: fail closed.
            let NamedOrBlankNode::NamedNode(holder) = &t.subject else {
                continue;
            };
            let fact = StaticAdmittedFact {
                triple: t.clone(),
                issuer: r.source.clone(),
                holder: holder.clone(),
                not_after_unix_secs,
            };
            if !admitted.contains(&fact) {
                admitted.push(fact);
            }
        }
    }
    admitted
}

/// `scope_covers(scope, target)` — exact-IRI or ancestor-container containment
/// (Solid slash-semantics). v1 is exact-or-prefix: a rule scoped to a container
/// covers its members; a rule scoped to a resource covers exactly that resource. A
/// rule never broadens beyond its scope (fail-closed direction).
pub fn scope_covers(scope: &NamedNode, target: &NamedNode) -> bool {
    let s = scope.as_str();
    let t = target.as_str();
    if s == t {
        return true;
    }
    // Container scope (slash-terminated) covers any descendant resource.
    s.ends_with('/') && t.starts_with(s)
}

/// Whether a predicate is in the reserved derivation-internal / auth space that no
/// external source may ever assert (the `solidx:` / `urn:sparq:` / `acl:` / `acp:`
/// guard — the analogue of `sparq-solid`'s reserved-predicate guard). A trust rule
/// cannot grant a source the right to launder these in (§3.3 item 2).
fn is_reserved_predicate(p: &NamedNode) -> bool {
    let s = p.as_str();
    s.starts_with("https://sparq.dev/ns/solidx#")
        || s.starts_with("https://sparq.dev/ns/auth#")
        || s.starts_with("urn:sparq:")
        || s.starts_with("http://www.w3.org/ns/auth/acl#")
        || s.starts_with("http://www.w3.org/ns/solid/acp#")
        || s == vocab::ADMITTED
}

/// `shape_admits(shape, t, G)` — does the SHACL shape admit triple `t` in the context
/// of graph `G`? Runs the shipped, terminating `sparq-shacl` validator (the `forShape`
/// primitive; `forPredicate` desugars to it). The triple is admitted iff its SUBJECT
/// is a focus node the shape TARGETS (so an off-type triple's subject is not selected)
/// AND that focus node CONFORMS to the shape.
///
/// For the single-predicate desugaring (`sh:targetSubjectsOf P` + `sh:path P ;
/// sh:minCount 1`), this is exactly "`t`'s predicate is `P`" — so a source trusted for
/// `schema:age` cannot admit an `acl:agent` / arbitrary triple: that triple's subject
/// is not a `targetSubjectsOf schema:age` focus node, so the shape never selects it.
fn shape_admits(shape: &ShapeRef, t: &Triple, graph: &[Triple]) -> bool {
    use sparq_shacl::{graph_from_triples, validate};

    // Only the triple's subject matters for whether the statement-type is trusted; but
    // the shape is validated over the WHOLE credential graph so that a property shape
    // referencing the subject's other triples can see them.
    let data = graph_from_triples(graph.iter().cloned());
    let shapes = graph_from_triples(shape.triples.iter().cloned());
    let report = validate(&data, &shapes);

    // The triple's subject must be selected as a focus node (i.e. the shape TARGETS
    // it). We re-derive the targeted-subject set from the desugared/inline shape: a
    // node is a focus node iff there is a `sh:targetSubjectsOf P` whose `P` the subject
    // asserts. Conformance is then "no violation for that focus node".
    let subj = subject_term(&t.subject);
    if !shape_targets_subject(shape, &subj, graph) {
        return false;
    }
    // Conformance for this focus node: no Violation result on it.
    !report
        .results
        .iter()
        .any(|r| r.focus_node == subj && r.severity.ends_with("#Violation"))
}

/// Whether the shape's `sh:targetSubjectsOf` predicates select `subject` over `graph`
/// — i.e. `subject` asserts at least one of the shape's target predicates. This is the
/// statement-type gate: an off-type triple's subject is not selected.
fn shape_targets_subject(shape: &ShapeRef, subject: &Term, graph: &[Triple]) -> bool {
    let target_preds: Vec<&str> = shape
        .triples
        .iter()
        .filter(|t| t.predicate.as_str() == vocab::SH_TARGET_SUBJECTS_OF)
        .filter_map(|t| match &t.object {
            Term::NamedNode(n) => Some(n.as_str()),
            _ => None,
        })
        .collect();
    if target_preds.is_empty() {
        return false; // a shape with no subjects-of target selects nothing here
    }
    graph.iter().any(|t| {
        subject_term(&t.subject) == *subject && target_preds.contains(&t.predicate.as_str())
    })
}

fn subject_is(t: &Triple, agent: &NamedNode) -> bool {
    matches!(&t.subject, NamedOrBlankNode::NamedNode(n) if n.as_str() == agent.as_str())
}

fn subject_term(s: &NamedOrBlankNode) -> Term {
    match s {
        NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n.clone()),
        NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iri(s: &str) -> NamedNode {
        NamedNode::new(s).unwrap()
    }

    #[test]
    fn scope_covers_exact_and_container() {
        let r = iri("https://pod.ex/docs/x");
        assert!(scope_covers(&r, &iri("https://pod.ex/docs/x")));
        assert!(!scope_covers(&r, &iri("https://pod.ex/docs/y")));
        let c = iri("https://pod.ex/docs/");
        assert!(scope_covers(&c, &iri("https://pod.ex/docs/x")));
        assert!(!scope_covers(&c, &iri("https://pod.ex/other/x")));
    }

    #[test]
    fn reserved_predicate_guard_blocks_internal_vocab() {
        assert!(is_reserved_predicate(&iri(
            "https://sparq.dev/ns/solidx#creator"
        )));
        assert!(is_reserved_predicate(&iri(
            "http://www.w3.org/ns/auth/acl#agent"
        )));
        assert!(is_reserved_predicate(&iri(vocab::ADMITTED)));
        assert!(!is_reserved_predicate(&iri("http://schema.org/age")));
    }
}
