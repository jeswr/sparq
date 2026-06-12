//! Test chunk 0 for prod:NamespaceDecl
//! Contains 7 tests

use dep::xpath::stub_prodnamespacedecl;

#[test]
fn namespacedecl_17() {
    //  Verify that 'abc' is a valid namespace declaration Test was modified on 07/06/0
    // XPath: declare namespace px = \"http://www.example.com/abc\"; let $
    // Expected: true
    // Calls stub function - will fail until prod:NamespaceDecl is implemented
    let _ = stub_prodnamespacedecl();
}

#[test]
fn namespacedecl_20() {
    //  Verify that namespace URI may contain a number. Test was modified on 07/06/06 i
    // XPath: declare namespace xx = \"http://www.example.com/abc123\"; le
    // Expected: true
    // Calls stub function - will fail until prod:NamespaceDecl is implemented
    let _ = stub_prodnamespacedecl();
}

#[test]
fn namespacedecl_21() {
    //  Verify that namespace URI and prefix can contain the same value. Test was modif
    // XPath: declare namespace abc = \"http://www.example.com/abc\"; let
    // Expected: true
    // Calls stub function - will fail until prod:NamespaceDecl is implemented
    let _ = stub_prodnamespacedecl();
}

#[test]
fn namespacedecl_23() {
    //  Verify that a namespace URI conatining 'abc&amp;' is a valid namespace URI duri
    // XPath: declare namespace abc = \"http://www.example.com/abc&amp;\";
    // Expected: true
    // Calls stub function - will fail until prod:NamespaceDecl is implemented
    let _ = stub_prodnamespacedecl();
}

#[test]
fn k_namespaceprolog_1() {
    //  A 'declare namespace' expression containing many comments, using apostrophes fo
    // XPath: (::)declare(::)namespace(::)ncname(::)=(::)'http://example.c
    // Expected: true
    // Calls stub function - will fail until prod:NamespaceDecl is implemented
    let _ = stub_prodnamespacedecl();
}

#[test]
fn k_namespaceprolog_2() {
    //  A 'declare namespace' expression containing many comments, using quotes for the
    // XPath: (::)declare(::)namespace(::)ncname(::)=(::)\"http://example.
    // Expected: true
    // Calls stub function - will fail until prod:NamespaceDecl is implemented
    let _ = stub_prodnamespacedecl();
}

#[test]
fn k2_namespaceprolog_5() {
    //  It is ok to undeclare a non-bound namespace.
    // XPath: declare namespace thisPrefixIsNotBoundExampleCom = \"\"; tru
    // Expected: true
    // Calls stub function - will fail until prod:NamespaceDecl is implemented
    let _ = stub_prodnamespacedecl();
}
