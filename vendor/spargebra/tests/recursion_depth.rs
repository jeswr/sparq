// [OPUS-4.8] The vendored crate's `[lints.clippy]` table is pedantic and aimed
// at library code; this is an integration-test crate, so a handful of those
// lints are inappropriate here and are relaxed file-wide:
//   - tests_outside_test_module: an integration test crate IS the test module;
//     there is no library code to separate the tests from.
//   - expect_used: panicking on the unexpected is exactly what a test wants.
//   - assertions_on_result_states: `assert!(... .is_err())` is the clearest way
//     to state "this query must be rejected" and is the whole point of the test.
//   - single_char_add_str / unseparated_literal_suffix: cosmetic.
// (None of these gate in the real build: the project lints spargebra as a
//  `[patch.crates-io]` dependency, and clippy does not lint dependency targets.)
#![allow(
    clippy::tests_outside_test_module,
    clippy::expect_used,
    clippy::assertions_on_result_states,
    clippy::single_char_add_str,
    clippy::unseparated_literal_suffix
)]

// [OPUS-4.8] Regression tests for the SPARQL parser recursion-depth cap
// (bead sq-v5dg, threat-model boundary B2 / T-PARSE-DoS).
//
// Before this guard, `spargebra`'s `peg` recursive-descent parser mapped each
// level of `{ }` / `( )` / `[ ]` / `<<( )>>` nesting onto native call-stack
// frames with no bound, so an attacker-controlled query string of thousands of
// nested delimiters overflowed the stack and ABORTED the process — a DoS
// reachable from the unauthenticated server. The parser now returns a clean
// `Err` ("too deeply nested") instead.
//
// Each pathological case is run on a deliberately small (2 MiB) thread stack —
// the size of a tokio worker / blocking-pool thread, which is where the server
// actually parses — to *prove* the input is rejected gracefully rather than
// overflowing. If the cap were absent or ineffective, the spawned thread would
// abort the whole test process (SIGABRT), failing the suite loudly.

use spargebra::SparqlParser;

/// Run `f` on a 2 MiB stack (tokio's default worker/blocking-pool stack). A
/// stack overflow there aborts the process, so reaching the `join()` at all is
/// itself part of the assertion: the parser must have returned, not recursed to
/// death.
fn on_small_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(f)
        .expect("spawn 2 MiB-stack thread")
        .join()
        .expect("parser thread must return, not overflow the stack")
}

fn parse_ok(query: &str) -> bool {
    SparqlParser::new()
        .with_prefix("", "http://example.com/")
        .unwrap()
        .parse_query(query)
        .is_ok()
}

/// A pathologically nested input must be rejected with a clean parse error,
/// never a stack-overflow abort. 8000 levels is ~9x the empirical 2 MiB
/// release-build overflow point (~900) and ~44x the debug-build one (~180), so
/// without the cap this would reliably abort the process.
const PATHOLOGICAL: usize = 8000;

#[test]
fn deeply_nested_group_graph_patterns_error_not_overflow() {
    let err = on_small_stack(|| {
        let mut q = String::from("SELECT * WHERE ");
        for _ in 0..PATHOLOGICAL {
            q.push('{');
        }
        q.push_str(" ?s ?p ?o ");
        for _ in 0..PATHOLOGICAL {
            q.push('}');
        }
        SparqlParser::new().parse_query(&q).err()
    });
    let err = err.expect("a pathologically nested query must be a parse Err, not Ok");
    assert!(
        err.to_string().contains("too deeply nested"),
        "expected the recursion-depth error, got: {err}"
    );
}

#[test]
fn deeply_nested_bracketed_expressions_error_not_overflow() {
    let err = on_small_stack(|| {
        let mut q = String::from("SELECT * WHERE { FILTER(");
        for _ in 0..PATHOLOGICAL {
            q.push('(');
        }
        q.push('1');
        for _ in 0..PATHOLOGICAL {
            q.push(')');
        }
        q.push_str(") }");
        SparqlParser::new().parse_query(&q).err()
    });
    let err = err.expect("a pathologically nested expression must be a parse Err, not Ok");
    assert!(
        err.to_string().contains("too deeply nested"),
        "expected the recursion-depth error, got: {err}"
    );
}

#[test]
fn deeply_nested_unary_not_error_not_overflow() {
    on_small_stack(|| {
        let mut q = String::from("SELECT * WHERE { FILTER(");
        for _ in 0..PATHOLOGICAL {
            q.push('!');
        }
        q.push_str("true) }");
        // sparql-12 allows `!!`; whether this is the depth error or a different
        // syntax error, it MUST be an Err and MUST NOT overflow the stack.
        assert!(SparqlParser::new().parse_query(&q).is_err());
    });
}

#[test]
fn deeply_nested_property_paths_error_not_overflow() {
    on_small_stack(|| {
        let mut q = String::from("SELECT * WHERE { ?s ");
        for _ in 0..PATHOLOGICAL {
            q.push('(');
        }
        q.push_str("<http://example.com/p>");
        for _ in 0..PATHOLOGICAL {
            q.push(')');
        }
        q.push_str(" ?o }");
        assert!(SparqlParser::new().parse_query(&q).is_err());
    });
}

#[test]
fn deeply_nested_collections_error_not_overflow() {
    on_small_stack(|| {
        let mut q = String::from("SELECT * WHERE { ?s ?p ");
        for _ in 0..PATHOLOGICAL {
            q.push('(');
        }
        q.push_str(" 1 ");
        for _ in 0..PATHOLOGICAL {
            q.push(')');
        }
        q.push_str(" }");
        assert!(SparqlParser::new().parse_query(&q).is_err());
    });
}

