//! Additional tests for datetime operations

use dep::xpath::{
    datetime_equal, datetime_from_components, datetime_from_components_with_tz,
    datetime_from_epoch_microseconds, datetime_ge, datetime_greater_than, datetime_le,
    datetime_less_than, datetime_timezone_offset, day_from_datetime, duration_equal,
    duration_from_components, duration_ge, duration_greater_than, duration_le, duration_less_than,
    duration_to_microseconds, hours_from_datetime, microseconds_from_datetime,
    minutes_from_datetime, month_from_datetime, seconds_from_datetime, timezone_from_datetime,
    XsdDateTime, XsdDayTimeDuration, year_from_datetime,
};

// ============================================================================
// Tests moved from xpath/src/types.nr
// ============================================================================

#[test]
fn test_datetime_creation() {
    let dt = XsdDateTime::new(1000000);
    assert(dt.epoch_microseconds == 1000000);
}

#[test]
fn test_datetime_equality() {
    let dt1 = XsdDateTime::new(1000000);
    let dt2 = XsdDateTime::new(1000000);
    let dt3 = XsdDateTime::new(2000000);
    assert(dt1 == dt2);
    assert(dt1 != dt3);
}

#[test]
fn test_duration_creation() {
    let dur = XsdDayTimeDuration::new(3600000000, false);
    assert(dur.microseconds == 3600000000);
    assert(dur.negative == false);
}

#[test]
fn test_duration_zero() {
    let dur = XsdDayTimeDuration::zero();
    assert(dur.microseconds == 0);
    assert(dur.negative == false);
}

// ============================================================================
// Tests moved from xpath/src/datetime.nr
// ============================================================================

#[test]
fn test_datetime_unix_epoch() {
    // Unix epoch: 1970-01-01 00:00:00 UTC
    let dt = datetime_from_epoch_microseconds(0);
    assert(year_from_datetime(dt) == 1970);
    assert(month_from_datetime(dt) == 1);
    assert(day_from_datetime(dt) == 1);
    assert(hours_from_datetime(dt) == 0);
    assert(minutes_from_datetime(dt) == 0);
    assert(seconds_from_datetime(dt) == 0);
}

#[test]
fn test_datetime_components() {
    // 2000-06-15 14:30:45.123456 UTC
    // Microseconds from 1970-01-01 = 961079445123456
    let micros: Field = 961079445123456;
    let dt = datetime_from_epoch_microseconds(micros);

    assert(year_from_datetime(dt) == 2000);
    assert(month_from_datetime(dt) == 6);
    assert(day_from_datetime(dt) == 15);
    assert(hours_from_datetime(dt) == 14);
    assert(minutes_from_datetime(dt) == 30);
    assert(seconds_from_datetime(dt) == 45);
    assert(microseconds_from_datetime(dt) == 123456);
}

#[test]
fn test_datetime_comparison() {
    let dt1 = datetime_from_epoch_microseconds(1000000);
    let dt2 = datetime_from_epoch_microseconds(2000000);
    let dt3 = datetime_from_epoch_microseconds(1000000);

    assert(datetime_equal(dt1, dt3) == true);
    assert(datetime_equal(dt1, dt2) == false);

    assert(datetime_less_than(dt1, dt2) == true);
    assert(datetime_less_than(dt2, dt1) == false);
    assert(datetime_less_than(dt1, dt3) == false);

    assert(datetime_greater_than(dt2, dt1) == true);
    assert(datetime_greater_than(dt1, dt2) == false);
    assert(datetime_greater_than(dt1, dt3) == false);
}

#[test]
fn test_datetime_from_components_roundtrip() {
    // Create datetime and verify round-trip
    let dt = datetime_from_components(2024, 3, 15, 10, 30, 45, 500000);
    assert(year_from_datetime(dt) == 2024);
    assert(month_from_datetime(dt) == 3);
    assert(day_from_datetime(dt) == 15);
    assert(hours_from_datetime(dt) == 10);
    assert(minutes_from_datetime(dt) == 30);
    assert(seconds_from_datetime(dt) == 45);
    assert(microseconds_from_datetime(dt) == 500000);
}

// ============================================================================
// Additional tests
// ============================================================================

