//! Test chunk 1 for prod:WhereClause
//! Contains 1 tests

use dep::xpath::stub_prodwhereclause;

#[test]
fn cbcl_hash_join_012() {
    //  Tests hash join between anyAtomicType containing only untypedAtomic and string
    // XPath: declare variable $strings := ('1','2'); declare variable $un
    // Expected: 1 2
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}
