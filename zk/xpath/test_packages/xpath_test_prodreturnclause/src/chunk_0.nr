//! Test chunk 0 for prod:ReturnClause
//! Contains 6 tests

use dep::xpath::stub_prodreturnclause;

#[test]
fn returnexpr004() {
    //  For+Where+Return - filters in 'Where' expr and in 'Return' expr using predicate
    // XPath: for $f in /MyComputer//File where $f/@creation_date=\"08/06/
    // Expected: system\Toshiko
    // Calls stub function - will fail until prod:ReturnClause is implemented
    let _ = stub_prodreturnclause();
}

#[test]
fn returnexpr006() {
    //  FLWOR expression returns selected values
    // XPath: for $file in (//Folder)[1]/File return data( $file/FileName
    // Expected: File00000000000 File00000000001 File0000
    // Calls stub function - will fail until prod:ReturnClause is implemented
    let _ = stub_prodreturnclause();
}

#[test]
fn returnexpr008() {
    //  FLWOR expression returns constant value, independent of input bindings
    // XPath: for $file in (//Folder)[1]/File return 1
    // Expected: 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1 1
    // Calls stub function - will fail until prod:ReturnClause is implemented
    let _ = stub_prodreturnclause();
}

#[test]
fn returnexpr017() {
    //  Apply arithmetic operator inside 'return' statement
    // XPath: for $file in (//Folder)[1]/File return ($file/Stream/StreamS
    // Expected: 1002.33 1003.33 1004.33 1005.33 1006.33
    // Calls stub function - will fail until prod:ReturnClause is implemented
    let _ = stub_prodreturnclause();
}

#[test]
fn returnexpr018() {
    //  Apply comparison operator inside 'return' statement
    // XPath: for $file in (//Folder)[1]/File return ($file/Stream/StreamS
    // Expected: false false false true true true true tr
    // Calls stub function - will fail until prod:ReturnClause is implemented
    let _ = stub_prodreturnclause();
}

#[test]
fn returnexpr020() {
    //  'return' expression containing a typed value constructor function
    // XPath: for $file in (//Folder)[1]/File return xs:string( data( $fil
    // Expected: File00000000000 File00000000001 File0000
    // Calls stub function - will fail until prod:ReturnClause is implemented
    let _ = stub_prodreturnclause();
}
