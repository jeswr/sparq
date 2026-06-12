//! Test chunk 0 for op:anyURI-greater-than
//! Contains 10 tests

use dep::xpath::stub_opanyuri_greater_than;

#[test]
fn k2_anyurilege_1() {
    //  Invoked 'le' on xs:anyURI values.
    // XPath: xs:anyURI(\"http://example.com/A\") le xs:anyURI(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-greater-than is implemented
    let _ = stub_opanyuri_greater_than();
}

#[test]
fn k2_anyurilege_2() {
    //  Invoked 'le' on xs:anyURI values.
    // XPath: xs:anyURI(\"http://example.com/A\") le xs:anyURI(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-greater-than is implemented
    let _ = stub_opanyuri_greater_than();
}

#[test]
fn k2_anyurilege_3() {
    //  Invoked 'le' on xs:anyURI values.
    // XPath: not(xs:anyURI(\"http://example.com/B\") le xs:anyURI(\"http:
    // Expected: true
    // Calls stub function - will fail until op:anyURI-greater-than is implemented
    let _ = stub_opanyuri_greater_than();
}

#[test]
fn k2_anyurilege_4() {
    //  Invoked 'gt' on xs:anyURI values.
    // XPath: xs:anyURI(\"http://example.com/B\") ge xs:anyURI(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-greater-than is implemented
    let _ = stub_opanyuri_greater_than();
}

#[test]
fn k2_anyurilege_5() {
    //  Invoked 'gt' on xs:anyURI values.
    // XPath: xs:anyURI(\"http://example.com/B\") gt xs:anyURI(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-greater-than is implemented
    let _ = stub_opanyuri_greater_than();
}

#[test]
fn k2_anyurilege_6() {
    //  Invoked 'gt' on xs:anyURI values.
    // XPath: not(xs:anyURI(\"http://example.com/A\") gt xs:anyURI(\"http:
    // Expected: true
    // Calls stub function - will fail until op:anyURI-greater-than is implemented
    let _ = stub_opanyuri_greater_than();
}

#[test]
fn k2_anyurilege_7() {
    //  Invoked 'le' on xs:anyURI values.
    // XPath: xs:string(\"http://example.com/A\") le xs:anyURI(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-greater-than is implemented
    let _ = stub_opanyuri_greater_than();
}

#[test]
fn k2_anyurilege_8() {
    //  Invoked 'le' on xs:anyURI values.
    // XPath: not(xs:anyURI(\"http://example.com/B\") le xs:string(\"http:
    // Expected: true
    // Calls stub function - will fail until op:anyURI-greater-than is implemented
    let _ = stub_opanyuri_greater_than();
}

#[test]
fn k2_anyurilege_9() {
    //  Invoked 'gt' on xs:anyURI values.
    // XPath: xs:string(\"http://example.com/B\") gt xs:anyURI(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-greater-than is implemented
    let _ = stub_opanyuri_greater_than();
}

#[test]
fn k2_anyurilege_10() {
    //  Invoked 'gt' on xs:anyURI values.
    // XPath: xs:anyURI(\"http://example.com/B\") gt xs:string(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-greater-than is implemented
    let _ = stub_opanyuri_greater_than();
}
