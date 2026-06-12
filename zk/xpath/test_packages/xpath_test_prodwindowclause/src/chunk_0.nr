//! Test chunk 0 for prod:WindowClause
//! Contains 13 tests

use dep::xpath::stub_prodwindowclause;

#[test]
fn slidingwindowexpr501a() {
    // Variant of 501 in push mode
    // XPath: <out>{for sliding window $w in (1, 2, 3, 4)        start at
    // Expected: 1 2 2 3 3 4 4
    // Calls stub function - will fail until prod:WindowClause is implemented
    let _ = stub_prodwindowclause();
}

#[test]
fn slidingwindowexpr502a() {
    // Variant of 502 in push mode
    // XPath: <out>{for sliding window $w in (1, 2, 3, 4)        start at
    // Expected: 1 2 2 3 3 4
    // Calls stub function - will fail until prod:WindowClause is implemented
    let _ = stub_prodwindowclause();
}

#[test]
fn slidingwindowexpr505a() {
    // Variation of previous test in push mode
    // XPath: <out>{for sliding window $w in (1, 2, 3, 4)        start at
    // Expected: 1 2 3 4
    // Calls stub function - will fail until prod:WindowClause is implemented
    let _ = stub_prodwindowclause();
}

#[test]
fn tumblingwindowexpr505a() {
    // Variation of previous test in push mode
    // XPath: <out>{for tumbling window $w in (1, 2, 3, 4)        start at
    // Expected: 1
    // Calls stub function - will fail until prod:WindowClause is implemented
    let _ = stub_prodwindowclause();
}

#[test]
fn slidingwindowexpr506a() {
    // Variation of previous test in push mode
    // XPath: <out>{           for sliding window $w in (1, 2, 3, 4, 14, 1
    // Expected: 1-2-3-4-14-13-12-11 2-3-4-14-13-12 3-4-1
    // Calls stub function - will fail until prod:WindowClause is implemented
    let _ = stub_prodwindowclause();
}

#[test]
fn slidingwindowexpr509() {
    // Sliding Window Clause, average window size
    // XPath: avg(               for sliding window $w in (1, 2, \"london\
    // Expected: 1.5
    // Calls stub function - will fail until prod:WindowClause is implemented
    let _ = stub_prodwindowclause();
}

#[test]
fn tumblingwindowexpr509() {
    // Tumbling Window Clause, average window size
    // XPath: avg(               for tumbling window $w in (1, 2, \"london
    // Expected: 2
    // Calls stub function - will fail until prod:WindowClause is implemented
    let _ = stub_prodwindowclause();
}

#[test]
fn tumblingwindowexpr529() {
    // Tumbling Window Clause, all variables are in scope in the following where clause
    // XPath: for tumbling window $w in (1 to 10)           start $s at $x
    // Expected: true
    // Calls stub function - will fail until prod:WindowClause is implemented
    let _ = stub_prodwindowclause();
}

#[test]
fn slidingwindowexpr529() {
    // Sliding Window Clause, all variables are in scope in the following where clause
    // XPath: for sliding window $w in (1 to 10)           start $s at $x
    // Expected: true
    // Calls stub function - will fail until prod:WindowClause is implemented
    let _ = stub_prodwindowclause();
}

#[test]
fn tumblingwindowexpr534() {
    // Tumbling Window Clause, overwrite all variables of preceding for clauses
    // XPath: for $x1 in 11           for $x2 in 12           for $x3 in 1
    // Expected: 1 2 3 4 5 6 7 8 9 10 1 1 2 10 10 9
    // Calls stub function - will fail until prod:WindowClause is implemented
    let _ = stub_prodwindowclause();
}

#[test]
fn slidingwindowexpr534() {
    // Sliding Window Clause, overwrite all variables of preceding for clauses
    // XPath: for $x1 in 11           for $x2 in 12           for $x3 in 1
    // Expected: 1 2 3 4 5 6 7 8 9 10 1 1 2 10 10 9 2 3 4
    // Calls stub function - will fail until prod:WindowClause is implemented
    let _ = stub_prodwindowclause();
}

#[test]
fn tumblingwindowexpr560() {
    // Tumbling Window Clause. Saxon bug 4330.
    // XPath: declare namespace output = \"http://www.w3.org/2010/xslt-xqu
    // Expected: true
    // Calls stub function - will fail until prod:WindowClause is implemented
    let _ = stub_prodwindowclause();
}

#[test]
fn slidingwindowexpr563a() {
    // Variation of previous test in push mode
    // XPath: <out>{           for sliding window $w in (0, 1, 2, 3, 4, 14
    // Expected: 1-2-3-4-14-13-12-11 2-3-4-14-13-12 3-4-1
    // Calls stub function - will fail until prod:WindowClause is implemented
    let _ = stub_prodwindowclause();
}
