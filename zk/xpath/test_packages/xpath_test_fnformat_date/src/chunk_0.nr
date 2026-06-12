//! Test chunk 0 for fn:format-date
//! Contains 50 tests

use dep::xpath::stub_fnformat_date;

#[test]
fn format_date_001a() {
    // test format-date: basic numeric formats
    // XPath: format-date($d,\"[Y]-[M01]-[D]\")
    // Expected: 2003-09-7
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_001b() {
    // test format-date: basic numeric formats
    // XPath: format-date($d,\"[M]-[D]-[Y]\")
    // Expected: 9-7-2003
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_001c() {
    // test format-date: basic numeric formats
    // XPath: format-date($d,\"[D]-[M]-[Y]\")
    // Expected: 7-9-2003
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_001d() {
    // test format-date: basic numeric formats
    // XPath: format-date($d,\"[D1] [MI] [Y]\")
    // Expected: 7 IX 2003
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_001e() {
    // test format-date: basic numeric formats
    // XPath: format-date($d,\"[[[Y]-[M01]-[D01]]]\")
    // Expected: [2003-09-07]
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_001f() {
    // test format-date: basic numeric formats
    // XPath: format-date($d,\"[[[Y0001]-[M01]-[D01]]]\")
    // Expected: [2003-09-07]
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_001g() {
    // test format-date: basic numeric formats
    // XPath: format-date($d,\"([Y01]-[M01]-[D01])\")
    // Expected: (03-09-07)
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_005() {
    // Test format-date: upper-case roman numerals for year
    // XPath: string-join( for $i in 1 to 100 return format-date($t + xs:y
    // Expected: MCMLI MCMLII MCMLIII MCMLIV MCMLV MCMLVI
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_006() {
    // Test format-date: lower-case roman numerals for year (width specifier ignored)
    // XPath: string-join( for $i in 1 to 100 return format-date($t +
    // Expected: dcccxvii dcccxxxiv dcccli dccclxviii dcc
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_007a() {
    // Test format-date: numeric formats using Thai digits
    // XPath: format-date($d,\"[Y????]-[M??]-[D?]\")
    // Expected: ????-??-?
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_007b() {
    // Test format-date: numeric formats using Thai digits
    // XPath: format-date($d,\"[M?]-[D?]-[Y????]\")
    // Expected: ?-?-????
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_007c() {
    // Test format-date: numeric formats using Thai digits
    // XPath: format-date($d,\"([Y??]-[M??]-[D??])\")
    // Expected: (??-??-??)
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_008a() {
    // Test format-date: numeric formats using Osmanya (non-BMP) digits
    // XPath: format-date($d,\"[Y????]-[M??]-[D?]\")
    // Expected: ????-??-?
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_008b() {
    // Test format-date: numeric formats using Osmanya (non-BMP) digits
    // XPath: format-date($d,\"[M?]-[D?]-[Y????]\")
    // Expected: ?-?-????
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_008c() {
    // Test format-date: numeric formats using Osmanya (non-BMP) digits
    // XPath: format-date($d,\"([Y??]-[M??]-[D??])\")
    // Expected: (??-??-??)
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_009() {
    // Test format-date: test format-date: ISO week numbers Specifically, in the ISO ca
    // XPath: for $i in 1 to 48, $d in $t + xs:yearMonthDuration('P1M')*$i
    // Expected: [2004-05-01:18] [2004-06-01:23] [2004-07
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_010() {
    // test format-date: ISO day within week Specifically, in the ISO calendar the days
    // XPath: for $i in 1 to 48, $d in $t + xs:yearMonthDuration('P1M')*$i
    // Expected: [2004-01-01:04] [2004-02-01:07] [2004-03
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_011() {
    // test format-date: ISO week number within month --> Specifically, in the ISO cale
    // XPath: for $i in 1 to 48, $d in $t + xs:dayTimeDuration('P3D')*$i
    // Expected: [2005-12-04:1] [2005-12-07:2] [2005-12-1
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_013a() {
    // test format-date(): effect of width specifiers
    // XPath: format-date($t, '[Y,4-4]')
    // Expected: 0985
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_013b() {
    // test format-date(): effect of width specifiers
    // XPath: format-date($t, '[Y,3-4]')
    // Expected: 985
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_013c() {
    // test format-date(): effect of width specifiers
    // XPath: format-date($t, '[Y,2-5]')
    // Expected: 985
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_013d() {
    // test format-date(): effect of width specifiers
    // XPath: format-date($t, '[Y,2-2]')
    // Expected: 85
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_013e() {
    // test format-date(): effect of width specifiers
    // XPath: format-date($t, '[Y,2-*]')
    // Expected: 985
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_013f() {
    // test format-date(): effect of width specifiers
    // XPath: format-date($t, '[Y,*-4]')
    // Expected: 985
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_013g() {
    // test format-date(): effect of width specifiers
    // XPath: format-date($t, '[Y,3]')
    // Expected: 985
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_013h() {
    // test format-date(): effect of width specifiers
    // XPath: format-date($t, '[M,4-4]')
    // Expected: 0003
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_013i() {
    // test format-date(): effect of width specifiers
    // XPath: format-date($t, '[M,1-4]')
    // Expected: 3
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_013j() {
    // test format-date(): effect of width specifiers
    // XPath: format-date($t, '[M,2-5]')
    // Expected: 03
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_013k() {
    // test format-date(): effect of width specifiers
    // XPath: format-date($t, '[M,2-2]')
    // Expected: 03
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_013l() {
    // test format-date(): effect of width specifiers
    // XPath: format-date($t, '[M,1-*]')
    // Expected: 3
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_013m() {
    // test format-date(): effect of width specifiers
    // XPath: format-date($t, '[M,*-2]')
    // Expected: 3
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_013n() {
    // test format-date(): effect of width specifiers
    // XPath: format-date($t, '[M,3]')
    // Expected: 003
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_014() {
    // test format-date(): timezones in +nn:nn notation
    // XPath: string-join( for $z in -28 to +28 return format-date(adjust-
    // Expected: -14:00 -13:30 -13:00 -12:30 -12:00 -11:3
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_015() {
    // test format-date(): timezones in GMT+x notation
    // XPath: string-join( for $z in -28 to +28 return format-date(adjust-
    // Expected: GMT-14; GMT-13:30; GMT-13; GMT-12:30; GM
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_016() {
    // test format-date(): timezones in GMT+x notation
    // XPath: string-join( for $z in -28 to +28 return format-date(adjust-
    // Expected: GMT-14:00; GMT-13:30; GMT-13:00; GMT-12:
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_017() {
    // test format-date(): timezones in military notation
    // XPath: string-join( for $z in -12 to +12 return format-date(adjust-
    // Expected: Y X W V U T S R Q P O N Z A B C D E F G
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_018() {
    // test format-date(): timezone-less date in military notation
    // XPath: format-date(xs:date('1987-12-13'), '[ZZ]')
    // Expected: J
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_019() {
    // test format-date(): non-integral timezone in military notation
    // XPath: format-date(xs:date('1987-12-13+05:30'), '[ZZ]')
    // Expected: +05:30
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_020() {
    // test format-date(): extreme timezone in military notation
    // XPath: format-date(xs:date('1987-12-13+13:00'), '[ZZ]')
    // Expected: +13:00
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_021() {
    // test format-date(): timezones with an alternate separator
    // XPath: string-join( for $z in -28 to +28 return format-date(adjust-
    // Expected: GMT-14~00; GMT-13~30; GMT-13~00; GMT-12~
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_022() {
    // test format-date(): timezones with non-ASCII digits
    // XPath: format-date(xs:date('2012-05-18+05:30'), '[Z??:??]')
    // Expected: +??:??
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_023() {
    // test format-date(): timezones with three-digit format
    // XPath: string-join( for $z in -28 to +28 return format-date(adjust-
    // Expected: -14:00; -13:30; -13:00; -12:30; -12:00;
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_024() {
    // test format-date(): timezones with three-digit format, no separator
    // XPath: string-join( for $z in -28 to +28 return format-date(adjust-
    // Expected: -1400; -1330; -1300; -1230; -1200; -1130
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_025() {
    // test format-date(): timezones with two-digit format, no separator
    // XPath: string-join( for $z in -28 to +28 return format-date(adjust-
    // Expected: -14; -13:30; -13; -12:30; -12; -11:30; -
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_026() {
    // test format-date(): timezones with one-digit format, no separator, t modifier
    // XPath: string-join( for $z in -28 to +28 return format-date(adjust-
    // Expected: -14; -13:30; -13; -12:30; -12; -11:30; -
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_027() {
    // test format-date(): timezones with astral plane digits
    // XPath: format-date(xs:date('2012-05-18+05:30'), '[Z?:??]')
    // Expected: +?:??
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_028() {
    // test format-date(): whitespace within a variable marker is ignored
    // XPath: format-date(xs:date('2012-05-18'), '[ D 01 ] [M 0 1] [ Y 0 0
    // Expected: 18 05 2012
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_029() {
    // test format-date(): semicolon as grouping separator
    // XPath: format-date(xs:date('2012-05-18'), '[Y9;999]')
    // Expected: 2;012
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_030() {
    // test format-date(): comma as grouping separator
    // XPath: format-date(xs:date('2012-05-18'), '[Y9,999,*]')
    // Expected: 2,012
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_031() {
    // test format-date(): grouping separators
    // XPath: format-date(xs:date('2012-05-18'), '[Y9,99-9,*]')
    // Expected: 2,01-2
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}
