//! The w3c/N3 community-group test suite (the manifests EYE and cwm run),
//! mapped onto the `sparq_reason::n3` engine:
//!
//! - `manifest-reasoner.ttl` — `test:TestN3Reason`: run the action document to
//!   its forward closure (`reason_n3_terms`), shape the output per the cwm
//!   options (`think`/`rules` + `data`/`conclusions`), and compare with the
//!   reference graph under blank-node isomorphism (formula terms compared
//!   structurally).
//! - `manifest-parser.ttl` + `manifest-extended.ttl` — N3 syntax (positive =
//!   must parse, negative = must be rejected) and `TestN3Eval` (parse →
//!   isomorphic statements).
//! - `TurtleTests/manifest.ttl` — Turtle through the N3 parser (N3 is a
//!   superset of Turtle): syntax both ways plus eval against the N-Triples
//!   expectation parsed by oxttl.
//!
//! cwm-option semantics honored: `data` drops formula-valued facts and rules
//! from the output; `conclusions` outputs ONLY derived facts; `think` vs
//! `rules` both run to fixpoint here (cwm's `--rules` is single-pass — the
//! suites' reference outputs coincide on these cases or fail loudly);
//! `strings` (log:outputString) is out of scope.

use super::report::{Outcome, TestResult};
use crate::rdf::{as_node, iri_to_path, MiniGraph};
use rustc_hash::FxHashMap;
use sparq_reason::n3::parser::{self, Parsed};
use sparq_reason::n3::Term as NTerm;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

const MF: &str = "http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#";
const TEST: &str = "https://w3c.github.io/N3/tests/test.n3#";
const RDFT: &str = "http://www.w3.org/ns/rdftest#";
const LOG_IMPLIES: &str = "http://www.w3.org/2000/10/swap/log#implies";
const LOG_IMPLIED_BY: &str = "http://www.w3.org/2000/10/swap/log#impliedBy";

const TEST_TIMEOUT: Duration = Duration::from_secs(10);

pub fn notes() -> Vec<String> {
    vec![
        "Source: w3c/N3 `tests/` (pinned clone). The reasoner manifest measures EYE/cwm \
         parity of the N3 rule engine; the parser/extended/Turtle manifests measure the N3 \
         parser subset (positive = must parse, negative = must be rejected). Reference \
         graphs are compared under blank-node isomorphism (formulae structurally); \
         `test:strings` (log:outputString) tests are out of scope."
            .to_string(),
    ]
}

pub fn run_suite(n3_root: &Path, out: &mut Vec<TestResult>) -> Result<(), String> {
    let tests = n3_root.join("tests");
    if !tests.is_dir() {
        return Err(format!(
            "{} not found — run scripts/fetch-inference-suites.sh",
            tests.display()
        ));
    }
    for (suite, manifest) in [
        ("n3/reasoner", "N3Tests/manifest-reasoner.ttl"),
        ("n3/parser", "N3Tests/manifest-parser.ttl"),
        ("n3/extended", "N3Tests/manifest-extended.ttl"),
        ("n3/turtle", "TurtleTests/manifest.ttl"),
    ] {
        run_manifest(&tests.join(manifest), suite, out)?;
    }
    Ok(())
}

fn run_manifest(path: &Path, suite: &str, out: &mut Vec<TestResult>) -> Result<(), String> {
    let g = MiniGraph::load(path)?;
    // Walk by declared type (robust against the one malformed entries-list
    // token upstream), in manifest order via the subjects' first occurrence.
    let mut seen = std::collections::HashSet::new();
    for t in &g.triples {
        let node = t.subject.clone();
        if !seen.insert(node.clone()) {
            continue;
        }
        let types = g.types_of(&node);
        let kind = types.iter().find_map(|ty| {
            ty.strip_prefix(TEST)
                .or_else(|| ty.strip_prefix(RDFT).and_then(|r| r.strip_prefix("Test")))
        });
        let Some(kind) = kind else { continue };
        let name = g
            .str_object(&node, &format!("{MF}name"))
            .unwrap_or_default();
        let name = if name.is_empty() || suite == "n3/reasoner" {
            // Reasoner mf:names repeat upstream; the entry id is the useful key.
            match &node {
                oxrdf::NamedOrBlankNode::NamedNode(n) => n
                    .as_str()
                    .rsplit(['#', '/'])
                    .next()
                    .unwrap_or(n.as_str())
                    .to_string(),
                oxrdf::NamedOrBlankNode::BlankNode(b) => format!("_:{}", b.as_str()),
            }
        } else {
            name
        };
        let file = |pred: &str| {
            g.object(&node, &format!("{MF}{pred}")).and_then(|t| match t {
                oxrdf::Term::NamedNode(n) => iri_to_path(n.as_str()),
                _ => None,
            })
        };
        let Some(action) = file("action") else {
            continue; // subjects without mf:action are manifest scaffolding
        };
        let result = file("result");
        let outcome = match kind {
            "TestN3PositiveSyntax" | "TurtlePositiveSyntax" => syntax_test(&action, true),
            "TestN3NegativeSyntax" | "TurtleNegativeSyntax" => syntax_test(&action, false),
            "TestN3Eval" => eval_test_n3(&action, result.as_deref()),
            "TurtleEval" => eval_test_turtle(&action, result.as_deref(), true),
            "TurtleNegativeEval" => eval_test_turtle(&action, result.as_deref(), false),
            "TestN3Reason" => reason_test(&g, &node, &action, result.as_deref()),
            other => Outcome::OutOfScope(format!("unhandled test type {other}")),
        };
        out.push(TestResult {
            suite: suite.to_string(),
            name,
            outcome,
        });
    }
    Ok(())
}

