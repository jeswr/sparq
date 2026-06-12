//! Unit tests for binary comparison functions

use dep::xpath::{
    base64binary_equal, base64binary_greater_than, base64binary_less_than, hexbinary_equal,
    hexbinary_greater_than, hexbinary_less_than,
};

// ============================================================================
// HexBinary Equality Tests
// ============================================================================

#[test]
fn test_hexbinary_equal_same() {
    let a: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    let b: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    assert(hexbinary_equal(a, b));
}

#[test]
fn test_hexbinary_equal_different() {
    let a: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    let b: [u8; 4] = [0x01, 0x02, 0x03, 0x05];
    assert(!hexbinary_equal(a, b));
}

#[test]
fn test_hexbinary_equal_empty() {
    let a: [u8; 0] = [];
    let b: [u8; 0] = [];
    assert(hexbinary_equal(a, b));
}

// ============================================================================
// HexBinary Less Than Tests
// ============================================================================

#[test]
fn test_hexbinary_less_than_first_byte() {
    let a: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    let b: [u8; 4] = [0x02, 0x02, 0x03, 0x04];
    assert(hexbinary_less_than(a, b));
    assert(!hexbinary_less_than(b, a));
}

#[test]
fn test_hexbinary_less_than_last_byte() {
    let a: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    let b: [u8; 4] = [0x01, 0x02, 0x03, 0x05];
    assert(hexbinary_less_than(a, b));
}

#[test]
fn test_hexbinary_less_than_equal_values() {
    let a: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    let b: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    assert(!hexbinary_less_than(a, b));
}

// ============================================================================
// HexBinary Greater Than Tests
// ============================================================================

#[test]
fn test_hexbinary_greater_than_first_byte() {
    let a: [u8; 4] = [0x02, 0x02, 0x03, 0x04];
    let b: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    assert(hexbinary_greater_than(a, b));
    assert(!hexbinary_greater_than(b, a));
}

#[test]
fn test_hexbinary_greater_than_last_byte() {
    let a: [u8; 4] = [0x01, 0x02, 0x03, 0x05];
    let b: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    assert(hexbinary_greater_than(a, b));
}

#[test]
fn test_hexbinary_greater_than_equal_values() {
    let a: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    let b: [u8; 4] = [0x01, 0x02, 0x03, 0x04];
    assert(!hexbinary_greater_than(a, b));
}

// ============================================================================
// Base64Binary Tests (delegates to hexbinary)
// ============================================================================

#[test]
fn test_base64binary_equal() {
    let a: [u8; 4] = [0x48, 0x65, 0x6c, 0x6c]; // "Hell" in ASCII
    let b: [u8; 4] = [0x48, 0x65, 0x6c, 0x6c];
    let c: [u8; 4] = [0x48, 0x65, 0x6c, 0x6f]; // "Helo"
    assert(base64binary_equal(a, b));
    assert(!base64binary_equal(a, c));
}

#[test]
fn test_base64binary_less_than() {
    let a: [u8; 4] = [0x48, 0x65, 0x6c, 0x6c];
    let b: [u8; 4] = [0x48, 0x65, 0x6c, 0x6f];
    assert(base64binary_less_than(a, b));
    assert(!base64binary_less_than(b, a));
}

#[test]
fn test_base64binary_greater_than() {
    let a: [u8; 4] = [0x48, 0x65, 0x6c, 0x6f];
    let b: [u8; 4] = [0x48, 0x65, 0x6c, 0x6c];
    assert(base64binary_greater_than(a, b));
    assert(!base64binary_greater_than(b, a));
}
