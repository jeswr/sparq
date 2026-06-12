//! Test chunk 0 for prod:BoundarySpaceDecl
//! Contains 4 tests

use dep::xpath::stub_prodboundaryspacedecl;

#[test]
fn boundary_space_021() {
    //  Demonstrates stripping/preserving of boundary spaces by element constructors du
    // XPath: declare boundary-space strip; (\" \",10, 20, 30, 40,\" \")
    // Expected: 10 20 30 40
    // Calls stub function - will fail until prod:BoundarySpaceDecl is implemented
    let _ = stub_prodboundaryspacedecl();
}

#[test]
fn boundary_space_022() {
    //  Demonstrates stripping/preserving of boundary spaces by element constructors du
    // XPath: declare boundary-space preserve; (\" \",10, 20, 30, 40,\" \"
    // Expected: 10 20 30 40
    // Calls stub function - will fail until prod:BoundarySpaceDecl is implemented
    let _ = stub_prodboundaryspacedecl();
}

#[test]
fn k_boundaryspaceprolog_1() {
    //  A simple 'declare boundary-space' declaration, specifying 'preserve'.
    // XPath: (::)declare(::)boundary-space(::)strip(::); 1 eq 1
    // Expected: true
    // Calls stub function - will fail until prod:BoundarySpaceDecl is implemented
    let _ = stub_prodboundaryspacedecl();
}

#[test]
fn k_boundaryspaceprolog_2() {
    //  A simple 'declare boundary-space' declaration, specifying 'strip'.
    // XPath: (::)declare(::)boundary-space(::)strip(::); 1 eq 1
    // Expected: true
    // Calls stub function - will fail until prod:BoundarySpaceDecl is implemented
    let _ = stub_prodboundaryspacedecl();
}
