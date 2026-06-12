//! Test chunk 0 for fn:error
//! Contains 4 tests

use dep::xpath::stub_fnerror;

#[test]
fn k_errorfunc_1() {
    //  A test whose essence is: `if(true()) then true() else error()`.
    // XPath: if(true()) then true() else error()
    // Expected: true
    // Calls stub function - will fail until fn:error is implemented
    let _ = stub_fnerror();
}

#[test]
fn k_errorfunc_2() {
    //  A test whose essence is: `if(true()) then true() else error(QName('', 'local'),
    // XPath: if(true()) then true() else error(QName(\"\", \"local\"), \"
    // Expected: true
    // Calls stub function - will fail until fn:error is implemented
    let _ = stub_fnerror();
}

#[test]
fn k_errorfunc_7() {
    //  A test whose essence is: `if(false()) then error((), 'description') else true()
    // XPath: if(false()) then error((), \"description\") else true()
    // Expected: true
    // Calls stub function - will fail until fn:error is implemented
    let _ = stub_fnerror();
}

#[test]
fn cbcl_error_029() {
    //  fn:error() might never be evaluated here the static type of the typeswitch expr
    // XPath: declare function local:item() { if (current-date() gt xs:dat
    // Expected: true
    // Calls stub function - will fail until fn:error is implemented
    let _ = stub_fnerror();
}
