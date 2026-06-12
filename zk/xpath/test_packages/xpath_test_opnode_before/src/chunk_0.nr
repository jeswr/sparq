//! Test chunk 0 for op:node-before
//! Contains 10 tests

use dep::xpath::stub_opnode_before;

#[test]
fn nodeexpression27() {
    //  Test of a ComparisonExpression with the operands set as follows           opera
    // XPath: /works[1]/employee[1]/empnum[1] << /works[1]/employee[1]/emp
    // Expected: false
    // Calls stub function - will fail until op:node-before is implemented
    let _ = stub_opnode_before();
}

#[test]
fn nodeexpression31() {
    // Name nodeexpression31  Test of a ComparisonExpression testing the           rule
    // XPath: (($works/works[1]/employee[1]/empnum[1] << $staff/staff[1]/e
    // Expected: true
    // Calls stub function - will fail until op:node-before is implemented
    let _ = stub_opnode_before();
}

#[test]
fn nodeexpression32() {
    //  Test of a ComparisonExpression with the operands set as follows           opera
    // XPath: (/staff[1]/employee[1]/empnum[1]) << (/staff[1]/employee[1]/
    // Expected: false
    // Calls stub function - will fail until op:node-before is implemented
    let _ = stub_opnode_before();
}

#[test]
fn nodeexpressionhc6() {
    //  Test of a Node before Expression used as an argument to a 'fn:not' function.
    // XPath: fn:not((/staff[1]/employee[1]/empnum[1]) << (/staff[1]/emplo
    // Expected: true
    // Calls stub function - will fail until op:node-before is implemented
    let _ = stub_opnode_before();
}

#[test]
fn nodeexpressionhc7() {
    //  Test of a Node before Expression used as part of a boolean-less-than expression
    // XPath: ((/staff[1]/employee[1]/empnum[1]) << (/staff[1]/employee[1]
    // Expected: false
    // Calls stub function - will fail until op:node-before is implemented
    let _ = stub_opnode_before();
}

#[test]
fn nodeexpressionhc8() {
    //  Test of a Node before Expression used as part of a boolean-less-than expression
    // XPath: ((/staff[1]/employee[1]/empnum[1]) << (/staff[1]/employee[1]
    // Expected: true
    // Calls stub function - will fail until op:node-before is implemented
    let _ = stub_opnode_before();
}

#[test]
fn nodeexpressionhc9() {
    //  Test of a Node before Expression used as part of a boolean-greater-than express
    // XPath: ((/staff[1]/employee[1]/empnum[1]) << (/staff[1]/employee[1]
    // Expected: false
    // Calls stub function - will fail until op:node-before is implemented
    let _ = stub_opnode_before();
}

#[test]
fn nodeexpressionhc10() {
    //  Test of a Node before Expression used as part of a boolean-greater-than express
    // XPath: ((/staff[1]/employee[1]/empnum[1]) << (/staff[1]/employee[1]
    // Expected: true
    // Calls stub function - will fail until op:node-before is implemented
    let _ = stub_opnode_before();
}

#[test]
fn cbcl_node_before_001() {
    //  test node before operator
    // XPath: let $node := <a> <b/> <c/> </a>         return not(exactly-o
    // Expected: false
    // Calls stub function - will fail until op:node-before is implemented
    let _ = stub_opnode_before();
}

#[test]
fn cbcl_node_before_002() {
    //  test node before operator
    // XPath: let $node := <a> <b/> <c/> </a>         return not(not(exact
    // Expected: true
    // Calls stub function - will fail until op:node-before is implemented
    let _ = stub_opnode_before();
}
