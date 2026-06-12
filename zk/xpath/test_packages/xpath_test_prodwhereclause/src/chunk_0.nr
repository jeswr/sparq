//! Test chunk 0 for prod:WhereClause
//! Contains 50 tests

use dep::xpath::stub_prodwhereclause;

#[test]
fn whereexpr022() {
    //  Composite where clause depending on multiple variables
    // XPath: for $file in //File           let $namelen := string-length(
    // Expected: File00000000000
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereexpr023() {
    //  Where clause appearing after a count clause
    // XPath: for $file in //File          count $c          where ($file/
    // Expected: 2
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereexpr024() {
    //  Multiple where clauses in suboptimal order
    // XPath: for $file in //File           let $prefix := substring($file
    // Expected: File00000000000
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereexpr025() {
    //  Where clause with no dependencies on FLWOR variables
    // XPath: for $file in //File           where current-date() gt xs:dat
    // Expected: File00000000000
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereexpr026() {
    //  Where clause that depends on position()
    // XPath: let $doc := (/)          return (11 to 15)!             (for
    // Expected: File00000000000 File00000000001 File0000
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereexpr027() {
    //  Where clause that depends on last()
    // XPath: let $doc := (/)          return (11 to 15)!             (for
    // Expected: File00000000004 File00000000004 File0000
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereexpr028() {
    //  Where clause with let (and no for)
    // XPath: let $x := 1 to 10          where $x = 1          return $x
    // Expected: 1 2 3 4 5 6 7 8 9 10
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereexpr030() {
    //  Where clause with 'outer for'
    // XPath: declare variable $n external := 10;          for $x allowing
    // Expected: 1 2 3 4
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereexpr031() {
    //  Where clause with 'outer for'
    // XPath: declare variable $n external := 0;          for $x allowing
    // Expected: []
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereexpr033() {
    //  Integer-valued where clause. A trap for systems that rewrite where clauses as p
    // XPath: declare variable $n external := 0;          for $x in -2 to
    // Expected: -2 -1 1 2
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereexpr034() {
    // A where clause that doesn't depend on the previous for clause (Saxon optimisatio
    // XPath: for $i in 1 to 10          for $j in 10 to 14          where
    // Expected: 11 12 13 14 15 13 14 15 16 17
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereexpr035() {
    // Alternative way of writing WhereExpr034 that triggers Saxon optimisation bug 476
    // XPath: for $i in 1 to 10          return              for $j in 10
    // Expected: 11 12 13 14 15 13 14 15 16 17
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereclause_1() {
    //  use of where clause involving an 'or' expression. Use fn:true()
    // XPath: let $var := (fn:true()) where $var or fn:true() return $var
    // Expected: true
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereclause_2() {
    //  use of where clause involving an 'or' expression. Use fn:false() Use fn:not()
    // XPath: let $var := (fn:false()) where fn:not($var or fn:false()) re
    // Expected: false
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereclause_3() {
    //  use of where clause involving an 'or' ('and' operator) expression. Use fn:true(
    // XPath: let $var := (fn:true()) where $var and fn:true() return $var
    // Expected: true
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereclause_4() {
    //  use of where clause involving an 'or' ('and' operator) expression. Use fn:false
    // XPath: let $var := (fn:false()) where fn:not($var and fn:false()) r
    // Expected: false
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereclause_5() {
    //  use of where clause with a typeswitch expression
    // XPath: let $var := 100 where typeswitch($var) case $i as xs:string
    // Expected: 100
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereclause_6() {
    //  use of where clause used with a string value and fn:string
    // XPath: let $var := \"String\" where fn:string($var) = \"String\" re
    // Expected: String
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereclause_7() {
    //  use of where clause used with a string-length function.
    // XPath: let $var := \"String\" where fn:string-length($var) = 6 retu
    // Expected: 6
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereclause_8() {
    //  use of where clause used with a fn:count() function.
    // XPath: let $var := 100 where fn:count(($var)) = 1 return fn:count((
    // Expected: 1
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereclause_9() {
    //  use of where clause used with is comparator.
    // XPath: let $var := <anElement>Some content</anElement> where $var i
    // Expected: true
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereclause_10() {
    //  use of where clause used with quantified expression (some keyword).
    // XPath: let $var := 1           where some $x in (1, 2) satisfies fn
    // Expected: 1
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereclause_11() {
    //  use of where clause used with quantified expression (every keyword).
    // XPath: let $var := 1           where every $x in (1, 1) satisfies f
    // Expected: 1
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn whereclause_12() {
    // A Saxon optimization bug - #4870
    // XPath: let $seq := (1, reverse(1 to 2))          for $item in disti
    // Expected: 1
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn k_whereexpr_1() {
    //  A for/where combination where the cardinality of the return statement is crucia
    // XPath: (for $fo in (1, 2, 3) where $fo eq 3 return $fo)
    // Expected: 3
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn k_whereexpr_2() {
    //  Get the string value of the return statement of a for expression after being fi
    // XPath: string(exactly-one((for $fo in (1, 2, 3) where $fo eq 3 retu
    // Expected: 3
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn k_whereexpr_4() {
    //  A for/where expression combined with fn:count().
    // XPath: for $i in 1 where count(($i, 2, timezone-from-time(current-t
    // Expected: true
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn k_whereexpr_5() {
    //  A for expression binding to one single value, combined with a positive where cl
    // XPath: empty(for $i in 1 where false() return $i)
    // Expected: true
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn k_whereexpr_6() {
    //  A for expression binding to one single value, combined with a negative where cl
    // XPath: (for $i in 1 where true() return $i) eq 1
    // Expected: true
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn k_whereexpr_7() {
    //  A for expression binding to one single value, combined with a positive where cl
    // XPath: empty(for $i in 1 where (1, current-time())[1] treat as xs:i
    // Expected: true
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn k_whereexpr_8() {
    //  A for expression binding to one single value, combined with a negative where cl
    // XPath: (for $i in 1 where (1, current-time())[1] treat as xs:intege
    // Expected: true
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn k_whereexpr_9() {
    //  A for/where expression combined with fn:boolean.
    // XPath: for $i in (1, 2, current-time())[1] where fn:boolean($i trea
    // Expected: true
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn k_whereexpr_10() {
    //  A for/where expression combined with fn:boolean and xs:anyURI.
    // XPath: for $i in (1, 2, current-time())[1] where xs:anyURI(\"exampl
    // Expected: true
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn k_whereexpr_11() {
    //  A for/where expression combined with fn:boolean and xs:anyURI.
    // XPath: empty(for $i in (1, 2, current-time())[1] where xs:anyURI(\"
    // Expected: true
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn k2_whereexpr_1b() {
    //  Multiple where clauses are allowed in XQuery 3.0.
    // XPath: for $a in 1 where true() where true() return $a
    // Expected: 1
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_hash_join_2() {
    //  A query that can be optimized down to a hash join - with integer types.
    // XPath: for $x in (xs:integer(\"123\"), xs:integer(\"234\")) for $y
    // Expected: 123
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_hash_join_3() {
    //  A query that can be optimized down to a hash join - with date types of varying
    // XPath: for $x in (xs:date(\"2010-10-10\"), xs:date(\"1997-01-01+12:
    // Expected: 2010-10-10 1997-01-02+12:00
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_hash_join_005() {
    //  Test tricky hash-join implicit conversion case.
    // XPath: declare function local:durationOrFloat($i as xs:integer) { (
    // Expected: 123 234
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_product_001() {
    //  Test products which return empty sequence
    // XPath: declare function local:odds($arg as xs:integer*) as xs:integ
    // Expected: false
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_set_from_optional_item_001() {
    //  Tests set from optional item returning empty sequence
    // XPath: declare function local:odds($arg as xs:integer?) as xs:integ
    // Expected: false
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_hash_join_007() {
    //  Test a hash join that produces a static type check error, but is fine in dynami
    // XPath: declare function local:sequence($arg as xs:integer) as item(
    // Expected: 1 1 3 3 5 5 7 7 9 9
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_hash_join_008() {
    //  Test a hash join that casts untypedAtomic values to multiple types
    // XPath: declare function local:sequence($arg as xs:integer) as item(
    // Expected: 1 1
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_group_by_key_002() {
    //  A variant on group-by-key that does not involve a left outer join
    // XPath: let $items := for $x in 1 to 100 return $x mod 10, $distinct
    // Expected: 1 1 1 1 1 1 1 1 1 1 2 2 2 2 2 2 2 2 2 2
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_group_by_key_003() {
    //  A group-by-key that has to be careful about errors
    // XPath: let $items := for $x in 1 to 100 return $x mod 10, $distinct
    // Expected: 1000 500 333 250 200 167 143 125 111 0
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_group_by_key_004() {
    //  A group by that has to be careful about errors
    // XPath: let $items := for $x in 1 to 100 return $x mod 10, $distinct
    // Expected: 1000 500 333 250 200 167 143 125 111 0
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_left_outer_join_001() {
    //  A left outer join with some tricky error avoidance
    // XPath: let $items := for $x in 1 to 100 return $x mod 10, $distinct
    // Expected: 1000 500 333 250 200 167 143 125 111 0
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_hash_join_011() {
    //  Test a hash join with sequences for keys
    // XPath: for $x in <t><r><d>1</d><d>1</d><d>1</d></r><r><d>1</d><d>2<
    // Expected: 111=111 111=321 123=111 123=246 123=321
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_map_concat_001() {
    //  Test to hit EvaluateOptional on MapConcatSingle
    // XPath: declare function local:f($x) { if ($x) then true() else () }
    // Expected: true true
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_map_concat_002() {
    //  Test to hit Evaluate on MapConcatOptional
    // XPath: declare function local:f($x) { if ($x) then true() else () }
    // Expected: true
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}

#[test]
fn cbcl_map_concat_003() {
    //  Test product with optional input
    // XPath: declare function local:f($x) { if ($x) then true() else () }
    // Expected: true 1 true 2 true 3 true 4 true 5 true
    // Calls stub function - will fail until prod:WhereClause is implemented
    let _ = stub_prodwhereclause();
}
