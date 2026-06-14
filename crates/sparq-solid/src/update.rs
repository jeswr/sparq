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
//! - a `DELETE/INSERT … WHERE` template with a `GRAPH ?var` slot is resolved
//!   *precisely* ([OPUS-4.8] sq-biss): the operation's WHERE pattern is evaluated to
//!   enumerate the CONCRETE graphs `?var` actually binds to, and write is required only
//!   on THOSE graphs (per the read path's per-graph model) — not on every store graph.
//!   This stays fail-closed: the WHERE is evaluated against **exactly the dataset the
//!   engine will instantiate the templates over** (the full store, re-scoped by `USING`
//!   — see the soundness note on [`resolve_var_graphs`]), so the graphs checked are
//!   exactly the graphs the apply could touch; if any binding is not a writable named
//!   graph the update is denied, and any binding that cannot be reduced to a concrete
//!   named graph (a blank-node graph name, or a WHERE the analysis cannot evaluate)
//!   falls back to the conservative all-graphs check below (never a hole);
//! - a target whose graph name cannot be determined statically by the above — a
//!   `CLEAR`/`DROP` of `ALL`/`NAMED` graphs, or a `GRAPH ?var` slot that fell back —
//!   is treated *conservatively*: the actor must be able to write **every** named graph
//!   currently in the store, or the whole update is denied. This is sound (it can only
//!   reject an update the precise analysis might have allowed), never permissive.
//!
//! After a permitted update that touched an `.acl`/`.acr`/group document, the auth view
//! is automatically re-materialized (epoch bump → session cache dropped), so a changed
//! rule takes effect on the next query/update — the design doc's
//! "after any acl/acr/group-doc write, re-materialize" requirement (§4.4).

use crate::loader::{ACL_SUFFIX, ACR_SUFFIX};
use crate::{AuthIndex, Mode, Session};
use oxrdf::{NamedNode, Term, Variable};
use spargebra::algebra::{GraphPattern, GraphTarget, QueryDataset};
use spargebra::term::{GraphName, GraphNamePattern};
use spargebra::{GraphUpdateOperation, Query, SparqlParser, Update};

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

/// A `DELETE/INSERT … WHERE` operation with at least one `GRAPH ?var` template slot,
/// captured so [`resolve_var_graphs`] can evaluate the WHERE and turn the variable slots
/// into concrete per-solution graph targets. [OPUS-4.8] sq-biss.
#[derive(Debug)]
struct VarGraphResolve {
    /// The operation's WHERE pattern (what determines the bindings).
    pattern: GraphPattern,
    /// The operation's `USING` dataset, if any (re-scopes WHERE evaluation just as the
    /// engine's apply does — keep the two in lock-step or the checked set diverges from
    /// the written set).
    using: Option<QueryDataset>,
    /// The graph-slot variables and the base need each carries (delete slots → `Write`,
    /// insert slots → `WriteOrAppend`). The same variable may appear with both needs
    /// (deleted and inserted under the same name); both are recorded.
    slots: Vec<(Variable, Need)>,
}

