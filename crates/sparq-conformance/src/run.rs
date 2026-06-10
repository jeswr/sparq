//! Per-test execution: load the test dataset into a `sparq_core::Graph`, run
//! the query/update through `sparq_engine`, and compare against the expected
//! results. Each engine invocation runs on a watchdog thread so a hang or
//! panic in the engine becomes a recorded FAIL instead of killing the run.

use crate::compare::{rows_equal, Row};
use crate::manifest::TestEntry;
use crate::rdf::{file_iri, parse_file};
use crate::results::{parse_expected, Binding, Expected};
use oxrdf::{BlankNode, Term, Triple};
use spargebra::algebra::GraphPattern;
use spargebra::{Query, SparqlParser};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

/// Per-test wall-clock budget before the watchdog declares a FAIL(timeout).
const TEST_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone)]
pub enum Status {
    Pass,
    Fail(String),
    Skip(String),
}

/// Builds an N-Quads document for the test dataset: `data` files into the
/// default graph, `graph_data` files into their named graphs. Blank node
/// labels are prefixed per-file so distinct files never share bnodes.
fn build_nquads(data: &[PathBuf], graph_data: &[(String, PathBuf)]) -> Result<String, String> {
    let mut out = String::new();
    let mut file_idx = 0usize;
    let emit = |triples: Vec<Triple>, graph: Option<&str>, idx: usize, out: &mut String| {
        for t in triples {
            let t = prefix_bnodes(t, idx);
            match graph {
                Some(g) => out.push_str(&format!(
                    "{} {} {} <{}> .\n",
                    t.subject, t.predicate, t.object, g
                )),
                None => out.push_str(&format!("{} {} {} .\n", t.subject, t.predicate, t.object)),
            }
        }
    };
    for d in data {
        emit(parse_file(d)?, None, file_idx, &mut out);
        file_idx += 1;
    }
    for (g, d) in graph_data {
        emit(parse_file(d)?, Some(g), file_idx, &mut out);
        file_idx += 1;
    }
    Ok(out)
}

fn prefix_bnodes(t: Triple, idx: usize) -> Triple {
    let map = |term: Term| -> Term {
        match term {
            Term::BlankNode(b) => {
                Term::BlankNode(BlankNode::new_unchecked(format!("f{idx}x{}", b.as_str())))
            }
            other => other,
        }
    };
    let subject = match map(Term::from(t.subject)) {
        Term::NamedNode(n) => oxrdf::NamedOrBlankNode::NamedNode(n),
        Term::BlankNode(b) => oxrdf::NamedOrBlankNode::BlankNode(b),
        _ => unreachable!(),
    };
    Triple {
        subject,
        predicate: t.predicate,
        object: map(t.object),
    }
}

/// Prepends a BASE so relative IRIs in the query resolve against the query
/// file's location (the engine API takes no base; an explicit in-text BASE
/// later in the prologue simply overrides this one, which is fine).
fn with_base(text: &str, base: &str) -> String {
    format!("BASE <{base}>\n{text}")
}

/// True when the outermost solution modifiers include ORDER BY (the only case
/// where the W3C tests demand sequence comparison).
fn is_ordered(p: &GraphPattern) -> bool {
    match p {
        GraphPattern::OrderBy { .. } => true,
        GraphPattern::Project { inner, .. }
        | GraphPattern::Distinct { inner }
        | GraphPattern::Reduced { inner }
        | GraphPattern::Slice { inner, .. } => is_ordered(inner),
        _ => false,
    }
}

