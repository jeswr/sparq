//! Test chunk 0 for prod:AxisStep.following
//! Contains 20 tests

use dep::xpath::stub_prodaxisstep_following;

#[test]
fn following_2() {
    //  Evaluation of the following axis for which the given node does not exists. Uses
    // XPath: fn:count(/works/employee[1]/following::noSuchNode)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_3() {
    //  Evaluation of the following axis that is part of an 'is' expression (return tru
    // XPath: exactly-one(/works/employee[12]/following::employee) is exac
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_4() {
    //  Evaluation of the following axis that is part of an 'is' expression (return fal
    // XPath: exactly-one(/works[1]/employee[12]/following::employee) is e
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_5() {
    //  Evaluation of the following axis that is part of an 'node before' expression (r
    // XPath: (/works[1]/employee[11]/following::employee[1]) << (/works[1
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_6() {
    //  Evaluation of the following axis that is part of an 'node before' expression an
    // XPath: exactly-one(/works[1]/employee[12]/following::employee) << e
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_7() {
    //  Evaluation of the following axis that is part of an 'node before' expression bo
    // XPath: exactly-one(/works[1]/employee[12]/following::employee) << e
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_8() {
    //  Evaluation of the following axis that is part of an 'node after' expression (re
    // XPath: exactly-one(/works[1]/employee[13]) >> exactly-one(/works[1]
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_9() {
    //  Evaluation of the following axis that is part of an 'node after' expression wit
    // XPath: exactly-one(/works[1]/employee[12]/following::employee) >> e
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_10() {
    //  Evaluation of the following axis that is part of an 'node after' expression wit
    // XPath: exactly-one(/works[1]/employee[12]) >> exactly-one(/works[1]
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_14() {
    //  Evaluation of the following axis that is part of an 'except' operation. Both op
    // XPath: fn:count((/works[1]/employee[12]/following::employee) except
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_16() {
    //  Evaluation of the following axis that is part of a boolean expression ('and' an
    // XPath: (/works[1]/employee[12]/following::employee) and fn:true()
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_17() {
    //  Evaluation of the following axis that is part of a boolean expression ('and' an
    // XPath: (/works[1]/employee[12]/following::employee) and fn:false()
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_18() {
    //  Evaluation of the following axis that is part of a boolean expression ('or' and
    // XPath: (/works[1]/employee[12]/following::employee) or fn:true()
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_19() {
    //  Evaluation of the following axis that is part of a boolean expression ('or' and
    // XPath: (/works[1]/employee[12]/following::employee) or fn:false()
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_20() {
    //  Evaluation of the following axis that used as part of the deep-equal-function.
    // XPath: fn:deep-equal(/works[1]/employee[12]/following::employee,/wo
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_21() {
    //  Evaluation of the following axis used together with a newly constructed element
    // XPath: let $var := <anElement>Some Content</anElement> return fn:co
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn following_22() {
    // Use following::text(). Saxon bug 4149.
    // XPath: exists(//employee[@name=\"John Doe 10\"]/following::text()[.
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn k2_followingaxis_2() {
    //  Evaluate the child node from the last node in a tree.
    // XPath: declare variable $i := <root> <child/> <child/> <child> <chi
    // Expected: 1 1
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn k2_followingaxis_3() {
    //  Evaluate the child node from root of a tree.
    // XPath: declare variable $i := <root> <child/> <child/> <child> <chi
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}

#[test]
fn k2_followingaxis_4() {
    //  Evaluate fn:count() on a range of nodes, navigated with axis following.
    // XPath: declare variable $i := <root> <child/> <child/> <child> <chi
    // Expected: 0 5 4 0 0 0 0
    // Calls stub function - will fail until prod:AxisStep.following is implemented
    let _ = stub_prodaxisstep_following();
}
