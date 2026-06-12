//! Test chunk 1 for prod:DirElemContent
//! Contains 11 tests

use dep::xpath::stub_proddirelemcontent;

#[test]
fn k2_directconelemcontent_31() {
    //  Two atomic values with a text node inbetween(#2).
    // XPath: string(<e>{1}A{1}</e>)
    // Expected: 1A1
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_32() {
    //  Tests the type of the typed value of a directly constructed element node.
    // XPath: data(<e>dsa</e>) instance of xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_33() {
    //  Tests the typed value of a directly constructed element node.
    // XPath: data(<e>dsa</e>)
    // Expected: dsa
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_34() {
    //  Check that a directly constructed element gets the right type.
    // XPath: <e/> instance of element(*, xs:anyType)
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_35() {
    //          Check that a directly constructed element gets the right type(#2).
    // XPath: <e/> instance of element(*, xs:untyped)
    // Expected: false
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_35a() {
    //          Check that a directly constructed element gets the right type(#2).
    // XPath: <e/> instance of element(*, xs:untyped)
    // Expected: false
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_35b() {
    //          Check that a directly constructed element gets the right type(#2).
    // XPath: <e/> instance of element(*, xs:untyped)
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_36() {
    //  Check that a directly constructed element gets the right type(#3).
    // XPath: declare construction strip; <e/> instance of element(*, xs:u
    // Expected: true
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_37() {
    //  Check that a directly constructed element gets the right type, and that the nam
    // XPath: <e/> instance of element(a, xs:anyType)
    // Expected: false
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn k2_directconelemcontent_38() {
    //  Check that a directly constructed element gets the right type, and that the nam
    // XPath: declare construction strip; <e/> instance of element(b, xs:u
    // Expected: false
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}

#[test]
fn cbcl_ns_fixup_1() {
    //  Ensure that namespace fixup occurs when namespace prefix names clash
    // XPath: let $x := <ns:foo xmlns:ns=\"http://www.w3.org/foo\" ns:attr
    // Expected: 3
    // Calls stub function - will fail until prod:DirElemContent is implemented
    let _ = stub_proddirelemcontent();
}
