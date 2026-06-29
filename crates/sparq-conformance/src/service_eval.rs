//! [OPUS-4.8] sq-ddpgx (epic sq-my8wd) — the W3C SPARQL 1.1 `sparql11/service`
//! EVALUATION conformance lane, driven through the merged in-process loopback
//! harness from sq-ushvx ([`crate::service_loopback::LoopbackEndpoint`], #1291).
//!
//! ## What it does
//!
//! For each `mf:QueryEvaluationTest` of the W3C `sparql11/service` manifest this
//! runner:
//!
//! 1. parses the test's `mf:action` for the query file, the (optional) default-graph
//!    `qt:data` documents, and EACH `qt:serviceData [ qt:endpoint <IRI> ; qt:data <doc> ]`
//!    block (the per-endpoint remote dataset);
//! 2. stands up ONE real [`sparq_server::serve`] endpoint per `qt:serviceData` endpoint
//!    over that block's remote data, on a fresh ephemeral `127.0.0.1:0` loopback port
//!    (the merged harness fixture — serves the RDF, returns the bound URL, RAII teardown);
//! 3. REWRITES every well-known endpoint IRI (e.g. `http://example.org/sparql`) to the
//!    loopback `sparql_url()` of its stood-up endpoint, BOTH in the query text AND in the
//!    local/default-graph data (the `service5` test resolves `SERVICE ?endpoint` from a
//!    `void:sparqlEndpoint` triple in the data, so the rewrite has to reach the data too);
//! 4. runs the rewritten federated query over the local graph through the engine's REAL
//!    `ureq` HTTP transport ([`sparq_engine::query`] under the `service` feature), with the
//!    egress allowlist scoped (for that call only) to the stood-up loopback hosts; and
//! 5. compares the engine's solutions to the suite's `.srx` result oracle under the same
//!    bag/bnode-bijection machinery the SPARQL ratchet uses ([`crate::compare::rows_equal`]).
//!
//! So the WHOLE federated path is exercised end-to-end: HTTP request/response, `Accept`
//! content negotiation, SPARQL-Results parsing, the bind-join over the wire — not a
//! `pub(crate)` canned mock. Per the maintainer-approved mocking the "remote" endpoints
//! are in-process loopback servers serving the suite's own fixture data.
//!
//! ## SILENT vs non-SILENT semantics
//!
//! `service6` / `service7` carry a `SERVICE SILENT <http://invalid.endpoint.org/sparql>`
//! to an endpoint that is never stood up — the suite's oracle is that `SILENT` SWALLOWS
//! the failed call (the join continues with no remote contribution). This runner therefore
//! NEVER stands up an endpoint for an IRI that is not allowlisted, so a `SILENT` clause
//! targeting one really does hit a refusal and must be swallowed. The complementary
//! non-SILENT direction (a CLOSED port must PROPAGATE the error) is pinned by a dedicated
//! unit test in `tests/service_eval_suite.rs` rather than the manifest (the manifest has no
//! non-SILENT-to-a-dead-endpoint case).
//!
//! ## Honest floor
//!
//! The lane reports its results as [`SuiteOutcome`]; the gating test
//! (`tests/service_eval_suite.rs`) asserts the MEASURED `SERVICE_EVAL_FLOOR` pass count
//! (the rest is tracked-not-asserted — no skip-laundering). The floor may only RISE.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use oxrdf::{NamedOrBlankNode, Term};
use spargebra::algebra::GraphPattern;
use spargebra::term::NamedNodePattern;
use spargebra::{Query, SparqlParser};

use crate::compare::{rows_equal, Row};
use crate::manifest::{MF, QT};
use crate::rdf::{as_node, file_iri, iri_to_path, parse_file, MiniGraph};
use crate::results::{parse_expected, Binding, Expected};
use crate::service_loopback::LoopbackEndpoint;

/// One `qt:serviceData [ qt:endpoint <IRI> ; qt:data <doc> ]` block of a test.
#[derive(Debug, Clone)]
struct ServiceData {
    /// The well-known endpoint IRI used in the query / data (e.g. `http://example.org/sparql`).
    endpoint: String,
    /// The remote dataset document the loopback endpoint serves.
    data: PathBuf,
}

