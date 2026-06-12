//! Test chunk 0 for prod:CurlyArrayConstructor
//! Contains 1 tests

use dep::xpath::stub_prodcurlyarrayconstructor;

#[test]
fn curlyarray_010() {
    // curly array constructor, lookup
    // XPath: declare function local:fun() {1 to 5}; array{local:fun()}?1
    // Expected: 1
    // Calls stub function - will fail until prod:CurlyArrayConstructor is implemented
    let _ = stub_prodcurlyarrayconstructor();
}