/// What a parsed update needs in order to be permitted.
#[derive(Debug, Default)]
struct WriteReqs {
    /// Concrete (named-graph, need) requirements gathered from static targets.
    graphs: Vec<(NamedNode, Need)>,
    /// The update targets the default graph (always denied — pod data is never there).
    touches_default: bool,
    /// `DELETE/INSERT … WHERE` operations carrying a `GRAPH ?var` template slot, resolved
    /// precisely in [`check`] before the wildcard fallback. [OPUS-4.8] sq-biss.
    var_graphs: Vec<VarGraphResolve>,
    /// The update has a target whose graph cannot be determined statically (a
    /// `CLEAR`/`DROP` `ALL`/`NAMED`, or a `GRAPH ?var` slot whose precise resolution fell
    /// back): the actor must be able to write every named graph in the store, in this
    /// mode, or the update is denied.
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

/// Record the target of a `DELETE`/`INSERT` template quad slot. A concrete named-node or
/// default-graph slot is recorded immediately; a `GRAPH ?var` slot is collected into
/// `var_slots` so the enclosing operation can resolve it precisely against its WHERE
/// ([OPUS-4.8] sq-biss). The strongest need for any one variable wins.
fn push_graph_name_pattern(
    reqs: &mut WriteReqs,
    var_slots: &mut Vec<(Variable, Need)>,
    g: &GraphNamePattern,
    base: Need,
) {
    match g {
        GraphNamePattern::DefaultGraph => reqs.touches_default = true,
        GraphNamePattern::NamedNode(n) => push_graph(reqs, n, base),
        GraphNamePattern::Variable(v) => {
            // Don't escalate to the wildcard yet: collect the (var, need) so the
            // operation's WHERE can be evaluated and the concrete bound graphs checked.
            match var_slots.iter_mut().find(|(x, _)| x == v) {
                Some((_, n)) => *n = strongest(*n, base),
                None => var_slots.push((v.clone(), base)),
            }
        }
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
            GraphUpdateOperation::DeleteInsert { delete, insert, using, pattern } => {
                // Deletes always need Write; inserts need Write-or-Append. The WHERE
                // pattern only READS, so it is not a write target. A `GRAPH ?var` slot is
                // collected into `var_slots` and resolved precisely against this
                // operation's own WHERE/USING (not the whole-update conservative
                // wildcard). [OPUS-4.8] sq-biss.
                let mut var_slots: Vec<(Variable, Need)> = Vec::new();
                for d in delete {
                    push_graph_name_pattern(&mut reqs, &mut var_slots, &d.graph_name, Need::Write);
                }
                for i in insert {
                    push_graph_name_pattern(&mut reqs, &mut var_slots, &i.graph_name, Need::WriteOrAppend);
                }
                if !var_slots.is_empty() {
                    reqs.var_graphs.push(VarGraphResolve {
                        pattern: (**pattern).clone(),
                        using: using.clone(),
                        slots: var_slots,
                    });
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

/// Resolve a `DELETE/INSERT … WHERE` operation's `GRAPH ?var` template slots into the
/// concrete (named-graph, need) requirements its solutions actually produce.
/// [OPUS-4.8] sq-biss.
///
/// `Ok(reqs)` is the precise per-graph requirement set: write is needed only on the
/// graphs `?var` actually binds to. `Err(need)` means the resolution must fall back to
/// the conservative all-graphs wildcard at `need` (fail-closed); the caller raises it.
///
/// # Soundness — why the WHERE runs over the FULL store, not the actor's read view
///
/// The apply step ([`sparq_engine::update_in_place`]) instantiates the templates by
/// evaluating this exact WHERE pattern over the **full store**, then writes the resulting
/// graph slots. To check the right set of graphs we must enumerate the SAME bindings the
/// engine will — anything narrower (e.g. the actor's authorized read view) could miss a
/// graph the apply would write and open a hole, which the bead's note explicitly warns
/// against. Evaluating over the full store is *not* an information leak: we never return
/// rows to the actor — only an allow/deny verdict, and we deny unless the actor can WRITE
/// every graph the apply could touch (a graph the actor cannot even read is certainly not
/// writable, so it forces a deny). The verdict is exactly the one the conservative
/// wildcard already computed, only tightened to the graphs that genuinely appear as
/// targets.
///
/// Fail-closed cases that fall back to the wildcard (rather than risk a hole):
///
/// - the operation carries a `USING`/`WITH` clause: the apply re-scopes the WHERE through
///   `Dataset::build_using`, whose `named: None` (the parser's encoding of `WITH`) keeps
///   ALL store named graphs — but a plain query's `FROM`/`FROM NAMED` (`build_active`)
///   treats `named: None` as the EMPTY named set, so a serialized SELECT would
///   *under-count* the `GRAPH ?g` bindings. Rather than reproduce the update-side dataset
///   semantics (private to the engine), we stay conservative whenever a re-scope is
///   present — `Err(need)`;
/// - the WHERE cannot be evaluated (parse/serialize/engine error) — `Err(need)`;
/// - a slot binds to a **blank-node** graph name: the engine *would* write to it
///   ([`gnp_subst`] accepts blank nodes), but the auth view only ever grants write on
///   named graphs, so write can never be proven — `Err(need)`.
///
/// Bindings the engine *drops* (an unbound variable, or a literal/triple where a graph
/// name is required — [`gnp_subst`] returns `None`) produce no write and are ignored.
fn resolve_var_graphs(
    graph: &sparq_core::Graph,
    r: &VarGraphResolve,
) -> Result<Vec<(NamedNode, Need)>, Need> {
    // The strongest need across this operation's slots is the fallback level (and the
    // floor for any escalation): if we must give up, give up at the level that protects
    // every slot.
    let fallback = r.slots.iter().map(|(_, n)| *n).fold(Need::WriteOrAppend, strongest);

    // USING/WITH re-scopes the WHERE in a way a plain query cannot faithfully reproduce
    // (see the doc note) — fail closed to the conservative all-graphs check.
    if r.using.is_some() {
        return Err(fallback);
    }

    // Build `SELECT ?v… { WHERE }` and evaluate it exactly as the engine instantiates the
    // templates: the full store, no dataset re-scope (the USING/WITH case bailed above).
    let select = Query::Select {
        dataset: None,
        pattern: GraphPattern::Project {
            inner: Box::new(r.pattern.clone()),
            variables: r.slots.iter().map(|(v, _)| v.clone()).collect(),
        },
        base_iri: None,
    };
    let Ok(result) = sparq_engine::query(graph, &select.to_string()) else {
        return Err(fallback); // un-evaluable WHERE → conservative
    };

    // Column index of each slot variable in the result (a projected variable absent from
    // the row vars never binds → contributes no write).
    let col = |v: &Variable| result.vars.iter().position(|x| x == v);

    let mut out: Vec<(NamedNode, Need)> = Vec::new();
    for (v, need) in &r.slots {
        let Some(i) = col(v) else { continue };
        for row in &result.rows {
            match row.get(i).and_then(|c| c.as_ref()) {
                Some(Term::NamedNode(n)) => {
                    // Per-graph need, honouring the control-doc convention exactly as a
                    // static target would. (`check` flags re-materialization when a
                    // resolved target is an `.acl`/`.acr` graph.)
                    out.push((n.clone(), need_for_graph(n.as_str(), *need)));
                }
                // A blank-node graph name IS written by the engine but can never be
                // authorized → fail closed to the wildcard.
                Some(Term::BlankNode(_)) => return Err(fallback),
                // Unbound, or a literal/triple term: the engine drops the quad
                // (`gnp_subst` → None), so no write happens — nothing to authorize.
                _ => {}
            }
        }
    }
    Ok(out)
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
    let mut reqs = analyze(&upd);

    if reqs.touches_default {
        return Err(
            "update denied: writes to the default graph are not permitted (pod data lives in \
             named graphs only)"
                .to_owned(),
        );
    }

    // Precise resolution of `GRAPH ?var` template slots ([OPUS-4.8] sq-biss): evaluate
    // each such operation's WHERE to enumerate the concrete graphs it actually targets,
    // and fold them into the static per-graph requirements — instead of demanding write
    // on every store graph. A resolution that cannot be reduced safely raises the
    // conservative wildcard (handled below). Done BEFORE the static loop runs the checks,
    // so resolved targets are authorized on exactly the same footing as static ones.
    // (Resolve first, then fold in — `resolve_var_graphs` borrows `reqs.var_graphs`, the
    // folding mutates the rest of `reqs`.)
    let resolutions: Vec<Result<Vec<(NamedNode, Need)>, Need>> =
        reqs.var_graphs.iter().map(|r| resolve_var_graphs(graph, r)).collect();
    for res in resolutions {
        match res {
            Ok(resolved) => {
                // A precisely-resolved variable-graph op still re-materializes on success:
                // its targets are not statically known to avoid `.acl`/`.acr` AND group
                // documents (the latter have no naming convention — affects_auth_view
                // can't catch them), and re-materialization is only a cost, never a
                // security hole. Match the conservative path's "any dynamic write
                // re-materializes" guarantee. [OPUS-4.8] sq-biss.
                if !resolved.is_empty() {
                    reqs.rematerialize_hint = true;
                }
                reqs.graphs.extend(resolved);
            }
            // Fail-closed: fall back to demanding write on every store graph.
            Err(need) => raise_wildcard(&mut reqs, need),
        }
    }

    // Static per-graph requirements (now including the precisely-resolved variable-graph
    // targets).
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
