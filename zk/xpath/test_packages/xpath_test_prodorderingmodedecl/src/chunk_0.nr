//! Test chunk 0 for prod:OrderingModeDecl
//! Contains 2 tests

use dep::xpath::stub_prodorderingmodedecl;

#[test]
fn k_defaultorderingprolog_1() {
    //  A simple 'declare ordering mode' declaration, specifying 'preserve'.
    // XPath: declare(::)ordering ordered; 1 eq 1
    // Expected: true
    // Calls stub function - will fail until prod:OrderingModeDecl is implemented
    let _ = stub_prodorderingmodedecl();
}

#[test]
fn k_defaultorderingprolog_2() {
    //  A simple 'declare ordering mode' declaration, specifying 'strip'.
    // XPath: declare(::)ordering unordered; 1 eq 1
    // Expected: true
    // Calls stub function - will fail until prod:OrderingModeDecl is implemented
    let _ = stub_prodorderingmodedecl();
}
