//! Test chunk 0 for op:except
//! Contains 47 tests

use dep::xpath::stub_opexcept;

#[test]
fn fn_except_node_args_001() {
    //  arg: node
    // XPath: /bib/book[1]/title except /bib/book[1]/title
    // Expected:
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn fn_except_node_args_003() {
    //  arg: node
    // XPath: (/bib/book[3] except root(exactly-one(/bib/book[3]/title)))/
    // Expected: 2000
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn fn_except_node_args_006() {
    //  arg: processing-instruction node & node
    // XPath: (/processing-instruction() except /processing-instruction(PI
    // Expected: PI2
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn fn_except_node_args_008() {
    //  arg: text node & node
    // XPath: string-join((for $node in /bib/book/title/text() except /bib
    // Expected: TCP/IP Illustrated|Advanced Programming
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn fn_except_node_args_009() {
    //  arg: processing-instruction node & node
    // XPath: (/processing-instruction() except /bib/book[2]/title)/name()
    // Expected: PI1 PI2
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn fn_except_node_args_010() {
    //  arg: processing-instruction node & node
    // XPath: (/processing-instruction(PI1) except /bib/book)/name()
    // Expected: PI1
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn fn_except_node_args_017() {
    //  arg: node & node
    // XPath: number( (/atomic:root/atomic:integer) except (/atomic:root/a
    // Expected: 12678967543233
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn fn_except_node_args_021() {
    //  arg: text node & text node
    // XPath: (/atomic:root/atomic:QName/text()) except (/atomic:root/atom
    // Expected:
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn fn_except_node_args_022() {
    //  arg: node & node
    // XPath: string-join((for $node in ((//price/text()) , (//price/text(
    // Expected: 65.95|65.95| 39.95|129.95
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_2() {
    //  Use a node kind keyword, text, to test query parsing.
    // XPath: empty(<e/>/(author except text))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_3() {
    //  Use a node kind keyword, node, to test query parsing.
    // XPath: empty(<e/>/(author except node))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_4() {
    //  Use a node kind keyword, element, to test query parsing.
    // XPath: empty(<e/>/(author except element))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_5() {
    //  Use a node kind keyword, attribute, to test query parsing.
    // XPath: empty(<e/>/(author except attribute))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_6() {
    //  Use a node kind keyword, document-node, to test query parsing.
    // XPath: empty(<e/>/(author except document-node))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_7() {
    //  Use a node kind keyword, comment, to test query parsing.
    // XPath: empty(<e/>/(author except comment))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_8() {
    //  Use a node kind keyword, processing-instruction, to test query parsing.
    // XPath: empty(<e/>/(author except processing-instruction))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_9() {
    //  Use a node kind keyword, item, to test query parsing.
    // XPath: empty(<e/>/(author except item))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_10() {
    //  Use a node kind keyword, document, to test query parsing.
    // XPath: empty(<e/>/(author except document))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_11() {
    //  Use a node kind keyword, if, to test query parsing.
    // XPath: empty(<e/>/(author except if))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_12() {
    //  Use a node kind keyword, then, to test query parsing.
    // XPath: empty(<e/>/(author except then))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_13() {
    //  Use a node kind keyword, mod, to test query parsing.
    // XPath: empty(<e/>/(author except mod))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_14() {
    //  Use a node kind keyword, div, to test query parsing.
    // XPath: empty(<e/>/(author except div))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_15() {
    //  Use a node kind keyword, empty-sequence, to test query parsing.
    // XPath: empty(<e/>/(author except empty-sequence))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_16() {
    //  Use a node kind keyword, schema-attribute, to test query parsing.
    // XPath: empty(<e/>/(author except schema-attribute))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_17() {
    //  Use a node kind keyword, schema-element, to test query parsing.
    // XPath: empty(<e/>/(author except schema-element))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_18() {
    //  Use a computed attribute constructor as right operand.
    // XPath: count(<e/>/(a except attribute {\"name\"} {()}))
    // Expected: 0
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_19() {
    //  Use a computed attribute constructor as right operand(#2).
    // XPath: count(<e/>/(a except attribute name {()}))
    // Expected: 0
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_22() {
    //  Use a computed attribute constructor as right operand.
    // XPath: count(<e/>/(a except processing-instruction {\"name\"} {()})
    // Expected: 0
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_23() {
    //  Use a computed attribute constructor as right operand.
    // XPath: count(<e/>/(a except processing-instruction name {}))
    // Expected: 0
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_24() {
    //  Use a computed attribute constructor as right operand.
    // XPath: count(<e/>/(a except comment {()}))
    // Expected: 0
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_25() {
    //  Use a computed attribute constructor as right operand.
    // XPath: count(<e/>/(a except text {()}))
    // Expected: 0
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_26() {
    //  Use a name test that is the descendant axis as right operand.
    // XPath: empty(<e/>/(a except descendant))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_27() {
    //  Use a name test that is the attribute axis as right operand.
    // XPath: empty(<e/>/(a except attribute))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_28() {
    //  Use a name test that is the self axis as right operand.
    // XPath: empty(<e/>/(a except self))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_29() {
    //  Use a name test that is the descendant-or-self axis as right operand.
    // XPath: empty(<e/>/(a except descendant-or-self))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_30() {
    //  Use a name test that is the following-sibling axis as right operand.
    // XPath: empty(<e/>/(a except following-sibling))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_31() {
    //  Use a name test that is the following axis as right operand.
    // XPath: empty(<e/>/(a except following))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_32() {
    //  Use a name test that is the preceding-sibling axis as right operand.
    // XPath: empty(<e/>/(a except preceding-sibling))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_33() {
    //  Use a name test that is the preceding axis as right operand.
    // XPath: empty(<e/>/(a except preceding))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_34() {
    //  Use a name test that is the parent axis as right operand.
    // XPath: empty(<e/>/(a except parent))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_35() {
    //  Use a name test that is the ancestor axis as right operand.
    // XPath: empty(<e/>/(a except ancestor))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_36() {
    //  Use a name test that is the ancestor axis as right operand.
    // XPath: empty(<e/>/(a except ancestor))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_37() {
    //  Use a name test that is the ancestor-or-self axis as right operand.
    // XPath: empty(<e/>/(a except ancestor-or-self))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_38() {
    //  Use 'declare as right operand.
    // XPath: empty(<e/>/(a except declare))
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn k2_seqexcept_39() {
    //  Extract the boolean value from an except expression.
    // XPath: boolean(//employee[location = \"Denver\"] except //employee[
    // Expected: true
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn combiningnodeseqexcepthc4() {
    //  Simple combination of node sequences that evaluates to the empty sequence. Use
    // XPath: for $h in ( count((//hours) except (//hours))) order by numb
    // Expected: 0
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}

#[test]
fn combiningnodeseqexcepthc5() {
    //  Simple combination of node sequences involving multiple xml data sources and re
    // XPath: for $h in ( count(($works//hours) except ($staff//grade,$wor
    // Expected: 0
    // Calls stub function - will fail until op:except is implemented
    let _ = stub_opexcept();
}