#[test]
fn test_datetime_year_boundaries() {
    // Start of year
    let dt1 = datetime_from_components(2024, 1, 1, 0, 0, 0, 0);
    assert(year_from_datetime(dt1) == 2024);
    assert(month_from_datetime(dt1) == 1);
    assert(day_from_datetime(dt1) == 1);

    // End of year
    let dt2 = datetime_from_components(2024, 12, 31, 23, 59, 59, 999999);
    assert(year_from_datetime(dt2) == 2024);
    assert(month_from_datetime(dt2) == 12);
    assert(day_from_datetime(dt2) == 31);
    assert(hours_from_datetime(dt2) == 23);
    assert(minutes_from_datetime(dt2) == 59);
    assert(seconds_from_datetime(dt2) == 59);
    assert(microseconds_from_datetime(dt2) == 999999);
}

#[test]
fn test_datetime_different_years() {
    let dt2000 = datetime_from_components(2000, 6, 15, 12, 0, 0, 0);
    let dt2024 = datetime_from_components(2024, 6, 15, 12, 0, 0, 0);

    assert(datetime_less_than(dt2000, dt2024) == true);
    assert(datetime_greater_than(dt2024, dt2000) == true);
    assert(datetime_equal(dt2000, dt2024) == false);
}

#[test]
fn test_datetime_microsecond_precision() {
    let dt1 = datetime_from_components(2024, 6, 15, 12, 0, 0, 0);
    let dt2 = datetime_from_components(2024, 6, 15, 12, 0, 0, 1);

    // One microsecond difference
    assert(datetime_less_than(dt1, dt2) == true);
    assert(datetime_equal(dt1, dt2) == false);
}

#[test]
fn test_datetime_from_epoch() {
    // 1 second after epoch
    let dt = datetime_from_epoch_microseconds(1_000_000);
    assert(year_from_datetime(dt) == 1970);
    assert(month_from_datetime(dt) == 1);
    assert(day_from_datetime(dt) == 1);
    assert(hours_from_datetime(dt) == 0);
    assert(minutes_from_datetime(dt) == 0);
    assert(seconds_from_datetime(dt) == 1);
}

#[test]
fn test_timezone_from_datetime() {
    // Test UTC timezone (offset 0)
    let dt_utc = datetime_from_components_with_tz(2024, 6, 15, 12, 0, 0, 0, 0);
    assert(datetime_timezone_offset(dt_utc) == 0);
    let tz_utc = timezone_from_datetime(dt_utc);
    assert(duration_to_microseconds(tz_utc) == 0);

    // Test negative timezone offset: -05:00 (EST)
    // -5 hours = -300 minutes
    let dt_est = datetime_from_components_with_tz(2024, 6, 15, 12, 0, 0, 0, -300);
    assert(datetime_timezone_offset(dt_est) == -300);
    let tz_est = timezone_from_datetime(dt_est);
    // -300 minutes = -18,000 seconds = -18,000,000,000 microseconds
    assert(duration_to_microseconds(tz_est) == -18_000_000_000);

    // Test positive timezone offset: +05:30 (IST)
    // +5.5 hours = +330 minutes
    let dt_ist = datetime_from_components_with_tz(2024, 6, 15, 12, 0, 0, 0, 330);
    assert(datetime_timezone_offset(dt_ist) == 330);
    let tz_ist = timezone_from_datetime(dt_ist);
    // +330 minutes = +19,800 seconds = +19,800,000,000 microseconds
    assert(duration_to_microseconds(tz_ist) == 19_800_000_000);

    // Test negative timezone offset: -08:00 (PST)
    // -8 hours = -480 minutes
    let dt_pst = datetime_from_components_with_tz(2024, 6, 15, 12, 0, 0, 0, -480);
    assert(datetime_timezone_offset(dt_pst) == -480);
    let tz_pst = timezone_from_datetime(dt_pst);
    // -480 minutes = -28,800 seconds = -28,800,000,000 microseconds
    assert(duration_to_microseconds(tz_pst) == -28_800_000_000);
}

// ============================================================================
// Tests for new comparison operators (<=, >=)
// ============================================================================

