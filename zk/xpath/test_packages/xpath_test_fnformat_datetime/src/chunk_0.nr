//! Test chunk 0 for fn:format-dateTime
//! Contains 50 tests

use dep::xpath::stub_fnformat_datetime;

#[test]
fn format_datetime_001a() {
    // test format-dateTime: basic numeric formats
    // XPath: format-dateTime($d,\"[Y]-[M01]-[D]\")
    // Expected: 2003-09-7
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_001b() {
    // test format-dateTime: basic numeric formats
    // XPath: format-dateTime($d,\"[M]-[D]-[Y]\")
    // Expected: 9-7-2003
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_001c() {
    // test format-dateTime: basic numeric formats
    // XPath: format-dateTime($d,\"[D]-[M]-[Y]\")
    // Expected: 7-9-2003
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_001d() {
    // test format-dateTime: basic numeric formats
    // XPath: format-dateTime($d,\"[D1] [MI] [Y]\")
    // Expected: 7 IX 2003
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_001e() {
    // test format-dateTime: basic numeric formats
    // XPath: format-dateTime($d,\"[[[Y]-[M01]-[D01]]]\")
    // Expected: [2003-09-07]
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_001f() {
    // test format-dateTime: basic numeric formats
    // XPath: format-dateTime($d,\"[[[Y0001]-[M01]-[D01]]]\")
    // Expected: [2003-09-07]
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_001g() {
    // test format-dateTime: basic numeric formats
    // XPath: format-dateTime($d,\"([Y01]-[M01]-[D01])\")
    // Expected: (03-09-07)
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_002a() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H01]:[m01]\")
    // Expected: 09:15
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_002b() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]\")
    // Expected: 9:15
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_002c() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H01]:[m01]:[s01]\")
    // Expected: 09:15:06
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_002d() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]:[s]\")
    // Expected: 9:15:06
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_002e() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]:[s1]\")
    // Expected: 9:15:6
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_002f() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]:[s01]:[f001]\")
    // Expected: 9:15:06:456
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_002g() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]:[s].[f,1-1]\")
    // Expected: 9:15:06.4
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_002h() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]:[s].[f1,1-1]\")
    // Expected: 9:15:06.4
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_002i() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]:[s].[f01]\")
    // Expected: 9:15:06.45
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_002j() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]:[s].[f001]\")
    // Expected: 9:15:06.456
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003a() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[Y]-[M01]-[D]\")
    // Expected: 2003-09-7
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003b() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[M]-[D]-[Y]\")
    // Expected: 9-7-2003
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003c() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[D]-[M]-[Y]\")
    // Expected: 7-9-2003
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003d() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[D1] [MI] [Y]\")
    // Expected: 7 IX 2003
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003e() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[[[Y]-[M01]-[D01]]]\")
    // Expected: [2003-09-07]
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003f() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[[[Y0001]-[M01]-[D01]]]\")
    // Expected: [2003-09-07]
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003g() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"([Y01]-[M01]-[D01])\")
    // Expected: (03-09-07)
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003h() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H01]:[m01]\")
    // Expected: 09:15
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003i() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]\")
    // Expected: 9:15
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003j() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H01]:[m01]:[s01]\")
    // Expected: 09:15:06
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003k() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]:[s]\")
    // Expected: 9:15:06
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003l() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]:[s1]\")
    // Expected: 9:15:6
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003m() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]:[s].[f,1-1]\")
    // Expected: 9:15:06.4
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003n() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]:[s].[f1,1-1]\")
    // Expected: 9:15:06.4
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003p() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]:[s].[f01]\")
    // Expected: 9:15:06.45
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003q() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,\"[H]:[m]:[s].[f001]\")
    // Expected: 9:15:06.456
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_003r() {
    // Test format-dateTime: basic numeric formats
    // XPath: format-dateTime($t,'[Y0001]-[M01]-[D01]T[H01]:[m01]:[s01].[f
    // Expected: 2003-09-07T09:15:06.456
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_004() {
    // Test format-dateTime: 12-hour clock
    // XPath: for $i in 1 to 24 return         format-dateTime($t + xs:day
    // Expected: 10.15 11.15 12.15 1.15 2.15 3.15 4.15 5.
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_005() {
    // Test format-dateTime: upper-case roman numerals for year
    // XPath: string-join(           for $i in 1 to 100 return           f
    // Expected: MCMLI; MCMLII; MCMLIII; MCMLIV; MCMLV; M
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_006() {
    // Test format-dateTime: lower-case roman numerals for year (with the width specifi
    // XPath: string-join(           for $i in 1 to 100 return           f
    // Expected: dcccxvii; dcccxxxiv; dcccli; dccclxviii;
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_006a() {
    // Test format-dateTime: lower-case roman numerals for year (with the width specifi
    // XPath: format-dateTime($t, '[Yi,4-4]')
    // Expected: miv
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_009() {
    // Test format-dateTime: test format-dateTime: ISO week numbers Specifically, in th
    // XPath: for $i in 1 to 48,                 $d in $t + xs:yearMonthDu
    // Expected: [2004-01-01T12:00:00: 1] [2004-02-01T12:
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_010() {
    // test format-dateTime: ISO day within week Specifically, in the ISO calendar the
    // XPath: for $i in 1 to 48,                 $d in $t + xs:yearMonthDu
    // Expected: [2004-01-01T12:00:00: 04 [2004-02-01T12:
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_011() {
    //         Test format-dateTime: ISO week number within month.         Specifically
    // XPath: for $i in 1 to 48,                 $d in $t + xs:yearMonthDu
    // Expected: [2006-01-01T12:00:00: 5 [2006-02-01T12:0
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_012() {
    // test format-dateTime(): a,b,c... numbering sequence (not actually useful but all
    // XPath: for $i in 1 to 60 return             format-dateTime($t + xs
    // Expected: P.g Q.h R.i S.j T.k U.l V.m W.n X.o Y.p
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013a() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[Y,4-4]')
    // Expected: 0985
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013b() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[Y,3-4]')
    // Expected: 985
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013c() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[Y,2-5]')
    // Expected: 985
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013d() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[Y,2-2]')
    // Expected: 85
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013e() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[Y,2-*]')
    // Expected: 985
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013f() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[Y,*-4]')
    // Expected: 985
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013g() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[Y,3]')
    // Expected: 985
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}

#[test]
fn format_datetime_013h() {
    // test format-dateTime(): effect of width specifiers
    // XPath: format-dateTime($t, '[M,4-4]')
    // Expected: 0003
    // Calls stub function - will fail until fn:format-dateTime is implemented
    let _ = stub_fnformat_datetime();
}
