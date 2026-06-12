//! Test chunk 1 for prod:FunctionDecl
//! Contains 17 tests

use dep::xpath::stub_prodfunctiondecl;

#[test]
fn cbcl_function_declaration_002() {
    //  Test passing a large sequence to a function.
    // XPath: declare function local:count($x) { count($x) }; local:count(
    // Expected: 200000
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_001() {
    // Check that reserved function name attribute is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_003() {
    // Check that reserved function name comment is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_005() {
    // Check that reserved function name document-node is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_007() {
    // Check that reserved function name element is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_009() {
    // Check that reserved function name empty-sequence is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_011() {
    // Check that reserved function name function is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_013() {
    // Check that reserved function name if is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_015() {
    // Check that reserved function name item is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_017() {
    // Check that reserved function name namespace-node is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_019() {
    // Check that reserved function name node is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_021() {
    // Check that reserved function name processing-instruction is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_023() {
    // Check that reserved function name schema-attribute is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_025() {
    // Check that reserved function name schema-element is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_027() {
    // Check that reserved function name switch is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_029() {
    // Check that reserved function name text is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_decl_reserved_function_names_031() {
    // Check that reserved function name typeswitch is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}
