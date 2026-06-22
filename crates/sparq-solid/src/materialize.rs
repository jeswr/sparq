//! The materializer pipeline: facts (loader) + N3 rules (rules/*.n3) → `reason_n3`
//! strata → the auth-view named graph `<urn:sparq:auth>` swapped into the dataset.
//!
//! Stratification (design doc §3.5): the engine's `log:notIncludes` never retracts, so
//! each negated predicate must be COMPLETE before its stratum runs. WAC negates only
//! input facts → 1 call. ACP: accepts (A) → rejections (B) → grants (C) → 3 calls,
//! each seeded with the previous closure.

use crate::loader::{assemble_input, strip_reserved_graphs, System};
use crate::{AccessProvenance, AUTH_GRAPH, AUTH_NS};
use oxrdf::{NamedNode, Term};
use rustc_hash::FxHashSet;
use sparq_core::dict::Dict;
use sparq_core::Graph;
use sparq_reason::reason_n3;
use std::fmt::Write as _;
// `std::time::Instant` is unusable on `wasm32-unknown-unknown` — `Instant::now()`
// panics there (no monotonic clock). The wall-clock plumbing for `stats.millis` is
// purely informational, so it is `cfg`-gated off and reported as `0.0` on wasm32
// rather than trapping at runtime (sq-7agop). [OPUS-4.8]
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

const COMMON_RULES: &str = include_str!("../rules/common.n3");
const WAC_RULES: &str = include_str!("../rules/wac.n3");
const ACP_A: &str = include_str!("../rules/acp-a.n3");
const ACP_B: &str = include_str!("../rules/acp-b.n3");
const ACP_C: &str = include_str!("../rules/acp-c.n3");

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const EXCEPT_MATCHER: &str = "https://sparq.dev/ns/auth#exceptMatcher";
/// solidx facts copied into the auth view for matchers referenced by conditional
/// grants (the session layer evaluates `exceptMatcher`s from these). [OPUS-4.8]
/// sq-3jtd.6 adds `acceptsIssuerP` — the noneOf exception check is three-dimensional.
const MATCHER_FACTS: [&str; 3] = [
    "https://sparq.dev/ns/solidx#acceptsAgentP",
    "https://sparq.dev/ns/solidx#acceptsClientP",
    "https://sparq.dev/ns/solidx#acceptsIssuerP",
];

/// What a `materialize_*` run produced: auth-view size, per-stratum closure sizes,
/// and wall-clock time. Purely informational (logging / benchmarking) — the result
/// itself is the `<urn:sparq:auth>` graph installed into the dataset.
///
/// # Examples
///
/// ```no_run
/// let mut graph = sparq_core::Graph::load_dataset(&sparq_solid::wac_fixture(), "nquads")?;
/// let stats = sparq_solid::materialize_wac(&mut graph)?;
/// println!("{} auth triples, closure {:?}, {:.0} ms",
///          stats.auth_triples, stats.strata_facts, stats.millis);
/// # Ok::<(), String>(())
/// ```
#[derive(Debug, Default)]
pub struct MaterializeStats {
    /// Triples in the produced auth view.
    pub auth_triples: usize,
    /// Closure size after each reasoning stratum (1 entry for WAC, 3 for ACP).
    pub strata_facts: Vec<usize>,
    /// Wall-clock total.
    pub millis: f64,
}

/// Materialize the WAC auth view (single stratum) and install it as `<urn:sparq:auth>`.
///
/// The free-function form of [`crate::PodStore::materialize_wac`], for callers
/// managing a [`Graph`] directly: strips reserved `urn:sparq:` graphs, assembles the
/// reasoning input (`.acl` graphs + group documents + structural facts — pod content
/// is **never** fed to the reasoner), runs `rules/common.n3` + `rules/wac.n3`
/// through `sparq_reason::reason_n3`, and swaps the filtered closure in as the
/// `<urn:sparq:auth>` named graph (replacing any previous view).
///
/// Note: a [`crate::PodStore`] does NOT observe direct calls on its `graph` field —
/// use the method form so the session index and cache are rebuilt.
///
/// # Errors
///
/// Returns `Err` if an agent / group-member / origin IRI collides with the reserved
/// principal encoding (starts with `urn:sparq:` or contains the literal `&client=`),
/// or if the reasoner rejects the assembled N3 source. The dataset's previous auth
/// view (if any) is left untouched on error.
///
/// # Examples
///
/// ```
/// use oxrdf::Term;
///
/// let nquads = r#"
/// <https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
/// <https://pod.ex/.acl#owner> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <https://pod.ex/.acl> .
/// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#default> <https://pod.ex/> <https://pod.ex/.acl> .
/// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#agent> <https://alice.ex/card#me> <https://pod.ex/.acl> .
/// <https://pod.ex/.acl#owner> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acl> .
/// "#;
/// let mut graph = sparq_core::Graph::load_dataset(nquads, "nquads")?;
/// let stats = sparq_solid::materialize_wac(&mut graph)?;
/// assert!(stats.auth_triples > 0);
/// // the view is an ordinary named graph now
/// assert!(graph.named.iter().any(|(name, _)| matches!(
///     name, Term::NamedNode(n) if n.as_str() == sparq_solid::AUTH_GRAPH)));
/// # Ok::<(), String>(())
/// ```
pub fn materialize_wac(graph: &mut Graph) -> Result<MaterializeStats, String> {
    #[cfg(not(target_arch = "wasm32"))]
    let t0 = Instant::now();
    strip_reserved_graphs(graph);
    // WAC has no creator/owner vocabulary; provenance is ignored (the loader skips it).
    let input = assemble_input(graph, System::Wac, &AccessProvenance::new())?;
    let src = format!("{input}\n{COMMON_RULES}\n{WAC_RULES}");
    let mut dict = Dict::new();
    let closure = reason_n3(&mut dict, &src)?;
    let mut stats = install_auth_view(graph, &dict, &closure);
    stats.strata_facts = vec![closure.len()];
    stats.millis = elapsed_millis(
        #[cfg(not(target_arch = "wasm32"))]
        t0,
    );
    Ok(stats)
}

