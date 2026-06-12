//! Test chunk 0 for fn:default-language
//! Contains 4 tests

use dep::xpath::stub_fndefault_language;

#[test]
fn default_language_002() {
    // Consistency with format-integer
    // XPath: format-integer(17, \"Ww\") eq format-integer(17, \"Ww\", def
    // Expected: true
    // Calls stub function - will fail until fn:default-language is implemented
    let _ = stub_fndefault_language();
}

#[test]
fn default_language_003() {
    // Consistency with format-date
    // XPath: format-date(current-date(), \"[FNn] [D] [MNn] [Y]\") eq
    // Expected: true
    // Calls stub function - will fail until fn:default-language is implemented
    let _ = stub_fndefault_language();
}

#[test]
fn default_language_004() {
    // Dynamic call
    // XPath: let $f := (fn:default-language#0, fn:default-language#0) ret
    // Expected: true
    // Calls stub function - will fail until fn:default-language is implemented
    let _ = stub_fndefault_language();
}

#[test]
fn default_language_005() {
    // Lookup
    // XPath: let $f := function-lookup(QName(\"http://www.w3.org/2005/xpa
    // Expected: true
    // Calls stub function - will fail until fn:default-language is implemented
    let _ = stub_fndefault_language();
}
