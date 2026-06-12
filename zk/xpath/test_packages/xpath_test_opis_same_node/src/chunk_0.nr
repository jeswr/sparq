//! Test chunk 0 for op:is-same-node
//! Contains 22 tests

use dep::xpath::stub_opis_same_node;

#[test]
fn nodeexpression6() {
    //  Test of a ComparisonExpression with the operands set as follows operand1 = Sing
    // XPath: <a>50000</a> is <a>50000</a>
    // Expected: false
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn nodeexpression7() {
    //  Test of a ComparisonExpression with the operands set as follows operand1 = Sing
    // XPath: <a>50000</a> is /works[1]/employee[1]/empnum[1]
    // Expected: false
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn nodeexpression8() {
    //  Test of a ComparisonExpression with the operands set as follows operand1 = Sing
    // XPath: <a>50000</a> is (/staff[1]/employee[1]/empnum[1])
    // Expected: false
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn nodeexpression10() {
    //  Test of a ComparisonExpression with the operands set as follows operand1 = Sing
    // XPath: /works[1]/employee[1]/empnum[1] is <a>50000</a>
    // Expected: false
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn nodeexpression11() {
    //  Test of a ComparisonExpression with the operands set as follows operand1 = Sing
    // XPath: /works[1]/employee[1]/empnum[1] is /works[1]/employee[1]/emp
    // Expected: true
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn nodeexpression12() {
    //  Test of a ComparisonExpression with the operands set as follows operand1 = Sing
    // XPath: $works/works[1]/employee[1]/empnum[1] is $staff/staff[1]/emp
    // Expected: false
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn nodeexpression14() {
    //  Test of a ComparisonExpression with the operands set as follows operand1 = Sequ
    // XPath: (/staff[1]/employee[1]/empnum[1]) is <a>50000</a>
    // Expected: false
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn nodeexpression15() {
    //  Test of a ComparisonExpression with the operands set as follows operand1 = Sequ
    // XPath: ($staff/staff[1]/employee[1]/empnum[1]) is $works/works[1]/e
    // Expected: false
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn nodeexpression16() {
    //  Test of a ComparisonExpression with the operands set as follows operand1 = Sequ
    // XPath: (/staff[1]/employee[1]/empnum[1]) is (/staff[1]/employee[1]/
    // Expected: true
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn nodeexpressionhc1() {
    //  Test of a Node Expression used as an argument to the fn not function
    // XPath: fn:not((/staff[1]/employee[1]/empnum[1]) is (/staff[1]/emplo
    // Expected: false
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn nodeexpressionhc2() {
    //  Test of a Node Expression used as part of a boolean-less-than expression (lt op
    // XPath: ((/staff[1]/employee[1]/empnum[1]) is (/staff[1]/employee[1]
    // Expected: false
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn nodeexpressionhc3() {
    //  Test of a Node Expression used as part of a boolean less than expression (ge) o
    // XPath: ((/staff[1]/employee[1]/empnum[1]) is (/staff[1]/employee[1]
    // Expected: true
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn nodeexpressionhc4() {
    //  Test of a Node Expression used as part of a boolean greater than expression (gt
    // XPath: ((/staff[1]/employee[1]/empnum[1]) is (/staff[1]/employee[1]
    // Expected: false
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn nodeexpressionhc5() {
    //  Test of a Node Expression used as part of a boolean greater than expression (le
    // XPath: ((/staff[1]/employee[1]/empnum[1]) is (/staff[1]/employee[1]
    // Expected: true
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn k2_nodesame_1() {
    //  Ensure node identity is is correct for variables and element constructors.
    // XPath: declare variable $var := <elem/>; <a>{$var}</a>/elem[1] is $
    // Expected: false
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn k2_nodesame_2() {
    //  Ensure node identity is is correct between variable references.
    // XPath: declare variable $var := <elem/>; $var is $var
    // Expected: true
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn k2_nodesame_3() {
    //  Ensure node identity is is correct for variables and element constructors(#2).
    // XPath: declare variable $var := <elem/>; not($var is <elem/>)
    // Expected: true
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn k2_nodesame_4() {
    //  Ensure node identity of nodes used with node constructors.
    // XPath: declare variable $e := attribute name {()}; $e is $e, <is/>
    // Expected: true false
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn k2_nodesame_6() {
    //  Compare two empty sequences, that are tricky to infer at compile time.
    // XPath: empty(zero-or-one(<e/>/*) is zero-or-one(<e/>/*))
    // Expected: true
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn cbcl_node_same_001() {
    //  test is same node operator
    // XPath: let $node := <a> <b/> <c/> </a>         return not(exactly-o
    // Expected: true
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn cbcl_node_same_002() {
    //  test is same node operator
    // XPath: let $node := <a> <b/> <c/> </a>         return not(not(exact
    // Expected: false
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}

#[test]
fn cbcl_is_same_node_001() {
    //  Check node identity for return values of creative user defined functions.
    // XPath: declare function local:f() as node() { <a/> };         local
    // Expected: false
    // Calls stub function - will fail until op:is-same-node is implemented
    let _ = stub_opis_same_node();
}
