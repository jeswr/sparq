//! Test chunk 11 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_374() {
    //  The canonical form of an xs:time containing 24:00:00 is 00:00:00.
    // XPath: string(xs:time(\"24:00:00\")) eq \"00:00:00\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_389() {
    //  Simple test of casting a xs:date with UTC timezone to xs:gYear.
    // XPath: xs:gYear(xs:date(\"2002-11-23Z\")) eq xs:gYear(\"2002Z\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_390() {
    //  Simple test of casting a xs:date with UTC offset to xs:gYear.
    // XPath: xs:gYear(xs:date(\"2002-11-23-13:37\")) eq xs:gYear(\"2002-1
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_391() {
    //  Simple test of casting a xs:date with UTC timezone to xs:gYearMonth.
    // XPath: xs:gYearMonth(xs:date(\"2002-11-23Z\")) eq xs:gYearMonth(\"2
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_392() {
    //  Simple test of casting a xs:date with UTC offset to xs:gYearMonth.
    // XPath: xs:gYearMonth(xs:date(\"2002-11-23-13:37\")) eq xs:gYearMont
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_393() {
    //  Simple test of casting a xs:date with UTC timezone to xs:gMonth.
    // XPath: xs:gMonth(xs:date(\"2002-11-23-00:00\")) eq xs:gMonth(\"--11
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_394() {
    //  Simple test of casting a xs:date with UTC offset to xs:gMonth.
    // XPath: xs:gMonth(xs:date(\"2002-11-23-13:37\")) eq xs:gMonth(\"--11
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_395() {
    //  Simple test of casting a xs:date with UTC timezone to xs:gMonthDay.
    // XPath: xs:gMonthDay(xs:date(\"2002-11-23-00:00\")) eq xs:gMonthDay(
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_396() {
    //  Simple test of casting a xs:date with UTC offset to xs:gMonthDay.
    // XPath: xs:gMonthDay(xs:date(\"2002-11-23-13:37\")) eq xs:gMonthDay(
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_397() {
    //  Simple test of casting a xs:date with UTC timezone to xs:gDay.
    // XPath: xs:gDay(xs:date(\"2002-11-23-00:00\")) eq xs:gDay(\"---23Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_398() {
    //  Simple test of casting a xs:date with UTC offset to xs:gDay.
    // XPath: xs:gDay(xs:date(\"2002-11-23-13:37\")) eq xs:gDay(\"---23-13
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_399() {
    //  Simple test of casting a xs:date with UTC timezone to xs:time.
    // XPath: xs:dateTime(xs:date(\"2002-11-23-00:00\")) eq xs:dateTime(\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_400() {
    //  Simple test of casting a xs:date with UTC offset to xs:dateTime.
    // XPath: xs:dateTime(xs:date(\"2002-11-23-13:37\")) eq xs:dateTime(\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_401() {
    //  Ensure casting xs:date to xs:string, with timezone '-00:00' is properly handled
    // XPath: xs:string(xs:date(\"1999-12-01-00:00\")) eq \"1999-12-01Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_402() {
    //  Ensure casting xs:date to xs:string, with timezone '+00:00' is properly handled
    // XPath: xs:string(xs:date(\"1999-12-01+00:00\")) eq \"1999-12-01Z\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_403() {
    //  Ensure when casting xs:date to xs:string that an empty milli seconds part is no
    // XPath: xs:string(xs:date(\"2002-02-15\")) eq \"2002-02-15\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_404() {
    //  Ensure when casting xs:date to xs:string that milli seconds have no trailing ze
    // XPath: xs:string(xs:date(\"2002-02-15\")) eq \"2002-02-15\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_405() {
    //  Ensure when casting xs:date to xs:string that milli seconds are properly padded
    // XPath: xs:string(xs:date(\"2002-02-15\")) eq \"2002-02-15\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_406() {
    //  Ensure when casting xs:date to xs:string that milli seconds are properly padded
    // XPath: xs:string(xs:date(\"2002-02-15\")) eq \"2002-02-15\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_407() {
    //  Ensure when casting xs:date to xs:string that a milli seconds part of '435' is
    // XPath: xs:string(xs:date(\"2002-02-15\")) eq \"2002-02-15\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_408() {
    //  Ensure when casting xs:date to xs:string that a milli seconds part of '11' is h
    // XPath: xs:string(xs:date(\"2002-02-15\")) eq \"2002-02-15\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_409() {
    //  Ensure when casting xs:date to xs:string that a milli seconds part of '110' is
    // XPath: xs:string(xs:date(\"2002-02-15\")) eq \"2002-02-15\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_413() {
    //  Verify with 'instance of' that the xs:untypedAtomic constructor function produc
    // XPath: subsequence((\"dummy\", 1.1, xs:untypedAtomic(\"an arbitrary
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_414() {
    //  A simple test exercising the whitespace facet for type xs:untypedAtomic.
    // XPath: xs:untypedAtomic(\" an arbitrary string(untypedAtomic source
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_415() {
    //  An empty string is a valid lexical representation of xs:untypedAtomic.
    // XPath: xs:untypedAtomic(\"\") eq xs:untypedAtomic(\"\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_416() {
    //  Cast a simple xs:untypedAtomic value to 'an arbitrary string(untypedAtomic sour
    // XPath: xs:untypedAtomic(xs:untypedAtomic( \"an arbitrary string(unt
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_417() {
    //  Applying fn:boolean() to a value of type xs:untypedAtomic that has the lexical
    // XPath: boolean(xs:untypedAtomic(\"an arbitrary string(untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_418() {
    //  Casting from xs:untypedAtomic to xs:untypedAtomic is allowed and should always
    // XPath: xs:untypedAtomic(\"an arbitrary string(untypedAtomic source)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_419() {
    //  'castable as' involving xs:untypedAtomic as source type and xs:untypedAtomic as
    // XPath: xs:untypedAtomic(\"an arbitrary string(untypedAtomic source)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_420() {
    //  Casting from xs:untypedAtomic to xs:string is allowed and should always succeed
    // XPath: xs:untypedAtomic(\"an arbitrary string(untypedAtomic source)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_421() {
    //  'castable as' involving xs:untypedAtomic as source type and xs:string as target
    // XPath: xs:untypedAtomic(\"an arbitrary string(untypedAtomic source)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_423() {
    //  'castable as' involving xs:untypedAtomic as source type and xs:QName as target
    // XPath: not(xs:untypedAtomic(\"an arbitrary string(untypedAtomic sou
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_427() {
    //  Verify with 'instance of' that the xs:string constructor function produces valu
    // XPath: subsequence((\"dummy\", 1.1, xs:string(\"an arbitrary string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_428() {
    //  A simple test exercising the whitespace facet for type xs:string.
    // XPath: xs:string(\" an arbitrary string \") eq xs:string(\" an arbi
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_429() {
    //  An empty string is a valid lexical representation of xs:string.
    // XPath: xs:string(\"\") eq xs:string(\"\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_430() {
    //  Cast a simple xs:untypedAtomic value to 'an arbitrary string' .
    // XPath: xs:string(xs:untypedAtomic( \"an arbitrary string\" )) eq xs
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_431() {
    //  Applying fn:boolean() to a value of type xs:string that has the lexical value '
    // XPath: boolean(xs:string(\"an arbitrary string\"))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_432() {
    //  Casting from xs:string to xs:untypedAtomic is allowed and should always succeed
    // XPath: xs:string(\"an arbitrary string\") cast as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_433() {
    //  'castable as' involving xs:string as source type and xs:untypedAtomic as target
    // XPath: xs:string(\"an arbitrary string\") castable as xs:untypedAto
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_434() {
    //  Casting from xs:string to xs:string is allowed and should always succeed.
    // XPath: xs:string(\"an arbitrary string\") cast as xs:string eq xs:s
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_435() {
    //  'castable as' involving xs:string as source type and xs:string as target type s
    // XPath: xs:string(\"an arbitrary string\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_439() {
    //  Verify with 'instance of' that the xs:float constructor function produces value
    // XPath: subsequence((\"dummy\", 1.1, xs:float(\"3.4e5\")), 3, 1) ins
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_440() {
    //  A simple test exercising the whitespace facet for type xs:float.
    // XPath: xs:float(\" 3.4e5 \") eq xs:float(\" 3.4e5 \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_442() {
    //  Cast a simple xs:untypedAtomic value to '3.4e5' .
    // XPath: xs:float(xs:untypedAtomic( \"3.4e5\" )) eq xs:float(\"3.4e5\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_443() {
    //  Applying fn:boolean() to a value of type xs:float that has the lexical value '
    // XPath: boolean(xs:float(\"3.4e5\"))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_444() {
    //  Casting from xs:float to xs:untypedAtomic is allowed and should always succeed.
    // XPath: xs:float(\"3.4e5\") cast as xs:untypedAtomic ne xs:untypedAt
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_445() {
    //  'castable as' involving xs:float as source type and xs:untypedAtomic as target
    // XPath: xs:float(\"3.4e5\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_446() {
    //  Casting from xs:float to xs:string is allowed and should always succeed.
    // XPath: xs:float(\"3.4e5\") cast as xs:string ne xs:string(\"an arbi
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_447() {
    //  'castable as' involving xs:float as source type and xs:string as target type sh
    // XPath: xs:float(\"3.4e5\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_448() {
    //  Casting from xs:float to xs:float is allowed and should always succeed.
    // XPath: xs:float(\"3.4e5\") cast as xs:float eq xs:float(\"3.4e5\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
