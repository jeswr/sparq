//! Test chunk 0 for op:divide-dayTimeDuration-by-dayTimeDuration
//! Contains 6 tests

use dep::xpath::{duration_divide_by_duration, duration_from_microseconds, XsdDouble};

#[test]
fn op_divide_daytimeduration_by_daytimeduration2args_1() {
    // Test: op-divide-dayTimeDuration-by-dayTimeDuration2args-1
    // Written By: Carmelo Montanez
    // Date: Tue Apr 12 16:29:08 GMT-05:00 2005
    // Purpose: Evaluates The 'op:divide-dayTimeDuration-by-dayTimeDuration' operator
    // with the arguments set as follows:
    // $arg1 = xs:dayTimeDuration(lower bound)
    // $arg2 = xs:dayTimeDuration(lower bound)
    // XPath: xs:dayTimeDuration("P0DT0H0M0S") div xs:dayTimeDuration("P0DT0H0M01S")
    // Expected: assert-eq 0
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(1000000);
    assert(duration_divide_by_duration(dur1, dur2) == XsdDouble::zero());
}

#[test]
fn op_divide_daytimeduration_by_daytimeduration2args_2() {
    // Test: op-divide-dayTimeDuration-by-dayTimeDuration2args-2
    // Written By: Carmelo Montanez
    // Date: Tue Apr 12 16:29:08 GMT-05:00 2005
    // Purpose: Evaluates The 'op:divide-dayTimeDuration-by-dayTimeDuration' operator
    // with the arguments set as follows:
    // $arg1 = xs:dayTimeDuration(mid range)
    // $arg2 = xs:dayTimeDuration(lower bound)
    // XPath: xs:dayTimeDuration("P15DT11H59M59S") div xs:dayTimeDuration("P0DT0H0M01S")
    // Expected: assert-eq 1339199
    let dur1 = duration_from_microseconds(1339199000000);
    let dur2 = duration_from_microseconds(1000000);
    assert(duration_divide_by_duration(dur1, dur2).to_bits() == 4698502627627892736);
}

#[test]
fn op_divide_daytimeduration_by_daytimeduration2args_3() {
    // Test: op-divide-dayTimeDuration-by-dayTimeDuration2args-3
    // Written By: Carmelo Montanez
    // Date: Tue Apr 12 16:29:08 GMT-05:00 2005
    // Purpose: Evaluates The 'op:divide-dayTimeDuration-by-dayTimeDuration' operator
    // with the arguments set as follows:
    // $arg1 = xs:dayTimeDuration(upper bound)
    // $arg2 = xs:dayTimeDuration(lower bound)
    // XPath: xs:dayTimeDuration("P31DT23H59M59S") div xs:dayTimeDuration("P0DT0H0M01S")
    // Expected: assert-eq 2764799
    let dur1 = duration_from_microseconds(2764799000000);
    let dur2 = duration_from_microseconds(1000000);
    assert(duration_divide_by_duration(dur1, dur2).to_bits() == 4703191771989934080);
}

#[test]
fn op_divide_daytimeduration_by_daytimeduration2args_4() {
    // Test: op-divide-dayTimeDuration-by-dayTimeDuration2args-4
    // Written By: Carmelo Montanez
    // Date: Tue Apr 12 16:29:08 GMT-05:00 2005
    // Purpose: Evaluates The 'op:divide-dayTimeDuration-by-dayTimeDuration' operator
    // with the arguments set as follows:
    // $arg1 = xs:dayTimeDuration(lower bound)
    // $arg2 = xs:dayTimeDuration(mid range)
    // XPath: xs:dayTimeDuration("P0DT0H0M0S") div xs:dayTimeDuration("P15DT11H59M59S")
    // Expected: assert-eq 0
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(1339199000000);
    assert(duration_divide_by_duration(dur1, dur2) == XsdDouble::zero());
}

#[test]
fn op_divide_daytimeduration_by_daytimeduration2args_5() {
    // Test: op-divide-dayTimeDuration-by-dayTimeDuration2args-5
    // Written By: Carmelo Montanez
    // Date: Tue Apr 12 16:29:08 GMT-05:00 2005
    // Purpose: Evaluates The 'op:divide-dayTimeDuration-by-dayTimeDuration' operator
    // with the arguments set as follows:
    // $arg1 = xs:dayTimeDuration(lower bound)
    // $arg2 = xs:dayTimeDuration(upper bound)
    // XPath: xs:dayTimeDuration("P0DT0H0M0S") div xs:dayTimeDuration("P31DT23H59M59S")
    // Expected: assert-eq 0
    let dur1 = duration_from_microseconds(0);
    let dur2 = duration_from_microseconds(2764799000000);
    assert(duration_divide_by_duration(dur1, dur2) == XsdDouble::zero());
}

#[test]
fn op_divide_daytimeduration_by_dtd_8() {
    // Test: op-divide-dayTimeDuration-by-dTD-8
    // Written By: Carmelo Montanez
    // Date: June 30, 2005
    // Purpose: Evaluates The 'divide-dayTimeDuration-by-dTD' operator that
    // returns a negative value.
    // XPath: (xs:dayTimeDuration("P10DT01H01M") div xs:dayTimeDuration("-P10DT01H01M"))
    // Expected: assert-eq -1
    let dur1 = duration_from_microseconds(867660000000);
    let dur2 = duration_from_microseconds(-867660000000);
    assert(duration_divide_by_duration(dur1, dur2).to_bits() == 13830554455654793216);
}
