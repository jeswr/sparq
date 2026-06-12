//! Test chunk 0 for prod:PositionalVar
//! Contains 24 tests

use dep::xpath::stub_prodpositionalvar;

#[test]
fn k_forexprpositionalvar_6() {
    //  Verify positional variable with fn:deep-equal().
    // XPath: deep-equal((1, 2, 3), for $i at $p in (1, 2, 3) return $p)
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_7() {
    //  Extract the EBV from a positional variable.
    // XPath: deep-equal((true(), true()), for $i at $p in (1, 2) return b
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_9() {
    //  Verify that the position is properly computed for fn:string-to-codepoints().
    // XPath: deep-equal((1, 2, 3), for $i at $p in string-to-codepoints(\
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_10() {
    //  Verify that the position is properly computed for fn:string-to-codepoints().
    // XPath: deep-equal((1, 2, 3), for $i at $p in distinct-values((1, 2,
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_12() {
    //  Verify that the position is properly computed for fn:insert-before().
    // XPath: deep-equal((1, 2, 3, 4), for $i at $p in insert-before((1, c
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_13() {
    //  Verify that the position is properly computed for fn:insert-before()(#2).
    // XPath: deep-equal((1, 2, 3, 4), for $i at $p in insert-before((1, c
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_14() {
    //  Verify that the position is properly computed for fn:insert-before()(#3).
    // XPath: deep-equal((1, 2, 3, 4), for $i at $p in insert-before((1, c
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_15() {
    //  Verify that the position is properly computed for the range expression.
    // XPath: deep-equal((1, 2, 3, 4), for $i at $p in 1 to 4 return $p)
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_16() {
    //  Verify that the position is properly computed for the range expression(#2).
    // XPath: deep-equal((1, 2, 3, 4), for $i at $p in -10 to -7 return $p
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_17() {
    //  Verify that the position is properly computed for fn:remove().
    // XPath: deep-equal((1, 2, 3), for $i at $p in remove((1, 2, 3, curre
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_18() {
    //  Verify that the position is properly computed for fn:remove().
    // XPath: deep-equal((1, 2, 3), for $i at $p in remove((1, 2, 3, curre
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_19() {
    //  Verify that the position is properly computed for fn:remove().
    // XPath: deep-equal((1, 2, 3), for $i at $p in remove((1, 2, current-
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_20() {
    //  Verify that the position is properly computed for fn:remove().
    // XPath: deep-equal((1, 2, 3), for $i at $p in remove((1, 2, current-
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_21() {
    //  Verify that the position is properly computed for fn:remove().
    // XPath: deep-equal((1, 2, 3), for $i at $p in remove((1, 2, 3, curre
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_22() {
    //  Verify that the position is properly computed for fn:remove().
    // XPath: deep-equal((1, 2, 3), for $i at $p in remove((1, 2, 3, curre
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_23() {
    //  Verify that the position is properly computed for fn:subsequence().
    // XPath: 1 eq (for $i at $p in subsequence((1, 2, 3, current-time()),
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_26() {
    //  Verify that the position is properly computed for fn:subsequence().
    // XPath: deep-equal((1, 2), for $i at $p in subsequence((1, 2, 3, cur
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_27() {
    //  Verify that the position is properly computed for fn:subsequence().
    // XPath: deep-equal((1, 2), for $i at $p in subsequence((1, 2, 3, cur
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_28() {
    //  Verify that the position is properly computed for fn:subsequence().
    // XPath: deep-equal((1, 2), for $i at $p in subsequence((1, 2, 3, cur
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_29() {
    //  Verify that the position is properly computed for a singleton value.
    // XPath: 1 eq (for $i at $p in 0 return $p)
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k_forexprpositionalvar_30() {
    //  Evaluate the positional and binding expression at the same time.
    // XPath: deep-equal(for $i at $p in (1, 2, 3, 4) return ($i, $p), (1,
    // Expected: true
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k2_forexprpositionalvar_1() {
    //  Combine fn:remove() and a positional for-variable.
    // XPath: for $i at $p in remove((1, 2, 3), 10) return $p
    // Expected: 1 2 3
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k2_forexprpositionalvar_3() {
    //  Usage of two positional variable references.
    // XPath: let $tree := <e> <a id=\"1\"/> <a id=\"2\"/> <a id=\"3\"/> <
    // Expected: true 1 true 2 true 3
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}

#[test]
fn k2_forexprpositionalvar_4() {
    //  Cache a positional variable with a let-binding.
    // XPath: for $i at $pos in (3 to 6) let $let := $pos + 1 return ($let
    // Expected: 2 1 3 2 4 3 5 4
    // Calls stub function - will fail until prod:PositionalVar is implemented
    let _ = stub_prodpositionalvar();
}
