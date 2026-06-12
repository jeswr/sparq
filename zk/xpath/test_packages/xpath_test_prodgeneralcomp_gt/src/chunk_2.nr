//! Test chunk 2 for prod:GeneralComp.gt
//! Contains 16 tests

use dep::xpath::stub_prodgeneralcomp_gt;

#[test]
fn k_gencompgt_2() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(1 > () )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_3() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(() > 1 )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_4() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: (1, 2, 3) > 1
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_5() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: (1, 2, 3) > 2
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_6() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: (1, 2, 3) > 0
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_7() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 4 > (1, 2, 3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_8() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 2 > (1, 2, 3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_9() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 3 > (1, 2, 3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_10() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 2 > 1
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_11() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 1 >= 1
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_12() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: xs:untypedAtomic(\"true\") > false()
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_13() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: true() > xs:untypedAtomic(\"false\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_14() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: not(xs:untypedAtomic(\"false\") > true())
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_15() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: not(false() > xs:untypedAtomic(\"true\"))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_18() {
    //  > combined with count().
    // XPath: count((1, 2, 3, timezone-from-time(current-time()), 4)) > 0
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}

#[test]
fn k_gencompgt_19() {
    //  >= combined with count().
    // XPath: count((1, 2, 3, timezone-from-time(current-time()), 4)) >= 0
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.gt is implemented
    let _ = stub_prodgeneralcomp_gt();
}