pub fn run_query_test(entry: &TestEntry) -> Status {
    let Some(query_path) = entry.action.query.clone() else {
        return Status::Fail("manifest entry has no qt:query".into());
    };
    if let Some(feat) = &entry.action.unsupported_feature {
        return Status::Skip(format!("{feat} not supported"));
    }
    let query_text = match std::fs::read_to_string(&query_path) {
        Ok(t) => t,
        Err(e) => return Status::Fail(format!("read query: {e}")),
    };
    let base = file_iri(&query_path);

    // Classify the query up front (the engine re-parses internally).
    let parsed = match SparqlParser::new().with_base_iri(&base) {
        Ok(p) => p.parse_query(&query_text),
        Err(e) => return Status::Fail(format!("bad base IRI: {e}")),
    };
    let (pattern, dataset, is_ask) = match &parsed {
        Ok(Query::Select {
            pattern, dataset, ..
        }) => (pattern, dataset, false),
        Ok(Query::Ask { pattern, dataset, .. }) => (pattern, dataset, true),
        Ok(Query::Construct { dataset, .. }) => {
            if dataset.is_some() {
                return Status::Skip("FROM / FROM NAMED dataset clause not supported".into());
            }
            return run_construct_test(entry, &query_text, &base);
        }
        Ok(Query::Describe { .. }) => {
            // The engine implements DESCRIBE (CBD), but the spec leaves the result
            // form to the implementation, so expected graphs are not comparable.
            return Status::Skip("DESCRIBE result form is implementation-defined (engine returns CBD)".into());
        }
        Err(e) => return Status::Fail(format!("query parse error: {e}")),
    };
    if dataset.is_some() {
        return Status::Skip("FROM / FROM NAMED dataset clause not supported".into());
    }
    let ordered = is_ordered(pattern);

    let Some(result_path) = entry.result_file.clone() else {
        return Status::Skip("no mf:result".into());
    };
    let expected = match parse_expected(&result_path) {
        Ok(e) => e,
        Err(e) if e.starts_with("unsupported result format") => return Status::Skip(e),
        Err(e) => return Status::Fail(format!("expected-result parse error: {e}")),
    };

    let nquads = match build_nquads(&entry.action.data, &entry.action.graph_data) {
        Ok(n) => n,
        Err(e) => return Status::Fail(format!("data load error: {e}")),
    };
    let query_with_base = with_base(&query_text, &base);

    // ASK: run on the engine's native boolean path and compare to the expected boolean.
    if is_ask {
        let expected_bool = match expected {
            Expected::Boolean(b) => b,
            Expected::Bindings { .. } => {
                return Status::Fail("expected result is a binding set, query is ASK".into())
            }
        };
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = (|| {
                let graph = sparq_core::Graph::load_dataset(&nquads, "nquads")?;
                sparq_engine::ask(&graph, &query_with_base)
            })();
            let _ = tx.send(result);
        });
        return match rx.recv_timeout(TEST_TIMEOUT) {
            Ok(Ok(actual)) if actual == expected_bool => Status::Pass,
            Ok(Ok(actual)) => Status::Fail(format!("ASK mismatch: expected {expected_bool}, got {actual}")),
            Ok(Err(e)) => Status::Fail(format!("engine error: {e}")),
            Err(mpsc::RecvTimeoutError::Timeout) => Status::Fail("timeout (20s)".into()),
            Err(mpsc::RecvTimeoutError::Disconnected) => Status::Fail("engine panicked".into()),
        };
    }

    // Engine invocation on a watchdog thread.
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = (|| {
            let graph = sparq_core::Graph::load_dataset(&nquads, "nquads")?;
            let res = sparq_engine::query(&graph, &query_with_base)?;
            let vars: Vec<String> = res.vars.iter().map(|v| v.as_str().to_string()).collect();
            Ok::<_, String>((vars, res.rows))
        })();
        let _ = tx.send(result);
    });
    let (actual_vars, actual_rows) = match rx.recv_timeout(TEST_TIMEOUT) {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => return Status::Fail(format!("engine error: {e}")),
        Err(mpsc::RecvTimeoutError::Timeout) => return Status::Fail("timeout (20s)".into()),
        Err(mpsc::RecvTimeoutError::Disconnected) => return Status::Fail("engine panicked".into()),
    };

    let (exp_vars, exp_rows, indexed) = match expected {
        Expected::Bindings {
            vars,
            rows,
            indexed,
        } => (vars, rows, indexed),
        Expected::Boolean(b) => {
            return Status::Fail(format!("expected result is boolean ({b}), query is SELECT"))
        }
    };

    // Variable sets must agree (when the expected file declares them).
    let actual_var_set: BTreeSet<&str> = actual_vars.iter().map(|s| s.as_str()).collect();
    if !exp_vars.is_empty() {
        let exp_var_set: BTreeSet<&str> = exp_vars.iter().map(|s| s.as_str()).collect();
        if exp_var_set != actual_var_set {
            return Status::Fail(format!(
                "variables mismatch: expected {{{}}}, got {{{}}}",
                exp_vars.join(", "),
                actual_vars.join(", ")
            ));
        }
    }

    // Align both sides on a shared variable order.
    let mut all_vars: BTreeSet<String> = actual_vars.iter().cloned().collect();
    all_vars.extend(exp_vars.iter().cloned());
    for row in &exp_rows {
        all_vars.extend(row.iter().map(|(v, _)| v.clone()));
    }
    let order: Vec<String> = all_vars.into_iter().collect();

    let exp: Vec<Row> = exp_rows.iter().map(|r| align_binding(r, &order)).collect();
    let act: Vec<Row> = actual_rows
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

    // Sequence comparison only when the query orders AND the expected encoding
    // preserves order (SRX document order, or rs:index in result-set graphs).
    let compare_ordered =
        ordered && (result_path.extension().is_some_and(|e| e == "srx") || indexed);
    match rows_equal(&exp, &act, compare_ordered) {
        Ok(true) => Status::Pass,
        Ok(false) => Status::Fail(format!(
            "result mismatch: expected {} solution(s), got {}{}{}",
            exp.len(),
            act.len(),
            if compare_ordered { " (ordered)" } else { "" },
            diff_sample(&order, &exp, &act)
        )),
        Err(e) => Status::Fail(e),
    }
}

