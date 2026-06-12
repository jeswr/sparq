//! XPath Type Casting Functions
//!
//! Provides type conversion functions between XPath/XSD datatypes.
//! These implement the casting rules defined in XPath 2.0 and XQuery 1.0.
//!
//! Key functions:
//! - `fn_boolean_from_string`: Computes EBV from string bytes
//! - `fn_boolean_from_unsigned`: Computes EBV from unsigned integer
//! - `string_to_boolean`: Converts xs:string to xs:boolean (parses "true"/"false")
//! - `boolean_to_string`: Converts xs:boolean to xs:string
//! - `integer_to_string`: Converts xs:integer to xs:string
//! - `string_to_integer`: Parses xs:integer from string
//!
//! Note: `fn_boolean_from_string_len` and `fn_boolean_from_integer` are in boolean.nr

// ============================================================================
// Effective Boolean Value (EBV) Functions (Additional)
// ============================================================================

/// Computes the Effective Boolean Value (EBV) of a string represented as bytes.
/// XPath 2.0 spec: EBV of xs:string is false if empty, true otherwise.
///
/// # Arguments
/// * `bytes` - The string bytes (unused, but represents the string content)
/// * `len` - The actual length of the string
///
/// # Returns
/// * `true` if the string is non-empty (len > 0)
/// * `false` if the string is empty (len == 0)
pub fn fn_boolean_from_string<let N: u32>(_bytes: [u8; N], len: u32) -> bool {
    len > 0
}

/// Computes the Effective Boolean Value (EBV) of an unsigned integer.
///
/// # Arguments
/// * `value` - The numeric value (as u64)
///
/// # Returns
/// * `true` if value is non-zero
/// * `false` if value is zero
pub fn fn_boolean_from_unsigned(value: u64) -> bool {
    value != 0
}

// ============================================================================
// String to Type Conversions
// ============================================================================

/// Converts the string "true" or "false" to xs:boolean.
/// Also accepts "1" and "0" per XPath spec.
///
/// # Arguments
/// * `bytes` - The string bytes
/// * `len` - The actual length of the string
///
/// # Returns
/// * `(true, true)` if string is "true" or "1"
/// * `(false, true)` if string is "false" or "0"
/// * `(false, false)` if string is invalid (second bool indicates validity)
pub fn string_to_boolean<let N: u32>(bytes: [u8; N], len: u32) -> (bool, bool) {
    let mut result_val = false;
    let mut result_valid = false;

    // Safely get bytes with default 0 if out of bounds
    let b0 = if N > 0 { bytes[0] } else { 0 };
    let b1 = if N > 1 { bytes[1] } else { 0 };
    let b2 = if N > 2 { bytes[2] } else { 0 };
    let b3 = if N > 3 { bytes[3] } else { 0 };
    let b4 = if N > 4 { bytes[4] } else { 0 };

    // Check for "true" (case-sensitive per XSD)
    let is_true = (len == 4)
        & (b0 == 116) // 't'
        & (b1 == 114) // 'r'
        & (b2 == 117) // 'u'
        & (b3 == 101); // 'e'

    if is_true {
        result_val = true;
        result_valid = true;
    }

    // Check for "false" (case-sensitive per XSD)
    let is_false = (len == 5)
        & (b0 == 102) // 'f'
        & (b1 == 97) // 'a'
        & (b2 == 108) // 'l'
        & (b3 == 115) // 's'
        & (b4 == 101); // 'e'

    if is_false {
        result_val = false;
        result_valid = true;
    }

    // Check for "1"
    let is_one = (len == 1) & (b0 == 49);
    if is_one {
        result_val = true;
        result_valid = true;
    }

    // Check for "0"
    let is_zero = (len == 1) & (b0 == 48);
    if is_zero {
        result_val = false;
        result_valid = true;
    }

    (result_val, result_valid)
}

/// Parses a string of decimal digits to an unsigned integer.
/// Returns (value, success).
///
/// # Arguments
/// * `bytes` - The string bytes containing digits
/// * `len` - The actual length of the string
///
/// # Returns
/// * `(value, true)` if parsing succeeded
/// * `(0, false)` if parsing failed (non-digit character)
pub fn string_to_unsigned<let N: u32>(bytes: [u8; N], len: u32) -> (u64, bool) {
    let mut result: u64 = 0;
    let mut valid = true;

    // Empty string is invalid
    if len == 0 {
        valid = false;
    }

    for i in 0..N {
        if (i < len) & valid {
            let c = bytes[i];
            // Check if digit (0x30 = '0', 0x39 = '9')
            if (c >= 48) & (c <= 57) {
                result = result * 10 + (c - 48) as u64;
            } else {
                valid = false;
            }
        }
    }

    (result, valid)
}

