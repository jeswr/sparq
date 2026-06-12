//! Time functions for XPath/SPARQL
//!
//! Implements xs:time operations.
//! Time values are stored as microseconds since midnight plus timezone offset.
//!
//! XPath functions implemented:
//! - fn:hours-from-time, fn:minutes-from-time, fn:seconds-from-time
//! - fn:timezone-from-time
//! - op:time-equal, op:time-less-than, op:time-greater-than

use crate::types::{XsdDayTimeDuration, XsdTime};

// ============================================================================
// Time Constants
// ============================================================================

global MICROS_PER_SECOND: u64 = 1_000_000;
global MICROS_PER_MINUTE: u64 = 60_000_000;
global MICROS_PER_HOUR: u64 = 3_600_000_000;

// ============================================================================
// Time Construction Functions
// ============================================================================

/// Create a time from hour, minute, second components (UTC)
pub fn time_from_components(hours: u8, minutes: u8, seconds: u8) -> XsdTime {
    time_from_components_with_micros(hours, minutes, seconds, 0)
}

/// Create a time from hour, minute, second, microsecond components (UTC)
pub fn time_from_components_with_micros(
    hours: u8,
    minutes: u8,
    seconds: u8,
    microseconds: u32,
) -> XsdTime {
    XsdTime::from_components(hours, minutes, seconds, microseconds)
}

/// Create a time from components with timezone
pub fn time_from_components_with_tz(
    hours: u8,
    minutes: u8,
    seconds: u8,
    microseconds: u32,
    tz_offset_minutes: i16,
) -> XsdTime {
    XsdTime::from_components_with_tz(hours, minutes, seconds, microseconds, tz_offset_minutes)
}

/// Create a time from microseconds since midnight (UTC)
pub fn time_from_microseconds(micros: u64) -> XsdTime {
    XsdTime::new(micros)
}

/// Create a time from microseconds since midnight with timezone
pub fn time_from_microseconds_with_tz(micros: u64, tz_offset_minutes: i16) -> XsdTime {
    XsdTime::new_with_tz(micros, tz_offset_minutes)
}

// ============================================================================
// Time Component Extraction
// ============================================================================

/// fn:hours-from-time - Extract hours component (0-23)
pub fn hours_from_time(time: XsdTime) -> u8 {
    (time.microseconds_since_midnight / MICROS_PER_HOUR) as u8
}

/// fn:minutes-from-time - Extract minutes component (0-59)
pub fn minutes_from_time(time: XsdTime) -> u8 {
    let hour_remainder = time.microseconds_since_midnight % MICROS_PER_HOUR;
    (hour_remainder / MICROS_PER_MINUTE) as u8
}

/// fn:seconds-from-time - Extract seconds component (0-59)
pub fn seconds_from_time(time: XsdTime) -> u8 {
    let minute_remainder = time.microseconds_since_midnight % MICROS_PER_MINUTE;
    (minute_remainder / MICROS_PER_SECOND) as u8
}

/// Extract microseconds within the current second (0-999999)
pub fn microseconds_from_time(time: XsdTime) -> u32 {
    (time.microseconds_since_midnight % MICROS_PER_SECOND) as u32
}

/// fn:timezone-from-time - Extract timezone as a dayTimeDuration
pub fn timezone_from_time(time: XsdTime) -> XsdDayTimeDuration {
    let offset_micros: i64 = (time.tz_offset_minutes as i64) * 60_000_000;
    XsdDayTimeDuration::from_signed_micros(offset_micros)
}

// ============================================================================
// Time Comparison Functions
// ============================================================================

/// op:time-equal - Compare two times for equality (in UTC)
pub fn time_equal(a: XsdTime, b: XsdTime) -> bool {
    a.utc_microseconds() == b.utc_microseconds()
}

/// op:time-less-than - Returns true if a is before b (in UTC)
pub fn time_less_than(a: XsdTime, b: XsdTime) -> bool {
    a.utc_microseconds() < b.utc_microseconds()
}

/// op:time-greater-than - Returns true if a is after b (in UTC)
pub fn time_greater_than(a: XsdTime, b: XsdTime) -> bool {
    a.utc_microseconds() > b.utc_microseconds()
}