/// Materialize the ACP auth view (three strata) and install it as `<urn:sparq:auth>`.
///
/// The free-function form of [`crate::PodStore::materialize_acp`]. Same contract as
/// [`materialize_wac`], but the input graphs are the `.acr` ones and the rules run
/// as three stratified `reason_n3` calls (`rules/acp-a.n3` accepts →
/// `rules/acp-b.n3` rejections → `rules/acp-c.n3` grants), each seeded with the
/// previous closure — the engine's negation-as-failure never retracts, so each
/// negated predicate must be complete before its stratum runs (design doc §3.5).
///
/// # Errors
///
/// As [`materialize_wac`].
///
/// # Examples
///
/// ```
/// // One document; an ACR on the pod root grants alice Read on all member resources.
/// let nquads = r#"
/// <https://pod.ex/notes/n1#it> <https://ex.dev/ns#title> "hello" <https://pod.ex/notes/n1> .
/// <https://pod.ex/.acr> <http://www.w3.org/ns/solid/acp#memberAccessControl> <https://pod.ex/.acr#c> <https://pod.ex/.acr> .
/// <https://pod.ex/.acr#c> <http://www.w3.org/ns/solid/acp#apply> <https://pod.ex/.acr#pol> <https://pod.ex/.acr> .
/// <https://pod.ex/.acr#pol> <http://www.w3.org/ns/solid/acp#allow> <http://www.w3.org/ns/auth/acl#Read> <https://pod.ex/.acr> .
/// <https://pod.ex/.acr#pol> <http://www.w3.org/ns/solid/acp#allOf> <https://pod.ex/.acr#m> <https://pod.ex/.acr> .
/// <https://pod.ex/.acr#m> <http://www.w3.org/ns/solid/acp#agent> <https://alice.ex/card#me> <https://pod.ex/.acr> .
/// "#;
/// let mut graph = sparq_core::Graph::load_dataset(nquads, "nquads")?;
/// let stats = sparq_solid::materialize_acp(&mut graph)?;
/// assert_eq!(stats.strata_facts.len(), 3); // accepts → rejections → grants
///
/// let index = sparq_solid::AuthIndex::from_graph(&graph);
/// let alice = sparq_solid::Session { agent: Some("https://alice.ex/card#me"), client: None, issuer: None, now: None };
/// assert_eq!(index.accessible(&alice, sparq_solid::Mode::Read).len(), 2); // n1 + notes/
/// # Ok::<(), String>(())
/// ```
pub fn materialize_acp(graph: &mut Graph) -> Result<MaterializeStats, String> {
    materialize_acp_with(graph, &AccessProvenance::new())
}

