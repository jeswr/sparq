//! Test chunk 0 for fn:position
//! Contains 50 tests

use dep::xpath::stub_fnposition;

#[test]
fn position_1() {
    //  Evaluation of 'fn'position', where context function is an element node and posi
    // XPath: for $h in (/works) return $h/employee[position() = 1]/string
    // Expected: Jane Doe 1
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn position_3() {
    //  Evaluation of 'fn'position', where the last item is selected.
    // XPath: for $h in (/works) return $h/employee[position() = 13]/strin
    // Expected: Jane Doe 13
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn position_4() {
    //  Evaluation of 'fn'position' together with 'fn:last'.
    // XPath: for $h in (/works) return $h/employee[position() = last()]/s
    // Expected: Jane Doe 13
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn position_5() {
    //  Evaluation of 'fn:position' together with 'fn:last' (format last() = position()
    // XPath: for $h in (/works) return $h/employee[last() = position()]/s
    // Expected: Jane Doe 13
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn position_6() {
    //  Evaluation of 'fn'position' together with a variable of type integer
    // XPath: for $var in 1 return for $h in (/works) return $h/employee[p
    // Expected: Jane Doe 1
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn position_7() {
    //  Evaluation of 'fn'position' together with a variable of type string casted to i
    // XPath: for $var in \"1\" return for $h in (/works) return $h/employ
    // Expected: Jane Doe 1
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn position_9() {
    //  Evaluation of 'fn:position' where the position function is used inside a second
    // XPath: for $h in (/works) return $h/employee[@name][position() = 1]
    // Expected: Jane Doe 1
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn position_10() {
    //  Evaluation of 'fn:position' where the position function is used inside the firs
    // XPath: for $h in (/works) return $h/employee[position() = 1][@name]
    // Expected: Jane Doe 1
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn position_11() {
    //  Evaluation of 'fn:position' used in conjucntion with the fn:not function
    // XPath: for $h in (/works/employee[12]/overtime) return $h/day[not(p
    // Expected: Tuesday
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn position_13() {
    //  Evaluation of 'fn:position' used together with an addition operation.
    // XPath: for $h in (/works) return $h/employee[position() = (2 + 2)]/
    // Expected: John Doe 4
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn position_14() {
    //  Evaluation of 'fn:position' used together with a subtraction operation.
    // XPath: for $h in (/works) return $h/employee[position() = (5 - 2)]/
    // Expected: Jane Doe 3
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn position_15() {
    //  Evaluation of 'fn:position' used together with a multiplication operation.
    // XPath: for $h in (/works) return $h/employee[position() = (3 * 2)]/
    // Expected: John Doe 6
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn position_16() {
    //  Evaluation of 'fn:position' used together with a division (div) operation.
    // XPath: for $h in (/works) return $h/employee[position() = (6 div 2)
    // Expected: Jane Doe 3
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn position_17() {
    //  Evaluation of 'fn:position' used together with a division (idiv) operation.
    // XPath: for $h in (/works) return $h/employee[position() = (6 idiv 2
    // Expected: Jane Doe 3
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn position_23() {
    // Top-level call on position()
    // XPath: position()
    // Expected: 1
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_2() {
    //  A test whose essence is: `(1, 2, 3)[if(1) then 1 else position()]`.
    // XPath: (1, 2, 3)[if(1) then 1 else position()]
    // Expected: 1
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_3() {
    //  fn:position() can never return 0('=').
    // XPath: empty((1, 2, 3, current-time(), current-date(), 6, 7, 8) [po
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_4() {
    //  fn:position() can never return 0('='), #2.
    // XPath: empty((1, 2, 3, current-time(), current-date(), 6, 7, 8) [0
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_5() {
    //  fn:position() can never return 0('eq').
    // XPath: empty((1, 2, 3, current-time(), current-date(), 6, 7, 8) [po
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_6() {
    //  fn:position() can never return 0('eq'), #2.
    // XPath: empty((1, 2, 3, current-time(), current-date(), 6, 7, 8) [0
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_7() {
    //  fn:position() can never return 0('!=').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [posit
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_8() {
    //  fn:position() can never return 0('ne').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [posit
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_9() {
    //  fn:position() can never return 0('!='), #2.
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [0 !=
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_10() {
    //  fn:position() can never return 0('ne'), #2.
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [0 ne
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_11() {
    //  fn:position() can never return anything less than 1(lt).
    // XPath: empty((1, 2, 3, current-time(), current-date(), 6, 7, 8) [po
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_12() {
    //  fn:position() can never return anything less than 1(<).
    // XPath: empty((1, 2, 3, current-time(), current-date(), 6, 7, 8) [po
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_13() {
    //  fn:position() can never return anything less or equal to 0(le).
    // XPath: empty((1, 2, 3, current-time(), current-date(), 6, 7, 8) [po
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_14() {
    //  fn:position() can never return anything less or equal to 0(<=).
    // XPath: empty((1, 2, 3, current-time(), current-date(), 6, 7, 8) [po
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_15() {
    //  The return value of fn:position() is always greater than 0('ne').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [posit
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_16() {
    //  The return value of fn:position() is always greater than 0('>').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [posit
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_17() {
    //  The return value of fn:position() is always greater than 0('gt').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [posit
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_18() {
    //  The return value of fn:position() is always greater or equal to 0('>=').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [posit
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_19() {
    //  The return value of fn:position() is always greater or equal to 0('ge').
    // XPath: deep-equal( (1, 2, 3, remove((current-time(), 4), 1)) [posit
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_20() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: 1 eq (0, 1, current-time(), 4)[position() = 2] treat as xs:i
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_21() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: 1 eq (0, 1, current-time(), 4)[position() eq 2] treat as xs:
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_22() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: 1 eq (0, 1, current-time(), 4)[2 eq position()] treat as xs:
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_23() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: 1 eq (0, 1, current-time(), 4)[2 = position()] treat as xs:i
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_24() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((1, 2, 4), (1, 2, current-time(), 4)[position() !
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_25() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((1, 2, 4), (1, 2, current-time(), 4)[position() n
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_26() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((1, 2, 4), (1, 2, current-time(), 4)[3 ne positio
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_27() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((1, 2, 4), (1, 2, current-time(), 4)[3 != positio
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_28() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3, current-time())[position() l
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_29() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3, current-time())[position() <
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_30() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3, current-time())[4 gt positio
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_31() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3, current-time())[4 > position
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_32() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3, current-time())[position() l
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_33() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3, current-time())[position() <
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_34() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3, current-time())[3 ge positio
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_35() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3, current-time())[3 >= positio
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}

#[test]
fn k_contextpositionfunc_36() {
    //  position() combined with a comparison operator inside a predicate.
    // XPath: deep-equal((4, 5), (1, 2, current-time(), 4, 5)[3 lt positio
    // Expected: true
    // Calls stub function - will fail until fn:position is implemented
    let _ = stub_fnposition();
}
