//! Test chunk 0 for fn:current-dateTime
//! Contains 13 tests

use dep::xpath::stub_fncurrent_datetime;

#[test]
fn fn_current_datetime_5() {
    // dateTime: December 1, 2005  Evaluates The 'current-dateTime' function as part of
    // XPath: fn:current-dateTime() - fn:current-dateTime()
    // Expected: PT0S
    // Calls stub function - will fail until fn:current-dateTime is implemented
    let _ = stub_fncurrent_datetime();
}

#[test]
fn fn_current_datetime_9() {
    // dateTime: December 5, 2005  Evaluates The 'current-dateTime' function as part of
    // XPath: fn:current-dateTime() - fn:current-dateTime()
    // Expected: PT0S
    // Calls stub function - will fail until fn:current-dateTime is implemented
    let _ = stub_fncurrent_datetime();
}

#[test]
fn fn_current_datetime_12() {
    // dateTime: December 5, 2005  Evaluates string value The 'current-dateTime' as par
    // XPath: fn:string(fn:current-dateTime()) eq fn:string(fn:current-dat
    // Expected: true
    // Calls stub function - will fail until fn:current-dateTime is implemented
    let _ = stub_fncurrent_datetime();
}

#[test]
fn fn_current_datetime_13() {
    // dateTime: December 5, 2005  Evaluates The 'current-dateTime' function as part of
    // XPath: fn:string(fn:current-dateTime()) ne fn:string(fn:current-dat
    // Expected: false
    // Calls stub function - will fail until fn:current-dateTime is implemented
    let _ = stub_fncurrent_datetime();
}

#[test]
fn fn_current_datetime_14() {
    // dateTime: December 5, 2005  Evaluates The 'current-dateTime' function as part of
    // XPath: fn:string(fn:current-dateTime()) le fn:string(fn:current-dat
    // Expected: true
    // Calls stub function - will fail until fn:current-dateTime is implemented
    let _ = stub_fncurrent_datetime();
}

#[test]
fn fn_current_datetime_15() {
    // dateTime: December 5, 2005  Evaluates The 'current-dateTime' function as part of
    // XPath: fn:string(fn:current-dateTime()) ge fn:string(fn:current-dat
    // Expected: true
    // Calls stub function - will fail until fn:current-dateTime is implemented
    let _ = stub_fncurrent_datetime();
}

#[test]
fn fn_current_datetime_16() {
    // dateTime: December 5, 2005  Evaluates The 'current-dateTime' function as part of
    // XPath: fn:string(fn:current-dateTime()) and fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:current-dateTime is implemented
    let _ = stub_fncurrent_datetime();
}

#[test]
fn fn_current_datetime_17() {
    // dateTime: December 5, 2005  Evaluates The 'current-dateTime' function as part of
    // XPath: fn:string(fn:current-dateTime()) and fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:current-dateTime is implemented
    let _ = stub_fncurrent_datetime();
}

#[test]
fn fn_current_datetime_18() {
    // dateTime: December 5, 2005  Evaluates The 'current-dateTime' function as part of
    // XPath: fn:string(fn:current-dateTime()) or fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:current-dateTime is implemented
    let _ = stub_fncurrent_datetime();
}

#[test]
fn fn_current_datetime_19() {
    // dateTime: December 5, 2005  Evaluates The 'current-dateTime' function as part of
    // XPath: fn:string(fn:current-dateTime()) or fn:false()
    // Expected: true
    // Calls stub function - will fail until fn:current-dateTime is implemented
    let _ = stub_fncurrent_datetime();
}

#[test]
fn fn_current_datetime_20() {
    // dateTime: December 5, 2005  Evaluates The 'current-dateTime' function (string va
    // XPath: fn:not(fn:string(fn:current-dateTime()))
    // Expected: false
    // Calls stub function - will fail until fn:current-dateTime is implemented
    let _ = stub_fncurrent_datetime();
}

#[test]
fn fn_current_datetime_25() {
    // Test that the current dateTime has a timezone
    // XPath: exists(timezone-from-dateTime(current-dateTime()))
    // Expected: true
    // Calls stub function - will fail until fn:current-dateTime is implemented
    let _ = stub_fncurrent_datetime();
}

#[test]
fn k_contextcurrentdatetimefunc_2() {
    //  Test that the Dynamic Context property 'current dateTime' when presented as a x
    // XPath: current-dateTime() eq current-dateTime()
    // Expected: true
    // Calls stub function - will fail until fn:current-dateTime is implemented
    let _ = stub_fncurrent_datetime();
}
