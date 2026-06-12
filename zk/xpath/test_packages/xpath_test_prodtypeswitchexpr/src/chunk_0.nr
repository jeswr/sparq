//! Test chunk 0 for prod:TypeswitchExpr
//! Contains 28 tests

use dep::xpath::stub_prodtypeswitchexpr;

#[test]
fn typeswitchhc17() {
    //  Simple test for evaluation of atomic value (integer) and return contains intege
    // XPath: typeswitch (5)          case $i as xs:integer return xs:inte
    // Expected: 2
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn typeswitchhc18() {
    //  Simple test for evaluation of atomic value (decimal) and return clause conatini
    // XPath: typeswitch (5.1)          case $i as xs:decimal return xs:de
    // Expected: 4.2
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn typeswitchhc19() {
    //  Simple test for evaluation of atomic value (double) and return clause containin
    // XPath: typeswitch (5.1E2)          case $i as xs:integer return <wr
    // Expected: 620
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn typeswitchhc20() {
    //  Simple test for evaluation of atomic value (string) and return clause containin
    // XPath: typeswitch (\"A String\")          case $i as xs:decimal ret
    // Expected: 8
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn typeswitchhc21() {
    //  Simple test for evaluation of atomic value ('1') and return clause containing a
    // XPath: typeswitch (1)          case $i as xs:double return <wrap>te
    // Expected: 3
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn typeswitchhc22() {
    //  Simple test for evaluation of atomic value ('1') and return clause containing a
    // XPath: typeswitch (1)          case $i as xs:double return <wrap>te
    // Expected: 2
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn typeswitchhc23() {
    //  Simple test for evaluation of atomic value ('1') and return clause containing a
    // XPath: typeswitch (1)          case $i as xs:double return <wrap>te
    // Expected: 10
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn typeswitchhc24() {
    //  Simple test for evaluation of atomic value ('1') and return clause containing a
    // XPath: typeswitch (1)          case $i as xs:double return <wrap>te
    // Expected: 5
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn typeswitchhc25() {
    //  Simple test for evaluation of atomic value ('1') and return clause containing a
    // XPath: typeswitch (1)          case $i as xs:double return <wrap>te
    // Expected: 5
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn k_sequenceexprtypeswitch_1() {
    //  typeswitch test where the sequence types only differs in cardinality.
    // XPath: (typeswitch((1, 2)) case xs:integer return -1 case xs:intege
    // Expected: true
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn k_sequenceexprtypeswitch_2() {
    //  A string literal is of type xs:string, even though it can be promoted to xs:any
    // XPath: (typeswitch(\"a string\") case xs:anyURI return -1 case xs:s
    // Expected: true
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn k_sequenceexprtypeswitch_3() {
    //  typeswitch test: A string literal is of type xs:string, even though it can be p
    // XPath: (typeswitch(\"a string\") case xs:untypedAtomic return -1 ca
    // Expected: true
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn k_sequenceexprtypeswitch_4() {
    //  A sequence of atomic items are not treated individually, but as a whole.
    // XPath: (typeswitch((1, \"a string\")) case xs:integer return -1 cas
    // Expected: true
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn k_sequenceexprtypeswitch_5() {
    //  A typeswitch scenario which in some implementations trigger certain optimizatio
    // XPath: (typeswitch(((1, current-time())[1])) case element() return
    // Expected: true
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn k_sequenceexprtypeswitch_6() {
    //  A typeswitch scenario involving empty-sequence(). Both the 'xs:integer*' branch
    // XPath: (typeswitch(()) case xs:integer* return 1 case empty-sequenc
    // Expected: true
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn k_sequenceexprtypeswitch_7() {
    //  A typeswitch with an operand expression being the comma operator using no paran
    // XPath: (typeswitch(1, 2, 3) case xs:string+ return -1 case xs:integ
    // Expected: true
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn k_sequenceexprtypeswitch_8() {
    //  A typeswitch where the case clauses will never be evaluated. In some implementa
    // XPath: (typeswitch(1, 2, current-time()) case element() return -1 c
    // Expected: true
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn k2_sequenceexprtypeswitch_1() {
    //  Extract the EBV from the result of a typeswitch.
    // XPath: boolean(typeswitch (current-time(), 1, 3e3, \"foo\") case no
    // Expected: true
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn k2_sequenceexprtypeswitch_16() {
    //  Use variables with type declarations.
    // XPath: typeswitch(<e/>, <e/>) case $b as element() return concat(\"
    // Expected: 1
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn typeswitch_union_nomatch() {
    // Typeswitch with union, no match
    // XPath: typeswitch (<e/>)    case $i as xs:integer | xs:string retur
    // Expected: 0
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn typeswitch_union_nomatch_2() {
    // Typeswitch with union, no match
    // XPath: typeswitch (<e/>)    case xs:integer | xs:string return 1
    // Expected: 0
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn typeswitch_119() {
    //           Purpose: Test typeswitch with an impossible branch. This is not an err
    // XPath: let $x := map{'x':1, 'y':2}          let $y := map{'A':1, 'B
    // Expected: 1
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn cbcl_typeswitch_001() {
    //  Tests evaluateoptionalitem
    // XPath: upper-case(typeswitch ((1 to 10)[. mod 2 = 0]) case xs:integ
    // Expected: INT
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn cbcl_typeswitch_002() {
    //  Tests evaluateOptionalItem with default clause
    // XPath: upper-case(typeswitch ((1 to 10)[. div 2 = 0]) case xs:integ
    // Expected: FALSE
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn cbcl_typeswitch_003() {
    //  Tests typeswitch clause which always returns the same value
    // XPath: typeswitch ((1 to 10)[. mod 2 = 0]) case xs:integer+ return
    // Expected: true
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn cbcl_typeswitch_004() {
    //  Tests optimization of typeswitch clause to 'instance of'
    // XPath: typeswitch ((1 to 10)[. mod 2 = 0]) case xs:integer+ return
    // Expected: false
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn cbcl_typeswitch_005() {
    //  Tests which calls Evaluate method
    // XPath: subsequence((1 to 10)[. mod 2 = 0] instance of xs:integer+,1
    // Expected: true
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}

#[test]
fn cbcl_typeswitch_006() {
    //  Test which calls EvaluateToOptionalItem method
    // XPath: xs:string((1 to 10)[. mod 2 = 0] instance of xs:integer+)
    // Expected: true
    // Calls stub function - will fail until prod:TypeswitchExpr is implemented
    let _ = stub_prodtypeswitchexpr();
}
