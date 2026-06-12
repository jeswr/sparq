//! Test chunk 0 for fn:parse-json
//! Contains 16 tests

use dep::xpath::stub_fnparse_json;

#[test]
fn fn_parse_json_038() {
    // Anything allowed at Top level under ECMA spec
    // XPath: parse-json('true')
    // Expected: true
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_039() {
    // Anything allowed at Top level under ECMA spec
    // XPath: parse-json('false')
    // Expected: false
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_053() {
    // Invalid XML char, no fallback
    // XPath: parse-json('\"\uFFFF\"')
    // Expected: codepoints-to-string(65533)
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_054() {
    // Unpaired surrogate, no fallback
    // XPath: parse-json('\"\uDEAD\"')
    // Expected: codepoints-to-string(65533)
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_055() {
    // Character not in XML 1.0, no fallback
    // XPath: parse-json('\"\b\"')
    // Expected: codepoints-to-string(65533)
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_056() {
    // Invalid XML char, no fallback
    // XPath: parse-json('{\"\uFFFF\":\"\"}') => map:keys()
    // Expected: codepoints-to-string(65533)
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_057() {
    // Unpaired surrogate, no fallback
    // XPath: parse-json('{\"\uDEAD\":\"\"}') => map:keys()
    // Expected: codepoints-to-string(65533)
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_058() {
    // Character not in XML 1.0, no fallback
    // XPath: parse-json('{\"\b\":\"\"}') => map:keys()
    // Expected: codepoints-to-string(65533)
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_059() {
    // Invalid XML char, fallback
    // XPath: parse-json('\"\uFFFF\"', map{'fallback':lower-case#1})
    // Expected: "\uffff"
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_060() {
    // Unpaired surrogate, fallback
    // XPath: parse-json('\"\uDEAD\"', map{'fallback':lower-case#1})
    // Expected: "\udead"
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_062() {
    // Invalid XML char, no fallback
    // XPath: parse-json('{\"\uFFFF\":\"\"}', map{'fallback':lower-case#1}
    // Expected: "\uffff"
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_063() {
    // Unpaired surrogate, no fallback
    // XPath: parse-json('{\"\uDEAD\":\"\"}', map{'fallback':lower-case#1}
    // Expected: "\udead"
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_925() {
    // Anything not allowed at Top level under RFC spec
    // XPath: parse-json('true', map{'spec':'RFC4627'})
    // Expected: true
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_926() {
    // Anything not allowed at Top level under RFC spec
    // XPath: parse-json('false', map{'spec':'RFC4627'})
    // Expected: false
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_928() {
    // Anything not allowed at Top level under RFC spec
    // XPath: parse-json('93.7', map{'spec':'RFC4627'})
    // Expected: 93.7
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}

#[test]
fn fn_parse_json_929() {
    // Anything not allowed at Top level under RFC spec
    // XPath: parse-json('\"abcd\n\"', map{'spec':'RFC4627','escape':true(
    // Expected: 'abcd\n'
    // Calls stub function - will fail until fn:parse-json is implemented
    let _ = stub_fnparse_json();
}
