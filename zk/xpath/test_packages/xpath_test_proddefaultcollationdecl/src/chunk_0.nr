//! Test chunk 0 for prod:DefaultCollationDecl
//! Contains 2 tests

use dep::xpath::stub_proddefaultcollationdecl;

#[test]
fn k_collationprolog_1() {
    //  A 'declare default collation' that uses a relative URI combined with setting th
    // XPath: declare base-uri \"http://www.w3.org/2005/xpath-functions/\"
    // Expected: true
    // Calls stub function - will fail until prod:DefaultCollationDecl is implemented
    let _ = stub_proddefaultcollationdecl();
}

#[test]
fn k_collationprolog_3() {
    //  Any implementation must support setting the default collation to the Unicode Co
    // XPath: declare default collation \"http://www.w3.org/2005/xpath-fun
    // Expected: true
    // Calls stub function - will fail until prod:DefaultCollationDecl is implemented
    let _ = stub_proddefaultcollationdecl();
}
