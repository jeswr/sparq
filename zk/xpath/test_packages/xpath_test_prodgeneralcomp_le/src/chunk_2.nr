//! Test chunk 2 for prod:GeneralComp.le
//! Contains 8 tests

use dep::xpath::stub_prodgeneralcomp_le;

#[test]
fn k_gencomplteq_1() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(() <= () )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn k_gencomplteq_2() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(1 <= () )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn k_gencomplteq_3() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(() <= 1 )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn k_gencomplteq_4() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 1 <= 1
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn k_gencomplteq_5() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: xs:untypedAtomic(\"false\") <= false()
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn k_gencomplteq_6() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: false() <= xs:untypedAtomic(\"false\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn k_gencomplteq_7() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: xs:untypedAtomic(\"false\") <= false()
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}

#[test]
fn k_gencomplteq_8() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: true() <= xs:untypedAtomic(\"true\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.le is implemented
    let _ = stub_prodgeneralcomp_le();
}