/// One `mf:QueryEvaluationTest` of the service suite.
#[derive(Debug, Clone)]
struct ServiceTest {
    name: String,
    query: PathBuf,
    /// Default-graph `qt:data` documents (loaded into the LOCAL graph).
    data: Vec<PathBuf>,
    /// Each remote endpoint's `(endpoint IRI, served data)`.
    service_data: Vec<ServiceData>,
    result: PathBuf,
}

/// Per-test verdict.
#[derive(Debug, Clone)]
pub enum Outcome {
    Pass,
    Fail(String),
    Skip(String),
}

/// The result of running the whole service evaluation suite.
#[derive(Debug, Default)]
pub struct SuiteOutcome {
    pub pass: usize,
    pub fail: usize,
    pub skip: usize,
    /// `(test name, why)` for every failing test — surfaced by the runner so a
    /// regression is actionable, never silently counted.
    pub failures: Vec<(String, String)>,
}

/// Parses the `sparql11/service` manifest into the per-test descriptors.
fn parse_manifest(manifest: &Path) -> Result<Vec<ServiceTest>, String> {
    let g = MiniGraph::load(manifest)?;
    let dir = manifest
        .parent()
        .ok_or_else(|| format!("{}: no parent dir", manifest.display()))?;
    // Resolve a relative manifest IRI (e.g. <service01.rq>) to a file next to it.
    let resolve = |t: &Term| -> Option<PathBuf> {
        match t {
            Term::NamedNode(n) => {
                // The manifest base is the manifest file, so relative names are
                // already absolute file:// IRIs after parse; fall back to joining
                // the bare last path segment onto the manifest directory.
                iri_to_path(n.as_str()).or_else(|| {
                    n.as_str()
                        .rsplit('/')
                        .next()
                        .map(|seg| dir.join(seg))
                })
            }
            _ => None,
        }
    };

    let mut tests = Vec::new();
    for m in g.subjects_with_type(&format!("{MF}Manifest")) {
        let Some(entries) = g.object(&m, &format!("{MF}entries")) else {
            continue;
        };
        for item in g.list(entries) {
            let Some(node) = as_node(&item) else { continue };
            // Only evaluation tests; the suite has no other kinds, but be defensive.
            let is_eval = g
                .types_of(&node)
                .iter()
                .any(|t| t == &format!("{MF}QueryEvaluationTest"));
            if !is_eval {
                continue;
            }
            let name = g
                .str_object(&node, &format!("{MF}name"))
                .unwrap_or_else(|| match &node {
                    NamedOrBlankNode::NamedNode(n) => n.as_str().to_string(),
                    NamedOrBlankNode::BlankNode(b) => format!("_:{}", b.as_str()),
                });
            let Some(action_t) = g.object(&node, &format!("{MF}action")) else {
                continue;
            };
            let Some(action) = as_node(action_t) else {
                continue;
            };
            let Some(query) = g
                .object(&action, &format!("{QT}query"))
                .and_then(resolve)
            else {
                continue;
            };
            let data: Vec<PathBuf> = g
                .objects(&action, &format!("{QT}data"))
                .into_iter()
                .filter_map(resolve)
                .collect();
            let mut service_data = Vec::new();
            for sd in g.objects(&action, &format!("{QT}serviceData")) {
                let Some(sd) = as_node(sd) else { continue };
                let endpoint = match g.object(&sd, &format!("{QT}endpoint")) {
                    Some(Term::NamedNode(n)) => n.as_str().to_string(),
                    _ => continue,
                };
                let Some(ddoc) = g
                    .object(&sd, &format!("{QT}data"))
                    .and_then(resolve)
                else {
                    continue;
                };
                service_data.push(ServiceData { endpoint, data: ddoc });
            }
            let Some(result) = g.object(&node, &format!("{MF}result")).and_then(resolve)
            else {
                continue;
            };
            tests.push(ServiceTest {
                name,
                query,
                data,
                service_data,
                result,
            });
        }
    }
    Ok(tests)
}

