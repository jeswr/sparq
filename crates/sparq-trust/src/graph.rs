//! # `graph.rs` — the depth-bounded certification-edge trust-graph closure (fail-closed)
//!
//! The pod-side evaluation the merged `trustx:Certification` vocabulary
//! ([`crate::framework_vocab`]) lacked: [`derive_effective_rules`] runs a **depth-bounded
//! (depth-N)**, **attenuation-only**, **fail-closed** closure over signed
//! certification edges to derive additional [`TrustRule`]s **AHEAD of** — and consumed
//! by — the UNCHANGED admission gate ([`mod@crate::admit`]). It is a pure PRE-PROCESSING
//! step: it produces the `Vec<TrustRule>` admission then consumes verbatim; it does not
//! change the gate.
//!
//! ```text
//!   direct_rules (Control-gated anchors)  +  signed trustx:Certification edges
//!                                │
//!                                ▼
//!   ┌──────────────────────────────────────────────────────────────────┐
//!   │  derive_effective_rules  (this module, NEW, OPT-IN `cert-graph`)   │
//!   │  depth-N · attenuation-only · fail-closed certification closure    │
//!   │  every derived rule ⊆ (anchor ∩ cert scope ∩ validity window)      │
//!   └──────────────────────────────────────────────────────────────────┘
//!                                │  effective_rules == direct_rules ++ derived
//!                                ▼
//!   ADMISSION gate  (crate::admit, UNCHANGED — consumes a flat Vec<TrustRule>)
//! ```
//!
//! ## The certification model (design `research/trust-expression-spec.md` §3.4 / D4)
//!
//! A [`Certification`] is a signed edge: an authority (the **certifier**) attests that
//! another issuer (the **certified issuer**) is certified — under a framework, over a
//! **scope**, within a **validity window** — to issue statements of that scope. A
//! Trusted-List / DIATF-register entry IS one such edge. The closure lets a pod that
//! anchors trust in a *certifier* transitively (up to `depth_bound` hops) admit facts from
//! issuers that certifier vouches for — but ONLY narrowed to what the certifier itself may
//! confer, and, at hop *k*, only to what hop *k−1* itself derived.
//!
//! ## The HARD invariant — fail-closed, attenuation-ONLY (a broadening bug is escalation)
//!
//! This is authorisation-derivation logic: a rule that grants an issuer authority its
//! certifier does not hold is a **privilege escalation**, not a bug to paper over. The
//! closure therefore guarantees:
//!
//! - **Attenuation only, at EVERY depth.** Every derived rule's scope is the *intersection*
//!   `anchor_scope ∩ cert_scope ∩ validity_window` and its statement-type shape is the
//!   *intersection* of the anchor's shape with the cert's — a certification can only
//!   **NARROW**, never **WIDEN**, the certifier's own authority. Where narrowing cannot
//!   be *proven* (undecidable SHACL-shape containment), the edge contributes **NOTHING**
//!   (design §3.4: "undecidable containment ⇒ NOT contained ⇒ contributes nothing").
//!   The ⊆ ceiling is **re-derived per hop** against the rule the previous hop produced —
//!   never against the root anchor — so the chain telescopes:
//!   `derived_N ⊆ derived_{N-1} ⊆ … ⊆ derived_1 ⊆ anchor`. A hop that broadens is denied
//!   *at that hop*, and the tail beyond it is therefore never reached.
//! - **Target-set-CONTRAVARIANT, conformance-COVARIANT shape containment.** SHACL shapes
//!   have two kinds of statement: **selection/target** predicates (`sh:targetSubjectsOf`,
//!   `sh:targetObjectsOf`, `sh:targetNode`, `sh:targetClass`, `sh:targetWhere`, the
//!   implicit-class typing) that pick focus nodes by the **UNION** of targets, and
//!   **conformance** constraints (`sh:property`/`sh:minCount`/`sh:datatype`/…) that a
//!   selected node must satisfy. Adding a conformance constraint NARROWS (fewer nodes
//!   conform); but adding a target predicate WIDENS (more nodes are selected). So a cert
//!   shape narrows the anchor ONLY when its selection set is a **SUBSET** of the anchor's
//!   (targets may only SHRINK) AND its conformance constraints are a **SUPERSET** of the
//!   anchor's (constraints may only GROW). Any un-modelled selection predicate ⇒ FAIL
//!   CLOSED. Treating "more edges ⇒ narrower" uniformly is UNSOUND: an extra
//!   `sh:targetSubjectsOf` edge is a broadening, not a narrowing (`sq-pfae.15` review fix).
//! - **Polarity-checked monotonicity (`sq-wh546` fix).** "Constraints may only GROW ⇒
//!   narrower" is itself sound ONLY for purely-CONJUNCTIVE constraints. SHACL has
//!   NON-MONOTONE components (`sh:not`, `sh:or`, `sh:xone`, `sh:maxCount`,
//!   `sh:qualifiedValueShape`/`sh:qualifiedMaxCount`, …) under which ADDING an edge
//!   BROADENS the admitted set (e.g. an extra `sh:datatype` INSIDE an `sh:not`). The
//!   narrowing gate therefore first requires a monotonicity PROOF for EVERY constraint
//!   predicate in BOTH shape closures (the `MONOTONE_CONFORMANCE_LOCALS` allowlist); any
//!   non-monotone or unknown predicate ⇒ the edge is rejected as
//!   [`EdgeRejection::Broadening`] (unknown ⇒ no proof ⇒ fail closed).
//! - **A CHECKED signature, never a self-asserted one.** The edge is admitted only if a
//!   signature over its domain-separated [`certification_message`] verifies under the
//!   **certifier's** key — the key bound by the certifier's own anchor [`TrustRule`]
//!   (`is_reserved`-style discipline: a graph *claiming* `trustx:certifies` proves
//!   nothing). A forged/absent signature ⇒ NOTHING.
//! - **Fail-closed on time.** An expired / not-yet-valid / (positive-)status-missing edge
//!   ⇒ NOTHING; the derived rule's freshness is `min(anchor.fresh_within, window)`.
//! - **No cycle-driven amplification, within OR across rounds.** The load-bearing mechanism
//!   is the CYCLE gate, now evaluated against the whole closure-so-far rather than only the
//!   direct anchors: a self-certifying edge (`certifier == certified_issuer`), or one whose
//!   certified issuer already holds a matching-key rule (direct OR derived at an earlier
//!   hop), is dropped — it can add no authority the graph did not already hold, and admitting
//!   it would let a cycle launder a broadening. A cycle `A → B → A` (or any longer loop)
//!   therefore converges after one traversal instead of re-entering and re-attenuating
//!   forever, and each edge contributes at most ONE rule to the whole closure —
//!   **`derived.len() <= certifications.len()` at any `depth_bound`** — so depth cannot
//!   multiply authority.
//!
//!   A **visited set keyed on `(certifier, certified_issuer)`** sits alongside it as a
//!   REDUNDANT, explicit statement of that same bound. Stated honestly: it is belt-and-braces,
//!   not the guard — disabling it turns NO test in this crate red, because the cycle gate
//!   already denies every edge it would have skipped (an edge that has fired leaves its
//!   certified issuer holding a matching-key rule). It is kept because it makes the
//!   ≤1-rule-per-edge bound a visible loop invariant rather than a consequence of reasoning
//!   about the cycle gate's key-equality test, and because it shrinks the per-round scan as
//!   rounds deepen. Do NOT cite it as the reason a cycle is safe; cite the cycle gate.
//! - **Depth-bounded.** The closure runs at most `depth_bound` rounds: round 1's anchors are
//!   `direct_rules`; round *k*'s anchors are exactly the rules round *k−1* derived that also
//!   pass the meta-scope gate below. It is iterative, not recursive-in-the-reasoner (the
//!   `sq-tu4e` discipline is untouched), and it stops early the moment a round derives
//!   nothing (or derives nothing that may certify). A `depth_bound` of `0` short-circuits to
//!   `direct_rules`; a chain LONGER than `depth_bound` simply has its tail left underived
//!   (fail-closed — the tail is denied, never approximated).
//! - **Meta-scope non-escalation (the depth>1 precondition).** A certification confers the
//!   authority to ISSUE statements of a scope; it does NOT on its own confer the authority to
//!   CONFER, which is strictly stronger. So a DERIVED rule may act as an edge source at a
//!   deeper hop only if its own incoming certification scope covers certification-issuing
//!   itself — concretely, only if its statement-type shape SELECTS `trustx:Certification`
//!   statements (the module-private `confers_certification_authority`). Being certified for
//!   `schema:age` makes you an ISSUER, not a REGISTRAR: without this gate,
//!   `GOV certifies MID (for age)` would
//!   let MID confer age authority on issuers GOV and the pod never considered — MID gaining
//!   an authority its certification never granted, which is a broadening in the meta
//!   dimension. `research/trust-graph-authorisation-2026-07.md` §2.4 rule 3 states the
//!   invariant and §2.5 made it the stated precondition for depth>1; this is where it is
//!   enforced. DIRECT anchors are exempt (they are the pod's own explicit decision), which is
//!   also what keeps `depth_bound = 1` byte-identical to the v1 closure.
//! - **Strict additivity.** ZERO certifications (or none that survive the gate) ⇒ output
//!   is `direct_rules` **byte-identical** (same rules, same order, cloned verbatim).
//! - **Opt-in.** The whole module is behind the default-OFF `cert-graph` feature; with it
//!   OFF nothing here is compiled and the default `sparq-trust` build is byte-identical.
//!
//! ## Honest scope — NO cryptographic-soundness or privacy claim (read first)
//!
//! Like the rest of this crate, `derive_effective_rules` is a research PoC and makes **no
//! settled cryptographic-soundness, anonymity, or unlinkability claim**. It is a
//! **fail-closed attenuation** layer: a CHECKED certifier signature over a canonical
//! message, plus scope/time narrowing — a *trust-anchored delegation* claim, not
//! cryptography. Framework trust is ANCHORED, not proven (design §7.2). The ZK estate it
//! shares a signature primitive with is internally re-audited but has **no external
//! accredited-cryptographer sign-off** (open gate `sq-qhy4`), and `sparq-mpc` is
//! honest-majority **semi-honest only**. The certifier's *own* key is still
//! operator-/DID-asserted (the live forgery vector D′, `sq-pfae.3`) — this layer defeats
//! a *broadening* or *forged-edge* escalation given honest anchor keys; it does not close
//! the upstream key-trust gap.
//!
//! [OPUS-4.8] sq-pfae.15 (epic sq-pfae, issue #940; design record
//! `research/trust-expression-spec.md` §3.4). Written while Fable unavailable — flag for
//! re-review when Fable returns. 🤖 SPARQ agent — certification-edge trust-graph closure.
//!
//! [OPUS-5] sq-13096 (issue #3087) — the depth-N generalisation: the v1 closure was
//! DEPTH-1 ONLY (a derived rule was never re-used as an anchor, so a framework-of-frameworks
//! chain `A certifies B, B certifies C` derived nothing for `C`). It now iterates rounds up
//! to `depth_bound` with a `(certifier, certified_issuer)` visited set, a per-hop
//! re-derivation of the ⊆ ceiling, and the SAME signature/time/shape gates per hop — plus the
//! meta-scope non-escalation gate above, which the depth>1 deferral in
//! `research/trust-graph-authorisation-2026-07.md` §2.5 named as its precondition and which
//! the bead's own spec did not enumerate.
//! 🤖 SPARQ agent — transitive framework-of-frameworks certification.

use crate::framework_vocab::{TRUSTX_CERTIFICATION, TRUSTX_CERTIFIES};
use crate::policy::{ShapeRef, TrustRule};
use crate::vocab::{RDF_TYPE, SH_NS};
use oxrdf::{NamedNode, Term};
use sparq_zk::encode::{encode_term, encode_triple, salt_from_bytes};
use sparq_zk::field::Fr;
use sparq_zk::poseidon2;
use sparq_zk::sig::{
    commitment_message, holder_key_digest, public_key_to_hex, signature_from_hex, verify, PublicKey,
};

/// `rdfs:Class` — a node typed this (with SHACL constraints) is an implicit-class SHACL
/// target, so `root rdf:type rdfs:Class` is a SELECTION edge (the root becomes a class
/// target), NOT a conformance constraint (the `sq-pfae.15` review fix, [`is_selection_edge`]).
const RDFS_CLASS: &str = "http://www.w3.org/2000/01/rdf-schema#Class";
/// `sh:ShapeClass` — SHACL 1.2 "a class that is also a node shape"; like `rdfs:Class` it
/// makes the root an implicit-class target (`sparq-shacl` honours both; see
/// `crates/sparq-shacl/src/model.rs` `focus_nodes`).
const SH_SHAPE_CLASS: &str = "http://www.w3.org/ns/shacl#ShapeClass";

