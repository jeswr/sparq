// AUTHORED-BY Claude Opus 4.8
//! Web Access Control rule-matching: turn a parsed `.acl` graph into the set of [`AccessMode`]s
//! granted to a requester for a given resource + inheritance scope.
//!
//! The ACL document is parsed via `oxttl`/`oxjsonld` into `oxrdf::Triple`s (the house rule — NEVER
//! hand-parse/concat ACL by string) and matched here against the `acl:` vocabulary. This is the
//! semantic port of prod-solid-server `src/authz/acl.ts`.
//!
//! A rule (an `acl:Authorization`) grants a requester a mode when:
//!  - the rule's scope predicate (`acl:accessTo` for an own ACL, `acl:default`/`acl:defaultForNew`
//!    for an inherited ancestor ACL) references the target resource, AND
//!  - the rule matches the requester — by `acl:agent <webid>`, by `acl:agentClass foaf:Agent`
//!    (public — everyone, incl. anonymous), or by `acl:agentClass acl:AuthenticatedAgent` (any
//!    authenticated WebID), AND
//!  - the rule lists the mode via `acl:mode`.
//!
//! `acl:agentGroup` is recognised but NEVER matches in v1 (group-membership resolution is a
//! follow-up) — fail-closed, exactly as prod-solid-server.

use std::collections::BTreeSet;

use oxrdf::{NamedOrBlankNode, Term, Triple};

use super::mode::AccessMode;

const ACL: &str = "http://www.w3.org/ns/auth/acl#";

fn acl_iri(local: &str) -> String {
    format!("{ACL}{local}")
}

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const FOAF_AGENT: &str = "http://xmlns.com/foaf/0.1/Agent";

/// Which scope of a rule applies: a rule for the resource itself (`acl:accessTo`), or one inherited
/// from an ancestor container (`acl:default`, plus the legacy `acl:defaultForNew`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AclScope {
    /// The ACL is the resource's OWN ACL: only `acl:accessTo <resource>` rules apply.
    AccessTo,
    /// The ACL belongs to an ancestor container: only `acl:default <container>` (or the legacy
    /// `acl:defaultForNew`) rules apply (WAC inheritance).
    Default,
}

/// The verified requester identity as the matcher needs it.
#[derive(Debug, Clone, Default)]
pub struct Requester<'a> {
    /// The requester's WebID, or `None` for an anonymous/public request.
    pub web_id: Option<&'a str>,
    /// The request's HTTP `Origin` header (the requesting web app's origin), or `None` when the
    /// request carried no `Origin` (e.g. a non-browser / same-origin / server-to-server request).
    ///
    /// WAC's `acl:origin` restriction is matched against THIS value: a rule that lists one or more
    /// `acl:origin` values grants ONLY when this origin matches one of them. A request with NO
    /// `Origin` can never satisfy an `acl:origin`-restricted rule (fail-closed) — but it is fully
    /// unaffected by a rule that carries no `acl:origin` (the common case).
    pub origin: Option<&'a str>,
}

impl<'a> Requester<'a> {
    pub fn anonymous() -> Self {
        Self {
            web_id: None,
            origin: None,
        }
    }
    pub fn authenticated(web_id: &'a str) -> Self {
        Self {
            web_id: Some(web_id),
            origin: None,
        }
    }
    fn is_authenticated(&self) -> bool {
        self.web_id.is_some()
    }
}

/// Compute the set of access modes the `.acl` graph (`triples`) grants to `requester` for `resource`
/// under the inheritance `scope`.
///
/// Returns an empty set when no rule matches (fail-closed). A malformed `acl:mode` object (e.g. a
/// literal where a NamedNode is expected) is IGNORED, never fatal — ACL documents are user-controlled,
/// so a single bad triple must not deny a whole, otherwise-valid rule or crash authorization.
pub fn modes_for(
    triples: &[Triple],
    resource: &str,
    requester: &Requester<'_>,
    scope: AclScope,
) -> BTreeSet<AccessMode> {
    let mut granted = BTreeSet::new();

    for rule in authorization_subjects(triples) {
        if !applies_to_resource(triples, &rule, resource, scope) {
            continue;
        }
        if !matches_agent(triples, &rule, requester) {
            continue;
        }
        // `acl:origin` restriction (app-scoping): a rule with one or more `acl:origin` values grants
        // ONLY when the request's Origin matches one of them. A rule with NO `acl:origin` applies
        // regardless of origin (the common case). Checked AFTER the agent match — both must hold.
        if !matches_origin(triples, &rule, requester) {
            continue;
        }
        for mode in granted_modes(triples, &rule) {
            granted.insert(mode);
        }
    }
    granted
}

