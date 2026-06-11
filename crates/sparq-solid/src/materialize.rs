//! The materializer pipeline: facts (loader) + N3 rules (rules/*.n3) → `reason_n3`
//! strata → the auth-view named graph `<urn:sparq:auth>` swapped into the dataset.
//!
//! Stratification (design doc §3.5): the engine's `log:notIncludes` never retracts, so
//! each negated predicate must be COMPLETE before its stratum runs. WAC negates only
//! input facts → 1 call. ACP: accepts (A) → rejections (B) → grants (C) → 3 calls,
//! each seeded with the previous closure.

use crate::loader::{assemble_input, strip_reserved_graphs, System};
use crate::{AUTH_GRAPH, AUTH_NS};
use oxrdf::{NamedNode, Term};
use rustc_hash::FxHashSet;
use sparq_core::dict::Dict;
use sparq_core::Graph;
use sparq_reason::reason_n3;
use std::fmt::Write as _;
use std::time::Instant;

const COMMON_RULES: &str = include_str!("../rules/common.n3");
const WAC_RULES: &str = include_str!("../rules/wac.n3");
const ACP_A: &str = include_str!("../rules/acp-a.n3");
const ACP_B: &str = include_str!("../rules/acp-b.n3");
const ACP_C: &str = include_str!("../rules/acp-c.n3");

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const EXCEPT_MATCHER: &str = "https://sparq.dev/ns/auth#exceptMatcher";
/// solidx facts copied into the auth view for matchers referenced by conditional
/// grants (the session layer evaluates `exceptMatcher`s from these).
const MATCHER_FACTS: [&str; 2] =
    ["https://sparq.dev/ns/solidx#acceptsAgentP", "https://sparq.dev/ns/solidx#acceptsClientP"];

#[derive(Debug, Default)]
pub struct MaterializeStats {
    /// Triples in the produced auth view.
    pub auth_triples: usize,
    /// Closure size after each reasoning stratum.
    pub strata_facts: Vec<usize>,
    /// Wall-clock total.
    pub millis: f64,
}

/// Materialize the WAC auth view (single stratum) and install it as `<urn:sparq:auth>`.
pub fn materialize_wac(graph: &mut Graph) -> Result<MaterializeStats, String> {
    let t0 = Instant::now();
    strip_reserved_graphs(graph);
    let input = assemble_input(graph, System::Wac)?;
    let src = format!("{input}\n{COMMON_RULES}\n{WAC_RULES}");
    let mut dict = Dict::new();
    let closure = reason_n3(&mut dict, &src)?;
    let mut stats = install_auth_view(graph, &dict, &closure);
    stats.strata_facts = vec![closure.len()];
    stats.millis = t0.elapsed().as_secs_f64() * 1e3;
    Ok(stats)
}

/// Materialize the ACP auth view (three strata) and install it as `<urn:sparq:auth>`.
pub fn materialize_acp(graph: &mut Graph) -> Result<MaterializeStats, String> {
    let t0 = Instant::now();
    strip_reserved_graphs(graph);
    let input = assemble_input(graph, System::Acp)?;
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
    stats.millis = t0.elapsed().as_secs_f64() * 1e3;
    Ok(stats)
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
