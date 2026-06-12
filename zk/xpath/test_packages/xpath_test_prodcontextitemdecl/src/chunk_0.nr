//! Test chunk 0 for prod:ContextItemDecl
//! Contains 19 tests

use dep::xpath::stub_prodcontextitemdecl;

#[test]
fn contextdecl_014() {
    // Forwards reference to context item.
    // XPath: declare variable $x := . + 5;         declare context item :
    // Expected: 22
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_016() {
    // Context position in default value for external variable.
    // XPath: declare variable $y := (<a>1</a>,<a>2</a>,<a>3</a>,<a>4</a>,
    // Expected: 1
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_017() {
    // Context last in default value for external variable.
    // XPath: declare variable $y := <root><a>1</a>,<a>2</a>,<a>3</a>,<a>4
    // Expected: 1
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_028() {
    // Context item bound to an atomic value
    // XPath: declare context item := 3; . + 4
    // Expected: 7
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_029() {
    // Context item bound to a node
    // XPath: declare context item := <a>bananas</a>;         string-lengt
    // Expected: 7
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_030() {
    // Context item bound to a node
    // XPath: declare context item := <a id=\"qwerty\">bananas</a>;
    // Expected: 6
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_031() {
    // Context item bound to a function item
    // XPath: declare context item := contains(?, \"e\");         .(\"rasp
    // Expected: true
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_034() {
    // Context item successfully matches its type
    // XPath: declare context item as xs:integer := (1 to 17)[position() =
    // Expected: 5
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_035() {
    // Context item successfully matches its type
    // XPath: declare context item as xs:integer := (1 to 17)[position() =
    // Expected: 5
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_040() {
    // Require an external context item, no required type
    // XPath: declare context item external;         . instance of documen
    // Expected: true
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_041() {
    // Require an external context item, with required type
    // XPath: declare context item as document-node() external;         na
    // Expected: "works"
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_042() {
    // Require an external context item, with default (no type)
    // XPath: declare context item external := 17;         . = 17
    // Expected: true
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_043() {
    // Require an external context item, with default (with type)
    // XPath: declare context item as xs:integer external := 17;         .
    // Expected: true
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_047() {
    // Context item declaration constraining type appears in library module
    // XPath: import module namespace m=\"http://www.w3.org/TestModules/li
    // Expected: true
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_049() {
    // Context item declaration with type appears in library module
    // XPath: import module namespace m=\"http://www.w3.org/TestModules/li
    // Expected: true
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_055() {
    // Forwards reference to context item involving function lookup
    // XPath: declare variable $f := function-lookup(xs:QName(\"fn:\"||$p)
    // Expected: "e"
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_056() {
    // Two modules constrain the context item type, and the types overlap
    // XPath: import module namespace m=\"http://www.w3.org/TestModules/li
    // Expected: true
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_058() {
    // Two modules constrain the context item type, and the types overlap
    // XPath: import module namespace m=\"http://www.w3.org/TestModules/li
    // Expected: true
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}

#[test]
fn contextdecl_061() {
    //        Global variable bound to context item. Saxon bug 5590.
    // XPath: declare context item := 'x';       declare variable $a := .;
    // Expected: 1
    // Calls stub function - will fail until prod:ContextItemDecl is implemented
    let _ = stub_prodcontextitemdecl();
}
