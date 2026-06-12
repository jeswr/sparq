//! Duration functions for XPath/SPARQL
//!
//! Implements xsd:dayTimeDuration operations.
//! Duration values are stored as signed microseconds in a Field.
//!
//! XPath operators implemented:
//! - op:add-dayTimeDurations, op:subtract-dayTimeDurations
//! - op:multiply-dayTimeDuration, op:divide-dayTimeDuration
//! - op:add-dayTimeDuration-to-dateTime, op:subtract-dayTimeDuration-from-dateTime
//! - op:dayTimeDuration-equal, op:dayTimeDuration-less-than, op:dayTimeDuration-greater-than

use crate::types::{XsdDateTime, XsdDayTimeDuration};

// ============================================================================
// Time Constants (as i64 for signed arithmetic)
// ============================================================================

/// Microseconds per second
global MICROS_PER_SECOND: i64 = 1_000_000;

/// Microseconds per minute
global MICROS_PER_MINUTE: i64 = 60_000_000;

/// Microseconds per hour
global MICROS_PER_HOUR: i64 = 3_600_000_000;

/// Microseconds per day
global MICROS_PER_DAY: i64 = 86_400_000_000;

// ============================================================================
// Duration Construction Functions
// ============================================================================

/// Create a duration from total microseconds (can be negative)
pub fn duration_from_microseconds(micros: i64) -> XsdDayTimeDuration {
    XsdDayTimeDuration::from_signed_micros(micros)
}

/// Create a duration from components
/// days, hours, minutes, seconds can be any values; they will be normalized
pub fn duration_from_components(
    negative: bool,
    days: u32,
    hours: u32,
    minutes: u32,
    seconds: u32,
    microseconds: u32,
) -> XsdDayTimeDuration {
    let total: i64 = (days as i64) * MICROS_PER_DAY
        + (hours as i64) * MICROS_PER_HOUR
        + (minutes as i64) * MICROS_PER_MINUTE
        + (seconds as i64) * MICROS_PER_SECOND
        + microseconds as i64;

    let signed_total = if negative { -total } else { total };
    XsdDayTimeDuration::from_signed_micros(signed_total)
}

/// Create a zero duration
pub fn duration_zero() -> XsdDayTimeDuration {
    XsdDayTimeDuration::zero()
}

// ============================================================================
// Duration Component Extraction
// ============================================================================

/// Extract total microseconds from duration (signed)
pub fn duration_to_microseconds(dur: XsdDayTimeDuration) -> i64 {
    dur.to_signed_micros()
}

/// Check if duration is negative
pub fn duration_is_negative(dur: XsdDayTimeDuration) -> bool {
    dur.to_signed_micros() < 0
}

/// Extract days component (absolute value)
pub fn days_from_duration(dur: XsdDayTimeDuration) -> u32 {
    let micros = dur.to_signed_micros();
    let abs_micros: i64 = if micros < 0 { -micros } else { micros };
    (abs_micros / MICROS_PER_DAY) as u32
}

/// Extract hours component (0-23, absolute value)
pub fn hours_from_duration(dur: XsdDayTimeDuration) -> u8 {
    let micros = dur.to_signed_micros();
    let abs_micros: i64 = if micros < 0 { -micros } else { micros };
    let day_remainder = abs_micros % MICROS_PER_DAY;
    (day_remainder / MICROS_PER_HOUR) as u8
}

/// Extract minutes component (0-59, absolute value)
pub fn minutes_from_duration(dur: XsdDayTimeDuration) -> u8 {
    let micros = dur.to_signed_micros();
    let abs_micros: i64 = if micros < 0 { -micros } else { micros };
    let hour_remainder = abs_micros % MICROS_PER_HOUR;
    (hour_remainder / MICROS_PER_MINUTE) as u8
}

