//! Test chunk 0 for prod:OptionDecl.serialization
//! Contains 1 tests

use dep::xpath::stub_prodoptiondecl_serialization;

#[test]
fn serialization_027a() {
    //           This test is a variant of Serialization-27. Here we assume the seriali
    // XPath: declare namespace output = \"http://www.w3.org/2010/xslt-xqu
    // Expected: banana
    // Calls stub function - will fail until prod:OptionDecl.serialization is implemented
    let _ = stub_prodoptiondecl_serialization();
}
