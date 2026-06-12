//! Test chunk 0 for prod:DirElemContent.namespace
//! Contains 8 tests

use dep::xpath::stub_proddirelemcontent_namespace;

#[test]
fn k2_directconelemnamespace_5() {
    //  Verify the 'xml' prefix is in scope in a direct constructor.
    // XPath: count(in-scope-prefixes(<e/>)[. eq \"xml\"])
    // Expected: 1
    // Calls stub function - will fail until prod:DirElemContent.namespace is implemented
    let _ = stub_proddirelemcontent_namespace();
}

#[test]
fn k2_directconelemnamespace_6() {
    //  Verify the 'xml' prefix is in scope in a computed constructor.
    // XPath: count(in-scope-prefixes(element e{})[. eq \"xml\"])
    // Expected: 1
    // Calls stub function - will fail until prod:DirElemContent.namespace is implemented
    let _ = stub_proddirelemcontent_namespace();
}

#[test]
fn k2_directconelemnamespace_41() {
    //  Ensure all namespaces in-scope, through a couple of elements, are found by in-s
    // XPath: declare namespace p = \"http://www.example.com/A\"; \"START\
    // Expected: START  A B xml END
    // Calls stub function - will fail until prod:DirElemContent.namespace is implemented
    let _ = stub_proddirelemcontent_namespace();
}

#[test]
fn k2_directconelemnamespace_53() {
    //  Ensure a namespace undeclaration aren't counted as a namespace binding.
    // XPath: count(in-scope-prefixes(<a xmlns=\"http://example.com/\"> <e
    // Expected: 1
    // Calls stub function - will fail until prod:DirElemContent.namespace is implemented
    let _ = stub_proddirelemcontent_namespace();
}

#[test]
fn k2_directconelemnamespace_54() {
    //  Ensure an undeclaration is treated properly.
    // XPath: count(<e xmlns=\"http://example.com/\"><a xmlns=\"\"/></e>/n
    // Expected: 1
    // Calls stub function - will fail until prod:DirElemContent.namespace is implemented
    let _ = stub_proddirelemcontent_namespace();
}

#[test]
fn directconelemnamespace_5() {
    //  Use the ''-escape mechanism. Modified by MHK to avoid generating output that wo
    // XPath: namespace-uri(<p:e xmlns:p='http://ns.example.com/ns?val='''
    // Expected: http://ns.example.com/ns?val='''asd
    // Calls stub function - will fail until prod:DirElemContent.namespace is implemented
    let _ = stub_proddirelemcontent_namespace();
}

#[test]
fn directconelemnamespace_6() {
    //  Use the ''-escape mechanism. Modified by MHK to avoid generating output that wo
    // XPath: namespace-uri(<e xmlns='http://ns.example.com/ns?val=''asd'/
    // Expected: http://ns.example.com/ns?val='asd
    // Calls stub function - will fail until prod:DirElemContent.namespace is implemented
    let _ = stub_proddirelemcontent_namespace();
}

#[test]
fn k2_coninscopenamespace_1() {
    //  Ensure the in-scope prefixes are correct for all top-level elements.
    // XPath: let $i := document{<e1/>, <e2/>, <e3/>, <e4/>} return (in-sc
    // Expected: xml xml xml xml
    // Calls stub function - will fail until prod:DirElemContent.namespace is implemented
    let _ = stub_proddirelemcontent_namespace();
}
