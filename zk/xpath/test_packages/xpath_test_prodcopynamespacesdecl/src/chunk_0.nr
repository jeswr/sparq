//! Test chunk 0 for prod:CopyNamespacesDecl
//! Contains 23 tests

use dep::xpath::stub_prodcopynamespacesdecl;

#[test]
fn copynamespace_4() {
    //  Evaluates copy namespace declaration with value set to 'preserve inherit'.
    // XPath: declare namespace foo = \"http://example.org\";         decl
    // Expected: foo xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_5() {
    //  Evaluates copy namespace declaration with value set to 'preserve no-inherit' .
    // XPath: declare namespace foo = \"http://example.org\";         decl
    // Expected: foo xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_6() {
    //  Evaluates copy namespace declaration with value set to 'preserve inherit' . The
    // XPath: declare namespace foo = \"http://example.org\";         decl
    // Expected: foo xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_7() {
    //  Evaluates copy namespace declaration with value set to 'no-preserve no-inherit'
    // XPath: declare copy-namespaces no-preserve,no-inherit;         decl
    // Expected: xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_8() {
    //  Evaluates copy namespace declaration with value set to 'preserve no-inherit' .
    // XPath: declare copy-namespaces preserve,no-inherit;         declare
    // Expected: existingNamespace xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_9() {
    //  Evaluates copy namespace declaration with value set to 'no-preserve inherit' .
    // XPath: declare copy-namespaces no-preserve,inherit;         declare
    // Expected: newNamespace xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_10() {
    //  Evaluates copy namespace declaration with value set to 'preserve inherit' . Use
    // XPath: declare copy-namespaces preserve,inherit;         declare va
    // Expected: existingNamespace newNamespace xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_11() {
    //  Evaluates copy namespace declaration with value set to 'preserve inherit' . Use
    // XPath: declare copy-namespaces preserve,inherit;         declare va
    // Expected: existingNamespace xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_12() {
    //  Evaluates copy namespace declaration with value set to 'preserve inherit' . Use
    // XPath: declare copy-namespaces preserve,inherit;         declare va
    // Expected: existingNamespace newNamespace xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_13() {
    //  Evaluates copy namespace declaration with value set to 'preserve inherit' . Use
    // XPath: declare copy-namespaces preserve,inherit;         declare va
    // Expected: xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_14() {
    //  Evaluates copy namespace declaration with value set to 'preserve inherit' . Use
    // XPath: declare copy-namespaces preserve,inherit;         declare va
    // Expected: SOMESPACE somespace xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_15() {
    //  Evaluates copy namespace declaration with value set to 'preserve inherit' . wit
    // XPath: declare copy-namespaces preserve,inherit;         declare va
    // Expected: namespace1 namespace2 namespace3 xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_16() {
    //  Evaluates copy namespace declaration with value set to 'no-preserve inherit' .
    // XPath: declare copy-namespaces no-preserve,inherit;         declare
    // Expected: namespace3 xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_17() {
    //  Evaluates copy namespace declaration with value set to 'no-preserve no-inherit'
    // XPath: declare copy-namespaces no-preserve,no-inherit;         decl
    // Expected: xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_18() {
    //  Evaluates copy namespace declaration with value set to 'preserve no-inherit' .
    // XPath: declare copy-namespaces preserve,no-inherit;         declare
    // Expected: namespace1 xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_19() {
    //  Evaluates copy namespace declaration with value set to 'preserve inherit' . wit
    // XPath: declare copy-namespaces preserve,inherit;         declare va
    // Expected: namespace2 namespace3 xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_20() {
    //  Evaluates copy namespace declaration with value set to 'no-preserve inherit' .
    // XPath: declare copy-namespaces no-preserve,inherit;         declare
    // Expected: namespace3 xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_21() {
    //  Evaluates copy namespace declaration with value set to 'no-preserve no-inherit'
    // XPath: declare copy-namespaces no-preserve,no-inherit;         decl
    // Expected: xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn copynamespace_22() {
    //  Evaluates copy namespace declaration with value set to 'preserve no-inherit' .
    // XPath: declare copy-namespaces preserve,no-inherit;         declare
    // Expected: namespace2 xml
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn k_copynamespacesprolog_2() {
    //  A 'declare copy-namespaces' declaration specifying no-preserve and no-inherit.
    // XPath: declare copy-namespaces no-preserve, no-inherit; 1 eq 1
    // Expected: true
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn k_copynamespacesprolog_3() {
    //  A 'declare copy-namespaces' declaration specifying no-preserve and no-inherit.
    // XPath: declare copy-namespaces preserve, inherit; 1 eq 1
    // Expected: true
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn k2_copynamespacesprolog_6() {
    //  Check in-scope bindings of constructed nodes.
    // XPath: declare namespace ns = \"http://example.com/\"; <e xmlns:app
    // Expected: appearsUnused | appearsUnused | appearsU
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}

#[test]
fn k2_copynamespacesprolog_9() {
    //  Use direct element constructors inside enclosed expressions.
    // XPath: declare copy-namespaces preserve, no-inherit;           decl
    // Expected: namespace1 xml | namespace1 namespace2 n
    // Calls stub function - will fail until prod:CopyNamespacesDecl is implemented
    let _ = stub_prodcopynamespacesdecl();
}
