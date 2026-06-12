//! Test chunk 0 for fn:current-time
//! Contains 14 tests

use dep::xpath::stub_fncurrent_time;

#[test]
fn fn_current_time_5() {
    //  Evaluates The 'current-time' function as part of a '-' operation.
    // XPath: fn:current-time() - fn:current-time()
    // Expected: PT0S
    // Calls stub function - will fail until fn:current-time is implemented
    let _ = stub_fncurrent_time();
}

#[test]
fn fn_current_time_9() {
    //  Evaluates The 'current-time' function as part of of a subtraction expression. B
    // XPath: fn:current-time() - fn:current-time()
    // Expected: PT0S
    // Calls stub function - will fail until fn:current-time is implemented
    let _ = stub_fncurrent_time();
}

#[test]
fn fn_current_time_12() {
    //  Evaluates string value The 'current-time' as part of a 'numeric-equal' expressi
    // XPath: fn:string(fn:current-time()) eq fn:string(fn:current-time())
    // Expected: true
    // Calls stub function - will fail until fn:current-time is implemented
    let _ = stub_fncurrent_time();
}

#[test]
fn fn_current_time_13() {
    //  Evaluates The 'current-time' function as part of an equal expression (ne operat
    // XPath: fn:string(fn:current-time()) ne fn:string(fn:current-time())
    // Expected: false
    // Calls stub function - will fail until fn:current-time is implemented
    let _ = stub_fncurrent_time();
}

#[test]
fn fn_current_time_14() {
    //  Evaluates The 'current-time' function as part of an equal expression (le operat
    // XPath: fn:string(fn:current-time()) le fn:string(fn:current-time())
    // Expected: true
    // Calls stub function - will fail until fn:current-time is implemented
    let _ = stub_fncurrent_time();
}

#[test]
fn fn_current_time_15() {
    //  Evaluates The 'current-time' function as part of an equal expression (ge operat
    // XPath: fn:string(fn:current-time()) ge fn:string(fn:current-time())
    // Expected: true
    // Calls stub function - will fail until fn:current-time is implemented
    let _ = stub_fncurrent_time();
}

#[test]
fn fn_current_time_16() {
    //  Evaluates The 'current-time' function as part of a boolean expression ('and' op
    // XPath: fn:string(fn:current-time()) and fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:current-time is implemented
    let _ = stub_fncurrent_time();
}

#[test]
fn fn_current_time_17() {
    //  Evaluates The 'current-time' function as part of a boolean expression ('and' op
    // XPath: fn:string(fn:current-time()) and fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:current-time is implemented
    let _ = stub_fncurrent_time();
}

#[test]
fn fn_current_time_18() {
    //  Evaluates The 'current-time' function as part of a boolean expression ('or' ope
    // XPath: fn:string(fn:current-time()) or fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:current-time is implemented
    let _ = stub_fncurrent_time();
}

#[test]
fn fn_current_time_19() {
    //  Evaluates The 'current-time' function as part of a boolean expression ('or' ope
    // XPath: fn:string(fn:current-time()) or fn:false()
    // Expected: true
    // Calls stub function - will fail until fn:current-time is implemented
    let _ = stub_fncurrent_time();
}

#[test]
fn fn_current_time_20() {
    //  Evaluates The 'current-time' function (string value)as an argument to the fn:no
    // XPath: fn:not(fn:string(fn:current-time()))
    // Expected: false
    // Calls stub function - will fail until fn:current-time is implemented
    let _ = stub_fncurrent_time();
}

#[test]
fn fn_current_time_21() {
    // Test that the current time has a timezone
    // XPath: exists(timezone-from-time(current-time()))
    // Expected: true
    // Calls stub function - will fail until fn:current-time is implemented
    let _ = stub_fncurrent_time();
}

#[test]
fn fn_current_time_22() {
    // Test that the current time is the same as the time part of current dateTime
    // XPath: current-time() = xs:time(current-dateTime())
    // Expected: true
    // Calls stub function - will fail until fn:current-time is implemented
    let _ = stub_fncurrent_time();
}

#[test]
fn k_contextcurrenttimefunc_2() {
    //  Test that the Dynamic Context property 'current dateTime' when presented as a x
    // XPath: current-time() eq current-time()
    // Expected: true
    // Calls stub function - will fail until fn:current-time is implemented
    let _ = stub_fncurrent_time();
}