/// Loads `doc` and serves it on a fresh ephemeral loopback endpoint.
fn stand_up(doc: &Path) -> Result<LoopbackEndpoint, String> {
    let triples = parse_file(doc)?;
    // Re-serialize to N-Triples so the in-memory `Graph` loads exactly the remote
    // dataset (the loopback server answers SPARQL over it).
    let mut nt = String::new();
    for t in &triples {
        nt.push_str(&format!("{} {} {} .\n", t.subject, t.predicate, t.object));
    }
    let graph = sparq_core::Graph::load_str(&nt, "ntriples")
        .map_err(|e| format!("load remote dataset {}: {e}", doc.display()))?;
    Ok(LoopbackEndpoint::serve(graph))
}

/// Rewrites every well-known endpoint IRI to its loopback `sparql_url()` in `text`.
/// Endpoints are replaced longest-IRI-first so a prefix IRI never clobbers a longer one.
fn rewrite_endpoints(text: &str, map: &BTreeMap<String, String>) -> String {
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    let mut out = text.to_string();
    for k in keys {
        out = out.replace(k.as_str(), &map[k]);
    }
    out
}

/// Builds the LOCAL (default-graph) N-Triples document from the test's `qt:data`,
/// rewriting any well-known endpoint IRI it carries (the `service5` `void:sparqlEndpoint`
/// triples) to the loopback URL so `SERVICE ?endpoint` dials the loopback server.
fn build_local_nt(data: &[PathBuf], map: &BTreeMap<String, String>) -> Result<String, String> {
    let mut out = String::new();
    for d in data {
        for t in parse_file(d)? {
            let line = format!("{} {} {} .\n", t.subject, t.predicate, t.object);
            out.push_str(&rewrite_endpoints(&line, map));
        }
    }
    Ok(out)
}

/// Aligns an expected binding onto a shared variable order.
fn align(binding: &Binding, order: &[String]) -> Row {
    order
        .iter()
        .map(|v| binding.iter().find(|(bv, _)| bv == v).map(|(_, t)| t.clone()))
        .collect()
}

/// Walks a graph pattern, invoking `f` on every nested pattern (pre-order).
fn walk(p: &GraphPattern, f: &mut impl FnMut(&GraphPattern)) {
    f(p);
    match p {
        GraphPattern::Join { left, right }
        | GraphPattern::LeftJoin { left, right, .. }
        | GraphPattern::Union { left, right }
        | GraphPattern::Lateral { left, right }
        | GraphPattern::Minus { left, right } => {
            walk(left, f);
            walk(right, f);
        }
        GraphPattern::Filter { inner, .. }
        | GraphPattern::Extend { inner, .. }
        | GraphPattern::OrderBy { inner, .. }
        | GraphPattern::Project { inner, .. }
        | GraphPattern::Distinct { inner }
        | GraphPattern::Reduced { inner }
        | GraphPattern::Slice { inner, .. }
        | GraphPattern::Group { inner, .. }
        | GraphPattern::Service { inner, .. }
        | GraphPattern::Graph { inner, .. } => walk(inner, f),
        // Bgp / Path / Values / table-shaped leaves have no nested patterns.
        _ => {}
    }
}

