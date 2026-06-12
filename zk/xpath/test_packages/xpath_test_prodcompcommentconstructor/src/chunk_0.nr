//! Test chunk 0 for prod:CompCommentConstructor
//! Contains 4 tests

use dep::xpath::stub_prodcompcommentconstructor;

#[test]
fn constr_compcomment_parent_1() {
    //  empty parent
    // XPath: count((comment {'comment'})/..)
    // Expected: 0
    // Calls stub function - will fail until prod:CompCommentConstructor is implemented
    let _ = stub_prodcompcommentconstructor();
}

#[test]
fn constr_compcomment_string_1() {
    //  string value of computed comment node
    // XPath: fn:string(comment {'a', element a {}, 'b'})
    // Expected: a  b
    // Calls stub function - will fail until prod:CompCommentConstructor is implemented
    let _ = stub_prodcompcommentconstructor();
}

#[test]
fn constr_compcomment_data_1() {
    //  typed value of computed comment node
    // XPath: fn:data(comment {'a', element a {}, 'b'})
    // Expected: a  b
    // Calls stub function - will fail until prod:CompCommentConstructor is implemented
    let _ = stub_prodcompcommentconstructor();
}

#[test]
fn k2_computeconcomment_5() {
    //  Test the atomized type.
    // XPath: data(comment {\"content\"}) instance of xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CompCommentConstructor is implemented
    let _ = stub_prodcompcommentconstructor();
}
