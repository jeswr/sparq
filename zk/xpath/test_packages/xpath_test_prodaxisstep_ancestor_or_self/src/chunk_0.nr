//! Test chunk 0 for prod:AxisStep.ancestor-or-self
//! Contains 22 tests

use dep::xpath::stub_prodaxisstep_ancestor_or_self;

#[test]
fn ancestorself_2() {
    //  Evaluation of the ancestor-or-self axis for which the given node does not exist
    // XPath: fn:count(/works/employee[1]/ancestor-or-self::noSuchNode)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_3() {
    //  Evaluation of the ancestor-or-self axis that is part of an 'is' expression (ret
    // XPath: exactly-one(/works/employee[1]/ancestor-or-self::works) is e
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_4() {
    //  Evaluation of the ancestor-or-self axis that is part of an 'is' expression (ret
    // XPath: exactly-one(/works/employee[1]/ancestor-or-self::works) is e
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_5() {
    //  Evaluation of the ancestor-or-self axis that is part of an 'node before' expres
    // XPath: exactly-one(/works/employee[1]/ancestor-or-self::works) << e
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_6() {
    //  Evaluation of the ancestor-or-self axis that is part of an 'node before' expres
    // XPath: exactly-one(/works/employee[1]/ancestor-or-self::works) << e
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_7() {
    //  Evaluation of the ancestor-or-self axis that is part of an 'node before' expres
    // XPath: exactly-one(/works/employee[1]/ancestor-or-self::works) << e
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_8() {
    //  Evaluation of the ancestor-or-self axis that is part of an 'node after' express
    // XPath: exactly-one(/works/employee[1]) >> exactly-one(/works/employ
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_9() {
    //  Evaluation of the ancestor-or-self axis that is part of an 'node after' express
    // XPath: exactly-one(/works/employee[1]/ancestor-or-self::works) >> e
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_10() {
    //  Evaluation of the ancestor-or-self axis that is part of an 'node after' express
    // XPath: exactly-one(/works/employee[1]/ancestor-or-self::works) >> e
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_11() {
    //  Evaluation of the ancestor-or-self axis that is part of an 'union ' operation.
    // XPath: ((/works/employee[12]/*/day/ancestor-or-self::overtime) | (/
    // Expected: 2
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_12() {
    //  Evaluation of the ancestor-or-self axis that is part of an 'union' operation. B
    // XPath: ((/works/employee[12]/*/day[1]/ancestor-or-self::overtime) |
    // Expected: 2
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_13() {
    //  Evaluation of the ancestor-or-self axis that is part of an 'intersect' operatio
    // XPath: ((/works/employee[12]/overtime/day/ancestor-or-self::employe
    // Expected: John Doe 12
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_14() {
    //  Evaluation of the ancestor-self axis that is part of an 'except' operation. Bot
    // XPath: fn:count((/works/employee[12]/overtime/day[ancestor-or-self:
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_15() {
    //  Evaluation of the ancestor-or-self axis that is part of an 'except' operation.
    // XPath: (/works/employee[12]/overtime/day[ancestor-or-self::overtime
    // Expected: Tuesday
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_16() {
    //  Evaluation of the ancestor-or-self axis that is part of a boolean expression ('
    // XPath: (/works/employee[12]/overtime/day[ancestor-or-self::overtime
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_17() {
    //  Evaluation of the ancestor-or-self axis that is part of a boolean expression ('
    // XPath: (/works/employee[12]/overtime/day[ancestor-or-self::overtime
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_18() {
    //  Evaluation of the ancestor-or-self axis that is part of a boolean expression ('
    // XPath: (/works/employee[12]/overtime/day[ancestor-or-self::overtime
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_19() {
    //  Evaluation of the ancestor-or-self axis that is part of a boolean expression ('
    // XPath: (/works/employee[12]/overtime/day[ancestor-or-self::overtime
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn ancestorself_20() {
    //  Evaluation of the ancestor-or-self axis that used as part of the deep-equal-fun
    // XPath: fn:deep-equal(/works/employee[12]/overtime/ancestor-or-self:
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn unabbreviatedsyntax_11() {
    //  Evaluate selecting an ancestor or self (ancestor-or-self::employee)- Select the
    // XPath: (for $h in (/works/employee[1]/hours) return $h/ancestor-or-
    // Expected: Jane Doe 1
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn k2_ancestor_or_selfaxis_7() {
    //  Apply fn:count() to the result of axis ancestor-or-self.
    // XPath: count(<a> <b c=\"\"> <c/> </b> <d/> </a>/b/c/(ancestor-or-se
    // Expected: 3
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}

#[test]
fn k2_ancestor_or_selfaxis_8() {
    //  Apply fn:count() to the result of axis ancestor-or-self(#2).
    // XPath: <a> <b c=\"\"> <c/> </b> <d/> </a>//count(ancestor-or-self::
    // Expected: 1 2 3 2
    // Calls stub function - will fail until prod:AxisStep.ancestor-or-self is implemented
    let _ = stub_prodaxisstep_ancestor_or_self();
}
