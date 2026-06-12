//! Test chunk 0 for prod:AxisStep.following-sibling
//! Contains 22 tests

use dep::xpath::stub_prodaxisstep_following_sibling;

#[test]
fn followingsibling_2() {
    //  Evaluation of the following-sibling axis for which the given node does not exis
    // XPath: fn:count(/works/employee[1]/following-sibling::noSuchNode)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_3() {
    //  Evaluation of the following-sibling axis that is part of an 'is' expression (re
    // XPath: exactly-one(/works/employee[12]/following-sibling::employee)
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_4() {
    //  Evaluation of the following-sibling axis that is part of an 'is' expression (re
    // XPath: exactly-one(/works[1]/employee[12]/following-sibling::employ
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_5() {
    //  Evaluation of the following-sibling axis that is part of an 'node before' expre
    // XPath: exactly-one(/works[1]/employee[12]/overtime/day[1]/following
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_6() {
    //  Evaluation of the following-sibling axis that is part of an 'node before' expre
    // XPath: exactly-one(/works[1]/employee[12]/following-sibling::employ
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_7() {
    //  Evaluation of the following-sibling axis that is part of an 'node before' expre
    // XPath: exactly-one(/works[1]/employee[12]/following-sibling::employ
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_8() {
    //  Evaluation of the following-sibling axis that is part of an 'node after' expres
    // XPath: exactly-one(/works[1]/employee[13]) >> exactly-one(/works[1]
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_9() {
    //  Evaluation of the following-sibling axis that is part of an 'node after' expres
    // XPath: exactly-one(/works[1]/employee[12]/following-sibling::employ
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_10() {
    //  Evaluation of the following-sibling axis that is part of an 'node after' expres
    // XPath: exactly-one(/works[1]/employee[12]) >> exactly-one(/works[1]
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_14() {
    //  Evaluation of the following-sibling axis that is part of an 'except' operation.
    // XPath: fn:count((/works[1]/employee[12]/following-sibling::employee
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_16() {
    //  Evaluation of the following-sibling axis that is part of a boolean expression (
    // XPath: (/works[1]/employee[12]/following-sibling::employee) and fn:
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_17() {
    //  Evaluation of the following-sibling axis that is part of a boolean expression (
    // XPath: (/works[1]/employee[12]/following-sibling::employee) and fn:
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_18() {
    //  Evaluation of the following-sibling axis that is part of a boolean expression (
    // XPath: (/works[1]/employee[12]/following-sibling::employee) or fn:t
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_19() {
    //  Evaluation of the following-sibling axis that is part of a boolean expression (
    // XPath: (/works[1]/employee[12]/following-sibling::employee) or fn:f
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_20() {
    //  Evaluation of the following-sibling axis that used as part of the deep-equal-fu
    // XPath: fn:deep-equal(/works[1]/employee[12]/following-sibling::empl
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn followingsibling_21() {
    //  Evaluation of the following axis used together with a newly constructed element
    // XPath: let $var := <anElement>Some Content</anElement> return fn:co
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn k2_following_siblingaxis_3() {
    //  Apply fn:count() to the result of axis following-sibling.
    // XPath: count(<root> <child/> <child/> <child attr=\"foo\" attr2=\"f
    // Expected: 2
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn k2_following_siblingaxis_4() {
    //  Evaluate fn:count() on a range of nodes, navigated with axis following.
    // XPath: declare variable $i := <root> <child/> <child/> <child> <chi
    // Expected: 0 2 1 0 0 0 0
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn k2_following_siblingaxis_7() {
    //  Apply count to axis following-sibling.
    // XPath: count(<root> <child/> </root>/following-sibling::node())
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn k2_following_siblingaxis_8() {
    //  Apply count to axis following-sibling, combined with the comma operator.
    // XPath: 1, <root> <child/> </root>/following-sibling::node(), 1
    // Expected: 1 1
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn k2_following_siblingaxis_9() {
    //  Apply count to axis following-sibling to a single node, combined with the comma
    // XPath: 1, <root/>/following-sibling::node(), 1
    // Expected: 1 1
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}

#[test]
fn following_sibling_attr() {
    //  Apply the following-sibling axis to an attribute node.
    // XPath: <foo a='1' b='2' c='3'> <bar>4</bar> <bar>5</bar> <bar>6</ba
    // Expected:
    // Calls stub function - will fail until prod:AxisStep.following-sibling is implemented
    let _ = stub_prodaxisstep_following_sibling();
}
