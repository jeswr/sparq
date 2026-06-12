//! Test chunk 0 for fn:implicit-timezone
//! Contains 7 tests

use dep::xpath::stub_fnimplicit_timezone;

#[test]
fn fn_implicit_timezone_4() {
    //  Evaluation of 'fn:implicit-timezone' as part of a subtraction operation.
    // XPath: fn:string(fn:implicit-timezone() - fn:implicit-timezone())
    // Expected: PT0S
    // Calls stub function - will fail until fn:implicit-timezone is implemented
    let _ = stub_fnimplicit_timezone();
}

#[test]
fn fn_implicit_timezone_7() {
    //  Evaluation of 'fn:implicit-timezone' as part of a multiplication operation. Sec
    // XPath: fn:string(fn:implicit-timezone() * 0)
    // Expected: PT0S
    // Calls stub function - will fail until fn:implicit-timezone is implemented
    let _ = stub_fnimplicit_timezone();
}

#[test]
fn fn_implicit_timezone_8() {
    //  Evaluation of 'fn:implicit-timezone' as part of a multiplication operation. Sec
    // XPath: fn:string(fn:implicit-timezone() * -0)
    // Expected: PT0S
    // Calls stub function - will fail until fn:implicit-timezone is implemented
    let _ = stub_fnimplicit_timezone();
}

#[test]
fn fn_implicit_timezone_13() {
    //  Evaluation of 'fn:implicit-timezone' as part of a division operation. Both oper
    // XPath: (implicit-timezone() + xs:dayTimeDuration('PT1S')) div (impl
    // Expected: 1
    // Calls stub function - will fail until fn:implicit-timezone is implemented
    let _ = stub_fnimplicit_timezone();
}

#[test]
fn fn_implicit_timezone_24() {
    // Test that implicit timezone is in range -14h to +14h
    // XPath: implicit-timezone() ge xs:dayTimeDuration('-PT14H') and impl
    // Expected: true
    // Calls stub function - will fail until fn:implicit-timezone is implemented
    let _ = stub_fnimplicit_timezone();
}

#[test]
fn k_contextimplicittimezonefunc_2() {
    //  Simple test of implicit-timezone().
    // XPath: seconds-from-duration(implicit-timezone()) le 0 or seconds-f
    // Expected: true
    // Calls stub function - will fail until fn:implicit-timezone is implemented
    let _ = stub_fnimplicit_timezone();
}

#[test]
fn k_contextimplicittimezonefunc_3() {
    //  Test that implicit-timezone() do return a value.
    // XPath: exists(seconds-from-duration(implicit-timezone()))
    // Expected: true
    // Calls stub function - will fail until fn:implicit-timezone is implemented
    let _ = stub_fnimplicit_timezone();
}
