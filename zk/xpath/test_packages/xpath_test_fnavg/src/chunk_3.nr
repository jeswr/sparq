//! Test chunk 3 for fn:avg
//! Contains 23 tests

use dep::xpath::stub_fnavg;

#[test]
fn k_seqavgfunc_26() {
    //  A test whose essence is: `avg(xs:untypedAtomic('3')) instance of xs:double`.
    // XPath: avg(xs:untypedAtomic(\"3\")) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_27() {
    //  A test whose essence is: `avg((1, 2, xs:untypedAtomic('3'))) instance of xs:dou
    // XPath: avg((1, 2, xs:untypedAtomic(\"3\"))) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_28() {
    //  A test whose essence is: `avg((1, 2, xs:untypedAtomic('3'))) eq 2`.
    // XPath: avg((1, 2, xs:untypedAtomic(\"3\"))) eq 2
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_29() {
    //  A test whose essence is: `avg((xs:float(1), 2, xs:untypedAtomic('3'))) eq 2`.
    // XPath: avg((xs:float(1), 2, xs:untypedAtomic(\"3\"))) eq 2
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_30() {
    //  A test whose essence is: `avg((xs:float(1), 2, xs:untypedAtomic('3'))) instance
    // XPath: avg((xs:float(1), 2, xs:untypedAtomic(\"3\"))) instance of x
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_39() {
    //  A test whose essence is: `avg((xs:yearMonthDuration('P20Y'), xs:yearMonthDurati
    // XPath: avg((xs:yearMonthDuration(\"P20Y\"), xs:yearMonthDuration(\"
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_40() {
    //  A test whose essence is: `empty(avg( () ))`.
    // XPath: empty(avg( () ))
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_41() {
    //  A test whose essence is: `string(avg((xs:float('INF'), xs:float('-INF')))) eq '
    // XPath: string(avg((xs:float('INF'), xs:float('-INF')))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_42() {
    //  A test whose essence is: `string(avg(((3, 4, 5), xs:float('NaN')))) eq 'NaN'`.
    // XPath: string(avg(((3, 4, 5), xs:float('NaN')))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k_seqavgfunc_43() {
    //  A test whose essence is: `avg((3, 4, 5)) eq 4.0`.
    // XPath: avg((3, 4, 5)) eq 4.0
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k2_seqavgfunc_1() {
    //  Ensure the return type is correct when type promotion is required.
    // XPath: avg((xs:float('NaN'), 2, 3, 4, xs:double('NaN'))) instance o
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k2_seqavgfunc_2() {
    //  Ensure the return type is correct when type promotion is required(#2).
    // XPath: avg((xs:float('NaN'), 2, 3.3, 4, xs:double('NaN'))) instance
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn k2_seqavgfunc_3() {
    //  Check the average of two integers.
    // XPath: avg((1, 1))
    // Expected: 1
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_3() {
    //  Test fn:sum on a sequence of xs:dayTimeDuration arguments
    // XPath: avg((xs:dayTimeDuration(\"P1D\"), xs:dayTimeDuration(\"PT2H\
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_4() {
    //  Test fn:avg on a sequence of xs:dayTimeDuration arguments
    // XPath: avg(for $x in 1 to 10 return xs:dayTimeDuration(concat(\"PT\
    // Expected: PT5H30M
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_5() {
    //  Test fn:avg on a sequence of xs:yearMonthDuration arguments
    // XPath: avg((xs:yearMonthDuration(\"P1Y\"), xs:yearMonthDuration(\"P
    // Expected: true
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_6() {
    //  Test fn:avg on a sequence of xs:yearMonthDuration arguments
    // XPath: avg(for $x in 1 to 9 return xs:yearMonthDuration(concat(\"P\
    // Expected: P5M
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn fn_avg_10() {
    //  Test fn:avg on an array
    // XPath: avg([1,2,3,4,5])
    // Expected: 3
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn cbcl_avg_004() {
    // Test fn:avg on xs:dayTimeDurations
    // XPath: fn:avg((xs:dayTimeDuration(\"P1DT2H\"), xs:dayTimeDuration(\
    // Expected: P1D
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn cbcl_avg_006() {
    //  Test fn:avg on xs:yearMonthDurations
    // XPath: fn:avg((xs:yearMonthDuration(\"P1Y1M\"), xs:yearMonthDuratio
    // Expected: P1Y
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn cbcl_avg_008() {
    // Test fn:avg on mixed numeric types.
    // XPath: typeswitch (fn:avg((xs:float(1), xs:double(2), xs:float(3)))
    // Expected: 2
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn cbcl_avg_011() {
    // Test fn:avg returning double NaN.
    // XPath: avg((xs:double(\"NaN\"), day-from-date(current-date())))
    // Expected: NaN
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}

#[test]
fn cbcl_avg_012() {
    // Test fn:avg returning float NaN.
    // XPath: avg((xs:float(\"NaN\"), day-from-date(current-date())))
    // Expected: NaN
    // Calls stub function - will fail until fn:avg is implemented
    let _ = stub_fnavg();
}
