//! Test chunk 1 for fn:format-dateTime
//! Contains 28 tests

use dep::xpath::stub_fnformat_datetime;

#[test]
fn format_datetime_013i() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[M,1-4]')
    // Expected: 3
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013j() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[M,2-5]')
    // Expected: 03
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013k() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[M,2-2]')
    // Expected: 03
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013l() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[M,1-*]')
    // Expected: 3
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013m() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[M,*-2]')
    // Expected: 3
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013n() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[M,3]')
    // Expected: 003
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013p() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[f,4-4]')
    // Expected: 4560
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013q() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[f,1-4]')
    // Expected: 456
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013r() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[f,2-5]')
    // Expected: 456
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013s() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[f,2-2]')
    // Expected: 45
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013t() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[f,1-*]')
    // Expected: 456
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013u() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[f,*-2]')
    // Expected: 45
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013v() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[f,3]')
    // Expected: 456
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_014() {
    // test format-dateTime(): timezones in +nn:nn notation
    // XPath: string-join(                for $z in -28 to +28
    // Expected: 0715-14:00; 0745-13:30; 0815-13:00; 0845
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_015() {
    // test format-dateTime(): timezones in GMT+x notation
    // XPath: string-join(                for $z in -28 to +28
    // Expected: 0715GMT-14; 0745GMT-13:30; 0815GMT-13; 0
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_016() {
    // test format-dateTime(): timezones in GMT+x notation
    // XPath: string-join(                for $z in -28 to +28
    // Expected: 0715GMT-14:00; 0745GMT-13:30; 0815GMT-13
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_017() {
    // test format-dateTime(): timezones in GMT+x notation
    // XPath: string-join(                for $z in -28 to +28
    // Expected: 0715GMT-14; 0745GMT-13:30; 0815GMT-13; 0
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_018() {
    // test format-dateTime(): timezones in GMT+x notation
    // XPath: string-join(                for $z in -28 to +28
    // Expected: 0715GMT-14; 0745GMT-13:30; 0815GMT-13; 0
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_019a() {
    //           Test format-dateTime(): timezone is adjusted based on Olson timezone n
    // XPath: format-dateTime(xs:dateTime('2015-02-15T12:00:00Z'), '[D] [M
    // Expected: '15 February 2015 @ 07:00 -05:00'
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_019b() {
    //           Test format-dateTime(): timezone is adjusted based on Olson timezone n
    // XPath: format-dateTime(xs:dateTime('2015-08-15T12:00:00Z'), '[D] [M
    // Expected: '15 August 2015 @ 08:00 -04:00'
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_019c() {
    //           Test format-dateTime(): timezone is adjusted based on Olson timezone n
    // XPath: format-dateTime(xs:dateTime('2015-02-15T12:00:00Z'), '[D] [M
    // Expected: '15 February 2015 @ 07:00 EST'
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_019d() {
    //           Test format-dateTime(): timezone is adjusted based on Olson timezone n
    // XPath: format-dateTime(xs:dateTime('2015-08-15T12:00:00Z'), '[D] [M
    // Expected: '15 August 2015 @ 08:00 EDT'
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_019e() {
    //           Test format-dateTime(): timezone is adjusted based on Olson timezone n
    // XPath: format-dateTime(xs:dateTime('2015-02-15T12:00:00Z'), '[D] [M
    // Expected: '15 February 2015 @ 13:00 +01:00'
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_019f() {
    //           Test format-dateTime(): timezone is adjusted based on Olson timezone n
    // XPath: format-dateTime(xs:dateTime('2015-08-15T12:00:00Z'), '[D] [M
    // Expected: '15 August 2015 @ 14:00 +02:00'
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_019g() {
    //           Test format-dateTime(): timezone is adjusted based on Olson timezone n
    // XPath: format-dateTime(xs:dateTime('2015-02-15T12:00:00Z'), '[D] [M
    // Expected: '15 February 2015 @ 13:00 CET'
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_019h() {
    //           Test format-dateTime(): timezone is adjusted based on Olson timezone n
    // XPath: format-dateTime(xs:dateTime('2015-08-15T12:00:00Z'), '[D] [M
    // Expected: '15 August 2015 @ 14:00 CEST'
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_en142() {
    // English AM/PM
    // XPath: for $i in 0 to 23 return         let $t := $b + xs:dayTimeDu
    // Expected: 12~10am 1~10am 2~10am 3~10am 4~10am 5~10
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_en143() {
    // English noon/midnight. The US convention seems to be noon=AM, midnight=PM,
    // XPath: for $i in 0 to 1 return         let $t := $b + xs:dayTimeDur
    // Expected: 12~00am 12~00pm
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}
