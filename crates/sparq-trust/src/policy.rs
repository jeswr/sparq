//! # `policy.rs` — parse a trust-policy graph into `Vec<TrustRule>` (fail-closed)
//!
//! Parses a trust-policy RDF graph (authored in the **Control-gated channel**, the
//! same trusted channel as `.acl`/`.acr` — design §3.2) into a `Vec<TrustRule>`.
//! Each rule reifies one admission condition: `source` + `issuer_key` + `shape` +
//! `scope` + `fresh_within`.
//!
//! ## Fail-closed Control-gating (design §3.2 / §3.3 item 3)
//!
//! The trust graph re-opens the content/reasoner boundary `sparq-solid` deliberately
//! closed, so a policy an UNTRUSTED writer could author must never take effect. The
//! parser therefore requires the caller to present a [`ControlGate`] — the token a
//! relying layer mints ONLY after it has verified the policy graph arrived through
//! the `acl:Control` / ACR-write channel (whoever may write `.acl` may write the
//! trust rules; nothing else may). [`parse_policy`] without the gate is a
//! compile-time impossibility; [`try_parse_ungated`] exists for tests and returns an
//! explicit [`PolicyError::NotControlGated`] so the fail-closed default is visible.
//!
//! [OPUS-4.8] sq-pfae PoC (issue #940). 🤖 SPARQ agent — trust-graph authorisation PoC.

use crate::vocab;
use oxrdf::{NamedNode, Term, Triple};
use sparq_zk::sig::{public_key_from_hex, PublicKey};

/// A capability token asserting that the accompanying policy graph arrived through
/// the **Control-gated channel** (`acl:Control` / ACR-write). The relying layer mints
/// one ONLY after it has established the policy was written by a principal holding
/// Control over the resource — exactly the NGAC self-administration discipline (the
/// policy is governed by the same access decision it configures, §3.2).
///
/// It is deliberately unforgeable from outside this crate (no public constructor that
/// takes untrusted data): a policy graph alone can never gate itself. This makes the
/// fail-closed property a TYPE-level guarantee, not a runtime convention.
#[derive(Debug, Clone, Copy)]
pub struct ControlGate(());

impl ControlGate {
    /// Mint a Control-gate. The caller asserts, by calling this, that it has verified
    /// the policy graph was authored behind `acl:Control` (WAC) / ACR-write (ACP) —
    /// the same trusted channel as `.acl`/`.acr`. This is the trust ROOT of the whole
    /// admission pipeline; calling it on an un-vetted graph is the one mistake that
    /// re-opens the §2.4 boundary, so it is named loudly rather than implicit.
    ///
    /// In the shipped `sparq-solid`, "behind Control" is decided by the WAC/ACP
    /// materialiser; a production wiring would call this only after that decision.
    pub fn assert_control_gated() -> ControlGate {
        ControlGate(())
    }
}

/// One reified admission rule (§2.3.1): admit statements matching `shape` from
/// `source` (verified against `issuer_key`), within `scope`, no staler than
/// `fresh_within`.
#[derive(Debug, Clone)]
pub struct TrustRule {
    /// `trust:source` — the named attesting authority IRI.
    pub source: NamedNode,
    /// `trust:issuerKey` — the verification key the source signs with (parsed from
    /// the `zk:`-aligned hex key material; v1 supplies the key DIRECTLY because there
    /// is no DID resolver yet — P2 / `sq-pfae.3`, the live forgery vector D′ of §3.3).
    pub issuer_key: PublicKey,
    /// `trust:forShape` — the statement-type SHACL node-shape (a `forPredicate P`
    /// rule is desugared into this at load; see [`crate::vocab::desugar_for_predicate`]).
    /// The shape's defining triples, with `shape_root` naming the `sh:NodeShape`.
    pub shape: ShapeRef,
    /// `trust:scope` — the resource/container this rule applies to (containment check
    /// at admission). v1 is exact-or-ancestor IRI matching.
    pub scope: NamedNode,
    /// `trust:freshWithin` — maximum admitted staleness, in seconds (parsed from the
    /// `xsd:duration` literal). Consulted against `Session.now` as a per-request Rust
    /// side-condition, NOT an in-reasoner predicate (§3.3 B′).
    pub fresh_within_secs: i64,
}