/// The certification-scope a certification edge confers — a *statement-type* constraint
/// plus the *resource* scope it applies over. The derived rule's authority is the
/// intersection of the certifier's anchor with this scope, so a `CertScope` can only
/// **narrow** the certifier's authority (§3.4 scope-conformance).
///
/// The statement-type is modelled the same way [`TrustRule::shape`] is (a `trust:forShape`
/// SHACL node-shape, `trust:forPredicate` desugared into it), so the closure reuses the
/// crate's ONE statement-type idiom rather than inventing a second. `AnyServiceScope` is
/// the honest DIATF service-level granularity ("everything this certified service issues")
/// — the *coarsest* scope, and the ONLY one that never narrows a shape.
#[derive(Debug, Clone)]
pub enum CertScope {
    /// `trustx:AnyServiceScope` — the coarsest, service-level scope. It imposes NO
    /// statement-type narrowing of its own; the derived rule inherits the certifier
    /// anchor's shape unchanged. (It never *broadens* — the anchor shape is the ceiling.)
    AnyService,
    /// A statement-type-scoped certification: the issuer is certified only for statements
    /// matching this SHACL node-shape (a `trust:forPredicate` predicate is desugared into
    /// a single-predicate shape upstream, exactly as [`crate::policy`] does).
    Shape(ShapeRef),
}

/// A signed certification EDGE: an authority (`certifier`) attests that `certified_issuer`
/// is certified — under a framework — over `scope`, within `[valid_from, valid_until]`.
///
/// It is admitted into the closure ONLY if its [`certification_message`] signature
/// verifies under `certifier_key` (a CHECKED signature — never a self-asserted
/// `trustx:certifies` triple), the window covers `now`, and the derived authority
/// narrows (never broadens) the certifier's own anchor rule. Any failure ⇒ the edge
/// contributes NOTHING (fail-closed).
#[derive(Debug, Clone)]
pub struct Certification {
    /// `trustx:` — the certifying authority's identity (the framework operator / a
    /// higher issuer). To confer authority it MUST match the `source` of an anchor
    /// [`TrustRule`] in `direct_rules` (the certifier only confers what it itself holds).
    pub certifier: NamedNode,
    /// The certifier's verification key. The edge signature must verify under it, and it
    /// must equal the `issuer_key` of the matching anchor rule (so a graph cannot name a
    /// certifier whose key the pod does not already anchor).
    pub certifier_key: PublicKey,
    /// `trustx:certifies` — the certified issuer's identity (the DID/key being vouched
    /// for). The derived rule authorises THIS issuer, within the intersected scope.
    pub certified_issuer: NamedNode,
    /// The certified issuer's verification key — the derived [`TrustRule::issuer_key`],
    /// bound into the signed [`certification_message`] so the certifier's signature
    /// ATTESTS exactly which key it is vouching for (a substituted key breaks the
    /// signature — the delegation `sq-l5og` key-binding discipline applied to certs).
    pub certified_key: PublicKey,
    /// `trustx:scope` — what the issuer is certified to issue (the statement-type / scope
    /// narrowing). The derived rule's shape is `anchor.shape ∩ scope`.
    pub scope: CertScope,
    /// `trustx:validFrom` — inclusive start of the certification window (Unix seconds).
    pub valid_from_unix_secs: i64,
    /// `trustx:validUntil` — inclusive end of the certification window (Unix seconds). An
    /// edge with `valid_until < valid_from` is ill-formed and contributes nothing.
    pub valid_until_unix_secs: i64,
    /// The certifier's Schnorr signature over [`certification_message`], hex
    /// (`compressed(R) ‖ s`), exactly as `sparq_zk::sig` produces. An absent/forged
    /// signature fails closed.
    pub signature_hex: String,
}

/// Why a single certification edge contributed NOTHING to the derived rule set. Returned
/// by [`explain_edge`] (the per-edge fail-closed reason) so the adversarial matrix can
/// assert *which* gate denied a forged / broadening / expired edge, not merely that the
/// output was empty. NOT part of the fast path — [`derive_effective_rules`] simply drops
/// a rejected edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeRejection {
    /// The certifier names no anchor [`TrustRule`] in the hop's anchor set, OR the anchor it
    /// names binds a different key than `certifier_key` — the pod does not anchor this
    /// certifier (at this hop), so it can confer nothing.
    NoAnchor,
    /// The edge signature is absent or did not verify under `certifier_key` over the
    /// domain-separated [`certification_message`] (a self-asserted / forged edge).
    SignatureInvalid,
    /// The window is ill-formed (`valid_until < valid_from`) or does not cover `now`
    /// (expired / not-yet-valid) — fail-closed on time.
    OutOfWindow,
    /// `certifier == certified_issuer` (self-certification), or the certified issuer already
    /// holds a matching-key rule in the closure so far (a direct anchor, or one derived at an
    /// earlier hop) — a cycle that could launder a broadening or add no new authority.
    /// Dropped.
    Cyclic,
    /// The certification scope is NOT provably contained in the certifier's anchor scope
    /// (a broadening attempt, or an undecidable containment). Attenuation-only ⇒ dropped.
    Broadening,
    /// The `depth_bound` was `0` — no hop is permitted at all, so no edge can derive.
    ///
    /// An edge that is merely *deeper* than the bound is NOT reported here: it never gets a
    /// hop to be evaluated at, so [`explain_edge`] against the anchors it would have needed
    /// reports [`EdgeRejection::NoAnchor`] (its certifier is not in that anchor set). The
    /// closure-level statement is the behavioural one — a chain longer than `depth_bound`
    /// has its tail left underived (see [`derive_effective_rules`]).
    OverDepth,
}

/// Derive the **effective** trust-rule set from the Control-gated anchor rules plus a set
/// of signed certification edges — a depth-bounded, attenuation-only, fail-closed closure
/// that runs AHEAD of the UNCHANGED admission gate ([`mod@crate::admit`]).
///
/// - `direct_rules` — the anchor rules parsed from the Control-gated policy
///   ([`crate::policy::parse_policy`]). They are the trust ROOT; only a certifier that
///   already appears here (as a rule `source` with the matching key) can confer authority.
/// - `certifications` — the signed [`Certification`] edges (from a framework's published
///   Trusted-List / register). Each is CHECKED, never trusted on assertion.
/// - `now_unix_secs` — the evaluation instant, checked against each edge's window.
/// - `depth_bound` — the maximum number of certification HOPS. `1` is the depth-1 closure
///   (only `direct_rules` may certify); `2` additionally admits a framework-of-frameworks
///   chain `A certifies B, B certifies C`; `0` short-circuits to `direct_rules` verbatim.
///   A chain longer than `depth_bound` has its tail left **underived** (fail-closed).
///
/// The result is `direct_rules` (cloned **verbatim, in order** — strict additivity) with
/// the surviving derived rules appended, **shallowest hop first**. **Zero surviving
/// certifications ⇒ output is `direct_rules` byte-identical**; the admission gate then
/// consumes the result exactly as it consumes a hand-authored policy — no gate change.
///
/// Every derived rule satisfies the HARD invariant `derived ⊆ (anchor ∩ cert scope ∩
/// validity window)` (module docs) — **at every depth**, because hop *k* re-derives the ⊆
/// ceiling against the rule hop *k−1* produced, so the chain telescopes
/// `derived_N ⊆ … ⊆ derived_1 ⊆ anchor`. Any unverifiable / expired / out-of-window /
/// cyclic / over-depth / broadening edge contributes NOTHING, at whichever hop it is
/// reached. See [`explain_edge`] for the per-edge reason a forged / broadening / expired
/// edge is denied.
///
/// ## Termination and the no-amplification bound
///
/// Rounds are capped by `depth_bound` and stop early as soon as a round derives nothing.
/// **Each edge contributes at most ONE rule to the whole closure**, at the SHALLOWEST hop
/// that admits it — so `derived.len() <= certifications.len()` regardless of `depth_bound`,
/// and a cycle `A → B → A` cannot re-enter to amplify. That bound is enforced by the CYCLE
/// gate (an edge that has fired leaves its certified issuer holding a matching-key rule,
/// which the gate then rejects); the cross-round `seen` set keyed on
/// `(certifier, certified_issuer)` states the same bound redundantly and is documented as
/// belt-and-braces in the module docs, not as the guard.
pub fn derive_effective_rules(
    direct_rules: &[TrustRule],
    certifications: &[Certification],
    now_unix_secs: i64,
    depth_bound: u32,
) -> Vec<TrustRule> {
    // Strict additivity: the anchors are always the prefix, cloned verbatim in order.
    let mut effective: Vec<TrustRule> = direct_rules.to_vec();

    // Depth 0 (or no edges): the closure is a no-op — output is direct_rules byte-identical.
    if depth_bound == 0 || certifications.is_empty() {
        return effective;
    }

    // The cross-round visited set (sq-13096). Keyed on the edge IDENTITY `(certifier IRI,
    // certified-issuer IRI)`, it is populated ONLY on a SUCCESSFUL derivation — an edge that
    // failed at hop k (typically `NoAnchor`, because its certifier had not been derived yet)
    // MUST stay eligible at hop k+1, which is exactly what makes the chain transitive. Once
    // an edge HAS derived, it never derives again at a deeper hop.
    //
    // HONEST NOTE — this is REDUNDANT with GATE 1, not the guard. A fired edge leaves its
    // certified issuer holding a matching-key rule, so the cycle gate (which reads the whole
    // closure-so-far) already rejects the re-attempt; disabling this set turns no test red.
    // It is kept because it makes the ≤1-rule-per-edge bound a visible loop invariant and
    // shrinks the per-round scan as rounds deepen. Cite GATE 1, not this, for cycle safety.
    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    // The anchors usable at the CURRENT hop: hop 1 is the direct anchors; hop k>1 is EXACTLY
    // the rules hop k-1 derived. Restricting to the previous round's output (rather than all
    // of `effective`) is what gives `depth_bound` its meaning — and costs nothing, since any
    // edge an earlier round's anchors could have derived either already did (and is `seen`)
    // or failed a gate that a re-attempt against the same anchors would fail identically.
    let mut frontier: Vec<TrustRule> = direct_rules.to_vec();

    for hop in 1..=depth_bound {
        // Collected separately so `effective` stays a stable SNAPSHOT for the whole round:
        // the cycle gate must not see rules derived earlier in THIS round, or the outcome
        // would depend on the order of `certifications` (two certifiers vouching for the
        // same issuer at the same hop must both derive, exactly as at depth-1).
        let mut derived_this_round: Vec<TrustRule> = Vec::new();
        let mut edges_this_round: Vec<(String, String)> = Vec::new();
        for cert in certifications {
            let edge = (
                cert.certifier.as_str().to_owned(),
                cert.certified_issuer.as_str().to_owned(),
            );
            if seen.contains(&edge) {
                continue;
            }
            // `hop == 1` marks the anchors as the pod's DIRECT rules, which exempts them from
            // the meta-scope gate in GATE 2 — see `derive_one_explained`.
            if let Some(rule) = derive_one(&frontier, &effective, cert, now_unix_secs, hop == 1) {
                derived_this_round.push(rule);
                edges_this_round.push(edge);
            }
        }
        if derived_this_round.is_empty() {
            // Fixpoint: no edge is reachable from this frontier, so no deeper hop can be
            // either. Stop early rather than spinning out the remaining rounds.
            break;
        }
        seen.extend(edges_this_round);
        effective.extend(derived_this_round.iter().cloned());
        frontier = derived_this_round;
    }
    effective
}

/// Whether a DERIVED rule may itself act as a certification edge SOURCE at a deeper hop —
/// the **meta-scope non-escalation** gate (`research/trust-graph-authorisation-2026-07.md`
/// §2.4 rule 3, the precondition that record set on depth>1).
///
/// A certification confers the authority to ISSUE statements of a scope. It does NOT, on its
/// own, confer the authority to CONFER — that is a strictly stronger, meta-level authority.
/// An entity may act as an edge source only if its own incoming certification scope covers
/// certification-issuing itself, which here means its statement-type shape must **select**
/// `trustx:Certification` statements:
///
/// - `sh:targetSubjectsOf trustx:certifies` / `sh:targetObjectsOf trustx:certifies`, or
/// - `sh:targetClass trustx:Certification`.
///
/// Only ROOT-level SELECTION edges count ([`selection_edges`]) — a conformance constraint
/// mentioning the term selects nothing. Anything else — including
/// [`CertScope::AnyService`] over an attribute-only anchor — is **NOT** certification
/// authority: unknown ⇒ no proof ⇒ fail closed, exactly as everywhere else in this module.
///
/// Note the shape is the ceiling on BOTH readings at once, which is what makes this
/// composable: a registrar anchored to select `trustx:certifies` AND `schema:age` can pass an
/// `age`-narrowed rule down a chain (selection only ever shrinks), and the issuer at the end
/// — whose shape no longer selects `trustx:certifies` — cannot extend the chain further.
fn confers_certification_authority(shape: &ShapeRef) -> bool {
    let certifies = format!("I{}", TRUSTX_CERTIFIES);
    let certification = format!("I{}", TRUSTX_CERTIFICATION);
    selection_edges(shape).iter().any(|(predicate, object)| {
        match predicate.strip_prefix(SH_NS) {
            Some("targetSubjectsOf" | "targetObjectsOf") => *object == certifies,
            Some("targetClass") => *object == certification,
            // Every other selection form (targetNode, targetWhere, implicit-class typing, an
            // un-modelled `sh:target*`) carries no proof that certification statements are in
            // scope ⇒ no certification authority.
            _ => false,
        }
    })
}

