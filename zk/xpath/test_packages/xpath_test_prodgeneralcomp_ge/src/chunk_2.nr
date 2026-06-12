//! Test chunk 2 for prod:GeneralComp.ge
//! Contains 10 tests

use dep::xpath::stub_prodgeneralcomp_ge;

#[test]
fn greaterthaneqonanyuri_1() {
    // Test - greaterthaneqonanyuri-1  Evaluation of 'ge' operator on xs:anyURI datatyp
    // XPath: (xs:anyURI(\"http://www.example/com\")) >= (xs:anyURI(\"http
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn k_gencompgteq_1() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(() >= () )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn k_gencompgteq_2() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(1 >= () )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn k_gencompgteq_3() {
    //  General comparison where one or more operands is the empty sequence.
    // XPath: not(() >= 1 )
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn k_gencompgteq_5() {
    //  An expression involving the '>=' operator that trigger certain optimization pat
    // XPath: count((0, timezone-from-time(current-time()))) >= 1
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn k_gencompgteq_6() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: xs:untypedAtomic(\"false\") >= false()
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn k_gencompgteq_7() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: true() >= xs:untypedAtomic(\"false\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn k_gencompgteq_8() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: false() >= xs:untypedAtomic(\"false\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn k_gencompgteq_9() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: xs:untypedAtomic(\"false\") >= false()
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}

#[test]
fn k_gencompgteq_10() {
    //  <= combined with count().
    // XPath: 1 <= count((1, 2, 3, timezone-from-time(current-time()), 4))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.ge is implemented
    let _ = stub_prodgeneralcomp_ge();
}