/// Materialize the ACP auth view from the `.acr` graphs PLUS the TRUSTED per-resource
/// creator/owner facts in `provenance`, resolving `acp:CreatorAgent` / `acp:OwnerAgent`
/// matchers ([OPUS-4.8] sq-3jtd.5).
///
/// The free-function form of [`crate::PodStore::materialize_acp_with`]. Identical to
/// [`materialize_acp`] (three stratified `reason_n3` calls) except the loader also
/// synthesizes `<r> solidx:creator|owner <webid>` facts from `provenance` — the trusted
/// channel for "who created/owns `<r>`". These facts are **never** read from the resource
/// graphs (design doc §2.4): a writer cannot self-grant via a forged `solidx:creator`
/// triple in a document they control.
///
/// `materialize_acp(graph)` is exactly `materialize_acp_with(graph, &AccessProvenance::new())`
/// — with no provenance, no `CreatorAgent`/`OwnerAgent` matcher ever grants (fail-closed).
///
/// # Errors
///
/// As [`materialize_acp`], and additionally if a creator/owner WebID collides with the
/// reserved principal encoding (starts with `urn:sparq:` or contains the literal
/// `&client=`).
pub fn materialize_acp_with(
    graph: &mut Graph,
    provenance: &AccessProvenance,
) -> Result<MaterializeStats, String> {
    #[cfg(not(target_arch = "wasm32"))]
    let t0 = Instant::now();
    strip_reserved_graphs(graph);
    let input = assemble_input(graph, System::Acp, provenance)?;
    let mut strata_facts = Vec::new();

    let mut dict = Dict::new();
    let c1 = reason_n3(&mut dict, &format!("{input}\n{COMMON_RULES}\n{ACP_A}"))?;
    strata_facts.push(c1.len());
    let f1 = closure_to_n3(&dict, &c1);

    let mut d2 = Dict::new();
    let c2 = reason_n3(&mut d2, &format!("{f1}\n{ACP_B}"))?;
    strata_facts.push(c2.len());
    let f2 = closure_to_n3(&d2, &c2);

    let mut d3 = Dict::new();
    let c3 = reason_n3(&mut d3, &format!("{f2}\n{ACP_C}"))?;
    strata_facts.push(c3.len());

    let mut stats = install_auth_view(graph, &d3, &c3);
    stats.strata_facts = strata_facts;
    stats.millis = elapsed_millis(
        #[cfg(not(target_arch = "wasm32"))]
        t0,
    );
    Ok(stats)
}

/// Wall-clock milliseconds since `t0`, or `0.0` on `wasm32-unknown-unknown` where
/// `std::time::Instant` is unavailable (sq-7agop). `stats.millis` is purely
/// informational (logging / benchmarking), so wasm32 simply reports no timing
/// rather than trapping. [OPUS-4.8]
#[cfg(not(target_arch = "wasm32"))]
fn elapsed_millis(t0: Instant) -> f64 {
    t0.elapsed().as_secs_f64() * 1e3
}

#[cfg(target_arch = "wasm32")]
fn elapsed_millis() -> f64 {
    0.0
}

/// Re-serialize a stratum's ground closure as facts for the next stratum.
fn closure_to_n3(dict: &Dict, closure: &[[sparq_core::dict::Id; 3]]) -> String {
    let mut out = String::with_capacity(closure.len() * 64);
    for t in closure {
        let _ = writeln!(out, "{} {} {} .", dict.term(t[0]), dict.term(t[1]), dict.term(t[2]));
    }
    out
}

/// Whether a closure triple belongs in the auth view: `auth:*` predicates, `rdf:type`
/// with an `auth:` class, plus the accept-set facts of matchers referenced by
/// `auth:exceptMatcher` (collected in a first pass).
fn install_auth_view(graph: &mut Graph, dict: &Dict, closure: &[[sparq_core::dict::Id; 3]]) -> MaterializeStats {
    // pass 1: matchers referenced by conditional grants
    let mut except_matchers: FxHashSet<sparq_core::dict::Id> = FxHashSet::default();
    for t in closure {
        if let Term::NamedNode(p) = dict.term(t[1]) {
            if p.as_str() == EXCEPT_MATCHER {
                except_matchers.insert(t[2]);
            }
        }
    }
    // pass 2: filter + re-intern into a fresh sub-graph dictionary
    let mut adict = Dict::new();
    let mut ids: Vec<[sparq_core::dict::Id; 3]> = Vec::new();
    for t in closure {
        let Term::NamedNode(p) = dict.term(t[1]) else { continue };
        let keep = p.as_str().starts_with(AUTH_NS)
            || (p.as_str() == RDF_TYPE
                && matches!(dict.term(t[2]), Term::NamedNode(o) if o.as_str().starts_with(AUTH_NS)))
            || (MATCHER_FACTS.contains(&p.as_str()) && except_matchers.contains(&t[0]));
        // exclude the rules' own mode-mapping constants (solidx:allowPred facts have a
        // solidx predicate and are filtered out by the conditions above already)
        if !keep {
            continue;
        }
        let s = dict.term(t[0]);
        let o = dict.term(t[2]);
        ids.push([adict.intern(&s), adict.intern(&Term::NamedNode(p)), adict.intern(&o)]);
    }
    let stats = MaterializeStats { auth_triples: ids.len(), ..Default::default() };
    let auth = Graph::from_parts(adict, ids);
    let name = Term::NamedNode(NamedNode::new_unchecked(AUTH_GRAPH));
    if let Some(slot) = graph.named.iter_mut().find(|(n, _)| *n == name) {
        slot.1 = auth;
    } else {
        graph.named.push((name, auth));
    }
    stats
}
