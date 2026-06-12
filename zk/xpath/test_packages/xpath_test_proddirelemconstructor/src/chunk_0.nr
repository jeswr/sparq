//! Test chunk 0 for prod:DirElemConstructor
//! Contains 8 tests

use dep::xpath::stub_proddirelemconstructor;

#[test]
fn constr_elem_empty_3() {
    //  Number of child nodes for empty element
    // XPath: fn:count((<elem/>)/node())
    // Expected: 0
    // Calls stub function - will fail until prod:DirElemConstructor is implemented
    let _ = stub_proddirelemconstructor();
}

#[test]
fn k2_directconelem_1() {
    //  Ensure processing-instructions aren't included when extracting the string-value
    // XPath: string(<pi>{<?pi x?>}</pi>) eq \"\"
    // Expected: true
    // Calls stub function - will fail until prod:DirElemConstructor is implemented
    let _ = stub_proddirelemconstructor();
}

#[test]
fn k2_directconelem_2() {
    //  Ensure comments aren't included when extracting the string-value from elements.
    // XPath: string(<a attr=\"content\"><!-- NOTINC -->1<b>2<c><!-- NOTIN
    // Expected: true
    // Calls stub function - will fail until prod:DirElemConstructor is implemented
    let _ = stub_proddirelemconstructor();
}

#[test]
fn k2_directconelem_44() {
    //  Extract the string value of a single element node.
    // XPath: string(<e>text</e>)
    // Expected: text
    // Calls stub function - will fail until prod:DirElemConstructor is implemented
    let _ = stub_proddirelemconstructor();
}

#[test]
fn k2_directconelem_45() {
    //  Extract the typed value of a single element node.
    // XPath: data(<e>text</e>)
    // Expected: text
    // Calls stub function - will fail until prod:DirElemConstructor is implemented
    let _ = stub_proddirelemconstructor();
}

#[test]
fn k2_directconelem_48() {
    //  In XQuery, as opposed to XML, the sequence ]]< is allowed in element content.
    // XPath: <e>]]></e>
    // Expected: ]]>
    // Calls stub function - will fail until prod:DirElemConstructor is implemented
    let _ = stub_proddirelemconstructor();
}

#[test]
fn cbcl_directconelem_001() {
    // Checks the in-scope namespaces of directly constructed element.
    // XPath: declare default element namespace \"http://example.com/0\";
    // Expected: xmlns:a="http://example.com/1" xmlns:b="
    // Calls stub function - will fail until prod:DirElemConstructor is implemented
    let _ = stub_proddirelemconstructor();
}

#[test]
fn cbcl_directconelem_002() {
    // Checks the in-scope namespaces of directly constructed element.
    // XPath: declare default element namespace \"http://example.com/0\";
    // Expected: xmlns:a="http://example.com/1" xmlns:b="
    // Calls stub function - will fail until prod:DirElemConstructor is implemented
    let _ = stub_proddirelemconstructor();
}
