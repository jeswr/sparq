//! Comparison utilities for XPath/SPARQL
//!
//! Provides generic comparison functions for values.

/// Generic value equality using the Eq trait
pub fn value_equal<T>(a: T, b: T) -> bool
where
    T: Eq,
{
    a == b
}

/// Generic value less-than using the Ord trait
pub fn value_less_than<T>(a: T, b: T) -> bool
where
    T: Ord,
{
    a < b
}

/// Generic value greater-than using the Ord trait
pub fn value_greater_than<T>(a: T, b: T) -> bool
where
    T: Ord,
{
    a > b
}

/// Generic value less-than-or-equal using the Ord trait
pub fn value_le<T>(a: T, b: T) -> bool
where
    T: Ord,
{
    a <= b
}

/// Generic value greater-than-or-equal using the Ord trait
pub fn value_ge<T>(a: T, b: T) -> bool
where
    T: Ord,
{
    a >= b
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_value_equal() {
    assert(value_equal(5, 5) == true);
    assert(value_equal(5, 3) == false);
    assert(value_equal(true, true) == true);
    assert(value_equal(true, false) == false);
}

#[test]
fn test_value_less_than() {
    assert(value_less_than(3 as u32, 5 as u32) == true);
    assert(value_less_than(5 as u32, 3 as u32) == false);
    assert(value_less_than(5 as u32, 5 as u32) == false);
}

#[test]
fn test_value_greater_than() {
    assert(value_greater_than(5 as u32, 3 as u32) == true);
    assert(value_greater_than(3 as u32, 5 as u32) == false);
    assert(value_greater_than(5 as u32, 5 as u32) == false);
}

#[test]
fn test_value_le() {
    assert(value_le(3 as u32, 5 as u32) == true);
    assert(value_le(5 as u32, 5 as u32) == true);
    assert(value_le(5 as u32, 3 as u32) == false);
}

#[test]
fn test_value_ge() {
    assert(value_ge(5 as u32, 3 as u32) == true);
    assert(value_ge(5 as u32, 5 as u32) == true);
    assert(value_ge(3 as u32, 5 as u32) == false);
}