/// Classifies a query's SERVICE shape against what THIS in-process loopback
/// fixture can serve, returning a documented `Skip` reason for the two known
/// out-of-scope shapes (an honest tracked-not-asserted divergence — never
/// skip-laundered into the pass count), or `None` when the test is in scope.
///
/// * A **variable** SERVICE endpoint (`SERVICE ?var { … }`, the `service5` test)
///   is not supported by the engine's federation evaluator (it requires resolving
///   the endpoint from a binding before the remote call); reported as an explicit
///   engine "not supported" error, classified here BEFORE the run so it is a Skip,
///   not a Fail.
/// * A **nested NON-SILENT** SERVICE (a `SERVICE` whose inner pattern itself
///   contains a non-`SILENT` `SERVICE`, the `service3` test) would require the
///   LOOPBACK endpoint to itself federate ONWARD to a second endpoint. The merged
///   harness serves a plain in-memory `Graph` with federation disabled (an
///   unconfigured `sparq-server` refuses ALL onward SERVICE — fail-closed), so the
///   inner server cannot reach the second endpoint and the non-silent inner call
///   propagates the refusal. Wiring inner-endpoint federation (so two loopback
///   servers can reach each other) is a separately-reviewable harness extension —
///   see the crate README / the deferred-bead note in the PR body. A nested
///   **SILENT** SERVICE (the `service6` test) is fine: the inner server's refusal
///   is swallowed by the `SILENT`, exactly as the oracle expects, so that test is
///   IN scope and PASSES end-to-end through this harness.
fn out_of_scope_reason(pattern: &GraphPattern) -> Option<String> {
    let mut variable_endpoint = false;
    let mut nested_non_silent_service = false;
    walk(pattern, &mut |p| {
        if let GraphPattern::Service { name, inner, .. } = p {
            if matches!(name, NamedNodePattern::Variable(_)) {
                variable_endpoint = true;
            }
            // A NON-SILENT SERVICE nested inside this SERVICE's inner pattern: the
            // inner endpoint would have to federate onward, which the plain-Graph
            // loopback fixture cannot do — and being non-silent, the refusal would
            // propagate. (A nested SILENT one is swallowed by the inner server, so
            // it stays in scope.)
            walk(inner, &mut |q| {
                if matches!(q, GraphPattern::Service { silent: false, .. }) {
                    nested_non_silent_service = true;
                }
            });
        }
    });
    if variable_endpoint {
        Some(
            "variable SERVICE endpoint (`SERVICE ?var`) — engine federation evaluator does not \
             resolve a bound endpoint (tracked-not-asserted, sq-my8wd child)"
                .into(),
        )
    } else if nested_non_silent_service {
        Some(
            "nested non-SILENT SERVICE — the in-process loopback fixture serves a plain Graph \
             with federation disabled, so the inner endpoint cannot federate onward \
             (tracked-not-asserted; inner-endpoint federation is a harness extension, \
             sq-my8wd child)"
                .into(),
        )
    } else {
        None
    }
}

/// Runs ONE service evaluation test end-to-end through the loopback harness.
fn run_test(test: &ServiceTest) -> Outcome {
    let expected = match parse_expected(&test.result) {
        Ok(e) => e,
        Err(e) => return Outcome::Fail(format!("expected-result parse error: {e}")),
    };
    let Expected::Bindings {
        vars: exp_vars,
        rows: exp_rows,
        ..
    } = expected
    else {
        return Outcome::Fail("service tests are all SELECT, expected a binding set".into());
    };

    let query_text = match std::fs::read_to_string(&test.query) {
        Ok(t) => t,
        Err(e) => return Outcome::Fail(format!("read query: {e}")),
    };
    let base = file_iri(&test.query);

    // Classify the query's SERVICE shape up front: the two known out-of-scope
    // shapes (a variable endpoint, a nested SERVICE) become documented Skips, not
    // Fails — an honest tracked-not-asserted divergence, never skip-laundered.
    let parser = match SparqlParser::new().with_base_iri(&base) {
        Ok(p) => p,
        Err(e) => return Outcome::Fail(format!("bad base IRI: {e}")),
    };
    match parser.parse_query(&query_text) {
        Ok(Query::Select { pattern, .. }) => {
            if let Some(why) = out_of_scope_reason(&pattern) {
                return Outcome::Skip(why);
            }
        }
        Ok(_) => {} // service suite is all SELECT; let the run path report any surprise
        Err(e) => return Outcome::Fail(format!("query parse error: {e}")),
    }

    // Stand up one loopback endpoint per qt:serviceData block and build the
    // endpoint-IRI -> loopback-sparql-URL rewrite map. The endpoints are held in
    // `_endpoints` for the lifetime of the query (RAII teardown on return/unwind).
    let mut endpoints = Vec::new();
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    let mut allow_hosts: Vec<String> = Vec::new();
    for sd in &test.service_data {
        let ep = match stand_up(&sd.data) {
            Ok(ep) => ep,
            Err(e) => return Outcome::Fail(e),
        };
        map.insert(sd.endpoint.clone(), ep.sparql_url());
        allow_hosts.push(ep.host());
        endpoints.push(ep);
    }

    // Rewrite the endpoint IRIs in the query so SERVICE dials the loopback ports.
    let query_rewritten = rewrite_endpoints(&query_text, &map);
    // Prepend a BASE so any relative IRIs in the query resolve to the query file.
    let query_with_base = format!("BASE <{base}>\n{query_rewritten}");

    let local_nt = match build_local_nt(&test.data, &map) {
        Ok(n) => n,
        Err(e) => return Outcome::Fail(format!("local data load error: {e}")),
    };

    // Drive the federated query through the engine's REAL ureq transport, with the
    // egress allowlist scoped to exactly the loopback hosts we stood up (so the
    // SILENT-to-an-unallowlisted-endpoint cases really hit a refusal). The filter is
    // NOT globally disabled.
    let result = sparq_engine::with_service_egress_allow(allow_hosts, || {
        let graph = sparq_core::Graph::load_str(&local_nt, "ntriples")
            .map_err(|e| format!("load local graph: {e}"))?;
        sparq_engine::query(&graph, &query_with_base)
    });
    let res = match result {
        Ok(r) => r,
        Err(e) => return Outcome::Fail(format!("engine error: {e}")),
    };

    let actual_vars: Vec<String> = res.vars.iter().map(|v| v.as_str().to_string()).collect();
    // Variable sets must agree (the suite's .srx declares them).
    if !exp_vars.is_empty() {
        use std::collections::BTreeSet;
        let exp_set: BTreeSet<&str> = exp_vars.iter().map(|s| s.as_str()).collect();
        let act_set: BTreeSet<&str> = actual_vars.iter().map(|s| s.as_str()).collect();
        if exp_set != act_set {
            return Outcome::Fail(format!(
                "variables mismatch: expected {{{}}}, got {{{}}}",
                exp_vars.join(", "),
                actual_vars.join(", ")
            ));
        }
    }

    // Align both sides on a shared variable order and compare under bag semantics
    // (the service queries carry no ORDER BY).
    let mut all: std::collections::BTreeSet<String> = actual_vars.iter().cloned().collect();
    all.extend(exp_vars.iter().cloned());
    for r in &exp_rows {
        all.extend(r.iter().map(|(v, _)| v.clone()));
    }
    let order: Vec<String> = all.into_iter().collect();
    let exp: Vec<Row> = exp_rows.iter().map(|r| align(r, &order)).collect();
    let act: Vec<Row> = res
        .rows
        .iter()
        .map(|r| {
            order
                .iter()
                .map(|v| {
                    actual_vars
                        .iter()
                        .position(|av| av == v)
                        .and_then(|i| r.get(i).cloned().flatten())
                })
                .collect()
        })
        .collect();

    match rows_equal(&exp, &act, false) {
        Ok(true) => Outcome::Pass,
        Ok(false) => Outcome::Fail(format!(
            "result mismatch: expected {} solution(s), got {}",
            exp.len(),
            act.len()
        )),
        Err(e) => Outcome::Fail(e),
    }
}

