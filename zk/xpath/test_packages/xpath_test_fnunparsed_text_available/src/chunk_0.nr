//! Test chunk 0 for fn:unparsed-text-available
//! Contains 42 tests

use dep::xpath::stub_fnunparsed_text_available;

#[test]
fn fn_unparsed_text_available_003() {
    // Attempts to reference the 'unparsed-text-available' function with arity one.
    // XPath: fn:exists( fn:unparsed-text-available#1 )
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_004() {
    // Attempts to reference the 'unparsed-text-available' function with arity two.
    // XPath: fn:exists( fn:unparsed-text-available#2 )
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_013() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.example.org/#fragmen
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_014() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.example.org/#fragmen
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_015() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.example.org/%gg\")
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_016() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.example.org/%gg\", \
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_017() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\":/\")
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_018() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\":/\", \"utf-8\")
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_019() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available( \"http://www.w3.org/fots/unparse
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_020() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available( \"http://www.w3.org/fots/unparse
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_021() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"surely-nobody-supports-this:/pa
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_022() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"surely-nobody-supports-this:/pa
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_023() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"unparsed-text/text-plain-utf-8.
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_024() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"unparsed-text/text-plain-utf-8.
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_025() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"does-not-exist.txt\")
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_026() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"does-not-exist.txt\", \"utf-8\"
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_027() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"text-plain-utf-8.txt\")
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_028() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"text-plain-utf-8.txt\", \"utf-8
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_029() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"C:\file-might-exist.txt\")
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_030() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"C:\file-might-exist.txt\", \"ut
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_031() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"\")
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_032() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"\", \"utf-8\")
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_033() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"\")
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_034() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"\", \"utf-8\")
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_035() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_036() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_037() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_038() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_039() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_040() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_041() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_042() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_043() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_045() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_046() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_048() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_049() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_050() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_051() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available(\"http://www.w3.org/fots/unparsed
    // Expected: true
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_052() {
    // Evaluates the 'unparsed-text-available' function with the argument set as follow
    // XPath: fn:unparsed-text-available($href)
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_053() {
    // unparsed-text-available(()) returns false
    // XPath: fn:unparsed-text-available(\"x\"[current-date() lt xs:date('
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}

#[test]
fn fn_unparsed_text_available_054() {
    // unparsed-text-available((), 'utf-8') returns false
    // XPath: fn:unparsed-text-available(\"x\"[current-date() lt xs:date('
    // Expected: false
    // Calls stub function - will fail until fn:unparsed-text-available is implemented
    let _ = stub_fnunparsed_text_available();
}
