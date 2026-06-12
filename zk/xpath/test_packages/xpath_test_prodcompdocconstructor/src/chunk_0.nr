//! Test chunk 0 for prod:CompDocConstructor
//! Contains 29 tests

use dep::xpath::stub_prodcompdocconstructor;

#[test]
fn constr_docnode_adjtext_1() {
    //  merge adjacent atomic values to text node
    // XPath: count((document {1, 'string', 1,2e3})/text())
    // Expected: 1
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_adjtext_2() {
    //  merge adjacent atomic values and text nodes to text node
    // XPath: count((document {1, //text(), 'string'})/text())
    // Expected: 1
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_adjtext_3() {
    //  merge adjacent atomic values between other nodes to text node
    // XPath: count((document {1, 2, <a/>, 3, 4, <b/>, 5, 6})/text())
    // Expected: 3
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_nested_1() {
    //  nested computed element constructors
    // XPath: document {1, document {2, document {document {()}, document
    // Expected: 1234
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_nested_3() {
    //  Check that text nodes are merged in nested document constructors.
    // XPath: count(document {1, document{2}, document { document {()}, do
    // Expected: 1
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_nested_4() {
    //  Check that empty text nodes are stripped in nested document constructors.
    // XPath: count(document {\"\", document{\"\"}, document { document {(
    // Expected: 0
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_enclexpr_1() {
    //  empty computed element content
    // XPath: document {()}
    // Expected:
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_enclexpr_2() {
    //  atomic values in computed element content
    // XPath: document {1,'a',3.5,4e2}
    // Expected: 1 a 3.5 400
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_enclexpr_4() {
    //  empty string in element content
    // XPath: document {1, '', 2}
    // Expected: 1  2
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_nodeid_1() {
    //  Copied element node has new node identity
    // XPath: for $x in <a/>, $y in document {$x} return exactly-one($y/a)
    // Expected: false
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_nodeid_3() {
    //  Copied comment node has new node identity
    // XPath: for $x in <!--comment-->, $y in document {$x} return exactly
    // Expected: false
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_nodeid_4() {
    //  Copied pi node has new node identity
    // XPath: for $x in <?pi content?>, $y in document {$x} return exactly
    // Expected: false
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_nodeid_5() {
    //  Copied text node has new node identity
    // XPath: for $x in <a>text</a>, $y in document {$x/text()} return exa
    // Expected: false
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_constrmod_1() {
    //  strip IDREFS type
    // XPath: declare construction strip; fn:count(fn:data((document {//*:
    // Expected: 1
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_string_1() {
    //  string value of element
    // XPath: fn:string(document {'a', element a {}, 'b'})
    // Expected: ab
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn constr_docnode_data_1() {
    //  typed value of element
    // XPath: fn:data(document {'a', element a {}, 'b'})
    // Expected: ab
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn k2_condocnode_14() {
    //  Extract the string value from a document node.
    // XPath: string(document{\"abc\"})
    // Expected: abc
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn k2_condocnode_15() {
    //  Test the atomized type.
    // XPath: data(document {\"content\"}) instance of xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn k2_condocnode_16() {
    //  Test node identity of two document nodes.
    // XPath: document {\"content\"} is document{\"content\"}
    // Expected: false
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn k2_condocnode_17() {
    //  Extract the string value from a complex document node.
    // XPath: string(document{\"string\", <e>more<a>even more</a><b attr=\
    // Expected: stringmoreeven moreButNotThis content
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn k2_condocnode_18() {
    //  Ensure processing instructions and comments are ignored when extracting the str
    // XPath: string(document{ text {\"data\"}, processing-instruction nam
    // Expected: datadatadatadata
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn k2_condocnode_19() {
    //  A recursive construction of document nodes.
    // XPath: count(document{document{document{document{()}}}}/child::node
    // Expected: 0
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn k2_condocnode_20() {
    //  A recursive construction of document nodes, combined with the comma operator.
    // XPath: 1, document{document{document{document{()}}}}/child::node(),
    // Expected: 1 1
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn k2_condocnode_23() {
    //  Extract the string value from a document node with four computed text nodes.
    // XPath: string(document{ text {\"data\"}, text {\"data\"}, text {\"d
    // Expected: datadatadatadata
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn k2_condocnode_24() {
    //  Ensure text nodes gets merged.
    // XPath: count(document{ text {\"data\"}, text {\"data\"}, <e/>, text
    // Expected: 3
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn k2_condocnode_25() {
    //  Ensure text nodes gets merged(#2).
    // XPath: count(document{ text {\"data\"}, text {\"data\"}, text {\"da
    // Expected: 1
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn k2_condocnode_30() {
    //  Ensure two text nodes appearing after a document constructor are merged properl
    // XPath: count(<a>{document {text{'a'}}}b</a>/node())
    // Expected: 1
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn k2_condocnode_31() {
    //  Ensure a text node is properly constructed with nested document constructors, w
    // XPath: string(document {1, document {2, document {document {()}, 3,
    // Expected: 123456
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}

#[test]
fn k2_condocnode_32() {
    //  Ensure a text node is properly constructed with nested document constructors, w
    // XPath: document {1, document {2, document {document {()}, 3, docume
    // Expected: 123456
    // Calls stub function - will fail until prod:CompDocConstructor is implemented
    let _ = stub_prodcompdocconstructor();
}
