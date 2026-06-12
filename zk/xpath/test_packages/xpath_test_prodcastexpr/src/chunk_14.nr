//! Test chunk 14 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_609() {
    //  'castable as' involving xs:integer as source type and xs:dayTimeDuration as tar
    // XPath: not(xs:integer(\"6789\") castable as xs:dayTimeDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_611() {
    //  'castable as' involving xs:integer as source type and xs:dateTime as target typ
    // XPath: not(xs:integer(\"6789\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_613() {
    //  'castable as' involving xs:integer as source type and xs:time as target type sh
    // XPath: not(xs:integer(\"6789\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_615() {
    //  'castable as' involving xs:integer as source type and xs:date as target type sh
    // XPath: not(xs:integer(\"6789\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_617() {
    //  'castable as' involving xs:integer as source type and xs:gYearMonth as target t
    // XPath: not(xs:integer(\"6789\") castable as xs:gYearMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_619() {
    //  'castable as' involving xs:integer as source type and xs:gYear as target type s
    // XPath: not(xs:integer(\"6789\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_621() {
    //  'castable as' involving xs:integer as source type and xs:gMonthDay as target ty
    // XPath: not(xs:integer(\"6789\") castable as xs:gMonthDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_623() {
    //  'castable as' involving xs:integer as source type and xs:gDay as target type sh
    // XPath: not(xs:integer(\"6789\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_625() {
    //  'castable as' involving xs:integer as source type and xs:gMonth as target type
    // XPath: not(xs:integer(\"6789\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_626() {
    //  Casting from xs:integer to xs:boolean is allowed and should always succeed.
    // XPath: xs:integer(\"6789\") cast as xs:boolean eq xs:boolean(\"true
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_627() {
    //  'castable as' involving xs:integer as source type and xs:boolean as target type
    // XPath: xs:integer(\"6789\") castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_629() {
    //  'castable as' involving xs:integer as source type and xs:base64Binary as target
    // XPath: not(xs:integer(\"6789\") castable as xs:base64Binary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_631() {
    //  'castable as' involving xs:integer as source type and xs:hexBinary as target ty
    // XPath: not(xs:integer(\"6789\") castable as xs:hexBinary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_633() {
    //  'castable as' involving xs:integer as source type and xs:anyURI as target type
    // XPath: not(xs:integer(\"6789\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_635() {
    //  'castable as' involving xs:integer as source type and xs:QName as target type s
    // XPath: not(xs:integer(\"6789\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_639() {
    //  Verify with 'instance of' that the xs:duration constructor function produces va
    // XPath: subsequence((\"dummy\", 1.1, xs:duration(\"P1Y2M3DT10H30M\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_640() {
    //  A simple test exercising the whitespace facet for type xs:duration.
    // XPath: xs:duration(\" P1Y2M3DT10H30M \") eq xs:duration(\" P1Y2M3DT
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_642() {
    //  Cast a simple xs:untypedAtomic value to 'P1Y2M3DT10H30M' .
    // XPath: xs:duration(xs:untypedAtomic( \"P1Y2M3DT10H30M\" )) eq xs:du
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_644() {
    //  Casting from xs:duration to xs:untypedAtomic is allowed and should always succe
    // XPath: xs:duration(\"P1Y2M3DT10H30M\") cast as xs:untypedAtomic ne
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_645() {
    //  'castable as' involving xs:duration as source type and xs:untypedAtomic as targ
    // XPath: xs:duration(\"P1Y2M3DT10H30M\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_646() {
    //  Casting from xs:duration to xs:string is allowed and should always succeed.
    // XPath: xs:duration(\"P1Y2M3DT10H30M\") cast as xs:string ne xs:stri
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_647() {
    //  'castable as' involving xs:duration as source type and xs:string as target type
    // XPath: xs:duration(\"P1Y2M3DT10H30M\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_649() {
    //  'castable as' involving xs:duration as source type and xs:float as target type
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:float)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_651() {
    //  'castable as' involving xs:duration as source type and xs:double as target type
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:double)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_653() {
    //  'castable as' involving xs:duration as source type and xs:decimal as target typ
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:decimal)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_655() {
    //  'castable as' involving xs:duration as source type and xs:integer as target typ
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:integer)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_656() {
    //  Casting from xs:duration to xs:duration is allowed and should always succeed.
    // XPath: xs:duration(\"P1Y2M3DT10H30M\") cast as xs:duration eq xs:du
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_657() {
    //  'castable as' involving xs:duration as source type and xs:duration as target ty
    // XPath: xs:duration(\"P1Y2M3DT10H30M\") castable as xs:duration
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_658() {
    //  Casting from xs:duration to xs:yearMonthDuration is allowed and should always s
    // XPath: xs:duration(\"P1Y2M3DT10H30M\") cast as xs:yearMonthDuration
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_659() {
    //  'castable as' involving xs:duration as source type and xs:yearMonthDuration as
    // XPath: xs:duration(\"P1Y2M3DT10H30M\") castable as xs:yearMonthDura
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_660() {
    //  Casting from xs:duration to xs:dayTimeDuration is allowed and should always suc
    // XPath: xs:duration(\"P1Y2M3DT10H30M\") cast as xs:dayTimeDuration n
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_661() {
    //  'castable as' involving xs:duration as source type and xs:dayTimeDuration as ta
    // XPath: xs:duration(\"P1Y2M3DT10H30M\") castable as xs:dayTimeDurati
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_663() {
    //  'castable as' involving xs:duration as source type and xs:dateTime as target ty
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_665() {
    //  'castable as' involving xs:duration as source type and xs:time as target type s
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_667() {
    //  'castable as' involving xs:duration as source type and xs:date as target type s
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_669() {
    //  'castable as' involving xs:duration as source type and xs:gYearMonth as target
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:gYearMont
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_671() {
    //  'castable as' involving xs:duration as source type and xs:gYear as target type
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_673() {
    //  'castable as' involving xs:duration as source type and xs:gMonthDay as target t
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:gMonthDay
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_675() {
    //  'castable as' involving xs:duration as source type and xs:gDay as target type s
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_677() {
    //  'castable as' involving xs:duration as source type and xs:gMonth as target type
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_679() {
    //  'castable as' involving xs:duration as source type and xs:boolean as target typ
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:boolean)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_681() {
    //  'castable as' involving xs:duration as source type and xs:base64Binary as targe
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:base64Bin
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_683() {
    //  'castable as' involving xs:duration as source type and xs:hexBinary as target t
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:hexBinary
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_685() {
    //  'castable as' involving xs:duration as source type and xs:anyURI as target type
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_687() {
    //  'castable as' involving xs:duration as source type and xs:QName as target type
    // XPath: not(xs:duration(\"P1Y2M3DT10H30M\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_691() {
    //  Verify with 'instance of' that the xs:yearMonthDuration constructor function pr
    // XPath: subsequence((\"dummy\", 1.1, xs:yearMonthDuration(\"P1Y12M\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_692() {
    //  A simple test exercising the whitespace facet for type xs:yearMonthDuration.
    // XPath: xs:yearMonthDuration(\" P1Y12M \") eq xs:yearMonthDuration(\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_694() {
    //  Cast a simple xs:untypedAtomic value to 'P1Y12M' .
    // XPath: xs:yearMonthDuration(xs:untypedAtomic( \"P1Y12M\" )) eq xs:y
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_696() {
    //  Casting from xs:yearMonthDuration to xs:untypedAtomic is allowed and should alw
    // XPath: xs:yearMonthDuration(\"P1Y12M\") cast as xs:untypedAtomic ne
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_697() {
    //  'castable as' involving xs:yearMonthDuration as source type and xs:untypedAtomi
    // XPath: xs:yearMonthDuration(\"P1Y12M\") castable as xs:untypedAtomi
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
