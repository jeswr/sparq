//! Test chunk 0 for prod:ConstructionDecl.schema
//! Contains 2 tests

use dep::xpath::stub_prodconstructiondecl_schema;

#[test]
fn constr_compelem_constrmod_1() {
    //  strip IDREFS type
    // XPath: declare construction strip;          import schema namespace
    // Expected: 1
    // Calls stub function - will fail until prod:ConstructionDecl.schema is implemented
    let _ = stub_prodconstructiondecl_schema();
}

#[test]
fn constr_compelem_constrmod_5() {
    //  strip IDREFS type in attribute
    // XPath: declare construction strip;          import schema namespace
    // Expected: 1
    // Calls stub function - will fail until prod:ConstructionDecl.schema is implemented
    let _ = stub_prodconstructiondecl_schema();
}
