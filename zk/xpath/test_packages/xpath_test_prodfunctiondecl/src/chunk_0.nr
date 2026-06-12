//! Test chunk 0 for prod:FunctionDecl
//! Contains 50 tests

use dep::xpath::stub_prodfunctiondecl;

#[test]
fn function_declaration_001() {
    //  Demonstrate function declaration in different combination
    // XPath: declare namespace foo = \"http://www..oracle.com/xquery/test
    // Expected: 1
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_declaration_005() {
    //  Demonstrate function declaration in different combination
    // XPath: declare function local:mysum($i as xs:integer, $j as xs:inte
    // Expected: 2 3 4 5 6
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_declaration_006() {
    //  Demonstrate function declaration in different combination
    // XPath: declare function local:mysum($i as xs:integer, $j as xs:inte
    // Expected: 1 2 3 4 5
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_declaration_007() {
    //  Demonstrate function declaration in different combination
    // XPath: declare variable $a := 1;         declare function local:foo
    // Expected: 101
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_declaration_011() {
    //  Demonstrate function declaration - forward declaration
    // XPath: declare function local:fn1 ($n as xs:integer) as xs:integer
    // Expected: 10
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_declaration_012() {
    //  Demonstrate function declaration in different combination
    // XPath: declare function local:fn1 ($n as xs:integer) as xs:integer
    // Expected: 10
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_declaration_013() {
    //  Demonstrate function declaration in different combination
    // XPath: declare function local:foo2($i as xs:string) as xs:string {l
    // Expected: abc
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_declaration_014() {
    //  Demonstrate function declaration in different combination
    // XPath: declare function local:odd($x as xs:integer) as xs:boolean {
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_declaration_015() {
    //  Demonstrate function declaration in different combination
    // XPath: declare function local:odd($x as xs:integer) as xs:boolean {
    // Expected: false
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_declaration_020() {
    //  Demonstrate function declaration - overloading
    // XPath: declare namespace foo = \"http://www..oracle.com/xquery/test
    // Expected: 100 1 2 3 15
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn function_declaration_026() {
    //  Function Declaration using global variable. A global node test, which uses the
    // XPath: declare variable $global := node();         declare function
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k_functionprolog_1() {
    //  The 'XPath Data Types' namespace is not reserved anymore, although it was in ol
    // XPath: declare namespace test = \"http://www.w3.org/2005/xpath-data
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k_functionprolog_12b() {
    //  XQuery 3.0: Variable appearing after a function declaration is in scope inside
    // XPath: declare function local:computeSum() { $myVariable };
    // Expected: 1
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k_functionprolog_18() {
    //  Two user functions using global variables.
    // XPath: declare variable $var1 := 1;         declare function local:
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k_functionprolog_19() {
    //  A user function where all its arguments are unused.
    // XPath: declare function local:func1($a1, $a2, $a3) { 1 };         l
    // Expected: 1
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k_functionprolog_20() {
    //  User functions which in some implementations causes constant propagation combin
    // XPath: declare function local:func($choose, $whenTrue, $whenFalse)
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k_functionprolog_21() {
    //  User functions where the middle argument of three arguments is unused.
    // XPath: declare function local:func($a as xs:integer, $unused, $c as
    // Expected: 4
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k_functionprolog_22() {
    //  User functions where the first argument of three arguments is unused.
    // XPath: declare function local:func($unused, $b as xs:integer, $c as
    // Expected: 5
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k_functionprolog_23() {
    //  User functions where the last argument of three arguments is unused.
    // XPath: declare function local:func($a as xs:integer, $b as xs:integ
    // Expected: 3
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k_functionprolog_25() {
    //  A function and a variable can have the same name.
    // XPath: declare variable $local:myName := 1;         declare functio
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k_functionprolog_30() {
    //  Overloading user functions based on arity.
    // XPath: declare function local:myName($var as xs:integer) as xs:inte
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k_functionprolog_39() {
    //  Function arguments shadows global variables.
    // XPath: declare variable $local:myVar := 1;         declare function
    // Expected: 1 2
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k_functionprolog_40() {
    //  Variables declared inside functions shadow function arguments.
    // XPath: declare function local:myFunction($local:myVar as xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k_functionprolog_62() {
    //  Check that a global variable is in scope despite an unused function being decla
    // XPath: declare variable $my := 3;         declare function local:my
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k_functionprolog_67() {
    //  A function call that reminds of the range expression.
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_1() {
    //  Have two function callsites as arguments to 'eq'.
    // XPath: declare function local:myFunction($arg as xs:integer) as xs:
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_2() {
    //  Have two function callsites as arguments to 'ne'.
    // XPath: declare function local:myFunction($arg as xs:integer) as xs:
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_3() {
    //  Have two function callsites as arguments to '='.
    // XPath: declare function local:myFunction($arg as xs:integer) as xs:
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_4() {
    //  Have two function callsites as arguments to '!='.
    // XPath: declare function local:myFunction($arg as xs:integer) as xs:
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_7() {
    //  A declared return value invokes numeric promotion.
    // XPath: declare function local:myFunction() as xs:float { 4.0 };
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_8() {
    //  A recursive function stretching through several function calls.
    // XPath: declare function local:myFunction($arg as xs:integer) as xs:
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_9() {
    //  A recursive function multiplying a sequence.
    // XPath: declare function local:multiplySequence($input as xs:integer
    // Expected: 120
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_10() {
    //  A function whose name is only '_' and is declared in the default namespace.
    // XPath: declare default function namespace \"http://example.com\";
    // Expected: 1
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_11() {
    //  A function whose name is only '_'.
    // XPath: declare function local:_() { 1 };         local:_()
    // Expected: 1
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_15() {
    //  Use a range variable inside the assignment expression of a global variable.
    // XPath: declare variable $var1 := let $var1 := 1 return 1;         $
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_17() {
    //  Conversions are invoked when an element is returned from a function with declar
    // XPath: declare function local:func() as xs:integer { <e>1</e> };
    // Expected: 1
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_18() {
    //  Conversions are invoked when an attribute is returned from a function with decl
    // XPath: declare function local:func() as xs:integer { attribute name
    // Expected: 1
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_19() {
    //  Conversions are invoked when a document is returned from a function with declar
    // XPath: declare function local:func() as xs:integer { document {\"1\
    // Expected: 1
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_22() {
    //  Conversions are invoked when a text node is returned from a function with decla
    // XPath: declare function local:func() as xs:integer? { text {\"1\"}
    // Expected: 1
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_23() {
    //  Call a function that subsequently calls a recursive function.
    // XPath: declare function local:recursiveFunction($i as xs:integer) a
    // Expected: 8
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_28() {
    //  Compute the levenshtein distance between strings.
    // XPath: declare function local:levenshtein($arg1 as xs:string, $arg2
    // Expected: 0 0 0 0 2 2 1 3
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_31() {
    //  Pass a sequence of mixed atomic values and nodes to an argument which has no ty
    // XPath: declare function local:foo($arg) as xs:boolean* { $arg };
    // Expected: true true false false true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_33() {
    //  Mix in an atomic value which matches the type it's being converted to. $arg doe
    // XPath: declare function local:foo($arg ) as xs:boolean* { $arg };
    // Expected: true true false false true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_34() {
    //  Call fn:true() and fn:false() and pass to a user function.
    // XPath: declare function local:foo($arg) as xs:boolean* { $arg };
    // Expected: true false false false
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_35() {
    //  Pass an untyped value.
    // XPath: declare function local:foo($arg) as xs:boolean* { $arg };
    // Expected: false
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_36() {
    //  Pass an untyped value(different cardinality).
    // XPath: declare function local:foo($arg) as xs:boolean { $arg };
    // Expected: false
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_37() {
    //  Mix in an atomic value which matches the type it's being converted to. $arg doe
    // XPath: declare function local:foo($arg ) as xs:boolean* { $arg };
    // Expected: true true false false true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_39() {
    //  Use complex real-world case for function calls, with automatic conversion not n
    // XPath: declare variable $A:=(<A>{local:functionA()}</A>);         d
    // Expected: testing ...
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn k2_functionprolog_40() {
    //  Use complex real-world case for function calls, with automatic conversion.
    // XPath: declare variable $A:=(<A>{local:functionA()}</A>);         d
    // Expected: testing ...
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}

#[test]
fn cbcl_function_decl_001() {
    //  check that XPST0005 is NOT a valid response
    // XPath: declare function local:nothing() as empty-sequence() { () };
    // Expected: true
    // Calls stub function - will fail until prod:FunctionDecl is implemented
    let _ = stub_prodfunctiondecl();
}
