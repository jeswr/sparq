//! Test chunk 0 for fn:escape-html-uri
//! Contains 30 tests

use dep::xpath::stub_fnescape_html_uri;

#[test]
fn fn_escape_html_uri1args_1() {
    // Test escape-html-uri from example defined in functions and operators specificati
    // XPath: escape-html-uri(\"http://www.example.com/00/Weather/CA/Los A
    // Expected: http://www.example.com/00/Weather/CA/Los
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri1args_2() {
    // Test escape-html-uri from example defined in functions and operators specificati
    // XPath: escape-html-uri(\"javascript:if (navigator.browserLanguage =
    // Expected: javascript:if (navigator.browserLanguage
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri1args_3() {
    // Test escape-html-uri with zero-length string argument
    // XPath: escape-html-uri('')
    // Expected:
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri1args_4() {
    // Test escape-html-uri with empty sequence argument
    // XPath: escape-html-uri(())
    // Expected:
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_1() {
    //  Examines that the fn:escape-html-uri function does not escape the lower cases l
    // XPath: fn:escape-html-uri(\"abcdedfghijklmnopqrstuvwxyz\")
    // Expected: abcdedfghijklmnopqrstuvwxyz
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_2() {
    //  Examines that the fn:escape-html-uri function does not escape the upper cases l
    // XPath: fn:escape-html-uri(\"ABCDEFGHIJKLMNOPQRSTUVWXYZ\")
    // Expected: ABCDEFGHIJKLMNOPQRSTUVWXYZ
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_3() {
    //  Examines that the fn:escape-html-uri function does not escape digits.
    // XPath: fn:escape-html-uri(\"a0123456789\")
    // Expected: a0123456789
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_4() {
    //  Examines that the fn:escape-html-uri function does not escape the space.
    // XPath: fn:escape-html-uri(\"example example\")
    // Expected: example example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_5() {
    //  Examines that the fn:escape-html-uri function does not escape the '!' symbol.
    // XPath: fn:escape-html-uri(\"example!example\")
    // Expected: example!example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_6() {
    //  Examines that the fn:escape-html-uri function does not escape the '#' symbol.
    // XPath: fn:escape-html-uri(\"example#example\")
    // Expected: example#example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_7() {
    //  Examines that the fn:escape-html-uri function does not escape the '$' symbol.
    // XPath: fn:escape-html-uri(\"example$example\")
    // Expected: example$example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_8() {
    //  Examines that the fn:escape-html-uri function does not escape the ''' symbol.
    // XPath: fn:escape-html-uri(\"example'example\")
    // Expected: example'example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_9() {
    //  Examines that the fn:escape-html-uri function does not escape the '(' symbol.
    // XPath: fn:escape-html-uri(\"example(example\")
    // Expected: example(example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_10() {
    //  Examines that the fn:escape-html-uri function does not escape the ')' symbol.
    // XPath: fn:escape-html-uri(\"example)example\")
    // Expected: example)example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_11() {
    //  Examines that the fn:escape-html-uri function does not escape the '*' symbol.
    // XPath: fn:escape-html-uri(\"example*example\")
    // Expected: example*example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_12() {
    //  Examines that the fn:escape-html-uri function does not escape the '+' symbol.
    // XPath: fn:escape-html-uri(\"example+example\")
    // Expected: example+example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_13() {
    //  Examines that the fn:escape-html-uri function does not escape the ',' symbol.
    // XPath: fn:escape-html-uri(\"example,example\")
    // Expected: example,example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_14() {
    //  Examines that the fn:escape-html-uri function does not escape the '-' symbol.
    // XPath: fn:escape-html-uri(\"example-example\")
    // Expected: example-example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_15() {
    //  Examines that the fn:escape-html-uri function does not escape the '.' symbol.
    // XPath: fn:escape-html-uri(\"example.example\")
    // Expected: example.example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_16() {
    //  Examines that the fn:escape-html-uri function does not escape the '/' symbol.
    // XPath: fn:escape-html-uri(\"example/example\")
    // Expected: example/example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_17() {
    //  Examines that the fn:escape-html-uri function does not escape the ';' symbol.
    // XPath: fn:escape-html-uri(\"example;example\")
    // Expected: example;example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_18() {
    //  Examines that the fn:escape-html-uri function does not escape the ':' symbol.
    // XPath: fn:escape-html-uri(\"example:example\")
    // Expected: example:example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_19() {
    //  Examines that the fn:escape-html-uri function does not escape the '@' symbol.
    // XPath: fn:escape-html-uri(\"example@example\")
    // Expected: example@example
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_20() {
    //  Examines that the fn:escape-html-uri function does escape the euro symbol.
    // XPath: fn:escape-html-uri(\"example??example\")
    // Expected: example%C3%A9%E2%82%ACexample
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn fn_escape_html_uri_21() {
    //  Examines that the fn:escape-html-uri function does escape the euro symbol.
    // XPath: fn:escape-html-uri(\"example?example\")
    // Expected: example%E2%82%ACexample
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn k_escapehtmlurifunc_3() {
    //  A test whose essence is: `escape-html-uri(()) eq ''`.
    // XPath: escape-html-uri(()) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn k_escapehtmlurifunc_4() {
    //  Combine fn:concat and fn:escape-html-uri.
    // XPath: escape-html-uri(\"http://www.example.com/00/Weather/CA/Los A
    // Expected: true
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn k_escapehtmlurifunc_5() {
    //  Combine fn:concat and fn:escape-html-uri.
    // XPath: escape-html-uri(\"javascript:if (navigator.browserLanguage =
    // Expected: true
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn k_escapehtmlurifunc_6() {
    //  Invoke fn:normalize-space() on the return value of fn:escape-html-uri().
    // XPath: normalize-space(iri-to-uri((\"example.com\", current-time())
    // Expected: example.com
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}

#[test]
fn cbcl_escape_html_uri_001() {
    //  test fn:escape-html-uri with a variety of characters
    // XPath: fn:escape-html-uri(codepoints-to-string((9, 65, 128)))
    // Expected: %09A%C2%80
    // Calls stub function - will fail until fn:escape-html-uri is implemented
    let _ = stub_fnescape_html_uri();
}
