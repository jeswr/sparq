//! Test chunk 0 for prod:UnorderedExpr
//! Contains 11 tests

use dep::xpath::stub_produnorderedexpr;

#[test]
fn orderexpr_15() {
    //  fn:subsequence ordered
    // XPath: ordered {fn:subsequence((1,2,3,4),2,2)}
    // Expected: 2 3
    // Calls stub function - will fail until prod:UnorderedExpr is implemented
    let _ = stub_produnorderedexpr();
}

#[test]
fn orderexpr_17() {
    //  fn:reverse ordered
    // XPath: ordered {fn:reverse((3,2))}
    // Expected: 2 3
    // Calls stub function - will fail until prod:UnorderedExpr is implemented
    let _ = stub_produnorderedexpr();
}

#[test]
fn orderexpr_19() {
    //  FLWOR ordered
    // XPath: ordered { for $i in (//part[@partid = 1], //part[@partid = 2
    // Expected: 4 6
    // Calls stub function - will fail until prod:UnorderedExpr is implemented
    let _ = stub_produnorderedexpr();
}

#[test]
fn orderedunorderedexpr_1() {
    //  Evaluation of ordered expression together with if expression ('some' operator).
    // XPath: ordered {if (fn:true()) then (0,1,2,3,4) else (\"A\",\"B\",\
    // Expected: 0 1 2 3 4
    // Calls stub function - will fail until prod:UnorderedExpr is implemented
    let _ = stub_produnorderedexpr();
}

#[test]
fn orderedunorderedexpr_2() {
    //  Evaluation of ordered expression used with 'or' expression ('and' operator).
    // XPath: ordered {if (1 eq 1 and 2 eq 2) then (0,1,2,3,4) else (\"a\"
    // Expected: 0 1 2 3 4
    // Calls stub function - will fail until prod:UnorderedExpr is implemented
    let _ = stub_produnorderedexpr();
}

#[test]
fn orderedunorderedexpr_3() {
    //  Evaluation of ordered expression used with 'or' expression ('or' operator).
    // XPath: ordered {if (1 eq 1 or 2 eq 3) then (0,1,2,3,4) else (\"a\",
    // Expected: 0 1 2 3 4
    // Calls stub function - will fail until prod:UnorderedExpr is implemented
    let _ = stub_produnorderedexpr();
}

#[test]
fn orderedunorderedexpr_4() {
    //  Evaluation of ordered expression used with quantified expression.
    // XPath: ordered {if (some $x in (1, 2, 3), $y in (2, 3, 4) satisfies
    // Expected: 0 1 2 3 4
    // Calls stub function - will fail until prod:UnorderedExpr is implemented
    let _ = stub_produnorderedexpr();
}

#[test]
fn orderedunorderedexpr_5() {
    //  Evaluation of ordered expression used with quantified expression.
    // XPath: ordered {if (every $x in (1, 2, 3) satisfies $x < 4) then (0
    // Expected: 0 1 2 3 4
    // Calls stub function - will fail until prod:UnorderedExpr is implemented
    let _ = stub_produnorderedexpr();
}

#[test]
fn orderedunorderedexpr_6() {
    //  Evaluation of ordered expression used with typeswitch expression.
    // XPath: ordered {typeswitch(123) case $i as xs:string return (\"a\",
    // Expected: 1 2 3 4
    // Calls stub function - will fail until prod:UnorderedExpr is implemented
    let _ = stub_produnorderedexpr();
}

#[test]
fn k_orderexpr_3() {
    //  A simple test of 'ordered{}'.
    // XPath: ordered{true()}
    // Expected: true
    // Calls stub function - will fail until prod:UnorderedExpr is implemented
    let _ = stub_produnorderedexpr();
}

#[test]
fn k_orderexpr_4() {
    //  A simple test of 'unordered{}'.
    // XPath: unordered{true()}
    // Expected: true
    // Calls stub function - will fail until prod:UnorderedExpr is implemented
    let _ = stub_produnorderedexpr();
}
