//! Test chunk 3 for prod:GeneralComp.eq
//! Contains 9 tests

use dep::xpath::stub_prodgeneralcomp_eq;

#[test]
fn gencompeq_16() {
    //  General comparison that might be optimized.
    // XPath: declare variable $p external := xs:untypedAtomic('-840'); $p
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_17() {
    //  General comparison that might be optimized.
    // XPath: declare variable $p external := xs:untypedAtomic('-022e0');
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_18() {
    //  General comparison that might be optimized.
    // XPath: declare variable $p external := (105, 22e0); $p = (1, 8, 13,
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_19() {
    //  General comparison that might be optimized.
    // XPath: declare variable $p external := (105, xs:untypedAtomic('22e0
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_20() {
    //  General comparison that might be optimized.
    // XPath: declare variable $p external := xs:untypedAtomic('000504e0')
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_21() {
    //  General comparison that might be optimized.
    // XPath: declare variable $p external := xs:untypedAtomic('860302');
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_22() {
    //  General comparison involving a cast to QName.
    // XPath: declare namespace z = \"http://example.com/z\";         decl
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_23() {
    //  General comparison with a mixture of types on both sides.
    // XPath: declare variable $p external := (1 to 10, xs:untypedAtomic('
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_25() {
    // Only one comparison allowed per expression, unless parenthesized
    // XPath: ((true() = true()) = true()) = true()
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}