/// A compact sample of rows present on only one side (label-sensitive, so it is
/// a debugging hint, not the verdict), to make mismatch reports actionable.
fn diff_sample(order: &[String], exp: &[Row], act: &[Row]) -> String {
    let fmt_row = |r: &Row| -> String {
        let cells: Vec<String> = r
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                t.as_ref()
                    .map(|t| format!("?{}={t}", order.get(i).map(String::as_str).unwrap_or("?")))
            })
            .collect();
        format!("{{{}}}", cells.join(" "))
    };
    let keys = |rows: &[Row]| -> Vec<String> { rows.iter().map(fmt_row).collect() };
    let (ek, ak) = (keys(exp), keys(act));
    let only = |a: &[String], b: &[String]| -> Vec<String> {
        let mut pool = b.to_vec();
        a.iter()
            .filter(|k| {
                if let Some(p) = pool.iter().position(|x| x == *k) {
                    pool.remove(p);
                    false
                } else {
                    true
                }
            })
            .take(2)
            .cloned()
            .collect()
    };
    let miss = only(&ek, &ak);
    let extra = only(&ak, &ek);
    let mut s = String::new();
    if !miss.is_empty() {
        s.push_str(&format!("; expected-only e.g. {}", miss.join(", ")));
    }
    if !extra.is_empty() {
        s.push_str(&format!("; actual-only e.g. {}", extra.join(", ")));
    }
    s
}

fn align_binding(binding: &Binding, order: &[String]) -> Row {
    order
        .iter()
        .map(|v| {
            binding
                .iter()
                .find(|(bv, _)| bv == v)
                .map(|(_, t)| t.clone())
        })
        .collect()
}

/// CONSTRUCT evaluation test (T16): the expected result is an RDF *graph* document,
/// compared against the engine's constructed graph as triples-as-rows under the same
/// bnode-bijection machinery the update tests use (graphs are sets — unordered).
fn run_construct_test(entry: &TestEntry, query_text: &str, base: &str) -> Status {
    let Some(result_path) = entry.result_file.clone() else {
        return Status::Skip("no mf:result".into());
    };
    let mut expected: Vec<Row> = match parse_file(&result_path) {
        Ok(triples) => triples
            .into_iter()
            .map(|t| {
                vec![
                    Some(Term::from(t.subject)),
                    Some(Term::NamedNode(t.predicate)),
                    Some(t.object),
                ]
            })
            .collect(),
        Err(e) => return Status::Fail(format!("expected-graph parse error: {e}")),
    };
    dedup_rows(&mut expected); // an RDF graph is a set

    let nquads = match build_nquads(&entry.action.data, &entry.action.graph_data) {
        Ok(n) => n,
        Err(e) => return Status::Fail(format!("data load error: {e}")),
    };
    let query_with_base = with_base(query_text, base);

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = (|| {
            let graph = sparq_core::Graph::load_dataset(&nquads, "nquads")?;
            let triples = sparq_engine::construct(&graph, &query_with_base)?;
            let rows: Vec<Row> = triples
                .into_iter()
                .map(|t| {
                    vec![
                        Some(Term::from(t.subject)),
                        Some(Term::NamedNode(t.predicate)),
                        Some(t.object),
                    ]
                })
                .collect();
            Ok::<_, String>(rows)
        })();
        let _ = tx.send(result);
    });
    let actual = match rx.recv_timeout(TEST_TIMEOUT) {
        Ok(Ok(r)) => r, // the engine already dedups (set semantics)
        Ok(Err(e)) => return Status::Fail(format!("engine error: {e}")),
        Err(mpsc::RecvTimeoutError::Timeout) => return Status::Fail("timeout (20s)".into()),
        Err(mpsc::RecvTimeoutError::Disconnected) => return Status::Fail("engine panicked".into()),
    };

    match rows_equal(&expected, &actual, false) {
        Ok(true) => Status::Pass,
        Ok(false) => Status::Fail(format!(
            "graph mismatch: expected {} triple(s), got {}",
            expected.len(),
            actual.len()
        )),
        Err(e) => Status::Fail(e),
    }
}