/// Parse on a watchdog thread (the recursive-descent parser may loop on
/// pathological input — that becomes a recorded FAIL, not a hung harness).
fn parse_with_watchdog(src: String, base: String) -> Result<Result<Parsed, String>, String> {
    let (tx, rx) = mpsc::channel();
    // Generous stack: the stress-test files nest deeply and the parser
    // recurses per nesting level (bounded by its MAX_DEPTH guard).
    let _ = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let result = std::panic::catch_unwind(|| parser::parse_with_base(&src, &base));
            let _ = tx.send(result);
        });
    match rx.recv_timeout(TEST_TIMEOUT) {
        Ok(Ok(r)) => Ok(r),
        Ok(Err(_)) => Err("parser panicked".into()),
        Err(mpsc::RecvTimeoutError::Timeout) => Err("timeout (10s) in parser".into()),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err("parser panicked".into()),
    }
}

fn read(path: &Path) -> Result<String, Outcome> {
    std::fs::read_to_string(path).map_err(|e| Outcome::Fail(format!("read {}: {e}", path.display())))
}

fn syntax_test(action: &Path, positive: bool) -> Outcome {
    let src = match read(action) {
        Ok(s) => s,
        Err(o) => return o,
    };
    let base = crate::rdf::file_iri(action);
    match parse_with_watchdog(src, base) {
        Ok(Ok(_)) if positive => Outcome::Pass,
        Ok(Ok(_)) => Outcome::Fail("negative syntax: parser accepted an invalid document".into()),
        Ok(Err(_)) if !positive => Outcome::Pass,
        Ok(Err(e)) => Outcome::Fail(format!("parse error: {e}")),
        Err(e) if !positive => {
            // A panic on invalid input is still a rejection, but parsers
            // should reject cleanly — keep it loud.
            Outcome::Fail(format!("rejected via {e} (should error cleanly)"))
        }
        Err(e) => Outcome::Fail(e),
    }
}

/// All statements of a parsed document, with rules re-encoded back into their
/// surface triple form so eval comparisons see them.
fn statements(p: &Parsed) -> Vec<[NTerm; 3]> {
    let mut stmts = p.facts.clone();
    for r in &p.rules {
        stmts.push([
            NTerm::Formula(r.premise.clone()),
            NTerm::Iri(LOG_IMPLIES.into()),
            NTerm::Formula(r.conclusion.clone()),
        ]);
    }
    for r in &p.backward_rules {
        stmts.push([
            NTerm::Formula(r.conclusion.clone()),
            NTerm::Iri(LOG_IMPLIED_BY.into()),
            NTerm::Formula(r.premise.clone()),
        ]);
    }
    stmts
}

fn eval_test_n3(action: &Path, result: Option<&Path>) -> Outcome {
    let Some(result) = result else {
        return Outcome::Fail("TestN3Eval without mf:result".into());
    };
    let (src, expected_src) = match (read(action), read(result)) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(o), _) | (_, Err(o)) => return o,
    };
    let parsed = match parse_with_watchdog(src, crate::rdf::file_iri(action)) {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => return Outcome::Fail(format!("action parse error: {e}")),
        Err(e) => return Outcome::Fail(e),
    };
    let expected = match parse_with_watchdog(expected_src, crate::rdf::file_iri(result)) {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => return Outcome::Fail(format!("expected parse error: {e}")),
        Err(e) => return Outcome::Fail(e),
    };
    if n3_iso(&statements(&parsed), &statements(&expected)) {
        Outcome::Pass
    } else {
        Outcome::Fail("parsed statements not isomorphic to the reference".into())
    }
}