/// Explain, per edge, why it did (`Ok`) or did NOT (`Err`) contribute — the fail-closed
/// reason the adversarial matrix asserts against. It mirrors [`derive_effective_rules`]'s
/// gate exactly (same checks, same order), returning the derived [`TrustRule`] on success
/// or the [`EdgeRejection`] on the FIRST failing gate.
///
/// The last two arguments identify the **single hop** being explained, and they must agree:
///
/// - `anchors` — the closure state at that hop. Pass the pod's `direct_rules` for hop 1; for
///   hop *k* of a multi-hop chain pass `derive_effective_rules(direct_rules, certs, now,
///   k - 1)`.
/// - `hop` — the 1-based hop index. `0` denies every edge as [`EdgeRejection::OverDepth`]
///   (no hop is permitted at all). `1` marks `anchors` as the pod's DIRECT rules, which
///   exempts them from the meta-scope gate; `>= 1 + 1` marks them as DERIVED, so only an
///   anchor whose shape selects `trustx:Certification` statements may certify.
///
/// ## Why ONE slice is enough, and where it is an over-approximation
///
/// The shared gate takes TWO rule sets, and the closure feeds it two DIFFERENT ones at hop
/// *k > 1*: the **frontier** (exactly what hop *k−1* derived — the certifying set, GATE 2)
/// and the **known** set (the whole closure so far — the cycle set, GATE 1). `explain_edge`
/// has one slice and passes it as both, so the set documented above is EXACT for GATE 1 and a
/// strict **superset** for GATE 2 — it also carries the direct anchors and every round older
/// than *k−1*, none of which the closure could certify from at that hop. Routing through one
/// shared gate is therefore NOT on its own why the two paths agree: that is a property of the
/// per-hop gate, not of the composed closure.
///
/// They agree anyway, and GATE 1 is the reason. Take any candidate anchor `R` this call
/// offers that hop *k*'s real frontier did not: `R` was ITSELF the frontier at some earlier
/// round *j*, and the edge's verdict against `R` is the same at both hops — GATES 3 and 4 do
/// not read the anchor set at all, GATE 5 reads only `R`, and GATE 1 saw a SMALLER closure at
/// round *j* so it was there if anything laxer (as was GATE 2, if *j* was hop 1: direct
/// anchors skip the meta-scope gate). So if `R` would admit the edge here, the closure
/// already admitted it at round *j*. That earlier admission leaves the certified issuer
/// holding a matching-key rule, and GATE 1 — which
/// reads the WHOLE closure so far — denies the re-attempt as [`EdgeRejection::Cyclic`]. So an
/// `Ok` here is an edge the closure derives at THIS hop, and the same rule; an `Err` is an
/// edge it does not derive at all. Pinned, mutation-witnessed, by
/// `an_older_round_certifier_cannot_re_admit_an_edge_at_a_later_hop` in
/// `tests/certification_graph_e2e.rs` — disable GATE 1's scan of the known set and it goes red.
///
/// Two honest caveats, both on the REASON and never on admission (so neither can escalate):
/// an edge the closure admitted at a SHALLOWER hop reports `Cyclic` here, where the closure's
/// loop simply skipped it as already-derived; and an edge whose certifier has left the
/// frontier reports whichever gate the stale anchor fails (e.g.
/// [`EdgeRejection::Broadening`]) rather than the [`EdgeRejection::NoAnchor`] the closure
/// reaches first.
///
/// Pass hop `1` with a set of DERIVED anchors and you are asking a different question (what a
/// pod would decide if it anchored those rules directly), and will get that question's answer.
pub fn explain_edge(
    anchors: &[TrustRule],
    cert: &Certification,
    now_unix_secs: i64,
    hop: u32,
) -> Result<TrustRule, EdgeRejection> {
    if hop == 0 {
        return Err(EdgeRejection::OverDepth);
    }
    derive_one_explained(anchors, anchors, cert, now_unix_secs, hop == 1)
}

/// The per-hop derivation used by the fast path: `Some(rule)` iff every gate passes, else
/// `None` (the reason is discarded — see [`explain_edge`] for it).
fn derive_one(
    anchors: &[TrustRule],
    known: &[TrustRule],
    cert: &Certification,
    now_unix_secs: i64,
    anchors_are_direct: bool,
) -> Option<TrustRule> {
    derive_one_explained(anchors, known, cert, now_unix_secs, anchors_are_direct).ok()
}

/// The single shared gate for ONE hop, returning the derived rule OR the first
/// [`EdgeRejection`]. Both [`derive_effective_rules`] (via [`derive_one`], once per round)
/// and [`explain_edge`] route through here, so ON THE SAME TWO INPUTS the fast path and the
/// explained path can NEVER diverge on what they admit — including on the meta-scope gate,
/// which is why that gate lives HERE and not in the closure's round loop. `explain_edge`
/// cannot in fact supply the same two inputs at hop k>1 (it has one slice); that it still
/// agrees is a separate argument, made on [`explain_edge`] itself.
///
/// The two rule slices play DIFFERENT roles and are only the same at hop 1:
/// - `anchors` — the rules that may CERTIFY at this hop (GATE 2). At hop 1 the pod's direct
///   anchors; at hop k>1 exactly what hop k-1 derived. This is what bounds the depth.
/// - `known` — every rule the closure holds so far (direct + all previously derived), used
///   ONLY by the cycle gate (GATE 1) so a loop that re-certifies an issuer the graph already
///   vouches for adds nothing. Widening the cycle gate to the whole closure is what stops a
///   CROSS-ROUND cycle from laundering a broadening.
///
/// `anchors_are_direct` says whether `anchors` is the pod's own Control-gated rule set (hop
/// 1) or a set the closure DERIVED (hop k>1). It selects the meta-scope gate in GATE 2.
fn derive_one_explained(
    anchors: &[TrustRule],
    known: &[TrustRule],
    cert: &Certification,
    now_unix_secs: i64,
    anchors_are_direct: bool,
) -> Result<TrustRule, EdgeRejection> {
    // GATE 1 — CYCLE. A self-certifying edge, or one whose certified issuer already anchors
    // the certifier, is dropped BEFORE any other work: it can confer no authority the graph
    // did not already hold, and admitting it is the shape a cycle would use to launder a
    // broadening. (Checked first so a forged self-cert can't even reach the sig check.)
    if cert.certifier.as_str() == cert.certified_issuer.as_str() {
        return Err(EdgeRejection::Cyclic);
    }
    if known.iter().any(|r| {
        r.source.as_str() == cert.certified_issuer.as_str()
            && public_key_to_hex(&r.issuer_key) == public_key_to_hex(&cert.certified_key)
    }) {
        // The "certified" issuer ALREADY holds a rule in the closure under this key — a
        // direct anchor, or one derived at an earlier hop. The edge would close a cycle
        // (`A → B → A`, or any longer loop) and can add no authority the graph did not
        // already hold, so it is dropped. Checking against the WHOLE closure so far, not just
        // `direct_rules`, is what keeps a CROSS-ROUND cycle from laundering a broadening: a
        // deeper hop cannot re-grant an issuer a wider rule than the one it already earned.
        return Err(EdgeRejection::Cyclic);
    }

    // GATE 2 — ANCHOR. The certifier MUST be an anchor OF THIS HOP whose key matches. This is
    // the depth bound made concrete: at hop 1 only a `direct_rules` source may certify, and at
    // hop k only what hop k-1 derived — so an edge is never evaluated at a depth the caller
    // did not authorise. It also binds the certifier's key to the one the pod holds for it
    // (directly, or as attested by the previous hop's signed edge), so a graph cannot name a
    // certifier whose key it does not already hold.
    // All anchors the certifier holds (a certifier may be anchored for several statement-types
    // over the same key — e.g. `age` AND `name`). We keep the WHOLE set so GATE 5 can pick the
    // one the certification provably narrows, never just the first.
    //
    // META-SCOPE NON-ESCALATION (`research/trust-graph-authorisation-2026-07.md` §2.4 rule 3 —
    // the precondition that record set on depth>1). At hop k>1 the anchors are DERIVED rules,
    // and a certification confers the authority to ISSUE statements of a scope, NOT the
    // authority to CONFER, which is strictly stronger. So a derived rule may certify only if
    // its own incoming certification scope covers certification-issuing itself — i.e. only if
    // its shape SELECTS `trustx:Certification` statements. Being certified to issue
    // `schema:age` makes you an ISSUER, not a REGISTRAR: without this, `GOV certifies MID (for
    // age)` would let MID confer age authority on an issuer GOV and the pod never considered.
    // That broadening is invisible to GATE 5, because the rule MID confers IS inside MID's own
    // shape — it is a broadening in the META dimension, which is exactly why the design record
    // called it moot at depth 1 and deferred depth>1 pending "the certification-authority scope
    // shape". The shape IS that scope, so no new vocabulary is needed.
    //
    // DIRECT anchors are deliberately EXEMPT: they are the pod's own explicit decision about
    // who may confer, and exempting them is what keeps `depth_bound = 1` byte-identical to the
    // pre-sq-13096 closure. An anchor that fails this gate is simply not a candidate, so the
    // edge reports `NoAnchor` — fail-closed, and honest (the pod anchors no CERTIFIER here).
    let candidate_anchors: Vec<&TrustRule> = anchors
        .iter()
        .filter(|r| {
            r.source.as_str() == cert.certifier.as_str()
                && public_key_to_hex(&r.issuer_key) == public_key_to_hex(&cert.certifier_key)
                && (anchors_are_direct || confers_certification_authority(&r.shape))
        })
        .collect();
    if candidate_anchors.is_empty() {
        return Err(EdgeRejection::NoAnchor);
    }

    // GATE 3 — CHECKED SIGNATURE. The edge is admitted only if a signature over the
    // domain-separated `certification_message` verifies under the CERTIFIER's key. A graph
    // merely *claiming* `trustx:certifies` proves nothing; a forged/absent sig fails closed.
    let msg = commitment_message(&certification_message(cert));
    let sig = signature_from_hex(&cert.signature_hex).ok_or(EdgeRejection::SignatureInvalid)?;
    if !verify(&cert.certifier_key, &msg, &sig) {
        return Err(EdgeRejection::SignatureInvalid);
    }

    // GATE 4 — TIME. The window must be well-formed AND cover `now` (inclusive). An expired
    // / not-yet-valid / inverted-window edge fails closed. (Positive, existence-of-a-covering
    // -window semantics — never evidence-of-absence; the OWA/monotonicity §3.3 D3 rule.)
    if cert.valid_until_unix_secs < cert.valid_from_unix_secs
        || now_unix_secs < cert.valid_from_unix_secs
        || now_unix_secs > cert.valid_until_unix_secs
    {
        return Err(EdgeRejection::OutOfWindow);
    }

    // GATE 5 — ATTENUATION (shape). The derived statement-type is anchor.shape ∩ cert.scope,
    // and it MUST be provably ⊆ the anchor's shape (a cert can only narrow). Try EACH candidate
    // anchor and take the FIRST one the certification provably narrows — so a certifier anchored
    // for several types is correctly narrowed by a cert scoped to one of them (never widened:
    // the chosen anchor is always genuine pod authority). If NO candidate anchor is narrowed by
    // this cert scope, it is a broadening ⇒ contributes nothing. Where narrowing cannot be
    // PROVEN (undecidable SHACL-shape containment), each attempt yields None ⇒ Broadening.
    // At hop k>1 the candidate anchors are hop k-1's DERIVED rules, so this same check
    // re-derives the ⊆ ceiling against the already-attenuated shape — the chain telescopes
    // (derived_k ⊆ derived_{k-1} ⊆ … ⊆ anchor) and a broadening at ANY hop is denied there.
    let (anchor, derived_shape) = candidate_anchors
        .iter()
        .find_map(|a| narrowed_shape(&a.shape, &cert.scope).map(|s| (*a, s)))
        .ok_or(EdgeRejection::Broadening)?;

    // GATE 6 — ATTENUATION (freshness). The derived rule is no fresher than the anchor AND
    // its window is no wider than the certification window: `min(anchor.fresh_within, window)`.
    // A cert can only shrink the admitted staleness, never widen it.
    let window_secs = cert
        .valid_until_unix_secs
        .saturating_sub(now_unix_secs)
        .max(0);
    let derived_fresh = anchor.fresh_within_secs.min(window_secs);

    // The derived rule: it authorises the CERTIFIED issuer + key, but its scope (resource),
    // shape (statement-type), and freshness are all the certifier anchor's — NARROWED by the
    // certification. `scope` is the anchor's resource scope UNCHANGED: a certification narrows
    // WHO and WHAT-TYPE, never widens WHERE (the resource scope is the certifier's ceiling).
    Ok(TrustRule {
        source: cert.certified_issuer.clone(),
        issuer_key: cert.certified_key,
        shape: derived_shape,
        scope: anchor.scope.clone(),
        fresh_within_secs: derived_fresh,
    })
}

