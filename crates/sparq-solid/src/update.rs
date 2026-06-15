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

// --- [OPUS-4.8] sq-3jtd.1: differential test — PodStore's precise variable-GRAPH write set
//     EQUALS the engine's actual write set ----------------------------------------------------
//
// The security invariant the rest of this module rests on (module docs, line 26ff): for a
// `DELETE/INSERT … WHERE` with a `GRAPH ?var` slot, `resolve_var_graphs` must enumerate
// EXACTLY the set of named graphs `sparq_engine::update_in_place` will actually write — no
// MORE (an over-approximation only costs a false denial) and crucially no LESS (an
// under-approximation = a write that escapes authorization = a security hole).
//
// These tests are the differential GUARD around that invariant (and around sq-cnor's
// USING/WITH precision work): they compute the precise set from `resolve_var_graphs` AND the
// real write set from `sparq_engine::update_in_place_capturing` (whose `UpdateEffect::Delta`
// records the slot of every graph the engine actually touched) and assert the two are EQUAL —
// or, for the cases `resolve_var_graphs` deliberately bails on (USING/WITH), assert the
// fallback is an OVER-approximation of the real write set (sound: a superset can only deny,
// never leak).
//
// Shapes mirror PSS's `setAclPointer`/`putContainer` (gh-47): a `DELETE/INSERT … WHERE` whose
// WHERE has an OPTIONAL (the binding may or may not match → the graph set varies), including a
// case where the OPTIONAL does NOT bind (a naive resolver could over- or under-count), and a
// WITH-clause variant (WITH re-scopes the operation's default graph).
#[cfg(test)]
mod differential_writeset_tests {
    use super::*;
    use sparq_core::Graph;
    use sparq_engine::{update_in_place_capturing, QueryBudget, UpdateEffect};
    use std::collections::BTreeSet;

    /// A tiny PSS-shaped pod: two resource graphs (`r1`, `r2`) each holding a `title`; only
    /// `r1` ALSO holds the optional `aclPointer` triple that PSS's `setAclPointer` keys its
    /// OPTIONAL off. A third graph `r3` holds an unrelated triple (no title) so a title-keyed
    /// WHERE never binds it — present to prove the resolver does not over-count the store.
    fn pss_dataset() -> Graph {
        let nq = "\
<https://pod.ex/r1#it> <https://ex.dev/ns#title> \"r1\" <https://pod.ex/r1> .
<https://pod.ex/r1#it> <https://ex.dev/ns#aclPointer> <https://pod.ex/r1.acl> <https://pod.ex/r1> .
<https://pod.ex/r2#it> <https://ex.dev/ns#title> \"r2\" <https://pod.ex/r2> .
<https://pod.ex/r3#it> <https://ex.dev/ns#other> \"r3\" <https://pod.ex/r3> .
";
        Graph::load_dataset(nq, "nquads").expect("fixture loads")
    }

    /// The set of named graphs `sparq_engine::update_in_place` ACTUALLY writes for `sparql`,
    /// observed via the engine's own resolved effect log (`UpdateEffect::Delta { slot, … }`
    /// records every graph slot the engine touched). Default-graph writes (slot `None`) are
    /// reported separately via the bool — none of these PSS shapes target the default graph.
    ///
    /// Blank-node graph slots ARE included in the named set (keyed by their `_:id`): a graph
    /// name is, per RDF, a NamedNode OR a BlankNode, so dropping the blank-node case could mask
    /// a real engine write and defeat the differential guard. Any OTHER `Some(term)` shape (a
    /// literal or triple-term graph name) is malformed for a graph slot — we fail loudly so a
    /// regression that starts emitting such a slot is caught immediately rather than swallowed.
    fn engine_write_set(graph: &mut Graph, sparql: &str) -> (BTreeSet<String>, bool) {
        let effects = update_in_place_capturing(graph, sparql, &QueryBudget::unlimited())
            .expect("engine applies the update");
        let mut named = BTreeSet::new();
        let mut touched_default = false;
        for e in &effects {
            if let UpdateEffect::Delta { slot, .. } = e {
                match slot {
                    Some(Term::NamedNode(n)) => {
                        named.insert(n.as_str().to_owned());
                    }
                    // A blank-node graph name is a valid graph slot; record it (keyed by its
                    // blank-node label) so a write here cannot silently slip past the diff.
                    Some(Term::BlankNode(b)) => {
                        named.insert(format!("_:{}", b.as_str()));
                    }
                    // Literal / triple-term graph slots are malformed — never expected.
                    Some(other) => {
                        panic!("engine emitted a non-graph-name slot in a Delta effect: {other:?}")
                    }
                    None => touched_default = true,
                }
            }
        }
        (named, touched_default)
    }

