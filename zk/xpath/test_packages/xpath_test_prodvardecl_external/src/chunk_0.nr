//! Test chunk 0 for prod:VarDecl.external
//! Contains 30 tests

use dep::xpath::stub_prodvardecl_external;

#[test]
fn extvardeclwithouttype_24() {
    //  Evaluates an external variable reference without type, variable name includes n
    // XPath: declare namespace test = \"http://www.w3.org/fots/example-te
    // Expected: 2
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn k2_externalvariableswithout_8() {
    //  It is ok to have space between '$' and the name in variable names.
    // XPath: declare variable $ name := 3; $ name
    // Expected: 3
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn k2_externalvariableswithout_9() {
    //  Ensure a variable declaration doesn't violate stability rules.
    // XPath: declare variable $e := current-time(); let $i := ($e, 1 to 5
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn k2_externalvariableswithout_11() {
    //  Description ensure node identity is handled through variables that has cardinal
    // XPath: declare variable $a as attribute()* := (attribute name1 {()}
    // Expected: false false false
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn k2_externalvariableswithout_13() {
    //  Evaluate the boolean value of a variable.
    // XPath: declare variable $e := <e>{current-time()}</e>/(string-lengt
    // Expected: true SUCCESS
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn k2_externalvariableswithout_18b() {
    //  XQuery 3.0 does allow externals to have a default value.
    // XPath: declare variable $var external := 1; 1
    // Expected: 1
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn k2_externalvariableswithout_21() {
    //  Use the global focus in various ways three references.
    // XPath: declare variable $global := count(*); $global, <wrongFocus>
    // Expected: 1 1 1
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn k2_externalvariableswithout_22() {
    //  Use the global focus in various ways three references.
    // XPath: declare variable $global := count(*); <wrongFocus> <e1/> <e2
    // Expected: 1
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_1() {
    //  Evaluates an external variable declaration with type Use type xs:string.
    // XPath: declare variable $x as xs:string external; $x
    // Expected: abc
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_2() {
    //  Evaluates an external variable declaration with type Use type xs:date.
    // XPath: declare variable $x as xs:date external; fn:string($x)
    // Expected: 2000-01-01+05:00
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_3() {
    //  Evaluates an external variable declaration with type Use type xs:ineteger.
    // XPath: declare variable $x as xs:integer external; $x
    // Expected: 2
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_4() {
    //  Evaluates an external variable declaration with type Use type xs:double.
    // XPath: declare variable $x as xs:double external; $x
    // Expected: 120
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_5() {
    //  Evaluates an external variable declaration with type Use type xs:boolean.
    // XPath: declare variable $x as xs:boolean external; fn:string($x)
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_7() {
    //  Evaluates an external variable declaration with type Use type xs:decimal.
    // XPath: declare variable $x as xs:decimal external; $x
    // Expected: 12678967.543233
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_8() {
    //  Evaluates an external variable that adds two integers Both queries perform the
    // XPath: declare variable $x as xs:integer external; $x + $x
    // Expected: 4
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_9() {
    //  Evaluates an external variable that adds two integers This query does NOT perfo
    // XPath: declare variable $x as xs:integer external; $x
    // Expected: 2
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_10() {
    //  Evaluates an external variable that adds two integers Only THIS query performs
    // XPath: declare variable $x as xs:integer external; $x + $x
    // Expected: 2
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_11() {
    //  Evaluates an external variable that multiply two integers Both queries perform
    // XPath: declare variable $x as xs:integer external; $x * $x
    // Expected: 16
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_12() {
    //  Evaluates an external variable that subtract two integers Both queries perform
    // XPath: declare variable $x as xs:integer external; $x - xs:integer(
    // Expected: 2
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_13() {
    //  Evaluates an external variable that divides two integers Both queries perform t
    // XPath: declare variable $x as xs:integer external; $x idiv xs:integ
    // Expected: 5
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_14() {
    //  Evaluates an external variable that divides (div operator) two integers Both qu
    // XPath: declare variable $x as xs:integer external; $x div xs:intege
    // Expected: 2
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_15() {
    //  Evaluates an external variable that perform modulus operation on two integers B
    // XPath: declare variable $x as xs:integer external; $x mod xs:intege
    // Expected: 1
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_16() {
    //  Evaluates an external variable that evaluates a boolean expression Both queries
    // XPath: declare variable $x as xs:boolean external; $x and fn:false(
    // Expected: false
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_17() {
    //  Evaluates an external variable that evaluates a boolean expression Both queries
    // XPath: declare variable $x as xs:boolean external; $x or fn:false()
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_20() {
    //  Evaluates an external variable whose case is changed.
    // XPath: declare variable $x as xs:string external; fn:upper-case($x)
    // Expected: THIS STRING SHOULD BE ALL IN UPPER CASE
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_21() {
    //  Evaluates an external variable whose case is changed.
    // XPath: declare variable $x as xs:string external; fn:lower-case($x)
    // Expected: this string should all be in lower case
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_22() {
    //  Evaluates an external variable whose value is succesfully casted.
    // XPath: declare variable $x as xs:boolean external; fn:not($x)
    // Expected: false
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn extvardeclwithtype_24() {
    //  Evaluates an external variable reference with type, variable name includes name
    // XPath: declare namespace test = \"http://www.w3.org/fots/example-te
    // Expected: 2
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn k2_externalvariableswith_22a() {
    //  A complex type declaration.
    // XPath: declare variable $v as element(*, xs:untyped?)+ := <e/>; exi
    // Expected: false
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}

#[test]
fn k2_externalvariableswith_23() {
    //  A complex type declaration.
    // XPath: declare variable $v as element(elementName, xs:anyType?)+ :=
    // Expected: 1
    // Calls stub function - will fail until prod:VarDecl.external is implemented
    let _ = stub_prodvardecl_external();
}
