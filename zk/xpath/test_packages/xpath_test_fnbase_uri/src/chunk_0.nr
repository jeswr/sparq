//! Test chunk 0 for fn:base-uri
//! Contains 50 tests

use dep::xpath::stub_fnbase_uri;

#[test]
fn fn_base_uri_3() {
    //  Evaluation of base-uri function with argument set to empty sequence Use fn:coun
    // XPath: fn:count(fn:base-uri(()))
    // Expected: 0
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_9() {
    //  Evaluation of base-uri function with argument set to a directly constructed Ele
    // XPath: fn:string(fn:base-uri(<anElement xml:base=\"http://example.c
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_10() {
    //  Evaluation of base-uri function with argument set to a directly constructed Ele
    // XPath: fn:string(fn:base-uri(<anElement xml:base=\"http://www.examp
    // Expected: http://www.example.com
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_12() {
    //  Evaluation of base-uri function with argument set to a computed constructed Doc
    // XPath: declare base-uri \"http://example.org\";              fn:str
    // Expected: http://example.org
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_13() {
    //  Evaluation of base-uri function with argument set to a directly constructed ele
    // XPath: declare base-uri \"http://example.org\"; fn:string(fn:base-u
    // Expected: http://example.org
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_18() {
    //  Evaluation of base-uri function with argument set to a directly constructed ele
    // XPath: declare base-uri \"http://example.org\";          let $var :
    // Expected: http://example.org
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_19() {
    //  Evaluation of base-uri function with argument set to a directly constructed ele
    // XPath: let $var := <anElement xml:base=\"http://www.examples.com\">
    // Expected: http://www.examples.com
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_20() {
    //  Evaluation of base-uri function with argument set to a directly constructed ele
    // XPath: declare base-uri \"http://example.org\";          let $var :
    // Expected: http://www.examples.com
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_23() {
    //  Evaluation of base-uri function with argument set to a directly constructed Ele
    // XPath: fn:string(fn:base-uri(<anElement xml:base=\"http://example.c
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_29() {
    //  Evaluation of base-uri#0 function with context item set to a directly construct
    // XPath: fn:string((<anElement xml:base=\"http://example.com/examples
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_30() {
    //  Evaluation of base-uri#0 function with context item set to a directly construct
    // XPath: fn:string((<anElement xml:base=\"http://www.example.com\">El
    // Expected: http://www.example.com
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_32() {
    //  Evaluation of base-uri#0 function with context item set to a computed construct
    // XPath: declare base-uri \"http://example.org\";              fn:str
    // Expected: http://example.org
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_33() {
    //  Evaluation of base-uri#0 function with context item set to a directly construct
    // XPath: declare base-uri \"http://example.org\"; fn:string((<anEleme
    // Expected: http://example.org
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_38() {
    //  Evaluation of base-uri#0 function with context item set to a directly construct
    // XPath: declare base-uri \"http://example.org\";          let $var :
    // Expected: http://example.org
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_39() {
    //  Evaluation of base-uri#0 function with context item set to a directly construct
    // XPath: let $var := <anElement xml:base=\"http://www.examples.com\">
    // Expected: http://www.examples.com
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_40() {
    //  Evaluation of base-uri#0 function with context item set to a directly construct
    // XPath: declare base-uri \"http://example.org\";          let $var :
    // Expected: http://www.examples.com
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn fn_base_uri_43() {
    //  Evaluation of base-uri#0 function with context item set to a directly construct
    // XPath: fn:string((<anElement xml:base=\"http://example.com/examples
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k_baseurifunc_2() {
    //  A test whose essence is: `empty(base-uri(()))`.
    // XPath: empty(base-uri(()))
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_1() {
    //  Invoke on a comment node.
    // XPath: empty(document-uri(<!-- comment -->))
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_2() {
    //  Invoke on an attribute node.
    // XPath: empty(document-uri(attribute name {\"content\"}))
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_3() {
    //  Invoke on a processing instruction node.
    // XPath: empty(document-uri(<?target data?>))
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_4() {
    //  Invoke on a processing instruction node.
    // XPath: empty(document-uri(processing-instruction name {123}))
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_5() {
    //  Invoke on a text node.
    // XPath: empty(document-uri(text {123}))
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_6() {
    //  Invoke on a single element node.
    // XPath: empty(document-uri(<elem/>))
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_7() {
    //  Invoke on a single attribute node.
    // XPath: empty(document-uri(<elem attr=\"f\"/>/@attr))
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_8() {
    //  Invoke on a single document node.
    // XPath: empty(document-uri(document {1}))
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_9() {
    //  Ensure processing-instructions pick up the correct xml:base value.
    // XPath: let $i := <e xml:base=\"http://www.example.com/\"><?target d
    // Expected: http://www.example.com/
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_10() {
    //  Ensure computed processing-instructions pick up the correct xml:base value.
    // XPath: let $i := <e xml:base=\"http://www.example.com/\">{processin
    // Expected: http://www.example.com/
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_11() {
    //  Ensure comments pick up the correct xml:base value.
    // XPath: let $i := <e xml:base=\"http://www.example.com/\"><!-- conte
    // Expected: http://www.example.com/
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_12() {
    //  Ensure computed comments pick up the correct xml:base value.
    // XPath: let $i := <e xml:base=\"http://www.example.com/\">{comment {
    // Expected: http://www.example.com/
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_13() {
    //  Ensure computed comments don't pick up the base-uri from the static context.
    // XPath: declare base-uri \"http://www.example.com\"; empty(base-uri(
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_14() {
    //  Ensure computed comments don't pick up the base-uri from the static context.
    // XPath: declare base-uri \"http://www.example.com\"; empty(base-uri(
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_15() {
    //  Ensure computed processing-instructions don't pick up the base-uri from the sta
    // XPath: declare base-uri \"http://www.example.com\"; empty(base-uri(
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_16() {
    //  Ensure processing-instructions don't pick up the base-uri from the static conte
    // XPath: declare base-uri \"http://www.example.com\"; empty(base-uri(
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_17() {
    //  Ensure computed attributes don't pick up the base-uri from the static context.
    // XPath: declare base-uri \"http://www.example.com\"; empty(base-uri(
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_18() {
    //  Ensure computed attributes pick up the base-uri from parent.
    // XPath: declare base-uri \"http://www.example.com\"; let $i := <e at
    // Expected: http://www.example.com
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_19() {
    //  Ensure computed attributes pick up the resolved base-uri of the parent.
    // XPath: declare base-uri \"http://www.example.com/\"; let $i := <e x
    // Expected: http://www.example.com/xml
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_20() {
    //  Ensure computed attributes pick up the resolved base-uri of the parent(#2).
    // XPath: declare base-uri \"http://www.example.com/\"; let $i := <e x
    // Expected: http://www.example.com/xml
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_21() {
    //  Ensure that the return value of document-uri() is of correct type.
    // XPath: for $i in (1, base-uri(.), 3) return          typeswitch($i)
    // Expected: xs:integer xs:anyURI xs:integer
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_22() {
    //  Ensure that the base URI is empty for direct PI constructors.
    // XPath: declare base-uri \"http://example.com/baseURI\"; empty(base-
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_23() {
    //  Ensure that the base URI is empty for computed PI constructors.
    // XPath: declare base-uri \"http://example.com/baseURI\"; empty(base-
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_24() {
    //  Ensure that the base URI is empty for direct PI constructors, with no base-uri
    // XPath: empty(base-uri(<?target data?>))
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_25() {
    //  Ensure that the base URI is empty for computed PI constructors, with no base-ur
    // XPath: empty(base-uri(processing-instruction target {\"data\"}))
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_26() {
    //  Ensure that the base URI is empty for computed attribute constructors, with no
    // XPath: empty(base-uri(attribute name {\"value\"}))
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_27() {
    //  Ensure that the base URI is set for computed documents.
    // XPath: declare base-uri \"http://example.com/BASEURI\"; base-uri(do
    // Expected: http://example.com/BASEURI
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_28() {
    //  Check the document URI and base URI simultaneously for a computed document node
    // XPath: declare base-uri \"http://example.com/\"; let $i := document
    // Expected: Base URI: http://example.com/ Document U
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_29() {
    //  Use an URI in an xml:base element that is a valid URI, but an invalid HTTP URL.
    // XPath: let $i := fn:base-uri(<anElement xml:base=\"http:\\example.c
    // Expected: true
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_30() {
    //  Use an empty xml:base attribute.
    // XPath: fn:base-uri(<anElement xml:base=\"http://example.com/example
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_31() {
    //  Compute the base URI from a processing instruction.
    // XPath: fn:base-uri(exactly-one(<anElement xml:base=\"http://example
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}

#[test]
fn k2_baseurifunc_32() {
    //  Compute the base URI from a comment.
    // XPath: fn:base-uri(exactly-one(<anElement xml:base=\"http://example
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:base-uri is implemented
    let _ = stub_fnbase_uri();
}
