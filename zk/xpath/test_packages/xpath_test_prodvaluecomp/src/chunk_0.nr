//! Test chunk 0 for prod:ValueComp
//! Contains 39 tests

use dep::xpath::stub_prodvaluecomp;

#[test]
fn k_valcomptypechecking_30() {
    //  Compare xs:untypedAtomic and xs:string.
    // XPath: xs:untypedAtomic(\"a string\") eq \"a string\"
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn k_valcomptypechecking_31() {
    //  Compare xs:untypedAtomic and xs:string.
    // XPath: \"a string\" eq xs:untypedAtomic(\"a string\")
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn k_valcomptypechecking_32() {
    //  An expression involving the ge operator that trigger certain optimization paths
    // XPath: count((0, current-time())) ge 1
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn k_valcomptypechecking_33() {
    //  An expression involving the ne operator that trigger certain optimization paths
    // XPath: count((0, current-time())) ne 0
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn k_valcomptypechecking_34() {
    //  Test that fn:count combined with expressions that might disable compile time ev
    // XPath: not(count((1, 2, current-time())) eq 0)
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn k_valcomptypechecking_42() {
    //  A simple value comparison that in some Bison grammars triggers a bug.
    // XPath: count((1, 2)) eq 2
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comparison_5() {
    //  Value comparison on arrays.
    // XPath: [3] eq 3
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comparison_6() {
    //  Value comparison on arrays.
    // XPath: [3] le [3]
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comparison_12() {
    //  Value comparison on elements annotated as xs:int.
    // XPath: //test:comparable[1]/test:int eq //test:comparable[2]/test:i
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_string_1() {
    //  Value comparison on strings.
    // XPath: \"abc\" eq concat(\"a\", \"bc\")
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_string_3() {
    //  Value comparison on strings.
    // XPath: \"abc\" eq string(/works/@id)
    // Expected: false
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_string_6() {
    //  Value comparison on strings.
    // XPath: string(/works/@id) eq \"abc\"
    // Expected: false
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_string_8() {
    //  Value comparison on strings.
    // XPath: not(\"abc\" eq concat(\"a\", \"bc\"))
    // Expected: false
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_string_9() {
    //  Value comparison on strings.
    // XPath: not(\"abc\" eq /works/@id)
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_string_10() {
    //  Value comparison on strings.
    // XPath: not(\"abc\" eq string(/works/@id))
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_string_11() {
    //  Value comparison on strings.
    // XPath: not(\"abc\" eq xs:string(/works/@id))
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_string_12() {
    //  Value comparison on strings.
    // XPath: not(works/@id eq \"abc\")
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_string_13() {
    //  Value comparison on strings.
    // XPath: not(string(/works/@id) eq \"abc\")
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_string_14() {
    //  Value comparison on strings.
    // XPath: not(xs:string(/works/@id) eq \"abc\")
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_string_15() {
    //  Value comparison on strings. Saxon bug 3983.
    // XPath: not(/works/@iddddd eq \"\")
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_ne_string_1() {
    //  Value comparison on strings.
    // XPath: \"abc\" ne concat(\"a\", \"bc\")
    // Expected: false
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_ne_string_3() {
    //  Value comparison on strings.
    // XPath: \"abc\" ne string(/works/@id)
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_ne_string_6() {
    //  Value comparison on strings.
    // XPath: string(/works/@id) ne \"abc\"
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_ne_string_8() {
    //  Value comparison on strings.
    // XPath: not(\"abc\" ne concat(\"a\", \"bc\"))
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_ne_string_9() {
    //  Value comparison on strings.
    // XPath: not(\"abc\" ne /works/@id)
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_ne_string_10() {
    //  Value comparison on strings.
    // XPath: not(\"abc\" ne string(/works/@id))
    // Expected: false
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_ne_string_11() {
    //  Value comparison on strings.
    // XPath: not(\"abc\" ne xs:string(/works/@id))
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_ne_string_12() {
    //  Value comparison on strings.
    // XPath: not(works/@id ne \"abc\")
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_ne_string_13() {
    //  Value comparison on strings.
    // XPath: not(string(/works/@id) ne \"abc\")
    // Expected: false
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_ne_string_14() {
    //  Value comparison on strings.
    // XPath: not(xs:string(/works/@id) ne \"abc\")
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_ne_string_15() {
    //  Value comparison on strings. Saxon bug 3983.
    // XPath: not(/works/@iddddd ne \"\")
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_int_1() {
    //  Value comparison on integers.
    // XPath: 13 eq 12+1
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_int_3() {
    //  Value comparison on integers.
    // XPath: 40 eq xs:integer(/works/employee[1]/hours)
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_int_6() {
    //  Value comparison on integers.
    // XPath: 40 eq number(/works/employee[1]/hours)
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_int_7() {
    //  Value comparison on integers.
    // XPath: 40 eq xs:double(/works/employee[1]/hours)
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_double_1() {
    //  Value comparison on doubles.
    // XPath: 13e0 eq 12e0+1
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_double_3() {
    //  Value comparison on doubles.
    // XPath: 40e0 eq xs:double(/works/employee[1]/hours)
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_double_6() {
    //  Value comparison on doubles.
    // XPath: 40e0 eq number(/works/employee[1]/hours)
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}

#[test]
fn value_comp_eq_double_7() {
    //  Value comparison on doubles.
    // XPath: 40e0 eq xs:double(/works/employee[1]/hours)
    // Expected: true
    // Calls stub function - will fail until prod:ValueComp is implemented
    let _ = stub_prodvaluecomp();
}
