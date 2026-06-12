//! Test chunk 0 for fn:unparsed-text
//! Contains 18 tests

use dep::xpath::stub_fnunparsed_text;

#[test]
fn fn_unparsed_text_003() {
    // Attempts to reference the 'unparsed-text' function with arity one.
    // XPath: fn:exists( fn:unparsed-text#1 )
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_004() {
    // Attempts to reference the 'unparsed-text' function with arity two.
    // XPath: fn:exists( fn:unparsed-text#2 )
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_027() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href i
    // XPath: fn:unparsed-text(\"text-plain-utf-8.txt\")
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_028() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href i
    // XPath: fn:unparsed-text(\"text-plain-utf-8.txt\", \"utf-8\")
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_031() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href i
    // XPath: fn:unparsed-text(\"\")
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_032() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href i
    // XPath: fn:unparsed-text(\"\", \"utf-8\")
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_040() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href r
    // XPath: fn:unparsed-text(\"http://www.w3.org/fots/unparsed-text/inva
    // Expected: <?xml version="1.0" encoding="utf-16"?><
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_041() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href r
    // XPath: fn:unparsed-text(\"http://www.w3.org/fots/unparsed-text/text
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_042() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href r
    // XPath: fn:unparsed-text(\"http://www.w3.org/fots/unparsed-text/text
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_043() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href r
    // XPath: fn:unparsed-text(\"http://www.w3.org/fots/unparsed-text/text
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_044() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href r
    // XPath: fn:unparsed-text(\"http://www.w3.org/fots/unparsed-text/text
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_046() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href r
    // XPath: fn:unparsed-text(\"http://www.w3.org/fots/unparsed-text/text
    // Expected: <?xml version="1.0" encoding="utf-8"?><t
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_047() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href r
    // XPath: fn:unparsed-text(\"http://www.w3.org/fots/unparsed-text/text
    // Expected: <?xml version="1.0" encoding="utf-16"?><
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_049() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href r
    // XPath: string-length(fn:unparsed-text(\"http://www.w3.org/fots/unpa
    // Expected: 400
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_050() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href r
    // XPath: string-length(fn:unparsed-text(\"http://www.w3.org/fots/unpa
    // Expected: 400
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_051() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href r
    // XPath: string-length(fn:unparsed-text(\"http://www.w3.org/fots/unpa
    // Expected: 400
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_052() {
    // Evaluates the 'unparsed-text' function with the argument set as follows: $href r
    // XPath: string-length(fn:unparsed-text(\"http://www.w3.org/fots/unpa
    // Expected: 400
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}

#[test]
fn fn_unparsed_text_055() {
    // Unparsed text reading a file with a surrogate pair at an awkward position
    // XPath: string-length(unparsed-text('http://www.w3.org/fots/unparsed
    // Expected: 2048
    // Calls stub function - will fail until fn:unparsed-text is implemented
    let _ = stub_fnunparsed_text();
}
