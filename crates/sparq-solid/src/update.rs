//! [OPUS-4.8] sq-xor3: the WRITE/update path enforcement (design doc §4.4 / §7 item 6).
//!
//! The read path ([`crate::PodStore::query_as`]) restricts a query to the session's
//! `auth:read` graph set. This module is its write-path mirror: before a SPARQL Update
//! (`INSERT`/`DELETE`/`DELETE…INSERT…WHERE`, `CLEAR`/`DROP`/`CREATE`/`LOAD`) is allowed
//! to mutate the store, every graph it could write is checked against the actor's
//! WAC/ACP **write** permission — `acl:Write` (→ [`Mode::Write`]) for a delete/clear and
//! `acl:Write` OR `acl:Append` (→ [`Mode::Write`] / [`Mode::Append`]) for a pure insert.
//! The permission model is exactly the read path's: the same `∪ allow ∖ ∪ deny`
//! per-mode graph sets from the materialized auth view ([`crate::AuthIndex::accessible`]).
//!
//! `.acl`/`.acr` documents need no special mode here: the WAC/ACP rules already
//! translate `acl:Control` on a resource into `auth:write` on its `.acl`/`.acr` graph
//! (design doc §3.3 — "Control ⇒ read+write of the ACL graph itself"), so requiring
//! `Write` on the `.acl`/`.acr` graph IS requiring Control on the resource — enforced
//! through exactly the same auth view, no Solid-specific branch. Writing one DOES,
//! however, invalidate the materialized view (the rules changed), so it triggers
//! re-materialization on success.
//!
//! Fail-closed, like the read path:
//!
//! - the **default graph** is never writable (pod data never lives there — design doc
//!   §2.1); any default-graph target is denied;
//! - a write to a graph the actor cannot write in the required mode is denied and the
//!   store is **not** mutated (the check runs entirely before [`update_in_place`]);
//! - a target whose graph name cannot be determined statically — a `DELETE/INSERT`
//!   template with a `GRAPH ?var` slot, or `CLEAR`/`DROP` of `ALL`/`NAMED` graphs —
//!   is treated *conservatively*: the actor must be able to write **every** named graph
//!   currently in the store, or the whole update is denied. This is sound (it can only
//!   reject an update the precise analysis might have allowed), never permissive. The
//!   precise per-solution check for variable graph slots is a documented follow-up
//!   (see the bead created from this module).
//!
//! After a permitted update that touched an `.acl`/`.acr`/group document, the auth view
//! is automatically re-materialized (epoch bump → session cache dropped), so a changed
//! rule takes effect on the next query/update — the design doc's
//! "after any acl/acr/group-doc write, re-materialize" requirement (§4.4).

use crate::loader::{ACL_SUFFIX, ACR_SUFFIX};
use crate::{AuthIndex, Mode, Session};
use oxrdf::{NamedNode, Term};
use spargebra::algebra::GraphTarget;
use spargebra::term::{GraphName, GraphNamePattern};
use spargebra::{GraphUpdateOperation, SparqlParser, Update};

/// The write permission a single graph target requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Need {
    /// A pure-insert touch: satisfied by `acl:Write` OR `acl:Append` (WAC: Append adds
    /// without removing). Maps to "graph ∈ accessible(Write) ∪ accessible(Append)".
    WriteOrAppend,
    /// A delete / clear / drop: requires `acl:Write` (Append cannot remove). Writing an
    /// `.acl`/`.acr` graph is also a `Write` need — but the auth view only ever grants
    /// `auth:write` on those graphs to `acl:Control` holders, so this is Control-gated
    /// without a special case (see the module docs).
    Write,
}

/// What a parsed update needs in order to be permitted.
#[derive(Debug, Default)]
struct WriteReqs {
    /// Concrete (named-graph, need) requirements gathered from static targets.
    graphs: Vec<(NamedNode, Need)>,
    /// The update targets the default graph (always denied — pod data is never there).
    touches_default: bool,
    /// The update has a target whose graph cannot be determined statically (a `GRAPH
    /// ?var` template slot, or `CLEAR`/`DROP` `ALL`/`NAMED`): the actor must be able to
    /// write every named graph in the store, in this mode, or the update is denied.
    wildcard: Option<Need>,
    /// Graph names the analysis saw mentioned as write targets that are `.acl`/`.acr` or
    /// group documents — a successful update touching any of these triggers a
    /// re-materialization. (Static targets only; a wildcard always re-materializes.)
    rematerialize_hint: bool,
}