/// The narrowed statement-type shape `anchor.shape ∩ cert.scope`, or `None` if narrowing
/// cannot be PROVEN (⇒ the edge contributes nothing — attenuation-only).
///
/// Decidable cases (the only ones that ever produce `Some`):
/// - `CertScope::AnyService` — imposes no statement-type narrowing of its own, so the
///   derived shape is the anchor's shape UNCHANGED (the anchor is the ceiling; this never
///   broadens).
/// - `CertScope::Shape(s)` where `s` **narrows** the anchor shape via the
///   **target-set-CONTRAVARIANT, conformance-COVARIANT** containment check
///   ([`shape_narrows`]): the cert's SELECTION/target predicates are a SUBSET of the
///   anchor's (targets may only shrink — an extra target WIDENS the admitted set), AND its
///   CONFORMANCE constraints are a SUPERSET of the anchor's (constraints may only grow, via
///   the injective structural match [`conformance_contains`]). The derived shape is the CERT
///   shape (the strictly-tighter one). If NOT provable ⇒ `None` (fail-closed).
///
/// Every other case — a cert shape that DROPS an anchor constraint, ADDS a target predicate
/// the anchor lacks (both broadenings), carries an UN-MODELLED selection predicate, or one
/// whose conformance containment cannot be structurally decided within the bounded matcher —
/// returns `None`. This is the design's "undecidable containment ⇒ NOT contained ⇒
/// contributes nothing" rule (§3.4), the SAFE side for authorisation: a false `None` merely
/// fails to admit; a false `Some` would escalate. The matcher is deliberately CONSERVATIVE —
/// it proves narrowing only for shapes it can structurally match, never guesses.
///
/// ## Why "more edges ⇒ narrower" is UNSOUND (the `sq-pfae.15` review fix)
///
/// A SHACL shape selects focus nodes by the **UNION** of its target predicates and then
/// requires each to satisfy every conformance constraint. Adding a CONFORMANCE constraint
/// removes conforming nodes (narrows); but adding a TARGET predicate selects MORE nodes
/// (widens). A prior version treated any injective structural superset as a narrowing — so a
/// cert = anchor shape **+ an extra `sh:targetSubjectsOf schema:email`** passed as a
/// "narrowing" and granted the certified issuer authority over `email` the certifier never
/// held (privilege escalation). Selection vocab is therefore CONTRAVARIANT to conformance
/// vocab, and the containment check splits the two ([`selection_only_shrinks`] +
/// [`conformance_contains`]).
fn narrowed_shape(anchor: &ShapeRef, cert_scope: &CertScope) -> Option<ShapeRef> {
    match cert_scope {
        // AnyService adds no statement-type constraint: the derived shape IS the anchor's
        // (the anchor remains the ceiling). No broadening is possible.
        CertScope::AnyService => Some(anchor.clone()),
        CertScope::Shape(cert_shape) => {
            // A cert shape narrows the anchor ONLY if BOTH hold:
            //   (a) its SELECTION/target set is a SUBSET of the anchor's — targets may only
            //       shrink; an extra target predicate WIDENS the admitted set (a broadening),
            //       and an UN-MODELLED selection predicate fails closed; AND
            //   (b) its CONFORMANCE constraints are a SUPERSET of the anchor's — constraints
            //       may only grow (fewer nodes conform), proven by the injective structural
            //       match.
            // If BOTH hold, the cert admits a SUBSET of what the anchor admits ⇒ a narrowing;
            // the derived shape is the cert's (strictly-tighter) shape. Otherwise ⇒ None.
            if shape_narrows(cert_shape, anchor) {
                Some(cert_shape.clone())
            } else {
                None
            }
        }
    }
}

/// Whether the `cert` shape provably NARROWS the `anchor` shape as an admitted-node set:
/// **selection-set-contravariant, conformance-set-covariant**. Both halves must hold:
///
/// - [`selection_only_shrinks`] — the cert's SELECTION/target predicates are a subset of the
///   anchor's (targets may only shrink; an extra target widens; an un-modelled selection
///   predicate fails closed), AND
/// - [`conformance_contains`] — the cert's CONFORMANCE constraints structurally contain the
///   anchor's (constraints may only grow), via the injective, root-anchored structural match.
///
/// Returns `true` ONLY when it PROVES both — any un-decidable / broadening case is the
/// fail-closed `false`. Splitting the two directions is the `sq-pfae.15` review fix: a
/// uniform "more edges ⇒ narrower" is unsound because a target edge widens, not narrows.
///
/// ## The polarity precondition (the `sq-wh546` fix)
///
/// The covariant half ("more conformance constraints ⇒ fewer conforming nodes") is a
/// MONOTONICITY argument, and it holds ONLY for purely-CONJUNCTIVE constraints. SHACL has
/// NON-MONOTONE constraint components — `sh:not`, `sh:or`, `sh:xone`, `sh:maxCount`,
/// `sh:qualifiedValueShape`/`sh:qualifiedMaxCount`, … — under which adding a constraint
/// edge INVERTS the effect: a cert that adds `sh:datatype` INSIDE an anchor's
/// `sh:not[...]` is an injective structural SUPERSET yet admits STRICTLY MORE nodes (the
/// negated inner shape becomes harder to satisfy, so the negation admits more). A
/// polarity-blind containment check reported that broadening cert as a narrowing —
/// privilege escalation. So BOTH shape closures must first pass
/// [`monotone_conjunctive_only`]: every constraint predicate must carry a monotonicity
/// PROOF ([`is_monotone_conformance_predicate`]); any non-monotone OR unknown predicate
/// anywhere in either closure ⇒ fail-closed `false` (the edge rejects as
/// [`EdgeRejection::Broadening`]).
fn shape_narrows(cert: &ShapeRef, anchor: &ShapeRef) -> bool {
    monotone_conjunctive_only(cert)
        && monotone_conjunctive_only(anchor)
        && selection_only_shrinks(cert, anchor)
        && conformance_contains(cert, anchor)
}

/// SHACL constraint predicates (local names under `sh:`) with a PROVEN monotonicity for
/// the injective-superset narrowing argument: in a purely-conjunctive node shape, ADDING
/// an edge with one of these predicates can only SHRINK (or preserve) the conforming set.
///
/// - `property` — conjunctive composition: another property shape to satisfy.
/// - `path` — a structural parameter naming which values a property shape constrains
///   (paired with the constraints beside it; on its own it constrains nothing). Complex
///   (blank-node) paths carry their own `sh:` structure (`sh:inversePath`,
///   `sh:alternativePath`, …) which is NOT in this list, so they fail closed below.
/// - `minCount` / `datatype` / `class` — conjunctive value constraints: each added edge
///   is one more requirement every value/focus node must meet.
///
/// DELIBERATELY ABSENT (non-monotone — adding one, or adding an edge inside one,
/// can GROW the admitted set): `not`, `or`, `xone`, `maxCount`, `qualifiedValueShape`,
/// `qualifiedMaxCount`, `in`, `languageIn`, `closed`/`ignoredProperties`, … — and every
/// OTHER predicate, known or future, for which no monotonicity proof has been written
/// down here. Unknown ⇒ NOT monotone ⇒ reject (fail-closed).
const MONOTONE_CONFORMANCE_LOCALS: [&str; 5] = ["property", "path", "minCount", "datatype", "class"];

/// Whether `predicate` is a conformance predicate the narrowing matcher holds a
/// monotonicity PROOF for ([`MONOTONE_CONFORMANCE_LOCALS`]). Anything else — a
/// non-monotone SHACL component (`sh:not`, `sh:or`, `sh:xone`, `sh:maxCount`,
/// `sh:qualifiedValueShape`, `sh:qualifiedMaxCount`, …), an unknown/future `sh:` term, or
/// a non-SHACL predicate (a SPARQL-based custom constraint component's parameter can be
/// ANY IRI) — is NOT monotone: unknown ⇒ reject.
fn is_monotone_conformance_predicate(predicate: &str) -> bool {
    predicate
        .strip_prefix(SH_NS)
        .is_some_and(|local| MONOTONE_CONFORMANCE_LOCALS.contains(&local))
}

/// The polarity gate (the `sq-wh546` fix): whether EVERY triple in the shape closure is
/// one the injective-superset narrowing argument is SOUND for. Each triple must be
/// either:
/// - a SELECTION/target edge ([`is_selection_edge`]) — handled CONTRAVARIANTLY by
///   [`selection_only_shrinks`], excluded from the covariant match; or
/// - an `rdf:type` declaration — not a SHACL constraint parameter (the class-TARGET
///   typing is already classified as selection above; `rdf:type sh:NodeShape` etc. is
///   inert); or
/// - a conformance predicate with a written monotonicity proof
///   ([`is_monotone_conformance_predicate`]).
///
/// ANY other predicate anywhere in the closure — a non-monotone SHACL component, an
/// un-modelled `sh:` term, or an arbitrary (possibly custom-constraint) IRI — makes the
/// monotonicity argument unsound for the WHOLE shape (an added edge in an anti-monotone
/// position BROADENS), so the shape is rejected wholesale: this function returns `false`
/// and [`shape_narrows`] fails closed. Applied to BOTH the cert and the anchor closure.
fn monotone_conjunctive_only(shape: &ShapeRef) -> bool {
    shape.triples.iter().all(|t| {
        let p = t.predicate.as_str();
        is_selection_edge(p, &t.object) || p == RDF_TYPE || is_monotone_conformance_predicate(p)
    })
}

/// Whether the cert's SELECTION/target set is a SUBSET of the anchor's — the CONTRAVARIANT
/// half of narrowing. SHACL selects focus nodes by the UNION of a shape's target predicates,
/// so a cert may narrow ONLY if it targets a subset of the nodes the anchor targets: every
/// selection edge the cert declares at its root must also be declared at the anchor's root.
///
/// Fail-closed on TWO axes:
/// - An **extra** cert selection edge the anchor lacks (a broadening, e.g. an additional
///   `sh:targetSubjectsOf schema:email`) ⇒ `false`.
/// - An **un-modelled** selection predicate (anything in the SHACL `sh:target*` family — or
///   implicit-class typing — this check does not fully understand) ⇒ `false` regardless, so
///   a future/unknown target vocab can never be silently treated as a conformance constraint
///   and slip through the covariant half.
///
/// A selection edge is a root-level `(predicate, object)` whose predicate is a SHACL target
/// predicate ([`is_selection_predicate`]), OR the implicit-class typing `root rdf:type
/// {rdfs:Class | sh:ShapeClass}` (which makes the root itself a class target). Selection is
/// compared as a SET of `(predicate, object)` root edges: the cert set must be `⊆` the anchor
/// set (object equality is by [`term_id`]; a target object is always ground in the shapes the
/// crate mints, so this is exact).
fn selection_only_shrinks(cert: &ShapeRef, anchor: &ShapeRef) -> bool {
    let anchor_sel = selection_edges(anchor);
    for edge in selection_edges(cert) {
        if !anchor_sel.contains(&edge) {
            // The cert declares a selection edge the anchor does not — it targets a node the
            // anchor never selected ⇒ a broadening ⇒ fail closed.
            return false;
        }
    }
    true
}

