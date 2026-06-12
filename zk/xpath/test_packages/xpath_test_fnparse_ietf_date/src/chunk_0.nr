//! Test chunk 0 for fn:parse-ietf-date
//! Contains 40 tests

use dep::xpath::stub_fnparse_ietf_date;

#[test]
fn parse_ietf_date_0() {
    // test parse-ietf-date is present
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 19:36:01 GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_1() {
    // leading-spaces
    // XPath: if (parse-ietf-date(\"  Wed, 20 Aug 2014 19:36:01 GMT\") = $
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_2() {
    // trailing spaces
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 19:36:01 GMT    \") =
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_3() {
    // no comma
    // XPath: if (parse-ietf-date(\"Wed 20 Aug 2014 19:36:01 GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_4() {
    // full day name
    // XPath: if (parse-ietf-date(\"Wednesday 20 Aug 2014 19:36:01 GMT\")
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_5() {
    // fractional seconds
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 19:36:01.25 GMT\") =
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_6() {
    // long fractional seconds
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 19:36:01.299 GMT\") =
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_7() {
    // all lower
    // XPath: if (parse-ietf-date(\"wed, 20 aug 2014 19:36:01 gmt\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_8() {
    // all upper
    // XPath: if (parse-ietf-date(\"WED, 20 AUG 2014 19:36:01 GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_9() {
    // no seconds
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 19:36 GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_10() {
    // no timezone
    // XPath: if (parse-ietf-date(\"WED, 20 AUG 2014 19:36:01\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_11() {
    // no seconds, no timezone
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 19:36\") = $d)    the
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_12() {
    // no space before timezone name
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 19:36:01GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_13() {
    // no seconds and space before timezone name
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 19:36GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_14() {
    // no day name (leaing space is required, I think a bug)
    // XPath: if (parse-ietf-date(\" 20 Aug 2014 19:36:01 GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_15() {
    // single digit day
    // XPath: if (parse-ietf-date(\"Tue, 9 Sep 2014 19:36:01 GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_16() {
    // single digit day with space
    // XPath: if (parse-ietf-date(\"Tue,  9 Sep 2014 19:36:01 GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_17() {
    // leading zero on day
    // XPath: if (parse-ietf-date(\"Tue, 09 Sep 2014 19:36:01 GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_18() {
    // no comma, single digit day
    // XPath: if (parse-ietf-date(\"Mon 8 Sep 2014 19:36:01 GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_19() {
    // dsep hyphen, no day name
    // XPath: if (parse-ietf-date(\"Aug-20 19:36 2014\") = $d)    then \"p
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_20() {
    // dsep S only, no day name
    // XPath: if (parse-ietf-date(\"Aug 20 19:36 2014\") = $d)    then \"p
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_21() {
    // dsep hyphen with spaces
    // XPath: if (parse-ietf-date(\"Wed, 20 - Aug - 2014 19:36\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_22() {
    // dsep hyphen with space
    // XPath: if (parse-ietf-date(\"Wed, 20- Aug- 2014 19:36\") = $d)    t
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_23() {
    // dsep hyphen with space
    // XPath: if (parse-ietf-date(\"Wed, 20 -Aug -2014 19:36\") = $d)    t
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_24() {
    // dayname is ignored even if wrong
    // XPath: if (parse-ietf-date(\"Sun, 20 Aug 2014 19:36:01 GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_25() {
    // dayname is ignored even if wrong
    // XPath: if (parse-ietf-date(\"Mon, 20 Aug 2014 19:36:01 GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_26() {
    // dayname is ignored even if wrong
    // XPath: if (parse-ietf-date(\"Tuesday, 20 Aug 2014 19:36:01 GMT\") =
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_27() {
    // dayname is ignored even if wrong
    // XPath: if (parse-ietf-date(\"Thursday, 20 Aug 2014 19:36:01 GMT\")
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_28() {
    // dayname is ignored even if wrong
    // XPath: if (parse-ietf-date(\"FRI, 20 Aug 2014 19:36:01 GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_29() {
    // dayname is ignored even if wrong
    // XPath: if (parse-ietf-date(\"SAT, 20 Aug 2014 19:36:01 GMT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_30() {
    // timezone name (UT)
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 19:36:01 UT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_31() {
    // timezone name (UTC)
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 19:36:01 UTC\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_32() {
    // timezone name (EST)
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 14:36:01 EST\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_33() {
    // timezone name (EDT)
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 15:36:01 EDT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_34() {
    // timezone name (CST)
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 13:36:01 CST\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_35() {
    // timezone name (CDT)
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 14:36:01 CDT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_36() {
    // timezone name (MST)
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 12:36:01 MST\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_37() {
    // timezone name (MDT)
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 13:36:01 MDT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_38() {
    // timezone name (PST)
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 11:36:01 PST\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}

#[test]
fn parse_ietf_date_39() {
    // timezone name (PDT)
    // XPath: if (parse-ietf-date(\"Wed, 20 Aug 2014 12:36:01 PDT\") = $d)
    // Expected: pass
    // Calls stub function - will fail until fn:parse-ietf-date is implemented
    let _ = stub_fnparse_ietf_date();
}