/// Whether `granted` satisfies the `required` mode. WAC's `acl:Write` subsumes `acl:Append` (a writer
/// may also append), so an `Append` requirement is met by either an explicit `Append` or a `Write`
/// grant. No other implications hold — `Control` does NOT imply Read/Write of the resource body.
pub fn satisfies(granted: &BTreeSet<AccessMode>, required: AccessMode) -> bool {
    if granted.contains(&required) {
        return true;
    }
    required == AccessMode::Append && granted.contains(&AccessMode::Write)
}

/// The subjects of every `?s a acl:Authorization` triple in the graph (the authorization rules).
fn authorization_subjects(triples: &[Triple]) -> Vec<NamedOrBlankNode> {
    let authorization = acl_iri("Authorization");
    let mut subjects: Vec<NamedOrBlankNode> = Vec::new();
    for t in triples {
        if t.predicate.as_str() == RDF_TYPE {
            if let Term::NamedNode(obj) = &t.object {
                if obj.as_str() == authorization && !subjects.contains(&t.subject) {
                    subjects.push(t.subject.clone());
                }
            }
        }
    }
    subjects
}

/// All NamedNode objects of `(subject, predicate)` in the graph.
fn named_objects<'a>(
    triples: &'a [Triple],
    subject: &NamedOrBlankNode,
    predicate: &str,
) -> Vec<&'a str> {
    let mut out = Vec::new();
    for t in triples {
        if &t.subject == subject && t.predicate.as_str() == predicate {
            if let Term::NamedNode(obj) = &t.object {
                out.push(obj.as_str());
            }
        }
    }
    out
}

/// Whether a rule's scope predicate references `resource`. WAC permits an authorization to list
/// MULTIPLE `acl:accessTo`/`acl:default` targets, so every object of the scope predicate is checked.
fn applies_to_resource(
    triples: &[Triple],
    rule: &NamedOrBlankNode,
    resource: &str,
    scope: AclScope,
) -> bool {
    let predicates: &[String] = &match scope {
        AclScope::AccessTo => vec![acl_iri("accessTo")],
        AclScope::Default => vec![acl_iri("default"), acl_iri("defaultForNew")],
    };
    for predicate in predicates {
        for obj in named_objects(triples, rule, predicate) {
            if obj == resource {
                return true;
            }
        }
    }
    false
}

/// Whether the rule grants access to the requester (by exact WebID, the public class, or the
/// authenticated class). `acl:agentGroup` intentionally NEVER matches in v1 (fail-closed).
fn matches_agent(triples: &[Triple], rule: &NamedOrBlankNode, requester: &Requester<'_>) -> bool {
    let agent_class = acl_iri("agentClass");
    let authenticated_agent = acl_iri("AuthenticatedAgent");

    // `acl:agentClass foaf:Agent` — public, matches every requester (authenticated or not).
    // `acl:agentClass acl:AuthenticatedAgent` — matches any authenticated WebID.
    for class in named_objects(triples, rule, &agent_class) {
        if class == FOAF_AGENT {
            return true;
        }
        if class == authenticated_agent && requester.is_authenticated() {
            return true;
        }
    }

    // `acl:agent <webid>` — matches the requester's exact WebID.
    if let Some(web_id) = requester.web_id {
        let agent = acl_iri("agent");
        for a in named_objects(triples, rule, &agent) {
            if a == web_id {
                return true;
            }
        }
    }

    false
}

/// Whether the rule's `acl:origin` restriction (if any) is satisfied by the request's Origin.
///
/// WAC `acl:origin` (the trusted-app restriction): a rule that lists one or more `acl:origin` values
/// applies ONLY to a request whose `Origin` header EXACTLY matches one of those values; a rule with
/// NO `acl:origin` value applies to ANY origin (the common, unrestricted case). A request that
/// carried no `Origin` (anonymous-of-origin: a non-browser, same-origin, or server-to-server request)
/// can NEVER satisfy an origin-restricted rule (fail-closed) — otherwise an app-scoped authorization
/// would be valid from any caller that simply omits the header.
fn matches_origin(triples: &[Triple], rule: &NamedOrBlankNode, requester: &Requester<'_>) -> bool {
    let origin_pred = acl_iri("origin");
    let allowed = named_objects(triples, rule, &origin_pred);
    if allowed.is_empty() {
        // Unrestricted rule — applies regardless of the request's Origin.
        return true;
    }
    // Origin-restricted: the request MUST carry an Origin that exactly matches a listed value.
    match requester.origin {
        Some(origin) => allowed.contains(&origin),
        None => false,
    }
}

