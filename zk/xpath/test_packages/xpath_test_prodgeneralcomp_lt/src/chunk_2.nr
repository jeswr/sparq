//! Test chunk 2 for prod:GeneralComp.lt
//! Contains 28 tests

use dep::xpath::stub_prodgeneralcomp_lt;

#[test]
fn k_gencomplt_1() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(() < () )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_2() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(1 < () )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_3() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(() < 1 )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_5() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: (1, 2, 3) < 4
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_6() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: (1, 2, 3) < 2
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_7() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: (1, 2, 3) < 3
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_8() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 1 < (1, 2, 3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_9() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 2 < (1, 2, 3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_10() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 0 < (1, 2, 3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_11() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 1 < 2
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_12() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: xs:untypedAtomic(\"false\") < true()
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_13() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: false() < xs:untypedAtomic(\"true\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_14() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: not(xs:untypedAtomic(\"true\") < false())
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_15() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: not(true() < xs:untypedAtomic(\"false\"))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_18() {
    //  <= combined with count().
    // XPath: 1 <= count((1, 2, 3, timezone-from-time(current-time()), 4))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k_gencomplt_19() {
    //  < combined with count().
    // XPath: 0 < count((1, 2, 3, timezone-from-time(current-time()), 4))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k2_gencomplt_1() {
    //  Do a comparison that requires conversion via element()->xs:untypedAtomic->xs:do
    // XPath: <e>1.1</e> < 3
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k2_gencomplt_2() {
    //  Do a comparison that requires conversion via element()->xs:untypedAtomic->xs:do
    // XPath: 3. < <e>1.1</e>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k2_gencomplt_3() {
    //  Do a comparison that requires conversion via element()->xs:untypedAtomic->xs:do
    // XPath: <e>1.1</e> < 3.
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k2_gencomplt_4() {
    //  Do a comparison that requires conversion via element()->xs:untypedAtomic->xs:do
    // XPath: 3 < <e>1.1</e>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k2_gencomplt_5() {
    //  Do a comparison that requires conversion via element()->xs:untypedAtomic->xs:do
    // XPath: <e>1.1</e> < 3e3
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k2_gencomplt_6() {
    //  Do a comparison that requires conversion via element()->xs:untypedAtomic->xs:do
    // XPath: 3e3 < <e>1.1</e>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k2_gencomplt_7() {
    //  Do a comparison that requires conversion via element()->xs:untypedAtomic->xs:do
    // XPath: <e>1.1</e> < xs:float(3e3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn k2_gencomplt_8() {
    //  Do a comparison that requires conversion via element()->xs:untypedAtomic->xs:do
    // XPath: xs:float(3e3) < <e>1.1</e>
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn gencomplt_9() {
    //  General comparison on arrays.
    // XPath: [3, 4, 5] < 4
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn gencomplt_10() {
    //  General comparison on arrays.
    // XPath: [[3, 4], 5] < [4, [5, 6]]
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn gencomplt_11() {
    //  General comparison on arrays.
    // XPath: [3, 4, 5] < []
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}

#[test]
fn gencomplt_14() {
    // Only one comparison allowed per expression, unless parenthesized
    // XPath: (true() < true()) < true()
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.lt is implemented
    let _ = stub_prodgeneralcomp_lt();
}
