//! Test chunk 1 for fn:format-date
//! Contains 43 tests

use dep::xpath::stub_fnformat_date;

#[test]
fn format_date_032() {
    // test format-date(): optional digit symbols in picture
    // XPath: format-date(xs:date(\"1200-01-01\"),\"[Y###9]\")
    // Expected: 1200
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_033() {
    // test format-date(): optional digit symbols in picture
    // XPath: format-date(xs:date(\"0900-01-01\"),\"[Y###9]\")
    // Expected: 900
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_034() {
    // test format-date(): large width modifier
    // XPath: format-date(xs:date('2012-05-18'), '[Y9999,25]')
    // Expected: 0000000000000000000002012
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_035() {
    // test format-date(): large width modifier, Thai digits
    // XPath: format-date(xs:date('2012-05-18'), '[Y????,10]')
    // Expected: ??????????
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_036() {
    // test format-date(): day in year
    // XPath: format-date(xs:date('2016-02-01'), '[d]')
    // Expected: 32
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_038() {
    // test format-date(): year component
    // XPath: format-date(xs:date('654321-01-01'), '[Y#0 ' || codepoints-t
    // Expected: 54321
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_039() {
    // test format-date(): year component
    // XPath: format-date(xs:date('654321-01-01'), '[Y#0,2-5]')
    // Expected: 54321
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_041() {
    // test format-date(): year component
    // XPath: format-date(xs:date('2016-01-01'), '[Y#.0]')
    // Expected: 1.6
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en101() {
    // English month names
    // XPath: for $i in 1 to 12 return let $d2 := $d + xs:yearMonthDuratio
    // Expected: JANUARY FEBRUARY MARCH APRIL MAY JUNE JU
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en102() {
    // English month names
    // XPath: for $i in 1 to 12 return let $d2 := $d + xs:yearMonthDuratio
    // Expected: january february march april may june ju
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en103() {
    // English month names
    // XPath: for $i in 1 to 12          return let $d2 := $d + xs:yearMon
    // Expected: January February March April May June Ju
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en104() {
    // English month names
    // XPath: for $i in 1 to 12          return let $d2 := $d + xs:yearMon
    // Expected: JAN FEB MAR APR MAY JUN JUL AUG SEP OCT
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en105() {
    // English month names
    // XPath: for $i in 1 to 12          return let $d2 := $d + xs:yearMon
    // Expected: jan feb mar apr may jun jul aug sep oct
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en106() {
    // English month names
    // XPath: for $i in 1 to 12          return let $d2 := $d + xs:yearMon
    // Expected: Jan Feb Mar Apr May Jun Jul Aug Sep Oct
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en111() {
    // English weekday names
    // XPath: for $i in 1 to 7 return let $d2 := $d + xs:dayTimeDuration('
    // Expected: MONDAY TUESDAY WEDNESDAY THURSDAY FRIDAY
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en112() {
    // English weekday names
    // XPath: for $i in 1 to 7 return let $d2 := $d + xs:dayTimeDuration('
    // Expected: monday tuesday wednesday thursday friday
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en113() {
    // English weekday names
    // XPath: for $i in 1 to 7 return let $d2 := $d + xs:dayTimeDuration('
    // Expected: Monday Tuesday Wednesday Thursday Friday
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en114() {
    // English weekday names
    // XPath: for $i in 1 to 7 return let $d2 := $d + xs:dayTimeDuration('
    // Expected: MON TUE WED THU FRI SAT SUN
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en115() {
    // English weekday names
    // XPath: for $i in 1 to 7 return let $d2 := $d + xs:dayTimeDuration('
    // Expected: mon tue wed thu fri sat sun
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en116() {
    // English weekday names
    // XPath: for $i in 1 to 7 return let $d2 := $d + xs:dayTimeDuration('
    // Expected: Mon Tue Wed Thu Fri Sat Sun
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en117() {
    // English weekday names, abbreviated, variable length
    // XPath: for $i in 1 to 7         return let $d2 := $d + xs:dayTimeDu
    // Expected: Mon true Tue true Wed true Thu true Fri
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en118() {
    // English weekday names, abbreviated, variable length
    // XPath: for $i in 1 to 7 return let $d2 := $d + xs:dayTimeDuration('
    // Expected: Mon true Tue true Wed true Thu true Fri
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en121() {
    // English ordinal numbers 1-31
    // XPath: for $i in 0 to 30 return let $d2 := $d + xs:dayTimeDuration(
    // Expected: 1st 2nd 3rd 4th 5th 6th 7th 8th 9th 10th
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en122() {
    // English ordinal numbers 1990-2020
    // XPath: for $i in 0 to 30 return let $d2 := $d + xs:yearMonthDuratio
    // Expected: 1990th 1991st 1992nd 1993rd 1994th 1995t
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en122b() {
    // English ordinal numbers 1990-2020
    // XPath: for $i in 0 to 30 return let $d2 := $d + xs:yearMonthDuratio
    // Expected: 1990th 1991st 1992nd 1993rd 1994th 1995t
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en129() {
    // English ordinal words 1-31 upper case
    // XPath: string-join( for $i in 0 to 30 return let $d2 := $d + xs:day
    // Expected: FIRST; SECOND; THIRD; FOURTH; FIFTH; SIX
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en130() {
    // English ordinal words 1-31 lower case
    // XPath: string-join( for $i in 0 to 30 return let $d2 := $d + xs:day
    // Expected: first; second; third; fourth; fifth; six
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en131() {
    // English ordinal words 1-31 title case
    // XPath: string-join( for $i in 0 to 30 return let $d2 := $d + xs:day
    // Expected: First; Second; Third; Fourth; Fifth; Six
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en132() {
    // English ordinal words 1990-2020 upper case
    // XPath: string-join( for $i in 0 to 30 return let $d2 := $d + xs:yea
    // Expected: ONETHOUSANDNINEHUNDREDNINETIETH;
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en133() {
    // English ordinal words 1990-2020 lower case
    // XPath: string-join( for $i in 0 to 30 return let $d2 := $d + xs:yea
    // Expected: onethousandninehundredninetieth;
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_en134() {
    // English ordinal words 1990-2020 title case
    // XPath: string-join( for $i in 0 to 30 return let $d2 := $d + xs:yea
    // Expected: OneThousandNineHundredNinetieth;
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_de101() {
    // German month names
    // XPath: for $i in 1 to 12 return let $d2 := $d + xs:yearMonthDuratio
    // Expected: JANUAR FEBRUAR M?RZ APRIL MAI JUNI JULI
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_de102() {
    // German month names
    // XPath: for $i in 1 to 12 return let $d2 := $d + xs:yearMonthDuratio
    // Expected: januar februar m?rz april mai juni juli
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_de103() {
    // German month names
    // XPath: for $i in 1 to 12           return let $d2 := $d + xs:yearMo
    // Expected: Januar Februar M?rz April Mai Juni Juli
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_de104() {
    // German month names
    // XPath: for $i in 1 to 12           return let $d2 := $d + xs:yearMo
    // Expected: JAN FEB M?R APR MAI JUN JUL AUG SEP OKT
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_de105() {
    // German month names
    // XPath: for $i in 1 to 12           return let $d2 := $d + xs:yearMo
    // Expected: jan feb m?r apr mai jun jul aug sep okt
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_de106() {
    // German month names
    // XPath: for $i in 1 to 12           return let $d2 := $d + xs:yearMo
    // Expected: Jan Feb M?r Apr Mai Jun Jul Aug Sep Okt
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_de111() {
    // German weekday names
    // XPath: for $i in 1 to 7 return let $d2 := $d + xs:dayTimeDuration('
    // Expected: MONTAG DIENSTAG MITTWOCH DONNERSTAG FREI
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_de112() {
    // German weekday names
    // XPath: for $i in 1 to 7 return let $d2 := $d + xs:dayTimeDuration('
    // Expected: montag dienstag mittwoch donnerstag frei
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_de113() {
    // German weekday names
    // XPath: for $i in 1 to 7 return let $d2 := $d + xs:dayTimeDuration('
    // Expected: Montag Dienstag Mittwoch Donnerstag Frei
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_de114() {
    // German weekday names
    // XPath: for $i in 1 to 7 return let $d2 := $d + xs:dayTimeDuration('
    // Expected: MO DI MI DO FR SA SO
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_de115() {
    // German weekday names
    // XPath: for $i in 1 to 7 return let $d2 := $d + xs:dayTimeDuration('
    // Expected: mo di mi do fr sa so
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}

#[test]
fn format_date_de116() {
    // German weekday names
    // XPath: for $i in 1 to 7 return let $d2 := $d + xs:dayTimeDuration('
    // Expected: Mo Di Mi Do Fr Sa So
    // Calls stub function - will fail until fn:format-date is implemented
    let _ = stub_fnformat_date();
}
