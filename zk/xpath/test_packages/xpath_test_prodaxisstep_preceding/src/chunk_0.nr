//! Test chunk 0 for prod:AxisStep.preceding
//! Contains 20 tests

use dep::xpath::stub_prodaxisstep_preceding;

#[test]
fn preceding_2() {
    //  Evaluation of the preceding axis for which the given node does not exists. Uses
    // XPath: fn:count(/works/employee[1]/preceding::noSuchNode)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_3() {
    //  Evaluation of the preceding axis that is part of an 'is' expression (return tru
    // XPath: exactly-one(/works/employee[2]/preceding::employee) is exact
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_4() {
    //  Evaluation of the preceding axis that is part of an 'is' expression (return fal
    // XPath: exactly-one(/works[1]/employee[2]/preceding::employee) is ex
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_5() {
    //  Evaluation of the preceding axis that is part of an 'node before' expression (r
    // XPath: exactly-one(/works[1]/employee[2]/preceding::employee) << ex
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_6() {
    //  Evaluation of the preceding axis that is part of an 'node before' expression an
    // XPath: exactly-one(/works[1]/employee[2]/preceding::employee) << ex
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_7() {
    //  Evaluation of the preceding axis that is part of an 'node before' expression bo
    // XPath: exactly-one(/works[1]/employee[2]/preceding::employee) << ex
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_8() {
    //  Evaluation of the preceding axis that is part of a 'node after' expression (ret
    // XPath: exactly-one(/works[1]/employee[13]) >> exactly-one(/works[1]
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_9() {
    //  Evaluation of the preceding axis that is part of an 'node after' expression wit
    // XPath: exactly-one(/works[1]/employee[2]/preceding::employee) >> ex
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_10() {
    //  Evaluation of the preceding axis that is part of an 'node after' expression wit
    // XPath: exactly-one(/works[1]/employee[13]) >> exactly-one(/works[1]
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_14() {
    //  Evaluation of the preceding axis that is part of an 'except' operation. Both op
    // XPath: fn:count((/works[1]/employee[12]/preceding::employee) except
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_16() {
    //  Evaluation of the preceding axis that is part of a boolean expression ('and' an
    // XPath: (/works[1]/employee[12]/preceding::employee) and fn:true()
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_17() {
    //  Evaluation of the preceding axis that is part of a boolean expression ('and' an
    // XPath: (/works[1]/employee[12]/preceding::employee) and fn:false()
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_18() {
    //  Evaluation of the preceding axis that is part of a boolean expression ('or' and
    // XPath: (/works[1]/employee[12]/preceding::employee) or fn:true()
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_19() {
    //  Evaluation of the preceding axis that is part of a boolean expression ('or' and
    // XPath: (/works[1]/employee[12]/preceding::employee) or fn:false()
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_20() {
    //  Evaluation of the preceding axis that used as part of the deep-equal-function.
    // XPath: fn:deep-equal(/works[1]/employee[12]/preceding::employee,/wo
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn preceding_21() {
    //  Evaluation of the preceding axis used together with a newly constructed element
    // XPath: let $var := <anElement>Some Content</anElement> return fn:co
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn k2_precedingaxis_1() {
    //  Evaluate the child node from the last node in a tree.
    // XPath: declare variable $i := <root> <child/> <child/> <child> <chi
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn k2_precedingaxis_2() {
    //  Evaluate from a node that has no preceding nodes.
    // XPath: declare variable $i := <root> <child> <child2> <child3> <lea
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn k2_precedingaxis_7() {
    //  Apply fn:count() to axis preceding.
    // XPath: declare variable $i := <root> <child> <preceding2/> <child2>
    // Expected: 2
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}

#[test]
fn k2_precedingaxis_8() {
    //  Apply fn:count() to axis preceding.
    // XPath: declare variable $i := <root> <child> <preceding2/> <child2>
    // Expected: 0 0 0 1 1 2 2 4 6 8
    // Calls stub function - will fail until prod:AxisStep.preceding is implemented
    let _ = stub_prodaxisstep_preceding();
}