/// The modes a rule lists via `acl:mode`. A non-NamedNode object is ignored (defensive — ACLs are
/// user-controlled), and an unrecognised mode IRI contributes nothing.
fn granted_modes(triples: &[Triple], rule: &NamedOrBlankNode) -> Vec<AccessMode> {
    let mode = acl_iri("mode");
    let mut modes = Vec::new();
    for iri in named_objects(triples, rule, &mode) {
        if let Some(m) = AccessMode::from_acl_iri(iri) {
            modes.push(m);
        }
    }
    modes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ldp::content::{parse_to_triples, RdfFormat};

    const RES: &str = "https://pod.example/alice/test/data";
    const CONTAINER: &str = "https://pod.example/alice/test/";
    const ALICE: &str = "https://pod.example/alice/profile/card#me";
    const BOB: &str = "https://pod.example/bob/profile/card#me";

    fn parse(ttl: &str) -> Vec<Triple> {
        parse_to_triples(
            RdfFormat::Turtle,
            ttl.as_bytes(),
            "https://pod.example/alice/test/.acl",
        )
        .expect("valid acl turtle")
    }

    fn modes(
        t: &[Triple],
        resource: &str,
        web_id: Option<&str>,
        scope: AclScope,
    ) -> BTreeSet<AccessMode> {
        let r = Requester {
            web_id,
            origin: None,
        };
        modes_for(t, resource, &r, scope)
    }

    /// Like [`modes`] but with an explicit request `Origin` — for the `acl:origin` tests.
    fn modes_with_origin(
        t: &[Triple],
        resource: &str,
        web_id: Option<&str>,
        origin: Option<&str>,
        scope: AclScope,
    ) -> BTreeSet<AccessMode> {
        let r = Requester { web_id, origin };
        modes_for(t, resource, &r, scope)
    }

    #[test]
    fn agent_access_to_grants_only_that_agent() {
        let ttl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
            <#bob> a acl:Authorization;
                   acl:agent <{BOB}>;
                   acl:accessTo <{RES}>;
                   acl:mode acl:Read."#
        );
        let t = parse(&ttl);
        // Bob gets read on the resource via its OWN acl.
        assert!(modes(&t, RES, Some(BOB), AclScope::AccessTo).contains(&AccessMode::Read));
        // Alice (a different agent) gets nothing.
        assert!(modes(&t, RES, Some(ALICE), AclScope::AccessTo).is_empty());
        // Anonymous gets nothing.
        assert!(modes(&t, RES, None, AclScope::AccessTo).is_empty());
        // Under the DEFAULT scope this accessTo rule does NOT apply.
        assert!(modes(&t, RES, Some(BOB), AclScope::Default).is_empty());
    }

    #[test]
    fn public_foaf_agent_grants_everyone_including_anonymous() {
        let ttl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
            @prefix foaf: <http://xmlns.com/foaf/0.1/>.
            <#pub> a acl:Authorization;
                   acl:agentClass foaf:Agent;
                   acl:accessTo <{RES}>;
                   acl:mode acl:Read."#
        );
        let t = parse(&ttl);
        assert!(modes(&t, RES, None, AclScope::AccessTo).contains(&AccessMode::Read));
        assert!(modes(&t, RES, Some(BOB), AclScope::AccessTo).contains(&AccessMode::Read));
    }

    #[test]
    fn authenticated_agent_grants_any_authenticated_but_not_anonymous() {
        let ttl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
            <#auth> a acl:Authorization;
                    acl:agentClass acl:AuthenticatedAgent;
                    acl:accessTo <{RES}>;
                    acl:mode acl:Write."#
        );
        let t = parse(&ttl);
        assert!(modes(&t, RES, Some(BOB), AclScope::AccessTo).contains(&AccessMode::Write));
        assert!(modes(&t, RES, None, AclScope::AccessTo).is_empty());
    }

    #[test]
    fn default_scope_only_matches_acl_default() {
        let ttl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
            <#bobdef> a acl:Authorization;
                      acl:agent <{BOB}>;
                      acl:default <{CONTAINER}>;
                      acl:mode acl:Read."#
        );
        let t = parse(&ttl);
        // The default rule grants Bob read under DEFAULT scope on the container.
        assert!(modes(&t, CONTAINER, Some(BOB), AclScope::Default).contains(&AccessMode::Read));
        // Under accessTo scope (the container's OWN acl), a default-only rule does NOT apply.
        assert!(modes(&t, CONTAINER, Some(BOB), AclScope::AccessTo).is_empty());
    }

    #[test]
    fn all_four_modes_are_recognised() {
        let ttl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
            <#full> a acl:Authorization;
                    acl:agent <{ALICE}>;
                    acl:accessTo <{RES}>;
                    acl:mode acl:Read, acl:Write, acl:Append, acl:Control."#
        );
        let t = parse(&ttl);
        let m = modes(&t, RES, Some(ALICE), AclScope::AccessTo);
        assert!(m.contains(&AccessMode::Read));
        assert!(m.contains(&AccessMode::Write));
        assert!(m.contains(&AccessMode::Append));
        assert!(m.contains(&AccessMode::Control));
    }

    #[test]
    fn agent_group_never_matches_v1_fail_closed() {
        let ttl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
            <#grp> a acl:Authorization;
                   acl:agentGroup <https://pod.example/groups#team>;
                   acl:accessTo <{RES}>;
                   acl:mode acl:Read."#
        );
        let t = parse(&ttl);
        // A member named via agentGroup is NOT granted (group resolution is a follow-up; fail-closed).
        assert!(modes(&t, RES, Some(BOB), AclScope::AccessTo).is_empty());
    }

    #[test]
    fn rule_for_a_different_resource_does_not_apply() {
        let other = "https://pod.example/alice/test/other";
        let ttl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
            <#x> a acl:Authorization;
                 acl:agent <{BOB}>;
                 acl:accessTo <{other}>;
                 acl:mode acl:Read."#
        );
        let t = parse(&ttl);
        assert!(modes(&t, RES, Some(BOB), AclScope::AccessTo).is_empty());
        assert!(modes(&t, other, Some(BOB), AclScope::AccessTo).contains(&AccessMode::Read));
    }

    #[test]
    fn satisfies_write_subsumes_append() {
        let mut g = BTreeSet::new();
        g.insert(AccessMode::Write);
        assert!(satisfies(&g, AccessMode::Append));
        assert!(satisfies(&g, AccessMode::Write));
        assert!(!satisfies(&g, AccessMode::Read));
        // Control does NOT imply read/write.
        let mut c = BTreeSet::new();
        c.insert(AccessMode::Control);
        assert!(!satisfies(&c, AccessMode::Read));
        assert!(!satisfies(&c, AccessMode::Write));
    }

    #[test]
    fn malformed_mode_is_ignored_not_fatal() {
        // A literal `acl:mode` object (malformed) is skipped; the valid mode still grants.
        let ttl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
            <#x> a acl:Authorization;
                 acl:agent <{BOB}>;
                 acl:accessTo <{RES}>;
                 acl:mode "not-a-mode";
                 acl:mode acl:Read."#
        );
        let t = parse(&ttl);
        let m = modes(&t, RES, Some(BOB), AclScope::AccessTo);
        assert_eq!(m.len(), 1);
        assert!(m.contains(&AccessMode::Read));
    }

    #[test]
    fn multiple_access_to_targets_on_one_rule() {
        let other = "https://pod.example/alice/test/other";
        let ttl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
            <#x> a acl:Authorization;
                 acl:agent <{BOB}>;
                 acl:accessTo <{RES}>, <{other}>;
                 acl:mode acl:Read."#
        );
        let t = parse(&ttl);
        assert!(modes(&t, RES, Some(BOB), AclScope::AccessTo).contains(&AccessMode::Read));
        assert!(modes(&t, other, Some(BOB), AclScope::AccessTo).contains(&AccessMode::Read));
    }

    // --- acl:origin (app-scoping) -------------------------------------------------------------

    const APP: &str = "https://app.example";
    const OTHER_APP: &str = "https://evil.example";

    fn origin_restricted_acl() -> Vec<Triple> {
        // Bob is granted Read on RES, but ONLY from the app at https://app.example.
        let ttl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
            <#bob> a acl:Authorization;
                   acl:agent <{BOB}>;
                   acl:origin <{APP}>;
                   acl:accessTo <{RES}>;
                   acl:mode acl:Read."#
        );
        parse(&ttl)
    }

    #[test]
    fn origin_restricted_rule_grants_only_from_matching_origin() {
        let t = origin_restricted_acl();
        // Matching Origin → granted.
        assert!(
            modes_with_origin(&t, RES, Some(BOB), Some(APP), AclScope::AccessTo)
                .contains(&AccessMode::Read)
        );
    }

    #[test]
    fn origin_restricted_rule_denies_from_other_origin() {
        let t = origin_restricted_acl();
        // A DIFFERENT Origin → not granted (the over-grant the HIGH finding is about).
        assert!(
            modes_with_origin(&t, RES, Some(BOB), Some(OTHER_APP), AclScope::AccessTo).is_empty()
        );
    }

    #[test]
    fn origin_restricted_rule_denies_when_origin_absent() {
        let t = origin_restricted_acl();
        // No Origin header at all → an origin-restricted rule must NOT match (fail-closed): a
        // non-browser/server-to-server caller cannot bypass an app restriction by omitting Origin.
        assert!(modes_with_origin(&t, RES, Some(BOB), None, AclScope::AccessTo).is_empty());
        // The legacy `modes` helper (no origin) must likewise deny an origin-restricted rule.
        assert!(modes(&t, RES, Some(BOB), AclScope::AccessTo).is_empty());
    }

    #[test]
    fn rule_without_acl_origin_grants_from_any_origin() {
        // The common case: a rule with NO acl:origin applies regardless of the request Origin.
        let ttl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
            <#bob> a acl:Authorization;
                   acl:agent <{BOB}>;
                   acl:accessTo <{RES}>;
                   acl:mode acl:Read."#
        );
        let t = parse(&ttl);
        // From a specific Origin.
        assert!(
            modes_with_origin(&t, RES, Some(BOB), Some(APP), AclScope::AccessTo)
                .contains(&AccessMode::Read)
        );
        // From a different Origin.
        assert!(
            modes_with_origin(&t, RES, Some(BOB), Some(OTHER_APP), AclScope::AccessTo)
                .contains(&AccessMode::Read)
        );
        // With no Origin at all.
        assert!(
            modes_with_origin(&t, RES, Some(BOB), None, AclScope::AccessTo)
                .contains(&AccessMode::Read)
        );
    }

    #[test]
    fn multiple_acl_origin_values_any_one_matches() {
        // A rule listing several acl:origin values grants from ANY of them, denies from outside the set.
        let app2 = "https://app2.example";
        let ttl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
            <#bob> a acl:Authorization;
                   acl:agent <{BOB}>;
                   acl:origin <{APP}>, <{app2}>;
                   acl:accessTo <{RES}>;
                   acl:mode acl:Read."#
        );
        let t = parse(&ttl);
        assert!(
            modes_with_origin(&t, RES, Some(BOB), Some(APP), AclScope::AccessTo)
                .contains(&AccessMode::Read)
        );
        assert!(
            modes_with_origin(&t, RES, Some(BOB), Some(app2), AclScope::AccessTo)
                .contains(&AccessMode::Read)
        );
        assert!(
            modes_with_origin(&t, RES, Some(BOB), Some(OTHER_APP), AclScope::AccessTo).is_empty()
        );
    }

    #[test]
    fn origin_restriction_also_applies_to_public_class_rules() {
        // An `acl:agentClass foaf:Agent` (public) rule that ALSO carries acl:origin is app-scoped: it
        // grants the public ONLY from the trusted origin, never from another origin or no Origin.
        let ttl = format!(
            r#"@prefix acl: <http://www.w3.org/ns/auth/acl#>.
            @prefix foaf: <http://xmlns.com/foaf/0.1/>.
            <#pub> a acl:Authorization;
                   acl:agentClass foaf:Agent;
                   acl:origin <{APP}>;
                   acl:accessTo <{RES}>;
                   acl:mode acl:Read."#
        );
        let t = parse(&ttl);
        assert!(
            modes_with_origin(&t, RES, None, Some(APP), AclScope::AccessTo)
                .contains(&AccessMode::Read)
        );
        assert!(modes_with_origin(&t, RES, None, Some(OTHER_APP), AclScope::AccessTo).is_empty());
        assert!(modes_with_origin(&t, RES, None, None, AclScope::AccessTo).is_empty());
    }
}
