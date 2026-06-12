//! Test chunk 0 for prod:CompAttrConstructor
//! Contains 7 tests

use dep::xpath::stub_prodcompattrconstructor;

#[test]
fn constr_compattr_parent_1() {
    //  empty parent
    // XPath: count((attribute attr {})/..)
    // Expected: 0
    // Calls stub function - will fail until prod:CompAttrConstructor is implemented
    let _ = stub_prodcompattrconstructor();
}

#[test]
fn constr_compattr_string_1() {
    //  string value of attribute
    // XPath: fn:string(attribute attr {'a', element a {}, 'b'})
    // Expected: a  b
    // Calls stub function - will fail until prod:CompAttrConstructor is implemented
    let _ = stub_prodcompattrconstructor();
}

#[test]
fn constr_compattr_data_1() {
    //  typed value of element
    // XPath: fn:data(attribute attr {'a', element a {}, 'b'})
    // Expected: a  b
    // Calls stub function - will fail until prod:CompAttrConstructor is implemented
    let _ = stub_prodcompattrconstructor();
}

#[test]
fn k2_computeconattr_36() {
    //  Empty CDATA sections generate no text nodes(#2).
    // XPath: count(<elem><![CDATA[]]></elem>/text())
    // Expected: 0
    // Calls stub function - will fail until prod:CompAttrConstructor is implemented
    let _ = stub_prodcompattrconstructor();
}

#[test]
fn k2_computeconattr_47() {
    //  Ensure the typed value of computed, constructed attributes has the correct type
    // XPath: data(attribute foo {\"content\"}) instance of xs:untypedAtom
    // Expected: true
    // Calls stub function - will fail until prod:CompAttrConstructor is implemented
    let _ = stub_prodcompattrconstructor();
}

#[test]
fn k2_computeconattr_54() {
    //  Ensure that a valid prefix has been constructed for a QName which the implement
    // XPath: string-length(xs:NCName(prefix-from-QName(node-name(attribut
    // Expected: true
    // Calls stub function - will fail until prod:CompAttrConstructor is implemented
    let _ = stub_prodcompattrconstructor();
}

#[test]
fn k2_computeconattr_55() {
    //  Ensure that the 'xml' prefix has been constructed for a QName which has the XML
    // XPath: prefix-from-QName(node-name(attribute {QName(\"http://www.w3
    // Expected: xml
    // Calls stub function - will fail until prod:CompAttrConstructor is implemented
    let _ = stub_prodcompattrconstructor();
}