fn eval_test_turtle(action: &Path, result: Option<&Path>, positive: bool) -> Outcome {
    let Some(result) = result else {
        return Outcome::Fail("eval test without mf:result".into());
    };
    let src = match read(action) {
        Ok(s) => s,
        Err(o) => return o,
    };
    let parsed = match parse_with_watchdog(src, crate::rdf::file_iri(action)) {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => {
            return if positive {
                Outcome::Fail(format!("action parse error: {e}"))
            } else {
                Outcome::Pass // rejecting the document certainly avoids the wrong graph
            };
        }
        Err(e) => return Outcome::Fail(e),
    };
    // Expected side: N-Triples/Turtle ground graph via oxttl.
    let expected = match crate::rdf::parse_file(result) {
        Ok(t) => t,
        Err(e) => return Outcome::Fail(format!("expected parse error: {e}")),
    };
    let expected_stmts: Vec<[NTerm; 3]> = expected
        .iter()
        .map(|t| {
            [
                oxrdf_to_n3(&oxrdf::Term::from(t.subject.clone())),
                oxrdf_to_n3(&oxrdf::Term::NamedNode(t.predicate.clone())),
                oxrdf_to_n3(&t.object),
            ]
        })
        .collect();
    let iso = n3_iso(&statements(&parsed), &expected_stmts);
    match (iso, positive) {
        (true, true) | (false, false) => Outcome::Pass,
        (false, true) => Outcome::Fail("parsed graph not isomorphic to the reference".into()),
        (true, false) => Outcome::Fail("negative eval: graphs are isomorphic".into()),
    }
}

fn oxrdf_to_n3(t: &oxrdf::Term) -> NTerm {
    match t {
        oxrdf::Term::NamedNode(n) => NTerm::Iri(n.as_str().to_string()),
        oxrdf::Term::BlankNode(b) => NTerm::Blank(b.as_str().to_string()),
        oxrdf::Term::Literal(l) => NTerm::Lit(
            l.value().to_string(),
            l.datatype().as_str().to_string(),
            l.language().map(|s| s.to_string()),
        ),
        oxrdf::Term::Triple(_) => NTerm::Iri("urn:sparq:unsupported-triple-term".into()),
    }
}

// ---------------------------------------------------------------------------
// TestN3Reason
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Options {
    data: bool,
    conclusions: bool,
    strings: bool,
    filter: bool,
}

fn options(g: &MiniGraph, node: &oxrdf::NamedOrBlankNode) -> Options {
    let mut o = Options::default();
    if let Some(opt_t) = g.object(node, &format!("{TEST}options")) {
        if let Some(opt) = as_node(opt_t) {
            let flag = |name: &str| {
                matches!(
                    g.str_object(&opt, &format!("{TEST}{name}")).as_deref(),
                    Some("true") | Some("1")
                )
            };
            o.data = flag("data");
            o.conclusions = flag("conclusions");
            o.strings = flag("strings");
            o.filter = g.object(&opt, &format!("{TEST}filter")).is_some();
        }
    }
    o
}

fn reason_test(
    g: &MiniGraph,
    node: &oxrdf::NamedOrBlankNode,
    action: &Path,
    result: Option<&Path>,
) -> Outcome {
    let opts = options(g, node);
    if opts.strings {
        return Outcome::OutOfScope("log:outputString (test:strings) not implemented".into());
    }
    if opts.filter {
        return Outcome::OutOfScope("test:filter (apply-and-replace) not implemented".into());
    }
    let Some(result) = result else {
        return Outcome::Fail("TestN3Reason without mf:result".into());
    };
    let (src, expected_src) = match (read(action), read(result)) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(o), _) | (_, Err(o)) => return o,
    };

    // Closure on a watchdog thread.
    let base = crate::rdf::file_iri(action);
    let (tx, rx) = mpsc::channel();
    {
        let (src, base) = (src.clone(), base.clone());
        let _ = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(move || {
                let r = std::panic::catch_unwind(|| sparq_reason::reason_n3_terms(&src, Some(&base)));
                let _ = tx.send(r);
            });
    }
    let closure = match rx.recv_timeout(TEST_TIMEOUT) {
        Ok(Ok(Ok(c))) => c,
        Ok(Ok(Err(e))) => return Outcome::Fail(format!("reasoner error: {e}")),
        Ok(Err(_)) => return Outcome::Fail("reasoner panicked".into()),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            return Outcome::Fail("timeout (10s) in reasoner".into())
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            return Outcome::Fail("reasoner panicked".into())
        }
    };

    // Shape the output per the cwm options.
    let mut actual: Vec<[NTerm; 3]> = if opts.conclusions {
        // `--conclusions`: ONLY what the rules derived.
        closure.derived.clone()
    } else {
        closure.facts.clone()
    };
    if opts.data || opts.conclusions {
        // `--data`: drop formula-valued statements (rules are already not in
        // the fact set).
        actual.retain(|row| !row.iter().any(|t| matches!(t, NTerm::Formula(_))));
    }
    dedup(&mut actual);
    if !opts.data && !opts.conclusions {
        // Full-store output keeps the document's rules as statements.
        let parsed = match parse_with_watchdog(src, base) {
            Ok(Ok(p)) => p,
            Ok(Err(e)) => return Outcome::Fail(format!("action parse error: {e}")),
            Err(e) => return Outcome::Fail(e),
        };
        actual.extend(statements(&Parsed {
            facts: Vec::new(),
            rules: parsed.rules,
            backward_rules: parsed.backward_rules,
        }));
    }

    let expected = match parse_with_watchdog(expected_src, crate::rdf::file_iri(result)) {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => return Outcome::Fail(format!("expected parse error: {e}")),
        Err(e) => return Outcome::Fail(e),
    };
    let mut expected_stmts = statements(&expected);
    dedup(&mut expected_stmts);

    if n3_iso(&actual, &expected_stmts) {
        Outcome::Pass
    } else {
        Outcome::Fail(format!(
            "output not isomorphic to the reference ({} vs {} statements{})",
            actual.len(),
            expected_stmts.len(),
            diff_hint(&actual, &expected_stmts)
        ))
    }
}

