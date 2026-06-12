//! Test chunk 0 for fn:encode-for-uri
//! Contains 25 tests

use dep::xpath::stub_fnencode_for_uri;

#[test]
fn fn_encode_for_uri1args_1() {
    // Test encode-for-uri from example defined in functions and operators specificatio
    // XPath: fn:encode-for-uri (\"http://www.example.com/00/Weather/CA/Lo
    // Expected: http%3A%2F%2Fwww.example.com%2F00%2FWeat
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri1args_2() {
    // Test encode-for-uri from example defined in functions and operators specificatio
    // XPath: encode-for-uri(\"~b?b?\")
    // Expected: ~b%C3%A9b%C3%A9
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri1args_3() {
    // Test encode-for-uri from example defined in functions and operators specificatio
    // XPath: encode-for-uri(\"100% organic\")
    // Expected: 100%25%20organic
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri1args_4() {
    // Test encode-for-uri with zero-length string argument
    // XPath: encode-for-uri('')
    // Expected:
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri1args_5() {
    // Test encode-for-uri with empty sequence argument
    // XPath: encode-for-uri(())
    // Expected:
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_1() {
    //  Evaluation of fn-encode-for-uri function with argument thast ha nothing to enco
    // XPath: (fn:encode-for-uri(\"example\"))
    // Expected: example
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_2() {
    //  Examines that fn-encode-for-uri function does escape the '#' symbol.
    // XPath: (fn:encode-for-uri(\"examples#example\"))
    // Expected: examples%23example
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_3() {
    //  Examines that fn-encode-for-uri function does not escape the '-' symbol.
    // XPath: (fn:encode-for-uri(\"examples-example\"))
    // Expected: examples-example
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_4() {
    //  Examines that fn-encode-for-uri function does not escape the '_' symbol.
    // XPath: (fn:encode-for-uri(\"examples_example\"))
    // Expected: examples_example
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_5() {
    //  Examines that fn-encode-for-uri function does not escape the '.' symbol.
    // XPath: (fn:encode-for-uri(\"examples.example\"))
    // Expected: examples.example
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_6() {
    //  Examines that fn-encode-for-uri function does escapes the '!' symbol.
    // XPath: (fn:encode-for-uri(\"examples!example\"))
    // Expected: examples%21example
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_7() {
    //  Examines that fn-encode-for-uri function does not escape the '~' symbol.
    // XPath: (fn:encode-for-uri(\"examples~example\"))
    // Expected: examples~example
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_8() {
    //  Examines that fn-encode-for-uri function does escape the '*' symbol.
    // XPath: (fn:encode-for-uri(\"examples*example\"))
    // Expected: examples%2Aexample
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_9() {
    //  Examines that fn-encode-for-uri function does escape the ''' symbol.
    // XPath: (fn:encode-for-uri(\"examples'example\"))
    // Expected: examples%27example
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_10() {
    //  Examines that fn-encode-for-uri function does escape the '(' symbol.
    // XPath: (fn:encode-for-uri(\"examples(example\"))
    // Expected: examples%28example
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_11() {
    //  Examines that fn-encode-for-uri function does escape the ')' symbol.
    // XPath: (fn:encode-for-uri(\"examples)example\"))
    // Expected: examples%29example
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_12() {
    //  Examines that fn-encode-for-uri function does not escape numbers.
    // XPath: (fn:encode-for-uri(\"examples0123456789example\"))
    // Expected: examples0123456789example
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_13() {
    //  Examines that fn-encode-for-uri function escapes the space.
    // XPath: (fn:encode-for-uri(\"examples example\"))
    // Expected: examples%20example
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_14() {
    //  Examines that fn-encode-for-uri function escapes the '/' character.
    // XPath: (fn:encode-for-uri(\"examples/example\"))
    // Expected: examples%2Fexample
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_15() {
    //  Examines that fn-encode-for-uri function escapes the ':' character.
    // XPath: (fn:encode-for-uri(\"http:examples\"))
    // Expected: http%3Aexamples
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn fn_encode_for_uri_16() {
    //  Examines that fn-encode-for-uri function escapes the '%' character.
    // XPath: (fn:encode-for-uri(\"http%20examples\"))
    // Expected: http%2520examples
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn k_encodeurifunc_3() {
    //  A test whose essence is: `encode-for-uri(()) eq ''`.
    // XPath: encode-for-uri(()) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn k_encodeurifunc_4() {
    //  Combine fn:concat and fn:encode-for-uri.
    // XPath: concat(\"http://www.example.com/\", encode-for-uri(\"~b?b?\"
    // Expected: true
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn k_encodeurifunc_5() {
    //  Combine fn:concat and fn:encode-for-uri.
    // XPath: concat(\"http://www.example.com/\", encode-for-uri(\"100% or
    // Expected: true
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}

#[test]
fn k_encodeurifunc_6() {
    //  Invoke fn:normalize-space() on the return value of fn:encode-for-uri().
    // XPath: normalize-space(encode-for-uri((\"some string\", current-tim
    // Expected: some%20string
    // Calls stub function - will fail until fn:encode-for-uri is implemented
    let _ = stub_fnencode_for_uri();
}
