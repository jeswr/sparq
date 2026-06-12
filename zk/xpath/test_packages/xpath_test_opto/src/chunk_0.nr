//! Test chunk 0 for op:to
//! Contains 50 tests

use dep::xpath::stub_opto;

#[test]
fn rangeexpr_1() {
    //  Evaluation of a single range expression using positive integers.
    // XPath: (10, 1 to 4)
    // Expected: 10 1 2 3 4
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_2() {
    //  Evaluation of a range expression of length one containing the single integer 10
    // XPath: 10 to 10
    // Expected: 10
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_3() {
    //  Evaluation of a range expression that results in a sequence of length 0. Uses '
    // XPath: fn:count(15 to 10)
    // Expected: 0
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_4() {
    //  Evaluation of a range expression that uses the 'reverse' function.
    // XPath: fn:reverse(10 to 15)
    // Expected: 15 14 13 12 11 10
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_6() {
    //  Evaluation of a range expression, where both operands are addition operations.
    // XPath: ((1+2) to (2+2))
    // Expected: 3 4
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_7() {
    //  Evaluation of a range expression, where the first operand are negative number.
    // XPath: (-4,-3 to 2)
    // Expected: -4 -3 -2 -1 0 1 2
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_8() {
    //  Evaluation of a range expression, where both operands are negative integers.
    // XPath: (-4, -3 to -1)
    // Expected: -4 -3 -2 -1
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_9() {
    //  Evaluation of a range expression, where the first operand is 'xs:integer' funct
    // XPath: (xs:integer(1) to 5)
    // Expected: 1 2 3 4 5
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_10() {
    //  Evaluation of a range expression, where the second operand is 'xs:integer' func
    // XPath: (1 to xs:integer(5))
    // Expected: 1 2 3 4 5
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_11() {
    //  Evaluation of a range expression, where both operands are 'xs:integer' function
    // XPath: (xs:integer(1) to xs:integer(5))
    // Expected: 1 2 3 4 5
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_12() {
    //  Evaluation of a range expression, using the 'fn:min' function
    // XPath: (fn:min((1,2)) to 5)
    // Expected: 1 2 3 4 5
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_13() {
    //  Evaluation of a range expression, using the 'fn:max' function
    // XPath: (fn:max((1,2)) to 5)
    // Expected: 2 3 4 5
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_14() {
    //  Evaluation of a range expression, using the 'fn:min' and 'fn:max' functions
    // XPath: (fn:min((1,2)) to fn:max((6,7)))
    // Expected: 1 2 3 4 5 6 7
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_15() {
    //  Evaluation of a range expression as an argument to a 'fn:min' function)
    // XPath: fn:min((1 to 5))
    // Expected: 1
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_16() {
    //  Evaluation of a range expression as an argument to a 'fn:max' function)
    // XPath: fn:max((1 to 5))
    // Expected: 5
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_17() {
    //  Evaluation of a range expression as an argument to an 'fn:avg' function)
    // XPath: fn:avg((1 to 5))
    // Expected: 3
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_18() {
    //  Evaluation of a range expression as an argument to an 'fn:count' function)
    // XPath: fn:count((1 to 5))
    // Expected: 5
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_19() {
    //  Evaluation of a range expression, where the first operand is a multiplication o
    // XPath: ((3*2) to 10)
    // Expected: 6 7 8 9 10
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_20() {
    //  Evaluation of a range expression, where the second operand is a multiplication
    // XPath: (1 to (3*2))
    // Expected: 1 2 3 4 5 6
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_21() {
    //  Evaluation of a range expression, where both operands are multiplication operat
    // XPath: ((1*2) to (3*2))
    // Expected: 2 3 4 5 6
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_22() {
    //  Evaluation of a range expression, where the first operand is a subtraction oper
    // XPath: ((3 - 2) to 10)
    // Expected: 1 2 3 4 5 6 7 8 9 10
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_23() {
    //  Evaluation of a range expression, where the second operand is a subtraction ope
    // XPath: (1 to (3 - 2))
    // Expected: 1
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_24() {
    //  Evaluation of a range expression, where both operands are subtraction operation
    // XPath: ((2 - 1) to (7 - 1))
    // Expected: 1 2 3 4 5 6
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_25() {
    //  Evaluation of a range expression, where the first operand is a division operati
    // XPath: ((6 idiv 2) to 10)
    // Expected: 3 4 5 6 7 8 9 10
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_26() {
    //  Evaluation of a range expression, where the second operand is a division operat
    // XPath: (1 to (10 idiv 2))
    // Expected: 1 2 3 4 5
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_27() {
    //  Evaluation of a range expression, where both operands are division operations.
    // XPath: ((5 idiv 5) to (8 idiv 2))
    // Expected: 1 2 3 4
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_34() {
    // 'to' operator binds more tightly than '||'
    // XPath: \"it is \" || 10 to 1 || \"already\"
    // Expected: "it is already"
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_36() {
    // General comparison involving range expressions
    // XPath: year-from-date(current-date()) = 1 to 5000
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_37() {
    // General comparison involving range expressions
    // XPath: year-from-date(current-date()) = 5000 to 10000
    // Expected: false
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_38() {
    // General comparison involving range expressions: many-to-many
    // XPath: year-from-date(current-date()) to 4000 = 1500 to 2500
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_39() {
    // General comparison involving range expressions: empty endpoint to range
    // XPath: 1000 = 1 to year-from-date(current-date())[. gt 3000]
    // Expected: false
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_40() {
    // General comparison involving range expressions: empty operand
    // XPath: 1 to 5000 = year-from-date(current-date())[. gt 3000]
    // Expected: false
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn rangeexpr_42() {
    // General comparison involving range expressions in a predicate with singleton ran
    // XPath: (1 to 5000)[position() = 2000 to round(year-from-date(curren
    // Expected: 2000
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_2() {
    //  A test whose essence is: `empty(30 to 3)`.
    // XPath: empty(30 to 3)
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_3() {
    //  A test whose essence is: `empty(0 to -3)`.
    // XPath: empty(0 to -3)
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_6() {
    //  A test whose essence is: `empty(-1 to -3)`.
    // XPath: empty(-1 to -3)
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_7() {
    //  A test whose essence is: `count(1 to 4) eq 4`.
    // XPath: count(1 to 4) eq 4
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_8() {
    //  A test whose essence is: `count(0 to 4) eq 5`.
    // XPath: count(0 to 4) eq 5
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_9() {
    //  A test whose essence is: `count(-5 to -0) eq 6`.
    // XPath: count(-5 to -0) eq 6
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_10() {
    //  A test whose essence is: `count((10, 1 to 4)) eq 5`.
    // XPath: count((10, 1 to 4)) eq 5
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_11() {
    //  A test whose essence is: `subsequence(-3 to -1, 1, 1) eq -3`.
    // XPath: subsequence(-3 to -1, 1, 1) eq -3
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_12() {
    //  A test whose essence is: `subsequence(-3 to -1, 3, 1) eq -1`.
    // XPath: subsequence(-3 to -1, 3, 1) eq -1
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_13() {
    //  A test whose essence is: `empty(reverse(4 to 1))`.
    // XPath: empty(reverse(4 to 1))
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_14() {
    //  A test whose essence is: `subsequence(reverse(1 to 3), 1, 1) eq 3`.
    // XPath: subsequence(reverse(1 to 3), 1, 1) eq 3
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_15() {
    //  A test whose essence is: `subsequence(reverse(1 to 3), 3, 1) eq 1`.
    // XPath: subsequence(reverse(1 to 3), 3, 1) eq 1
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_16() {
    //  A test whose essence is: `subsequence(reverse(1 to 4), 2, 1) eq 3`.
    // XPath: subsequence(reverse(1 to 4), 2, 1) eq 3
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_17() {
    //  A test whose essence is: `subsequence(reverse(1 to 4), 3, 1) eq 2`.
    // XPath: subsequence(reverse(1 to 4), 3, 1) eq 2
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_18() {
    //  A test whose essence is: `subsequence(reverse(-4 to -1), 2, 1) eq -2`.
    // XPath: subsequence(reverse(-4 to -1), 2, 1) eq -2
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_19() {
    //  A test whose essence is: `deep-equal((-1, -2, -3, -4), reverse(-4 to -1))`.
    // XPath: deep-equal((-1, -2, -3, -4), reverse(-4 to -1))
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}

#[test]
fn k_rangeexpr_20() {
    //  A test whose essence is: `deep-equal((), reverse(0 to -5))`.
    // XPath: deep-equal((), reverse(0 to -5))
    // Expected: true
    // Calls stub function - will fail until op:to is implemented
    let _ = stub_opto();
}
