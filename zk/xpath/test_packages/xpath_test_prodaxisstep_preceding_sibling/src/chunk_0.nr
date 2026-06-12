//! Test chunk 0 for prod:AxisStep.preceding-sibling
//! Contains 22 tests

use dep::xpath::stub_prodaxisstep_preceding_sibling;

#[test]
fn preceding_sibling_2() {
    //  Evaluation of the preceding-sibling axis for which the given node does not exis
    // XPath: fn:count(/works/employee[1]/preceding-sibling::noSuchNode)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_3() {
    //  Evaluation of the preceding-sibling axis that is part of an 'is' expression (re
    // XPath: exactly-one(/works/employee[2]/preceding-sibling::employee)
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_4() {
    //  Evaluation of the preceding-sibling axis that is part of an 'is' expression (re
    // XPath: exactly-one(/works[1]/employee[2]/preceding-sibling::employe
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_5() {
    //  Evaluation of the preceding-sibling axis that is part of an 'node before' expre
    // XPath: exactly-one(/works[1]/employee[2]/preceding-sibling::employe
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_6() {
    //  Evaluation of the preceding-sibling axis that is part of an 'node before' expre
    // XPath: exactly-one(/works[1]/employee[2]/preceding-sibling::employe
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_7() {
    //  Evaluation of the preceding-sibling axis that is part of an 'node before' expre
    // XPath: exactly-one(/works[1]/employee[2]/preceding-sibling::employe
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_8() {
    //  Evaluation of the preceding-sibling axis that is part of a 'node after' express
    // XPath: exactly-one(/works[1]/employee[13]) >> exactly-one(/works[1]
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_9() {
    //  Evaluation of the preceding-sibling axis that is part of an 'node after' expres
    // XPath: exactly-one(/works[1]/employee[2]/preceding-sibling::employe
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_10() {
    //  Evaluation of the preceding-sibling axis that is part of an 'node after' expres
    // XPath: exactly-one(/works[1]/employee[13]) >> exactly-one(/works[1]
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_14() {
    //  Evaluation of the preceding-sibling axis that is part of an 'except' operation.
    // XPath: fn:count((/works[1]/employee[12]/preceding-sibling::employee
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_16() {
    //  Evaluation of the preceding-sibling axis that is part of a boolean expression (
    // XPath: (/works[1]/employee[12]/preceding-sibling::employee) and fn:
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_17() {
    //  Evaluation of the preceding-sibling axis that is part of a boolean expression (
    // XPath: (/works[1]/employee[12]/preceding-sibling::employee) and fn:
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_18() {
    //  Evaluation of the preceding-sibling axis that is part of a boolean expression (
    // XPath: (/works[1]/employee[12]/preceding-sibling::employee) or fn:t
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_19() {
    //  Evaluation of the preceding-sibling axis that is part of a boolean expression (
    // XPath: (/works[1]/employee[12]/preceding-sibling::employee) or fn:f
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_20() {
    //  Evaluation of the preceding-sibling axis that used as part of the deep-equal-fu
    // XPath: fn:deep-equal(/works[1]/employee[12]/preceding-sibling::empl
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_21() {
    //  Evaluation of the preceding-sibling axis used together with a newly constructed
    // XPath: let $var := <anElement>Some Content</anElement> return fn:co
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn k2_preceding_siblingaxis_1() {
    //  Iterate from the root node.
    // XPath: declare variable $i := <root> <child> <preceding2/> <child2>
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn k2_preceding_siblingaxis_2() {
    //  Iterate from the root node.
    // XPath: declare variable $i := <root> <child> <preceding2/> <child2>
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn k2_preceding_siblingaxis_3() {
    //  Apply fn:count() to the result of axis preceding.
    // XPath: declare variable $i := <root> <child> <preceding2/> <child2>
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn k2_preceding_siblingaxis_4() {
    //  Apply fn:count() to the result of axis preceding.
    // XPath: declare variable $i := <root> <child> <preceding2/> <child2>
    // Expected: 2
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn k2_preceding_siblingaxis_5() {
    //  Apply fn:count() to the result of axis preceding.
    // XPath: declare variable $i := <root> <child> <preceding2/> <child2>
    // Expected: 0 0 0 1 0 1 0 2 2 1
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}

#[test]
fn preceding_sibling_attr() {
    //  Apply the preceding-sibling axis to an attribute node.
    // XPath: <foo a='1' b='2' c='3'> <bar>4</bar> <bar>5</bar> <bar>6</ba
    // Expected:
    // Calls stub function - will fail until prod:AxisStep.preceding-sibling is implemented
    let _ = stub_prodaxisstep_preceding_sibling();
}
