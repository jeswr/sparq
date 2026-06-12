//! Test chunk 0 for fn:namespace-uri
//! Contains 24 tests

use dep::xpath::stub_fnnamespace_uri;

#[test]
fn fn_namespace_uri_4() {
    //  Evaluation of the fn:namespace-uri function argument set to empty sequence.
    // XPath: fn:namespace-uri(())
    // Expected: ""
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_5() {
    //  Evaluation of the fn:namespace-uri function argument set to a comment node. Use
    // XPath: namespace-uri((//comment())[1])
    // Expected: ""
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_6() {
    //  Evaluation of the fn:namespace-uri function argument set to a computed comment
    // XPath: namespace-uri(<!--comment-->)
    // Expected: ""
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_7() {
    //  Evaluation of the fn:namespace-uri function argument set to a processing instru
    // XPath: namespace-uri((//processing-instruction())[1])
    // Expected: ""
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_8() {
    //  Evaluation of the fn:namespace-uri function argument set to a constructed proce
    // XPath: namespace-uri(<?pi data?>)
    // Expected: ""
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_9() {
    //  Evaluation of the fn:namespace-uri function argument set to a text node. Use th
    // XPath: namespace-uri((//text())[1])
    // Expected: ""
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_10() {
    //  Evaluation of the fn:namespace-uri function argument set to an element node wit
    // XPath: namespace-uri(/*)
    // Expected: ""
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_11() {
    //  Evaluation of the fn:namespace-uri function argument set to a computed element
    // XPath: fn:namespace-uri(element elementNode {\"with no namespace\"}
    // Expected: ""
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_12() {
    //  Evaluation of the fn:namespace-uri function argument set to an attribute node w
    // XPath: fn:namespace-uri(/works/employee[1]/@name)
    // Expected: ""
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_14() {
    //  Evaluation of the fn:namespace-uri function argument set to an element node wit
    // XPath: fn:namespace-uri(/works[1]/employee[1])
    // Expected: ""
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_15() {
    //  Evaluation of the fn:namespace-uri function argument set to an attribute node w
    // XPath: fn:namespace-uri(/works[1]/employee[1]/@name)
    // Expected: ""
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_18() {
    //  Evaluation of the fn:namespace-uri function with argument set to a computed ele
    // XPath: declare namespace ex = \"http://www.example.com/examples\";
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_19() {
    // Evaluation of the fn:namespace-uri function with argument set to a computed elem
    // XPath: declare namespace ex = \"http://www.example.com/examples\";
    // Expected:
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_20() {
    //  Evaluation of the fn:namespace-uri function with argument set to a direct eleme
    // XPath: declare namespace ex = \"http://www.example.com/examples\";
    // Expected:
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_21() {
    //  Evaluation of the fn:namespace-uri function with argument set to a direct eleme
    // XPath: declare namespace ex = \"http://www.example.com/examples\";
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_22() {
    //  Evaluation of the fn:namespace-uri function with argument set to a direct eleme
    // XPath: declare default element namespace \"http://www.example.com/e
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_23() {
    //  Evaluation of the fn:namespace-uri function with argument set to a direct eleme
    // XPath: declare default element namespace \"http://www.example.com/e
    // Expected: http://www.example.com/exampleswithPrefi
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_24() {
    //  Evaluation of the fn:namespace-uri function with argument set to a computed ele
    // XPath: declare default element namespace \"http://www.example.com/e
    // Expected: http://www.example.com/exampleswithPrefi
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn fn_namespace_uri_25() {
    //  Evaluation of the fn:namespace-uri function with argument set to a computed ele
    // XPath: declare default element namespace \"http://www.example.com/e
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn k_nodenamespaceurifunc_3() {
    //  A test whose essence is: `namespace-uri(()) eq xs:anyURI('')`.
    // XPath: namespace-uri(()) eq xs:anyURI(\"\")
    // Expected: true
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn k2_nodenamespaceurifunc_1() {
    //  Extract the namespace URI from a processing instruction node.
    // XPath: namespace-uri(<?target data?>) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn k2_nodenamespaceurifunc_2() {
    //  Extract the namespace URI from a comment node.
    // XPath: namespace-uri(<!--comment-->) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn k2_nodenamespaceurifunc_3() {
    //  Extract the namespace URI from a text node.
    // XPath: namespace-uri(text{()}) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}

#[test]
fn cbcl_namespace_uri_001() {
    //  test fn:boolean on fn:namespace-uri
    // XPath: fn:boolean(fn:namespace-uri(<element />))
    // Expected: false
    // Calls stub function - will fail until fn:namespace-uri is implemented
    let _ = stub_fnnamespace_uri();
}