pub fn run_update_test(entry: &TestEntry) -> Status {
    let Some(request_path) = entry.update_request.clone() else {
        return Status::Fail("manifest entry has no ut:request".into());
    };
    if !entry.update_pre.graph_data.is_empty() || !entry.update_post.graph_data.is_empty() {
        return Status::Skip("named graphs in update not supported".into());
    }
    let request_text = match std::fs::read_to_string(&request_path) {
        Ok(t) => t,
        Err(e) => return Status::Fail(format!("read request: {e}")),
    };
    let request = with_base(&request_text, &file_iri(&request_path));

    let nquads = match build_nquads(&entry.update_pre.data, &[]) {
        Ok(n) => n,
        Err(e) => return Status::Fail(format!("data load error: {e}")),
    };

    // Expected post-state (default graph).
    let mut expected: Vec<Row> = Vec::new();
    for (i, d) in entry.update_post.data.iter().enumerate() {
        match parse_file(d) {
            Ok(triples) => {
                for t in triples {
                    let t = prefix_bnodes(t, i);
                    expected.push(vec![
                        Some(Term::from(t.subject)),
                        Some(Term::NamedNode(t.predicate)),
                        Some(t.object),
                    ]);
                }
            }
            Err(e) => return Status::Fail(format!("expected-data parse error: {e}")),
        }
    }
    dedup_rows(&mut expected);

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = (|| {
            let graph = sparq_core::Graph::load_dataset(&nquads, "nquads")?;
            let updated = sparq_engine::update(&graph, &request)?;
            // Dump the resulting default graph.
            let scan = updated.store.scan(&[None, None, None]);
            let mut rows: Vec<Row> = Vec::new();
            for r in scan.rows.iter() {
                let [s, p, o] = scan.to_spo(r);
                rows.push(vec![
                    Some(updated.dict.term(s)),
                    Some(updated.dict.term(p)),
                    Some(updated.dict.term(o)),
                ]);
            }
            Ok::<_, String>(rows)
        })();
        let _ = tx.send(result);
    });
    let mut actual = match rx.recv_timeout(TEST_TIMEOUT) {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            // Operations the engine refuses (named-graph ops, USING, LOAD…) are
            // explicit "not yet supported" errors — report them as skips.
            return if e.contains("not yet supported") || e.contains("not supported") {
                Status::Skip(format!("engine: {e}"))
            } else {
                Status::Fail(format!("engine error: {e}"))
            };
        }
        Err(mpsc::RecvTimeoutError::Timeout) => return Status::Fail("timeout (20s)".into()),
        Err(mpsc::RecvTimeoutError::Disconnected) => return Status::Fail("engine panicked".into()),
    };
    dedup_rows(&mut actual);

    match rows_equal(&expected, &actual, false) {
        Ok(true) => Status::Pass,
        Ok(false) => Status::Fail(format!(
            "graph mismatch: expected {} triple(s), got {}",
            expected.len(),
            actual.len()
        )),
        Err(e) => Status::Fail(e),
    }
}

/// Syntactic (label-sensitive) dedup — RDF graphs are sets; the bnode
/// bijection in the comparison handles label differences across the two sides.
fn dedup_rows(rows: &mut Vec<Row>) {
    let mut seen = std::collections::HashSet::new();
    rows.retain(|r| {
        let key = r
            .iter()
            .map(|t| t.as_ref().map(|t| t.to_string()).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\u{1}");
        seen.insert(key)
    });
}
