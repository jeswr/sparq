//! Test chunk 0 for fn:format-time
//! Contains 50 tests

use dep::xpath::stub_fnformat_time;

#[test]
fn format_time_002a() {
    // Test format-time: basic numeric formats
    // XPath: format-time($t,\"[H01]:[m01]\")
    // Expected: 09:15
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_002b() {
    // Test format-time: basic numeric formats
    // XPath: format-time($t,\"[H]:[m]\")
    // Expected: 9:15
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_002c() {
    // Test format-time: basic numeric formats
    // XPath: format-time($t,\"[H01]:[m01]:[s01]\")
    // Expected: 09:15:06
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_002d() {
    // Test format-time: basic numeric formats
    // XPath: format-time($t,\"[H]:[m]:[s]\")
    // Expected: 9:15:06
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_002e() {
    // Test format-time: basic numeric formats
    // XPath: format-time($t,\"[H]:[m]:[s1]\")
    // Expected: 9:15:6
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_002f() {
    // Test format-time: basic numeric formats
    // XPath: format-time($t,\"[H]:[m]:[s01]:[f001]\")
    // Expected: 9:15:06:456
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_002g() {
    // Test format-time: basic numeric formats
    // XPath: format-time($t,\"[H]:[m]:[s].[f,1-1]\")
    // Expected: 9:15:06.4
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_002h() {
    // Test format-time: basic numeric formats
    // XPath: format-time($t,\"[H]:[m]:[s].[f1,1-1]\")
    // Expected: 9:15:06.4
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_002i() {
    // Test format-time: basic numeric formats
    // XPath: format-time($t,\"[H]:[m]:[s].[f01]\")
    // Expected: 9:15:06.45
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_002j() {
    // Test format-time: basic numeric formats
    // XPath: format-time($t,\"[H]:[m]:[s].[f001]\")
    // Expected: 9:15:06.456
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_004() {
    // Test format-time: 12-hour clock
    // XPath: for $i in 1 to 24 return         format-time($t + xs:dayTime
    // Expected: 10.15 11.15 12.15 1.15 2.15 3.15 4.15 5.
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_013n() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[m,3]')
    // Expected: 015
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_013p() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,4-4]')
    // Expected: 4560
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_013q() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,1-4]')
    // Expected: 456
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_013r() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,2-5]')
    // Expected: 456
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_013s() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,2-2]')
    // Expected: 45
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_013t() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,1-*]')
    // Expected: 456
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_013u() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,*-2]')
    // Expected: 45
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_013v() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,3]')
    // Expected: 456
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_014() {
    // test format-time(): timezones in +nn:nn notation
    // XPath: string-join(                for $z in -28 to +28
    // Expected: 0715-14:00; 0745-13:30; 0815-13:00; 0845
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_015() {
    // test format-time(): timezones in GMT+x notation
    // XPath: string-join(                for $z in -28 to +28
    // Expected: 0715GMT-14:00; 0745GMT-13:30; 0815GMT-13
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_016() {
    // test format-time(): timezones in GMT+x notation
    // XPath: string-join(                for $z in -28 to +28
    // Expected: 0715GMT-14:00; 0745GMT-13:30; 0815GMT-13
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_017() {
    // test format-time(): timezones in GMT+x notation
    // XPath: string-join(                for $z in -28 to +28
    // Expected: 0715GMT-14:00; 0745GMT-13:30; 0815GMT-13
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_018() {
    // test format-time(): timezones in GMT+x notation
    // XPath: string-join(                for $z in -28 to +28
    // Expected: 0715GMT-14:00; 0745GMT-13:30; 0815GMT-13
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_019() {
    // Test format-time: fractional seconds in Thai digits
    // XPath: format-time(xs:time('09:15:06.456'),\"[f???]\")
    // Expected: ???
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_020() {
    // Test format-time: non-ASCII digits
    // XPath: format-time(xs:time('09:15:06.456'),\"[H?]:[m?]:[s??]:[f???]
    // Expected: ?:??:??:???
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_023p() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,4-4]')
    // Expected: 0060
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_023q() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,1-4]')
    // Expected: 006
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_023r() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,2-5]')
    // Expected: 006
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_023s() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,2-2]')
    // Expected: 00
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_023t() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,1-*]')
    // Expected: 006
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_023u() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,*-2]')
    // Expected: 0
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_023v() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,3]')
    // Expected: 006
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_024p() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,4-4]')
    // Expected: 1000
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_024q() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,1-4]')
    // Expected: 1
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_024r() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,2-5]')
    // Expected: 10
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_024s() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,2-2]')
    // Expected: 10
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_024t() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,1-*]')
    // Expected: 1
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_024u() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,*-2]')
    // Expected: 1
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn format_time_024v() {
    // test format-time(): effect of width specifiers
    // XPath: format-time($t, '[f,3]')
    // Expected: 100
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_001() {
    // Fractional seconds: default
    // XPath: format-time(xs:time('12:01:01.123'), '[f]')
    // Expected: "123"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_002() {
    // Fractional seconds: explicit default
    // XPath: format-time(xs:time('12:01:01.123'), '[f1]')
    // Expected: "123"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_003() {
    // Fractional seconds: two digits
    // XPath: format-time(xs:time('12:01:01.123'), '[f99]')
    // Expected: "12"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_004() {
    // Fractional seconds: three digits
    // XPath: format-time(xs:time('12:01:01.123'), '[f001]')
    // Expected: "123"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_005() {
    // Fractional seconds: optional digits
    // XPath: format-time(xs:time('12:01:01.123'), '[f99#]')
    // Expected: "123"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_006() {
    // Fractional seconds: optional digits
    // XPath: format-time(xs:time('12:01:01.12'), '[f99#]')
    // Expected: "12"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_007() {
    // Fractional seconds: add trailing zeros
    // XPath: format-time(xs:time('12:01:01.12'), '[f777]')
    // Expected: "120"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_008() {
    // Fractional seconds: truncation
    // XPath: format-time(xs:time('12:01:01.127'), '[f01]')
    // Expected: "12"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_009() {
    // Fractional seconds: truncation
    // XPath: format-time(xs:time('12:01:01.125'), '[f01]')
    // Expected: "12"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}

#[test]
fn millisecs_010() {
    // Fractional seconds: truncation
    // XPath: format-time(xs:time('12:01:01.135'), '[f01]')
    // Expected: "13"
    // Calls stub function - will fail until fn:format-time is implemented
    let _ = stub_fnformat_time();
}