/// The set of a shape's ROOT-level selection/target edges, keyed as `(predicate, object-id)`.
/// Includes every SHACL target predicate ([`is_selection_predicate`]) asserted on the root,
/// plus the implicit-class typing (`root rdf:type {rdfs:Class | sh:ShapeClass}`) rendered as
/// a synthetic `(rdf:type, <object-id>)` edge so it is compared like any other target. Only
/// ROOT edges select focus nodes in SHACL; conformance constraints reachable through
/// `sh:property` blank nodes never widen the target set, so they are excluded here (they are
/// the covariant half, checked by [`conformance_contains`]).
fn selection_edges(shape: &ShapeRef) -> std::collections::HashSet<(String, String)> {
    let root_id = term_id(&shape.root);
    shape
        .triples
        .iter()
        .filter(|t| term_id(&Term::from(t.subject.clone())) == root_id)
        .filter(|t| is_selection_edge(t.predicate.as_str(), &t.object))
        .map(|t| (t.predicate.as_str().to_string(), term_id(&t.object)))
        .collect()
}

/// Whether a root triple `(predicate, object)` is a SELECTION/target edge (contributes focus
/// nodes), as opposed to a conformance constraint. A selection edge is:
/// - any SHACL target predicate ([`is_selection_predicate`]) — `sh:targetNode` /
///   `sh:targetClass` / `sh:targetSubjectsOf` / `sh:targetObjectsOf` / `sh:targetWhere` and
///   ANY other `sh:target*` (fail-closed on un-modelled `sh:target*` names); OR
/// - the implicit-class typing `rdf:type {rdfs:Class | sh:ShapeClass}` (the root becomes a
///   class target). A plain `rdf:type sh:NodeShape` is NOT a target — it declares the node a
///   shape without selecting any focus node — so it stays a (harmless) conformance edge.
fn is_selection_edge(predicate: &str, object: &Term) -> bool {
    if is_selection_predicate(predicate) {
        return true;
    }
    // Implicit-class target: `root rdf:type rdfs:Class` (or SHACL-1.2 `sh:ShapeClass`) makes
    // the root a class target. `rdf:type sh:NodeShape` does not select anything.
    predicate == RDF_TYPE
        && matches!(object, Term::NamedNode(n) if n.as_str() == RDFS_CLASS || n.as_str() == SH_SHAPE_CLASS)
}

/// Whether `predicate` is a SHACL target/selection predicate. Recognises the whole
/// `sh:target*` family by prefix so an UN-MODELLED `sh:target…` (e.g. a future selection
/// mechanism) is still treated as selection ⇒ fail-closed at [`selection_only_shrinks`],
/// never silently admitted as a conformance constraint. This is the "any un-modelled
/// selection predicate ⇒ fail-closed" rule (the `sq-pfae.15` review fix).
fn is_selection_predicate(predicate: &str) -> bool {
    predicate
        .strip_prefix(SH_NS)
        .is_some_and(|local| local.starts_with("target"))
}

/// Whether the `sup` (cert) shape's CONFORMANCE constraints CONTAIN the `sub` (anchor)
/// shape's, via a bounded, root-anchored, **injective** structural match — the COVARIANT half
/// of narrowing (more constraints ⇒ fewer conforming nodes). SELECTION/target root edges are
/// EXCLUDED from this match (they are the contravariant half, [`selection_only_shrinks`]);
/// this compares only conformance constraints.
///
/// Identifying `sub.root` with `sup.root`, every conformance triple reachable from `sub.root`
/// must have a DISTINCT matching triple reachable from `sup.root`, with blank-node objects
/// matched RECURSIVELY (structural position, not label) under an injective blank-node
/// assignment, and IRI/literal objects matched exactly.
///
/// **Injectivity is the soundness guarantee against a false positive:** each anchor
/// constraint edge is matched to a *distinct* cert constraint edge, and each anchor blank node
/// to a *distinct* cert blank node — so a cert with FEWER constraints can never be reported as
/// containing an anchor with more (a broadening reported as narrowing = escalation). It is
/// SOUND for the SHACL node-shapes the crate mints (the `forPredicate`/`forShape` idiom: a
/// root with a few direct constraints + one level of `sh:property` blank nodes) and
/// conservative beyond them — it returns `true` only when it PROVES an injective embedding, so
/// any un-provable case is the fail-closed (safe) `false`.
fn conformance_contains(sup: &ShapeRef, sub: &ShapeRef) -> bool {
    let mut mapping: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    node_contains(&sup.root, sup, &sub.root, sub, &mut mapping)
}

/// Recursive node match with an injective blank-node `mapping` (anchor-node-id →
/// cert-node-id): does the CONFORMANCE constraint sub-tree rooted at `sub_node` embed into the
/// one rooted at `sup_node`? Every outgoing CONFORMANCE `(predicate, object)` edge of
/// `sub_node` must be matched to a DISTINCT conformance edge of `sup_node` (injective edge
/// assignment), and any blank-node object must map injectively (consistently across the whole
/// match) into a cert blank node.
///
/// At the ROOT node, SELECTION/target edges are filtered OUT (they are handled contravariantly
/// by [`selection_only_shrinks`]); nested blank-node property shapes never carry a root target
/// predicate, so the filter is a no-op below the root — but it is applied uniformly for
/// safety.
fn node_contains(
    sup_node: &Term,
    sup: &ShapeRef,
    sub_node: &Term,
    sub: &ShapeRef,
    mapping: &mut std::collections::HashMap<String, String>,
) -> bool {
    // Enforce a consistent, injective blank-node assignment sub → sup. If sub_node is already
    // mapped, it must map to THIS sup_node; and no OTHER sub node may already map to sup_node.
    let sub_id = term_id(sub_node);
    let sup_id = term_id(sup_node);
    match mapping.get(&sub_id) {
        Some(existing) if *existing == sup_id => return true, // already matched, consistent
        Some(_) => return false,                              // sub_node maps elsewhere — inconsistent
        None => {}
    }
    if mapping.values().any(|v| *v == sup_id) {
        return false; // sup_node already used by a DIFFERENT sub node — not injective
    }
    mapping.insert(sub_id.clone(), sup_id.clone());

    // CONFORMANCE edges only: selection/target edges are excluded from the covariant match
    // (they are compared contravariantly by `selection_only_shrinks`). Including a cert target
    // edge here would let an extra target on the cert be treated as a mere "extra constraint"
    // (superset ⇒ narrowing) — the exact unsound equivalence the review fixed.
    let sup_edges: Vec<(&str, &Term)> = edges_of(sup, sup_node)
        .into_iter()
        .filter(|(p, o)| !is_selection_edge(p, o))
        .collect();
    let sub_edge_list: Vec<(&str, &Term)> = edges_of(sub, sub_node)
        .into_iter()
        .filter(|(p, o)| !is_selection_edge(p, o))
        .collect();
    // Injective edge match: assign each sub edge to a DISTINCT sup edge (by index).
    let mut used: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let ok = sub_edge_list.iter().all(|(sub_pred, sub_obj)| {
        let hit = sup_edges.iter().enumerate().find(|(i, (sup_pred, sup_obj))| {
            !used.contains(i)
                && sup_pred == sub_pred
                && obj_matches(sup_obj, sup, sub_obj, sub, mapping)
        });
        if let Some((i, _)) = hit {
            used.insert(i);
            true
        } else {
            false
        }
    });
    if !ok {
        // Roll back this node's mapping so a failed branch does not poison a sibling.
        mapping.remove(&sub_id);
    }
    ok
}

/// Whether a cert-shape object `sup_obj` matches an anchor-shape object `sub_obj`: a
/// blank-node `sub_obj` matches by RECURSING (its sub-tree must embed into `sup_obj`'s,
/// injectively); an IRI/literal matches by exact equality.
fn obj_matches(
    sup_obj: &Term,
    sup: &ShapeRef,
    sub_obj: &Term,
    sub: &ShapeRef,
    mapping: &mut std::collections::HashMap<String, String>,
) -> bool {
    match (sup_obj, sub_obj) {
        // A blank-node constraint node in the anchor must be structurally matched by a
        // blank-node constraint node in the cert (recurse into their sub-trees, injectively).
        (Term::BlankNode(_), Term::BlankNode(_)) => {
            node_contains(sup_obj, sup, sub_obj, sub, mapping)
        }
        // A blank node on one side and a ground term on the other never match.
        (_, Term::BlankNode(_)) | (Term::BlankNode(_), _) => false,
        // Ground objects (IRIs / literals) match by exact term equality.
        (a, b) => a == b,
    }
}

/// The outgoing `(predicate, object)` edges whose subject is `node`.
fn edges_of<'a>(shape: &'a ShapeRef, node: &Term) -> Vec<(&'a str, &'a Term)> {
    let node_id = term_id(node);
    shape
        .triples
        .iter()
        .filter(|t| term_id(&Term::from(t.subject.clone())) == node_id)
        .map(|t| (t.predicate.as_str(), &t.object))
        .collect()
}

/// A stable id for an RDF term used only for the module-local structural match (NOT a
/// canonicalisation). Blank nodes are keyed by their local id; the match identifies the two
/// shape roots explicitly (via the recursion entry) so a root-label mismatch never defeats
/// containment.
fn term_id(t: &Term) -> String {
    match t {
        Term::NamedNode(n) => format!("I{}", n.as_str()),
        Term::BlankNode(b) => format!("B{}", b.as_str()),
        Term::Literal(l) => format!("L{}", l),
        // RDF-1.2 triple terms: rendered via Debug — only used for the structural match,
        // where any inequality is the safe (fail-closed) side.
        other => format!("T{:?}", other),
    }
}

/// The message a certifier signs (and [`derive_effective_rules`] verifies): a
/// domain-separated `Poseidon2` hash binding the WHOLE edge — the certifier, the CERTIFIED
/// issuer AND ITS KEY, the scope, and the validity window.
///
/// ## Why every field MUST be in the signed preimage (the attenuation soundness)
///
/// Binding all of these is load-bearing, not cosmetic — it is the certification analogue
/// of `delegation::hop_message`'s `sq-l5og` key-binding fix:
///
/// - Binding `certified_key` means an attacker cannot lift a genuine certification and
///   substitute its OWN key as the certified issuer's — that would break the certifier's
///   signature (rejected at the sig gate). Without it, a captured edge would let a forger
///   ride the certifier's authority under an attacker-chosen key.
/// - Binding the scope + window means a forger cannot lift a narrow certification and
///   re-present it with a BROADER scope or a longer window — the tampered message no longer
///   verifies. The scope narrowing is thus attested by the certifier, not asserted by the
///   holder.
///
/// The certifier signs this via `SecretKey::sign_commitment` (which wraps it in the
/// domain-separated `commitment_message` before signing), and the closure verifies against
/// the same wrapped form — so the value returned here is the *pre-wrap* message. The domain
/// tag `"TRUSTCRT"` keeps a certification signature from ever being confused with a
/// delegation-hop, credential-commitment, or holder-PoP signature (distinct domain tags).
pub fn certification_message(cert: &Certification) -> Fr {
    // Domain tag "TRUSTCRT" — a certification-edge signature is never confused with a
    // delegation hop ("TRUSTDLG"), a commitment, or a PoP (cross-protocol confusion defence).
    const SIG_DOMAIN_CERTIFICATION: u64 = 0x5452_5553_5443_5254; // "TRUSTCRT"
    let mut inputs: Vec<Fr> = vec![
        Fr::from(SIG_DOMAIN_CERTIFICATION),
        iri_to_field(&cert.certifier),
        key_to_field(&cert.certifier_key),
        iri_to_field(&cert.certified_issuer),
        key_to_field(&cert.certified_key),
        Fr::from(cert.valid_from_unix_secs.unsigned_abs()),
        Fr::from(u64::from(cert.valid_from_unix_secs < 0)),
        Fr::from(cert.valid_until_unix_secs.unsigned_abs()),
        Fr::from(u64::from(cert.valid_until_unix_secs < 0)),
    ];
    // The scope is folded into the signed preimage so a forger cannot re-present a narrow
    // certification with a broader scope: a scope kind tag + the shape's constraint set.
    match &cert.scope {
        CertScope::AnyService => {
            inputs.push(Fr::from(SCOPE_KIND_ANY_SERVICE));
        }
        CertScope::Shape(shape) => {
            inputs.push(Fr::from(SCOPE_KIND_SHAPE));
            inputs.push(Fr::from(shape.triples.len() as u64));
            // Sign over each shape-defining triple via the estate's domain-separated
            // `encode_triple`, so any tampering with the scope breaks the signature. Signed
            // in listed order — the honest signer re-serialises the same shape identically.
            let salt = salt_from_bytes(&[0u8; 32]);
            for t in &shape.triples {
                // A triple that fails to encode (only an RDF-1.2 triple-term subject/object
                // can) folds a fixed sentinel instead of silently dropping — the shape still
                // signs deterministically and the structural narrowing gate stays the arbiter.
                let f = encode_triple(t, &salt).unwrap_or_else(|| Fr::from(UNENCODABLE_TRIPLE_SENTINEL));
                inputs.push(f);
            }
        }
    }
    poseidon2::hash(&inputs)
}