/// Parses a string to a signed integer (handles optional leading '-' or '+').
/// Returns (value, success).
///
/// # Arguments
/// * `bytes` - The string bytes
/// * `len` - The actual length of the string
///
/// # Returns
/// * `(value, true)` if parsing succeeded
/// * `(0, false)` if parsing failed
pub fn string_to_integer<let N: u32>(bytes: [u8; N], len: u32) -> (i64, bool) {
    // Empty string is invalid
    let mut valid = len > 0;

    let mut start: u32 = 0;
    let mut negative = false;

    // Check for sign (only if we have at least one char)
    if len > 0 {
        if bytes[0] == 45 {
            // '-'
            negative = true;
            start = 1;
        } else if bytes[0] == 43 {
            // '+'
            start = 1;
        }
    }

    // Must have at least one digit after sign
    if start >= len {
        valid = false;
    }

    let mut result: i64 = 0;

    for i in 0..N {
        if (i >= start) & (i < len) & valid {
            let c = bytes[i];
            // Check if digit
            if (c >= 48) & (c <= 57) {
                result = result * 10 + (c - 48) as i64;
            } else {
                valid = false;
            }
        }
    }

    let final_result = if valid {
        if negative {
            -result
        } else {
            result
        }
    } else {
        0
    };

    (final_result, valid)
}

// ============================================================================
// Type to String Conversions
// ============================================================================

/// Converts xs:boolean to xs:string.
///
/// # Arguments
/// * `value` - The boolean value
///
/// # Returns
/// * `([u8; 5], u32)` - bytes and length ("true" = 4, "false" = 5)
pub fn boolean_to_string(value: bool) -> ([u8; 5], u32) {
    let mut result: [u8; 5] = [0; 5];

    if value {
        // "true"
        result[0] = 116; // 't'
        result[1] = 114; // 'r'
        result[2] = 117; // 'u'
        result[3] = 101; // 'e'
        (result, 4)
    } else {
        // "false"
        result[0] = 102; // 'f'
        result[1] = 97; // 'a'
        result[2] = 108; // 'l'
        result[3] = 115; // 's'
        result[4] = 101; // 'e'
        (result, 5)
    }
}

/// Converts an unsigned integer to xs:string.
///
/// # Arguments
/// * `value` - The unsigned integer value
///
/// # Returns
/// * `([u8; N], u32)` - bytes and actual length
pub fn unsigned_to_string<let N: u32>(value: u64) -> ([u8; N], u32) {
    let mut result: [u8; N] = [0; N];

    // Handle zero case
    let is_zero = value == 0;
    if is_zero {
        result[0] = 48; // '0'
    }

    // Count digits (only if non-zero)
    let mut temp = value;
    let mut digit_count: u32 = 0;
    for _ in 0..20 {
        // Max 20 digits for u64
        if temp > 0 {
            digit_count += 1;
            temp = temp / 10;
        }
    }

    // Write digits in reverse order (only if non-zero)
    let mut write_pos = digit_count;
    let mut v = value;
    for _ in 0..20 {
        if v > 0 {
            write_pos -= 1;
            if write_pos < N {
                result[write_pos] = 48 + (v % 10) as u8; // '0' + digit
            }
            v = v / 10;
        }
    }

    let final_len = if is_zero { 1 } else { digit_count };
    (result, final_len)
}