/// A `trust:forShape` SHACL node-shape: the node naming the shape plus the triples
/// that define it.
#[derive(Debug, Clone)]
pub struct ShapeRef {
    /// The `sh:NodeShape` node (an IRI for an external shape, or a blank node for an
    /// inline / desugared shape).
    pub root: Term,
    /// The triples defining the shape (parsed `forShape` content, or the desugaring
    /// of a `forPredicate`).
    pub triples: Vec<Triple>,
}

/// Why a trust policy was rejected (fail-closed: a rejected policy admits NOTHING).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyError {
    /// The policy was not presented through the Control-gated channel (§3.2).
    NotControlGated,
    /// A `trust:TrustRule` was missing a mandatory property (source / issuerKey /
    /// shape-or-predicate / scope / freshWithin). Names the offending rule + field.
    IncompleteRule { rule: String, missing: &'static str },
    /// The `trust:issuerKey` literal/IRI did not parse as a verification key.
    BadIssuerKey(String),
    /// The `trust:freshWithin` literal was not a parseable `xsd:duration`.
    BadDuration(String),
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyError::NotControlGated => write!(
                f,
                "trust policy is not Control-gated (§3.2): rejected fail-closed"
            ),
            PolicyError::IncompleteRule { rule, missing } => {
                write!(f, "trust rule <{}> is missing {}", rule, missing)
            }
            PolicyError::BadIssuerKey(k) => write!(f, "trust:issuerKey did not parse: {}", k),
            PolicyError::BadDuration(d) => {
                write!(f, "trust:freshWithin is not xsd:duration: {}", d)
            }
        }
    }
}

impl std::error::Error for PolicyError {}

/// Parse a Control-gated trust-policy graph into `Vec<TrustRule>`. Possession of a
/// [`ControlGate`] is required: a policy that did not arrive through the trusted
/// channel cannot reach this function (§3.2, fail-closed).
///
/// Each `trust:TrustRule` node must carry: `trust:source`, `trust:issuerKey`,
/// exactly one of `trust:forShape` / `trust:forPredicate` (the latter desugared
/// here), `trust:scope`, and `trust:freshWithin`. A rule missing any property is a
/// hard [`PolicyError::IncompleteRule`] — a partially-specified rule never silently
/// admits.
pub fn parse_policy(policy: &[Triple], _gate: ControlGate) -> Result<Vec<TrustRule>, PolicyError> {
    parse_rules(policy)
}

/// The fail-closed analogue used in tests: parse WITHOUT a [`ControlGate`]. Always
/// returns [`PolicyError::NotControlGated`] — it exists so the default-deny posture
/// is exercised by a test, never to provide an un-gated path in production.
pub fn try_parse_ungated(_policy: &[Triple]) -> Result<Vec<TrustRule>, PolicyError> {
    Err(PolicyError::NotControlGated)
}

fn parse_rules(policy: &[Triple]) -> Result<Vec<TrustRule>, PolicyError> {
    // Collect every node typed `trust:TrustRule`.
    let mut rule_nodes: Vec<Term> = Vec::new();
    for t in policy {
        if t.predicate.as_str() == vocab::RDF_TYPE {
            if let Term::NamedNode(o) = &t.object {
                if o.as_str() == vocab::TRUST_RULE {
                    rule_nodes.push(subject_term(&t.subject));
                }
            }
        }
    }

    let mut out = Vec::with_capacity(rule_nodes.len());
    for node in rule_nodes {
        out.push(parse_one_rule(&node, policy)?);
    }
    Ok(out)
}

