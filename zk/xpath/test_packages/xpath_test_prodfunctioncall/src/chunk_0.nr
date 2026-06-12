//! Test chunk 0 for prod:FunctionCall
//! Contains 24 tests

use dep::xpath::stub_prodfunctioncall;

#[test]
fn functioncall_001() {
    // Test that arguments are atomized - built in function
    // XPath: concat(<a>X</a>, <a>Y</a>)
    // Expected: XY
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn functioncall_002() {
    // Test that arguments are atomized - constructor function
    // XPath: xs:boolean(<a>0</a>)
    // Expected: false
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn functioncall_004() {
    // Test that arguments are atomized - user-defined function
    // XPath: declare function local:f($in as xs:boolean) as xs:boolean {
    // Expected: false
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn functioncall_006() {
    // Test that arguments are atomized - anonymous inline function
    // XPath: let $f := function ($in as xs:boolean) as xs:boolean { $in }
    // Expected: false
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn functioncall_023() {
    // Test that a sequence of arguments is atomized
    // XPath: string-join((<a>X</a>, <a>Y</a>, <a>Z</a>), '')
    // Expected: XYZ
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn k_functioncallexpr_15a() {
    //  No function by name fn:format-number() exists in XQuery 1.0 (although one does
    // XPath: format-number(3, \"0000\")
    // Expected: "0003"
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn k_functioncallexpr_16a() {
    //  No function by name fn:format-time() exists in XQuery 1.0 (although one does in
    // XPath: matches(format-time(current-time(), \"[H01]:[m01]\"), \"[0-2
    // Expected: true
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn k_functioncallexpr_17a() {
    //  No function by name fn:format-time() exists in XQuery 1.0 (although one does in
    // XPath: matches(format-time(current-time(), \"[H01]:[m01]\", (), (),
    // Expected: true
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn k_functioncallexpr_25a() {
    //  Function by name fn:generate-id() does exist in XQuery 3.0.
    // XPath: generate-id(<a/>) castable as xs:NCName
    // Expected: true
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn k2_functioncallexpr_1() {
    //  Two user functions with many arguments.
    // XPath: declare function local:bar($c, $d, $e, $f, $g, $h, $i, $j, $
    // Expected: 1
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn k2_functioncallexpr_10() {
    //  Check that nodes, when passed through function arguments, have proper node iden
    // XPath: declare variable $a := <a/>;          declare function local
    // Expected: true false true
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn k2_functioncallexpr_11() {
    //  A very simple string-difference function.
    // XPath: declare function local:compare($arg1 as xs:string, $arg2 as
    // Expected: 0 1 1 0 2 1 0 1 3
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn k2_functioncallexpr_12() {
    //  Trigger an infinite recursion in one implementation.
    // XPath: declare function local:func1() { if(local:func2('b')) then 3
    // Expected: 3
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn k2_functioncallexpr_13() {
    //  Have a function which recurses infintely, but which never is called.
    // XPath: declare function local:foo($arg) { local:foo(local:foo(1)) }
    // Expected: 1
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn cbcl_promotion_001() {
    //  Ensure sequence types are promoted correctly
    // XPath: string-join( (xs:anyURI('http://www.microsoft.com'), xs:anyU
    // Expected: http://www.microsoft.com http://www.goog
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn cbcl_promotion_002() {
    //  Test that type promotion occurs correctly for values passed to user-defined fun
    // XPath: declare function local:index-of($seq as xs:double*, $item as
    // Expected: 2
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn cbcl_promotion_003() {
    //  Test that type promotion occurs correctly for values returned from user-defined
    // XPath: declare function local:index-of($seq as xs:integer*, $item a
    // Expected: 3
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn cbcl_promotion_004() {
    //  Test that type promotion occurs correctly for sequence values returned from use
    // XPath: declare function local:f() as xs:double* {            if (da
    // Expected: 4
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn cbcl_promotion_007() {
    //  Check that correct type constraint is inserted.
    // XPath: declare function local:f($v as xs:double*) as xs:double+ { i
    // Expected: 1 2 3
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn function_call_reserved_function_names_014() {
    // Check that reserved function name function is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn function_call_reserved_function_names_015() {
    // Check that reserved function name namespace-node is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn function_call_reserved_function_names_016() {
    // Check that reserved function name switch is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn function_call_reserved_function_names_039() {
    // Check that reserved function name array is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}

#[test]
fn function_call_reserved_function_names_043() {
    // Check that reserved function name map is handled correctly.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionCall is implemented
    let _ = stub_prodfunctioncall();
}
