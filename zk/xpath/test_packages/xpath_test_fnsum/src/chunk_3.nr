//! Test chunk 3 for fn:sum
//! Contains 50 tests

use dep::xpath::stub_fnsum;

#[test]
fn fn_sumnni3args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:sum((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 0
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnni3args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:sum((xs:nonNegativeInteger(\"303884545991464527\"),xs:non
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnni3args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:sum((xs:nonNegativeInteger(\"999999999999999999\"),xs:non
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnni3args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:sum((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnni3args_5() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:sum((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnni3args_6() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:sum((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 303884545991464527
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumnni3args_7() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:nonN
    // XPath: fn:sum((xs:nonNegativeInteger(\"0\"),xs:nonNegativeInteger(\
    // Expected: 999999999999999999
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumsht3args_1() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:sum((xs:short(\"-5324\"),xs:short(\"-27444\"),xs:short(\"
    // Expected: -60212
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumsht3args_2() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:sum((xs:short(\"-27444\"),xs:short(\"-5324\"),xs:short(\"
    // Expected: -32768
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumsht3args_3() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:sum((xs:short(\"0\"),xs:short(\"0\"),xs:short(\"-5324\"))
    // Expected: -5324
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sumsht3args_4() {
    //  Evaluates The 'sum' function with the arguments set as follows: $arg1 = xs:shor
    // XPath: fn:sum((xs:short(\"0\"),xs:short(\"0\"),xs:short(\"32767\"))
    // Expected: 32767
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_3() {
    //  A test whose essence is: `sum((), 3) eq 3`.
    // XPath: sum((), 3) eq 3
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_4() {
    //  A test whose essence is: `sum((), 3) instance of xs:integer`.
    // XPath: sum((), 3) instance of xs:integer
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_5() {
    //  A test whose essence is: `empty(sum((), ()))`.
    // XPath: empty(sum((), ()))
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_6() {
    //  A test whose essence is: `sum((), 0.0) instance of xs:decimal`.
    // XPath: sum((), 0.0) instance of xs:decimal
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_7() {
    //  A test whose essence is: `sum((), 0.0) eq 0.0`.
    // XPath: sum((), 0.0) eq 0.0
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_8() {
    //  A test whose essence is: `sum(()) eq 0`.
    // XPath: sum(()) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_9() {
    //  A test whose essence is: `sum(()) instance of xs:integer`.
    // XPath: sum(()) instance of xs:integer
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_10() {
    //  A test whose essence is: `sum((-5, -0, -3, -6)) eq -14`.
    // XPath: sum((-5, -0, -3, -6)) eq -14
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_11() {
    //  A test whose essence is: `string(sum((1, 2, 3, xs:float('NaN')))) eq 'NaN'`.
    // XPath: string(sum((1, 2, 3, xs:float(\"NaN\")))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_12() {
    //  A test whose essence is: `string(sum((1, 2, 3, xs:double('NaN')))) eq 'NaN'`.
    // XPath: string(sum((1, 2, 3, xs:double(\"NaN\")))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_13() {
    //  A test whose essence is: `string(sum((xs:double('NaN'), 1, 2, 3))) eq 'NaN'`.
    // XPath: string(sum((xs:double(\"NaN\"), 1, 2, 3))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_14() {
    //  A test whose essence is: `string(sum((xs:float('NaN'), 1, 2, 3))) eq 'NaN'`.
    // XPath: string(sum((xs:float(\"NaN\"), 1, 2, 3))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_15() {
    //  A test whose essence is: `string(sum((1, 2, xs:double('NaN'), 1, 2, 3))) eq 'Na
    // XPath: string(sum((1, 2, xs:double(\"NaN\"), 1, 2, 3))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_16() {
    //  A test whose essence is: `string(sum((1, 2, xs:float('NaN'), 1, 2, 3))) eq 'NaN
    // XPath: string(sum((1, 2, xs:float(\"NaN\"), 1, 2, 3))) eq \"NaN\"
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_17() {
    //  A test whose essence is: `sum(xs:untypedAtomic('3')) instance of xs:double`.
    // XPath: sum(xs:untypedAtomic(\"3\")) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_18() {
    //  A test whose essence is: `sum((1, 2, xs:untypedAtomic('3'))) instance of xs:dou
    // XPath: sum((1, 2, xs:untypedAtomic(\"3\"))) instance of xs:double
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_19() {
    //  A test whose essence is: `sum((1, 2, xs:untypedAtomic('3'))) eq 6`.
    // XPath: sum((1, 2, xs:untypedAtomic(\"3\"))) eq 6
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_20() {
    //  A test whose essence is: `sum((xs:float(1), 2, xs:untypedAtomic('3'))) eq 6`.
    // XPath: sum((xs:float(1), 2, xs:untypedAtomic(\"3\"))) eq 6
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_21() {
    //  A test whose essence is: `sum((xs:float(1), 2, xs:untypedAtomic('3'))) instance
    // XPath: sum((xs:float(1), 2, xs:untypedAtomic(\"3\"))) instance of x
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_22() {
    //  fn:sum() having an input sequence whose static type is xs:anyAtomicType.
    // XPath: sum(remove((1.0, xs:float(1), 2, xs:untypedAtomic(\"3\")), 1
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_27() {
    //  A test whose essence is: `sum(((),())) eq 0`.
    // XPath: sum(((),())) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_28() {
    //  A test whose essence is: `sum(()) eq 0`.
    // XPath: sum(()) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_29() {
    //  A test whose essence is: `sum((3, 4, 5)) eq 12`.
    // XPath: sum((3, 4, 5)) eq 12
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_30() {
    //  A test whose essence is: `sum((xs:yearMonthDuration('P20Y'), xs:yearMonthDurati
    // XPath: sum((xs:yearMonthDuration(\"P20Y\"), xs:yearMonthDuration(\"
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_31() {
    //  A test whose essence is: `sum((xs:yearMonthDuration('P20Y'), xs:yearMonthDurati
    // XPath: sum((xs:yearMonthDuration(\"P20Y\"), xs:yearMonthDuration(\"
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k_seqsumfunc_32() {
    //  A test whose essence is: `sum((1 to 100)[. < 0], 0) eq 0`.
    // XPath: sum((1 to 100)[. < 0], 0) eq 0
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k2_seqsumfunc_2() {
    //  Ensure the return type is correct when type promotion is required.
    // XPath: sum((xs:float('NaN'), 2, 3, 4, xs:double('NaN'))) instance o
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k2_seqsumfunc_3() {
    //  Ensure the return type is correct when type promotion is required(#2).
    // XPath: sum((xs:float('NaN'), 2, 3.3, 4, xs:double('NaN'))) instance
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn k2_seqsumfunc_4() {
    //  Ensure the return type is properly inferred.
    // XPath: sum(xs:unsignedShort(\"1\")) instance of xs:unsignedShort
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sum_1() {
    //  Test fn:sum on a sequence of xs:dayTimeDuration arguments
    // XPath: sum((xs:dayTimeDuration(\"P1D\"), xs:dayTimeDuration(\"PT1H\
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sum_2() {
    //  Test fn:sum on a sequence of xs:dayTimeDuration arguments
    // XPath: sum((), xs:dayTimeDuration(\"PT0S\"))
    // Expected: PT0S
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sum_3() {
    //  Test fn:sum on a sequence of xs:dayTimeDuration arguments
    // XPath: sum(for $x in 1 to 10 return xs:dayTimeDuration(concat(\"PT\
    // Expected: P2DT7H
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sum_4() {
    //  Test fn:sum on a sequence of xs:yearMonthDuration arguments
    // XPath: sum((xs:yearMonthDuration(\"P1Y\"), xs:yearMonthDuration(\"P
    // Expected: true
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sum_5() {
    //  Test fn:sum on a sequence of xs:yearMonthDuration arguments
    // XPath: sum((), xs:yearMonthDuration(\"P0M\"))
    // Expected: P0M
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sum_6() {
    //  Test fn:sum on a sequence of xs:yearMonthDuration arguments
    // XPath: sum(for $x in 1 to 10 return xs:yearMonthDuration(concat(\"P
    // Expected: P4Y7M
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sum_8() {
    //  Test fn:sum with $zero as xs:duration
    // XPath: sum((), xs:duration(\"P0M\"))
    // Expected: PT0S
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sum_11() {
    //  Test fn:sum on an array
    // XPath: sum([1,2,3,4,5])
    // Expected: 15
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sum_12() {
    //  Second argument is ignored if sequence is not empty
    // XPath: sum((1 to 5)[. lt year-from-date(current-date())], 17)
    // Expected: 15
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}

#[test]
fn fn_sum_13() {
    //  Second argument is ignored if sequence is not empty, even if it has a crazy typ
    // XPath: sum((1 to 5)[. lt year-from-date(current-date())], \"bonkers
    // Expected: 15
    // Calls stub function - will fail until fn:sum is implemented
    let _ = stub_fnsum();
}
