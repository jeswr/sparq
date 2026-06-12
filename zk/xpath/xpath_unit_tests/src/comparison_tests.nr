//! Unit tests for comparison utility functions

use dep::xpath::{value_equal, value_ge, value_greater_than, value_le, value_less_than};

// ============================================================================
// value_equal Tests
// ============================================================================

#[test]
fn test_value_equal_integers_same() {
    assert(value_equal(42, 42));
}

#[test]
fn test_value_equal_integers_different() {
    assert(!value_equal(42, 43));
}

#[test]
fn test_value_equal_booleans() {
    assert(value_equal(true, true));
    assert(value_equal(false, false));
    assert(!value_equal(true, false));
}

#[test]
fn test_value_equal_u8() {
    let a: u8 = 255;
    let b: u8 = 255;
    let c: u8 = 0;
    assert(value_equal(a, b));
    assert(!value_equal(a, c));
}

// ============================================================================
// value_less_than Tests
// ============================================================================

#[test]
fn test_value_less_than_true() {
    assert(value_less_than(3 as u32, 5 as u32));
}

#[test]
fn test_value_less_than_false() {
    assert(!value_less_than(5 as u32, 3 as u32));
}

#[test]
fn test_value_less_than_equal_values() {
    assert(!value_less_than(5 as u32, 5 as u32));
}

// ============================================================================
// value_greater_than Tests
// ============================================================================

#[test]
fn test_value_greater_than_true() {
    assert(value_greater_than(5 as u32, 3 as u32));
}

#[test]
fn test_value_greater_than_false() {
    assert(!value_greater_than(3 as u32, 5 as u32));
}

#[test]
fn test_value_greater_than_equal_values() {
    assert(!value_greater_than(5 as u32, 5 as u32));
}

// ============================================================================
// value_le Tests
// ============================================================================

#[test]
fn test_value_le_less() {
    assert(value_le(3 as u32, 5 as u32));
}

#[test]
fn test_value_le_equal() {
    assert(value_le(5 as u32, 5 as u32));
}

#[test]
fn test_value_le_greater() {
    assert(!value_le(5 as u32, 3 as u32));
}

// ============================================================================
// value_ge Tests
// ============================================================================

#[test]
fn test_value_ge_greater() {
    assert(value_ge(5 as u32, 3 as u32));
}

#[test]
fn test_value_ge_equal() {
    assert(value_ge(5 as u32, 5 as u32));
}

#[test]
fn test_value_ge_less() {
    assert(!value_ge(3 as u32, 5 as u32));
}
