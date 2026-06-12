//! Test chunk 0 for op:same-key
//! Contains 3 tests

use dep::xpath::stub_opsame_key;

#[test]
fn same_key_023() {
    // op:same-key over strings.
    // XPath: let $keys :=            let $range := 48 to 122            f
    // Expected: true
    // Calls stub function - will fail until op:same-key is implemented
    let _ = stub_opsame_key();
}

#[test]
fn same_key_024() {
    // op:same-key over strings and numeric.
    // XPath: let $keys :=            let $range := 48 to 122            f
    // Expected: true
    // Calls stub function - will fail until op:same-key is implemented
    let _ = stub_opsame_key();
}

#[test]
fn same_key_025() {
    // op:same-key over numeric values.
    // XPath: let $keys := (1 to 10000) ! xs:float(1 div .)          let $
    // Expected: true
    // Calls stub function - will fail until op:same-key is implemented
    let _ = stub_opsame_key();
}