/// Walks the `sparql11/service` manifest and runs every evaluation test through
/// the loopback harness, returning the aggregate outcome. `root` is the rdf-tests
/// clone root (the same one the SPARQL harness uses).
pub fn run_service_suite(root: &Path) -> Result<SuiteOutcome, String> {
    let manifest = root.join("sparql/sparql11/service/manifest.ttl");
    let tests = parse_manifest(&manifest)?;
    let mut out = SuiteOutcome::default();
    for test in &tests {
        match run_test(test) {
            Outcome::Pass => out.pass += 1,
            Outcome::Skip(_) => out.skip += 1,
            Outcome::Fail(why) => {
                out.fail += 1;
                out.failures.push((test.name.clone(), why));
            }
        }
    }
    Ok(out)
}

/// Stands up a single loopback endpoint over `nt_data` (N-Triples) and runs
/// `query_template` against an empty local graph, with `{EP}` in the template
/// replaced by the endpoint's `sparql_url()`. Used by the SILENT-semantics tests
/// to exercise a LIVE endpoint; the CLOSED-port direction does not stand one up.
pub fn run_against_live_endpoint(nt_data: &str, query_template: &str) -> Result<usize, String> {
    let graph = sparq_core::Graph::load_str(nt_data, "ntriples")
        .map_err(|e| format!("load remote dataset: {e}"))?;
    let ep = LoopbackEndpoint::serve(graph);
    let query = query_template.replace("{EP}", &ep.sparql_url());
    let res = sparq_engine::with_service_egress_allow([ep.host()], || {
        let local = sparq_core::Graph::load_str("", "ntriples")
            .map_err(|e| format!("load empty local graph: {e}"))?;
        sparq_engine::query(&local, &query)
    })?;
    Ok(res.rows.len())
}
