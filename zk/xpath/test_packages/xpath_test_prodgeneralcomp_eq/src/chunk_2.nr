//! Test chunk 2 for prod:GeneralComp.eq
//! Contains 50 tests

use dep::xpath::stub_prodgeneralcomp_eq;

#[test]
fn k_gencompeq_2() {
    //  General comparisons involving the empty sequence.
    // XPath: not((() = ()))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_3() {
    //  General comparisons involving the empty sequence.
    // XPath: not((() = 1))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_4() {
    //  General comparisons involving the empty sequence.
    // XPath: not(1 = ())
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_6() {
    //  Example from the XPath 2.0 specification.
    // XPath: (1, 2) = (2, 3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_7() {
    //  Example from the XPath 2.0 specification.
    // XPath: (2, 3) = (3, 4)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_8() {
    //  Example from the XPath 2.0 specification.
    // XPath: not((1, 2) = (3, 4))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_11() {
    //  General comparison causing numeric promotion from xs:untypedAtomic.
    // XPath: xs:untypedAtomic(\"1\") = 1
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_12() {
    //  General comparison causing numeric promotion from xs:untypedAtomic.
    // XPath: not(xs:untypedAtomic(\"2\") = 1)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_13() {
    //  General comparison causing numeric promotion from xs:untypedAtomic.
    // XPath: xs:untypedAtomic(\"1\") = 1
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_14() {
    //  General comparison causing numeric promotion from xs:untypedAtomic.
    // XPath: 1 = xs:untypedAtomic(\"1\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_15() {
    //  General comparison causing numeric promotion from xs:untypedAtomic.
    // XPath: 1 = xs:untypedAtomic(\"1\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_20() {
    //  An expression involving the '=' operator that trigger certain optimization path
    // XPath: count(remove(remove((current-time(), 1), 1), 1)) = 0
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_21() {
    //  Test that fn:count combined with expressions that might disable compile time ev
    // XPath: not(0 = count((1, 2, timezone-from-time(current-time()))))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_22() {
    //  Test that fn:count combined with expressions that might disable compile time ev
    // XPath: 0 != count((1, 2, timezone-from-time(current-time())))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_24() {
    //  General comparison involving xs:untypedAtomic/xs:string.
    // XPath: \"a string\" = \"a string\"
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_25() {
    //  General comparison involving xs:untypedAtomic/xs:string.
    // XPath: xs:untypedAtomic(\"a string\") = \"a string\"
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_26() {
    //  General comparison involving xs:untypedAtomic/xs:string.
    // XPath: \"a string\" = xs:untypedAtomic(\"a string\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_27() {
    //  General comparison involving xs:untypedAtomic/xs:string.
    // XPath: not(xs:untypedAtomic(\"a string\") = \"a stringDIFF\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_28() {
    //  General comparison involving xs:untypedAtomic/xs:string.
    // XPath: not(\"a string\" = xs:untypedAtomic(\"a stringDIFF\"))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_29() {
    //  General comparison involving xs:untypedAtomic/xs:string.
    // XPath: not(\"a string\" = \"a stringDIFF\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_36() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 1 = 1
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_37() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: (1, 2, 3) = 1
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_38() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: (1, 2, 3) = 2
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_39() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: (1, 2, 3) = 3
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_40() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 2 = (1, 2, 3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_41() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 1 = (1, 2, 3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_42() {
    //  General comparison where the operands are various various sequence of xs:intege
    // XPath: 3 = (1, 2, 3)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_43() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: xs:untypedAtomic(\"false\") = false()
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_44() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: false() = xs:untypedAtomic(\"false\")
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_45() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: not(xs:untypedAtomic(\"true\") = false())
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k_gencompeq_46() {
    //  General comparison causing a xs:untypedAtomic value to be cast to xs:boolean, a
    // XPath: (true() = xs:untypedAtomic(\"true\"))
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k2_gencompeq_1() {
    //  Compare two values returned from fn:lower-case().
    // XPath: declare variable $vA as xs:string := (\"B STRING\", current-
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k2_gencompeq_2() {
    //  Compare two values returned from fn:upper-case().
    // XPath: declare variable $vA as xs:string := (\"B STRING\", current-
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k2_gencompeq_3() {
    //  Compare two non-matching values returned from fn:lower-case().
    // XPath: declare variable $vA as xs:string := (\"B STRING\", current-
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k2_gencompeq_4() {
    //  Compare two non-matching values returned from fn:upper-case().
    // XPath: declare variable $vA as xs:string := (\"B STRING\", current-
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k2_gencompeq_5() {
    //  Compare two non-matching values returned from fn:lower-case() and fn:upper-case
    // XPath: declare variable $vA as xs:string := (\"B STRING\", current-
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn k2_gencompeq_6() {
    //  Compare two non-matching values returned from fn:lower-case() and fn:upper-case
    // XPath: declare variable $vA as xs:string := (\"B STRING\", current-
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_1() {
    //  General comparison where both types are instances of xs:untypedAtomic.
    // XPath: xs:untypedAtomic(\"a string\") = xs:untypedAtomic(\"a string
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_2() {
    //  General comparison where one of the types is xs:untypedAtomic and the other xs:
    // XPath: xs:untypedAtomic(\"1\") = xs:NCName(\"string\")
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_3() {
    //  General comparison where one of the types is xs:untypedAtomic and the other xs:
    // XPath: xs:dayTimeDuration(\"P3DT08H34M12.143S\") = xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_5() {
    //  General comparison where one of the types is xs:untypedAtomic and the other xs:
    // XPath: xs:untypedAtomic(\"P1999Y10M\") = xs:yearMonthDuration(\"P19
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_7() {
    //  General comparison on arrays.
    // XPath: [3, 4, 5] = 4
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_8() {
    //  General comparison on arrays.
    // XPath: [[3, 4], 5] = [4, [5, 6]]
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_9() {
    //  General comparison on arrays.
    // XPath: [3, 4, 5] = []
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_10() {
    //  General comparison that might be optimized.
    // XPath: declare variable $p external := 276; $p = (1 to 1000)
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_11() {
    //  General comparison that might be optimized.
    // XPath: declare variable $p external := -276; (1 to 1000) = $p
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_12() {
    //  General comparison that might be optimized.
    // XPath: declare variable $p external := xs:untypedAtomic('000504e0')
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_13() {
    //  General comparison that might be optimized.
    // XPath: declare variable $p external := xs:untypedAtomic('4.5'); (1
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_14() {
    //  General comparison that might be optimized.
    // XPath: declare variable $p external := 17; $p = (1, 8, 13, 14, 98,
    // Expected: true
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}

#[test]
fn gencompeq_15() {
    //  General comparison that might be optimized.
    // XPath: declare variable $p external := -101; (1, 8, 13, 14, 98, -22
    // Expected: false
    // Calls stub function - will fail until prod:GeneralComp.eq is implemented
    let _ = stub_prodgeneralcomp_eq();
}
