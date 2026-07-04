//! Numeric functions and operators for XPath/SPARQL
//!
//! Implements numeric operations for integers.
//! Float operations require the ieee754 dependency (future).
//!
//! XPath operators implemented:
//! - op:numeric-add, op:numeric-subtract, op:numeric-multiply
//! - op:numeric-integer-divide (`idiv`, truncating -- `numeric_divide_int`;
//!   `op:numeric-divide` over integers returns the xs:decimal quotient and
//!   lives in `numeric_types` as `numeric_divide_int_as_double`)
//! - op:numeric-unary-plus, op:numeric-unary-minus
//! - op:numeric-equal, op:numeric-less-than, op:numeric-greater-than
//! - fn:abs

// ============================================================================
// Integer Operations (using i64)
// ============================================================================

/// op:numeric-add for integers
pub fn numeric_add_int(a: i64, b: i64) -> i64 {
    a + b
}

/// op:numeric-subtract for integers
pub fn numeric_subtract_int(a: i64, b: i64) -> i64 {
    a - b
}

/// op:numeric-multiply for integers
pub fn numeric_multiply_int(a: i64, b: i64) -> i64 {
    a * b
}

/// op:numeric-integer-divide (`idiv`) for integers -- truncating division,
/// rounding toward zero: `7 idiv 2 = 3`, `-7 idiv 2 = -3` (Noir's i64 `/`
/// truncates toward zero, matching XPath F&O 4.2.7).
///
/// This is NOT `op:numeric-divide`: on two xs:integer operands `div` returns
/// the xs:decimal quotient (`7 div 2 = 3.5`) -- see
/// `numeric_types::numeric_divide_int_as_double`, which `xpath_op::numeric_divide`
/// wires to. The two operators are distinct code paths. [FABLE-5]
///
/// Note: Division by zero will cause an assertion failure (err:FOAR0001)
pub fn numeric_divide_int(a: i64, b: i64) -> i64 {
    assert(b != 0, "Division by zero");
    a / b
}

/// op:numeric-unary-plus for integers
/// Returns the value unchanged (identity operation)
pub fn numeric_unary_plus_int(a: i64) -> i64 {
    a
}

/// op:numeric-unary-minus for integers
/// Returns the negation of the value
pub fn numeric_unary_minus_int(a: i64) -> i64 {
    -a
}

/// op:numeric-equal for integers
pub fn numeric_equal_int(a: i64, b: i64) -> bool {
    a == b
}

/// op:numeric-less-than for integers
pub fn numeric_less_than_int(a: i64, b: i64) -> bool {
    a < b
}

/// op:numeric-greater-than for integers
pub fn numeric_greater_than_int(a: i64, b: i64) -> bool {
    a > b
}

/// op:numeric-less-than-or-equal for integers
pub fn numeric_le_int(a: i64, b: i64) -> bool {
    a <= b
}

/// op:numeric-greater-than-or-equal for integers
pub fn numeric_ge_int(a: i64, b: i64) -> bool {
    a >= b
}

/// fn:abs for integers
/// Returns the absolute value
pub fn abs_int(a: i64) -> i64 {
    if a < 0 {
        -a
    } else {
        a
    }
}

/// op:numeric-mod for integers
/// Returns the remainder of integer division
pub fn numeric_mod_int(a: i64, b: i64) -> i64 {
    assert(b != 0, "Modulo by zero");
    a % b
}

/// fn:round for integers
/// Returns the value unchanged (identity operation for integers)
pub fn round_int(a: i64) -> i64 {
    a
}

/// fn:ceiling for integers
/// Returns the value unchanged (identity operation for integers)
pub fn ceil_int(a: i64) -> i64 {
    a
}

/// fn:floor for integers
/// Returns the value unchanged (identity operation for integers)
pub fn floor_int(a: i64) -> i64 {
    a
}

/// Minimum of two integers
pub fn min_int(a: i64, b: i64) -> i64 {
    if a < b {
        a
    } else {
        b
    }
}

/// Maximum of two integers
pub fn max_int(a: i64, b: i64) -> i64 {
    if a > b {
        a
    } else {
        b
    }
}

