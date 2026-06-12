//! Test chunk 0 for prod:DirElemContent
//! Contains 50 tests

use dep::xpath::stub_proddirelemcontent;

#[test]
fn constr_cont_entref_1() {
    //  Pre-defined entity reference in element content
    // XPath: string-to-codepoints(<elem>&lt;</elem>)
    // Expected: 60
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_entref_2() {
    //  Pre-defined entity reference in element content
    // XPath: string-to-codepoints(<elem>&gt;</elem>)
    // Expected: 62
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_entref_3() {
    //  Pre-defined entity reference in element content
    // XPath: string-to-codepoints(<elem>&amp;</elem>)
    // Expected: 38
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_entref_4() {
    //  Pre-defined entity reference in element content
    // XPath: string-to-codepoints(<elem>&quot;</elem>)
    // Expected: 34
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_entref_5() {
    //  Pre-defined entity reference in element content
    // XPath: string-to-codepoints(<elem>&apos;</elem>)
    // Expected: 39
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_text_1() {
    //  count text nodes for direct element content
    // XPath: count((<elem>text</elem>)/text())
    // Expected: 1
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_text_2() {
    //  count text nodes for direct element content and CDATA section
    // XPath: count((<elem>text<![CDATA[cdata]]></elem>)/text())
    // Expected: 1
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_enclexpr_1() {
    //  1 text node for enclosed expression with atomic values
    // XPath: count((<elem>{1,'a',3.5,4e2}</elem>)/text())
    // Expected: 1
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_enclexpr_2() {
    //  1 text node for enclosed expression with atomic values
    // XPath: count((<elem>{1,'a',<a/>,3.5,4e2}</elem>)/text())
    // Expected: 2
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_nodeid_1() {
    //  Copied element node has new node identity
    // XPath: for $x in <a/>, $y in <elem>{$x}</elem> return exactly-one($
    // Expected: false
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_nodeid_2() {
    //  Copied attribute node has new node identity
    // XPath: for $x in <a b=\"b\"/>, $y in <elem>{$x/@b}</elem> return $y
    // Expected: false
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_nodeid_3() {
    //  Copied comment node has new node identity
    // XPath: for $x in <!--comment-->, $y in <elem>{$x}</elem> return exa
    // Expected: false
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_nodeid_4() {
    //  Copied pi node has new node identity
    // XPath: for $x in <?pi content?>, $y in <elem>{$x}</elem> return exa
    // Expected: false
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_nodeid_5() {
    //  Copied text node has new node identity
    // XPath: for $x in <a>text</a>, $y in <elem>{$x/text()}</elem> return
    // Expected: false
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_constrmod_1() {
    //  strip IDREFS type
    // XPath: declare construction strip; fn:count(fn:data(<elem>{//*:idre
    // Expected: 1
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_constrmod_5() {
    //  strip IDREFS type in attribute
    // XPath: declare construction strip; fn:count(fn:data((<elem>{//*:idr
    // Expected: 1
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_constrmod_9() {
    //  Truly preserve decimal type in attribute (Constr-cont-constrmod-8 doesn't do th
    // XPath: declare construction preserve; <elem>{//*:decimal/@*:attr}</
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_constrmod_10() {
    //  Truly preserve decimal type in attribute (Constr-cont-constrmod-8 doesn't do th
    // XPath: declare construction preserve; <elem>{//*:decimal}</elem>/*/
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_uripres_1() {
    //  Preserve base uri for copied element nodes
    // XPath: for $x in <a xml:base=\"http://www.example.com/base1\"><b/><
    // Expected: http://www.example.com/base2
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_adjtext_1() {
    //  single text node in element content
    // XPath: count((<elem>a{1,2,3}b</elem>)/text())
    // Expected: 1
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_adjtext_2() {
    //  two text nodes in element content
    // XPath: count((<elem>a{1,<a/>,3}b</elem>)/text())
    // Expected: 2
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_adjtext_3() {
    //  strip empty text node
    // XPath: count((<elem>{''}</elem>)/text())
    // Expected: 0
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_adjtext_4() {
    //  no text node for empty sequence constructed
    // XPath: count((<elem>{()}</elem>)/text())
    // Expected: 0
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_baseuri_1() {
    //  base-uri through xml:base attribute
    // XPath: fn:base-uri(<elem xml:base=\"http://www.example.com\"/>)
    // Expected: http://www.example.com
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_baseuri_2() {
    //  base-uri through parent
    // XPath: fn:base-uri(exactly-one((<elem xml:base=\"http://www.example
    // Expected: http://www.example.com
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_baseuri_3() {
    //  base-uri through declaration
    // XPath: declare base-uri \"http://www.example.com\"; fn:base-uri(<el
    // Expected: http://www.example.com
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_parent_1() {
    //  empty parent
    // XPath: count((<elem/>)/..)
    // Expected: 0
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_string_1() {
    //  string value of element
    // XPath: fn:string(<elem>a<a/>b</elem>)
    // Expected: ab
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_data_1() {
    //  typed value of element
    // XPath: fn:data(<elem>a<a/>b</elem>)
    // Expected: ab
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_document_1() {
    //  Check the merging of text nodes introduced by the contents of a document node T
    // XPath: count(<wrapper> {'abc', document {'def', <anode/>, 'ghi'}, '
    // Expected: 3
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_document_2() {
    //  Check the merging of text nodes introduced by the contents of a document node T
    // XPath: count(<wrapper> abc {document {'def', <anode/>, 'ghi'}} jkl
    // Expected: 3
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_document_4() {
    //  Check the merging of text nodes introduced by the contents of a document node.
    // XPath: count( document {'abc', 'def', document {'ghi', <anode/>, 'j
    // Expected: 3
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn constr_cont_document_5() {
    //  Check the merging of text nodes introduced by the contents of a document node.
    // XPath: count( document {'abc', 'def', document {'ghi', 'jkl'}, 'mno
    // Expected: 1
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_5() {
    //  Test that the typed value of element nodes is correct.
    // XPath: data(<name>some text</name>) instance of xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_6() {
    //  Test that the typed value of element nodes is correct.
    // XPath: not(data(<name>some text</name>) instance of xs:string)
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_7() {
    //  Test that the typed value of element nodes is correct.
    // XPath: <name>some, if(1) then else</name> instance of element()
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_8() {
    //  Test that the typed of element nodes is correct.
    // XPath: not(<name>some text</name> instance of xs:untypedAtomic)
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_9() {
    //  Test that the typed value of comment nodes is correct.
    // XPath: data(<!-- a comment -->) instance of xs:string
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_10() {
    //  Test that the typed value of comment nodes is correct.
    // XPath: not(data(<!-- a comment -->) instance of xs:untypedAtomic)
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_11() {
    //  Test that the typed value of comment nodes is correct.
    // XPath: <!-- a comment --> instance of comment()
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_12() {
    //  Test that the typed value of comment nodes is correct.
    // XPath: not(<!-- a comment --> instance of xs:untypedAtomic)
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_13() {
    //  Test that the typed value of comment nodes is correct.
    // XPath: not(<!-- a comment --> instance of xs:string)
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_14() {
    //  Test that the typed value of PI nodes is correct.
    // XPath: data(<?target content?>) instance of xs:string
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_15() {
    //  Test that the typed value of PI nodes is correct.
    // XPath: not(data(<?target content?>) instance of xs:untypedAtomic)
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_16() {
    //  Test that the typed value of PI nodes is correct.
    // XPath: <?target content?> instance of processing-instruction()
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_17() {
    //  Test that the typed value of PI nodes is correct.
    // XPath: not(<?target content?> instance of xs:untypedAtomic)
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_25() {
    //  XQuery comments inside elements are not recognized.
    // XPath: string(<eg> (: an (:example:) </eg>)
    // Expected: (: an (:example:)
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_27() {
    //  Many CDATA sections.
    // XPath: string(<elem><![CDATA[str]]>str<![CDATA[str]]><![CDATA[str]]
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_28() {
    //  Ensure CDATA doesn't receive special attention.
    // XPath: string(<elem><![CDATA[con<<< ]] >\"\"'*\"*\">>tent]]&#00;&#x
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_29() {
    //  Test that the type annotation of text nodes is correct.
    // XPath: data(text{\"content\"}) instance of xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}
