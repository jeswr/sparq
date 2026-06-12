//! Test chunk 0 for op:anyURI-equal
//! Contains 22 tests

use dep::xpath::stub_opanyuri_equal;

#[test]
fn k_anyuriequal_1() {
    //  A test whose essence is: `xs:anyURI('example.com/') eq xs:anyURI('example.com/'
    // XPath: xs:anyURI(\"example.com/\") eq xs:anyURI(\"example.com/\")
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k_anyuriequal_2() {
    //  A test whose essence is: `xs:anyURI('example.com/') ne xs:anyURI('example.com/N
    // XPath: xs:anyURI(\"example.com/\") ne xs:anyURI(\"example.com/No\")
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k_anyuriequal_3() {
    //  A test whose essence is: `xs:anyURI('example.com/') eq xs:string('example.com/'
    // XPath: xs:anyURI(\"example.com/\") eq xs:string(\"example.com/\")
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k_anyuriequal_4() {
    //  A test whose essence is: `xs:string('example.com/') eq xs:anyURI('example.com/'
    // XPath: xs:string(\"example.com/\") eq xs:anyURI(\"example.com/\")
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k_anyuriequal_5() {
    //  A test whose essence is: `xs:anyURI('example.com/') ne xs:string('example.com/N
    // XPath: xs:anyURI(\"example.com/\") ne xs:string(\"example.com/No\")
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k_anyuriequal_6() {
    //  A test whose essence is: `xs:string('example.com/') ne xs:anyURI('example.com/N
    // XPath: xs:string(\"example.com/\") ne xs:anyURI(\"example.com/No\")
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k_anyuriequal_7() {
    //  A test whose essence is: `xs:anyURI('example.com/') eq xs:untypedAtomic('exampl
    // XPath: xs:anyURI(\"example.com/\") eq xs:untypedAtomic(\"example.co
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k_anyuriequal_8() {
    //  A test whose essence is: `xs:untypedAtomic('example.com/') eq xs:anyURI('exampl
    // XPath: xs:untypedAtomic(\"example.com/\") eq xs:anyURI(\"example.co
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k_anyuriequal_9() {
    //  A test whose essence is: `xs:anyURI('example.com/') ne xs:untypedAtomic('exampl
    // XPath: xs:anyURI(\"example.com/\") ne xs:untypedAtomic(\"example.co
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k_anyuriequal_10() {
    //  A test whose essence is: `xs:untypedAtomic('example.com/') ne xs:anyURI('exampl
    // XPath: xs:untypedAtomic(\"example.com/\") ne xs:anyURI(\"example.co
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k2_anyuriequal_1() {
    //  Invoked 'eq' on xs:anyURI values.
    // XPath: xs:string(\"http://example.com/A\") eq xs:anyURI(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k2_anyuriequal_2() {
    //  Invoked 'eq' on xs:anyURI values.
    // XPath: xs:anyURI(\"http://example.com/A\") eq xs:string(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k2_anyuriequal_3() {
    //  Invoked 'eq' on xs:anyURI values.
    // XPath: xs:anyURI(\"http://example.com/A\") eq xs:anyURI(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k2_anyuriequal_4() {
    //  Invoked 'eq' on xs:anyURI values.
    // XPath: not(xs:string(\"http://example.com/A\") eq xs:anyURI(\"http:
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k2_anyuriequal_5() {
    //  Invoked 'eq' on xs:anyURI values.
    // XPath: not(xs:anyURI(\"http://example.com/A\") eq xs:string(\"http:
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k2_anyuriequal_6() {
    //  Invoked 'eq' on xs:anyURI values.
    // XPath: not(xs:anyURI(\"http://example.com/A\") eq xs:anyURI(\"http:
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k2_anyuriequal_7() {
    //  Invoked 'ne' on xs:anyURI values.
    // XPath: not(xs:anyURI(\"http://example.com/A\") ne xs:string(\"http:
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k2_anyuriequal_8() {
    //  Invoked 'ne' on xs:anyURI values.
    // XPath: not(xs:string(\"http://example.com/A\") ne xs:anyURI(\"http:
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k2_anyuriequal_9() {
    //  Invoked 'ne' on xs:anyURI values.
    // XPath: not(xs:anyURI(\"http://example.com/A\") ne xs:anyURI(\"http:
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k2_anyuriequal_10() {
    //  Invoked 'ne' on xs:anyURI values.
    // XPath: xs:anyURI(\"http://example.com/A\") ne xs:string(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k2_anyuriequal_11() {
    //  Invoked 'ne' on xs:anyURI values.
    // XPath: xs:string(\"http://example.com/A\") ne xs:anyURI(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}

#[test]
fn k2_anyuriequal_12() {
    //  Invoked 'ne' on xs:anyURI values.
    // XPath: xs:anyURI(\"http://example.com/A\") ne xs:anyURI(\"http://ex
    // Expected: true
    // Calls stub function - will fail until op:anyURI-equal is implemented
    let _ = stub_opanyuri_equal();
}