/// Converts a signed integer to xs:string.
///
/// # Arguments
/// * `value` - The signed integer value
///
/// # Returns
/// * `([u8; N], u32)` - bytes and actual length
pub fn integer_to_string<let N: u32>(value: i64) -> ([u8; N], u32) {
    let mut result: [u8; N] = [0; N];

    // Handle zero case
    let is_zero = value == 0;
    if is_zero {
        result[0] = 48; // '0'
    }

    let negative = value < 0;
    let abs_value: u64 = if negative {
        (-value) as u64
    } else {
        value as u64
    };

    // Count digits (only for non-zero)
    let mut temp = abs_value;
    let mut digit_count: u32 = 0;
    for _ in 0..20 {
        if temp > 0 {
            digit_count += 1;
            temp = temp / 10;
        }
    }

    let total_len = if is_zero {
        1
    } else if negative {
        digit_count + 1
    } else {
        digit_count
    };

    // Write sign if negative (only for non-zero)
    let start_pos: u32 = if negative & !is_zero {
        result[0] = 45; // '-'
        1
    } else {
        0
    };

    // Write digits in reverse order (only for non-zero)
    let mut write_pos = start_pos + digit_count;
    let mut v = abs_value;
    for _ in 0..20 {
        if v > 0 {
            write_pos -= 1;
            if write_pos < N {
                result[write_pos] = 48 + (v % 10) as u8;
            }
            v = v / 10;
        }
    }

    (result, total_len)
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_fn_boolean_from_string() {
    // Test with actual bytes - empty string
    assert(fn_boolean_from_string("".as_bytes(), 0) == false);
    // Non-empty
    assert(fn_boolean_from_string("hello".as_bytes(), 5) == true);
    assert(fn_boolean_from_string("x".as_bytes(), 1) == true);
}

#[test]
fn test_fn_boolean_from_unsigned() {
    assert(fn_boolean_from_unsigned(0) == false);
    assert(fn_boolean_from_unsigned(1) == true);
    assert(fn_boolean_from_unsigned(42) == true);
    assert(fn_boolean_from_unsigned(18446744073709551615) == true); // max u64
}

#[test]
fn test_string_to_boolean() {
    // "true"
    let (val1, ok1) = string_to_boolean("true".as_bytes(), 4);
    assert(ok1 == true);
    assert(val1 == true);

    // "false"
    let (val2, ok2) = string_to_boolean("false".as_bytes(), 5);
    assert(ok2 == true);
    assert(val2 == false);

    // "1"
    let (val3, ok3) = string_to_boolean("1".as_bytes(), 1);
    assert(ok3 == true);
    assert(val3 == true);

    // "0"
    let (val4, ok4) = string_to_boolean("0".as_bytes(), 1);
    assert(ok4 == true);
    assert(val4 == false);

    // Invalid
    let (_val5, ok5) = string_to_boolean("yes".as_bytes(), 3);
    assert(ok5 == false);
}

#[test]
fn test_boolean_to_string() {
    let (bytes1, len1) = boolean_to_string(true);
    assert(len1 == 4);
    let expected1 = "true".as_bytes();
    for i in 0..4 {
        assert(bytes1[i] == expected1[i]);
    }

    let (bytes2, len2) = boolean_to_string(false);
    assert(len2 == 5);
    let expected2 = "false".as_bytes();
    for i in 0..5 {
        assert(bytes2[i] == expected2[i]);
    }
}

#[test]
fn test_string_to_integer() {
    // Positive number
    let (val1, ok1) = string_to_integer("42".as_bytes(), 2);
    assert(ok1 == true);
    assert(val1 == 42);

    // Negative number
    let (val2, ok2) = string_to_integer("-123".as_bytes(), 4);
    assert(ok2 == true);
    assert(val2 == -123);

    // With plus sign
    let (val3, ok3) = string_to_integer("+456".as_bytes(), 4);
    assert(ok3 == true);
    assert(val3 == 456);

    // Zero
    let (val4, ok4) = string_to_integer("0".as_bytes(), 1);
    assert(ok4 == true);
    assert(val4 == 0);

    // Invalid
    let (_val5, ok5) = string_to_integer("abc".as_bytes(), 3);
    assert(ok5 == false);
}

#[test]
fn test_integer_to_string() {
    // Positive
    let (bytes1, len1): ([u8; 10], u32) = integer_to_string(42);
    assert(len1 == 2);
    assert(bytes1[0] == 52); // '4'
    assert(bytes1[1] == 50); // '2'

    // Negative
    let (bytes2, len2): ([u8; 10], u32) = integer_to_string(-123);
    assert(len2 == 4);
    assert(bytes2[0] == 45); // '-'
    assert(bytes2[1] == 49); // '1'
    assert(bytes2[2] == 50); // '2'
    assert(bytes2[3] == 51); // '3'

    // Zero
    let (bytes3, len3): ([u8; 10], u32) = integer_to_string(0);
    assert(len3 == 1);
    assert(bytes3[0] == 48); // '0'
}

#[test]
fn test_unsigned_to_string() {
    // Regular number
    let (bytes1, len1): ([u8; 10], u32) = unsigned_to_string(12345);
    assert(len1 == 5);
    let expected1 = "12345".as_bytes();
    for i in 0..5 {
        assert(bytes1[i] == expected1[i]);
    }

    // Zero
    let (bytes2, len2): ([u8; 10], u32) = unsigned_to_string(0);
    assert(len2 == 1);
    assert(bytes2[0] == 48); // '0'
}
