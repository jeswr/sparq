//! Test chunk 0 for prod:OrderByClause
//! Contains 18 tests

use dep::xpath::stub_prodorderbyclause;

#[test]
fn k2_orderbyexprwithout_1() {
    //  Use a relative, valid collation.
    // XPath: declare base-uri \"http://www.w3.org/2005/xpath-functions/\"
    // Expected: 1 2 3
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_5() {
    //  A simple sorting of integers.
    // XPath: for $i in (1, 3, 2) order by $i return $i
    // Expected: 1 2 3
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_6() {
    //  A simple sorting of integers(#2).
    // XPath: for $i in (1, 3, 2) order by $i return ($i, 2)
    // Expected: 1 2 2 2 3 2
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_11() {
    //  A sort where the for-binding is only used as a sort key.
    // XPath: for $a in (1, 4, 2) let $i := (1, 3, 2) order by $a return $
    // Expected: 1 3 2 1 3 2 1 3 2
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_12() {
    //  A sort with an unused for-binding.
    // XPath: for $a in (3, 2, 1), $b in (6, 5, 4) order by $a return $a
    // Expected: 1 1 1 2 2 2 3 3 3
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_13() {
    //  A sort with a for-binding whose only purpose is sorting.
    // XPath: for $a in (3, 2, 1), $b in (6, 5, 4) stable order by $b retu
    // Expected: 3 2 1 3 2 1 3 2 1
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_16() {
    //  Sort booleans.
    // XPath: for $i in (false(), true(), true(), false(), true(), false()
    // Expected: false false false true true true
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_17() {
    //  Extract the effective boolean value from a order by expression.
    // XPath: boolean((for $i in (false(), true(), true(), false(), true()
    // Expected: false
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_18() {
    //  Sort a single atomic value.
    // XPath: (for $i in current-time() order by $i return $i) eq current-
    // Expected: true
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_38() {
    //  'order by' combined with reverse().
    // XPath: for $i in (1, 2, 3) stable order by 1 return reverse(($i, \"
    // Expected: FO 1 FO 2 FO 3
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_41() {
    //  order by, with many let bindings inbetween.
    // XPath: for $a in (3, 2, 1) let $b := (2, 1), $c := (2, 1), $d := (2
    // Expected: 1 2 3
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_43() {
    //  Extract the effective boolean value from the result of order by.
    // XPath: if(for $i in <e> <a id=\"3\"/> <b id=\"2\"/> <c id=\"1\"/> <
    // Expected: 4
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_44() {
    //  Sort, with a where and let clause in between.
    // XPath: for $a in (2, 1) let $b := 1 where true() order by $a return
    // Expected: 1 2
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_45() {
    //  Short key-for involved in sorting.
    // XPath: for $a in (3, 2, 1), $b in (6) stable order by $b return $a
    // Expected: 3 2 1
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_46() {
    //  Sort special floating point values.
    // XPath: let $numbers := (1, 2, 1.3, 3e3, xs:double(\"NaN\"), xs:doub
    // Expected: NaN -INF 1 1.3 2 3000 INF SEP -INF 1 1.3
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_47() {
    //  Sort special floating point values(stable sort).
    // XPath: let $numbers := (1, 2, 1.3, 3e3, xs:double(\"NaN\"), xs:doub
    // Expected: NaN -INF 1 1.3 2 3000 INF SEP -INF 1 1.3
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_48() {
    //  Sort special floating point values.
    // XPath: let $numbers := (<e>NaN</e>, <e/>, <e/>, <e>NaN</e>, <e>NaN<
    // Expected: NaN NaN NaN NaN 3 INF SEP 3 INF NaN NaN
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}

#[test]
fn k2_orderbyexprwithout_49() {
    //  Sort special floating point values(stable sort).
    // XPath: let $numbers := (<e>NaN</e>, <e/>, <e/>, <e>NaN</e>, <e>NaN<
    // Expected: NaN NaN NaN NaN 3 INF SEP 3 INF NaN NaN
    // Calls stub function - will fail until prod:OrderByClause is implemented
    let _ = stub_prodorderbyclause();
}
