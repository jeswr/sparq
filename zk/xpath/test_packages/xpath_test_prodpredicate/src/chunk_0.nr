//! Test chunk 0 for prod:Predicate
//! Contains 50 tests

use dep::xpath::stub_prodpredicate;

#[test]
fn filterexpressionhc2() {
    //  Simple filter expression involving integers. Return integer from 1 to 25 divisi
    // XPath: ((1 to 25)[. mod 2 eq 0])
    // Expected: 2 4 6 8 10 12 14 16 18 20 22 24
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc3() {
    //  Simple filter expression that returns a sigle number
    // XPath: ((1 to 25)[25])
    // Expected: 25
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc6() {
    //  Simple filter expression involving the ge operator
    // XPath: ((1 to 25)[. ge 10])
    // Expected: 10 11 12 13 14 15 16 17 18 19 20 21 22 2
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc7() {
    //  Simple filter expression involving the lt opertor
    // XPath: ((1 to 25)[. lt 10])
    // Expected: 1 2 3 4 5 6 7 8 9
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc8() {
    //  Simple filter expression involving the le operator
    // XPath: ((1 to 25)[. le 10])
    // Expected: 1 2 3 4 5 6 7 8 9 10
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc9() {
    //  Simple filter expression involving the eq operator
    // XPath: ((1 to 25)[. eq 10])
    // Expected: 10
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc10() {
    //  Simple filter expression involving the ne operator
    // XPath: ((1 to 11)[. ne 10])
    // Expected: 1 2 3 4 5 6 7 8 9 11
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc11() {
    //  Simple filter expression involving a boolean 'and' expression
    // XPath: ((1 to 11)[(. eq 10) and (. mod 5 eq 0)])
    // Expected: 10
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc12() {
    //  Simple filter expression involving a boolean 'or' expression
    // XPath: ((1 to 11)[(. eq 10) or (. eq 5)])
    // Expected: 5 10
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc13() {
    //  Simple filter expression involving a division expression
    // XPath: ((1,2,4,5,6,7,8,9,10,11)[(. div 2 eq 5)])
    // Expected: 10
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc14() {
    //  Simple filter expression involving a multiplication expression
    // XPath: ((1,2,4,5,6,7,8,9,10,11)[(. * 2 eq 10)])
    // Expected: 5
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc15() {
    //  Simple filter expression involving an addition expression
    // XPath: ((1,2,4,5,6,7,8,9,10,11)[(. + 2 eq 10)])
    // Expected: 8
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc16() {
    //  Simple filter expression involving a subtration expression
    // XPath: ((1,2,4,5,6,7,8,9,10,11)[(. - 2 eq 6)])
    // Expected: 8
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc17() {
    //  Simple filter expression involving an integer division (idiv) expression
    // XPath: ((1,2,4,5,6,7,8,9,10,11)[(. idiv 2 eq 3)])
    // Expected: 6 7
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc18() {
    //  Simple filter expression using an xs:string function
    // XPath: ((1,2,3,4,5,6,7,8,9,10,11)[(xs:string(.) eq \"3\")])
    // Expected: 3
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc19() {
    //  Simple filter expression involving two simple predicates
    // XPath: ((1,2,3,4,5,6,7,8,9,10,11)[. gt 1][. gt 5])
    // Expected: 6 7 8 9 10 11
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc20() {
    //  Simple filter expression involving two complex predicates
    // XPath: ((1,2,3,4,5,6,7,8,9,10,11)[(. gt 1) and (. gt 2)][(. gt 5) a
    // Expected: 7 8 9 10 11
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn filterexpressionhc21() {
    //  Simple filter expression involving the 'fn:true' function
    // XPath: ((1,2,3,4,5,6,7,8,9,10,11)[fn:true()])
    // Expected: 1 2 3 4 5 6 7 8 9 10 11
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_6() {
    //  A test whose essence is: `empty((1, 2, 3)[0])`.
    // XPath: empty((1, 2, 3)[0])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_7() {
    //  A test whose essence is: `empty((1, 2, 3)[4])`.
    // XPath: empty((1, 2, 3)[4])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_8() {
    //  A test whose essence is: `empty((1, 2, 3)[0])`.
    // XPath: empty((1, 2, 3)[0])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_9() {
    //  A test whose essence is: `empty((1, 2, 3)[4])`.
    // XPath: empty((1, 2, 3)[4])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_10() {
    //  A test whose essence is: `empty((1, 2, 3)[0.1])`.
    // XPath: empty((1, 2, 3)[0.1])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_11() {
    //  A test whose essence is: `empty((1, 2, 3)[1.1])`.
    // XPath: empty((1, 2, 3)[1.1])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_12() {
    //  A test whose essence is: `empty((1, 2, 3)[1.01])`.
    // XPath: empty((1, 2, 3)[1.01])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_13() {
    //  A test whose essence is: `empty((1, 2, 3)[4])`.
    // XPath: empty((1, 2, 3)[4])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_14() {
    //  A test whose essence is: `empty((1, 2, 3)[4.1])`.
    // XPath: empty((1, 2, 3)[4.1])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_15() {
    //  A test whose essence is: `empty((1, 2, 3)[4.01])`.
    // XPath: empty((1, 2, 3)[4.01])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_16() {
    //  A test whose essence is: `not(empty((1, 2, 3)[1]))`.
    // XPath: not(empty((1, 2, 3)[1]))
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_17() {
    //  A test whose essence is: `not(empty((1, 2, 3)[3]))`.
    // XPath: not(empty((1, 2, 3)[3]))
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_18() {
    //  A test whose essence is: `(1, 2, 3)[1] eq 1`.
    // XPath: (1, 2, 3)[1] eq 1
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_19() {
    //  A test whose essence is: `(1, 2, 3)[1.0] eq 1`.
    // XPath: (1, 2, 3)[1.0] eq 1
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_20() {
    //  A test whose essence is: `(1, 2, 3)[1.0e0] eq 1`.
    // XPath: (1, 2, 3)[1.0e0] eq 1
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_21() {
    //  A test whose essence is: `(1, 2, 3)[3] eq 3`.
    // XPath: (1, 2, 3)[3] eq 3
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_22() {
    //  A test whose essence is: `(1, 2, 3)[3.0] eq 3`.
    // XPath: (1, 2, 3)[3.0] eq 3
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_23() {
    //  A test whose essence is: `(1, 2, 3)[3.0e0] eq 3`.
    // XPath: (1, 2, 3)[3.0e0] eq 3
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_24() {
    //  A test whose essence is: `(0, 1, 2)[1] eq 0`.
    // XPath: (0, 1, 2)[1] eq 0
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_25() {
    //  A test whose essence is: `(0, 1, 2)[2] eq 1`.
    // XPath: (0, 1, 2)[2] eq 1
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_26() {
    //  A test whose essence is: `(0, 1, 2)[3] eq 2`.
    // XPath: (0, 1, 2)[3] eq 2
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_27() {
    //  A test whose essence is: `(0)[1] eq 0`.
    // XPath: (0)[1] eq 0
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_28() {
    //  A test whose essence is: `0[1] eq 0`.
    // XPath: 0[1] eq 0
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_29() {
    //  A test whose essence is: `(0, 1)[1] eq 0`.
    // XPath: (0, 1)[1] eq 0
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_31() {
    //  A test whose essence is: `deep-equal((1, 2, 3), (1, 2, 3)[true()])`.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3)[true()])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_32() {
    //  A test whose essence is: `deep-equal((1, 2, 3), (1, 2, 3)[. instance of xs:inte
    // XPath: deep-equal((1, 2, 3), (1, 2, 3)[. instance of xs:integer])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_33() {
    //  The context item is used as the predicate, leading to a truth predicate.
    // XPath: deep-equal((true(), true(), true()), (false(), true(), true(
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_34() {
    //  The context item is used as the predicate, leading to a numeric predicate.
    // XPath: deep-equal((2, 3, 4, 5, 7, 8, 9), (0, 2, 3, 4, 5, 5, 7, 8, 1
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_35() {
    //  Source expression of a predicate doesn't have to use paranteses.
    // XPath: 1[true()] eq 1
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_36() {
    //  Use fn:current-time() inside a predicate.
    // XPath: (1, current-time())[1]
    // Expected: 1
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_37() {
    //  One xs:anyURI value is a valid predicate.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3)[xs:anyURI(\"example.com/\")]
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_39() {
    //  xs:untypedAtomic literal as predicate.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3)[xs:untypedAtomic(\"content\"
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}