/// Scope-kind domain tag for `AnyService`, folded into [`certification_message`] so the
/// scope kind itself is signed (a forger cannot flip a shape-scope to `AnyService`).
const SCOPE_KIND_ANY_SERVICE: u64 = 0x414e_5953_5643_0000; // "ANYSVC"
/// Scope-kind domain tag for a shape-scope.
const SCOPE_KIND_SHAPE: u64 = 0x5348_4150_4500_0000; // "SHAPE"
/// Sentinel folded for a shape triple that cannot be `encode_triple`-encoded (only an
/// RDF-1.2 triple-term subject/object), so the message stays deterministic; the structural
/// narrowing gate ([`narrowed_shape`]) remains the arbiter of what such a shape admits.
const UNENCODABLE_TRIPLE_SENTINEL: u64 = 0x5452_5553_5443_5458; // "TRUSTCTX"

/// Fold a key into the signed preimage as its domain-separated coordinate digest (reuses
/// `sparq_zk::sig::holder_key_digest`, `Poseidon2([ZKSIG_HK, x, y])` — no new hashing
/// primitive). The curve identity has no affine coordinates and is never a usable signing
/// key (`verify` rejects it independently); it is bound as a FIXED sentinel so an
/// identity-key edge is still deterministically signed/verified and the Schnorr layer's own
/// identity rejection keeps the path fail-closed. Mirrors `delegation::key_to_field`.
fn key_to_field(pk: &PublicKey) -> Fr {
    const IDENTITY_KEY_SENTINEL: u64 = 0x5452_5553_544b_4944; // "TRUSTKID"
    holder_key_digest(pk).unwrap_or_else(|_| Fr::from(IDENTITY_KEY_SENTINEL))
}

