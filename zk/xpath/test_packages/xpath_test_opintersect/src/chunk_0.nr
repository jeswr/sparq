//! Test chunk 0 for op:intersect
//! Contains 50 tests

use dep::xpath::stub_opintersect;

#[test]
fn fn_intersect_node_args_001() {
    // arg: node
    // XPath: (/bib/book[1]/title intersect /bib/book[1]/title)/string()
    // Expected: TCP/IP Illustrated
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn fn_intersect_node_args_002() {
    // arg: incorrect nodes
    // XPath: /bib/book/title intersect /bib/book[1]/title
    // Expected: TCP/IP Illustrated
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn fn_intersect_node_args_004() {
    // arg: text node & node
    // XPath: /bib/book[3]/title/text() intersect /bib/book/title/text()
    // Expected: Data on the Web
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn fn_intersect_node_args_005() {
    // arg: processing-instruction node & node
    // XPath: (/processing-instruction() intersect /processing-instruction
    // Expected: PI1 PI2
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn fn_intersect_node_args_006() {
    // arg: processing-instruction node & node
    // XPath: (/processing-instruction(PI1) intersect /processing-instruct
    // Expected: PI1
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn fn_intersect_node_args_007() {
    // arg: comment node & node
    // XPath: string-join((/comment() intersect /comment()), \"|\")
    // Expected: this file is a copy of bib.xml; just add
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn fn_intersect_node_args_008() {
    // arg: text node & node
    // XPath: /bib/book[3]/title/text() intersect /bib/book/title/text()
    // Expected: Data on the Web
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn fn_intersect_node_args_011() {
    // arg: comment node & node
    // XPath: (/bib/book intersect /bib/book)/string(@year)
    // Expected: 1994 1992 2000 1999
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn fn_intersect_node_args_018() {
    // arg: text node & text node
    // XPath: (/atomic:root/atomic:duration/text()) intersect (/atomic:roo
    // Expected: P1Y2M3DT10H30M
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn fn_intersect_node_args_019() {
    // arg: text node & text node
    // XPath: (/atomic:root/atomic:date/text()) intersect (/atomic:root/at
    // Expected: 2000-01-01+05:00
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn fn_intersect_node_args_020() {
    // arg: text node & text node
    // XPath: (/atomic:root/atomic:base64Binary/text()) intersect (/atomic
    // Expected: R0lGODlhcgGSALMAAAQCAEMmCZtuMFQxDS8b
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn fn_intersect_node_args_021() {
    // arg: text node & text node
    // XPath: (/atomic:root/atomic:QName/text()) intersect (/atomic:root/a
    // Expected: foo:aQname
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn fn_intersect_node_args_022() {
    // arg: text node & text node
    // XPath: (/atomic:root/atomic:idrefs/text()) intersect (/atomic:root/
    // Expected: id1 id2
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_2() {
    // Use a node kind keyword, text, to test query parsing.
    // XPath: empty(<e/>/(author intersect text))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_3() {
    // Use a node kind keyword, node, to test query parsing.
    // XPath: empty(<e/>/(author intersect node))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_4() {
    // Use a node kind keyword, element, to test query parsing.
    // XPath: empty(<e/>/(author intersect element))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_5() {
    // Use a node kind keyword, attribute, to test query parsing.
    // XPath: empty(<e/>/(author intersect attribute))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_6() {
    // Use a node kind keyword, document-node, to test query parsing.
    // XPath: empty(<e/>/(author intersect document-node))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_7() {
    // Use a node kind keyword, comment, to test query parsing.
    // XPath: empty(<e/>/(author intersect comment))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_8() {
    // Use a node kind keyword, processing-instruction, to test query parsing.
    // XPath: empty(<e/>/(author intersect processing-instruction))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_9() {
    // Use a node kind keyword, item, to test query parsing.
    // XPath: empty(<e/>/(author intersect item))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_10() {
    // Use a node kind keyword, document, to test query parsing.
    // XPath: empty(<e/>/(author intersect document))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_11() {
    // Use a node kind keyword, if, to test query parsing.
    // XPath: empty(<e/>/(author intersect if))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_12() {
    // Use a node kind keyword, then, to test query parsing.
    // XPath: empty(<e/>/(author intersect then))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_13() {
    // Use a node kind keyword, mod, to test query parsing.
    // XPath: empty(<e/>/(author intersect mod))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_14() {
    // Use a node kind keyword, div, to test query parsing.
    // XPath: empty(<e/>/(author intersect div))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_15() {
    // Use a node kind keyword, empty-sequence, to test query parsing.
    // XPath: empty(<e/>/(author intersect empty-sequence))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_16() {
    // Use a node kind keyword, schema-attribute, to test query parsing.
    // XPath: empty(<e/>/(author intersect schema-attribute))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_17() {
    // Use a node kind keyword, schema-element, to test query parsing.
    // XPath: empty(<e/>/(author intersect schema-element))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_18() {
    // Use a computed attribute constructor as right operand.
    // XPath: empty(<e/>/(a intersect attribute {\"name\"} {()}))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_19() {
    // Use a computed attribute constructor as right operand(#2).
    // XPath: empty(<e/>/(a intersect attribute name {()}))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_22() {
    // Use a computed attribute constructor as right operand.
    // XPath: empty(<e/>/(a intersect processing-instruction {\"name\"} {(
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_23() {
    // Use a computed attribute constructor as right operand.
    // XPath: empty(<e/>/(a intersect processing-instruction name {}))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_24() {
    // Use a computed attribute constructor as right operand.
    // XPath: empty(<e/>/(a intersect comment {()}))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_25() {
    // Use a computed attribute constructor as right operand.
    // XPath: empty(<e/>/(a intersect text {()}))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_26() {
    // Use a name test that is the descendant axis as right operand.
    // XPath: empty(<e/>/(a intersect descendant))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_27() {
    // Use a name test that is the attribute axis as right operand.
    // XPath: empty(<e/>/(a intersect attribute))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_28() {
    // Use a name test that is the self axis as right operand.
    // XPath: empty(<e/>/(a intersect self))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_29() {
    // Use a name test that is the descendant-or-self axis as right operand.
    // XPath: empty(<e/>/(a intersect descendant-or-self))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_30() {
    // Use a name test that is the following-sibling axis as right operand.
    // XPath: empty(<e/>/(a intersect following-sibling))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_31() {
    // Use a name test that is the following axis as right operand.
    // XPath: empty(<e/>/(a intersect following))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_32() {
    // Use a name test that is the preceding-sibling axis as right operand.
    // XPath: empty(<e/>/(a intersect preceding-sibling))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_33() {
    // Use a name test that is the preceding axis as right operand.
    // XPath: empty(<e/>/(a intersect preceding))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_34() {
    // Use a name test that is the parent axis as right operand.
    // XPath: empty(<e/>/(a intersect parent))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_35() {
    // Use a name test that is the ancestor axis as right operand.
    // XPath: empty(<e/>/(a intersect ancestor))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_36() {
    // Use a name test that is the ancestor axis as right operand.
    // XPath: empty(<e/>/(a intersect ancestor))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_37() {
    // Use a name test that is the ancestor-or-self axis as right operand.
    // XPath: empty(<e/>/(a intersect ancestor-or-self))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_38() {
    // Use 'declare as right operand.
    // XPath: empty(<e/>/(a intersect declare))
    // Expected: true
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_39() {
    // Extract the boolean value from a intersect expression.
    // XPath: boolean(//employee[location = \"Denver\"] intersect //employ
    // Expected: false
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}

#[test]
fn k2_seqintersect_42() {
    // Perform intersection between two singleton document nodes.
    // XPath: declare function local:function ($c as node()) { $c intersec
    // Expected: false
    // Calls stub function - will fail until op:intersect is implemented
    let _ = stub_opintersect();
}