#[test]
fn test_datetime_le() {
    let dt1 = datetime_from_components(2024, 1, 1, 0, 0, 0, 0);
    let dt2 = datetime_from_components(2024, 6, 15, 12, 30, 0, 0);
    let dt3 = datetime_from_components(2024, 1, 1, 0, 0, 0, 0);

    // dt1 < dt2
    assert(datetime_le(dt1, dt2) == true);
    // dt1 == dt3
    assert(datetime_le(dt1, dt3) == true);
    // dt2 > dt1
    assert(datetime_le(dt2, dt1) == false);
}

#[test]
fn test_datetime_ge() {
    let dt1 = datetime_from_components(2024, 1, 1, 0, 0, 0, 0);
    let dt2 = datetime_from_components(2024, 6, 15, 12, 30, 0, 0);
    let dt3 = datetime_from_components(2024, 6, 15, 12, 30, 0, 0);

    // dt2 > dt1
    assert(datetime_ge(dt2, dt1) == true);
    // dt2 == dt3
    assert(datetime_ge(dt2, dt3) == true);
    // dt1 < dt2
    assert(datetime_ge(dt1, dt2) == false);
}

#[test]
fn test_datetime_le_same_instant() {
    let dt = datetime_from_components(2024, 12, 25, 15, 30, 45, 123456);
    assert(datetime_le(dt, dt) == true);
}

#[test]
fn test_datetime_ge_same_instant() {
    let dt = datetime_from_components(2024, 12, 25, 15, 30, 45, 123456);
    assert(datetime_ge(dt, dt) == true);
}

#[test]
fn test_datetime_le_ge_with_microseconds() {
    let dt1 = datetime_from_components(2024, 1, 1, 0, 0, 0, 100);
    let dt2 = datetime_from_components(2024, 1, 1, 0, 0, 0, 200);

    assert(datetime_le(dt1, dt2) == true);
    assert(datetime_le(dt2, dt1) == false);
    assert(datetime_ge(dt2, dt1) == true);
    assert(datetime_ge(dt1, dt2) == false);
}

// ============================================================================
// Tests for duration comparison operators (<=, >=)
// ============================================================================

#[test]
fn test_duration_le() {
    let d1 = duration_from_components(false, 1, 0, 0, 0, 0); // 1 day
    let d2 = duration_from_components(false, 2, 0, 0, 0, 0); // 2 days
    let d3 = duration_from_components(false, 1, 0, 0, 0, 0); // 1 day

    // d1 < d2
    assert(duration_le(d1, d2) == true);
    // d1 == d3
    assert(duration_le(d1, d3) == true);
    // d2 > d1
    assert(duration_le(d2, d1) == false);
}

#[test]
fn test_duration_ge() {
    let d1 = duration_from_components(false, 1, 0, 0, 0, 0); // 1 day
    let d2 = duration_from_components(false, 2, 0, 0, 0, 0); // 2 days
    let d3 = duration_from_components(false, 2, 0, 0, 0, 0); // 2 days

    // d2 > d1
    assert(duration_ge(d2, d1) == true);
    // d2 == d3
    assert(duration_ge(d2, d3) == true);
    // d1 < d2
    assert(duration_ge(d1, d2) == false);
}

#[test]
fn test_duration_le_negative() {
    let d1 = duration_from_components(true, 2, 0, 0, 0, 0); // -2 days
    let d2 = duration_from_components(true, 1, 0, 0, 0, 0); // -1 day
    let d3 = duration_from_components(false, 1, 0, 0, 0, 0); // +1 day

    // -2 < -1
    assert(duration_le(d1, d2) == true);
    // -1 < +1
    assert(duration_le(d2, d3) == true);
    // -2 == -2
    assert(duration_le(d1, d1) == true);
}

#[test]
fn test_duration_ge_negative() {
    let d1 = duration_from_components(true, 2, 0, 0, 0, 0); // -2 days
    let d2 = duration_from_components(true, 1, 0, 0, 0, 0); // -1 day
    let d3 = duration_from_components(false, 1, 0, 0, 0, 0); // +1 day

    // -1 > -2
    assert(duration_ge(d2, d1) == true);
    // +1 > -1
    assert(duration_ge(d3, d2) == true);
    // -2 == -2
    assert(duration_ge(d1, d1) == true);
}
