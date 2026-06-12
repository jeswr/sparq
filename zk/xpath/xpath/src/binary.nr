//! Binary data functions for XPath/SPARQL
//!
//! Implements comparison operations for xs:hexBinary and xs:base64Binary types.
//! In Noir, binary data is represented as fixed-size byte arrays.
//!
//! XPath operators implemented:
//! - op:hexBinary-equal, op:hexBinary-less-than, op:hexBinary-greater-than
//! - op:base64Binary-equal, op:base64Binary-less-than, op:base64Binary-greater-than

// ============================================================================
// hexBinary Comparison Functions
// ============================================================================

/// op:hexBinary-equal - Compare two hexBinary values for equality
/// Binary values are represented as byte arrays
pub fn hexbinary_equal<let N: u32>(a: [u8; N], b: [u8; N]) -> bool {
    let mut equal = true;
    for i in 0..N {
        if a[i] != b[i] {
            equal = false;
        }
    }
    equal
}

/// op:hexBinary-less-than - Compare two hexBinary values lexicographically
/// Returns true if a < b (byte-by-byte comparison)
pub fn hexbinary_less_than<let N: u32>(a: [u8; N], b: [u8; N]) -> bool {
    let mut result = false;
    let mut decided = false;

    for i in 0..N {
        if !decided {
            if a[i] < b[i] {
                result = true;
                decided = true;
            } else if a[i] > b[i] {
                result = false;
                decided = true;
            }
        }
    }

    result
}

/// op:hexBinary-greater-than - Compare two hexBinary values lexicographically
/// Returns true if a > b (byte-by-byte comparison)
pub fn hexbinary_greater_than<let N: u32>(a: [u8; N], b: [u8; N]) -> bool {
    let mut result = false;
    let mut decided = false;

    for i in 0..N {
        if !decided {
            if a[i] > b[i] {
                result = true;
                decided = true;
            } else if a[i] < b[i] {
                result = false;
                decided = true;
            }
        }
    }

    result
}

// ============================================================================
// base64Binary Comparison Functions
// (Semantically identical to hexBinary - both are just byte sequences)
// ============================================================================

/// op:base64Binary-equal - Compare two base64Binary values for equality
pub fn base64binary_equal<let N: u32>(a: [u8; N], b: [u8; N]) -> bool {
    hexbinary_equal(a, b)
}

/// op:base64Binary-less-than - Compare two base64Binary values lexicographically
pub fn base64binary_less_than<let N: u32>(a: [u8; N], b: [u8; N]) -> bool {
    hexbinary_less_than(a, b)
}

/// op:base64Binary-greater-than - Compare two base64Binary values lexicographically
pub fn base64binary_greater_than<let N: u32>(a: [u8; N], b: [u8; N]) -> bool {
    hexbinary_greater_than(a, b)
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_hexbinary_equal() {
    let a: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    let b: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    let c: [u8; 4] = [0x01, 0x02, 0x03, 0x05];

    assert(hexbinary_equal(a, b) == true);
    assert(hexbinary_equal(a, c) == false);
}

#[test]
fn test_hexbinary_less_than() {
    let a: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    let b: [u8; 4] = [0x01, 0x02, 0x03, 0x05];
    let c: [u8; 4] = [0x01, 0x02, 0x04, 0x00];

    assert(hexbinary_less_than(a, b) == true);
    assert(hexbinary_less_than(a, c) == true);
    assert(hexbinary_less_than(b, a) == false);
}

#[test]
fn test_hexbinary_greater_than() {
    let a: [u8; 4] = [0x01, 0x02, 0x03, 0x05];
    let b: [u8; 4] = [0x01, 0x02, 0x03, 0x04];

    assert(hexbinary_greater_than(a, b) == true);
    assert(hexbinary_greater_than(b, a) == false);
}
