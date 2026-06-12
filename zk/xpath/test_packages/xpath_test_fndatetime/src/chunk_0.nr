//! Test chunk 0 for fn:dateTime
//! Contains 45 tests

use dep::xpath::stub_fndatetime;

#[test]
fn fn_datetime_1() {
    // Evaluation of 'fn:dateTime' function as per example 1 of the F & O Specs. for th
    // XPath: fn:dateTime(xs:date(\"1999-12-31\"), xs:time(\"12:00:00\"))
    // Expected: 1999-12-31T12:00:00
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_2() {
    // Evaluation of 'fn:dateTime' function as per example 2 of the F & O Specs. for th
    // XPath: fn:dateTime(xs:date(\"1999-12-31\"), xs:time(\"24:00:00\"))
    // Expected: 1999-12-31T00:00:00
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_3() {
    // Evaluation of 'fn:dateTime' function, where only the first argument have a timez
    // XPath: fn:dateTime(xs:date(\"1999-12-31Z\"), xs:time(\"23:00:00\"))
    // Expected: 1999-12-31T23:00:00Z
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_4() {
    // Evaluation of 'fn:dateTime' function, where only the second argument have a time
    // XPath: fn:dateTime(xs:date(\"1999-12-31\"), xs:time(\"23:00:00Z\"))
    // Expected: 1999-12-31T23:00:00Z
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_5() {
    // Evaluation of 'fn:dateTime' function, where both arguments have the same timezon
    // XPath: fn:dateTime(xs:date(\"1999-12-31Z\"), xs:time(\"23:00:00Z\")
    // Expected: 1999-12-31T23:00:00Z
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_6() {
    // Evaluation of 'fn:dateTime' function, where both arguments have the same timezon
    // XPath: fn:dateTime(xs:date(\"1999-12-31+10:00\"), xs:time(\"23:00:0
    // Expected: 1999-12-31T23:00:00+10:00
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_7() {
    // Evaluation of 'fn:dateTime' function together together with the  op:dateTime-equ
    // XPath: fn:dateTime(xs:date(\"1999-12-31+10:00\"), xs:time(\"23:00:0
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_8() {
    // Evaluation of 'fn:dateTime' function together together with the  op:dateTime-equ
    // XPath: fn:dateTime(xs:date(\"1999-12-31+10:00\"), xs:time(\"23:00:0
    // Expected: false
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_9() {
    // Evaluation of 'fn:dateTime' function together together with the  op:dateTime-equ
    // XPath: fn:dateTime(xs:date(\"1999-12-31+10:00\"), xs:time(\"23:00:0
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_10() {
    // Evaluation of 'fn:dateTime' function together together with the  op:dateTime-equ
    // XPath: fn:dateTime(xs:date(\"1999-12-31+10:00\"), xs:time(\"23:00:0
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_11() {
    // Evaluation of 'fn:dateTime' function together together with the  op:dateTime-equ
    // XPath: fn:dateTime(xs:date(\"1999-12-31+10:00\"), xs:time(\"23:00:0
    // Expected: false
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_12() {
    // Evaluation of 'fn:dateTime' function together together with the  op:dateTime-equ
    // XPath: fn:dateTime(xs:date(\"1999-12-31+10:00\"), xs:time(\"23:00:0
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_13() {
    // Evaluation of 'fn:dateTime' function together together with the  op:dateTime-equ
    // XPath: fn:dateTime(xs:date(\"1999-12-31+10:00\"), xs:time(\"23:00:0
    // Expected: false
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_14() {
    // Evaluation of 'fn:dateTime' function together together with the  op:dateTime-equ
    // XPath: fn:dateTime(xs:date(\"1999-12-31+10:00\"), xs:time(\"23:00:0
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_15() {
    // Evaluation of 'fn:dateTime' as an argument to the function 'year-from-dateTime'.
    // XPath: fn:year-from-dateTime(fn:dateTime(xs:date(\"1999-12-31+10:00
    // Expected: 1999
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_16() {
    // Evaluation of 'fn:dateTime' as an argument to the function 'month-from-dateTime'
    // XPath: fn:month-from-dateTime(fn:dateTime(xs:date(\"1999-12-31+10:0
    // Expected: 12
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_17() {
    // Evaluation of 'fn:dateTime' as an argument to the function 'day-from-dateTime'.
    // XPath: fn:day-from-dateTime(fn:dateTime(xs:date(\"1999-12-31+10:00\
    // Expected: 31
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_18() {
    // Evaluation of 'fn:dateTime' as an argument to the function 'hours-from-dateTime'
    // XPath: fn:hours-from-dateTime(fn:dateTime(xs:date(\"1999-12-31+10:0
    // Expected: 23
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_19() {
    // Evaluation of 'fn:dateTime' as an argument to the function 'minutes-from-dateTim
    // XPath: fn:minutes-from-dateTime(fn:dateTime(xs:date(\"1999-12-31+10
    // Expected: 0
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_20() {
    // Evaluation of 'fn:dateTime' as an argument to the function 'seconds-from-dateTim
    // XPath: fn:seconds-from-dateTime(fn:dateTime(xs:date(\"1999-12-31+10
    // Expected: 0
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_21() {
    // Evaluation of 'fn:dateTime' as an argument to the function 'timezone-from-dateTi
    // XPath: fn:timezone-from-dateTime(fn:dateTime(xs:date(\"1999-12-31+1
    // Expected: PT10H
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_22() {
    // Evaluation of 'fn:dateTime' as an argument to the function 'adjust-dateTime-to-t
    // XPath: fn:adjust-dateTime-to-timezone(fn:dateTime(xs:date(\"1999-12
    // Expected: 1999-12-31T23:00:00+10:00
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_23() {
    // Evaluation of 'fn:dateTime' as part of a subtraction operation.
    // XPath: fn:dateTime(xs:date(\"1999-12-31+10:00\"), xs:time(\"23:00:0
    // Expected: PT1H
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_24() {
    // Evaluation of 'fn:dateTime' used together with the 'op:add-yearMonthDuration-to-
    // XPath: fn:dateTime(xs:date(\"1999-12-31+10:00\"), xs:time(\"23:00:0
    // Expected: 2001-02-28T23:00:00+10:00
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_25() {
    // Evaluation of 'fn:dateTime' used together with the 'op:add-dayTimeDuration-to-da
    // XPath: fn:dateTime(xs:date(\"1999-12-31+10:00\"), xs:time(\"23:00:0
    // Expected: 2000-01-04T00:15:00+10:00
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_26() {
    // Evaluation of 'fn:dateTime' used together with the 'subtract-yearMonthDuration-f
    // XPath: fn:dateTime(xs:date(\"1999-12-31+10:00\"), xs:time(\"23:00:0
    // Expected: 1998-10-31T23:00:00+10:00
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_27() {
    // Evaluation of 'fn:dateTime' used together with the 'subtract-dayTimeDuration-fro
    // XPath: fn:dateTime(xs:date(\"1999-12-31+10:00\"), xs:time(\"23:00:0
    // Expected: 1999-12-28T21:45:00+10:00
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_28() {
    // Evaluation of 'fn:dateTime' where first argument is empty sequence
    // XPath: fn:count(fn:dateTime((), xs:time(\"23:00:00+10:00\")))
    // Expected: 0
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_29() {
    // Evaluation of 'fn:dateTime' where second argument is empty sequence
    // XPath: fn:count(fn:dateTime(xs:date(\"1999-12-31+10:00\"), ()))
    // Expected: 0
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn fn_datetime_30() {
    // Evaluation of 'fn:dateTime' where both arguments are equal to the empty sequence
    // XPath: fn:count(fn:dateTime((),()))
    // Expected: 0
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn k_datetimefunc_4() {
    // Passing the empty sequence as second argument is allowed(recent change in the sp
    // XPath: empty(dateTime(xs:date(\"2004-03-04\"), ()))
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn k_datetimefunc_5() {
    //  Passing the empty sequence as first argument is allowed(recent change in the sp
    // XPath: empty(dateTime((), xs:time(\"08:05:23\")))
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn k_datetimefunc_8() {
    //  Invoke fn:dateTime() with timezones +00:00 and -00:00.
    // XPath: dateTime(xs:date(\"2004-03-04-00:00\"), xs:time(\"08:05:23+0
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn k_datetimefunc_9() {
    // Invoke fn:dateTime() with identical timezones.
    // XPath: dateTime(xs:date(\"2004-03-04+11:00\"), xs:time(\"08:05:23+1
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn k_datetimefunc_10() {
    // Invoke fn:dateTime() with the first value having no timezone.
    // XPath: dateTime(xs:date(\"2004-03-04\"), xs:time(\"08:05:23-05:00\"
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn k_datetimefunc_11() {
    // Invoke fn:dateTime() with the second value having no timezone.
    // XPath: dateTime(xs:date(\"2004-03-04+11:00\"), xs:time(\"08:05:23\"
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn k_datetimefunc_12() {
    // Invoke fn:dateTime() with the time value being 24:00:00.
    // XPath: xs:string(dateTime(xs:date(\"1999-12-31\"), xs:time(\"24:00:
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn k_datetimefunc_13() {
    //  Invoke fn:dateTime() with timezones being canonical UTC('Z').
    // XPath: dateTime(xs:date(\"2004-03-04Z\"), xs:time(\"08:05:23Z\")) e
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn k_datetimefunc_14() {
    // Invoke fn:dateTime() with the first value's timezone being canonical UTC('Z').
    // XPath: dateTime(xs:date(\"2004-03-04Z\"), xs:time(\"08:05:23\")) eq
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn k_datetimefunc_15() {
    // Invoke fn:dateTime() with the first value's timezone being an arbitrary value.
    // XPath: dateTime(xs:date(\"2004-03-04+13:07\"), xs:time(\"08:05:23\"
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn k_datetimefunc_16() {
    // Invoke fn:dateTime() with the second value's timezone being an arbitrary value.
    // XPath: dateTime(xs:date(\"2004-03-04\"), xs:time(\"08:05:23+13:07\"
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn k_datetimefunc_17() {
    // Invoke fn:dateTime() where none of its values has a timezone.
    // XPath: dateTime(xs:date(\"2004-03-04\"), xs:time(\"08:05:23\")) eq
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn k_datetimefunc_18() {
    // Invoke fn:dateTime() with the first value's timezone being canonical UTC('Z').
    // XPath: dateTime(xs:date(\"2004-03-04\"), xs:time(\"08:05:23Z\")) eq
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn k2_datetimefunc_1() {
    // Passing the empty sequence as both arguments is allowed(recent change in the spe
    // XPath: empty(dateTime((), ()))
    // Expected: true
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}

#[test]
fn cbcl_datetime_001() {
    //  test evaluation of fn:dateTime to exactly one item Author: Tim Mills
    // XPath: fn:index-of( ( 'A', fn:current-dateTime(), 'B' ), fn:dateTim
    // Expected: 2
    // Calls stub function - will fail until fn:dateTime is implemented
    let _ = stub_fndatetime();
}
