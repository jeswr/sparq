//! Test chunk 0 for fn:iri-to-uri
//! Contains 38 tests

use dep::xpath::stub_fniri_to_uri;

#[test]
fn fn_iri_to_uri1args_1() {
    // Test iri-to-uri from example defined in functions and operators specification
    // XPath: iri-to-uri(\"http://www.example.com/00/Weather/CA/Los%20Ange
    // Expected: http://www.example.com/00/Weather/CA/Los
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri1args_2() {
    // Test iri-to-uri from example defined in functions and operators specification
    // XPath: iri-to-uri(\"http://www.example.com/~b?b?\")
    // Expected: http://www.example.com/~b%C3%A9b%C3%A9
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri1args_3() {
    // Test iri-to-uri with zero-length string argument
    // XPath: iri-to-uri('')
    // Expected:
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri1args_4() {
    // Test iri-to-uri with empty sequence argument
    // XPath: iri-to-uri(())
    // Expected:
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_1() {
    //  Examines the fn:iri-to-uri function with nothing to escape.
    // XPath: fn:iri-to-uri(\"example\")
    // Expected: example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_2() {
    //  Examines that the fn:iri-to-uri function does not escape numbers.
    // XPath: fn:iri-to-uri(\"example0123456789\")
    // Expected: example0123456789
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_3() {
    //  Examines that the fn:iri-to-uri function does not escapes the '#' symbol.
    // XPath: fn:iri-to-uri(\"example#example\")
    // Expected: example#example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_4() {
    //  Examines that the fn:iri-to-uri function does not escapes the '-' symbol.
    // XPath: fn:iri-to-uri(\"example-example\")
    // Expected: example-example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_5() {
    //  Examines that the fn:iri-to-uri function does not escape the '_' symbol.
    // XPath: fn:iri-to-uri(\"example_example\")
    // Expected: example_example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_6() {
    //  Examines that the fn:iri-to-uri function does not escapes the '.' symbol.
    // XPath: fn:iri-to-uri(\"example.example\")
    // Expected: example.example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_7() {
    //  Examines that the fn:iri-to-uri function does escape not the '!' symbol.
    // XPath: fn:iri-to-uri(\"example!example\")
    // Expected: example!example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_8() {
    //  Examines that the fn:iri-to-uri function does not escape the '~' symbol.
    // XPath: fn:iri-to-uri(\"example~example\")
    // Expected: example~example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_9() {
    //  Examines that the fn:iri-to-uri function does not escape the '*' symbol.
    // XPath: fn:iri-to-uri(\"example*example\")
    // Expected: example*example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_10() {
    //  Examines that the fn:iri-to-uri function does not escape the ''' symbol.
    // XPath: fn:iri-to-uri(\"example'example\")
    // Expected: example'example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_11() {
    //  Examines that the fn:iri-to-uri function does not escape the '(' symbol.
    // XPath: fn:iri-to-uri(\"example(example\")
    // Expected: example(example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_12() {
    //  Examines that the fn:iri-to-uri function does not escape the ')' symbol.
    // XPath: fn:iri-to-uri(\"example)example\")
    // Expected: example)example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_13() {
    //  Examines that the fn:iri-to-uri function does not escape the ';' symbol.
    // XPath: fn:iri-to-uri(\"example;example\")
    // Expected: example;example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_14() {
    //  Examines that the fn:iri-to-uri function does not escape the '/' symbol.
    // XPath: fn:iri-to-uri(\"example/example\")
    // Expected: example/example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_15() {
    //  Examines that the fn:iri-to-uri function does not escape the '?' symbol.
    // XPath: fn:iri-to-uri(\"example?example\")
    // Expected: example?example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_16() {
    //  Examines that the fn:iri-to-uri function does not escape the ':' symbol.
    // XPath: fn:iri-to-uri(\"example:example\")
    // Expected: example:example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_17() {
    //  Examines that the fn:iri-to-uri function does not escape the '@' symbol.
    // XPath: fn:iri-to-uri(\"example@example\")
    // Expected: example@example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_18() {
    //  Examines that the fn:iri-to-uri function does escape not the '&' symbol.
    // XPath: fn:iri-to-uri(\"example&amp;amp;example\")
    // Expected: example&amp;example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_18a() {
    //  Examines that the fn:iri-to-uri function does escape not the '&' symbol.
    // XPath: fn:iri-to-uri(\"example&amp;example\")
    // Expected: example&amp;example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_19() {
    //  Examines that the fn:iri-to-uri function does not escape the '=' symbol.
    // XPath: fn:iri-to-uri(\"example=example\")
    // Expected: example=example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_20() {
    //  Examines that the fn:iri-to-uri function does not escape '+' symbol.
    // XPath: fn:iri-to-uri(\"example+example\")
    // Expected: example+example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_21() {
    //  Examines that the fn:iri-to-uri function does not escape the '$' symbol.
    // XPath: fn:iri-to-uri(\"example$example\")
    // Expected: example$example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_22() {
    //  Examines that the fn:iri-to-uri function does not escape the ',' symbol.
    // XPath: fn:iri-to-uri(\"example,example\")
    // Expected: example,example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_23() {
    //  Examines that the fn:iri-to-uri function does not escape the '[' symbol.
    // XPath: fn:iri-to-uri(\"example[example\")
    // Expected: example[example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_24() {
    //  Examines that the fn:iri-to-uri function does not escape the ']' symbol.
    // XPath: fn:iri-to-uri(\"example]example\")
    // Expected: example]example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_25() {
    //  Examines that the fn:iri-to-uri function does not escape the '%' symbol.
    // XPath: fn:iri-to-uri(\"example%example\")
    // Expected: example%example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn fn_iri_to_uri_26() {
    //  Examines that the fn:iri-to-uri with an iri that contains a space.
    // XPath: fn:iri-to-uri(\"example example\")
    // Expected: example%20example
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn k_iritourifunc_3() {
    //  A test whose essence is: `iri-to-uri(()) eq ''`.
    // XPath: iri-to-uri(()) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn k2_iritourifunc_5() {
    //  Passing xs:anyURI causes type promotion.
    // XPath: iri-to-uri(xs:anyURI(\"a string\"))
    // Expected: a%20string
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn k2_iritourifunc_6() {
    //  Passing xs:untypedAtomic causes type promotion.
    // XPath: iri-to-uri(xs:untypedAtomic(\"a string\"))
    // Expected: a%20string
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn k2_iritourifunc_7() {
    //  Check some ASCII characters that must be escaped.
    // XPath: iri-to-uri(\"<> \"\"{}|\^`\")
    // Expected: %3C%3E%20%22%7B%7D%7C%5C%5E%60
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn k2_iritourifunc_8() {
    //  Pass a new-line character.
    // XPath: iri-to-uri(\" \")
    // Expected: %0A
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn k2_iritourifunc_9() {
    //  Pass the range of 0x20 to 0x126(hexa decimals). 0x32 to 0x126 in hexa decimal
    // XPath: iri-to-uri(codepoints-to-string(32 to 294))
    // Expected: %20!%22#$%&'()*+,-./0123456789:;%3C=%3E?
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}

#[test]
fn k2_iritourifunc_10() {
    //  Pass a relatively high range of unicode characters. 0x32 to 0x126 in hexa decim
    // XPath: iri-to-uri(codepoints-to-string(15000 to 16000))
    // Expected: %E3%AA%98%E3%AA%99%E3%AA%9A%E3%AA%9B%E3%
    // Calls stub function - will fail until fn:iri-to-uri is implemented
    let _ = stub_fniri_to_uri();
}
