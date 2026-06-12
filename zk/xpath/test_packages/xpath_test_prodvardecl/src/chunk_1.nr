//! Test chunk 1 for prod:VarDecl
//! Contains 42 tests

use dep::xpath::stub_prodvardecl;

#[test]
fn vardecl051() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := (xs:string(\"a\") , (), \"xyz\") ; $x
    // Expected: a xyz
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl052() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := (xs:string(\"a\") , xs:anyURI(\"www.e
    // Expected: a www.example.com
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl053() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := (xs:float(\"INF\") , xs:double(\"NaN\
    // Expected: INF NaN
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl054() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := (xs:boolean(\"true\") , xs:boolean(\"
    // Expected: true false 0
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl055() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := (xs:date(\"1993-03-31\") , xs:boolean
    // Expected: 1993-03-31 true abc
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl056() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := (xs:time(\"12:30:00\") , xs:string(\"
    // Expected: 12:30:00   2.000000000000002
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl057() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := ((1+1), (2-2)) ; $x
    // Expected: 2 0
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl058() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := ((1,2,2),(1,2,3),(123,\"\"),(),(\"\")
    // Expected: 1 2 2 1 2 3 123
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl060() {
    // Purpose - Variable with no type definition
    // XPath: declare variable $x := //Price/text() ; $x
    // Expected:
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl063() {
    // Purpose - forwards references to global variables allowed in XQuery 3.0
    // XPath: declare variable $x := $y + 3;         declare variable $y :
    // Expected: 25
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardecl064() {
    // Purpose - two forwards references to a global variable (Saxon bug 2232)
    // XPath: declare function local:f1() {$a};         declare function l
    // Expected: 38
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn internalvar_2() {
    //  Evaluates an external variable named '$local:var' declared with no type and mul
    // XPath: (::)declare(::)variable(::)$var(::):=(::)1(::);(::) 1(::)eq(
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn k2_internalvariableswithout_1b() {
    // Static circularity with no dynamic circularity: see bug 29590 (and Saxon bug 272
    // XPath: declare variable $local:myVar := local:myFunction();
    // Expected: 22
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn k2_internalvariableswithout_10() {
    //  Function arguments shadow global variables.
    // XPath: declare variable $local:myVar := local:myFunc(3); declare fu
    // Expected: 6
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn k2_internalvariableswithout_13() {
    //  A prolog containing 20 variable declarations.
    // XPath: declare variable $local:var1 := 1; declare variable $local:v
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_1() {
    //  Evaluates an internal variable declaration with type Use type xs:string.
    // XPath: declare variable $var as xs:string := \"abc\"; $var
    // Expected: abc
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_2() {
    //  Evaluates an internal variable declaration with type Use type xs:integer.
    // XPath: declare variable $var as xs:integer := 100; $var
    // Expected: 100
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_3() {
    //  Evaluates an internal variable declaration with type Use type xs:decimal.
    // XPath: declare variable $var as xs:decimal := 100; $var
    // Expected: 100
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_4() {
    //  Evaluates an internal variable declaration with type Use type xs:boolean (true
    // XPath: declare variable $var as xs:boolean := fn:true(); $var
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_5() {
    //  Evaluates an internal variable declaration with type Use type xs:boolean (false
    // XPath: declare variable $var as xs:boolean := fn:false(); $var
    // Expected: false
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_6() {
    //  Evaluates an internal variable declaration with type Use type xs:float
    // XPath: declare variable $var as xs:float := xs:float(12.5E10); $var
    // Expected: 1.25E11
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_7() {
    //  Evaluates an internal variable declaration with type Use type xs:double
    // XPath: declare variable $var as xs:double := xs:double(1267.43233E1
    // Expected: 1.26743233E15
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_8() {
    //  Evaluates an internal variable declaration with type Use type xs:anyURI
    // XPath: declare variable $var as xs:anyURI := xs:anyURI(\"http://exa
    // Expected: http://example.com
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_9() {
    //  Evaluates an internal variable declaration with type Use type xs:dateTime
    // XPath: declare variable $var as xs:dateTime := xs:dateTime(\"1999-1
    // Expected: 1999-11-28T09:00:00Z
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_10() {
    //  Evaluates an internal variable declaration with type Use type xs:time
    // XPath: declare variable $var as xs:time := xs:time(\"11:12:00Z\");
    // Expected: 11:12:00Z
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_11() {
    //  Evaluates an internal variable declaration with type Use type xs:date
    // XPath: declare variable $var as xs:date := xs:date(\"1999-11-28Z\")
    // Expected: 1999-11-28Z
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_14() {
    //  Evaluates an internal variable declaration with type and expression. Expression
    // XPath: declare variable $x as xs:integer := 10; declare variable $v
    // Expected: 11
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_15b() {
    //  Evaluates an internal variable declaration with type and expression.        Tes
    // XPath: declare variable $var as xs:integer := $e +1;        declare
    // Expected: 11
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_16() {
    //  Evaluates an internal variable declaration with type that uses a declared names
    // XPath: declare namespace p1 = \"http://www.example.com\";         d
    // Expected: 10
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_17() {
    //  Evaluates an internal variable declaration with type that uses a declared names
    // XPath: declare namespace p1 = \"http://www.example.com\";         d
    // Expected: 20
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_19() {
    //  Evaluates an internal variable declaration with type that uses the fn:count fun
    // XPath: declare variable $var as xs:integer := fn:count((10,2)); $va
    // Expected: 2
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_20() {
    //  Evaluates an internal variable declaration with type that uses the fn:string-le
    // XPath: declare variable $var as xs:integer := fn:string-length(\"AB
    // Expected: 3
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_21() {
    //  Evaluates an internal variable declaration with type that uses the fn:not funct
    // XPath: declare variable $var as xs:boolean := fn:not(fn:true()); $v
    // Expected: false
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn vardeclwithtype_22() {
    //  Evaluates an internal variable declaration with type that uses the fn:empty fun
    // XPath: declare variable $var as xs:boolean := fn:empty((1,2,3)); $v
    // Expected: false
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn internalvar_1() {
    //  Evaluates an external variable named '$local:var' declared as 'item()*' and mul
    // XPath: declare(::)variable(::)$var(::)as(::)item((: :))*(::):=(::)1
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn k_internalvariableswith_1() {
    //  A variable declaration involving assignment and type declaration, and many comm
    // XPath: declare(::)variable(::)$local:var(::)as(::)item((: :))* :=3(
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn k_internalvariableswith_2() {
    //  A variable declaration involving assignment, and many comments.
    // XPath: (::)declare(::)variable(::)$local:var(::):=(::)3;(::)1(::)eq
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn k_internalvariableswith_6b() {
    //  A prolog variable depending on a variable which is (in XQuery 3.0) in scope,
    // XPath: declare variable $var1 := $var2; declare variable $var2 := 2
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn k_internalvariableswith_9b() {
    //  A prolog variable depending on a variable which is (in XQuery 3.0) in scope.
    // XPath: declare variable $var1 := $var2; declare variable $var2 := 2
    // Expected: 2
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn k_internalvariableswith_11() {
    //  One prolog variable initialized via another.
    // XPath: declare variable $var1 := 2; declare variable $var2 := $var1
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn k_internalvariableswith_12() {
    //  One prolog variable depending on a user function appearing after it.
    // XPath: declare variable $var1 := local:myFunc(); declare function l
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}

#[test]
fn k_internalvariableswith_21() {
    //  A variable declaration involving assignment and type declaration, demonstrating
    // XPath: declare variable $local:var as item() *:=3; true()
    // Expected: true
    // Calls stub function - will fail until prod:VarDecl is implemented
    let _ = stub_prodvardecl();
}
