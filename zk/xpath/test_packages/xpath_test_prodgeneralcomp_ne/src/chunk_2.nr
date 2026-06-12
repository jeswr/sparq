//! Test chunk 2 for prod:GeneralComp.ne
//! Contains 28 tests

use dep::xpath::stub_prodgeneralcomp_ne;

#[test]
fn k_gencompne_2() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(() = 1 )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_3() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(() != () )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_4() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(1 != () )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_5() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(() != 1 )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_7() {
    //  Example from the XPath 2.0 specification.
    // XPath: (1, 2) != (2, 3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_8() {
    //  General comparison causing numeric promotion from xs:untypedAtomic.
    // XPath: xs:untypedAtomic(\"2\") != 1
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_9() {
    //  General comparison causing numeric promotion from xs:untypedAtomic.
    // XPath: not(xs:untypedAtomic(\"1\") != 1)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_10() {
    //  General comparison causing numeric promotion from xs:untypedAtomic.
    // XPath: not(\"2\" = xs:untypedAtomic(\"1\"))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_11() {
    //  General comparison causing numeric promotion from xs:untypedAtomic.
    // XPath: 2 != xs:untypedAtomic(\"1\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_12() {
    //  General comparison causing numeric promotion from xs:untypedAtomic.
    // XPath: not(1 != xs:untypedAtomic(\"1\"))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_13() {
    //  An expression involving the '!=' operator that trigger certain optimization pat
    // XPath: count((0, timezone-from-time(current-time()))) != 0
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_14() {
    //  General comparison involving xs:untypedAtomic/xs:string.
    // XPath: \"a string\" != \"a stringDIFF\"
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_15() {
    //  General comparison involving xs:untypedAtomic/xs:string.
    // XPath: not(\"a string\" != \"a string\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_16() {
    //  General comparison involving xs:untypedAtomic/xs:string.
    // XPath: xs:untypedAtomic(\"a string\") != \"a stringDIFF\"
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_17() {
    //  General comparison involving xs:untypedAtomic/xs:string.
    // XPath: \"a string\" != xs:untypedAtomic(\"a stringDIFF\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_18() {
    //  General comparison involving xs:untypedAtomic/xs:string.
    // XPath: not(\"a string\" != xs:untypedAtomic(\"a string\"))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_19() {
    //  General comparison involving xs:untypedAtomic/xs:string.
    // XPath: not(xs:untypedAtomic(\"a string\") != \"a string\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_20() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: xs:untypedAtomic(\"true\") != false()
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_21() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: false() != xs:untypedAtomic(\"true\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_22() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: not(xs:untypedAtomic(\"false\") != false())
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_23() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: not(false() != xs:untypedAtomic(\"false\"))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_24() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: (1, 2, 3) != 1
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_25() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: (1, 2, 3) != 2
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_26() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: (1, 2, 3) != 3
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_27() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 1 != 2
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_28() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 1 != (1, 2, 3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_29() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 2 != (1, 2, 3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}

#[test]
fn k_gencompne_30() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 3 != (1, 2, 3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ne is implemented
    let _ = stub_prodgeneralcomp_ne();
}
