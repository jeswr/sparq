//! Test chunk 0 for prod:VarDefaultValue
//! Contains 14 tests

use dep::xpath::stub_prodvardefaultvalue;

#[test]
fn extvardef_001a() {
    // external variable with a default value
    // XPath: declare variable $ext external := 0; $ext
    // Expected: 5
    // Calls stub function - will fail until prod:VarDefaultValue is implemented
    let _ = stub_prodvardefaultvalue();
}

#[test]
fn extvardef_007() {
    // external variable with a default value that depends on context
    // XPath: declare variable $ext as xs:date external := current-date()
    // Expected: true
    // Calls stub function - will fail until prod:VarDefaultValue is implemented
    let _ = stub_prodvardefaultvalue();
}

#[test]
fn extvardef_010() {
    // The variable values contain the values of all variables present in the static co
    // XPath: declare variable $a := 1;         declare variable $x extern
    // Expected: 3
    // Calls stub function - will fail until prod:VarDefaultValue is implemented
    let _ = stub_prodvardefaultvalue();
}

#[test]
fn extvardef_010a() {
    // Functions contain the values of all variables present in the static context.
    // XPath: declare variable $a external := 6 + local:foo();         dec
    // Expected: 28
    // Calls stub function - will fail until prod:VarDefaultValue is implemented
    let _ = stub_prodvardefaultvalue();
}

#[test]
fn extvardef_012() {
    // The named functions component contains a function for each statically known func
    // XPath: declare function local:aaa() { 1 };         declare variable
    // Expected: 3
    // Calls stub function - will fail until prod:VarDefaultValue is implemented
    let _ = stub_prodvardefaultvalue();
}

#[test]
fn extvardef_013() {
    // The named functions component contains a function for each statically known func
    // XPath: declare variable $y external;         declare variable $z ex
    // Expected: 28
    // Calls stub function - will fail until prod:VarDefaultValue is implemented
    let _ = stub_prodvardefaultvalue();
}

#[test]
fn extvardef_014() {
    // Context item in default value for external variable.
    // XPath: declare variable $x external := /works/employee[@name eq \"J
    // Expected: 1
    // Calls stub function - will fail until prod:VarDefaultValue is implemented
    let _ = stub_prodvardefaultvalue();
}

#[test]
fn extvardef_016a() {
    // Context position in default value for external variable.
    // XPath: declare variable $y := (<a>1</a>,<a>2</a>,<a>3</a>,<a>4</a>,
    // Expected: 1
    // Calls stub function - will fail until prod:VarDefaultValue is implemented
    let _ = stub_prodvardefaultvalue();
}

#[test]
fn extvardef_016b() {
    // Context last in default value for external variable.
    // XPath: declare variable $y := <root><a>1</a>,<a>2</a>,<a>3</a>,<a>4
    // Expected: 1
    // Calls stub function - will fail until prod:VarDefaultValue is implemented
    let _ = stub_prodvardefaultvalue();
}

#[test]
fn extvardef_017() {
    // Implicit cast of external variable default.
    // XPath: declare variable $x as xs:decimal external := xs:integer(10)
    // Expected: 10
    // Calls stub function - will fail until prod:VarDefaultValue is implemented
    let _ = stub_prodvardefaultvalue();
}

#[test]
fn extvardef_019() {
    // External default variable used in external default variable.
    // XPath: declare variable $x external := 10;         declare variable
    // Expected: 28
    // Calls stub function - will fail until prod:VarDefaultValue is implemented
    let _ = stub_prodvardefaultvalue();
}

#[test]
fn extvardef_020() {
    // External default variable used indirectly in external default variable.
    // XPath: declare variable $x external := 5;         declare variable
    // Expected: 28
    // Calls stub function - will fail until prod:VarDefaultValue is implemented
    let _ = stub_prodvardefaultvalue();
}

#[test]
fn extvardef_021() {
    // External default variable used indirectly in external default variable, with imp
    // XPath: declare variable $x as xs:integer external := xs:int(5);
    // Expected: fn:true()
    // Calls stub function - will fail until prod:VarDefaultValue is implemented
    let _ = stub_prodvardefaultvalue();
}

#[test]
fn extvardef_022() {
    // Using external default variable in a function declared before the variable.
    // XPath: declare function local:foo() {           $x         };
    // Expected: 5
    // Calls stub function - will fail until prod:VarDefaultValue is implemented
    let _ = stub_prodvardefaultvalue();
}
