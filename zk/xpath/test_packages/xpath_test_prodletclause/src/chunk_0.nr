//! Test chunk 0 for prod:LetClause
//! Contains 50 tests

use dep::xpath::stub_prodletclause;

#[test]
fn letexprwith_2() {
    //  Let clause used decimal type.
    // XPath: let $var as xs:decimal := 100 return $var
    // Expected: 100
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_3() {
    //  Let clause using integer type.
    // XPath: let $var as xs:integer := 100 return $var
    // Expected: 100
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_4() {
    //  Let clause using double type.
    // XPath: let $var as xs:double := 100E1 return $var
    // Expected: 1000
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_5() {
    //  Let clause using double type.
    // XPath: let $var as xs:string := \"A String\" return $var
    // Expected: A String
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_6() {
    //  Let clause using boolean (true) type.
    // XPath: let $var as xs:boolean := fn:true() return $var
    // Expected: true
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_7() {
    //  Let clause using boolean (false) type.
    // XPath: let $var as xs:boolean := fn:false() return $var
    // Expected: false
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_8() {
    //  Let clause using date type.
    // XPath: let $var as xs:date := xs:date(\"1999-05-31Z\") return $var
    // Expected: 1999-05-31Z
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_9() {
    //  Let clause using time type.
    // XPath: let $var as xs:time := xs:time(\"21:23:00Z\") return $var
    // Expected: 21:23:00Z
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_10() {
    //  Let clause using dateTime type.
    // XPath: let $var as xs:dateTime := xs:dateTime(\"1999-05-31T13:20:00
    // Expected: 1999-05-31T13:20:00Z
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_11() {
    //  Let clause using float type.
    // XPath: let $var as xs:float := xs:float(100) return $var
    // Expected: 100
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_12() {
    //  Let clause using an ineteger expression type.
    // XPath: let $var as xs:integer := 100+200 return $var
    // Expected: 300
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_13() {
    //  Let clause using the fn:count function.
    // XPath: let $var as xs:integer := fn:count((100,200)) return $var
    // Expected: 2
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_14() {
    //  Let clause using the fn:not function.
    // XPath: let $var as xs:boolean := fn:not(fn:true()) return $var
    // Expected: false
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_15() {
    //  Let clause using the 'and' operator function.
    // XPath: let $var as xs:boolean := fn:true() and fn:true() return $va
    // Expected: true
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_16() {
    //  Let clause using the 'or' operator function.
    // XPath: let $var as xs:boolean := fn:true() and fn:true() return $va
    // Expected: true
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_17() {
    //  Let clause using the 'string-length' function.
    // XPath: let $var as xs:integer := fn:string-length(\"A String\") ret
    // Expected: 8
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_18() {
    //  Let clause using a casting from integer to string
    // XPath: let $var as xs:string := xs:string((xs:integer(100))) return
    // Expected: 100
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_19() {
    //  Let clause using a casting from decimal to string
    // XPath: let $var as xs:string := xs:string((xs:decimal(100))) return
    // Expected: 100
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_20() {
    //  Let clause using a casting from double to string
    // XPath: let $var as xs:string := xs:string((xs:double(100E2))) retur
    // Expected: 10000
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_21() {
    //  Let clause using a casting from boolean to string
    // XPath: let $var as xs:string := xs:string(fn:true()) return $var
    // Expected: true
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_22() {
    //  Let clause using a integer type and addition expression on return clause
    // XPath: let $var as xs:integer := 100 return $var + 1
    // Expected: 101
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_23() {
    //  Let clause using an 'if' expression
    // XPath: let $var as xs:boolean := if (fn:true()) then fn:true() else
    // Expected: true
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexprwith_24() {
    //  Let clause using a 'typeswitch' expression
    // XPath: let $var as xs:string := typeswitch(fn:true()) case $i as xs
    // Expected: Test Passed
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexpr001() {
    // Purpose : Assign a simple string using let
    // XPath: let $x := \"92233720368547758\" return $x
    // Expected: 92233720368547758
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexpr002() {
    // Purpose : Assign a simple integer using let
    // XPath: let $x := 92233720368547758 return $x
    // Expected: 92233720368547758
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexpr003() {
    // Purpose : Use an arithmetic operator with let
    // XPath: let $x := 92233720368547758+1 return $x
    // Expected: 92233720368547759
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexpr004() {
    // Purpose : Use xs:long(upper bound)
    // XPath: let $x := xs:long(\"92233720368547758\") return $x
    // Expected: 92233720368547758
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexpr005() {
    // Purpose : Use xs:long(lower bound) +1
    // XPath: let $x := xs:long(\"-92233720368547758\")+1 return $x
    // Expected: -92233720368547757
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexpr006() {
    // Purpose : Use xs:double(upper bound)
    // XPath: let $x := xs:double(\"1.7976931348623157E308\") return $x
    // Expected: 1.7976931348623157E308
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexpr008() {
    // Purpose : More than one variable cross referencing compatible values
    // XPath: let $x:=1, $y:=$x+1 return $x
    // Expected: 1
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexpr011() {
    // Purpose : Use sequences with filter expressions
    // XPath: let $x :=(1 to 100)[. mod 5 eq 0] return $x
    // Expected: 5 10 15 20 25 30 35 40 45 50 55 60 65 70
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexpr012() {
    // Purpose : Use sequences with filter expressions with more than one variable
    // XPath: let $x :=(1 to 100)[. mod 5 eq 0], $y := $x[. mod 10 eq 0] r
    // Expected: 10 20 30 40 50 60 70 80 90 100
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexpr013() {
    // Purpose : Use a function in the let
    // XPath: let $x:=\"hello\", $y:=concat($x,\" there\") return $y
    // Expected: hello there
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexpr015() {
    // Purpose : Use a sequence of different types of scalars
    // XPath: let $x := (0,0.1e-1,2.0,'a',\"cat\",'',true()) return $x
    // Expected: 0 0.01 2 a cat  true
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn letexpr020() {
    // Purpose : Use three variables
    // XPath: let $a := 1 let $b := $a let $c := $a+$b return ($c)
    // Expected: 2
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn k_letexprwithout_4() {
    //  A binding in a let-expression shadows global variables.
    // XPath: declare variable $i := false();          declare variable $t
    // Expected: true
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn k2_letexprwithout_1() {
    //  fn:deep-equal combined with a for/let expression.
    // XPath: deep-equal((<b/>, <b/>, <b/>, <b/>), (for $v1 in (1, 2, 3, 4
    // Expected: true
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn k2_letexprwithout_2() {
    //  fn:deep-equal combined with a for/let expression(#2).
    // XPath: deep-equal((<b/>, <b/>, <b/>, <b/>), (for $v1 in (1, 2, 3, 4
    // Expected: true
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn k2_letexprwithout_16() {
    //  Ensure node identity is properly handled through a for binding.
    // XPath: declare variable $e := <e/>; for $i in (<a/>, $e, <c/>) retu
    // Expected: false true false
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn k2_letexprwithout_18() {
    //  Use a let-variable that acts as an alias for a function argument.
    // XPath: declare function local:foo($a as xs:integer) { if($a = 3) th
    // Expected: 3
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn k2_letexprwithout_23() {
    //  Bind the context item to a variable.
    // XPath: count(<e/>/(let $i := . return (string($i), data($i))))
    // Expected: 2
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn k2_letexprwithout_25() {
    //  A combination of expressions that triggers a bug in some parsers.
    // XPath: let $a := 1 return                  for $b in 1 return
    // Expected: 1
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn xquery10keywords() {
    // Purpose : keywords can be used as variable names
    // XPath: let $ancestor-or-self := 1        let $ancestor := 1
    // Expected: 2
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn xquery10keywords3() {
    // Purpose : keywords can be used as element names
    // XPath: <keywords> <ancestor-or-self/> <ancestor/> <and/> <as/> <asc
    // Expected: keywords
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn xquery30keywords() {
    // Purpose : keywords can be used as variable names
    // XPath: let $NaN := 1         let $allowing := 1         let $ancest
    // Expected: 2
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn xquery30keywords3() {
    // Purpose : keywords can be used as element names
    // XPath: <keywords>            <NaN/>            <allowing/>
    // Expected: keywords
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn xquery30keywords4() {
    // Purpose : keywords can be used as fn-prefixed function names
    // XPath: declare function local:NaN () {1};          declare function
    // Expected: true
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn xquery30keywords5() {
    // Purpose : keywords can be used as unprefixed function names
    // XPath: declare default function namespace \"http://www.w3.org/2005/
    // Expected: true
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn xquery31keywords1() {
    // Purpose : New XQuery 3.1 keyword can be used as variable name
    // XPath: let $exponent-separator := 1         return 2
    // Expected: 2
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}

#[test]
fn xquery31keywords3() {
    // Purpose : New XQuery 3.1 keyword can be used as element name
    // XPath: <keywords>            <exponent-separator/>         </keywor
    // Expected: keywords
    // Calls stub function - will fail until prod:LetClause is implemented
    let _ = stub_prodletclause();
}