/// fn:round-half-to-even for integers with precision
/// Rounds to the specified number of decimal places using banker's rounding.
/// For integers, precision 0 returns the value unchanged.
/// Negative precision rounds to powers of 10 (e.g., precision -1 rounds to nearest 10).
/// Banker's rounding: if exactly halfway, round to nearest even number.
pub fn round_half_to_even_int(value: i64, precision: i32) -> i64 {
    if precision >= 0 {
        // For integers with non-negative precision, no rounding needed
        value
    } else {
        // Negative precision: round to power of 10
        // precision = -1 means round to nearest 10
        // precision = -2 means round to nearest 100, etc.
        let scale = pow10(-precision as u32);
        let half_scale = scale / 2;

        // Get the remainder
        let remainder = if value >= 0 {
            value % scale
        } else {
            // For negative numbers, we need to handle differently
            let abs_val = -value;
            let abs_rem = abs_val % scale;
            if abs_rem == 0 {
                0
            } else {
                scale - abs_rem
            }
        };

        // Calculate the rounded-down value
        let rounded_down = value - remainder;

        // Determine if we should round up
        let should_round_up = if remainder > half_scale {
            true
        } else if remainder < half_scale {
            false
        } else {
            // Exactly halfway - round to even
            // Check if rounded_down / scale is odd
            let quotient = rounded_down / scale;
            let is_odd = (quotient % 2 != 0);
            is_odd
        };

        if should_round_up {
            rounded_down + scale
        } else {
            rounded_down
        }
    }
}

/// Helper function to compute 10^n for small n
fn pow10(n: u32) -> i64 {
    let mut result: i64 = 1;
    for _i in 0..n {
        result = result * 10;
    }
    result
}

// ============================================================================
// Number Conversion Functions
// ============================================================================

/// fn:number from boolean - Converts boolean to number
/// true -> 1, false -> 0
pub fn fn_number_from_bool(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

/// fn:number from integer - Identity function
pub fn fn_number_from_integer(value: i64) -> i64 {
    value
}

/// fn:number from string - Parses an integer from a string
/// Returns (value, is_valid) - is_valid is false if parsing fails (fn:number
/// maps a failed cast to NaN; the boolean is that NaN indicator).
///
/// Whitespace rule (documented): fn:number delegates to the xs:double cast,
/// and xs:double carries the XSD `whiteSpace="collapse"` facet -- LEADING and
/// TRAILING whitespace (#x20, #x9, #xA, #xD) is stripped before the numeral
/// is matched, but INTERIOR whitespace is not in the lexical space. So
/// `fn:number(' 12 ')` is 12 while `fn:number('1 2')` is NaN -- the numeral
/// ends at the first whitespace after it starts, and any later
/// non-whitespace byte invalidates the parse. Trailing NUL bytes are also
/// accepted as padding (fixed-width `str<N>` values). [FABLE-5]
///
/// Scope limitation (pre-existing, unchanged here): only integer numerals are
/// parsed -- no decimal point or exponent forms yet.
pub fn fn_number_from_string<let N: u32>(s: str<N>) -> (i64, bool) {
    let bytes = s.as_bytes();
    let mut index: u32 = 0;
    let mut value: i64 = 0;
    let mut negative = false;
    let mut has_digits = false;
    let mut is_valid = true;

    // Skip leading whitespace
    for i in 0..N {
        if (i as u32) == index {
            let b = bytes[i];
            if (b == 0x20) | (b == 0x09) | (b == 0x0A) | (b == 0x0D) {
                // space, tab, newline, carriage return
                index += 1;
            }
        }
    }

    // Check for sign
    for i in 0..N {
        if ((i as u32) == index) & is_valid {
            let b = bytes[i];
            if b == 0x2D {
                // '-'
                negative = true;
                index += 1;
            } else if b == 0x2B {
                // '+'
                index += 1;
            }
        }
    }

    // Parse digits. Once a whitespace/NUL byte is seen after this point the
    // numeral has ENDED: only more whitespace/NUL may follow. A digit (or any
    // other byte) after that is interior junk and invalidates the parse --
    // fn:number('1 2') must be NaN, not 12 (xsd whiteSpace="collapse" strips
    // only leading/trailing whitespace). [FABLE-5]
    let mut ended = false;
    for i in 0..N {
        if ((i as u32) >= index) & is_valid {
            let b = bytes[i];
            let is_ws = (b == 0x20) | (b == 0x09) | (b == 0x0A) | (b == 0x0D) | (b == 0x00);
            if is_ws {
                // Trailing whitespace or NUL padding - the numeral is over
                ended = true;
            } else if ended {
                // Non-whitespace after the numeral ended (interior junk)
                is_valid = false;
            } else if (b >= 0x30) & (b <= 0x39) {
                // '0'-'9'
                has_digits = true;
                value = value * 10 + ((b - 0x30) as i64);
            } else {
                // Invalid character
                is_valid = false;
            }
        }
    }

    if !has_digits {
        is_valid = false;
    }

    let result_value = if negative { -value } else { value };

    (result_value, is_valid)
}
