//! Test chunk 0 for fn:resolve-uri
//! Contains 30 tests

use dep::xpath::stub_fnresolve_uri;

#[test]
fn fn_resolve_uri_1() {
    //  Evaluation of resolve-uri function with relative argument set to empty sequence
    // XPath: fn:count(fn:resolve-uri((),\"BaseValue\"))
    // Expected: 0
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_2() {
    //  Evaluation of resolve-uri function with relative argument set to zero length st
    // XPath: declare base-uri \"http://www.example/\"; fn:string(fn:resol
    // Expected: http://www.example/
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_5() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(\"http://www.examples.com\",\"\"))
    // Expected: http://www.examples.com
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_6() {
    //  Evaluation of resolve-uri function with base argument set to an absolute URI (T
    // XPath: fn:string(fn:resolve-uri(\"examples\",\"http://www.examples.
    // Expected: http://www.examples.com/examples
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_7() {
    //           Evaluation of resolve-uri function with base argument set to an absolu
    // XPath: fn:string(fn:resolve-uri(\"examples\",xs:string(\"http://www
    // Expected: http://www.examples.com/examples
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_8() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(fn:string(\"examples\"),fn:string(\
    // Expected: http://www.examples.com/examples
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_10() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(fn:lower-case(\"EXAMPLES\"),fn:lowe
    // Expected: http://www.examples.com/examples
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_11() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(\"examples\",fn:substring(\"1234htt
    // Expected: http://www.examples.com/examples
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_12() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(\"examples\",fn:string-join(('http:
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_13() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(\"examples\",fn:concat(\"http://www
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_14() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(\"examples\",fn:substring-before(\"
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_15() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(\"examples\",fn:substring-after(\"1
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_16() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(fn:string(\"http://www.examples.com
    // Expected: http://www.examples.com/
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_17() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(fn:upper-case(\"http://www.examples
    // Expected: HTTP://WWW.EXAMPLES.COM
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_18() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(fn:lower-case(\"http://www.examples
    // Expected: http://www.examples.com
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_19() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(fn:substring(\"123http://www.exampl
    // Expected: http://www.examples.com
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_20() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(fn:string-join((\"http://www.exampl
    // Expected: http://www.examples.com
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_21() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(fn:concat(\"http://www.examples\",\
    // Expected: http://www.examples.com
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_22() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(fn:substring-before(\"http://www.ex
    // Expected: http://www.example.com
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_23() {
    //  Evaluation of resolve-uri function with relative argument set to an absolute UR
    // XPath: fn:string(fn:resolve-uri(fn:substring-after(\"123http://www.
    // Expected: http://www.example.com
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_25() {
    //  Evaluation of resolve-uri function with an absolute URI for the first argument
    // XPath: string(resolve-uri(\"http://www.example.com/a.html\",\"b.htm
    // Expected: http://www.example.com/a.html
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_27() {
    //  Evaluation of resolve-uri function with a query as part of the base URI
    // XPath: string(resolve-uri(\"b.html\", \"http://www.example.com/a.ht
    // Expected: http://www.example.com/b.html
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_29() {
    //  Evaluation of resolve-uri function with a non-hierarchic relative URI
    // XPath: string(resolve-uri(\"urn:isbn:01234567890X\", \"http://www.e
    // Expected: urn:isbn:01234567890X
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_30() {
    //  Both the absolute URI and relative URI reference are valid IRIs but not valid U
    // XPath: resolve-uri(codepoints-to-string(231)||\".html\", \"http://w
    // Expected: true
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn fn_resolve_uri_31() {
    //  Both the absolute URI and relative URI reference contain %-encoded characters
    // XPath: resolve-uri(\"%C3%A0.html\", \"http://www.example.com/%C3%A7
    // Expected: true
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn k_resolveurifunc_3() {
    //  A test whose essence is: `resolve-uri('relative/uri.ext', 'http://www.example.c
    // XPath: resolve-uri(\"relative/uri.ext\", \"http://www.example.com/\
    // Expected: true
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn k_resolveurifunc_4() {
    //  A test whose essence is: `resolve-uri('', 'http://www.example.com/') eq xs:anyU
    // XPath: resolve-uri(\"\", \"http://www.example.com/\") eq xs:anyURI(
    // Expected: true
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn k_resolveurifunc_5() {
    //  A test whose essence is: `empty(resolve-uri((), 'http://www.example.com/'))`.
    // XPath: empty(resolve-uri((), \"http://www.example.com/\"))
    // Expected: true
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn k_resolveurifunc_6() {
    //  A test whose essence is: `resolve-uri('http://www.example.com/absolute', 'http:
    // XPath: resolve-uri(\"http://www.example.com/absolute\", \"http://ww
    // Expected: true
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}

#[test]
fn cbcl_fn_resolve_uri_001() {
    //  Tests effective boolean value of resolve-uri
    // XPath: boolean(resolve-uri(string-join(for $x in 1 to 10 return \"b
    // Expected: true
    // Calls stub function - will fail until fn:resolve-uri is implemented
    let _ = stub_fnresolve_uri();
}