    /// PodStore's PRECISE variable-GRAPH target set for `sparql`, as computed by
    /// `resolve_var_graphs` over `graph`. `Some(set)` is the precise resolution; `None` means
    /// the resolver bailed to the conservative all-graphs wildcard (USING/WITH, blank-node
    /// binding, or an un-evaluable WHERE) — i.e. it demands write on EVERY store graph.
    fn resolve_var_graph_set(graph: &Graph, sparql: &str) -> Option<BTreeSet<String>> {
        let upd = SparqlParser::new().parse_update(sparql).expect("parses");
        let reqs = analyze(&upd);
        assert!(
            !reqs.var_graphs.is_empty(),
            "test update must carry a GRAPH ?var slot"
        );
        let mut out = BTreeSet::new();
        for r in &reqs.var_graphs {
            match resolve_var_graphs(graph, r) {
                Ok(resolved) => {
                    for (n, _need) in resolved {
                        out.insert(n.as_str().to_owned());
                    }
                }
                // Any operation that fell back to the wildcard makes the WHOLE update's
                // precise set undefined — report the fallback.
                Err(_) => return None,
            }
        }
        Some(out)
    }

    /// CASE 1 — OPTIONAL that BINDS, no re-scope: the precise resolver set must EQUAL the
    /// engine write set exactly. PSS `setAclPointer` shape: rewrite the pointer of every
    /// resource that has one. `?g` binds to the resources where the OPTIONAL matched.
    #[test]
    fn optional_bound_precise_set_equals_engine_write_set() {
        // DELETE the old pointer / INSERT a new one, keyed on the resource carrying a title;
        // the pointer triple sits behind an OPTIONAL, so `?p` is bound only for r1.
        let upd = "\
            DELETE { GRAPH ?g { ?s <https://ex.dev/ns#aclPointer> ?p } } \
            INSERT { GRAPH ?g { ?s <https://ex.dev/ns#aclPointer> <https://pod.ex/new.acl> } } \
            WHERE  { GRAPH ?g { ?s <https://ex.dev/ns#title> ?t . \
                                OPTIONAL { ?s <https://ex.dev/ns#aclPointer> ?p } } }";

        let graph = pss_dataset();
        let precise = resolve_var_graph_set(&graph, upd).expect("no re-scope -> precise");

        let mut applied = pss_dataset();
        let (written, default) = engine_write_set(&mut applied, upd);
        assert!(!default, "no default-graph write in this shape");

        // The DELETE half only produces a quad for r1 (where OPTIONAL bound `?p`); the INSERT
        // half produces a quad for r1 AND r2 (both have a title). The engine writes r1 and r2.
        // resolve_var_graphs must enumerate EXACTLY {r1, r2} — never r3 (no title), never the
        // whole store.
        assert_eq!(
            precise, written,
            "precise resolve_var_graphs set must EQUAL the engine's actual write set"
        );
        let expect: BTreeSet<String> = ["https://pod.ex/r1", "https://pod.ex/r2"]
            .map(String::from)
            .into();
        assert_eq!(written, expect, "engine wrote exactly r1+r2");
    }

    /// CASE 2 — OPTIONAL that does NOT bind for some rows: the precise set must still equal the
    /// engine write set. A naive resolver that counted the OPTIONAL's graph even when it is
    /// unbound (or skipped a graph because the OPTIONAL didn't match) would over/under-count.
    /// Here the DELETE template references the OPTIONAL var `?p`: for r2 (no pointer) `?p` is
    /// unbound, so the engine drops the DELETE quad — r2 is written ONLY via the INSERT half.
    #[test]
    fn optional_unbound_precise_set_equals_engine_write_set() {
        // DELETE keyed on the (possibly-unbound) OPTIONAL var; INSERT keyed on the title.
        // r1: OPTIONAL binds -> delete + insert. r2: OPTIONAL unbound -> insert only. r3: no
        // title -> never bound at all.
        let upd = "\
            DELETE { GRAPH ?g { ?s <https://ex.dev/ns#aclPointer> ?p } } \
            INSERT { GRAPH ?g { ?s <https://ex.dev/ns#mark> \"x\" } } \
            WHERE  { GRAPH ?g { ?s <https://ex.dev/ns#title> ?t . \
                                OPTIONAL { ?s <https://ex.dev/ns#aclPointer> ?p } } }";

        let graph = pss_dataset();
        let precise = resolve_var_graph_set(&graph, upd).expect("no re-scope -> precise");

        let mut applied = pss_dataset();
        let (written, default) = engine_write_set(&mut applied, upd);
        assert!(!default);

        // `?g` binds to r1 and r2 (both have a title). The engine writes both — r1 via
        // delete+insert, r2 via insert only (its DELETE quad is dropped, `?p` unbound). r3 is
        // never bound. The precise set must be exactly that, with NO phantom r3 and NO
        // escalation to all-store.
        assert_eq!(
            precise, written,
            "OPTIONAL-unbound: precise set must EQUAL the engine write set"
        );
        let expect: BTreeSet<String> = ["https://pod.ex/r1", "https://pod.ex/r2"]
            .map(String::from)
            .into();
        assert_eq!(written, expect, "engine wrote exactly r1+r2 (r3 unbound)");
    }

