//! Test chunk 0 for fn:resolve-QName
//! Contains 25 tests

use dep::xpath::stub_fnresolve_qname;

#[test]
fn fn_resolve_qname_3() {
    //  Evaluation of fn:resolve-qname function for which the give qname is the empty s
    // XPath: fn:count(fn:resolve-QName((), <anElement>Some content</anEle
    // Expected: 0
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_4() {
    //  Evaluation of fn:resolve-qname function for which the give qname has a prefix a
    // XPath: fn:string(fn:local-name-from-QName(fn:resolve-QName(\"p1:nam
    // Expected: name
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_5() {
    //  Evaluation of fn:resolve-qname function for which the give qname has a prefix a
    // XPath: fn:string(fn:namespace-uri-from-QName(fn:resolve-QName(\"p1:
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_6() {
    //  Evaluation of fn:resolve-qname function for which the give qname has a prefix a
    // XPath: fn:string(fn:local-name-from-QName(fn:resolve-QName(\"p1:nam
    // Expected: name
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_7() {
    //  Evaluation of fn:resolve-qname function for which the give qname has a prefix a
    // XPath: fn:string(fn:namespace-uri-from-QName(fn:resolve-QName(\"P1:
    // Expected: http://someothernamespace.com/examples
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_8() {
    //  Evaluation of fn:resolve-qname function for which the given qname has no prefix
    // XPath: fn:string(fn:local-name-from-QName(fn:resolve-QName(\"anElem
    // Expected: anElement
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_9() {
    //  Evaluation of fn:resolve-qname function for which the given qname has no prefix
    // XPath: fn:string(fn:namespace-uri-from-QName(fn:resolve-QName(\"anE
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_10() {
    //  Evaluation of fn:resolve-qname function for which the given qname has no prefix
    // XPath: fn:count(fn:namespace-uri-from-QName(fn:resolve-QName(\"anEl
    // Expected: 1
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_11() {
    //  Evaluation of fn:resolve-qname function for which the given qname has a prefix
    // XPath: fn:string(fn:local-name-from-QName(fn:resolve-QName(xs:strin
    // Expected: anElement
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_12() {
    //  Evaluation of fn:resolve-qname function for which the given qname has a prefix
    // XPath: fn:string(fn:namespace-uri-from-QName(fn:resolve-QName(xs:st
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_13() {
    //  Evaluation of fn:resolve-qname function for which the given qname has a prefix
    // XPath: fn:string(fn:local-name-from-QName(fn:resolve-QName(fn:conca
    // Expected: anElement
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_14() {
    //  Evaluation of fn:resolve-qname function for which the given qname has a prefix
    // XPath: fn:string(fn:namespace-uri-from-QName(fn:resolve-QName(fn:co
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_15() {
    //  Evaluation of fn:resolve-qname function for which the given qname has a prefix
    // XPath: fn:string(fn:local-name-from-QName(fn:resolve-QName(fn:strin
    // Expected: anElement
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_16() {
    //  Evaluation of fn:resolve-qname function for which the given qname has a prefix
    // XPath: fn:string(fn:namespace-uri-from-QName(fn:resolve-QName(fn:st
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_17() {
    //  Evaluation of fn:resolve-qname function for which the given qname has a prefix
    // XPath: fn:string(fn:local-name-from-QName(fn:resolve-QName(fn:subst
    // Expected: anElement
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_18() {
    //  Evaluation of fn:resolve-qname function for which the given qname has a prefix
    // XPath: fn:string(fn:namespace-uri-from-QName(fn:resolve-QName(fn:su
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_19() {
    //  Evaluation of fn:resolve-qname function for which the given qname has a prefix
    // XPath: fn:string(fn:local-name-from-QName(fn:resolve-QName(fn:subst
    // Expected: anElement
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_20() {
    //  Evaluation of fn:resolve-qname function for which the given qname has a prefix
    // XPath: fn:string(fn:namespace-uri-from-QName(fn:resolve-QName(fn:su
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_21() {
    //  Evaluation of fn:resolve-qname function for which the given qname has a prefix
    // XPath: fn:string(fn:local-name-from-QName(fn:resolve-QName(fn:subst
    // Expected: anElement
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_22() {
    //  Evaluation of fn:resolve-qname function for which the given qname has a prefix
    // XPath: fn:string(fn:namespace-uri-from-QName(fn:resolve-QName(fn:su
    // Expected: http://example.com/examples
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_23() {
    //  Test resolve-QName in an XPath environment
    // XPath: namespace-uri-from-QName(resolve-QName(\"dt:integer\", (//*:
    // Expected: http://www.w3.org/2001/XMLSchema
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_24() {
    //  Test resolve-QName in an XPath environment. Use the implicitly-declared XML nam
    // XPath: namespace-uri-from-QName(resolve-QName(\"xml:space\", (//*:C
    // Expected: http://www.w3.org/XML/1998/namespace
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_25() {
    //  Test resolve-QName in an XPath environment. Use the default namespace
    // XPath: namespace-uri-from-QName(resolve-QName(\"artist\", (//*:arti
    // Expected: http://www.example.org/music/records
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn fn_resolve_qname_26() {
    //  Test resolve-QName in an XPath environment. Use a no-namespace name.
    // XPath: namespace-uri-from-QName(resolve-QName(\"artist\", (//*:Clos
    // Expected:
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}

#[test]
fn cbcl_fn_resolve_qname_001() {
    //  Tests fn:resolve-QName on an empty prefix an a node without a default namespace
    // XPath: resolve-QName(\"blah\",<foo:a xmlns:foo=\"http://test/\"/>)
    // Expected: blah
    // Calls stub function - will fail until fn:resolve-QName is implemented
    let _ = stub_fnresolve_qname();
}
