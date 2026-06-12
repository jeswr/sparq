//! Test chunk 0 for fn:doc-available
//! Contains 7 tests

use dep::xpath::stub_fndoc_available;

#[test]
fn fn_doc_available_5() {
    //  Evaluation of fn:doc-available function using document URI of a known document.
    // XPath: fn:doc-available(document-uri(/))
    // Expected: true
    // Calls stub function - will fail until fn:doc-available is implemented
    let _ = stub_fndoc_available();
}

#[test]
fn fn_doc_available_6() {
    //  Evaluation of fn:doc-available function using a URI that does not exist.
    // XPath: fn:doc-available(\"file:///a/b/c/wefdobqciyvdsoihnfcpinads.x
    // Expected: false
    // Calls stub function - will fail until fn:doc-available is implemented
    let _ = stub_fndoc_available();
}

#[test]
fn fn_doc_available_7() {
    //  Evaluation of fn:doc-available function using an empty sequence.
    // XPath: fn:doc-available(())
    // Expected: false
    // Calls stub function - will fail until fn:doc-available is implemented
    let _ = stub_fndoc_available();
}

#[test]
fn fn_doc_available_8() {
    //  Evaluation of fn:doc-available function using a URI that exists but is not XML.
    // XPath: fn:doc-available(\"../prod/ModuleImport/module1-lib.xq\")
    // Expected: false
    // Calls stub function - will fail until fn:doc-available is implemented
    let _ = stub_fndoc_available();
}

#[test]
fn k2_seqdocavailablefunc_1a() {
    //  ':/' is an invalid URI, no scheme. Under erratum FO.E26, this may either throw
    // XPath: doc-available(':/')
    // Expected: false
    // Calls stub function - will fail until fn:doc-available is implemented
    let _ = stub_fndoc_available();
}

#[test]
fn cbcl_doc_available_001() {
    //  test fn:doc-available on ()
    // XPath: fn:doc-available( () )
    // Expected: false
    // Calls stub function - will fail until fn:doc-available is implemented
    let _ = stub_fndoc_available();
}

#[test]
fn cbcl_doc_available_003() {
    //  test fn:doc-available on a URI which is not that of a document
    // XPath: fn:doc-available( 'collection1' )
    // Expected: false
    // Calls stub function - will fail until fn:doc-available is implemented
    let _ = stub_fndoc_available();
}
