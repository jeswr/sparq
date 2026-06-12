//! Test chunk 0 for prod:BaseURIDecl
//! Contains 18 tests

use dep::xpath::stub_prodbaseuridecl;

#[test]
fn base_uri_4() {
    //  Verify that a base-uri declaration containing 'abc&amp;' is a valid base-uri de
    // XPath: declare base-uri \"http://www.example.com/abc&amp;\"; \"aaa\
    // Expected: aaa
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_6() {
    //  Verify that a base-uri declaration containing 'abc&apos;' is a valid base-uri d
    // XPath: declare base-uri \"http://www.example.com/abc&apos;\"; \"aaa
    // Expected: aaa
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_7() {
    //  Evaluates base-uri with the fn:resolve-uri function. Typical usage
    // XPath: declare base-uri \"http://www.example.com/\"; fn:string(fn:r
    // Expected: http://www.example.com/examples
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_9() {
    //  Evaluates base-uri with the fn:static-base-uri function. Typical usage
    // XPath: declare base-uri \"http://example.org\"; fn:string(fn:static
    // Expected: http://example.org
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_11() {
    //  Evaluates base-uri property can contain numbers. Used with static-base--uri fun
    // XPath: declare base-uri \"http://www.example.com/abc123\"; fn:strin
    // Expected: http://www.example.com/abc123
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_13() {
    //  Evaluates base-uri property can contain an escape apostrophe. Used with static-
    // XPath: declare base-uri 'http://www.example.com/abc'''; fn:string(f
    // Expected: http://www.example.com/abc'
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_15() {
    //  Evaluates base-uri property can contain a single character (after the 'http://'
    // XPath: declare base-uri \"http://A\"; fn:string(fn:static-base-uri(
    // Expected: http://A
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_20() {
    //  Evaluates that base-uri property can contain 'base-uri'. Used with static-base-
    // XPath: declare base-uri \"http://www.base-uri.com\"; fn:string(fn:s
    // Expected: http://www.base-uri.com
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_21() {
    //  Evaluates that base-uri property can contain 'BASE-URI'. Used with static-base-
    // XPath: declare base-uri \"http://www.BASE-URI.com\"; fn:string(fn:s
    // Expected: http://www.BASE-URI.com
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_22() {
    // Verify normalization of xs:anyURI (leading spaces). Used with static-base--uri f
    // XPath: declare base-uri \" http://www.example.org/examples\"; fn:st
    // Expected: http://www.example.org/examples
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_23() {
    // Verify normalization of xs:anyURI (trailing spaces). Used with static-base--uri
    // XPath: declare base-uri \"http://www.example.org/examples \"; fn:st
    // Expected: http://www.example.org/examples
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_25() {
    // Verify normalization of xs:anyURI (encoded whitespaces in the middle). Used with
    // XPath: declare base-uri \"http://www.example.org/%20%20examples\";
    // Expected: http://www.example.org/%20%20examples
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_26() {
    //  Evaluation of base-uri set to a relative value.
    // XPath: declare base-uri \"abc\"; fn:ends-with(fn:string(fn:static-b
    // Expected: true
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_27() {
    //  relative base-uri
    // XPath: declare base-uri \"http://www.example.com/\"; fn:base-uri(<e
    // Expected: http://www.example.com/fluster
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_28() {
    //  relative base-uri through parent
    // XPath: declare base-uri \"http://www.example.com/\"; fn:base-uri(ex
    // Expected: http://www.example.com/fluster
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_29() {
    //  relative base-uri through parent
    // XPath: declare base-uri \"http://www.example.com/\"; fn:base-uri(ex
    // Expected: http://www.example.com/fluster/now
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn k_baseuriprolog_1() {
    //  Test 'declare base-uri' with fn:static-base-uri().
    // XPath: declare(::)base-uri(::)\"http://example.com/declareBaseURITe
    // Expected: true
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}

#[test]
fn base_uri_modules_001() {
    // Test the base uri declaration in an imported module.
    // XPath: declare base-uri \"http://www.example.org/base1\";
    // Expected: false
    // Calls stub function - will fail until prod:BaseURIDecl is implemented
    let _ = stub_prodbaseuridecl();
}