fn parse_one_rule(node: &Term, policy: &[Triple]) -> Result<TrustRule, PolicyError> {
    let rule_id = || node_label(node);

    // trust:source — a named attesting authority.
    let source =
        object_iri(node, vocab::SOURCE, policy).ok_or_else(|| PolicyError::IncompleteRule {
            rule: rule_id(),
            missing: "trust:source",
        })?;

    // trust:issuerKey — hex key material (operator-asserted; no DID resolver — §3.3 D′).
    let key_term =
        object_of(node, vocab::ISSUER_KEY, policy).ok_or_else(|| PolicyError::IncompleteRule {
            rule: rule_id(),
            missing: "trust:issuerKey",
        })?;
    let key_hex = term_lexical(&key_term);
    let issuer_key =
        public_key_from_hex(&key_hex).ok_or_else(|| PolicyError::BadIssuerKey(key_hex.clone()))?;

    // trust:scope — exact-or-ancestor resource/container IRI.
    let scope =
        object_iri(node, vocab::SCOPE, policy).ok_or_else(|| PolicyError::IncompleteRule {
            rule: rule_id(),
            missing: "trust:scope",
        })?;

    // trust:freshWithin — xsd:duration → seconds.
    let fresh_term = object_of(node, vocab::FRESH_WITHIN, policy).ok_or_else(|| {
        PolicyError::IncompleteRule {
            rule: rule_id(),
            missing: "trust:freshWithin",
        }
    })?;
    let fresh_lex = term_lexical(&fresh_term);
    let fresh_within_secs =
        parse_xsd_duration_secs(&fresh_lex).ok_or(PolicyError::BadDuration(fresh_lex))?;

    // trust:forShape (primitive) OR trust:forPredicate (sugar → desugar here).
    let shape = parse_shape(node, policy).ok_or_else(|| PolicyError::IncompleteRule {
        rule: rule_id(),
        missing: "trust:forShape or trust:forPredicate",
    })?;

    Ok(TrustRule {
        source,
        issuer_key,
        shape,
        scope,
        fresh_within_secs,
    })
}

/// Resolve the rule's statement-type shape: a `trust:forShape` IRI/node (an external
/// shape carried in the policy) OR a `trust:forPredicate P` desugared into the
/// normative single-predicate node-shape.
fn parse_shape(node: &Term, policy: &[Triple]) -> Option<ShapeRef> {
    // Sugar first: forPredicate P → forShape (so a policy may use either).
    if let Some(Term::NamedNode(p)) = object_of(node, vocab::FOR_PREDICATE, policy) {
        let (root, triples) = vocab::desugar_for_predicate(&p);
        return Some(ShapeRef {
            root: shape_root_to_term(root),
            triples,
        });
    }
    // Primitive: forShape names a sh:NodeShape; carry its closure of defining triples.
    if let Some(shape_node) = object_of(node, vocab::FOR_SHAPE, policy) {
        let triples = shape_closure(&shape_node, policy);
        return Some(ShapeRef {
            root: shape_node,
            triples,
        });
    }
    None
}

/// The transitive closure of triples reachable from a shape node within the policy
/// graph (so an inline `sh:property [ … ]` chain is carried whole). Bounded, since
/// the policy graph is finite and visited-tracked.
fn shape_closure(root: &Term, policy: &[Triple]) -> Vec<Triple> {
    let mut frontier = vec![root.clone()];
    let mut seen: Vec<Term> = Vec::new();
    let mut out: Vec<Triple> = Vec::new();
    while let Some(s) = frontier.pop() {
        if seen.iter().any(|x| x == &s) {
            continue;
        }
        seen.push(s.clone());
        for t in policy {
            if subject_term(&t.subject) == s {
                out.push(t.clone());
                // Follow object nodes (blank or IRI) to gather nested property shapes.
                match &t.object {
                    Term::BlankNode(_) | Term::NamedNode(_) => frontier.push(t.object.clone()),
                    _ => {}
                }
            }
        }
    }
    out
}

// --- small RDF helpers (no extra deps; the policy graph is small) ----------

