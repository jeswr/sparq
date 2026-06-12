//! Test chunk 0 for fn:last
//! Contains 46 tests

use dep::xpath::stub_fnlast;

#[test]
fn last_1() {
    //  Evaluation of 'fn:last', where context function is an element node and the posi
    // XPath: for $h in (/works) return $h//employee[last()]/string(@name)
    // Expected: Jane Doe 13
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn last_4() {
    //  Evaluation of 'fn:last' together with 'fn:position'.
    // XPath: for $h in (/works) return $h/employee[position() = last()]/s
    // Expected: Jane Doe 13
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn last_5() {
    //  Evaluation of 'fn:last' together with 'fn:position' (format last() = position()
    // XPath: for $h in (/works) return $h/employee[last() = position()]/s
    // Expected: Jane Doe 13
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn last_6() {
    //  Evaluation of 'fn:last' together with the '<<' node operator
    // XPath: for $h in (/works) return $h/employee[last()] << $h/employee
    // Expected: false
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn last_7() {
    //  Evaluation of 'fn:last' together with the 'is' node operator casted to integer
    // XPath: for $h in (/works) return $h/employee[last()] is $h/employee
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn last_9() {
    //  Evaluation of 'fn:last' where the last function is used inside a second predica
    // XPath: for $h in (/works) return $h//employee[@name=\"Jane Doe 13\"
    // Expected: Jane Doe 13
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn last_10() {
    //  Evaluation of 'fn:last' where the last function is used inside the first predic
    // XPath: for $h in (/works) return $h//employee[last() = 13][@name =
    // Expected: Jane Doe 13
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn last_11() {
    //  Evaluation of 'fn:position' used in conjunction with the fn:not function
    // XPath: for $h in (/works/employee[12]/overtime) return $h/day[not(p
    // Expected: Tuesday
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn last_13() {
    //  Evaluation of 'fn:position' used together with an addition operation.
    // XPath: for $h in (/works) return $h/employee[position() = (2 + 2)]/
    // Expected: John Doe 4
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn last_14() {
    //  Evaluation of 'fn:position' used together with a subtraction operation.
    // XPath: for $h in (/works) return $h/employee[position() = (5 - 2)]/
    // Expected: Jane Doe 3
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn last_15() {
    //  Evaluation of 'fn:position' used together with a multiplication operation.
    // XPath: for $h in (/works) return $h/employee[position() = (3 * 2)]/
    // Expected: John Doe 6
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn last_16() {
    //  Evaluation of 'fn:position' used together with a division (div) operation.
    // XPath: for $h in (/works) return $h/employee[position() = (6 div 2)
    // Expected: Jane Doe 3
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn last_17() {
    //  Evaluation of 'fn:position' used together with a division (idiv) operation.
    // XPath: for $h in (/works) return $h/employee[position() = (6 idiv 2
    // Expected: Jane Doe 3
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn last_24() {
    // Top-level call on last()
    // XPath: last()
    // Expected: 1
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn last_27() {
    //  simple node selection with last()
    // XPath: count(document{<A><B/></A>}//*[last() <= 1])
    // Expected: 2
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_2() {
    //  A test whose essence is: `(1, 2, 3)[if(1) then 1 else last()]`.
    // XPath: (1, 2, 3)[if(1) then 1 else last()]
    // Expected: 1
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_3() {
    //  A test whose essence is: `deep-equal((1, 2, 3)[count((last(), last())) eq 2], (
    // XPath: deep-equal((1, 2, 3)[count((last(), last())) eq 2], (1, 2, 3
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_4() {
    //  A test whose essence is: `deep-equal((1, 2, 3)[last() eq last()], (1, 2, 3))`.
    // XPath: deep-equal((1, 2, 3)[last() eq last()], (1, 2, 3))
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_5() {
    //  A test whose essence is: `(1, 2, 3, current-time())[last() - 1] treat as xs:int
    // XPath: (1, 2, 3, current-time())[last() - 1] treat as xs:integer eq
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_6() {
    //  A test whose essence is: `empty((1, 2, 3, current-time())[last() + 1])`.
    // XPath: empty((1, 2, 3, current-time())[last() + 1])
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_11() {
    //  fn:last() can never return 0('!=').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [last(
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_12() {
    //  fn:last() can never return 0('ne').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [last(
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_13() {
    //  fn:last() can never return 0('!=').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [0 !=
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_14() {
    //  fn:last() can never return 0('ne'), #2.
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [0 ne
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_19() {
    //  The return value of fn:last() is always greater than 0('ne').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [last(
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_20() {
    //  The return value of fn:last() is always greater than 0('>').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [last(
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_21() {
    //  The return value of fn:last() is always greater than 0('gt').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [last(
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_22() {
    //  The return value of fn:last() is always greater or equal to 0('>=').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [last(
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_23() {
    //  The return value of fn:last() is always greater or equal to 0('ge').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [last(
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_24() {
    //  fn:last() inside a predicate combined with a offset.
    // XPath: (1, 2, 3, 4, current-time(), 4, 5, 6)[last() - 2] treat as x
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_25() {
    //  fn:last() inside a predicate combined with a offset.
    // XPath: (1, 2, 3, 4, current-time(), 4, 5, 6)[last() - 1] treat as x
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_26() {
    //  fn:last() inside a predicate.
    // XPath: (1, 2, 3, 4, current-time(), 4, 5, 6)[last()] treat as xs:in
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_27() {
    //  fn:last() inside a predicate and an insignificant offset.
    // XPath: (1, 2, 3, 4, current-time(), 4, 5, 6)[last() - 0] treat as x
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_28() {
    //  fn:last() inside a predicate combined with a range expression.
    // XPath: (1 to 6)[last()] eq 6
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn k_contextlastfunc_29() {
    //  fn:last() inside a predicate combined with a range expression and offset.
    // XPath: (-20 to -5)[last() - 3]
    // Expected: -8
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn cbcl_last_in_sequence_001() {
    //  test fn:first-in-sequence on sub-expression where quant = 1
    // XPath: declare function local:generate($arg as xs:integer?) { if ($
    // Expected: 3
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn cbcl_last_in_sequence_002() {
    //  test fn:first-in-sequence on a for expression
    // XPath: declare function local:generate($arg as xs:integer?) as xs:i
    // Expected: 5
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn cbcl_last_in_sequence_003() {
    //  test fn:first-in-sequence on a for expression
    // XPath: declare function local:generate($arg as xs:integer?) as xs:i
    // Expected: 2
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn cbcl_last_in_sequence_004() {
    //  test fn:first-in-sequence on a for-at expression
    // XPath: declare function local:generate($arg as xs:integer?) as xs:i
    // Expected: 6
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn cbcl_last_in_sequence_005() {
    //  test fn:first-in-sequence on a for-at expression
    // XPath: declare function local:generate($arg as xs:integer?) as xs:i
    // Expected: 2
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn cbcl_last_in_sequence_006() {
    //  test fn:first-in-sequence on a map expression
    // XPath: declare function local:generate($arg as xs:integer?) as xs:s
    // Expected: c
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn cbcl_last_in_sequence_007() {
    //  test fn:first-in-sequence on a map expression
    // XPath: declare function local:generate($arg as xs:integer?) as xs:i
    // Expected: 0
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn cbcl_last_in_sequence_009() {
    //  test fn:first-in-sequence on a for-each expression Author: Tim Mills
    // XPath: declare function local:generate($arg as xs:integer?) as xs:s
    // Expected: 3
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn cbcl_last_in_sequence_010() {
    //  test fn:boolean on fn:first-in-sequence
    // XPath: declare function local:generate($arg as xs:integer?) as xs:s
    // Expected: true
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn cbcl_last_in_sequence_011() {
    //  test fn:boolean on fn:first-in-sequence
    // XPath: declare function local:generate($arg as xs:integer?) as xs:s
    // Expected: false
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}

#[test]
fn cbcl_last_in_sequence_012() {
    //  test fn:first-in-sequence on a for-each expression
    // XPath: declare function local:generate($arg as xs:boolean) as xs:st
    // Expected: 3
    // Calls stub function - will fail until fn:last is implemented
    let _ = stub_fnlast();
}
