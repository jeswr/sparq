//! Test chunk 0 for fn:current-date
//! Contains 15 tests

use dep::xpath::stub_fncurrent_date;

#[test]
fn fn_current_date_5() {
    //  Evaluates The 'current-date' function as part of a '-' operation.
    // XPath: fn:current-date() - fn:current-date()
    // Expected: PT0S
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}

#[test]
fn fn_current_date_9() {
    //  Evaluates The 'current-date' function as part of of a subtraction expression. B
    // XPath: fn:current-date() - fn:current-date()
    // Expected: PT0S
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}

#[test]
fn fn_current_date_12() {
    //  Evaluates string value The 'current-date' as part of an equal expression (eq op
    // XPath: fn:string(fn:current-date()) eq fn:string(fn:current-date())
    // Expected: true
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}

#[test]
fn fn_current_date_13() {
    //  Evaluates The 'current-date' function as part of an equal expression (ne operat
    // XPath: fn:string(fn:current-date()) ne fn:string(fn:current-date())
    // Expected: false
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}

#[test]
fn fn_current_date_14() {
    //  Evaluates The 'current-date' function as part of an equal expression (le operat
    // XPath: fn:string(fn:current-date()) le fn:string(fn:current-date())
    // Expected: true
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}

#[test]
fn fn_current_date_15() {
    //  Evaluates The 'current-date' function as part of an equal expression (ge operat
    // XPath: fn:string(fn:current-date()) ge fn:string(fn:current-date())
    // Expected: true
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}

#[test]
fn fn_current_date_16() {
    //  Evaluates The 'current-date' function as part of a boolean expression ('and' op
    // XPath: fn:string(fn:current-date()) and fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}

#[test]
fn fn_current_date_17() {
    //  Evaluates The 'current-date' function as part of a boolean expression ('and' op
    // XPath: fn:string(fn:current-date()) and fn:false()
    // Expected: false
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}

#[test]
fn fn_current_date_18() {
    //  Evaluates The 'current-date' function as part of a boolean expression ('or' ope
    // XPath: fn:string(fn:current-date()) or fn:true()
    // Expected: true
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}

#[test]
fn fn_current_date_19() {
    //  Evaluates The 'current-date' function as part of a boolean expression ('or' ope
    // XPath: fn:string(fn:current-date()) or fn:false()
    // Expected: true
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}

#[test]
fn fn_current_date_20() {
    //  Evaluates The 'current-date' function (string value)as an argument to the fn:no
    // XPath: fn:not(fn:string(fn:current-date()))
    // Expected: false
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}

#[test]
fn fn_current_date_22() {
    // Test that current date is after 1 Jan 2010
    // XPath: fn:current-date() gt xs:date('2010-01-01')
    // Expected: true
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}

#[test]
fn fn_current_date_23() {
    // Test that current date has a timezone
    // XPath: exists(timezone-from-date(current-date()))
    // Expected: true
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}

#[test]
fn fn_current_time_24() {
    // Test that the current date is the same as the date part of current dateTime
    // XPath: current-date() = xs:date(current-dateTime())
    // Expected: true
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}

#[test]
fn k_contextcurrentdatefunc_2() {
    //  Test that the Dynamic Context property 'current dateTime' when presented as a x
    // XPath: current-date() eq current-date()
    // Expected: true
    // Calls stub function - will fail until fn:current-date is implemented
    let _ = stub_fncurrent_date();
}
