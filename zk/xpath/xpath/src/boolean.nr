//! Boolean functions and operators for XPath/SPARQL
//!
//! Implements:
//! - fn:not (logical NOT)
//! - Logical AND and OR
//! - Boolean comparison operators

/// XPath fn:not - Logical NOT
/// Returns true if the argument is false, and false if the argument is true.
pub fn fn_not(a: bool) -> bool {
    !a
}

/// Logical AND operation
/// Returns true if both arguments are true.
pub fn logical_and(a: bool, b: bool) -> bool {
    a & b
}

/// Logical OR operation
/// Returns true if at least one argument is true.
pub fn logical_or(a: bool, b: bool) -> bool {
    a | b
}

/// op:boolean-equal
/// Returns true if both arguments have the same boolean value.
pub fn boolean_equal(a: bool, b: bool) -> bool {
    a == b
}

/// op:boolean-less-than
/// Returns true if the first argument is less than the second.
/// In XPath boolean ordering: false < true
pub fn boolean_less_than(a: bool, b: bool) -> bool {
    // false (0) < true (1)
    // Only true when a is false and b is true
    (!a) & b
}

/// op:boolean-greater-than
/// Returns true if the first argument is greater than the second.
/// In XPath boolean ordering: true > false
pub fn boolean_greater_than(a: bool, b: bool) -> bool {
    // true (1) > false (0)
    // Only true when a is true and b is false
    a & (!b)
}

/// op:boolean-less-than-or-equal
/// Returns true if the first argument is less than or equal to the second.
/// In XPath boolean ordering: false <= true, false <= false, true <= true
pub fn boolean_le(a: bool, b: bool) -> bool {
    // a <= b is equivalent to (a < b) OR (a == b)
    // Or more simply: NOT (a > b)
    !(a & (!b))
}

/// op:boolean-greater-than-or-equal
/// Returns true if the first argument is greater than or equal to the second.
/// In XPath boolean ordering: true >= false, true >= true, false >= false
pub fn boolean_ge(a: bool, b: bool) -> bool {
    // a >= b is equivalent to (a > b) OR (a == b)
    // Or more simply: NOT (a < b)
    !((!a) & b)
}

/// fn:true - Returns the boolean value true
/// XPath: fn:true() as xs:boolean
pub fn fn_true() -> bool {
    true
}

/// fn:false - Returns the boolean value false
/// XPath: fn:false() as xs:boolean
pub fn fn_false() -> bool {
    false
}

// ============================================================================
// Error and Trace Functions
// ============================================================================

/// fn:error - Raises an error
/// In Noir/ZK context, this causes an assertion failure which prevents proof generation
/// This is the correct semantic: invalid inputs should not produce valid proofs
pub fn fn_error() {
    assert(false, "fn:error called");
}

/// fn:error with description
/// Raises an error with a custom message (though message is compile-time only in Noir)
pub fn fn_error_with_desc<let N: u32>(_description: str<N>) {
    // Note: In Noir, assertion messages are compile-time only
    // The description cannot be used dynamically
    assert(false, "fn:error called with description");
}

/// fn:trace - Returns its second argument and is intended to provide tracing
/// In Noir/ZK context, we can't print at runtime, so we just return the value
/// The trace message would need to be handled at compile time or ignored
pub fn fn_trace<T>(value: T, _label: str<32>) -> T {
    // In ZK proofs, we can't output anything at runtime
    // Just return the value unchanged
    value
}

// ============================================================================
// Effective Boolean Value
// ============================================================================

/// fn:boolean for numeric values - Computes effective boolean value
/// Returns false if the number is 0 or NaN, true otherwise
/// For our integer-based representation, 0 maps to false, non-zero to true
pub fn fn_boolean_from_integer(value: i64) -> bool {
    value != 0
}

/// fn:boolean for string values - Computes effective boolean value
/// Returns false if the string is empty, true otherwise
pub fn fn_boolean_from_string_len(string_len: u32) -> bool {
    string_len > 0
}

/// fn:boolean for sequences - Computes effective boolean value
/// Returns false if the sequence is empty, true if non-empty
/// Note: For sequences of one boolean, return that boolean value
pub fn fn_boolean_from_sequence_len(seq_len: u32) -> bool {
    seq_len > 0
}

/// fn:boolean identity - Boolean to boolean (passthrough)
pub fn fn_boolean_from_bool(value: bool) -> bool {
    value
}