/// Extract seconds component (0-59, absolute value)
pub fn seconds_from_duration(dur: XsdDayTimeDuration) -> u8 {
    let micros = dur.to_signed_micros();
    let abs_micros: i64 = if micros < 0 { -micros } else { micros };
    let minute_remainder = abs_micros % MICROS_PER_MINUTE;
    (minute_remainder / MICROS_PER_SECOND) as u8
}

// ============================================================================
// Duration Arithmetic
// ============================================================================

/// op:add-dayTimeDurations
/// Adds two durations
pub fn duration_add(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> XsdDayTimeDuration {
    let sum = a.to_signed_micros() + b.to_signed_micros();
    XsdDayTimeDuration::from_signed_micros(sum)
}

/// op:subtract-dayTimeDurations
/// Subtracts second duration from first
pub fn duration_subtract(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> XsdDayTimeDuration {
    let diff = a.to_signed_micros() - b.to_signed_micros();
    XsdDayTimeDuration::from_signed_micros(diff)
}

/// op:multiply-dayTimeDuration
/// Multiplies a duration by a scalar
pub fn duration_multiply(dur: XsdDayTimeDuration, factor: i64) -> XsdDayTimeDuration {
    let result = dur.to_signed_micros() * factor;
    XsdDayTimeDuration::from_signed_micros(result)
}

/// op:divide-dayTimeDuration
/// Divides a duration by a scalar
pub fn duration_divide(dur: XsdDayTimeDuration, divisor: i64) -> XsdDayTimeDuration {
    assert(divisor != 0, "Division by zero");
    let result = dur.to_signed_micros() / divisor;
    XsdDayTimeDuration::from_signed_micros(result)
}

/// op:divide-dayTimeDuration-by-dayTimeDuration
/// Divides two durations, returning a ratio as i64
pub fn duration_divide_by_duration(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> i64 {
    let b_micros = b.to_signed_micros();
    assert(b_micros != 0, "Division by zero duration");
    a.to_signed_micros() / b_micros
}

/// Negates a duration
pub fn duration_negate(dur: XsdDayTimeDuration) -> XsdDayTimeDuration {
    XsdDayTimeDuration::from_signed_micros(-dur.to_signed_micros())
}

// ============================================================================
// DateTime + Duration Arithmetic
// ============================================================================

/// op:add-dayTimeDuration-to-dateTime
/// Adds a duration to a dateTime
pub fn datetime_add_duration(dt: XsdDateTime, dur: XsdDayTimeDuration) -> XsdDateTime {
    let dt_micros: i64 = dt.epoch_microseconds as i64;
    let dur_micros = dur.to_signed_micros();
    let result = dt_micros + dur_micros;
    assert(result >= 0, "DateTime cannot be negative");
    let result_u64: u64 = result as u64;
    XsdDateTime::new(result_u64 as Field)
}

/// op:subtract-dayTimeDuration-from-dateTime
/// Subtracts a duration from a dateTime
pub fn datetime_subtract_duration(dt: XsdDateTime, dur: XsdDayTimeDuration) -> XsdDateTime {
    let dt_micros: i64 = dt.epoch_microseconds as i64;
    let dur_micros = dur.to_signed_micros();
    let result = dt_micros - dur_micros;
    assert(result >= 0, "DateTime cannot be negative");
    let result_u64: u64 = result as u64;
    XsdDateTime::new(result_u64 as Field)
}

/// op:subtract-dateTimes
/// Returns the duration between two dateTimes (a - b)
pub fn datetime_difference(a: XsdDateTime, b: XsdDateTime) -> XsdDayTimeDuration {
    let a_micros: i64 = a.epoch_microseconds as i64;
    let b_micros: i64 = b.epoch_microseconds as i64;
    XsdDayTimeDuration::from_signed_micros(a_micros - b_micros)
}

// ============================================================================
// Duration Comparison
// ============================================================================

/// op:dayTimeDuration-equal
pub fn duration_equal(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> bool {
    a.to_signed_micros() == b.to_signed_micros()
}

/// op:dayTimeDuration-less-than
pub fn duration_less_than(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> bool {
    a.to_signed_micros() < b.to_signed_micros()
}

/// op:dayTimeDuration-greater-than
pub fn duration_greater_than(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> bool {
    a.to_signed_micros() > b.to_signed_micros()
}

/// op:dayTimeDuration-less-than-or-equal
pub fn duration_le(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> bool {
    a.to_signed_micros() <= b.to_signed_micros()
}

/// op:dayTimeDuration-greater-than-or-equal
pub fn duration_ge(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> bool {
    a.to_signed_micros() >= b.to_signed_micros()
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_duration_from_components() {
    // 1 day, 2 hours, 30 minutes, 45 seconds
    let dur = duration_from_components(false, 1, 2, 30, 45, 0);
    assert(days_from_duration(dur) == 1);
    assert(hours_from_duration(dur) == 2);
    assert(minutes_from_duration(dur) == 30);
    assert(seconds_from_duration(dur) == 45);
    assert(duration_is_negative(dur) == false);
}

#[test]
fn test_duration_negative() {
    let dur = duration_from_components(true, 1, 0, 0, 0, 0);
    assert(duration_is_negative(dur) == true);
    assert(days_from_duration(dur) == 1);
}

#[test]
fn test_duration_arithmetic() {
    let d1 = duration_from_components(false, 1, 0, 0, 0, 0); // 1 day
    let d2 = duration_from_components(false, 0, 12, 0, 0, 0); // 12 hours

    // Add: 1 day + 12 hours = 1.5 days
    let sum = duration_add(d1, d2);
    assert(days_from_duration(sum) == 1);
    assert(hours_from_duration(sum) == 12);

    // Subtract: 1 day - 12 hours = 12 hours
    let diff = duration_subtract(d1, d2);
    assert(days_from_duration(diff) == 0);
    assert(hours_from_duration(diff) == 12);

    // Multiply: 12 hours * 2 = 1 day
    let product = duration_multiply(d2, 2);
    assert(days_from_duration(product) == 1);
    assert(hours_from_duration(product) == 0);

    // Divide: 1 day / 2 = 12 hours
    let quotient = duration_divide(d1, 2);
    assert(days_from_duration(quotient) == 0);
    assert(hours_from_duration(quotient) == 12);
}

#[test]
fn test_datetime_duration_arithmetic() {
    // Start at Unix epoch
    let dt = XsdDateTime::new(0);
    let one_day = duration_from_components(false, 1, 0, 0, 0, 0);

    // Add one day
    let dt_plus = datetime_add_duration(dt, one_day);
    let expected_micros: Field = 86_400_000_000;
    assert(dt_plus.epoch_microseconds == expected_micros);

    // Subtract back
    let dt_back = datetime_subtract_duration(dt_plus, one_day);
    assert(dt_back.epoch_microseconds == 0);
}

#[test]
fn test_datetime_difference() {
    let dt1 = XsdDateTime::new(0);
    let dt2_micros: Field = 86_400_000_000; // 1 day in microseconds
    let dt2 = XsdDateTime::new(dt2_micros);

    let diff = datetime_difference(dt2, dt1);
    assert(days_from_duration(diff) == 1);
    assert(duration_is_negative(diff) == false);

    let diff_neg = datetime_difference(dt1, dt2);
    assert(days_from_duration(diff_neg) == 1);
    assert(duration_is_negative(diff_neg) == true);
}

#[test]
fn test_duration_comparison() {
    let d1 = duration_from_components(false, 1, 0, 0, 0, 0);
    let d2 = duration_from_components(false, 2, 0, 0, 0, 0);
    let d3 = duration_from_components(false, 1, 0, 0, 0, 0);

    assert(duration_equal(d1, d3) == true);
    assert(duration_equal(d1, d2) == false);

    assert(duration_less_than(d1, d2) == true);
    assert(duration_less_than(d2, d1) == false);

    assert(duration_greater_than(d2, d1) == true);
    assert(duration_greater_than(d1, d2) == false);
}