#[test]
fn deeply_nested_blank_node_property_lists_error_not_overflow() {
    on_small_stack(|| {
        let mut q = String::from("SELECT * WHERE { ?s ?p ");
        for _ in 0..PATHOLOGICAL {
            q.push_str("[ <http://example.com/p> ");
        }
        q.push_str("1");
        for _ in 0..PATHOLOGICAL {
            q.push_str(" ]");
        }
        q.push_str(" }");
        assert!(SparqlParser::new().parse_query(&q).is_err());
    });
}

#[test]
fn deeply_nested_triple_terms_error_not_overflow() {
    on_small_stack(|| {
        // <<( <<( ... :p :o )>> :p :o )>>  (expression position, inside BIND)
        let mut q = String::from("SELECT * WHERE { BIND(");
        for _ in 0..PATHOLOGICAL {
            q.push_str("<<( ");
        }
        q.push_str("<http://example.com/s> <http://example.com/p> <http://example.com/o>");
        for _ in 0..PATHOLOGICAL {
            q.push_str(" <http://example.com/p> <http://example.com/o> )>>");
        }
        q.push_str(" AS ?x) }");
        assert!(SparqlParser::new().parse_query(&q).is_err());
    });
}

#[test]
fn nested_update_template_collections_error_not_overflow() {
    // The recursion-depth cap must also protect the UPDATE parse path, whose
    // INSERT/DELETE templates reuse the guarded Collection / BNode-list / group
    // productions.
    on_small_stack(|| {
        let mut q = String::from("INSERT DATA { <http://example.com/s> <http://example.com/p> ");
        for _ in 0..PATHOLOGICAL {
            q.push('(');
        }
        q.push_str(" 1 ");
        for _ in 0..PATHOLOGICAL {
            q.push(')');
        }
        q.push_str(" }");
        assert!(SparqlParser::new().parse_update(&q).is_err());
    });
}

// --- Positive controls: legitimately deep nesting (within the cap) still parses.

#[test]
fn legitimately_deep_group_nesting_still_parses() {
    // Deep but well under the 128 cap (and far under any conformance query).
    let depth = 64;
    let mut q = String::from("SELECT * WHERE ");
    for _ in 0..depth {
        q.push('{');
    }
    q.push_str(" ?s ?p ?o ");
    for _ in 0..depth {
        q.push('}');
    }
    assert!(parse_ok(&q), "a {depth}-deep group query within the cap must parse");
}

#[test]
fn legitimately_deep_expression_nesting_still_parses() {
    let depth = 64;
    let mut q = String::from("SELECT * WHERE { FILTER(");
    for _ in 0..depth {
        q.push('(');
    }
    q.push('1');
    for _ in 0..depth {
        q.push(')');
    }
    q.push_str(") }");
    assert!(parse_ok(&q), "a {depth}-deep expression within the cap must parse");
}

#[test]
fn ordinary_queries_unaffected() {
    // A representative spread of ordinary queries the cap must never touch.
    for q in [
        "SELECT ?s ?p ?o WHERE { ?s ?p ?o }",
        "SELECT * WHERE { { ?s ?p ?o } UNION { ?a ?b ?c } }",
        "SELECT * WHERE { ?s ?p ?o OPTIONAL { ?s ?x ?y FILTER(?y > 1) } }",
        "SELECT (COUNT(?s) AS ?n) WHERE { ?s ?p ?o } GROUP BY ?p",
        "SELECT * WHERE { ?s (<http://example.com/p>/<http://example.com/q>)+ ?o }",
        "SELECT * WHERE { ?s ?p ( 1 2 3 ) }",
        "SELECT * WHERE { [ <http://example.com/p> [ <http://example.com/q> 1 ] ] ?p ?o }",
        "SELECT * WHERE { { SELECT ?s WHERE { ?s ?p ?o } } }",
        "ASK { ?s ?p ?o FILTER EXISTS { ?s ?x ?y } }",
    ] {
        assert!(parse_ok(q), "ordinary query must still parse: {q}");
    }
}

/// The cap must be high enough that the deepest level a query nests to right at
/// the boundary still works, and one past it fails — pinning the limit so a
/// future edit that silently lowers it is caught. We don't hard-code 128 here;
/// we only assert the boundary is *somewhere* well above ordinary use and that
/// crossing it flips OK -> Err. (Group depth N uses one guard level per `{`.)
#[test]
fn cap_boundary_flips_ok_to_err() {
    let mut last_ok = 0usize;
    let mut first_err = 0usize;
    for depth in (8..=512).step_by(8) {
        let mut q = String::from("SELECT * WHERE ");
        for _ in 0..depth {
            q.push('{');
        }
        q.push_str(" ?s ?p ?o ");
        for _ in 0..depth {
            q.push('}');
        }
        if SparqlParser::new().parse_query(&q).is_ok() {
            last_ok = depth;
        } else if first_err == 0 {
            first_err = depth;
            break;
        }
    }
    assert!(
        last_ok >= 64,
        "the cap must allow at least 64 levels of group nesting (got last_ok={last_ok})"
    );
    assert!(
        first_err > last_ok && first_err <= 256,
        "crossing the cap must flip OK->Err in a sane range (last_ok={last_ok}, first_err={first_err})"
    );
}
