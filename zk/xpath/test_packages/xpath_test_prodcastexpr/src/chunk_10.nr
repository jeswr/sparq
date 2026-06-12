//! Test chunk 10 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_232() {
    //  Ensure casting xs:gYear to xs:string, with timezone '+00:00' is properly handle
    // XPath: xs:string(xs:gYear(\"1999+00:00\")) eq \"1999Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_233() {
    //  Ensure casting xs:gYear to xs:string, with timezone 'Z' is properly handled.
    // XPath: xs:string(xs:gYear(\"1999Z\")) eq \"1999Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_253() {
    //  Ensure casting xs:gDay to xs:string, with timezone '-00:00' is properly handled
    // XPath: xs:string(xs:gDay(\"---01-00:00\")) eq \"---01Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_254() {
    //  Ensure casting xs:gDay to xs:string, with timezone '+00:00' is properly handled
    // XPath: xs:string(xs:gDay(\"---01+00:00\")) eq \"---01Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_255() {
    //  Ensure casting xs:gDay to xs:string, with timezone 'Z' is properly handled.
    // XPath: xs:string(xs:gDay(\"---01Z\")) eq \"---01Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_268() {
    //  Ensure casting xs:gMonth to xs:string, with timezone '-00:00' is properly handl
    // XPath: xs:string(xs:gMonth(\"--01-00:00\")) eq \"--01Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_269() {
    //  Ensure casting xs:gMonth to xs:string, with timezone '+00:00' is properly handl
    // XPath: xs:string(xs:gMonth(\"--01+00:00\")) eq \"--01Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_270() {
    //  Ensure casting xs:gMonth to xs:string, with timezone 'Z' is properly handled.
    // XPath: xs:string(xs:gMonth(\"--01Z\")) eq \"--01Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_283() {
    //  Ensure casting xs:gYearMonth to xs:string, with timezone '-00:00' is properly h
    // XPath: xs:string(xs:gYearMonth(\"1999-01-00:00\")) eq \"1999-01Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_284() {
    //  Ensure casting xs:gYearMonth to xs:string, with timezone '+00:00' is properly h
    // XPath: xs:string(xs:gYearMonth(\"1999-01+00:00\")) eq \"1999-01Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_285() {
    //  Ensure casting xs:gYearMonth to xs:string, with timezone 'Z' is properly handle
    // XPath: xs:string(xs:gYearMonth(\"1999-01Z\")) eq \"1999-01Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_304() {
    //  Ensure casting xs:gMonthDay to xs:string, with timezone '-00:00' is properly ha
    // XPath: xs:string(xs:gMonthDay(\"--01-01-00:00\")) eq \"--01-01Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_305() {
    //  Ensure casting xs:gMonthDay to xs:string, with timezone '+00:00' is properly ha
    // XPath: xs:string(xs:gMonthDay(\"--01-01+00:00\")) eq \"--01-01Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_306() {
    //  Ensure casting xs:gMonthDay to xs:string, with timezone 'Z' is properly handled
    // XPath: xs:string(xs:gMonthDay(\"--01-01Z\")) eq \"--01-01Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_307() {
    //  Simple xs:gMonthDay test involving --02-29.
    // XPath: xs:gMonthDay(\"--02-29+00:00\") eq xs:gMonthDay(\"--02-29+00
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_326() {
    //  Simple test of casting a xs:dateTime with UTC timezone to xs:gYear.
    // XPath: xs:gYear(xs:dateTime(\"2002-11-23T23:12:23.867-00:00\")) eq
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_327() {
    //  Simple test of casting a xs:dateTime with UTC offset to xs:gYear.
    // XPath: xs:gYear(xs:dateTime(\"2002-11-23T23:12:23.867-13:37\")) eq
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_328() {
    //  Simple test of casting a xs:dateTime with UTC timezone to xs:gYearMonth.
    // XPath: xs:gYearMonth(xs:dateTime(\"2002-11-23T23:12:23.867-00:00\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_329() {
    //  Simple test of casting a xs:dateTime with UTC offset to xs:gYearMonth.
    // XPath: xs:gYearMonth(xs:dateTime(\"2002-11-23T23:12:23.867-13:37\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_330() {
    //  Simple test of casting a xs:dateTime with UTC timezone to xs:gMonth.
    // XPath: xs:gMonth(xs:dateTime(\"2002-11-23T22:12:23.867-00:00\")) eq
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_331() {
    //  Simple test of casting a xs:dateTime with UTC offset to xs:gMonth.
    // XPath: xs:gMonth(xs:dateTime(\"2002-11-23T22:12:23.867-13:37\")) eq
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_332() {
    //  Simple test of casting a xs:dateTime with UTC timezone to xs:gMonthDay.
    // XPath: xs:gMonthDay(xs:dateTime(\"2002-11-23T22:12:23.867-00:00\"))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_333() {
    //  Simple test of casting a xs:dateTime with UTC offset to xs:gMonthDay.
    // XPath: xs:gMonthDay(xs:dateTime(\"2002-11-23T22:12:23.867-13:37\"))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_334() {
    //  Simple test of casting a xs:dateTime with UTC timezone to xs:gDay.
    // XPath: xs:gDay(xs:dateTime(\"2002-11-23T22:12:23.867-00:00\")) eq x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_335() {
    //  Simple test of casting a xs:dateTime with UTC offset to xs:gDay.
    // XPath: xs:gDay(xs:dateTime(\"2002-11-23T22:12:23.867-13:37\")) eq x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_336() {
    //  Simple test of casting a xs:dateTime with UTC timezone to xs:date.
    // XPath: xs:date(xs:dateTime(\"2002-11-23T22:12:23.867-00:00\")) eq x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_337() {
    //  Simple test of casting a xs:dateTime with UTC offset to xs:date.
    // XPath: xs:date(xs:dateTime(\"2002-11-23T22:12:23.867-13:37\")) eq x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_338() {
    //  Simple test of casting a xs:dateTime with UTC timezone to xs:time.
    // XPath: xs:time(xs:dateTime(\"2002-11-23T22:12:23.867-00:00\")) eq x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_339() {
    //  Simple test of casting a xs:dateTime with UTC offset to xs:time.
    // XPath: xs:time(xs:dateTime(\"2002-11-23T22:12:23.867-13:37\")) eq x
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_340() {
    //  Ensure casting xs:dateTime to xs:string, with timezone '-00:00' is properly han
    // XPath: xs:string(xs:dateTime(\"1999-12-01T23:59:12.999-00:00\")) eq
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_341() {
    //  Ensure casting xs:dateTime to xs:string, with timezone '+00:00' is properly han
    // XPath: xs:string(xs:dateTime(\"1999-12-01T23:59:12.432+00:00\")) eq
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_342() {
    //  Ensure when casting xs:dateTime to xs:string that an empty milli seconds part i
    // XPath: xs:string(xs:dateTime(\"2002-02-15T21:01:23.000\")) eq \"200
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_343() {
    //  Ensure when casting xs:dateTime to xs:string that milli seconds have no trailin
    // XPath: xs:string(xs:dateTime(\"2002-02-15T21:01:23.100\")) eq \"200
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_344() {
    //  Ensure when casting xs:dateTime to xs:string that milli seconds are properly pa
    // XPath: xs:string(xs:dateTime(\"2002-02-15T21:01:23.001\")) eq \"200
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_345() {
    //  Ensure when casting xs:dateTime to xs:string that milli seconds are properly pa
    // XPath: xs:string(xs:dateTime(\"2002-02-15T21:01:23.010\")) eq \"200
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_346() {
    //  Ensure when casting xs:dateTime to xs:string that a milli seconds part of '435'
    // XPath: xs:string(xs:dateTime(\"2002-02-15T21:01:23.435\")) eq \"200
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_347() {
    //  Ensure when casting xs:dateTime to xs:string that a milli seconds part of '11'
    // XPath: xs:string(xs:dateTime(\"2002-02-15T21:01:23.11\")) eq \"2002
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_348() {
    //  Ensure when casting xs:dateTime to xs:string that a milli seconds part of '110'
    // XPath: xs:string(xs:dateTime(\"2002-02-15T21:01:23.110\")) eq \"200
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_349() {
    //  Simple xs:dateTime involving time with no milli seconds.
    // XPath: string(xs:dateTime(\"2000-08-01T12:44:05\")) eq \"2000-08-01
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_350() {
    //  The canonical form of an xs:dateTime containing 24:00:00 is the following day a
    // XPath: string(xs:dateTime(\"2004-03-31T24:00:00\")) eq \"2004-04-01
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_364() {
    //  Simple xs:time involving time with no milli seconds.
    // XPath: string(xs:time(\"12:44:05\")) eq \"12:44:05\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_365() {
    //  Ensure casting xs:time to xs:string, with timezone '-00:00' is properly handled
    // XPath: xs:string(xs:time(\"23:59:12.999-00:00\")) eq \"23:59:12.999
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_366() {
    //  Ensure casting xs:time to xs:string, with timezone '+00:00' is properly handled
    // XPath: xs:string(xs:time(\"23:59:12.432+00:00\")) eq \"23:59:12.432
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_367() {
    //  Ensure when casting xs:time to xs:string that an empty milli seconds part is no
    // XPath: xs:string(xs:time(\"21:01:23.000\")) eq \"21:01:23\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_368() {
    //  Ensure when casting xs:time to xs:string that milli seconds have no trailing ze
    // XPath: xs:string(xs:time(\"21:01:23.100\")) eq \"21:01:23.1\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_369() {
    //  Ensure when casting xs:time to xs:string that milli seconds are properly padded
    // XPath: xs:string(xs:time(\"21:01:23.001\")) eq \"21:01:23.001\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_370() {
    //  Ensure when casting xs:time to xs:string that milli seconds are properly padded
    // XPath: xs:string(xs:time(\"21:01:23.010\")) eq \"21:01:23.01\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_371() {
    //  Ensure when casting xs:time to xs:string that a milli seconds part of '435' is
    // XPath: xs:string(xs:time(\"21:01:23.435\")) eq \"21:01:23.435\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_372() {
    //  Ensure when casting xs:time to xs:string that a milli seconds part of '11' is h
    // XPath: xs:string(xs:time(\"21:01:23.11\")) eq \"21:01:23.11\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_373() {
    //  Ensure when casting xs:time to xs:string that a milli seconds part of '110' is
    // XPath: xs:string(xs:time(\"21:01:23.110\")) eq \"21:01:23.11\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
