//! Test chunk 4 for prod:AxisStep
//! Contains 18 tests

use dep::xpath::stub_prodaxisstep;

#[test]
fn k2_axes_67() {
    //  Apply the descendant axis on an element, as operand to the comma operator.
    // XPath: 1, <e/>/descendant::node(), 1
    // Expected: 1 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn k2_axes_69() {
    //  Apply the descendant axis on an element that has an attribute, as operand to th
    // XPath: 1, <e attr=\"\"/>/descendant::node(), 1
    // Expected: 1 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn k2_axes_71() {
    //  Apply function fn:empty() on the descendant axis applied on an element construc
    // XPath: empty(<e/>/descendant::node())
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn k2_axes_72() {
    //  Apply function fn:empty() on the descendant-or-self axis applied on an element
    // XPath: empty(<e/>/descendant-or-self::node())
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn k2_axes_73() {
    //  Apply function fn:empty() on the descendant axis applied on an element construc
    // XPath: empty(<e attr=\"\"/>/descendant::node())
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn k2_axes_74() {
    //  Apply function fn:empty() on the descendant-or-self axis applied on an element
    // XPath: empty(<e attr=\"\"/>/descendant-or-self::node())
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn k2_axes_78() {
    //  Apply fn:count() to a set of nodes involving the parent axis.
    // XPath: <a> <b c=\"\"/> <d/> </a>//node()/../count(.)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn k2_axes_79() {
    //  Apply fn:count() to a set of nodes involving the descendant axis.
    // XPath: count(<a> <b c=\"\"/> <d/> </a>//node())
    // Expected: 2
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn k2_axes_80() {
    //  Apply fn:count() to a set of nodes involving the descendant axis.
    // XPath: count(<a> <b c=\"\"/> <d/> </a>/descendant-or-self::node())
    // Expected: 3
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn k2_axes_83() {
    //  Simple parser test of the attribute() test appearing after the attribute axis.
    // XPath: empty(<e/>/attribute::attribute())
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn k2_axes_103() {
    //  Combine a function call with paths that requires sorting.
    // XPath: declare function local:isComplexType($typeID) { string($type
    // Expected: |       _17 _17  _11c _11c _11 _11  _17c
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn k2_axes_106() {
    //            Empty descendant axis: Saxon bug 4683
    // XPath: string-join(//item ! ('[' || position() || ',' || not(*) ||
    // Expected: [1,true,][2,false,foo1][3,true,][4,false
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn cbcl_followingaxis_001() {
    //  Tests for following-sibling::*
    // XPath: count(<a><b/><b/></a>/b[1]/following-sibling::*)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn cbcl_precedingaxis_001() {
    //  Tests for preceding-sibling::*
    // XPath: count(<a><b/><b/></a>/b[2]/preceding-sibling::*)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn cbcl_precedingaxis_002() {
    //  Test for preceding-sibling::b
    // XPath: count(<a><b/><b/></a>/b[2]/preceding-sibling::b)
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn cbcl_precedingaxis_003() {
    //  Test for preceding-sibling::b
    // XPath: count(<a><b/><b/></a>/b[1]/preceding-sibling::b)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn cbcl_except_001() {
    //  Tests Equals method on NodeOperator tests
    // XPath: let $x := <a><b/><c/><d/></a> return count(($x/(node() excep
    // Expected: 2
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}

#[test]
fn cbcl_wild_001() {
    //  Tests Equals function on wild localname tests
    // XPath: declare namespace foo=\"test\";         let $x := <a><foo:b/
    // Expected: 1
    // Calls stub function - will fail until prod:AxisStep is implemented
    let _ = stub_prodaxisstep();
}
