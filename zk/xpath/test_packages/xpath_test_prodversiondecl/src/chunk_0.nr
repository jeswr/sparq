//! Test chunk 0 for prod:VersionDecl
//! Contains 17 tests

use dep::xpath::stub_prodversiondecl;

#[test]
fn k_versionprolog_2() {
    //  A simple version declaration excluding encoding.
    // XPath: xquery version \"1.0\" ; 1 eq 1
    // Expected: true
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn k_versionprolog_2_v3() {
    //  A simple version declaration excluding encoding.
    // XPath: xquery version \"3.0\" ; 1 eq 1
    // Expected: true
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn k_versionprolog_2_v31() {
    //  A simple version declaration excluding encoding.
    // XPath: xquery version \"3.1\" ; 1 eq 1
    // Expected: true
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn prolog_version_4() {
    //  Simple version decl.
    // XPath: xquery version \"1.0\"; 1,2
    // Expected: 1 2
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn prolog_version_4_v3() {
    //  Simple version decl.
    // XPath: xquery version \"3.0\"; 1,2
    // Expected: 1 2
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn prolog_version_4_v31() {
    //  Simple version decl.
    // XPath: xquery version \"3.1\"; 1,2
    // Expected: 1 2
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn prolog_version_1() {
    //  Prolog version declaration with both version and encoding information (set to 1
    // XPath: xquery version \"1.0\" encoding \"utf-8\"; 1,2
    // Expected: 1 2
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn prolog_version_1_v3() {
    //  Prolog version declaration with both version and encoding information (set to 3
    // XPath: xquery version \"3.0\" encoding \"utf-8\"; 1,2
    // Expected: 1 2
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn prolog_version_3() {
    //  Prolog version declaration with both version and encoding information (set to 1
    // XPath: xquery version \"1.0\" encoding \"US-ASCII\"; 1,2
    // Expected: 1 2
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn prolog_version_3_v3() {
    //  Prolog version declaration with both version and encoding information (set to 3
    // XPath: xquery version \"3.0\" encoding \"US-ASCII\"; 1,2
    // Expected: 1 2
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn k_versionprolog_5() {
    //  A prolog containing many different declarations. TODO function declarations mis
    // XPath: xquery version \"1.0\" encoding \"ISO-8859-1\";          dec
    // Expected: true
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn k_versionprolog_5_v3() {
    //  A prolog containing many different declarations. TODO function declarations mis
    // XPath: xquery version \"3.0\" encoding \"ISO-8859-1\";          dec
    // Expected: true
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn version_declaration_023_v3() {
    //  In XQuery 3.0, you *can* omit the 'version' clause.
    // XPath: xquery encoding \"utf-8\"; 1 eq 1
    // Expected: true
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn k2_versionprolog_1() {
    //  a string literal reminding of an xquery version declaration.
    // XPath: 'xquery version \"1.0\" encoding \"UTF-8|#%\";' eq 'xquery v
    // Expected: true
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn k2_versionprolog_2() {
    //  a string literal reminding of an xquery version declaration.
    // XPath: \"xquery version '1.0' encoding 'UTF-8|#%';\" eq \"xquery ve
    // Expected: true
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn k2_versionprolog_3_v4() {
    //  Escape sequences allowed in version literal
    // XPath: xquery version '1&#x2e;0' encoding 'UTF-8'; true()
    // Expected: true
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}

#[test]
fn k2_versionprolog_6() {
    //  Escape sequences allowed in encoding literal
    // XPath: xquery version '1.0' encoding 'UTF&#x2d;8'; true()
    // Expected: true
    // Calls stub function - will fail until prod:VersionDecl is implemented
    let _ = stub_prodversiondecl();
}
