//! Test chunk 0 for prod:OptionDecl
//! Contains 4 tests

use dep::xpath::stub_prodoptiondecl;

#[test]
fn optiondeclprolog_1() {
    //  Evaluation of a simple prolog option declaration.
    // XPath: declare namespace exq = \"http://example.org/XQueryImplement
    // Expected: aaa
    // Calls stub function - will fail until prod:OptionDecl is implemented
    let _ = stub_prodoptiondecl();
}

#[test]
fn k_optiondeclarationprolog_2() {
    //  Five identical options appearing after each other.
    // XPath: declare(::)option(::)local:opt(::)\"option value\"(::);
    // Expected: true
    // Calls stub function - will fail until prod:OptionDecl is implemented
    let _ = stub_prodoptiondecl();
}

#[test]
fn k_optiondeclarationprolog_3() {
    //  A simple option using double quotes for the value.
    // XPath: declare(::)option(::)local:opt\"option value\"; 1 eq 1
    // Expected: true
    // Calls stub function - will fail until prod:OptionDecl is implemented
    let _ = stub_prodoptiondecl();
}

#[test]
fn k_optiondeclarationprolog_4() {
    //  A simple option using single quotes for the value.
    // XPath: declare(::)option(::)local:opt'option value'; 1 eq 1
    // Expected: true
    // Calls stub function - will fail until prod:OptionDecl is implemented
    let _ = stub_prodoptiondecl();
}
