//! Test chunk 0 for prod:ParenthesizedExpr
//! Contains 14 tests

use dep::xpath::stub_prodparenthesizedexpr;

#[test]
fn parenexpr_1() {
    //  Arithmetic operations
    // XPath: (1 + 2) * 3
    // Expected: 9
    // Calls stub function - will fail until prod:ParenthesizedExpr is implemented
    let _ = stub_prodparenthesizedexpr();
}

#[test]
fn parenexpr_2() {
    //  Arithmetic operations
    // XPath: 1 + (2 * 3)
    // Expected: 7
    // Calls stub function - will fail until prod:ParenthesizedExpr is implemented
    let _ = stub_prodparenthesizedexpr();
}

#[test]
fn parenexpr_3() {
    //  Arithmetic operations
    // XPath: -(2 + 5)
    // Expected: -7
    // Calls stub function - will fail until prod:ParenthesizedExpr is implemented
    let _ = stub_prodparenthesizedexpr();
}

#[test]
fn parenexpr_4() {
    //  Arithmetic operations
    // XPath: (-2) + 5
    // Expected: 3
    // Calls stub function - will fail until prod:ParenthesizedExpr is implemented
    let _ = stub_prodparenthesizedexpr();
}

#[test]
fn parenexpr_5() {
    //  Arithmetic operations
    // XPath: 2 + (4 idiv 2)
    // Expected: 4
    // Calls stub function - will fail until prod:ParenthesizedExpr is implemented
    let _ = stub_prodparenthesizedexpr();
}

#[test]
fn parenexpr_6() {
    //  Arithmetic operations
    // XPath: (2 + 4) idiv 2
    // Expected: 3
    // Calls stub function - will fail until prod:ParenthesizedExpr is implemented
    let _ = stub_prodparenthesizedexpr();
}

#[test]
fn parenexpr_7() {
    //  Arithmetic operations
    // XPath: 2 * (5 mod 3)
    // Expected: 4
    // Calls stub function - will fail until prod:ParenthesizedExpr is implemented
    let _ = stub_prodparenthesizedexpr();
}

#[test]
fn parenexpr_8() {
    //  Arithmetic operations
    // XPath: (2 * 5) mod 3
    // Expected: 1
    // Calls stub function - will fail until prod:ParenthesizedExpr is implemented
    let _ = stub_prodparenthesizedexpr();
}

#[test]
fn parenexpr_9() {
    //  Logical expressions
    // XPath: (fn:true() or fn:true()) and fn:false()
    // Expected: false
    // Calls stub function - will fail until prod:ParenthesizedExpr is implemented
    let _ = stub_prodparenthesizedexpr();
}

#[test]
fn parenexpr_10() {
    //  Logical expressions
    // XPath: fn:true() or (fn:true() and fn:false())
    // Expected: true
    // Calls stub function - will fail until prod:ParenthesizedExpr is implemented
    let _ = stub_prodparenthesizedexpr();
}

#[test]
fn parenexpr_11() {
    //  FLWOR expression
    // XPath: (for $x in (1) where (fn:true()) order by ($x) return ($x))
    // Expected: 1
    // Calls stub function - will fail until prod:ParenthesizedExpr is implemented
    let _ = stub_prodparenthesizedexpr();
}

#[test]
fn parenexpr_12() {
    //  if expression
    // XPath: for $x in (1,2) return (if (($x eq 1)) then ($x) else ($x +
    // Expected: 1 3
    // Calls stub function - will fail until prod:ParenthesizedExpr is implemented
    let _ = stub_prodparenthesizedexpr();
}

#[test]
fn parenexpr_13() {
    //  Literal
    // XPath: (1)
    // Expected: 1
    // Calls stub function - will fail until prod:ParenthesizedExpr is implemented
    let _ = stub_prodparenthesizedexpr();
}

#[test]
fn parenexpr_14() {
    //  Sequence
    // XPath: (1, (2, (3, 4)), (5))
    // Expected: 1 2 3 4 5
    // Calls stub function - will fail until prod:ParenthesizedExpr is implemented
    let _ = stub_prodparenthesizedexpr();
}