/// Whether `iri` is an access-control document (`.acl`/`.acr`) by naming convention.
fn is_control_graph(iri: &str) -> bool {
    iri.ends_with(ACL_SUFFIX) || iri.ends_with(ACR_SUFFIX)
}

/// Whether writing `iri` should trigger re-materialization: `.acl`/`.acr` documents
/// (the rules themselves). Group documents have no naming convention, so the caller
/// ALSO re-materializes on every wildcard update (which could touch a group doc), and a
/// deployment changing a group document goes through the explicit `materialize_*` path.
fn affects_auth_view(iri: &str) -> bool {
    is_control_graph(iri)
}

/// The need for a static named-graph target. Writing an `.acl`/`.acr` graph always needs
/// `Write` (Control-gated via the auth view, never satisfiable by an `Append` grant —
/// the rules grant Control-holders `auth:write` on the graph, not `auth:append`).
fn need_for_graph(iri: &str, base: Need) -> Need {
    if is_control_graph(iri) {
        Need::Write
    } else {
        base
    }
}

/// Record a static named-graph target with the appropriate need.
fn push_graph(reqs: &mut WriteReqs, n: &NamedNode, base: Need) {
    let need = need_for_graph(n.as_str(), base);
    if affects_auth_view(n.as_str()) {
        reqs.rematerialize_hint = true;
    }
    reqs.graphs.push((n.clone(), need));
}

/// Record the target of a data quad / template graph-name slot.
fn push_graph_name(reqs: &mut WriteReqs, g: &GraphName, base: Need) {
    match g {
        GraphName::DefaultGraph => reqs.touches_default = true,
        GraphName::NamedNode(n) => push_graph(reqs, n, base),
    }
}

/// Record the target of a `DELETE`/`INSERT` template quad slot (may be a variable).
fn push_graph_name_pattern(reqs: &mut WriteReqs, g: &GraphNamePattern, base: Need) {
    match g {
        GraphNamePattern::DefaultGraph => reqs.touches_default = true,
        GraphNamePattern::NamedNode(n) => push_graph(reqs, n, base),
        // A variable graph slot is only known after the WHERE evaluates: be
        // conservative and demand write on every graph in the store.
        GraphNamePattern::Variable(_) => raise_wildcard(reqs, base),
    }
}

/// Escalate to the wildcard requirement (the strongest seen wins: Control > Write >
/// WriteOrAppend).
fn raise_wildcard(reqs: &mut WriteReqs, need: Need) {
    reqs.rematerialize_hint = true; // a wildcard could touch a control doc
    reqs.wildcard = Some(match reqs.wildcard {
        None => need,
        Some(prev) => strongest(prev, need),
    });
}

fn strongest(a: Need, b: Need) -> Need {
    fn rank(n: Need) -> u8 {
        match n {
            Need::WriteOrAppend => 0,
            Need::Write => 1,
        }
    }
    if rank(a) >= rank(b) {
        a
    } else {
        b
    }
}

/// A `CLEAR`/`DROP` graph target needs `Write` (it removes triples).
fn push_clear_drop_target(reqs: &mut WriteReqs, target: &GraphTarget) {
    match target {
        GraphTarget::DefaultGraph => reqs.touches_default = true,
        GraphTarget::NamedNode(n) => push_graph(reqs, n, Need::Write),
        // NAMED / ALL touch every (named) graph — conservative wildcard at Write.
        GraphTarget::NamedGraphs | GraphTarget::AllGraphs => raise_wildcard(reqs, Need::Write),
    }
}

