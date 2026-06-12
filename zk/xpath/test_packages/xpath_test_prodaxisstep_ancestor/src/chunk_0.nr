//! Test chunk 0 for prod:AxisStep.ancestor
//! Contains 25 tests

use dep::xpath::stub_prodaxisstep_ancestor;

#[test]
fn ancestor_2() {
    //  Evaluation of the ancestor axis for which the given node does not exists. Uses
    // XPath: fn:count(/works/employee[1]/ancestor::noSuchNode)
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_3() {
    //  Evaluation of the ancestor axis that is part of an 'is' expression (return true
    // XPath: exactly-one(/works/employee[1]/ancestor::works) is exactly-o
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_4() {
    //  Evaluation of the ancestor axis that is part of an 'is' expression (return fals
    // XPath: exactly-one(/works/employee[1]/ancestor::works) is exactly-o
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_5() {
    //  Evaluation of the ancestor axis that is part of an 'node before' expression (re
    // XPath: exactly-one(/works/employee[1]/ancestor::works) << exactly-o
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_6() {
    //  Evaluation of the ancestor axis that is part of an 'node before' expression and
    // XPath: exactly-one(/works/employee[1]/ancestor::works) << exactly-o
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_7() {
    //  Evaluation of the ancestor axis that is part of an 'node before' expression bot
    // XPath: exactly-one(/works/employee[1]/ancestor::works) << exactly-o
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_8() {
    //  Evaluation of the ancestor axis that is part of an 'node after' expression (ret
    // XPath: exactly-one(/works/employee[1]) >> exactly-one(/works/employ
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_9() {
    //  Evaluation of the ancestor axis that is part of an 'node after' expression with
    // XPath: exactly-one(/works/employee[1]/ancestor::works) >> exactly-o
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_10() {
    //  Evaluation of the ancestor axis that is part of an 'node after' expression with
    // XPath: exactly-one(/works/employee[1]/ancestor::works) >> exactly-o
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_14() {
    //  Evaluation of the ancestor axis that is part of an 'except' operation. Both ope
    // XPath: fn:count((/works/employee[12]/overtime/day[ancestor::overtim
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_15() {
    //  Evaluation of the ancestor axis that is part of an 'except' operation. Both ope
    // XPath: (/works/employee[12]/overtime/day[ancestor::overtime]) excep
    // Expected: Tuesday
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_16() {
    //  Evaluation of the ancestor axis that is part of a boolean expression ('and' and
    // XPath: (/works/employee[12]/overtime/day[ancestor::overtime]) and f
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_17() {
    //  Evaluation of the ancestor axis that is part of a boolean expression ('and' and
    // XPath: (/works/employee[12]/overtime/day[ancestor::overtime]) and f
    // Expected: false
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_18() {
    //  Evaluation of the ancestor axis that is part of a boolean expression ('or' and
    // XPath: (/works/employee[12]/overtime/day[ancestor::overtime]) or fn
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_19() {
    //  Evaluation of the ancestor axis that is part of a boolean expression ('or' and
    // XPath: (/works/employee[12]/overtime/day[ancestor::overtime]) or fn
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_20() {
    //  Evaluation of the ancestor axis that used as part of the deep-equal-function.
    // XPath: fn:deep-equal(/works/employee[12]/overtime/ancestor::works,/
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn ancestor_21() {
    //  Evaluation of the ancestor axis used together with a newly constructed element.
    // XPath: let $var := <anElement>Some Content</anElement> return fn:co
    // Expected: 0
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn k2_ancestoraxis_10() {
    //  Apply axis ancestor on a single root node.
    // XPath: empty(<element/>/ancestor::node())
    // Expected: true
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn k2_ancestoraxis_13() {
    //  Apply axis ancestor on a single element.
    // XPath: 1, <element/>/ancestor::node(), 1
    // Expected: 1 1
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn k2_ancestoraxis_14() {
    //  Apply axis ancestor to a single processing instruction.
    // XPath: 1, <?target data?>/ancestor::node(), 1
    // Expected: 1 1
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn k2_ancestoraxis_15() {
    //  Apply axis ancestor to a single attribute.
    // XPath: 1, attribute name {\"content\"}/ancestor::node(), 1
    // Expected: 1 1
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn k2_ancestoraxis_16() {
    //  Apply axis ancestor to a single comment.
    // XPath: 1, <!-- content -->/ancestor::node(), 1
    // Expected: 1 1
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn k2_ancestoraxis_18() {
    //  Apply axis ancestor to a single text node.
    // XPath: 1, text {\"\"}/ancestor::node(), 1
    // Expected: 1 1
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn k2_ancestoraxis_19() {
    //  Apply fn:count() to the result of axis ancestor.
    // XPath: count(<a> <b c=\"\"> <c/> </b> <d/> </a>/b/c/(ancestor::*))
    // Expected: 2
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}

#[test]
fn k2_ancestoraxis_20() {
    //  Apply fn:count() to the result of axis ancestor(#2).
    // XPath: <a> <b c=\"\"> <c/> </b> <d/> </a>//count(ancestor::*)
    // Expected: 0 1 2 1
    // Calls stub function - will fail until prod:AxisStep.ancestor is implemented
    let _ = stub_prodaxisstep_ancestor();
}
