//! Test chunk 0 for prod:CompNamespaceConstructor
//! Contains 1 tests

use dep::xpath::stub_prodcompnamespaceconstructor;

#[test]
fn nscons_028() {
    // nscons-028 - dynamic namespace constructor - node equality and modules
    // XPath: import module namespace mod1=\"http://www.w3.org/TestModules
    // Expected: true false
    // Calls stub function - will fail until prod:CompNamespaceConstructor is implemented
    let _ = stub_prodcompnamespaceconstructor();
}
