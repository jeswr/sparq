//! Test chunk 1 for fn:string
//! Contains 3 tests

use dep::xpath::stub_fnstring;

#[test]
fn k_stringfunc_3() {
    // K-StringFunc-3 `string(1) eq '1'`.
    // XPath: string(1) eq \"1\"
    // Expected: true
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn k_stringfunc_4() {
    // K-StringFunc-4 `string(()) eq ''`.
    // XPath: string(()) eq \"\"
    // Expected: true
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}

#[test]
fn k_stringfunc_5() {
    // K-StringFunc-5 xs:string.
    // XPath: string-length(string(xs:string(current-time()))) gt 2
    // Expected: true
    // Calls stub function - will fail until fn:string is implemented
    let _ = stub_fnstring();
}