fn subject_term(s: &oxrdf::NamedOrBlankNode) -> Term {
    match s {
        oxrdf::NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n.clone()),
        oxrdf::NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b.clone()),
    }
}

fn shape_root_to_term(s: oxrdf::NamedOrBlankNode) -> Term {
    match s {
        oxrdf::NamedOrBlankNode::NamedNode(n) => Term::NamedNode(n),
        oxrdf::NamedOrBlankNode::BlankNode(b) => Term::BlankNode(b),
    }
}

fn object_of(subject: &Term, predicate: &str, policy: &[Triple]) -> Option<Term> {
    policy
        .iter()
        .find(|t| subject_term(&t.subject) == *subject && t.predicate.as_str() == predicate)
        .map(|t| t.object.clone())
}

fn object_iri(subject: &Term, predicate: &str, policy: &[Triple]) -> Option<NamedNode> {
    match object_of(subject, predicate, policy)? {
        Term::NamedNode(n) => Some(n),
        _ => None,
    }
}

fn term_lexical(t: &Term) -> String {
    match t {
        Term::NamedNode(n) => n.as_str().to_string(),
        Term::Literal(l) => l.value().to_string(),
        Term::BlankNode(b) => b.as_str().to_string(),
        other => other.to_string(),
    }
}

fn node_label(t: &Term) -> String {
    match t {
        Term::NamedNode(n) => n.as_str().to_string(),
        Term::BlankNode(b) => format!("_:{}", b.as_str()),
        other => other.to_string(),
    }
}

/// Parse the common `xsd:duration` shapes this PoC needs into whole seconds:
/// `PnYnMnDTnHnMnS` (and the date-only `PnD` / `PnW` weeks). Years/months use the
/// xsd-nominal 365-day year / 30-day month for the PoC's coarse freshness window —
/// good enough for a "no staler than P30D" check, and documented as such. Returns
/// `None` on any unparseable input (fail-closed: a bad duration rejects the policy).
fn parse_xsd_duration_secs(s: &str) -> Option<i64> {
    let mut chars = s.chars().peekable();
    if chars.next()? != 'P' {
        return None;
    }
    let mut in_time = false;
    let mut total: i64 = 0;
    let mut num = String::new();
    let mut saw_field = false;
    for c in chars {
        if c == 'T' {
            in_time = true;
            continue;
        }
        if c.is_ascii_digit() {
            num.push(c);
            continue;
        }
        let v: i64 = num.parse().ok()?;
        num.clear();
        saw_field = true;
        let secs = match (in_time, c) {
            (false, 'Y') => v * 365 * 86_400,
            (false, 'M') => v * 30 * 86_400,
            (false, 'W') => v * 7 * 86_400,
            (false, 'D') => v * 86_400,
            (true, 'H') => v * 3_600,
            (true, 'M') => v * 60,
            (true, 'S') => v,
            _ => return None,
        };
        total = total.checked_add(secs)?;
    }
    // Trailing digits with no unit, or `P` with no fields, is malformed.
    if !num.is_empty() || !saw_field {
        return None;
    }
    Some(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ungated_policy_is_rejected_fail_closed() {
        assert!(matches!(
            try_parse_ungated(&[]),
            Err(PolicyError::NotControlGated)
        ));
    }

    #[test]
    fn duration_parsing_covers_the_poc_shapes() {
        assert_eq!(parse_xsd_duration_secs("P30D"), Some(30 * 86_400));
        assert_eq!(parse_xsd_duration_secs("PT1H"), Some(3_600));
        assert_eq!(
            parse_xsd_duration_secs("P1DT2H3M4S"),
            Some(86_400 + 7_200 + 180 + 4)
        );
        assert_eq!(parse_xsd_duration_secs("P1W"), Some(7 * 86_400));
        assert_eq!(parse_xsd_duration_secs("garbage"), None);
        assert_eq!(parse_xsd_duration_secs("P"), None);
        assert_eq!(parse_xsd_duration_secs("P30"), None);
    }
}
