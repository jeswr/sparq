//! Test chunk 1 for prod:Predicate
//! Contains 50 tests

use dep::xpath::stub_prodpredicate;

#[test]
fn k_filterexpr_41() {
    //  A test whose essence is: `(0, 1, 2)[true()][1] eq 0`.
    // XPath: (0, 1, 2)[true()][1] eq 0
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_42() {
    //  A test whose essence is: `(1, 2, 3)[position() eq 2 or position() eq 3][2] eq 3
    // XPath: (1, 2, 3)[position() eq 2 or position() eq 3][2] eq 3
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_43() {
    //  A test whose essence is: `empty((1, 2, 3)[position() eq 2 or position() eq 3][3
    // XPath: empty((1, 2, 3)[position() eq 2 or position() eq 3][3])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_44() {
    //  A test whose essence is: `deep-equal((1, 2, 3), (1, 2, 3)[2 or 3])`.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3)[2 or 3])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_45() {
    //  A test whose essence is: `empty((1, 2, 3)[3][2])`.
    // XPath: empty((1, 2, 3)[3][2])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_46() {
    //  A test whose essence is: `empty((1, 2, 3)[3][0])`.
    // XPath: empty((1, 2, 3)[3][0])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_47() {
    //  An excessive nesting of various predicates.
    // XPath: (0, 2, 4, 5)[1][1][1][true()][1][true()][1] eq 0
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_48() {
    //  A large numeric xs:double predicate that evaluates to the empty sequence.
    // XPath: empty((1, 2, 3)[3e8])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_50() {
    //  A test whose essence is: `deep-equal(('a', 'b', 'c'), (0, 1, 2, 'a', 'b', 'c')[
    // XPath: deep-equal((\"a\", \"b\", \"c\"), (0, 1, 2, \"a\", \"b\", \"
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_60() {
    //  A test whose essence is: `deep-equal((1, 2, 3), (1, 2, 3)[position() >= 1])`.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3)[position() >= 1])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_61() {
    //  A test whose essence is: `deep-equal((1, 2, 3), (1, 2, 3)[3 >= position()])`.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3)[3 >= position()])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_62() {
    //  A test whose essence is: `deep-equal((1, 2, 3), (1, 2, 3)[position() ge 1])`.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3)[position() ge 1])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_63() {
    //  A test whose essence is: `deep-equal((1, 2, 3), (1, 2, 3)[3 ge position()])`.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3)[3 ge position()])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_64() {
    //  A test whose essence is: `deep-equal((0, 1, 2), (0, 1, 2)[position() eq positio
    // XPath: deep-equal((0, 1, 2), (0, 1, 2)[position() eq position()])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_65() {
    //  A test whose essence is: `deep-equal((0, 1, 2), (0, 1, 2)[position() = position
    // XPath: deep-equal((0, 1, 2), (0, 1, 2)[position() = position()])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_66() {
    //  A test whose essence is: `(0, 1, 2)[1 eq position()]`.
    // XPath: (0, 1, 2)[1 eq position()]
    // Expected: 0
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_67() {
    //  A test whose essence is: `(0, 1, 2)[3 eq position()]`.
    // XPath: (0, 1, 2)[3 eq position()]
    // Expected: 2
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_68() {
    //  A test whose essence is: `(0, 1, 2)[position() eq 3]`.
    // XPath: (0, 1, 2)[position() eq 3]
    // Expected: 2
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_69() {
    //  A test whose essence is: `deep-equal((1, 2, 3), (1, 2, 3)[number(.)])`.
    // XPath: deep-equal((1, 2, 3), (1, 2, 3)[number(.)])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_70() {
    //  A test whose essence is: `deep-equal((1, 2), (0, 1, 2)[if(. eq 1) then 2 else 3
    // XPath: deep-equal((1, 2), (0, 1, 2)[if(. eq 1) then 2 else 3])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_71() {
    //  A test whose essence is: `deep-equal((0, 1, 2), (0, 1, 2)[if(. eq 8) then 'str'
    // XPath: deep-equal((0, 1, 2), (0, 1, 2)[if(. eq 8) then \"str\" else
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_72() {
    //  A test whose essence is: `deep-equal((0, 1, 2), (0, 1, 2)[if(. eq 8) then 0 els
    // XPath: deep-equal((0, 1, 2), (0, 1, 2)[if(. eq 8) then 0 else posit
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_73() {
    //  A test whose essence is: `(0, 1, 2)[last()]`.
    // XPath: (0, 1, 2)[last()]
    // Expected: 2
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_74() {
    //  A test whose essence is: `(0, 1, '2')[last()]`.
    // XPath: (0, 1, \"2\")[last()]
    // Expected: 2
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_78() {
    //  Predicates involving the focus' context item.
    // XPath: (0, 1, 2)[. eq 0]
    // Expected: 0
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_79() {
    //  Predicates involving the focus' context item.
    // XPath: (0, 1, 2)[. eq 1]
    // Expected: 1
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_80() {
    //  Predicates involving the focus' context item.
    // XPath: (0, 1, 2)[. eq 2]
    // Expected: 2
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_81() {
    //  Predicates involving the focus' context item.
    // XPath: deep-equal((0, 1, 2), (0, 1, 2)[. eq 0 or . eq 1 or . eq 2])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_86() {
    //  A predicate that never can match.
    // XPath: empty(current-time()[2])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_87() {
    //  A predicate that never can match.
    // XPath: empty((1, 2, 3, current-time())[0])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_89() {
    //  A truth predicate that always match.
    // XPath: deep-equal(remove((1, 2, 3, current-time()), 4)[true()], (1,
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_90() {
    //  Nested predicate with multiple calls to fn:last().
    // XPath: (1, 2, 3)[(last(), last())[2]]
    // Expected: 3
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_95() {
    //  Filter expression where the predicate depends only on the context document (opt
    // XPath: /works/employee[@name=/works/employee[1]/@name]/@name/string
    // Expected: "Jane Doe 1"
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k_filterexpr_96() {
    //  Filter expression where the predicate depends only on the context document (opt
    // XPath: document{<works>{/tail(works/employee)}</works>}/works/emplo
    // Expected: "Jane Doe 3"
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k2_filterexpr_3() {
    //  An numeric predicate that is xs:decimal.
    // XPath: empty((1,2,3,4,5)[3.4])
    // Expected: true
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn predicates_30() {
    //  Evaluation of predicates reordering by an implementation
    // XPath: for $x in /works/employee[fn:position() lt 5][fn:position()
    // Expected: E1 P1 E1 P3
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn predicates_31() {
    //  Evaluation of predicates reordering by an implementation
    // XPath: for $x in /works/employee[fn:position() mod 2 eq 1][fn:posit
    // Expected: E1 P1 E1 P3 E1 P5 E2 P1
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn predicates_32() {
    //  Numeric predicate computed by subscripting
    // XPath: declare variable $i := (\"x\", 1);         declare variable
    // Expected: "x"
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn predicates_33() {
    //  Filter expression whose predicate compares boolean to untypedAtomic. Saxon bug
    // XPath: count(/works/employee[(xs:integer(hours) gt 40) = xs:untyped
    // Expected: 3
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn predicatesns_1() {
    //  Evaluation of a simple predicate with a 'true' value (uses 'fn:true'). Use of f
    // XPath: fn:count((//integer[fn:true()]))
    // Expected: 1
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k2_predicates_1() {
    //  Predicate combined with 'treat as'.
    // XPath: \"c\"[. treat as xs:string]
    // Expected: c
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn k2_predicates_9() {
    //  Use an axis step on an expression which has static type item(), from within a p
    // XPath: declare function local:foo($arg as item()) { string($arg/@ar
    // Expected: result
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn cbcl_filterexpr_001() {
    //  Test that negated floating point comparisons against the position variable opti
    // XPath: declare function local:nan() { xs:float(\"NaN\") };
    // Expected: 1 2 3 4 5 6 7 8 9 10
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn cbcl_first_in_sequence_001() {
    //  test fn:first-in-sequence on sub-expression where quant = 1
    // XPath: declare function local:generate($arg as xs:integer?) { if ($
    // Expected: 1
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn cbcl_first_in_sequence_002() {
    //  test fn:first-in-sequence on a for expression
    // XPath: declare function local:generate($arg as xs:integer?) as xs:i
    // Expected: 3
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn cbcl_first_in_sequence_003() {
    //  test fn:first-in-sequence on a for expression
    // XPath: declare function local:generate($arg as xs:integer?) as xs:i
    // Expected: 2
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn cbcl_first_in_sequence_004() {
    //  test fn:first-in-sequence on a for-at expression
    // XPath: declare function local:generate($arg as xs:integer?) as xs:i
    // Expected: 2
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn cbcl_first_in_sequence_005() {
    //  test fn:first-in-sequence on a for-at expression
    // XPath: declare function local:generate($arg as xs:integer?) as xs:i
    // Expected: 2
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn cbcl_first_in_sequence_006() {
    //  test fn:first-in-sequence on a map expression
    // XPath: declare function local:generate($arg as xs:integer?) as xs:s
    // Expected: a
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}

#[test]
fn cbcl_first_in_sequence_007() {
    //  test fn:first-in-sequence on a map expression
    // XPath: declare function local:generate($arg as xs:integer?) as xs:i
    // Expected: 0
    // Calls stub function - will fail until prod:Predicate is implemented
    let _ = stub_prodpredicate();
}