/// Embed an IRI as a field element via the estate's domain-separated `encode_term` (IRIs
/// are salt-independent there). No new hashing primitive; mirrors `delegation::iri_to_field`.
fn iri_to_field(n: &NamedNode) -> Fr {
    let salt = salt_from_bytes(&[0u8; 32]);
    encode_term(&Term::NamedNode(n.clone()), &salt)
        .expect("a NamedNode always encodes (only RDF-1.2 triple terms return None)")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::{BlankNode, Literal, NamedNode, Triple};
    use sparq_zk::sig::{public_key_to_hex, SecretKey};

    fn iri(s: &str) -> NamedNode {
        NamedNode::new(s).unwrap()
    }

    /// A deterministic key pair for a source, by seed.
    fn keypair(seed: u64) -> (SecretKey, PublicKey) {
        let sk = SecretKey::from_seed(seed);
        let pk = sk.public_key();
        (sk, pk)
    }

    /// A predicate-scoped shape (`trust:forPredicate P` desugared): a single-predicate
    /// SHACL node-shape targeting subjects of `p` and requiring `p` present.
    fn predicate_shape(p: &str) -> ShapeRef {
        let root = BlankNode::default();
        let prop = BlankNode::default();
        let sh_target_subjects_of = iri("http://www.w3.org/ns/shacl#targetSubjectsOf");
        let sh_property = iri("http://www.w3.org/ns/shacl#property");
        let sh_path = iri("http://www.w3.org/ns/shacl#path");
        let sh_mincount = iri("http://www.w3.org/ns/shacl#minCount");
        let pred = iri(p);
        ShapeRef {
            root: Term::BlankNode(root.clone()),
            triples: vec![
                Triple::new(root.clone(), sh_target_subjects_of, pred.clone()),
                Triple::new(root.clone(), sh_property, prop.clone()),
                Triple::new(prop.clone(), sh_path, pred),
                Triple::new(
                    prop,
                    sh_mincount,
                    Literal::new_simple_literal("1"),
                ),
            ],
        }
    }

    /// An anchor rule for `source`/`key` over `scope`, for statement-type `shape`.
    fn anchor(source: &str, key: PublicKey, shape: ShapeRef, scope: &str, fresh: i64) -> TrustRule {
        TrustRule {
            source: iri(source),
            issuer_key: key,
            shape,
            scope: iri(scope),
            fresh_within_secs: fresh,
        }
    }

    /// Build a signed certification edge, signed by `certifier_sk` over the message.
    fn signed_cert(
        certifier: &str,
        certifier_sk: &SecretKey,
        certified: &str,
        certified_key: PublicKey,
        scope: CertScope,
        valid_from: i64,
        valid_until: i64,
    ) -> Certification {
        let mut cert = Certification {
            certifier: iri(certifier),
            certifier_key: certifier_sk.public_key(),
            certified_issuer: iri(certified),
            certified_key,
            scope,
            valid_from_unix_secs: valid_from,
            valid_until_unix_secs: valid_until,
            signature_hex: String::new(),
        };
        // Sign the domain-separated message via sign_commitment (the same wrap the gate
        // verifies against).
        cert.signature_hex = certifier_sk.sign_commitment(&certification_message(&cert));
        cert
    }

    // ── Positive end-to-end case ────────────────────────────────────────────

    #[test]
    fn positive_certification_narrows_and_admits() {
        let (gov_sk, gov_pk) = keypair(1);
        let (_iss_sk, iss_pk) = keypair(2);
        let anchor_rule = anchor(
            "https://gov.example/framework",
            gov_pk,
            predicate_shape("https://schema.org/age"),
            "https://pod.ex/resourceX",
            30 * 86400,
        );
        let cert = signed_cert(
            "https://gov.example/framework",
            &gov_sk,
            "https://issuer.example/dvs",
            iss_pk,
            CertScope::AnyService,
            0,
            i64::MAX / 2,
        );
        let effective =
            derive_effective_rules(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&cert), 1_000, 1);
        assert_eq!(effective.len(), 2, "anchor + one derived rule");
        // The anchor is first, verbatim.
        assert_eq!(effective[0].source.as_str(), anchor_rule.source.as_str());
        // The derived rule authorises the CERTIFIED issuer with ITS key, over the anchor's
        // scope + shape (AnyService inherits the anchor shape), no fresher than the anchor.
        let derived = &effective[1];
        assert_eq!(derived.source.as_str(), "https://issuer.example/dvs");
        assert_eq!(
            public_key_to_hex(&derived.issuer_key),
            public_key_to_hex(&iss_pk)
        );
        assert_eq!(derived.scope.as_str(), "https://pod.ex/resourceX");
        assert!(
            derived.fresh_within_secs <= anchor_rule.fresh_within_secs,
            "attenuation: derived freshness never exceeds the anchor's"
        );
    }

    #[test]
    fn positive_edge_explains_as_admitted() {
        let (gov_sk, gov_pk) = keypair(1);
        let (_iss_sk, iss_pk) = keypair(2);
        let anchor_rule = anchor(
            "https://gov.example/framework",
            gov_pk,
            predicate_shape("https://schema.org/age"),
            "https://pod.ex/resourceX",
            30 * 86400,
        );
        let cert = signed_cert(
            "https://gov.example/framework",
            &gov_sk,
            "https://issuer.example/dvs",
            iss_pk,
            CertScope::AnyService,
            0,
            i64::MAX / 2,
        );
        let derived = explain_edge(&[anchor_rule], &cert, 1_000, 1)
            .expect("a well-formed edge is admitted");
        assert_eq!(derived.source.as_str(), "https://issuer.example/dvs");
    }

    // ── Strict additivity ───────────────────────────────────────────────────

    #[test]
    fn certification_message_is_deterministic_and_scope_sensitive() {
        // A DIRECT test of the public `certification_message`: it is deterministic for a
        // fixed edge, and CHANGES when the scope changes (so scope tampering breaks the sig).
        let (_g, gov_pk) = keypair(1);
        let (_i, iss_pk) = keypair(2);
        let base = Certification {
            certifier: iri(GOV_URL),
            certifier_key: gov_pk,
            certified_issuer: iri("https://iss.ex"),
            certified_key: iss_pk,
            scope: CertScope::AnyService,
            valid_from_unix_secs: 0,
            valid_until_unix_secs: 100,
            signature_hex: String::new(),
        };
        assert_eq!(
            certification_message(&base),
            certification_message(&base),
            "deterministic for a fixed edge"
        );
        let mut widened = base.clone();
        widened.scope = CertScope::Shape(predicate_shape("https://schema.org/age"));
        assert_ne!(
            certification_message(&base),
            certification_message(&widened),
            "the scope is folded into the message — a scope change alters it"
        );
    }

    const GOV_URL: &str = "https://gov.example/framework";

    #[test]
    fn zero_certifications_is_direct_rules_verbatim() {
        let (_sk, pk) = keypair(1);
        let rules = vec![
            anchor("https://a.ex", pk, predicate_shape("https://schema.org/age"), "https://pod.ex/", 100),
            anchor("https://b.ex", pk, predicate_shape("https://schema.org/name"), "https://pod.ex/x", 200),
        ];
        let out = derive_effective_rules(&rules, &[], 1_000, 1);
        assert_eq!(out.len(), rules.len());
        for (a, b) in rules.iter().zip(out.iter()) {
            assert_eq!(a.source.as_str(), b.source.as_str());
            assert_eq!(a.scope.as_str(), b.scope.as_str());
            assert_eq!(a.fresh_within_secs, b.fresh_within_secs);
            assert_eq!(public_key_to_hex(&a.issuer_key), public_key_to_hex(&b.issuer_key));
        }
    }

    #[test]
    fn depth_zero_short_circuits_to_direct_rules() {
        let (gov_sk, gov_pk) = keypair(1);
        let (_iss_sk, iss_pk) = keypair(2);
        let anchor_rule = anchor("https://gov.ex", gov_pk, predicate_shape("https://schema.org/age"), "https://pod.ex/", 100);
        let cert = signed_cert("https://gov.ex", &gov_sk, "https://iss.ex", iss_pk, CertScope::AnyService, 0, i64::MAX / 2);
        let out = derive_effective_rules(&[anchor_rule], std::slice::from_ref(&cert), 1_000, 0);
        assert_eq!(out.len(), 1, "depth 0 ⇒ no derivation");
        // And explain_edge reports OverDepth at depth 0. (`TrustRule` has no `PartialEq`,
        // so match on the Err variant rather than assert_eq! on the whole Result.)
        let (_g, gov_pk2) = keypair(1);
        let anchor2 = anchor("https://gov.ex", gov_pk2, predicate_shape("https://schema.org/age"), "https://pod.ex/", 100);
        assert!(matches!(
            explain_edge(&[anchor2], &cert, 1_000, 0),
            Err(EdgeRejection::OverDepth)
        ));
    }

    // ── DEPTH-N closure (sq-13096) ──────────────────────────────────────────

    /// A REGISTRAR shape: `predicate_shape(age)` PLUS a root `sh:targetSubjectsOf
    /// trustx:certifies` selection edge. It selects both age statements AND certification
    /// statements, so a rule carrying it passes the meta-scope gate and may act as an edge
    /// source at a deeper hop — unlike a plain attribute shape.
    fn registrar_shape(p: &str) -> ShapeRef {
        additive_target(p, TRUSTX_CERTIFIES)
    }

    /// A three-node framework-of-frameworks chain `GOV → MID → LEAF`, all `AnyService`,
    /// all in-window, each hop signed by the previous node's key. GOV is anchored with a
    /// REGISTRAR shape, so `AnyService` passes that certification authority down to MID and
    /// MID may extend the chain. Returns the GOV anchor and the two edges in chain order.
    fn two_hop_chain() -> (TrustRule, Certification, Certification) {
        let (gov_sk, gov_pk) = keypair(1);
        let (mid_sk, mid_pk) = keypair(2);
        let (_leaf_sk, leaf_pk) = keypair(3);
        let anchor_rule = anchor(
            GOV_URL,
            gov_pk,
            registrar_shape("https://schema.org/age"),
            "https://pod.ex/resourceX",
            30 * 86400,
        );
        let hop1 = signed_cert(
            GOV_URL,
            &gov_sk,
            "https://mid.ex",
            mid_pk,
            CertScope::AnyService,
            0,
            i64::MAX / 2,
        );
        let hop2 = signed_cert(
            "https://mid.ex",
            &mid_sk,
            "https://leaf.ex",
            leaf_pk,
            CertScope::AnyService,
            0,
            i64::MAX / 2,
        );
        (anchor_rule, hop1, hop2)
    }

    #[test]
    fn depth_two_derives_the_second_hop_depth_one_does_not() {
        let (anchor_rule, hop1, hop2) = two_hop_chain();
        let anchors = [anchor_rule];
        let certs = [hop1, hop2];
        // depth 1: only MID derives — the tail is left underived (the v1 behaviour, intact).
        let d1 = derive_effective_rules(&anchors, &certs, 1_000, 1);
        assert_eq!(d1.len(), 2, "depth 1 ⇒ anchor + the FIRST hop only");
        assert!(d1.iter().all(|r| r.source.as_str() != "https://leaf.ex"));
        // depth 2: MID (hop 1) then LEAF (hop 2, anchored on the DERIVED MID rule).
        let d2 = derive_effective_rules(&anchors, &certs, 1_000, 2);
        assert_eq!(d2.len(), 3, "depth 2 ⇒ anchor + both hops");
        assert_eq!(d2[1].source.as_str(), "https://mid.ex", "shallowest hop first");
        assert_eq!(d2[2].source.as_str(), "https://leaf.ex");
        // Transitive attenuation: every hop stays inside the ROOT anchor's ceiling.
        assert_eq!(d2[2].scope.as_str(), d2[0].scope.as_str(), "resource scope is the root ceiling");
        assert!(d2[2].fresh_within_secs <= d2[1].fresh_within_secs);
        assert!(d2[1].fresh_within_secs <= d2[0].fresh_within_secs);
    }

    #[test]
    fn depth_beyond_the_chain_length_adds_nothing_and_terminates() {
        // A generous depth_bound must not invent rules past the end of the chain: the
        // closure reaches its fixpoint at hop 2 and stops.
        let (anchor_rule, hop1, hop2) = two_hop_chain();
        let anchors = [anchor_rule];
        let certs = [hop1, hop2];
        let deep = derive_effective_rules(&anchors, &certs, 1_000, 64);
        assert_eq!(deep.len(), 3, "no rule beyond the chain's own length");
        // The no-amplification bound: derived count <= edge count, whatever the depth.
        assert!(deep.len() - anchors.len() <= certs.len());
    }

    #[test]
    fn cross_round_cycle_derives_nothing_at_any_depth() {
        // `GOV → MID` and `MID → GOV`. Hop 1 derives MID. Hop 2 would close the loop back
        // onto GOV — which already holds a matching-key rule — so the CYCLE gate (GATE 1,
        // reading the whole closure-so-far, not just the direct anchors) denies it and
        // NOTHING new is derived, at any depth.
        let (gov_sk, gov_pk) = keypair(1);
        let (mid_sk, mid_pk) = keypair(2);
        let anchor_rule = anchor(
            GOV_URL,
            gov_pk,
            registrar_shape("https://schema.org/age"),
            "https://pod.ex/resourceX",
            30 * 86400,
        );
        let forward = signed_cert(GOV_URL, &gov_sk, "https://mid.ex", mid_pk, CertScope::AnyService, 0, i64::MAX / 2);
        let back = signed_cert("https://mid.ex", &mid_sk, GOV_URL, gov_pk, CertScope::AnyService, 0, i64::MAX / 2);
        for depth in [1u32, 2, 3, 8] {
            let out = derive_effective_rules(std::slice::from_ref(&anchor_rule), &[forward.clone(), back.clone()], 1_000, depth);
            assert_eq!(out.len(), 2, "depth {}: anchor + MID only — the cycle adds nothing", depth);
            assert_eq!(out[1].source.as_str(), "https://mid.ex");
        }
    }

    /// DIRECT unit coverage of the meta-scope predicate: only a shape that SELECTS
    /// certification statements confers the authority to certify.
    #[test]
    fn confers_certification_authority_only_for_certification_selection() {
        // An attribute-only shape is an ISSUER scope, not a REGISTRAR scope.
        assert!(!confers_certification_authority(&predicate_shape("https://schema.org/age")));
        // A registrar shape selects `trustx:certifies` subjects ⇒ may certify.
        assert!(confers_certification_authority(&registrar_shape("https://schema.org/age")));
        // `sh:targetObjectsOf trustx:certifies` and `sh:targetClass trustx:Certification`
        // are the other two accepted selection forms.
        for (pred, obj) in [
            ("http://www.w3.org/ns/shacl#targetObjectsOf", TRUSTX_CERTIFIES),
            ("http://www.w3.org/ns/shacl#targetClass", TRUSTX_CERTIFICATION),
        ] {
            let mut s = predicate_shape("https://schema.org/age");
            if let Term::BlankNode(root) = s.root.clone() {
                s.triples.push(Triple::new(root, iri(pred), iri(obj)));
            }
            assert!(confers_certification_authority(&s), "{} {} confers", pred, obj);
        }
        // Fail-closed on every near miss: the term in a CONFORMANCE position selects nothing;
        // the wrong object under a certification-capable predicate proves nothing; and
        // `sh:targetNode` carries no proof that certification statements are in scope.
        let mut conformance_only = predicate_shape("https://schema.org/age");
        if let Term::BlankNode(prop) = conformance_only.triples[1].object.clone() {
            conformance_only.triples.push(Triple::new(
                prop,
                iri("http://www.w3.org/ns/shacl#path"),
                iri(TRUSTX_CERTIFIES),
            ));
        }
        assert!(
            !confers_certification_authority(&conformance_only),
            "the term in a conformance position selects nothing ⇒ no certification authority"
        );
        assert!(!confers_certification_authority(&additive_target(
            "https://schema.org/age",
            TRUSTX_CERTIFICATION // targetSubjectsOf the CLASS is not the `certifies` predicate
        )));
        let mut target_node = predicate_shape("https://schema.org/age");
        if let Term::BlankNode(root) = target_node.root.clone() {
            target_node.triples.push(Triple::new(
                root,
                iri("http://www.w3.org/ns/shacl#targetNode"),
                iri(TRUSTX_CERTIFIES),
            ));
        }
        assert!(!confers_certification_authority(&target_node));
    }

    /// THE META-SCOPE ESCALATION, end-to-end. GOV is anchored ONLY for `age` — an issuer
    /// scope, not a registrar scope. GOV legitimately certifies MID for `age`. MID then
    /// certifies LEAF for `age` — perfectly INSIDE MID's own derived shape, so every
    /// attenuation gate passes. It must STILL be denied: MID was certified to ISSUE age
    /// statements, never to CONFER age authority on anyone else, and admitting it would let a
    /// chain of ordinary issuers launder pod authority to an issuer GOV never vouched for.
    /// (`research/trust-graph-authorisation-2026-07.md` §2.4 rule 3, §2.5.)
    #[test]
    fn an_attribute_scoped_derived_rule_cannot_certify_at_a_deeper_hop() {
        let (gov_sk, gov_pk) = keypair(1);
        let (mid_sk, mid_pk) = keypair(2);
        let (_leaf_sk, leaf_pk) = keypair(3);
        // NOTE the anchor shape: attribute-only, NOT `registrar_shape`.
        let anchor_rule = anchor(
            GOV_URL,
            gov_pk,
            predicate_shape("https://schema.org/age"),
            "https://pod.ex/resourceX",
            30 * 86400,
        );
        let hop1 = signed_cert(GOV_URL, &gov_sk, "https://mid.ex", mid_pk, CertScope::AnyService, 0, i64::MAX / 2);
        // Strictly INSIDE MID's derived `age` shape — nothing but the meta-scope gate denies it.
        let hop2 = signed_cert(
            "https://mid.ex",
            &mid_sk,
            "https://leaf.ex",
            leaf_pk,
            CertScope::AnyService,
            0,
            i64::MAX / 2,
        );
        for depth in [2u32, 3, 16] {
            let out = derive_effective_rules(
                std::slice::from_ref(&anchor_rule),
                &[hop1.clone(), hop2.clone()],
                1_000,
                depth,
            );
            assert_eq!(out.len(), 2, "depth {}: MID derives, LEAF must NOT", depth);
            assert!(out.iter().all(|r| r.source.as_str() != "https://leaf.ex"));
        }
        // Control: with a REGISTRAR anchor — MID's certification scope now covers
        // certification-issuing — the very same hop 2 IS admitted. So the gate denies on
        // meta-scope, not on some unrelated defect of the fixture.
        let registrar_anchor = anchor(
            GOV_URL,
            gov_pk,
            registrar_shape("https://schema.org/age"),
            "https://pod.ex/resourceX",
            30 * 86400,
        );
        let out = derive_effective_rules(&[registrar_anchor], &[hop1, hop2], 1_000, 2);
        assert_eq!(out.len(), 3, "a registrar-scoped chain still admits LEAF");
        assert_eq!(out[2].source.as_str(), "https://leaf.ex");
    }

    #[test]
    fn a_broadening_second_hop_is_denied_and_kills_the_tail() {
        // GOV is anchored as a REGISTRAR for `age`; GOV → MID (AnyService) so MID inherits it.
        // MID then tries to certify LEAF for `name` — outside MID's OWN derived ceiling. That
        // hop is a broadening and must contribute nothing even though hop 1 was legitimate.
        let (gov_sk, gov_pk) = keypair(1);
        let (mid_sk, mid_pk) = keypair(2);
        let (_leaf_sk, leaf_pk) = keypair(3);
        let anchor_rule = anchor(
            GOV_URL,
            gov_pk,
            registrar_shape("https://schema.org/age"),
            "https://pod.ex/resourceX",
            30 * 86400,
        );
        let hop1 = signed_cert(GOV_URL, &gov_sk, "https://mid.ex", mid_pk, CertScope::AnyService, 0, i64::MAX / 2);
        let hop2 = signed_cert(
            "https://mid.ex",
            &mid_sk,
            "https://leaf.ex",
            leaf_pk,
            CertScope::Shape(predicate_shape("https://schema.org/name")),
            0,
            i64::MAX / 2,
        );
        let out = derive_effective_rules(std::slice::from_ref(&anchor_rule), &[hop1.clone(), hop2.clone()], 1_000, 4);
        assert_eq!(out.len(), 2, "the broadening 2nd hop derives nothing at any depth");
        assert!(out.iter().all(|r| r.source.as_str() != "https://leaf.ex"));
        // And the explained path names the gate, run against hop 1's own output.
        let hop1_out = derive_effective_rules(std::slice::from_ref(&anchor_rule), std::slice::from_ref(&hop1), 1_000, 1);
        assert!(matches!(
            explain_edge(&hop1_out, &hop2, 1_000, 2),
            Err(EdgeRejection::Broadening)
        ));
    }

    // ── DIRECT unit tests: target-set contravariance (sq-pfae.15 review fix) ──────

    /// `predicate_shape(p)` with an extra `root sh:targetSubjectsOf q` — an injective
    /// structural superset that WIDENS the target set (a broadening).
    fn additive_target(p: &str, extra: &str) -> ShapeRef {
        let mut s = predicate_shape(p);
        if let Term::BlankNode(root) = s.root.clone() {
            s.triples.push(Triple::new(
                root,
                iri("http://www.w3.org/ns/shacl#targetSubjectsOf"),
                iri(extra),
            ));
        }
        s
    }

    #[test]
    fn is_selection_predicate_recognises_all_sh_target_star() {
        for p in [
            "http://www.w3.org/ns/shacl#targetSubjectsOf",
            "http://www.w3.org/ns/shacl#targetObjectsOf",
            "http://www.w3.org/ns/shacl#targetNode",
            "http://www.w3.org/ns/shacl#targetClass",
            "http://www.w3.org/ns/shacl#targetWhere",
            "http://www.w3.org/ns/shacl#targetFooBar", // un-modelled `sh:target*` ⇒ still selection
        ] {
            assert!(is_selection_predicate(p), "{} must be a selection predicate", p);
        }
        // Conformance predicates are NOT selection.
        for p in [
            "http://www.w3.org/ns/shacl#property",
            "http://www.w3.org/ns/shacl#path",
            "http://www.w3.org/ns/shacl#minCount",
            "http://www.w3.org/ns/shacl#datatype",
        ] {
            assert!(!is_selection_predicate(p), "{} must NOT be a selection predicate", p);
        }
    }

    #[test]
    fn is_selection_edge_flags_implicit_class_typing_not_nodeshape() {
        let rdfs_class = Term::NamedNode(iri("http://www.w3.org/2000/01/rdf-schema#Class"));
        let shape_class = Term::NamedNode(iri("http://www.w3.org/ns/shacl#ShapeClass"));
        let node_shape = Term::NamedNode(iri("http://www.w3.org/ns/shacl#NodeShape"));
        // `rdf:type rdfs:Class` / `sh:ShapeClass` at the root is a class target (selection).
        assert!(is_selection_edge(RDF_TYPE, &rdfs_class));
        assert!(is_selection_edge(RDF_TYPE, &shape_class));
        // `rdf:type sh:NodeShape` selects nothing — a conformance edge, not selection.
        assert!(!is_selection_edge(RDF_TYPE, &node_shape));
    }

    #[test]
    fn selection_edges_collects_only_root_targets() {
        // The desugared `age` shape declares exactly one root target (`targetSubjectsOf age`);
        // its `sh:property` / `sh:path` / `sh:minCount` are conformance, not selection.
        let sel = selection_edges(&predicate_shape("https://schema.org/age"));
        assert_eq!(sel.len(), 1, "exactly one root selection edge");
        assert!(sel.iter().any(|(p, o)| p == "http://www.w3.org/ns/shacl#targetSubjectsOf"
            && o == "Ihttps://schema.org/age"));
    }

    #[test]
    fn selection_only_shrinks_rejects_additive_target_accepts_narrowing() {
        let anchor = predicate_shape("https://schema.org/age");
        // A genuine narrowing (same target, extra conformance) — selection is IDENTICAL ⇒ ok.
        let mut narrowing = anchor.clone();
        let prop = narrowing.triples[1].object.clone();
        if let Term::BlankNode(bn) = prop {
            narrowing.triples.push(Triple::new(
                bn,
                iri("http://www.w3.org/ns/shacl#datatype"),
                iri("http://www.w3.org/2001/XMLSchema#integer"),
            ));
        }
        assert!(selection_only_shrinks(&narrowing, &anchor), "same target set ⇒ selection shrinks (=)");
        // An additive target (age + email) ⇒ selection GROWS ⇒ rejected.
        let broadening = additive_target("https://schema.org/age", "https://schema.org/email");
        assert!(
            !selection_only_shrinks(&broadening, &anchor),
            "an extra target predicate widens the selection set ⇒ rejected"
        );
    }

    // ── Polarity (monotonicity) fail-closed gate — sq-wh546 ─────────────────

    /// A shape targeting subjects of `target` whose conformance contains a NEGATION:
    /// `root sh:not [ sh:property [ sh:path schema:age ; sh:minCount 1 ] ]` — i.e. "a
    /// conforming node must NOT have a `schema:age`". Returns the inner property blank
    /// node so a test can add a constraint INSIDE the negation (an anti-monotone
    /// position: adding a constraint there BROADENS the admitted set).
    fn sh_not_age_shape(target: &str) -> (ShapeRef, BlankNode) {
        let root = BlankNode::default();
        let not_node = BlankNode::default();
        let inner_prop = BlankNode::default();
        let shape = ShapeRef {
            root: Term::BlankNode(root.clone()),
            triples: vec![
                Triple::new(
                    root.clone(),
                    iri("http://www.w3.org/ns/shacl#targetSubjectsOf"),
                    iri(target),
                ),
                Triple::new(root, iri("http://www.w3.org/ns/shacl#not"), not_node.clone()),
                Triple::new(
                    not_node,
                    iri("http://www.w3.org/ns/shacl#property"),
                    inner_prop.clone(),
                ),
                Triple::new(
                    inner_prop.clone(),
                    iri("http://www.w3.org/ns/shacl#path"),
                    iri("https://schema.org/age"),
                ),
                Triple::new(
                    inner_prop.clone(),
                    iri("http://www.w3.org/ns/shacl#minCount"),
                    Literal::new_simple_literal("1"),
                ),
            ],
        };
        (shape, inner_prop)
    }

    /// The sq-wh546 escalation, end-to-end: a cert shape that adds `sh:datatype
    /// xsd:string` INSIDE the anchor's `sh:not[...]` is an injective structural
    /// SUPERSET of the anchor — but SEMANTICALLY BROADER (the extra constraint sits in
    /// an anti-monotone position: it makes the negated inner shape HARDER to satisfy,
    /// so `sh:not` admits MORE nodes). The pre-fix polarity-blind containment gate
    /// reported it as a narrowing and `explain_edge` derived a rule from it (verified
    /// as the failing reproduction before the fix); it must reject as `Broadening`.
    #[test]
    fn broadening_cert_inside_sh_not_is_rejected_fail_closed() {
        let (anchor_shape, inner_prop) = sh_not_age_shape("https://schema.org/name");
        // The cert shape = the anchor shape + `sh:datatype xsd:string` INSIDE the sh:not.
        let mut cert_shape = anchor_shape.clone();
        cert_shape.triples.push(Triple::new(
            inner_prop,
            iri("http://www.w3.org/ns/shacl#datatype"),
            iri("http://www.w3.org/2001/XMLSchema#string"),
        ));

        // SEMANTIC ground truth via the real SHACL validator: a node with an
        // INTEGER-typed age is REJECTED by the anchor ("must not have an age" — it has
        // one) but ADMITTED by the cert ("must not have a STRING age" — its age is an
        // integer, the negated inner shape fails, so sh:not is satisfied). The cert
        // therefore admits a node the certifier's anchor rejects ⇒ strictly BROADER.
        let node = iri("https://pod.ex/people/n");
        let data = sparq_shacl::graph_from_triples([
            Triple::new(node.clone(), iri("https://schema.org/name"), Literal::new_simple_literal("Bob")),
            Triple::new(
                node,
                iri("https://schema.org/age"),
                Literal::new_typed_literal("42", iri("http://www.w3.org/2001/XMLSchema#integer")),
            ),
        ]);
        let anchor_graph = sparq_shacl::graph_from_triples(anchor_shape.triples.iter().cloned());
        let cert_graph = sparq_shacl::graph_from_triples(cert_shape.triples.iter().cloned());
        assert!(
            !sparq_shacl::validate(&data, &anchor_graph).conforms,
            "ground truth: the ANCHOR rejects the integer-aged node"
        );
        assert!(
            sparq_shacl::validate(&data, &cert_graph).conforms,
            "ground truth: the CERT admits the very node the anchor rejects ⇒ the cert is BROADER"
        );

        // THE FIX (fail-closed polarity gate): the shape contains `sh:not` — a
        // non-monotone constraint — so the injective-superset argument is unsound and the
        // matcher must refuse to certify narrowing at every layer…
        assert!(
            !shape_narrows(&cert_shape, &anchor_shape),
            "polarity gate: a cert whose closure contains sh:not must NEVER be reported as narrowing"
        );
        assert!(
            narrowed_shape(&anchor_shape, &CertScope::Shape(cert_shape.clone())).is_none(),
            "polarity gate: narrowed_shape must fail closed on a non-monotone cert scope"
        );
        // …and end-to-end the edge is rejected as Broadening: no derived rule, no
        // escalation (a signed, in-window, anchored cert — ONLY the shape gate denies it).
        let (gov_sk, gov_pk) = keypair(1);
        let (_iss_sk, iss_pk) = keypair(2);
        let anchor_rule = anchor(GOV_URL, gov_pk, anchor_shape, "https://pod.ex/resourceX", 30 * 86400);
        let cert = signed_cert(
            GOV_URL,
            &gov_sk,
            "https://issuer.example/dvs",
            iss_pk,
            CertScope::Shape(cert_shape),
            0,
            i64::MAX / 2,
        );
        assert!(
            matches!(
                explain_edge(std::slice::from_ref(&anchor_rule), &cert, 1_000, 1),
                Err(EdgeRejection::Broadening)
            ),
            "the broadening cert must be rejected end-to-end as Broadening"
        );
        // And the fast path derives NOTHING from it (anchor passes through verbatim).
        let effective =
            derive_effective_rules(&[anchor_rule], std::slice::from_ref(&cert), 1_000, 1);
        assert_eq!(effective.len(), 1, "the broadening edge contributes nothing");
    }

    /// The polarity gate must not OVER-block: a genuine monotone narrowing — the same
    /// selection, plus an extra `sh:datatype` in a purely-CONJUNCTIVE position — is still
    /// admitted end-to-end (the attenuation the module exists to provide).
    #[test]
    fn monotone_conjunctive_narrowing_still_admitted_end_to_end() {
        let anchor_shape = predicate_shape("https://schema.org/age");
        // Same target; add `sh:datatype xsd:integer` on the (conjunctive) property shape.
        let mut cert_shape = anchor_shape.clone();
        if let Term::BlankNode(prop) = cert_shape.triples[1].object.clone() {
            cert_shape.triples.push(Triple::new(
                prop,
                iri("http://www.w3.org/ns/shacl#datatype"),
                iri("http://www.w3.org/2001/XMLSchema#integer"),
            ));
        }
        // Unit level: the polarity gate accepts the purely-conjunctive closures and the
        // narrowing is still proven.
        assert!(monotone_conjunctive_only(&anchor_shape));
        assert!(monotone_conjunctive_only(&cert_shape));
        assert!(
            shape_narrows(&cert_shape, &anchor_shape),
            "a purely-conjunctive datatype refinement still narrows (no over-blocking)"
        );
        // End-to-end: the edge is admitted and the derived shape is the (tighter) cert's.
        let (gov_sk, gov_pk) = keypair(1);
        let (_iss_sk, iss_pk) = keypair(2);
        let anchor_rule = anchor(GOV_URL, gov_pk, anchor_shape, "https://pod.ex/resourceX", 30 * 86400);
        let cert = signed_cert(
            GOV_URL,
            &gov_sk,
            "https://issuer.example/dvs",
            iss_pk,
            CertScope::Shape(cert_shape.clone()),
            0,
            i64::MAX / 2,
        );
        let derived = explain_edge(&[anchor_rule], &cert, 1_000, 1)
            .expect("a genuine monotone narrowing is still admitted");
        assert_eq!(derived.source.as_str(), "https://issuer.example/dvs");
        assert_eq!(
            derived.shape.triples.len(),
            cert_shape.triples.len(),
            "the derived statement-type is the tighter cert shape"
        );
    }

    /// DIRECT unit coverage of the polarity gate's predicate classification: every
    /// non-monotone SHACL component the crate's validator supports — and any unknown
    /// `sh:`/non-`sh:` constraint predicate — poisons the closure; the conjunctive
    /// allowlist plus selection/typing edges pass.
    #[test]
    fn monotone_gate_rejects_every_nonmonotone_and_unknown_predicate() {
        let base = predicate_shape("https://schema.org/age");
        assert!(monotone_conjunctive_only(&base), "the minted conjunctive idiom passes");
        for bad in [
            "http://www.w3.org/ns/shacl#not",
            "http://www.w3.org/ns/shacl#or",
            "http://www.w3.org/ns/shacl#xone",
            "http://www.w3.org/ns/shacl#maxCount",
            "http://www.w3.org/ns/shacl#qualifiedValueShape",
            "http://www.w3.org/ns/shacl#qualifiedMaxCount",
            "http://www.w3.org/ns/shacl#in",          // closed enumeration: adding an entry BROADENS
            "http://www.w3.org/ns/shacl#futureTerm",  // unknown sh: term ⇒ no proof ⇒ reject
            "https://example.org/customConstraint",   // custom-component parameter ⇒ reject
        ] {
            let mut s = base.clone();
            if let Term::BlankNode(root) = s.root.clone() {
                s.triples
                    .push(Triple::new(root, iri(bad), Term::BlankNode(BlankNode::default())));
            }
            assert!(
                !monotone_conjunctive_only(&s),
                "{bad} must poison the monotonicity proof (fail-closed)"
            );
            // And the poisoned shape can never be certified as narrowing, in EITHER role.
            assert!(!shape_narrows(&s, &base), "{bad}: poisoned cert must not narrow");
            assert!(!shape_narrows(&base, &s), "{bad}: poisoned anchor must not be narrowed");
        }
        // `rdf:type sh:NodeShape` stays inert (no false rejection of plain shape typing).
        let mut typed = base.clone();
        if let Term::BlankNode(root) = typed.root.clone() {
            typed.triples.push(Triple::new(
                root,
                iri(RDF_TYPE),
                iri("http://www.w3.org/ns/shacl#NodeShape"),
            ));
        }
        assert!(monotone_conjunctive_only(&typed));
    }

    #[test]
    fn shape_narrows_is_contravariant_on_targets() {
        let anchor = predicate_shape("https://schema.org/age");
        // Narrowing (add a datatype conformance constraint) ⇒ narrows.
        let mut narrower = anchor.clone();
        if let Term::BlankNode(bn) = narrower.triples[1].object.clone() {
            narrower.triples.push(Triple::new(
                bn,
                iri("http://www.w3.org/ns/shacl#datatype"),
                iri("http://www.w3.org/2001/XMLSchema#integer"),
            ));
        }
        assert!(shape_narrows(&narrower, &anchor), "extra conformance constraint ⇒ narrows");
        // Additive target ⇒ does NOT narrow (the escalation the review found).
        let broadening = additive_target("https://schema.org/age", "https://schema.org/email");
        assert!(
            !shape_narrows(&broadening, &anchor),
            "additive target predicate is a broadening, not a narrowing"
        );
        // AnyService path: narrowed_shape returns the anchor shape unchanged.
        assert!(narrowed_shape(&anchor, &CertScope::AnyService).is_some());
        // A Shape scope that adds a target ⇒ narrowed_shape yields None (fail-closed).
        assert!(narrowed_shape(&anchor, &CertScope::Shape(broadening)).is_none());
    }
}
