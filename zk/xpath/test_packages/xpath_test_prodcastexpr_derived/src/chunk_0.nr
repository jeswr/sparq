//! Test chunk 0 for prod:CastExpr.derived
//! Contains 50 tests

use dep::xpath::stub_prodcastexpr_derived;

#[test]
fn cast_derived_1() {
    // Casting from float to decimal.
    // XPath: let $value := xs:float(10.0) return $value cast as xs:decima
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_2() {
    //  Casting from float to integer.
    // XPath: let $value := xs:float(10.0) return $value cast as xs:decima
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_3() {
    //  Casting from float to nonPositiveInteger.
    // XPath: let $value := xs:float(-10.0) return $value cast as xs:nonPo
    // Expected: -10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_4() {
    //  Casting from float to a long.
    // XPath: let $value := xs:float(10.0) return $value cast as xs:long
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_5() {
    //  Casting from float to a nonNegativeInteger.
    // XPath: let $value := xs:float(10.0) return $value cast as xs:nonNeg
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_6() {
    //  Casting from float to a negativeInteger.
    // XPath: let $value := xs:float(-10.0) return $value cast as xs:negat
    // Expected: -10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_7() {
    //  Casting from float to an int.
    // XPath: let $value := xs:float(10.0) return $value cast as xs:int
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_8() {
    //  Casting from float to an unsignedLong.
    // XPath: let $value := xs:float(10.0) return $value cast as xs:unsign
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_9() {
    //  Casting from float to a positiveInteger.
    // XPath: let $value := xs:float(10.0) return $value cast as xs:positi
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_10() {
    //  Casting from float to a short.
    // XPath: let $value := xs:float(10.0) return $value cast as xs:short
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_11() {
    //  Casting from float to an unsignedInt.
    // XPath: let $value := xs:float(10.0) return $value cast as xs:unsign
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_12() {
    //  Casting from float to an unsignedShort.
    // XPath: let $value := xs:float(10.0) return $value cast as xs:unsign
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_13() {
    //  Casting from float to an unsignedByte.
    // XPath: let $value := xs:float(10.0) return $value cast as xs:unsign
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_14() {
    //  Casting from float to a byte.
    // XPath: let $value := xs:float(10.0) return $value cast as xs:byte
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_15() {
    //  Casting from double to a decimal.
    // XPath: let $value := xs:double(10E2) return $value cast as xs:decim
    // Expected: 1000
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_16() {
    //  Casting from double to an integer.
    // XPath: let $value := xs:double(10E2) return $value cast as xs:integ
    // Expected: 1000
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_17() {
    //  Casting from double to an positiveInteger.
    // XPath: let $value := xs:double(10E2) return $value cast as xs:posit
    // Expected: 1000
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_18() {
    //  Casting from double to a long.
    // XPath: let $value := xs:double(10E2) return $value cast as xs:long
    // Expected: 1000
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_19() {
    //  Casting from double to an int.
    // XPath: let $value := xs:double(10E2) return $value cast as xs:int
    // Expected: 1000
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_20() {
    //  Casting from double to an unsignedLong.
    // XPath: let $value := xs:double(10E2) return $value cast as xs:unsig
    // Expected: 1000
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_derived_21() {
    //  Casting from double to a short.
    // XPath: let $value := xs:double(10E2) return $value cast as xs:short
    // Expected: 1000
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_1() {
    // Casting from decimal to float.
    // XPath: let $value := xs:decimal(10.0) return $value cast as xs:floa
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_2() {
    //  Casting from intger to float.
    // XPath: let $value := xs:integer(10.0) return $value cast as xs:floa
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_3() {
    //  Casting from nonPositiveInteger to float.
    // XPath: let $value := xs:nonPositiveInteger(-10.0) return $value cas
    // Expected: -10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_4() {
    //  Casting from long to float.
    // XPath: let $value := xs:long(10.0) return $value cast as xs:float
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_5() {
    //  Casting from nonNegativeInteger to float.
    // XPath: let $value := xs:nonNegativeInteger(10.0) return $value cast
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_6() {
    //  Casting from a negativeInteger to a float.
    // XPath: let $value := xs:negativeInteger(-10.0) return $value cast a
    // Expected: -10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_7() {
    //  Casting from an int to a float.
    // XPath: let $value := xs:int(10.0) return $value cast as xs:float
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_8() {
    //  Casting from an unsignedLong to a float.
    // XPath: let $value := xs:unsignedLong(10.0) return $value cast as xs
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_9() {
    //  Casting from a positiveInteger to a float.
    // XPath: let $value := xs:positiveInteger(10.0) return $value cast as
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_10() {
    //  Casting from a shortto a float.
    // XPath: let $value := xs:short(10.0) return $value cast as xs:float
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_11() {
    //  Casting from an unsignedInt to a float.
    // XPath: let $value := xs:unsignedInt(10.0) return $value cast as xs:
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_12() {
    //  Casting from an unsignedShort to a float.
    // XPath: let $value := xs:unsignedShort(10.0) return $value cast as x
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_13() {
    //  Casting from an unsignedByte to a float.
    // XPath: let $value := xs:unsignedByte(10.0) return $value cast as xs
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_14() {
    //  Casting from a byte to a float.
    // XPath: let $value := xs:byte(10.0) return $value cast as xs:float
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_15() {
    //  Casting from a decimal to a double.
    // XPath: let $value := xs:decimal(10E2) return $value cast as xs:doub
    // Expected: 1000
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_16() {
    //  Casting from an integer to a double.
    // XPath: let $value := xs:integer(10E2) return $value cast as xs:doub
    // Expected: 1000
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_17() {
    //  Casting from a positiveInteger to a double.
    // XPath: let $value := xs:positiveInteger(10E2) return $value cast as
    // Expected: 1000
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_18() {
    //  Casting from a long to a double.
    // XPath: let $value := xs:long(10E2) return $value cast as xs:double
    // Expected: 1000
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_19() {
    //  Casting from an int to a double.
    // XPath: let $value := xs:int(10E2) return $value cast as xs:double
    // Expected: 1000
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_20() {
    //  Casting from an unsignedLong to a double.
    // XPath: let $value := xs:unsignedLong(10E2) return $value cast as xs
    // Expected: 1000
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_to_parent_21() {
    //  Casting from a short to a double.
    // XPath: let $value := xs:short(10E2) return $value cast as xs:double
    // Expected: 1000
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_1() {
    // Casting from integer to nonPositiveInteger.
    // XPath: let $value := xs:integer(-10.0) return $value cast as xs:non
    // Expected: -10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_2() {
    //  Casting from integer to long.
    // XPath: let $value := xs:integer(10.0) return $value cast as xs:long
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_3() {
    //  Casting from integer to nonNegativeInteger.
    // XPath: let $value := xs:integer(10.0) return $value cast as xs:nonN
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_4() {
    //  Casting from integer to negativeInteger.
    // XPath: let $value := xs:integer(-10.0) return $value cast as xs:neg
    // Expected: -10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_5() {
    //  Casting from integer to int.
    // XPath: let $value := xs:integer(10.0) return $value cast as xs:int
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_6() {
    //  Casting from integer to unsignedLong.
    // XPath: let $value := xs:integer(10.0) return $value cast as xs:unsi
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_7() {
    //  Casting from integer to positiveInteger.
    // XPath: let $value := xs:integer(10.0) return $value cast as xs:posi
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}

#[test]
fn cast_within_8() {
    //  Casting from integer to short.
    // XPath: let $value := xs:integer(10.0) return $value cast as xs:shor
    // Expected: 10
    // Calls stub function - will fail until prod:CastExpr.derived is implemented
    let _ = stub_prodcastexpr_derived();
}
