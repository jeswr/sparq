//! Test chunk 0 for prod:Annotation
//! Contains 20 tests

use dep::xpath::stub_prodannotation;

#[test]
fn annotation_1() {
    // A function annotation
    // XPath: declare namespace eg = \"http://example.com\";          decl
    // Expected: "bar"
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_2() {
    // A variable annotation
    // XPath: declare namespace eg = \"http://example.com\";          decl
    // Expected: "bar"
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_3() {
    // An inline function annotation
    // XPath: declare namespace eg = \"http://example.com\";          %eg:
    // Expected: "bar"
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_29() {
    // A variable annotation (per bug 16199)
    // XPath: declare %local:x variable $foo := \"bar\";          $foo
    // Expected: "bar"
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_30() {
    // An inline function annotation with parameters
    // XPath: declare namespace eg = \"http://example.com\";          %eg:
    // Expected: "bar"
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_31() {
    // An inline function annotation using an EQName
    // XPath: declare namespace eg = \"http://example.com\";          % Q{
    // Expected: "bar"
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_32() {
    // An inline function with multiple annotations
    // XPath: declare namespace eg = \"http://example.com\";          %eg:
    // Expected: "bar"
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_37() {
    //           Two function annotations with the same name and different values.
    // XPath: declare namespace a = \"http://www.example.org/annotation\";
    // Expected: "bar"
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_38() {
    //           Two variable annotations with the same name and different values.
    // XPath: declare namespace a = \"http://www.example.org/annotation\";
    // Expected: "bar"
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_assertion_1() {
    //           Tests that a function test with an annotation assertion          parse
    // XPath: declare namespace eg = \"http://example.com\";          () i
    // Expected: false
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_assertion_2() {
    //            An annotation assertion with a string literal parameter.
    // XPath: declare namespace eg = \"http://example.com\";          () i
    // Expected: false
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_assertion_3() {
    //            An annotation assertion with an integer literal parameter.
    // XPath: declare namespace eg = \"http://example.com\";          () i
    // Expected: false
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_assertion_4() {
    //            An annotation assertion with a decimal literal parameter.           T
    // XPath: declare namespace eg = \"http://example.com\";          () i
    // Expected: false
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_assertion_5() {
    //            An annotation assertion with a double literal parameter           Tes
    // XPath: declare namespace eg = \"http://example.com\";          () i
    // Expected: false
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_assertion_6() {
    //            An annotation assertion with multiple parameters           Tests the
    // XPath: declare namespace eg = \"http://example.com\";          () i
    // Expected: false
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_assertion_7() {
    //            An annotation assertion with an expanded QName           Tests the pa
    // XPath: () instance of %Q{http://example.com}x function(*)
    // Expected: false
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_assertion_8() {
    //            An annotation assertion with an expanded QName.           Tests the p
    // XPath: () instance of %Q{http://example.com}x function(*)
    // Expected: false
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_assertion_9() {
    //            Multiple annotation assertions.           Tests the parsing of annota
    // XPath: declare namespace eg = \"http://example.com\";          () i
    // Expected: false
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_assertion_10() {
    //            An annotation assertion on a typed function test.           Tests the
    // XPath: declare namespace eg = \"http://example.com\";          () i
    // Expected: false
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}

#[test]
fn annotation_assertion_19() {
    //           Two annotation assertions with the same name          Tests the parsin
    // XPath: declare namespace eg = \"http://example.com\";          () i
    // Expected: false
    // Calls stub function - will fail until prod:Annotation is implemented
    let _ = stub_prodannotation();
}
