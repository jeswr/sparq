// [FABLE-5] sq-ixc3.19 — native structured EXPLAIN / EXPLAIN ANALYZE for the plan explorer.
//
// The GUI's query path runs in the in-tab WASM engine, whose `explainPlanAnalyzeJson`
// binding is exact on row counts and q-error and measures wall time through the browser's
// performance.now() clock at host-timer resolution. This module is the second deliberate,
// reviewed native query surface
// (after `federation::query_service`, see the lib.rs note on the removed pre-wired
// commands): exactly ONE command, `explain_native`, scoped to running the analyze over a
// native snapshot of the live workspace store with a native monotonic clock. It reuses
// `query_service`'s snapshot wire
// pattern (the whole dataset as N-Quads, the same format the native loader returns) and
// returns `sparq-engine`'s typed plan tree (`explain_json::PlanNode`, sq-u4lgr/#902)
// serialised as camelCase JSON — the sq-jbqh4 schema contract, byte-identical to what the
// wasm binding and the server's `application/x-sparq-explain+json` response emit — so the
// panel renders all three sources with one component.
//
// ALWAYS COMPILED (no cargo feature): unlike `hdt`/`federation`/`odrl` there is no heavy
// optional dependency to keep out of the lean build — `sparq-engine/explain-json` is a
// zero-dependency feature — and the command performs pure local computation (no FS, no
// egress) over caller-supplied data the same webview already evaluates in wasm.
//
// SECURITY — FAIL-CLOSED EGRESS: EXPLAIN ANALYZE *executes* the query. Under the
// `federation` build (which links the engine's SERVICE client) every call installs the
// STRICT allowlist-only egress policy with an EMPTY allowlist, so an ANALYZE of a
// SERVICE-bearing query is refused pre-HTTP — this command never dials; federated
// execution stays `query_service`'s job with the user's explicit per-workspace allowlist.
// A lean build has no SERVICE client at all.

/// Build the engine's structured plan tree for `query` over a native snapshot of the
/// workspace store. `dataset` is the whole dataset as N-Quads (empty = fresh workspace);
/// `analyze` executes (SELECT/ASK only) and fills per-operator `actual` rows, REAL wall
/// `nanos`, and `qError`; plan-only supports every query form. Returns the camelCase
/// JSON tree (`operator`/`estimated`/`actual`/`nanos`/`qError`/`children`).
#[tauri::command]
pub fn explain_native(dataset: String, query: String, analyze: bool) -> Result<String, String> {
    run_explain_native(&dataset, &query, analyze)
}

/// The command body, factored out so it can be unit-tested without a Tauri runtime (the
/// same pattern as `federation::run_service_query`).
fn run_explain_native(dataset: &str, query: &str, analyze: bool) -> Result<String, String> {
    // An empty snapshot is a legal store (a fresh workspace); parse otherwise. N-Quads is
    // the GUI's whole-dataset wire format, so named graphs survive into the native explain.
    let graph = if dataset.trim().is_empty() {
        sparq_core::Graph::new()
    } else {
        sparq_core::Graph::load_dataset(dataset, "nquads")?
    };
    explain_with_egress_guard(&graph, query, analyze).map(|plan| plan.to_json())
}

/// Federation build: ANALYZE executes, so pin the STRICT empty egress allowlist — a
/// SERVICE endpoint is refused pre-HTTP (fail-closed), never dialed from this command.
#[cfg(feature = "federation")]
fn explain_with_egress_guard(
    graph: &sparq_core::Graph,
    query: &str,
    analyze: bool,
) -> Result<sparq_engine::PlanNode, String> {
    sparq_engine::with_service_egress_policy(true, Vec::new(), || {
        explain_plan_tree(graph, query, analyze)
    })
}

/// Lean build: no SERVICE client is linked, so there is nothing to guard.
#[cfg(not(feature = "federation"))]
fn explain_with_egress_guard(
    graph: &sparq_core::Graph,
    query: &str,
    analyze: bool,
) -> Result<sparq_engine::PlanNode, String> {
    explain_plan_tree(graph, query, analyze)
}

fn explain_plan_tree(
    graph: &sparq_core::Graph,
    query: &str,
    analyze: bool,
) -> Result<sparq_engine::PlanNode, String> {
    if analyze {
        sparq_engine::explain_plan_analyze(graph, query)
    } else {
        sparq_engine::explain_plan(graph, query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NQUADS: &str = "<http://ex/alice> <http://ex/name> \"Alice\" .\n\
                          <http://ex/bob> <http://ex/name> \"Bob\" .\n";

    /// Plan-only over a real snapshot: camelCase schema, nothing executed.
    #[test]
    fn plan_only_dry_run() {
        let json = run_explain_native(
            NQUADS,
            "SELECT ?n WHERE { ?s <http://ex/name> ?n }",
            false,
        )
        .unwrap();
        assert!(json.contains("\"operator\":"), "{json}");
        assert!(json.contains("\"actual\":null"), "{json}");
    }

    /// ANALYZE executes natively: exact row counts and a wall-nanos field measured by
    /// the native monotonic clock.
    #[test]
    fn analyze_fills_actual_rows_and_wall_nanos() {
        let json = run_explain_native(
            NQUADS,
            "SELECT ?n WHERE { ?s <http://ex/name> ?n }",
            true,
        )
        .unwrap();
        assert!(json.contains("\"actual\":2"), "{json}");
        assert!(!json.contains("\"nanos\":null"), "{json}");
    }

    /// An empty dataset is a legal fresh workspace; a malformed one is a loud error.
    #[test]
    fn empty_dataset_ok_malformed_err() {
        assert!(run_explain_native("", "SELECT ?s WHERE { ?s ?p ?o }", false).is_ok());
        assert!(run_explain_native("<http://ex/a> <http://ex/p>", "SELECT ?s WHERE { ?s ?p ?o }", false).is_err());
    }

    /// ANALYZE of a graph-valued form is the engine's clear client error.
    #[test]
    fn analyze_of_construct_is_err() {
        let err = run_explain_native(NQUADS, "CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }", true)
            .unwrap_err();
        assert!(err.contains("EXPLAIN ANALYZE supports"), "{err}");
    }
}
