//! Test chunk 0 for op:anyURI-less-than
//! Contains 8 tests

use dep::xpath::stub_opanyuri_less_than;

#[test]
fn k2_anyuriltgt_1() {
    //  Invoked 'lt' on xs:anyURI values.
    // XPath: xs:anyURI(\"http://example.com/A\") lt xs:anyURI(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-less-than is implemented
    let _ = stub_opanyuri_less_than();
}

#[test]
fn k2_anyuriltgt_2() {
    //  Invoked 'lt' on xs:anyURI values.
    // XPath: not(xs:anyURI(\"http://example.com/B\") lt xs:anyURI(\"http:
    // Expected: true
    // Calls stub function - will fail until op:anyURI-less-than is implemented
    let _ = stub_opanyuri_less_than();
}

#[test]
fn k2_anyuriltgt_3() {
    //  Invoked 'lt' on xs:anyURI values.
    // XPath: xs:string(\"http://example.com/A\") lt xs:anyURI(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-less-than is implemented
    let _ = stub_opanyuri_less_than();
}

#[test]
fn k2_anyuriltgt_4() {
    //  Invoked 'lt' on xs:anyURI values.
    // XPath: not(xs:anyURI(\"http://example.com/B\") lt xs:string(\"http:
    // Expected: true
    // Calls stub function - will fail until op:anyURI-less-than is implemented
    let _ = stub_opanyuri_less_than();
}

#[test]
fn k2_anyuriltgt_5() {
    //  Invoked 'gt' on xs:anyURI values.
    // XPath: not(xs:anyURI(\"http://example.com/A\") gt xs:anyURI(\"http:
    // Expected: true
    // Calls stub function - will fail until op:anyURI-less-than is implemented
    let _ = stub_opanyuri_less_than();
}

#[test]
fn k2_anyuriltgt_6() {
    //  Invoked 'gt' on xs:anyURI values.
    // XPath: not(xs:string(\"http://example.com/A\") gt xs:anyURI(\"http:
    // Expected: true
    // Calls stub function - will fail until op:anyURI-less-than is implemented
    let _ = stub_opanyuri_less_than();
}

#[test]
fn k2_anyuriltgt_7() {
    //  Invoked 'gt' on xs:anyURI values.
    // XPath: xs:anyURI(\"http://example.com/B\") gt xs:anyURI(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-less-than is implemented
    let _ = stub_opanyuri_less_than();
}

#[test]
fn k2_anyuriltgt_8() {
    //  Invoked 'gt' on xs:anyURI values.
    // XPath: xs:anyURI(\"http://example.com/B\") gt xs:string(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-less-than is implemented
    let _ = stub_opanyuri_less_than();
}
