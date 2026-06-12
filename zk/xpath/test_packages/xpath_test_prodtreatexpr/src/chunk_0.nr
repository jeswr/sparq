//! Test chunk 0 for prod:TreatExpr
//! Contains 45 tests

use dep::xpath::stub_prodtreatexpr;

#[test]
fn treat_as_2() {
    //  Evaluation of treat as expression where involving a string data type.
    // XPath: let $var := \"String 1\" return fn:concat($var treat as xs:s
    // Expected: String 1String 2
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_3() {
    //  Evaluation of treat as expression where involving an integer data type and div
    // XPath: let $var := 100 return ($var treat as xs:integer) div 2
    // Expected: 50
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_4() {
    //  Evaluation of treat as expression where involving an integer data type and div
    // XPath: let $var := 100 return fn:abs($var treat as xs:decimal)
    // Expected: 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_5() {
    //  Evaluation of treat as expression where involving an decimal/integer data types
    // XPath: let $var := xs:decimal(100) return fn:abs($var cast as xs:in
    // Expected: 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_6() {
    //  Evaluation of treat as expression where involving an xs:float data types and ab
    // XPath: let $var := xs:float(100) return fn:abs($var treat as xs:flo
    // Expected: 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_7() {
    //  Evaluation of treat as expression where involving an xs:double data type and ab
    // XPath: let $var := xs:double(100) return fn:abs($var treat as xs:do
    // Expected: 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_8() {
    //  Evaluation of treat as expression where involving an xs:boolean data type and f
    // XPath: let $var := xs:boolean(fn:true()) return fn:not($var treat a
    // Expected: false
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_9() {
    //  Evaluation of treat as expression where involving an xs:date data type and fn:y
    // XPath: let $var := xs:date(\"2000-01-01+05:00\") return fn:year-fro
    // Expected: 2000
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_10() {
    //  Evaluation of treat as expression where involving an xs:dateTime data type and
    // XPath: let $var := xs:dateTime(\"1999-05-31T13:20:00-05:00\") retur
    // Expected: 20
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_11() {
    //  Evaluation of treat as expression where involving an xs:time data type and fn:h
    // XPath: let $var := xs:time(\"01:23:00+05:00\") return fn:hours-from
    // Expected: 1
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_12() {
    //  Evaluation of treat as expression where involving an xs:integer data type and a
    // XPath: let $var := 100 return ($var treat as xs:integer, $var treat
    // Expected: 100 100 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_13() {
    //  Evaluation of treat as expression where involving an xs:integer data type and a
    // XPath: let $var := 100 return (xs:decimal($var) treat as xs:decimal
    // Expected: 100 100 100 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_14() {
    //  Evaluation of treat as expression with a 'cast as' expression and integer/float
    // XPath: let $var := -100 return fn:abs($var cast as xs:float treat a
    // Expected: 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_15() {
    //  Evaluation of treat as expression with a 'cast as' expression and integer/doubl
    // XPath: let $var := -100 return fn:abs($var cast as xs:double treat
    // Expected: 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_16() {
    //  Evaluation of treat as expression with a 'cast as' expression and float/integer
    // XPath: let $var := xs:float(-100) return fn:abs($var cast as xs:int
    // Expected: 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_17() {
    //  Evaluation of treat as expression with a 'cast as' expression and double/intege
    // XPath: let $var := xs:double(-100) return fn:abs($var cast as xs:in
    // Expected: 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_18() {
    //  Evaluation of treat as expression with a 'cast as' expression and double/float
    // XPath: let $var := xs:double(-100) return fn:abs($var cast as xs:fl
    // Expected: 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_19() {
    //  Evaluation of treat as expression with a 'cast as' expression and float/double
    // XPath: let $var := xs:float(-100) return fn:abs($var cast as xs:dou
    // Expected: 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_20() {
    //  Evaluation of treat as expression with a 'cast as' expression and decimal/doubl
    // XPath: let $var := xs:decimal(-100) return fn:abs($var cast as xs:d
    // Expected: 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_21() {
    //  Evaluation of treat as expression with a 'cast as' expression and decimal/float
    // XPath: let $var := xs:decimal(-100) return fn:abs($var cast as xs:f
    // Expected: 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_22() {
    //  Evaluation of treat as expression with a 'cast as' expression and float/decimal
    // XPath: let $var := xs:float(-100) return fn:abs($var cast as xs:dec
    // Expected: 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_23() {
    //  Evaluation of treat as expression with a 'cast as' expression and double/decima
    // XPath: let $var := xs:double(-100) return fn:abs($var cast as xs:de
    // Expected: 100
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_24() {
    //  Evaluation of treat as expression used as part of an addition operation.
    // XPath: let $var := (100+200) div 10 return ($var cast as xs:integer
    // Expected: 40
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_25() {
    //  Evaluation of treat as expression used as part of an subtraction operation.
    // XPath: let $var := (100+200) div 10 return ($var cast as xs:integer
    // Expected: 20
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn treat_as_27() {
    //  Can't treat an xs:anyURI as an xs:string even though it's promotable
    // XPath: string-join((\"a\", xs:anyURI(\"b\"), \"c\", xs:anyURI(\"d\"
    // Expected: "a"
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn k_seqexprtreat_5() {
    //  A test whose essence is: `xs:integer(3) treat as xs:decimal instance of xs:inte
    // XPath: xs:integer(3) treat as xs:decimal instance of xs:integer
    // Expected: true
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn k_seqexprtreat_6() {
    //  A test whose essence is: `'3' treat as xs:string eq '3'`.
    // XPath: \"3\" treat as xs:string eq '3'
    // Expected: true
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn k_seqexprtreat_7() {
    //  A test whose essence is: `xs:integer('3') treat as xs:decimal instance of xs:de
    // XPath: xs:integer(\"3\") treat as xs:decimal instance of xs:decimal
    // Expected: true
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn k_seqexprtreat_14() {
    //  A test whose essence is: `(3 treat as xs:integer ? * 3) eq 9`.
    // XPath: (3 treat as xs:integer ? * 3) eq 9
    // Expected: true
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn k_seqexprtreat_17() {
    //  A test whose essence is: `(remove((5, 1e0), 2) treat as xs:integer) eq 5`.
    // XPath: (remove((5, 1e0), 2) treat as xs:integer) eq 5
    // Expected: true
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn k2_seqexprtreat_2() {
    //  Use three pluses on a row, combined with 'treat as'.
    // XPath: 3 treat as item()+ + +1
    // Expected: 4
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn k2_seqexprtreat_4() {
    //  Treat the result of an empty node test, as the empty-sequence().
    // XPath: empty(<e/>/(* treat as empty-sequence()))
    // Expected: true
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn cbcl_treat_as_001() {
    //  Tests treat as with + quantifier
    // XPath: (1 to 10)[. mod 2 = 0] treat as xs:integer+
    // Expected: 2 4 6 8 10
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn cbcl_treat_as_002() {
    //  Tests treat as with * quantifier
    // XPath: (1 to 10)[. mod 2 = 0] treat as xs:integer*
    // Expected: 2 4 6 8 10
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn cbcl_treat_as_003() {
    //  Tests treat as with + quantifier with ? input
    // XPath: zero-or-one((1 to 10)[. div 2 = 1]) treat as xs:integer+
    // Expected: 2
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn cbcl_treat_as_004() {
    //  Tests treat as with * quantifier with ? input
    // XPath: zero-or-one((1 to 10)[. div 2 = 1]) treat as xs:integer*
    // Expected: 2
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn cbcl_treat_as_005() {
    //  Tests treat as with + quantifier with 1 input
    // XPath: exactly-one((1 to 10)[. div 2 = 1]) treat as xs:integer+
    // Expected: 2
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn cbcl_treat_as_006() {
    //  Tests treat as with * quantifier and 1 input
    // XPath: exactly-one((1 to 10)[. div 2 = 1]) treat as xs:integer*
    // Expected: 2
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn cbcl_treat_as_007() {
    //  Tests treat as with + quantifier
    // XPath: count((1 to 10)[. mod 2 = 0] treat as xs:integer+)
    // Expected: 5
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn cbcl_treat_as_008() {
    //  Tests treat as with + quantifier
    // XPath: subsequence((1 to 10)[. mod 2 = 0] treat as xs:integer+,1,2)
    // Expected: 2 4
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn cbcl_treat_as_016() {
    //  test treat as cardinality checks are correct
    // XPath: declare function local:range($arg as xs:integer) as xs:integ
    // Expected: 2
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn cbcl_treat_as_018() {
    //  test treat as cardinality checks are correct
    // XPath: declare function local:range($arg as xs:integer) as xs:integ
    // Expected: true
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn cbcl_treat_as_019() {
    //  test treat as cardinality checks are correct
    // XPath: declare function local:range($arg as xs:integer) as xs:integ
    // Expected: 2
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn cbcl_treat_as_022() {
    //  test treat as cardinality checks are correct
    // XPath: declare function local:range($arg as xs:integer) as xs:integ
    // Expected: 2
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}

#[test]
fn cbcl_treat_as_023() {
    //  test treat as cardinality checks are correct
    // XPath: declare function local:range($arg as xs:integer) as xs:integ
    // Expected: 2 4
    // Calls stub function - will fail until prod:TreatExpr is implemented
    let _ = stub_prodtreatexpr();
}
