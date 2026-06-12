//! Unit tests for sequence and context functions

use dep::xpath::{default_collation, default_language, fn_data_identity, implicit_timezone};

// ============================================================================
// Tests for implicit_timezone (fn:implicit-timezone)
// ============================================================================

#[test]
fn test_implicit_timezone_utc() {
    let result = implicit_timezone(0);
    assert(result == 0);
}

#[test]
fn test_implicit_timezone_positive_offset() {
    let result = implicit_timezone(300); // UTC+5
    assert(result == 300);
}

#[test]
fn test_implicit_timezone_negative_offset() {
    let result = implicit_timezone(-480); // UTC-8
    assert(result == -480);
}

#[test]
fn test_implicit_timezone_half_hour_offset() {
    let result = implicit_timezone(330); // UTC+5:30 (India)
    assert(result == 330);
}

// ============================================================================
// Tests for default_collation (fn:default-collation)
// ============================================================================

#[test]
fn test_default_collation_returns_codepoint_uri() {
    let (result, len) = default_collation::<60>();

    // Expected: "http://www.w3.org/2005/xpath-functions/collation/codepoint"
    assert(len == 58);

    // Check first few bytes: "http"
    assert(result[0] == 104); // 'h'
    assert(result[1] == 116); // 't'
    assert(result[2] == 116); // 't'
    assert(result[3] == 112); // 'p'
}

#[test]
fn test_default_collation_large_buffer() {
    let (result, len) = default_collation::<100>();

    assert(len == 58);
    // Extra bytes should be zero
    assert(result[58] == 0);
    assert(result[99] == 0);
}

// ============================================================================
// Tests for default_language (fn:default-language)
// ============================================================================

#[test]
fn test_default_language_returns_en() {
    let (result, len) = default_language::<10>();

    assert(len == 2);
    assert(result[0] == 101); // 'e'
    assert(result[1] == 110); // 'n'
}

#[test]
fn test_default_language_extra_bytes_zero() {
    let (result, len) = default_language::<10>();

    assert(len == 2);
    // Extra bytes should be zero
    assert(result[2] == 0);
    assert(result[9] == 0);
}

// ============================================================================
// Tests for fn_data_identity (fn:data)
// ============================================================================

#[test]
fn test_fn_data_identity_integer() {
    let value: i64 = 42;
    let result = fn_data_identity(value);
    assert(result == 42);
}

#[test]
fn test_fn_data_identity_negative() {
    let value: i64 = -100;
    let result = fn_data_identity(value);
    assert(result == -100);
}

#[test]
fn test_fn_data_identity_boolean_true() {
    let value = true;
    let result = fn_data_identity(value);
    assert(result == true);
}

#[test]
fn test_fn_data_identity_boolean_false() {
    let value = false;
    let result = fn_data_identity(value);
    assert(result == false);
}

#[test]
fn test_fn_data_identity_u32() {
    let value: u32 = 12345;
    let result = fn_data_identity(value);
    assert(result == 12345);
}
