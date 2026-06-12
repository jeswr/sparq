//! Unit tests for XSD time functions

use dep::xpath::{
    hours_from_time, microseconds_from_time, minutes_from_time, seconds_from_time, subtract_times,
    time_add_duration, time_equal, time_from_components, time_from_components_with_tz,
    time_from_microseconds, time_ge, time_greater_than, time_le, time_less_than,
    time_subtract_duration, XsdDayTimeDuration,
};

// ============================================================================
// Time Construction Tests
// ============================================================================

#[test]
fn test_time_from_components_midnight() {
    let time = time_from_components(0, 0, 0);
    assert(hours_from_time(time) == 0);
    assert(minutes_from_time(time) == 0);
    assert(seconds_from_time(time) == 0);
}

#[test]
fn test_time_from_components_noon() {
    let time = time_from_components(12, 0, 0);
    assert(hours_from_time(time) == 12);
    assert(minutes_from_time(time) == 0);
    assert(seconds_from_time(time) == 0);
}

#[test]
fn test_time_from_components_end_of_day() {
    let time = time_from_components(23, 59, 59);
    assert(hours_from_time(time) == 23);
    assert(minutes_from_time(time) == 59);
    assert(seconds_from_time(time) == 59);
}

#[test]
fn test_time_from_components_with_timezone() {
    // 10:30:00 with +5:30 timezone (India)
    let time = time_from_components_with_tz(10, 30, 0, 0, 330);
    assert(hours_from_time(time) == 10);
    assert(minutes_from_time(time) == 30);
}

#[test]
fn test_time_from_microseconds() {
    // 1 hour = 3,600,000,000 microseconds
    let time = time_from_microseconds(3_600_000_000);
    assert(hours_from_time(time) == 1);
    assert(minutes_from_time(time) == 0);
    assert(seconds_from_time(time) == 0);
}

// ============================================================================
// Time Component Extraction Tests
// ============================================================================

#[test]
fn test_hours_from_time() {
    let time = time_from_components(14, 30, 45);
    assert(hours_from_time(time) == 14);
}

#[test]
fn test_minutes_from_time() {
    let time = time_from_components(14, 30, 45);
    assert(minutes_from_time(time) == 30);
}

#[test]
fn test_seconds_from_time() {
    let time = time_from_components(14, 30, 45);
    assert(seconds_from_time(time) == 45);
}

#[test]
fn test_microseconds_from_time() {
    // Create time with 500000 microseconds
    let time = time_from_components_with_tz(14, 30, 45, 500000, 0);
    assert(microseconds_from_time(time) == 500000);
}

// ============================================================================
// Time Comparison Tests
// ============================================================================

#[test]
fn test_time_equal_same_time() {
    let t1 = time_from_components(10, 30, 0);
    let t2 = time_from_components(10, 30, 0);
    assert(time_equal(t1, t2));
}

#[test]
fn test_time_equal_different_times() {
    let t1 = time_from_components(10, 30, 0);
    let t2 = time_from_components(10, 31, 0);
    assert(!time_equal(t1, t2));
}

#[test]
fn test_time_equal_with_timezone_normalization() {
    // 10:00 UTC == 15:00 +05:00
    let t1 = time_from_components(10, 0, 0);
    let t2 = time_from_components_with_tz(15, 0, 0, 0, 300);
    assert(time_equal(t1, t2));
}

#[test]
fn test_time_less_than() {
    let t1 = time_from_components(9, 0, 0);
    let t2 = time_from_components(10, 0, 0);
    assert(time_less_than(t1, t2));
    assert(!time_less_than(t2, t1));
    assert(!time_less_than(t1, t1));
}

#[test]
fn test_time_greater_than() {
    let t1 = time_from_components(10, 0, 0);
    let t2 = time_from_components(9, 0, 0);
    assert(time_greater_than(t1, t2));
    assert(!time_greater_than(t2, t1));
    assert(!time_greater_than(t1, t1));
}

#[test]
fn test_time_le() {
    let t1 = time_from_components(9, 0, 0);
    let t2 = time_from_components(10, 0, 0);
    assert(time_le(t1, t2));
    assert(time_le(t1, t1));
    assert(!time_le(t2, t1));
}

#[test]
fn test_time_ge() {
    let t1 = time_from_components(10, 0, 0);
    let t2 = time_from_components(9, 0, 0);
    assert(time_ge(t1, t2));
    assert(time_ge(t1, t1));
    assert(!time_ge(t2, t1));
}

// ============================================================================
// Time Arithmetic Tests
// ============================================================================

#[test]
fn test_time_add_duration_simple() {
    let time = time_from_components(10, 0, 0);
    let dur = XsdDayTimeDuration::from_signed_micros(3_600_000_000); // +1 hour
    let result = time_add_duration(time, dur);
    assert(hours_from_time(result) == 11);
}

#[test]
fn test_time_subtract_duration_simple() {
    let time = time_from_components(10, 0, 0);
    let dur = XsdDayTimeDuration::from_signed_micros(3_600_000_000); // 1 hour
    let result = time_subtract_duration(time, dur);
    assert(hours_from_time(result) == 9);
}

#[test]
fn test_subtract_times() {
    let t1 = time_from_components(12, 0, 0);
    let t2 = time_from_components(10, 0, 0);
    let diff = subtract_times(t1, t2);
    // 2 hours = 7,200,000,000 microseconds
    assert(diff.to_signed_micros() == 7_200_000_000);
}
