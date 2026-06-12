//! Test chunk 0 for op:duration-equal
//! Contains 50 tests

use dep::xpath::stub_opduration_equal;

#[test]
fn op_duration_equal_1() {
    //  Evaluation of duration-equal operator as per example 1 for this operator from F
    // XPath: xs:duration(\"P1Y\") eq xs:duration(\"P12M\")
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_2() {
    //  Evaluation of duration-equal operator as per example 2 for this operator from F
    // XPath: xs:duration(\"PT24H\") eq xs:duration(\"P1D\")
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_3() {
    //  Evaluation of duration-equal operator as per example 3 for this operator from F
    // XPath: xs:duration(\"P1Y\") eq xs:duration(\"P365D\")
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_4() {
    //  Evaluation of duration-equal operator with both operands set to 'P1Y' (uses eq
    // XPath: xs:duration(\"P1Y\") eq xs:duration(\"P1Y\")
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_5() {
    //  Evaluation of duration-equal operator with both operands set to 'P1Y' (uses ne
    // XPath: xs:duration(\"P1Y\") ne xs:duration(\"P1Y\")
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_6() {
    //  Evaluation of duration-equal operator with both operands set to 'PT24H' (uses e
    // XPath: xs:duration(\"PT24H\") eq xs:duration(\"PT24H\")
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_7() {
    //  Evaluation of duration-equal operator with both operands set to 'PT24H' (uses n
    // XPath: xs:duration(\"PT24H\") ne xs:duration(\"PT24H\")
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_8() {
    //  Evaluation of duration-equal operator with both operands set to 'P12M' (uses eq
    // XPath: xs:duration(\"P12M\") eq xs:duration(\"P12M\")
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_9() {
    //  Evaluation of duration-equal operator with both operands set to 'P12M' (uses ne
    // XPath: xs:duration(\"P12M\") ne xs:duration(\"P12M\")
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_10() {
    //  Evaluation of duration-equal operator with both operands set to 'P365D' (uses e
    // XPath: xs:duration(\"P365D\") eq xs:duration(\"P365D\")
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_11() {
    //  Evaluation of duration-equal operator with both operands set to 'P365D' (uses n
    // XPath: xs:duration(\"P365D\") ne xs:duration(\"P365D\")
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_12() {
    //  Evaluation of duration-equal operator with both operands set to 'P36D' and 'P39
    // XPath: xs:duration(\"P36D\") eq xs:duration(\"P39D\")
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_13() {
    //  Evaluation of duration-equal operator with both operands set to 'P36D' and 'P39
    // XPath: xs:duration(\"P36D\") ne xs:duration(\"P39D\")
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_14() {
    //  Evaluation of duration-equal operator with both operands set to 'P36D' and used
    // XPath: fn:not(xs:duration(\"P36D\") eq xs:duration(\"P36D\"))
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_15() {
    //  Evaluation of duration-equal operator with both operands set to 'P36D' and used
    // XPath: fn:not(xs:duration(\"P36D\") ne xs:duration(\"P36D\"))
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_16() {
    //  Evaluation of duration-equal operator with both operands set to 'P36D' and used
    // XPath: xs:boolean(xs:duration(\"P36D\") eq xs:duration(\"P36D\"))
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_17() {
    //  Evaluation of duration-equal operator with both operands set to 'P36D' and used
    // XPath: xs:boolean(xs:duration(\"P36D\") ne xs:duration(\"P36D\"))
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_18() {
    //  Evaluation of duration-equal operator as part of boolean expression 'and' opera
    // XPath: (xs:duration(\"P36D\") eq xs:duration(\"P36D\")) and fn:true
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_19() {
    //  Evaluation of duration-equal operator as part of boolean expression 'and' opera
    // XPath: (xs:duration(\"P36D\") ne xs:duration(\"P36D\")) and fn:true
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_20() {
    //  Evaluation of duration-equal operator as part of boolean expression 'or' operat
    // XPath: (xs:duration(\"P36D\") eq xs:duration(\"P36D\")) or fn:true(
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_21() {
    //  Evaluation of duration-equal operator as part of boolean expression 'or' operat
    // XPath: (xs:duration(\"P36D\") ne xs:duration(\"P36D\")) or fn:true(
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_22() {
    //  Evaluation of duration-equal operator as part of boolean expression 'and' opera
    // XPath: (xs:duration(\"P36D\") eq xs:duration(\"P36D\")) and fn:fals
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_23() {
    //  Evaluation of duration-equal operator as part of boolean expression 'and' opera
    // XPath: (xs:duration(\"P36D\") ne xs:duration(\"P36D\")) and fn:fals
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_24() {
    //  Evaluation of duration-equal operator as part of boolean expression 'or' operat
    // XPath: (xs:duration(\"P36D\") eq xs:duration(\"P36D\")) or fn:false
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_25() {
    //  Evaluation of duration-equal operator as part of boolean expression 'or' operat
    // XPath: (xs:duration(\"P36D\") ne xs:duration(\"P36D\")) or fn:false
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_26() {
    //  Evaluation of duration-equal operator as per example 4 for this operator from t
    // XPath: xs:yearMonthDuration('P0Y') eq xs:dayTimeDuration('P0D')
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_duration_equal_27() {
    //  Evaluation of duration-equal operator as per example 5 for this operator from t
    // XPath: xs:yearMonthDuration('P1Y') eq xs:dayTimeDuration('P365D')
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_1() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as f
    // XPath: xs:yearMonthDuration(\"P0Y0M\") eq xs:yearMonthDuration(\"P0
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_2() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as f
    // XPath: xs:yearMonthDuration(\"P1000Y6M\") eq xs:yearMonthDuration(\
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_3() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as f
    // XPath: xs:yearMonthDuration(\"P2030Y12M\") eq xs:yearMonthDuration(
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_4() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as f
    // XPath: xs:yearMonthDuration(\"P0Y0M\") eq xs:yearMonthDuration(\"P1
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_5() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as f
    // XPath: xs:yearMonthDuration(\"P0Y0M\") eq xs:yearMonthDuration(\"P2
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_6() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as f
    // XPath: xs:yearMonthDuration(\"P0Y0M\") ne xs:yearMonthDuration(\"P0
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_7() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as f
    // XPath: xs:yearMonthDuration(\"P1000Y6M\") ne xs:yearMonthDuration(\
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_8() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as f
    // XPath: xs:yearMonthDuration(\"P2030Y12M\") ne xs:yearMonthDuration(
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_9() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as f
    // XPath: xs:yearMonthDuration(\"P0Y0M\") ne xs:yearMonthDuration(\"P1
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_10() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator with the arguments set as f
    // XPath: xs:yearMonthDuration(\"P0Y0M\") ne xs:yearMonthDuration(\"P2
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_11() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator (le) with the arguments set
    // XPath: xs:yearMonthDuration(\"P0Y0M\") le xs:yearMonthDuration(\"P0
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_12() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator (le) with the arguments set
    // XPath: xs:yearMonthDuration(\"P1000Y6M\") le xs:yearMonthDuration(\
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_13() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator (le) with the arguments set
    // XPath: xs:yearMonthDuration(\"P2030Y12M\") le xs:yearMonthDuration(
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_14() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator (le) with the arguments set
    // XPath: xs:yearMonthDuration(\"P0Y0M\") le xs:yearMonthDuration(\"P1
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_15() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator (le) with the arguments set
    // XPath: xs:yearMonthDuration(\"P0Y0M\") le xs:yearMonthDuration(\"P2
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_16() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator (ge) with the arguments set
    // XPath: xs:yearMonthDuration(\"P0Y0M\") ge xs:yearMonthDuration(\"P0
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_17() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator (ge) with the arguments set
    // XPath: xs:yearMonthDuration(\"P1000Y6M\") ge xs:yearMonthDuration(\
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_18() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator (ge) with the arguments set
    // XPath: xs:yearMonthDuration(\"P2030Y12M\") ge xs:yearMonthDuration(
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_19() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator (ge) with the arguments set
    // XPath: xs:yearMonthDuration(\"P0Y0M\") ge xs:yearMonthDuration(\"P1
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_yearmonthduration_equal2args_20() {
    //  Evaluates The 'op:yearMonthDuration-equal' operator (ge) with the arguments set
    // XPath: xs:yearMonthDuration(\"P0Y0M\") ge xs:yearMonthDuration(\"P2
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_daytimeduration_equal2args_1() {
    //  Evaluates The 'op:dayTimeDuration-equal' operator with the arguments set as fol
    // XPath: xs:dayTimeDuration(\"P0DT0H0M0S\") eq xs:dayTimeDuration(\"P
    // Expected: true
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_daytimeduration_equal2args_2() {
    //  Evaluates The 'op:dayTimeDuration-equal' operator with the arguments set as fol
    // XPath: xs:dayTimeDuration(\"P15DT11H59M59S\") eq xs:dayTimeDuration
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}

#[test]
fn op_daytimeduration_equal2args_3() {
    //  Evaluates The 'op:dayTimeDuration-equal' operator with the arguments set as fol
    // XPath: xs:dayTimeDuration(\"P31DT23H59M59S\") eq xs:dayTimeDuration
    // Expected: false
    // Calls stub function - will fail until op:duration-equal is implemented
    let _ = stub_opduration_equal();
}
