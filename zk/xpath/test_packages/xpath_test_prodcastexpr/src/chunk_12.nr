//! Test chunk 12 for prod:CastExpr
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr;

#[test]
fn k_seqexprcast_449() {
    //  'castable as' involving xs:float as source type and xs:float as target type sho
    // XPath: xs:float(\"3.4e5\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_450() {
    //  Casting from xs:float to xs:double is allowed and should always succeed.
    // XPath: xs:float(\"3.4e5\") cast as xs:double ne xs:double(\"3.3e3\"
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_451() {
    //  'castable as' involving xs:float as source type and xs:double as target type sh
    // XPath: xs:float(\"3.4e5\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_453() {
    //  'castable as' involving xs:float as source type and xs:duration as target type
    // XPath: not(xs:float(\"3.4e5\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_455() {
    //  'castable as' involving xs:float as source type and xs:yearMonthDuration as tar
    // XPath: not(xs:float(\"3.4e5\") castable as xs:yearMonthDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_457() {
    //  'castable as' involving xs:float as source type and xs:dayTimeDuration as targe
    // XPath: not(xs:float(\"3.4e5\") castable as xs:dayTimeDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_459() {
    //  'castable as' involving xs:float as source type and xs:dateTime as target type
    // XPath: not(xs:float(\"3.4e5\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_461() {
    //  'castable as' involving xs:float as source type and xs:time as target type shou
    // XPath: not(xs:float(\"3.4e5\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_463() {
    //  'castable as' involving xs:float as source type and xs:date as target type shou
    // XPath: not(xs:float(\"3.4e5\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_465() {
    //  'castable as' involving xs:float as source type and xs:gYearMonth as target typ
    // XPath: not(xs:float(\"3.4e5\") castable as xs:gYearMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_467() {
    //  'castable as' involving xs:float as source type and xs:gYear as target type sho
    // XPath: not(xs:float(\"3.4e5\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_469() {
    //  'castable as' involving xs:float as source type and xs:gMonthDay as target type
    // XPath: not(xs:float(\"3.4e5\") castable as xs:gMonthDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_471() {
    //  'castable as' involving xs:float as source type and xs:gDay as target type shou
    // XPath: not(xs:float(\"3.4e5\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_473() {
    //  'castable as' involving xs:float as source type and xs:gMonth as target type sh
    // XPath: not(xs:float(\"3.4e5\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_474() {
    //  Casting from xs:float to xs:boolean is allowed and should always succeed.
    // XPath: xs:float(\"3.4e5\") cast as xs:boolean eq xs:boolean(\"true\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_475() {
    //  'castable as' involving xs:float as source type and xs:boolean as target type s
    // XPath: xs:float(\"3.4e5\") castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_477() {
    //  'castable as' involving xs:float as source type and xs:base64Binary as target t
    // XPath: not(xs:float(\"3.4e5\") castable as xs:base64Binary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_479() {
    //  'castable as' involving xs:float as source type and xs:hexBinary as target type
    // XPath: not(xs:float(\"3.4e5\") castable as xs:hexBinary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_481() {
    //  'castable as' involving xs:float as source type and xs:anyURI as target type sh
    // XPath: not(xs:float(\"3.4e5\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_483() {
    //  'castable as' involving xs:float as source type and xs:QName as target type sho
    // XPath: not(xs:float(\"3.4e5\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_487() {
    //  Verify with 'instance of' that the xs:double constructor function produces valu
    // XPath: subsequence((\"dummy\", 1.1, xs:double(\"3.3e3\")), 3, 1) in
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_488() {
    //  A simple test exercising the whitespace facet for type xs:double.
    // XPath: xs:double(\" 3.3e3 \") eq xs:double(\" 3.3e3 \")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_490() {
    //  Cast a simple xs:untypedAtomic value to '3.3e3' .
    // XPath: xs:double(xs:untypedAtomic( \"3.3e3\" )) eq xs:double(\"3.3e
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_491() {
    //  Applying fn:boolean() to a value of type xs:double that has the lexical value '
    // XPath: boolean(xs:double(\"3.3e3\"))
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_492() {
    //  Casting from xs:double to xs:untypedAtomic is allowed and should always succeed
    // XPath: xs:double(\"3.3e3\") cast as xs:untypedAtomic ne xs:untypedA
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_493() {
    //  'castable as' involving xs:double as source type and xs:untypedAtomic as target
    // XPath: xs:double(\"3.3e3\") castable as xs:untypedAtomic
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_494() {
    //  Casting from xs:double to xs:string is allowed and should always succeed.
    // XPath: xs:double(\"3.3e3\") cast as xs:string ne xs:string(\"an arb
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_495() {
    //  'castable as' involving xs:double as source type and xs:string as target type s
    // XPath: xs:double(\"3.3e3\") castable as xs:string
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_496() {
    //  Casting from xs:double to xs:float is allowed and should always succeed.
    // XPath: xs:double(\"3.3e3\") cast as xs:float ne xs:float(\"3.4e5\")
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_497() {
    //  'castable as' involving xs:double as source type and xs:float as target type sh
    // XPath: xs:double(\"3.3e3\") castable as xs:float
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_498() {
    //  Casting from xs:double to xs:double is allowed and should always succeed.
    // XPath: xs:double(\"3.3e3\") cast as xs:double eq xs:double(\"3.3e3\
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_499() {
    //  'castable as' involving xs:double as source type and xs:double as target type s
    // XPath: xs:double(\"3.3e3\") castable as xs:double
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_501() {
    //  'castable as' involving xs:double as source type and xs:duration as target type
    // XPath: not(xs:double(\"3.3e3\") castable as xs:duration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_503() {
    //  'castable as' involving xs:double as source type and xs:yearMonthDuration as ta
    // XPath: not(xs:double(\"3.3e3\") castable as xs:yearMonthDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_505() {
    //  'castable as' involving xs:double as source type and xs:dayTimeDuration as targ
    // XPath: not(xs:double(\"3.3e3\") castable as xs:dayTimeDuration)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_507() {
    //  'castable as' involving xs:double as source type and xs:dateTime as target type
    // XPath: not(xs:double(\"3.3e3\") castable as xs:dateTime)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_509() {
    //  'castable as' involving xs:double as source type and xs:time as target type sho
    // XPath: not(xs:double(\"3.3e3\") castable as xs:time)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_511() {
    //  'castable as' involving xs:double as source type and xs:date as target type sho
    // XPath: not(xs:double(\"3.3e3\") castable as xs:date)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_513() {
    //  'castable as' involving xs:double as source type and xs:gYearMonth as target ty
    // XPath: not(xs:double(\"3.3e3\") castable as xs:gYearMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_515() {
    //  'castable as' involving xs:double as source type and xs:gYear as target type sh
    // XPath: not(xs:double(\"3.3e3\") castable as xs:gYear)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_517() {
    //  'castable as' involving xs:double as source type and xs:gMonthDay as target typ
    // XPath: not(xs:double(\"3.3e3\") castable as xs:gMonthDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_519() {
    //  'castable as' involving xs:double as source type and xs:gDay as target type sho
    // XPath: not(xs:double(\"3.3e3\") castable as xs:gDay)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_521() {
    //  'castable as' involving xs:double as source type and xs:gMonth as target type s
    // XPath: not(xs:double(\"3.3e3\") castable as xs:gMonth)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_522() {
    //  Casting from xs:double to xs:boolean is allowed and should always succeed.
    // XPath: xs:double(\"3.3e3\") cast as xs:boolean eq xs:boolean(\"true
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_523() {
    //  'castable as' involving xs:double as source type and xs:boolean as target type
    // XPath: xs:double(\"3.3e3\") castable as xs:boolean
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_525() {
    //  'castable as' involving xs:double as source type and xs:base64Binary as target
    // XPath: not(xs:double(\"3.3e3\") castable as xs:base64Binary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_527() {
    //  'castable as' involving xs:double as source type and xs:hexBinary as target typ
    // XPath: not(xs:double(\"3.3e3\") castable as xs:hexBinary)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_529() {
    //  'castable as' involving xs:double as source type and xs:anyURI as target type s
    // XPath: not(xs:double(\"3.3e3\") castable as xs:anyURI)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_531() {
    //  'castable as' involving xs:double as source type and xs:QName as target type sh
    // XPath: not(xs:double(\"3.3e3\") castable as xs:QName)
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}

#[test]
fn k_seqexprcast_535() {
    //  Verify with 'instance of' that the xs:decimal constructor function produces val
    // XPath: subsequence((\"dummy\", 1.1, xs:decimal(\"10.01\")), 3, 1) i
    // Expected: true
    // Calls stub function - will fail until prod:CastExpr is implemented
    let _ = stub_prodcastexpr();
}
