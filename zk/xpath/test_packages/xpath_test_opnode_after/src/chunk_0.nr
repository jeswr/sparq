//! Test chunk 0 for op:node-after
//! Contains 10 tests

use dep::xpath::stub_opnode_after;

#[test]
fn nodeexpression43() {
    //  Test of a ComparisonExpression with the operands set as follows operand1 = Sing
    // XPath: /works[1]/employee[1]/empnum[1] >> /works[1]/employee[1]/emp
    // Expected: false
    // Calls stub function - will fail until op:node-after is implemented
    let _ = stub_opnode_after();
}

#[test]
fn nodeexpression47() {
    // Name nodeexpression47  Test of a ComparisonExpression testing the rule that stat
    // XPath: (($works/works[1]/employee[1]/empnum[1] >> $staff/staff[1]/e
    // Expected: true
    // Calls stub function - will fail until op:node-after is implemented
    let _ = stub_opnode_after();
}

#[test]
fn nodeexpression48() {
    //  Test of a ComparisonExpression with the operands set as follows operand1 = Sequ
    // XPath: (/staff[1]/employee[1]/empnum[1]) >> (/staff[1]/employee[1]/
    // Expected: false
    // Calls stub function - will fail until op:node-after is implemented
    let _ = stub_opnode_after();
}

#[test]
fn nodeexpressionhc11() {
    // test nodeexpressionhc11  Test of a Node after expression used as an argument to
    // XPath: fn:not((/staff[1]/employee[1]/empnum[1]) >> (/staff[1]/emplo
    // Expected: true
    // Calls stub function - will fail until op:node-after is implemented
    let _ = stub_opnode_after();
}

#[test]
fn nodeexpressionhc12() {
    // test nodeexpressionhc12  Test of a node after expression used as part of a boole
    // XPath: ((/staff[1]/employee[1]/empnum[1]) >> (/staff[1]/employee[1]
    // Expected: false
    // Calls stub function - will fail until op:node-after is implemented
    let _ = stub_opnode_after();
}

#[test]
fn nodeexpressionhc13() {
    // test nodeexpressionhc13  Test of a node after expression used as part of a boole
    // XPath: ((/staff[1]/employee[1]/empnum[1]) >> (/staff[1]/employee[1]
    // Expected: true
    // Calls stub function - will fail until op:node-after is implemented
    let _ = stub_opnode_after();
}

#[test]
fn nodeexpressionhc14() {
    // test nodeexpression14hc  Test of a node after expression used as part of a boole
    // XPath: ((/staff[1]/employee[1]/empnum[1]) >> (/staff[1]/employee[1]
    // Expected: false
    // Calls stub function - will fail until op:node-after is implemented
    let _ = stub_opnode_after();
}

#[test]
fn nodeexpressionhc15() {
    // test nodeexpressionhc15  Test of a node after expression used as part of a boole
    // XPath: ((/staff[1]/employee[1]/empnum[1]) >> (/staff[1]/employee[1]
    // Expected: true
    // Calls stub function - will fail until op:node-after is implemented
    let _ = stub_opnode_after();
}

#[test]
fn cbcl_node_after_001() {
    //  test node after operator
    // XPath: let $node := <a> <b/> <c/> </a>         return not(exactly-o
    // Expected: true
    // Calls stub function - will fail until op:node-after is implemented
    let _ = stub_opnode_after();
}

#[test]
fn cbcl_node_after_002() {
    //  test node after operator
    // XPath: let $node := <a> <b/> <c/> </a>         return not(not(exact
    // Expected: false
    // Calls stub function - will fail until op:node-after is implemented
    let _ = stub_opnode_after();
}
