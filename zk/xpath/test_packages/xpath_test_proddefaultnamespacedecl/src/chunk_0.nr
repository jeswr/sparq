//! Test chunk 0 for prod:DefaultNamespaceDecl
//! Contains 18 tests

use dep::xpath::stub_proddefaultnamespacedecl;

#[test]
fn default_namespace_001() {
    //  Demonstrates Default namespace declaration, facilitates the use of unprefixed Q
    // XPath: declare default function namespace \"http://www..oracle.com/
    // Expected: 1
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn default_namespace_005() {
    //  Demonstrates Default namespace declaration, facilitates the use of unprefixed Q
    // XPath: declare default function namespace \"http://example.org\";
    // Expected: 2 3 4 5 6
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn default_namespace_006() {
    //  Demonstrates Default namespace declaration, facilitates the use of unprefixed Q
    // XPath: declare default function namespace \"http://example.org\";
    // Expected: 1 2 3 4 5
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn default_namespace_007() {
    //  Demonstrates Default namespace declaration, facilitates the use of unprefixed Q
    // XPath: declare default function namespace \"http://example.org\";
    // Expected: 101
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn default_namespace_011() {
    //  Demonstrates Default namespace declaration, facilitates the use of unprefixed Q
    // XPath: declare default function namespace \"http://example.org\";
    // Expected: 10
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn default_namespace_012() {
    //  Demonstrates Default namespace declaration, facilitates the use of unprefixed Q
    // XPath: declare default function namespace \"http://example.org\";
    // Expected: 10
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn default_namespace_013() {
    //  Demonstrates Default namespace declaration, facilitates the use of unprefixed Q
    // XPath: declare default function namespace \"http://example.org\";
    // Expected: abc
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn default_namespace_014() {
    //  Demonstrates Default namespace declaration, facilitates the use of unprefixed Q
    // XPath: declare default function namespace \"http://example.org\";
    // Expected: true
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn default_namespace_015() {
    //  Demonstrates Default namespace declaration, facilitates the use of unprefixed Q
    // XPath: declare default function namespace \"http://example.org\";
    // Expected: false
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn default_namespace_020() {
    //  Demonstrates Default namespace declaration, facilitates the use of unprefixed Q
    // XPath: declare default function namespace \"http://example.org\";
    // Expected: 100 1 2 3 15
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn default_namespace_021() {
    //  Demonstrates Default namespace declaration, facilitates the use of unprefixed Q
    // XPath: declare default element namespace \"http:/www.oracle.com/xqu
    // Expected: 9.5
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn k_defaultnamespaceprolog_1() {
    //  A 'declare default element namespace' expression containing many comments, usin
    // XPath: declare(:..:)default(:..:)element(:..:)namespace(:..:)'http:
    // Expected: true
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn k_defaultnamespaceprolog_2() {
    //  A 'declare default element namespace' expression containing many comments, usin
    // XPath: declare(:..:)default(:..:)element(:..:)namespace(:..:)\"http
    // Expected: true
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn k_defaultnamespaceprolog_3() {
    //  A 'declare default function namespace' expression containing many comments, usi
    // XPath: declare(:..:)default(:..:)function(:..:)namespace(:..:)'http
    // Expected: true
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn k_defaultnamespaceprolog_4() {
    //  A 'declare default function namespace' expression containing many comments, usi
    // XPath: declare(:..:)default(:..:)function(:..:)namespace(:..:)\"htt
    // Expected: true
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn k2_defaultnamespaceprolog_1() {
    //  In direct element constructors, the declared default element namespace is respe
    // XPath: declare default element namespace \"http://example.com/defel
    // Expected: http://example.com/defelementns
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn k2_defaultnamespaceprolog_2() {
    //  'xmlns' declarations on direct element constructors override the declared defau
    // XPath: declare default element namespace \"http://example.com/defel
    // Expected: http://example.com/overriden
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}

#[test]
fn k2_defaultnamespaceprolog_17() {
    //  Ensure that the right namespace binding is picked up.
    // XPath: declare default element namespace \"http://example.com/\";
    // Expected: 1
    // Calls stub function - will fail until prod:DefaultNamespaceDecl is implemented
    let _ = stub_proddefaultnamespacedecl();
}