    /// CASE 3 — an OPTIONAL whose WHERE key is ABSENT everywhere, so `?g` binds to NOTHING:
    /// the precise set must be EMPTY and equal the engine's (empty) write set. A resolver that
    /// fell back to the wildcard here would wrongly demand write on the whole store for a
    /// no-op update.
    #[test]
    fn optional_empty_binding_precise_set_equals_empty_engine_write_set() {
        // No resource carries `<ex:missing>`, so `?g` never binds; the update is a no-op.
        let upd = "\
            DELETE { GRAPH ?g { ?s <https://ex.dev/ns#aclPointer> ?p } } \
            WHERE  { GRAPH ?g { ?s <https://ex.dev/ns#missing> ?t . \
                                OPTIONAL { ?s <https://ex.dev/ns#aclPointer> ?p } } }";

        let graph = pss_dataset();
        let precise = resolve_var_graph_set(&graph, upd).expect("no re-scope -> precise");

        let mut applied = pss_dataset();
        let (written, default) = engine_write_set(&mut applied, upd);
        assert!(!default);

        assert!(written.is_empty(), "engine writes nothing (no binding)");
        assert_eq!(
            precise, written,
            "empty binding -> empty precise set, no wildcard"
        );
    }

    /// CASE 4 — the WITH-clause variant (PSS shapes can carry `WITH`). `resolve_var_graphs`
    /// DELIBERATELY bails to the conservative all-graphs wildcard whenever a USING/WITH
    /// re-scope is present (`update.rs` ~line 305; root-caused in sq-cnor): a plain serialized
    /// SELECT cannot reproduce the apply's `build_using` dataset (`WITH` keeps ALL store named
    /// graphs, a query's `FROM NAMED` of `named: None` keeps NONE), so a precise resolution
    /// could UNDER-count and open a hole. We therefore verify the SECURITY property directly:
    /// the conservative target (every store named graph) is a SUPERSET of what the engine
    /// actually writes. A superset can only DENY (never leak) — sound, the invariant holds in
    /// the safe direction. (When sq-cnor tightens this to a precise set, this test's superset
    /// check still holds and the equality below documents the tightening target.)
    #[test]
    fn with_clause_falls_back_to_sound_over_approximation() {
        // WITH re-scopes the operation's DEFAULT graph to r1; the GRAPH ?g slot still ranges
        // over named graphs. The resolver sees a re-scope and bails (USING is Some).
        let upd = "\
            WITH <https://pod.ex/r1> \
            DELETE { GRAPH ?g { ?s <https://ex.dev/ns#aclPointer> ?p } } \
            INSERT { GRAPH ?g { ?s <https://ex.dev/ns#aclPointer> <https://pod.ex/new.acl> } } \
            WHERE  { GRAPH ?g { ?s <https://ex.dev/ns#title> ?t . \
                                OPTIONAL { ?s <https://ex.dev/ns#aclPointer> ?p } } }";

        let graph = pss_dataset();
        // resolve_var_graphs returns Err(_) -> the precise set is undefined; PodStore falls
        // back to the all-graphs wildcard.
        assert!(
            resolve_var_graph_set(&graph, upd).is_none(),
            "WITH/USING re-scope must trigger the conservative wildcard fallback"
        );

        // The conservative target set is EVERY named graph in the store.
        let conservative: BTreeSet<String> = store_named_graphs(&graph)
            .iter()
            .map(|n| n.as_str().to_owned())
            .collect();

        let mut applied = pss_dataset();
        let (written, default) = engine_write_set(&mut applied, upd);

        // WITH re-scopes the operation's DEFAULT graph, but EVERY quad pattern in the DELETE,
        // INSERT and WHERE templates here is explicitly `GRAPH ?g { … }`-scoped, so no triple
        // lands in (or is read from) the default graph: the WITH target is never written. Assert
        // that signal rather than dropping it — a regression that started routing writes to the
        // WITH default graph would otherwise pass the subset check below unnoticed.
        assert!(
            !default,
            "WITH default-graph re-scope must NOT produce a default-graph write when every \
             template quad is explicitly GRAPH ?g-scoped"
        );

        // SECURITY invariant in the safe direction: the authorization target (all store
        // graphs) is a SUPERSET of the engine's actual write set — never an under-approximation
        // (which would let a write escape auth). Over-approximation is sound; it only denies.
        assert!(
            written.is_subset(&conservative),
            "fallback wildcard must COVER every graph the engine writes (no under-approx hole): \
             engine wrote {written:?}, wildcard covers {conservative:?}"
        );
        // Document the precision gap (sq-cnor): the engine actually writes only r1+r2, but the
        // wildcard demands write on the whole store -> the fallback is a strict over-approx here.
        let real: BTreeSet<String> = ["https://pod.ex/r1", "https://pod.ex/r2"]
            .map(String::from)
            .into();
        assert_eq!(
            written, real,
            "engine's real WITH write set is exactly r1+r2"
        );
        assert!(
            real.is_subset(&conservative) && conservative.len() > real.len(),
            "wildcard is a STRICT over-approximation of the engine write set (the sq-cnor gap)"
        );
    }
}