/// op:time-less-than-or-equal
pub fn time_le(a: XsdTime, b: XsdTime) -> bool {
    a.utc_microseconds() <= b.utc_microseconds()
}

/// op:time-greater-than-or-equal
pub fn time_ge(a: XsdTime, b: XsdTime) -> bool {
    a.utc_microseconds() >= b.utc_microseconds()
}

// ============================================================================
// Time Arithmetic
// ============================================================================

/// op:subtract-times - Returns the duration between two times
pub fn subtract_times(a: XsdTime, b: XsdTime) -> XsdDayTimeDuration {
    let diff = a.utc_microseconds() - b.utc_microseconds();
    XsdDayTimeDuration::from_signed_micros(diff)
}

/// op:add-dayTimeDuration-to-time - Add a duration to a time
/// Note: Result wraps around within 24 hours
pub fn time_add_duration(time: XsdTime, dur: XsdDayTimeDuration) -> XsdTime {
    let time_micros = time.microseconds_since_midnight as i64;
    let dur_micros = dur.to_signed_micros();
    let day_micros: i64 = 86_400_000_000;

    // Add duration and normalize to 0-24 hour range
    let result = time_micros + dur_micros;
    let normalized = ((result % day_micros) + day_micros) % day_micros;

    XsdTime::new_with_tz(normalized as u64, time.tz_offset_minutes)
}

/// op:subtract-dayTimeDuration-from-time - Subtract a duration from a time
pub fn time_subtract_duration(time: XsdTime, dur: XsdDayTimeDuration) -> XsdTime {
    let time_micros = time.microseconds_since_midnight as i64;
    let dur_micros = dur.to_signed_micros();
    let day_micros: i64 = 86_400_000_000;

    // Subtract duration and normalize to 0-24 hour range
    let result = time_micros - dur_micros;
    let normalized = ((result % day_micros) + day_micros) % day_micros;

    XsdTime::new_with_tz(normalized as u64, time.tz_offset_minutes)
}

// ============================================================================
// Timezone Adjustment Functions
// ============================================================================

/// fn:adjust-time-to-timezone - Adjust a time to a different timezone
/// The instant in time remains the same, only the local representation changes
pub fn adjust_time_to_timezone(time: XsdTime, new_tz_offset_minutes: i16) -> XsdTime {
    // Calculate the UTC microseconds
    let utc_micros = time.utc_microseconds();
    // Create a new time with the new timezone, adjusting for the offset difference
    let offset_diff = new_tz_offset_minutes as i64 - time.tz_offset_minutes as i64;
    let new_local_micros = utc_micros + offset_diff * 60_000_000;
    // Normalize to 0-24 hour range
    let day_micros: i64 = 86_400_000_000;
    let normalized = ((new_local_micros % day_micros) + day_micros) % day_micros;
    XsdTime::new_with_tz(normalized as u64, new_tz_offset_minutes)
}

/// fn:adjust-time-to-timezone with no argument - removes timezone
pub fn adjust_time_to_timezone_none(time: XsdTime) -> XsdTime {
    // Remove timezone but keep the same local time representation
    XsdTime::new(time.microseconds_since_midnight)
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_time_components() {
    let time = time_from_components(14, 30, 45);
    assert(hours_from_time(time) == 14);
    assert(minutes_from_time(time) == 30);
    assert(seconds_from_time(time) == 45);
}

#[test]
fn test_time_comparison() {
    let t1 = time_from_components(10, 0, 0);
    let t2 = time_from_components(11, 0, 0);

    assert(time_less_than(t1, t2) == true);
    assert(time_greater_than(t2, t1) == true);
    assert(time_equal(t1, t1) == true);
}

#[test]
fn test_time_with_timezone() {
    // 10:00 UTC is the same as 15:00 +05:00
    let t1 = time_from_components(10, 0, 0); // UTC
    let t2 = time_from_components_with_tz(15, 0, 0, 0, 300); // +05:00

    assert(time_equal(t1, t2) == true);
}
