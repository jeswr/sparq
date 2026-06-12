//! Test chunk 0 for fn:in-scope-prefixes
//! Contains 37 tests

use dep::xpath::stub_fnin_scope_prefixes;

#[test]
fn fn_in_scope_prefixes_3() {
    //  Evaluation of fn:in-scope-prefixes function for a directly constructed element
    // XPath: fn:in-scope-prefixes(<anElement>some content</anElement>)
    // Expected: xml
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_5() {
    //  Evaluation of fn:in-scope-prefixes function for a computed constructed element
    // XPath: fn:in-scope-prefixes(element anElement {\"Some content\"})
    // Expected: xml
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_6() {
    //  Evaluation of fn:in-scope-prefixes function for a computed constructed element
    // XPath: declare default element namespace \"http://www.example.com\"
    // Expected: 2 true
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_7() {
    //  Evaluation of fn:in-scope-prefixes function for a directly constructed element
    // XPath: declare default element namespace \"http://www.example.com\"
    // Expected: 2 true
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_9() {
    //  Evaluation of fn:in-scope-prefixes function for a directly constructed element
    // XPath: declare namespace p1 = \"http://www.example.com\"; fn:in-sco
    // Expected: xml
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_10() {
    //  Evaluation of fn:in-scope-prefixes function for a directly constructed element
    // XPath: fn:string-length(fn:in-scope-prefixes(<anElement>Some conten
    // Expected: 3
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_11() {
    //  Evaluation of fn:in-scope-prefixes function for a directly constructed element
    // XPath: fn:upper-case(fn:in-scope-prefixes(<anElement>Some content</
    // Expected: XML
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_12() {
    //  Evaluation of fn:in-scope-prefixes function for a directly constructed element
    // XPath: fn:lower-case(fn:in-scope-prefixes(<anElement>Some content</
    // Expected: xml
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_13() {
    //  Evaluation of fn:in-scope-prefixes function for a directly constructed element
    // XPath: fn:string-to-codepoints(fn:in-scope-prefixes(<anElement>Some
    // Expected: 120 109 108
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_14() {
    //  Evaluation of fn:in-scope-prefixes function for a directly constructed element
    // XPath: xs:string(fn:in-scope-prefixes(<anElement>Some content</anEl
    // Expected: xml
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_15() {
    //  Evaluation of fn:in-scope-prefixes function for a directly constructed element
    // XPath: fn:substring-before(fn:in-scope-prefixes(<anElement>Some con
    // Expected: x
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_16() {
    //  Evaluation of fn:in-scope-prefixes function for a directly constructed element
    // XPath: fn:substring-after(fn:in-scope-prefixes(<anElement>Some cont
    // Expected: l
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_17() {
    //  Evaluation of fn:in-scope-prefixes function for a directly constructed element
    // XPath: fn:concat(fn:in-scope-prefixes(<anElement>Some content</anEl
    // Expected: xmlm
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_18() {
    //  Evaluation of fn:in-scope-prefixes function for a directly constructed element
    // XPath: fn:string-join((fn:in-scope-prefixes(<anElement>Some content
    // Expected: xmlxml
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_19() {
    //  Evaluation of fn:in-scope-prefixes function for a directly constructed element
    // XPath: fn:substring(fn:in-scope-prefixes(<anElement>Some content</a
    // Expected: ml
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_20() {
    //  Evaluation of fn:in-scope-prefixes function for a directly constructed element
    // XPath: fn:contains(fn:in-scope-prefixes(<anElement>Some content</an
    // Expected: true
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn fn_in_scope_prefixes_26() {
    //  Evaluation of fn:in-scope-prefixes function for a document that undeclares the
    // XPath: in-scope-prefixes(/*/p)
    // Expected: xml
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_1() {
    //  For a directly constructed element fn:in-scope-prefixes() returns 'xml' and the
    // XPath: count(in-scope-prefixes(<e/>))
    // Expected: 1
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_2() {
    //  For a computed element fn:in-scope-prefixes() returns 'xml'.
    // XPath: count(in-scope-prefixes(element name {7}))
    // Expected: 1
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_6() {
    //  Ensure the default element namespace is in-scope properly.
    // XPath: count(in-scope-prefixes(<a xmlns=\"http://www.example.com\"
    // Expected: 4
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_7() {
    //  Ensure the default namespace is properly handled.
    // XPath: count(in-scope-prefixes(<a xmlns=\"\"/>))
    // Expected: 1
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_8() {
    //  Ensure the in-scope prefixes dealt with correctly for double default namespace
    // XPath: declare default element namespace \"http://www.example.com/A
    // Expected: 2
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_9() {
    //  Check that a prefix declaration is in scope in a sub-node.
    // XPath: for $i in fn:in-scope-prefixes(<e xmlns:p=\"http://example.c
    // Expected: a p xml
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_10() {
    //  Check that a default namespace declaration is picked up.
    // XPath: declare default element namespace \"http://www.example.com\"
    // Expected: 2
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_13() {
    //  Ensure the in-scope prefixes are correct with computed constructors when combin
    // XPath: declare default element namespace \"http://www.example.com/\
    // Expected: 2 2
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_14() {
    //  Check the in-scope namespaces of a single, computed element.
    // XPath: count(in-scope-prefixes(element e{()}))
    // Expected: 1
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_15() {
    //  Ensure the in-scope prefixes are correct with computed constructors.
    // XPath: let $i := element e { element b {()} } return (count(in-scop
    // Expected: 1 1
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_16() {
    //  Check that a default namespace declaration attribute on a direct element constr
    // XPath: let $i := <e xmlns=\"http://example.com/\">
    // Expected: 2 2
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_17() {
    //  Ensure the namespace used in the name is part of the in-scope prefixes.
    // XPath: declare namespace p = \"http://example.com/\"; count(in-scop
    // Expected: 2 2
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_19() {
    //  Count the in-scope namespaces of a node with name xml:space.
    // XPath: count(in-scope-prefixes(element xml:space {()}))
    // Expected: 1
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_20() {
    //  Count the in-scope namespaces of a node with name fn:space.
    // XPath: count(in-scope-prefixes(element fn:space {()}))
    // Expected: 2
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_21() {
    //  Count the in-scope namespaces of a node with name fn:space.
    // XPath: count(in-scope-prefixes(element xs:space {()}))
    // Expected: 2
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_22() {
    //  Count the in-scope namespaces of a node with name fn:space(#2).
    // XPath: count(in-scope-prefixes(<fn:space/>))
    // Expected: 2
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_23() {
    //  Count the in-scope namespaces of a node with name fn:space(#2).
    // XPath: count(in-scope-prefixes(<xs:space/>))
    // Expected: 2
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_24() {
    //  Ensure a prolog namespace declaration isn't in the in-scope bindings unless it'
    // XPath: declare namespace p = \"http://example.com/\"; count(in-scop
    // Expected: 1
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_25() {
    //  Check the in-scope namespaces of two nodes.
    // XPath: declare namespace a = \"http://example.com/1\";          dec
    // Expected: a b xml | a xml
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}

#[test]
fn k2_inscopeprefixesfunc_27() {
    //  Check in scope namespaces.
    // XPath: declare namespace a = \"http://example.com/1\"; declare name
    // Expected: a b xml
    // Calls stub function - will fail until fn:in-scope-prefixes is implemented
    let _ = stub_fnin_scope_prefixes();
}
