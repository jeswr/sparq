//! Test chunk 0 for fn:node-name
//! Contains 30 tests

use dep::xpath::stub_fnnode_name;

#[test]
fn fn_node_name_1() {
    //  Evaluation of node function with argument set empty sequence. Use fn:count to a
    // XPath: fn:count(fn:node-name(()))
    // Expected: 0
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_2() {
    //  Evaluation of node-name function with argument set to comment node. Use fn:coun
    // XPath: fn:count(fn:node-name(/works[1]/employee[2]/child::text()[la
    // Expected: 0
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_3() {
    //  Evaluation of node-name function with argument set to an element node. Uses loc
    // XPath: fn:local-name-from-QName(fn:node-name(/works[1]/employee[2])
    // Expected: employee
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_4() {
    //  Evaluation of node-name function with argument set to an element node. use loca
    // XPath: fn:local-name-from-QName(fn:node-name(<shoe size = \"5\"/>))
    // Expected: shoe
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_5() {
    //  Evaluation of node-name function with argument set to comment node. Use of fn:c
    // XPath: fn:count(fn:node-name(<!-- This a comment node -->))
    // Expected: 0
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_6() {
    //  Evaluation of node-name function with argument set to a Processing Instruction
    // XPath: fn:local-name-from-QName(fn:node-name(<?format role=\"output
    // Expected: format
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_7() {
    //  Evaluation of node-name function with argument set to a directly constructed el
    // XPath: fn:namespace-uri-from-QName(fn:node-name(<anelement xmlns =
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_8() {
    //  Evaluation of node-name function with argument set to a computed constructed el
    // XPath: fn:local-name-from-QName(fn:node-name(element elementName {}
    // Expected: elementName
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_9() {
    //  Evaluation of node-name function with argument set to a computed constructed el
    // XPath: fn:count(fn:namespace-uri-from-QName(fn:node-name(element el
    // Expected: 1
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_10() {
    //  Evaluation of node-name function with argument set to a computed constructed el
    // XPath: fn:local-name-from-QName(fn:node-name(element elementName {
    // Expected: elementName
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_11() {
    //  Evaluation of node-name function with argument set to a computed constructed at
    // XPath: fn:local-name-from-QName(fn:node-name(attribute attributeNam
    // Expected: attributeName
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_12() {
    //  Evaluation of node-name function with argument set to a computed constructed at
    // XPath: fn:local-name-from-QName(fn:node-name(attribute attributeNam
    // Expected: attributeName
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_13() {
    //  Evaluation of node-name function with argument set to a computed constructed at
    // XPath: fn:count(fn:namespace-uri-from-QName(fn:node-name(attribute
    // Expected: 1
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_14() {
    //  Evaluation of node-name function with argument set to a computed constructed do
    // XPath: fn:count(fn:local-name-from-QName(fn:node-name(document {\"\
    // Expected: 0
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_15() {
    //  Evaluation of node-name function with argument set to a computed constructed do
    // XPath: fn:count(fn:namespace-uri-from-QName(fn:node-name(document {
    // Expected: 0
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_16() {
    //  Evaluation of node-name function with argument set to a computed constructed do
    // XPath: fn:count(fn:local-name-from-QName(fn:node-name(document {\"<
    // Expected: 0
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_17() {
    //  Evaluation of node-name function with argument set to a computed constructed do
    // XPath: fn:count(fn:namespace-uri-from-QName(fn:node-name(document {
    // Expected: 0
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_18() {
    //  Evaluation of node-name function with argument set to a computed constructed te
    // XPath: fn:count(fn:local-name-from-QName(fn:node-name(text {\"\"}))
    // Expected: 0
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_19() {
    //  Evaluation of node-name function with argument set to a computed constructed te
    // XPath: fn:count(fn:namespace-uri-from-QName(fn:node-name(text {\"\"
    // Expected: 0
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_20() {
    //  Evaluation of node-name function with argument set to a computed constructed te
    // XPath: fn:count(fn:local-name-from-QName(fn:node-name(text {\"a tex
    // Expected: 0
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_21() {
    //  Evaluation of node-name function with argument set to a computed constructed te
    // XPath: fn:count(fn:namespace-uri-from-QName(fn:node-name(text {\"a
    // Expected: 0
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_22() {
    //  Evaluation of node-name function with argument set to a computed constructed pr
    // XPath: fn:local-name-from-QName(fn:node-name(processing-instruction
    // Expected: piName
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_23() {
    //  Evaluation of node-name function with argument set to a computed constructed pr
    // XPath: fn:count(fn:namespace-uri-from-QName(fn:node-name(processing
    // Expected: 1
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_25() {
    //  Evaluation of node-name function with no arguments on element node.
    // XPath: <node xmlns=\"http://example.com/ns\"/>/node-name()
    // Expected: QName("http://example.com/ns", "node")
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_26() {
    //  Evaluation of node-name function with no arguments on attribute node.
    // XPath: <node xml:space=\"preserve\" xmlns=\"http://example.com/ns\"
    // Expected: QName("http://www.w3.org/XML/1998/namesp
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_27() {
    //  Evaluation of node-name function with no arguments on processing instruction no
    // XPath: <?test data?>/node-name()
    // Expected: QName("", "test")
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn fn_node_name_28() {
    //  Evaluation of node-name function with no arguments on namespace node.
    // XPath: namespace{\"foo\"}{\"http://example.com/foo\"}/node-name()
    // Expected: QName("", "foo")
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn k_nodenamefunc_3() {
    //  A test whose essence is: `empty(node-name(()))`.
    // XPath: empty(node-name(()))
    // Expected: true
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn k2_nodenamefunc_3() {
    //  Check the return value of fn:node-name().
    // XPath: node-name(/*) instance of xs:QName
    // Expected: true
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}

#[test]
fn cbcl_node_name_001() {
    //  Test fn:node-name on a comment().
    // XPath: empty(node-name( comment { \"comments have no name \" } ) )
    // Expected: true
    // Calls stub function - will fail until fn:node-name is implemented
    let _ = stub_fnnode_name();
}
