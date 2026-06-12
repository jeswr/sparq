//! Test chunk 0 for op:union
//! Contains 45 tests

use dep::xpath::stub_opunion;

#[test]
fn fn_union_node_args_002() {
    //  arg: incorrect nodes
    // XPath: (/bib/book/title | /bib/book)/local-name()
    // Expected: book title book title book title book ti
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn fn_union_node_args_018() {
    //  arg: text node & text node
    // XPath: (/atomic:root/atomic:duration/text()) | (/atomic:root/atomic
    // Expected: P1Y2M3DT10H30M
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn fn_union_node_args_019() {
    //  arg: text node & text node
    // XPath: (/atomic:root/atomic:date/text()) | (/atomic:root/atomic:dat
    // Expected: 2000-01-01+05:00
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn fn_union_node_args_020() {
    //  arg: text node & text node
    // XPath: (/atomic:root/atomic:base64Binary/text()) | (/atomic:root/at
    // Expected: R0lGODlhcgGSALMAAAQCAEMmCZtuMFQxDS8b
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn fn_union_node_args_021() {
    //  arg: text node & text node
    // XPath: (/atomic:root/atomic:QName/text()) | (/atomic:root/atomic:QN
    // Expected: foo:aQname
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn fn_union_node_args_022() {
    //  arg: text node & text node
    // XPath: (/atomic:root/atomic:idrefs/text()) | (/atomic:root/atomic:i
    // Expected: id1 id2
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn fn_union_node_args_023() {
    //  arg: node & node
    // XPath: string-join(for $node in ((//price/text()) , (//price/text()
    // Expected: 65.95|65.95| 39.95|129.95
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_3() {
    //  Use 'comment' as right operand.
    // XPath: empty(<e/>/(author union comment))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_6() {
    //  Use a node kind keyword, text, to test query parsing.
    // XPath: empty(<e/>/(author union text))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_7() {
    //  Use a node kind keyword, node, to test query parsing.
    // XPath: empty(<e/>/(author union node))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_8() {
    //  Use a node kind keyword, element, to test query parsing.
    // XPath: empty(<e/>/(author union element))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_9() {
    //  Use a node kind keyword, attribute, to test query parsing.
    // XPath: empty(<e/>/(author union attribute))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_10() {
    //  Use a node kind keyword, document-node, to test query parsing.
    // XPath: empty(<e/>/(author union document-node))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_11() {
    //  Use a node kind keyword, comment, to test query parsing.
    // XPath: empty(<e/>/(author union comment))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_12() {
    //  Use a node kind keyword, processing-instruction, to test query parsing.
    // XPath: empty(<e/>/(author union processing-instruction))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_13() {
    //  Use a node kind keyword, item, to test query parsing.
    // XPath: empty(<e/>/(author union item))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_14() {
    //  Use a node kind keyword, document, to test query parsing.
    // XPath: empty(<e/>/(author union document))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_15() {
    //  Use a node kind keyword, if, to test query parsing.
    // XPath: empty(<e/>/(author union if))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_16() {
    //  Use a node kind keyword, then, to test query parsing.
    // XPath: empty(<e/>/(author union then))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_17() {
    //  Use a node kind keyword, mod, to test query parsing.
    // XPath: empty(<e/>/(author union mod))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_18() {
    //  Use a node kind keyword, div, to test query parsing.
    // XPath: empty(<e/>/(author union div))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_19() {
    //  Use a node kind keyword, empty-sequence, to test query parsing.
    // XPath: empty(<e/>/(author union empty-sequence))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_20() {
    //  Use a node kind keyword, schema-attribute, to test query parsing.
    // XPath: empty(<e/>/(author union schema-attribute))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_21() {
    //  Use a node kind keyword, schema-element, to test query parsing.
    // XPath: empty(<e/>/(author union schema-element))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_22() {
    //  Use a computed attribute constructor as right operand.
    // XPath: empty(<e/>/(a union attribute {\"name\"} {()}))
    // Expected: false
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_23() {
    //  Use a computed attribute constructor as right operand(#2).
    // XPath: empty(<e/>/(a union attribute name {()}))
    // Expected: false
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_26() {
    //  Use a processing instruction constructor as right operand.
    // XPath: empty(<e/>/(a union processing-instruction {\"name\"} {()}))
    // Expected: false
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_27() {
    //  Use a processing instruction constructor as right operand.
    // XPath: empty(<e/>/(a union processing-instruction name {}))
    // Expected: false
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_28() {
    //  Use a comment constructor as right operand.
    // XPath: empty(<e/>/(a union comment {()}))
    // Expected: false
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_29() {
    //  Use a text node constructor as right operand.
    // XPath: empty(<e/>/(a union text {()}))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_30() {
    //  Use a name test that resembles the descendant axis as right operand.
    // XPath: empty(<e/>/(a union descendant))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_31() {
    //  Use a name test that resembles the attribute axis as right operand.
    // XPath: empty(<e/>/(a union attribute))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_32() {
    //  Use a name test that resembles the self axis as right operand.
    // XPath: empty(<e/>/(a union self))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_33() {
    //  Use a name test that resembles the descendant-or-self axis as right operand.
    // XPath: empty(<e/>/(a union descendant-or-self))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_34() {
    //  Use a name test that resembles the following-sibling axis as right operand.
    // XPath: empty(<e/>/(a union following-sibling))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_35() {
    //  Use a name test that resembles the following axis as right operand.
    // XPath: empty(<e/>/(a union following))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_36() {
    //  Use a name test that resembles the preceding-sibling axis as right operand.
    // XPath: empty(<e/>/(a union preceding-sibling))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_37() {
    //  Use a name test that resembles the preceding axis as right operand.
    // XPath: empty(<e/>/(a union preceding))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_38() {
    //  Use a name test that resembles the parent axis as right operand.
    // XPath: empty(<e/>/(a union parent))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_39() {
    //  Use a name test that resembles the ancestor axis as right operand.
    // XPath: empty(<e/>/(a union ancestor))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_40() {
    //  Use a name test that resembles the ancestor axis as right operand.
    // XPath: empty(<e/>/(a union ancestor))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_41() {
    //  Use a name test that resembles the ancestor-or-self axis as right operand.
    // XPath: empty(<e/>/(a union ancestor-or-self))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_42() {
    //  Use 'declare as right operand.
    // XPath: empty(<e/>/(a union declare))
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_43() {
    //  Extract the boolean value from a union expression.
    // XPath: boolean(//employee[location = \"Denver\"] union //employee[l
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}

#[test]
fn k2_sequnion_44() {
    //  Extract the boolean value from a union expression(different keyword).
    // XPath: boolean(//employee[location = \"Denver\"] | //employee[last(
    // Expected: true
    // Calls stub function - will fail until op:union is implemented
    let _ = stub_opunion();
}
