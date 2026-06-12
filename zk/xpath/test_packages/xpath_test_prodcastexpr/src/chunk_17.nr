//! Test chunk 17 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_863() {
    //  'castable as' involving xs:time as source type and xs:integer as target type sh
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_865() {
    //  'castable as' involving xs:time as source type and xs:duration as target type s
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_867() {
    //  'castable as' involving xs:time as source type and xs:yearMonthDuration as targ
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:yearMonthDura
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_869() {
    //  'castable as' involving xs:time as source type and xs:dayTimeDuration as target
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:dayTimeDurati
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_871() {
    //  'castable as' involving xs:time as source type and xs:dateTime as target type s
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_872() {
    //  Casting from xs:time to xs:time is allowed and should always succeed.
    // XPath: xs:time(\"03:20:00-05:00\") cast as xs:time eq xs:time(\"03:
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_873() {
    //  'castable as' involving xs:time as source type and xs:time as target type shoul
    // XPath: xs:time(\"03:20:00-05:00\") castable as xs:time
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_875() {
    //  'castable as' involving xs:time as source type and xs:date as target type shoul
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_877() {
    //  'castable as' involving xs:time as source type and xs:gYearMonth as target type
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:gYearMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_879() {
    //  'castable as' involving xs:time as source type and xs:gYear as target type shou
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_881() {
    //  'castable as' involving xs:time as source type and xs:gMonthDay as target type
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:gMonthDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_883() {
    //  'castable as' involving xs:time as source type and xs:gDay as target type shoul
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_885() {
    //  'castable as' involving xs:time as source type and xs:gMonth as target type sho
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_887() {
    //  'castable as' involving xs:time as source type and xs:boolean as target type sh
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_889() {
    //  'castable as' involving xs:time as source type and xs:base64Binary as target ty
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:base64Binary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_891() {
    //  'castable as' involving xs:time as source type and xs:hexBinary as target type
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:hexBinary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_893() {
    //  'castable as' involving xs:time as source type and xs:anyURI as target type sho
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_895() {
    //  'castable as' involving xs:time as source type and xs:QName as target type shou
    // XPath: not(xs:time(\"03:20:00-05:00\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_899() {
    //  Verify with 'instance of' that the xs:date constructor function produces values
    // XPath: subsequence((\"dummy\", 1.1, xs:date(\"2004-10-13\")), 3, 1)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_900() {
    //  A simple test exercising the whitespace facet for type xs:date.
    // XPath: xs:date(\" 2004-10-13 \") eq xs:date(\" 2004-10-13 \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_902() {
    //  Cast a simple xs:untypedAtomic value to '2004-10-13' .
    // XPath: xs:date(xs:untypedAtomic( \"2004-10-13\" )) eq xs:date(\"200
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_904() {
    //  Casting from xs:date to xs:untypedAtomic is allowed and should always succeed.
    // XPath: xs:date(\"2004-10-13\") cast as xs:untypedAtomic ne xs:untyp
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_905() {
    //  'castable as' involving xs:date as source type and xs:untypedAtomic as target t
    // XPath: xs:date(\"2004-10-13\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_906() {
    //  Casting from xs:date to xs:string is allowed and should always succeed.
    // XPath: xs:date(\"2004-10-13\") cast as xs:string ne xs:string(\"an
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_907() {
    //  'castable as' involving xs:date as source type and xs:string as target type sho
    // XPath: xs:date(\"2004-10-13\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_909() {
    //  'castable as' involving xs:date as source type and xs:float as target type shou
    // XPath: not(xs:date(\"2004-10-13\") castable as xs:float)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_911() {
    //  'castable as' involving xs:date as source type and xs:double as target type sho
    // XPath: not(xs:date(\"2004-10-13\") castable as xs:double)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_913() {
    //  'castable as' involving xs:date as source type and xs:decimal as target type sh
    // XPath: not(xs:date(\"2004-10-13\") castable as xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_915() {
    //  'castable as' involving xs:date as source type and xs:integer as target type sh
    // XPath: not(xs:date(\"2004-10-13\") castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_917() {
    //  'castable as' involving xs:date as source type and xs:duration as target type s
    // XPath: not(xs:date(\"2004-10-13\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_919() {
    //  'castable as' involving xs:date as source type and xs:yearMonthDuration as targ
    // XPath: not(xs:date(\"2004-10-13\") castable as xs:yearMonthDuration
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_921() {
    //  'castable as' involving xs:date as source type and xs:dayTimeDuration as target
    // XPath: not(xs:date(\"2004-10-13\") castable as xs:dayTimeDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_922() {
    //  Casting from xs:date to xs:dateTime is allowed and should always succeed.
    // XPath: xs:date(\"2004-10-13\") cast as xs:dateTime ne xs:dateTime(\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_923() {
    //  'castable as' involving xs:date as source type and xs:dateTime as target type s
    // XPath: xs:date(\"2004-10-13\") castable as xs:dateTime
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_925() {
    //  'castable as' involving xs:date as source type and xs:time as target type shoul
    // XPath: not(xs:date(\"2004-10-13\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_926() {
    //  Casting from xs:date to xs:date is allowed and should always succeed.
    // XPath: xs:date(\"2004-10-13\") cast as xs:date eq xs:date(\"2004-10
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_927() {
    //  'castable as' involving xs:date as source type and xs:date as target type shoul
    // XPath: xs:date(\"2004-10-13\") castable as xs:date
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_928() {
    //  Casting from xs:date to xs:gYearMonth is allowed and should always succeed.
    // XPath: xs:date(\"2004-10-13\") cast as xs:gYearMonth ne xs:gYearMon
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_929() {
    //  'castable as' involving xs:date as source type and xs:gYearMonth as target type
    // XPath: xs:date(\"2004-10-13\") castable as xs:gYearMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_930() {
    //  Casting from xs:date to xs:gYear is allowed and should always succeed.
    // XPath: xs:date(\"2004-10-13\") cast as xs:gYear ne xs:gYear(\"1999\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_931() {
    //  'castable as' involving xs:date as source type and xs:gYear as target type shou
    // XPath: xs:date(\"2004-10-13\") castable as xs:gYear
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_932() {
    //  Casting from xs:date to xs:gMonthDay is allowed and should always succeed.
    // XPath: xs:date(\"2004-10-13\") cast as xs:gMonthDay ne xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_933() {
    //  'castable as' involving xs:date as source type and xs:gMonthDay as target type
    // XPath: xs:date(\"2004-10-13\") castable as xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_934() {
    //  Casting from xs:date to xs:gDay is allowed and should always succeed.
    // XPath: xs:date(\"2004-10-13\") cast as xs:gDay ne xs:gDay(\"---03\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_935() {
    //  'castable as' involving xs:date as source type and xs:gDay as target type shoul
    // XPath: xs:date(\"2004-10-13\") castable as xs:gDay
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_936() {
    //  Casting from xs:date to xs:gMonth is allowed and should always succeed.
    // XPath: xs:date(\"2004-10-13\") cast as xs:gMonth ne xs:gMonth(\"--1
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_937() {
    //  'castable as' involving xs:date as source type and xs:gMonth as target type sho
    // XPath: xs:date(\"2004-10-13\") castable as xs:gMonth
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_939() {
    //  'castable as' involving xs:date as source type and xs:boolean as target type sh
    // XPath: not(xs:date(\"2004-10-13\") castable as xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_941() {
    //  'castable as' involving xs:date as source type and xs:base64Binary as target ty
    // XPath: not(xs:date(\"2004-10-13\") castable as xs:base64Binary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_943() {
    //  'castable as' involving xs:date as source type and xs:hexBinary as target type
    // XPath: not(xs:date(\"2004-10-13\") castable as xs:hexBinary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
