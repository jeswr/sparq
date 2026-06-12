//! Test chunk 0 for prod:IfExpr
//! Contains 29 tests

use dep::xpath::stub_prodifexpr;

#[test]
fn condexpr008() {
    //  and-expression in test expression
    // XPath: if ( /Root/Customers[1]/@CustomerID = 'ALFKI' and /Root/Cust
    // Expected: pass
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn condexpr009() {
    //  or-expression in test expression
    // XPath: if (/Root/Customers[1]/@CustomerID = 'ALFKI' or /Root/Custom
    // Expected: pass
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn condexpr010() {
    // FileName: CondExpr010  a true expression in if expression
    // XPath: if (2 != 4) then 1 else 0
    // Expected: 1
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn condexpr011() {
    // FileName: CondExpr011  If expression as argument to a function
    // XPath: fn:string-length(if (2 != 3) then 'foo' else 'expanded-foo')
    // Expected: 3
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn condexpr012() {
    // FileName: CondExpr012  Adapted from example in spec; test expression is a relati
    // XPath: if (/doc/widget1/@unit-cost = /doc/widget2/@unit-cost) then
    // Expected: axolotl
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn condexpr013() {
    // FileName: CondExpr013  test expression has another if expression
    // XPath: if (if (5 != 3) then fn:true() else fn:empty(/doc/widget1))
    // Expected: search
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn condexpr014() {
    // FileName: CondExpr014  Two if expressions at same depth in larger expression
    // XPath: ( if (3 != 2) then 16 else 0 ) + ( if (8 = 7) then 4 else 1
    // Expected: 17
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn condexpr015() {
    // FileName: CondExpr015  Two if expressions at same depth in larger expression, ea
    // XPath: (if (3 != 2) then 16 else 0) + (if (8 = 7) then 4 else 1)
    // Expected: 17
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn condexpr018() {
    // FileName: CondExpr018  if where branches return different valid types
    // XPath: for $file in (//Folder)[1]/File return if( $file/FileName='F
    // Expected: File00000000000 File00000000001 File0000
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn condexpr20() {
    // FileName: CondExpr020  Test case where then-expression raises a dynamic error, b
    // XPath: if( false() ) then /Folder[1] cast as xs:double ? else 10 ca
    // Expected: 10
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn condexpr21() {
    // FileName: CondExpr021  Test case where else-expression raises a dynamic error, b
    // XPath: if( true() ) then 10 cast as xs:double ? else /Folder[1] cas
    // Expected: 10
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn condexpr022() {
    //  Test case where then-expression raises a dynamic error, but test-expression sel
    // XPath: if (doc-available('nodocument.xml')) then doc('nodocument.xm
    // Expected: 10
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k_condexpr_1() {
    //  A test whose essence is: `(if(2) then 1 else 0) eq 1`.
    // XPath: (if(2) then 1 else 0) eq 1
    // Expected: true
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k_condexpr_2() {
    //  A test whose essence is: `if(()) then false() else true()`.
    // XPath: if(()) then false() else true()
    // Expected: true
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k_condexpr_3() {
    //  An expression involving the if expression that trigger certain optimization pat
    // XPath: (if((1, current-time())[1] treat as xs:integer) then true()
    // Expected: true
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k_condexpr_4() {
    //  An expression involving the if expression that trigger certain optimization pat
    // XPath: (if(boolean((1, current-time())[1] treat as xs:integer)) the
    // Expected: true
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k_condexpr_5() {
    //  An expression involving the if expression that trigger certain optimization pat
    // XPath: if((1, 2, 3, hours-from-time(current-time()))[1]) then true(
    // Expected: true
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k_condexpr_6() {
    //  An expression involving the if expression that trigger certain optimization pat
    // XPath: string(if(boolean((1, current-time())[1] treat as xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k_condexpr_8() {
    //  An if-test which EBV cannot be extracted from.
    // XPath: 1 eq (if(xs:anyURI(\"example.com/\")) then 1 else 0)
    // Expected: true
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k_condexpr_9() {
    //  An if-test which EBV cannot be extracted from.
    // XPath: 0 eq (if(xs:anyURI(\"\")) then 1 else 0)
    // Expected: true
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k_condexpr_10() {
    //  An if-test applied on fn:count().
    // XPath: if(count((1, 2, 3, current-time(), 4))) then true() else 4
    // Expected: true
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k_condexpr_11() {
    //  An if-test applied on fn:count() combined with fn:not().
    // XPath: if(not(count(remove((1, 2, 3, current-time()), 1)))) then 3
    // Expected: true
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k_condexpr_12() {
    //  if-then clause combined with fn:boolean().
    // XPath: if(boolean((1, 2, 3, current-time())[1] treat as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k2_condexpr_2() {
    //  An expression that can be simplified to any of the result branches, no matter t
    // XPath: empty(if(<e>{current-time()}</e>) then 1 else 1)
    // Expected: false
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k2_condexpr_3() {
    //  It is ok to name a function 'unordered'.
    // XPath: declare default function namespace \"http://www.example.com/
    // Expected: 1
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k2_condexpr_4() {
    //  Extract the EBV from a node sequence. Sorting and deduplication isn't necessary
    // XPath: if(//(employee[location = \"Denver\"]/ancestor::*)) then 1 e
    // Expected: 1
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn k2_condexpr_7() {
    //  Use a set of expressions that trigger a bug in some parsers.
    // XPath: if (1) then 1 else if (1) then 1 else if (1) then 1 else if
    // Expected: 1
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn cbcl_condexpr_001() {
    //  Tests if(exists($x)) then f($x) else () optimization
    // XPath: let $x := zero-or-one((1 to 10)[. div 2 = 2])         return
    // Expected: 4
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}

#[test]
fn cbcl_condexpr_002() {
    //  tests if(empty($x)) then () else f($x) optimization
    // XPath: let $x := zero-or-one((1 to 10)[. div 2 = 2])         return
    // Expected: 4
    // Calls stub function - will fail until prod:IfExpr is implemented
    let _ = stub_prodifexpr();
}