fn dedup(rows: &mut Vec<[NTerm; 3]>) {
    let mut seen = std::collections::HashSet::new();
    rows.retain(|r| seen.insert(format!("{r:?}")));
}

/// A small label-sensitive sample of one-side-only statements (debug hint).
fn diff_hint(actual: &[[NTerm; 3]], expected: &[[NTerm; 3]]) -> String {
    let fmt = |r: &[NTerm; 3]| format!("{r:?}");
    let a: Vec<String> = actual.iter().map(fmt).collect();
    let e: Vec<String> = expected.iter().map(fmt).collect();
    let mut s = String::new();
    if let Some(m) = e.iter().find(|x| !a.contains(x)) {
        s.push_str(&format!("; expected-only e.g. {m}"));
    }
    if let Some(m) = a.iter().find(|x| !e.contains(x)) {
        s.push_str(&format!("; actual-only e.g. {m}"));
    }
    s.truncate(400);
    s
}

// ---------------------------------------------------------------------------
// N3-term graph isomorphism (blank nodes AND variables under one bijection;
// formula terms compared as recursive statement multisets).
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Bij {
    fwd: FxHashMap<String, String>,
    rev: FxHashMap<String, String>,
    trail: Vec<String>,
}

impl Bij {
    fn bind(&mut self, a: String, b: String) -> bool {
        match (self.fwd.get(&a), self.rev.get(&b)) {
            (Some(x), Some(y)) => x == &b && y == &a,
            (None, None) => {
                self.fwd.insert(a.clone(), b.clone());
                self.rev.insert(b, a.clone());
                self.trail.push(a);
                true
            }
            _ => false,
        }
    }
    fn mark(&self) -> usize {
        self.trail.len()
    }
    fn undo(&mut self, mark: usize) {
        while self.trail.len() > mark {
            let a = self.trail.pop().unwrap();
            if let Some(b) = self.fwd.remove(&a) {
                self.rev.remove(&b);
            }
        }
    }
}

const ISO_BUDGET: usize = 2_000_000;

pub(crate) fn n3_iso(a: &[[NTerm; 3]], b: &[[NTerm; 3]]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut bij = Bij::default();
    let mut used = vec![false; b.len()];
    let mut steps = 0usize;
    iso_search(0, a, b, &mut used, &mut bij, &mut steps)
}

fn iso_search(
    i: usize,
    a: &[[NTerm; 3]],
    b: &[[NTerm; 3]],
    used: &mut [bool],
    bij: &mut Bij,
    steps: &mut usize,
) -> bool {
    if i == a.len() {
        return true;
    }
    for j in 0..b.len() {
        if used[j] {
            continue;
        }
        *steps += 1;
        if *steps > ISO_BUDGET {
            return false;
        }
        let mark = bij.mark();
        if (0..3).all(|k| term_iso(&a[i][k], &b[j][k], bij, steps)) {
            used[j] = true;
            if iso_search(i + 1, a, b, used, bij, steps) {
                return true;
            }
            used[j] = false;
        }
        bij.undo(mark);
    }
    false
}

fn term_iso(a: &NTerm, b: &NTerm, bij: &mut Bij, steps: &mut usize) -> bool {
    match (a, b) {
        (NTerm::Blank(x), NTerm::Blank(y)) => bij.bind(format!("b:{x}"), format!("b:{y}")),
        (NTerm::Var(x), NTerm::Var(y)) => bij.bind(format!("v:{x}"), format!("v:{y}")),
        (NTerm::Formula(x), NTerm::Formula(y)) => {
            if x.len() != y.len() {
                return false;
            }
            let mut used = vec![false; y.len()];
            iso_search(0, x, y, &mut used, bij, steps)
        }
        _ => a == b,
    }
}