/// Walk a parsed update and collect everything it needs permission to write.
fn analyze(upd: &Update) -> WriteReqs {
    let mut reqs = WriteReqs::default();
    for op in &upd.operations {
        match op {
            GraphUpdateOperation::InsertData { data } => {
                for q in data {
                    push_graph_name(&mut reqs, &q.graph_name, Need::WriteOrAppend);
                }
            }
            GraphUpdateOperation::DeleteData { data } => {
                for q in data {
                    push_graph_name(&mut reqs, &q.graph_name, Need::Write);
                }
            }
            GraphUpdateOperation::DeleteInsert { delete, insert, .. } => {
                // Deletes always need Write; inserts need Write-or-Append. The WHERE
                // pattern only READS, so it is not a write target.
                for d in delete {
                    push_graph_name_pattern(&mut reqs, &d.graph_name, Need::Write);
                }
                for i in insert {
                    push_graph_name_pattern(&mut reqs, &i.graph_name, Need::WriteOrAppend);
                }
            }
            GraphUpdateOperation::Load { destination, .. } => {
                push_graph_name(&mut reqs, destination, Need::WriteOrAppend);
            }
            GraphUpdateOperation::Clear { graph, .. } | GraphUpdateOperation::Drop { graph, .. } => {
                push_clear_drop_target(&mut reqs, graph);
            }
            GraphUpdateOperation::Create { graph, .. } => {
                // Creating a named graph entry is a write to that graph.
                push_graph(&mut reqs, graph, Need::Write);
            }
        }
    }
    reqs
}

/// Does the session have `need` on the concrete graph `g`?
fn allowed(auth: &AuthIndex, s: &Session, g: &NamedNode, need: Need) -> bool {
    let has = |mode: Mode| auth.accessible(s, mode).iter().any(|x| x == g);
    match need {
        Need::Write => has(Mode::Write),
        Need::WriteOrAppend => has(Mode::Write) || has(Mode::Append),
    }
}

/// Every named graph currently in the store except the reserved auth view.
fn store_named_graphs(graph: &sparq_core::Graph) -> Vec<NamedNode> {
    graph
        .named
        .iter()
        .filter_map(|(n, _)| match n {
            Term::NamedNode(nn) if nn.as_str() != crate::AUTH_GRAPH => Some(nn.clone()),
            _ => None,
        })
        .collect()
}

/// The outcome of [`check`]: either the (deduped) set of graph names the permitted
/// update may touch — used to decide re-materialization — or a deny reason.
pub(crate) struct Permit {
    /// Whether a re-materialization should follow a successful apply.
    pub rematerialize: bool,
}

/// Authorize an update string for `session` against `auth` over the dataset `graph`,
/// WITHOUT mutating anything. `Ok(Permit)` means every target is writable; `Err(msg)`
/// is a deny (fail-closed) and the caller must not apply the update.
pub(crate) fn check(
    graph: &sparq_core::Graph,
    auth: &AuthIndex,
    session: &Session,
    sparql: &str,
) -> Result<Permit, String> {
    let upd = SparqlParser::new().parse_update(sparql).map_err(|e| e.to_string())?;
    let reqs = analyze(&upd);

    if reqs.touches_default {
        return Err(
            "update denied: writes to the default graph are not permitted (pod data lives in \
             named graphs only)"
                .to_owned(),
        );
    }

    // Static per-graph requirements.
    for (g, need) in &reqs.graphs {
        if !allowed(auth, session, g, *need) {
            return Err(format!(
                "update denied: session lacks {} permission on <{}>",
                need_label(*need),
                g.as_str()
            ));
        }
    }

    // Conservative wildcard requirement: must be able to write EVERY store graph.
    if let Some(need) = reqs.wildcard {
        for g in store_named_graphs(graph) {
            // For a wildcard the per-graph need still respects the control-doc
            // convention (writing an .acl under a CLEAR ALL needs the Write grant that
            // only Control-holders have).
            let g_need = strongest(need, need_for_graph(g.as_str(), need));
            if !allowed(auth, session, &g, g_need) {
                return Err(format!(
                    "update denied: a graph-wildcard operation (variable GRAPH target or \
                     CLEAR/DROP ALL|NAMED) requires {} permission on every graph, but the \
                     session lacks it on <{}>",
                    need_label(need),
                    g.as_str()
                ));
            }
        }
    }

    let rematerialize = reqs.rematerialize_hint || reqs.wildcard.is_some();
    Ok(Permit { rematerialize })
}

fn need_label(n: Need) -> &'static str {
    match n {
        Need::WriteOrAppend => "write/append",
        Need::Write => "write",
    }
}
