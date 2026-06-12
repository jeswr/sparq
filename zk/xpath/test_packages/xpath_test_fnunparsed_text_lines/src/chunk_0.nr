//! Test chunk 0 for fn:unparsed-text-lines
//! Contains 11 tests

use dep::xpath::stub_fnunparsed_text_lines;

#[test]
fn fn_unparsed_text_lines_003() {
    // Attempts to reference the 'unparsed-text-lines' function with arity one.
    // XPath: fn:exists(fn:unparsed-text-lines#1)
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-lines is implemented
    let _ = stub_fnunparsed_text_lines();
}

#[test]
fn fn_unparsed_text_lines_004() {
    // Attempts to reference the 'unparsed-text-lines' function with arity two.
    // XPath: fn:exists(fn:unparsed-text-lines#2)
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-lines is implemented
    let _ = stub_fnunparsed_text_lines();
}

#[test]
fn fn_unparsed_text_lines_027() {
    // Evaluates the 'unparsed-text-lines' function with the argument set as follows: $
    // XPath: fn:unparsed-text-lines(\"text-plain-utf-8.txt\")
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text-lines is implemented
    let _ = stub_fnunparsed_text_lines();
}

#[test]
fn fn_unparsed_text_lines_028() {
    // Evaluates the 'unparsed-text-lines' function with the argument set as follows: $
    // XPath: fn:unparsed-text-lines(\"text-plain-utf-8.txt\", \"utf-8\")
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text-lines is implemented
    let _ = stub_fnunparsed_text_lines();
}

#[test]
fn fn_unparsed_text_lines_031() {
    // Evaluates the 'unparsed-text-lines' function with the argument set as follows: $
    // XPath: fn:unparsed-text-lines(\"\")
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text-lines is implemented
    let _ = stub_fnunparsed_text_lines();
}

#[test]
fn fn_unparsed_text_lines_032() {
    // Evaluates the 'unparsed-text-lines' function with the argument set as follows: $
    // XPath: fn:unparsed-text-lines(\"\", \"utf-8\")
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text-lines is implemented
    let _ = stub_fnunparsed_text_lines();
}

#[test]
fn fn_unparsed_text_lines_041() {
    // Evaluates the 'unparsed-text-lines' function with the argument set as follows: $
    // XPath: fn:unparsed-text-lines(\"http://www.w3.org/fots/unparsed-tex
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text-lines is implemented
    let _ = stub_fnunparsed_text_lines();
}

#[test]
fn fn_unparsed_text_lines_042() {
    // Evaluates the 'unparsed-text-lines' function with the argument set as follows: $
    // XPath: fn:unparsed-text(\"http://www.w3.org/fots/unparsed-text/text
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text-lines is implemented
    let _ = stub_fnunparsed_text_lines();
}

#[test]
fn fn_unparsed_text_lines_043() {
    // Evaluates the 'unparsed-text-lines' function with the argument set as follows: $
    // XPath: fn:unparsed-text(\"http://www.w3.org/fots/unparsed-text/text
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text-lines is implemented
    let _ = stub_fnunparsed_text_lines();
}

#[test]
fn fn_unparsed_text_lines_044() {
    // Evaluates the 'unparsed-text-lines' function with the argument set as follows: $
    // XPath: fn:unparsed-text-lines(\"http://www.w3.org/fots/unparsed-tex
    // Expected: hello?world
    // Calls stub function - will fail until fn:unparsed-text-lines is implemented
    let _ = stub_fnunparsed_text_lines();
}

#[test]
fn fn_unparsed_text_lines_046() {
    // Evaluates the 'unparsed-text-lines' function with the argument set as follows: $
    // XPath: fn:unparsed-text-lines(\"http://www.w3.org/fots/unparsed-tex
    // Expected: <?xml version="1.0" encoding="utf-8"?><t
    // Calls stub function - will fail until fn:unparsed-text-lines is implemented
    let _ = stub_fnunparsed_text_lines();
}
