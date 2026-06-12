//! Date functions for XPath/SPARQL
//!
//! Implements xs:date operations.
//! Date values are stored as days since Unix epoch (1970-01-01) plus timezone offset.
//!
//! XPath functions implemented:
//! - fn:year-from-date, fn:month-from-date, fn:day-from-date
//! - fn:timezone-from-date
//! - op:date-equal, op:date-less-than, op:date-greater-than

use crate::types::{XsdDate, XsdDayTimeDuration};

// ============================================================================
// Date Construction Functions
// ============================================================================

/// Create a date from year, month, day components (UTC)
pub fn date_from_components(year: i32, month: u8, day: u8) -> XsdDate {
    let epoch_days = ymd_to_epoch_days(year, month, day);
    XsdDate::new(epoch_days)
}

/// Create a date from year, month, day components with timezone
///
/// Note: This function stores the date as the local calendar date (not adjusted for timezone).
/// The epoch_days represents the date in the local timezone, not normalized to UTC.
/// This design choice means dates with different timezones but the same local calendar date
/// will have the same epoch_days value. For XPath date semantics, this is intentional as
/// dates represent calendar dates, not instants in time.
pub fn date_from_components_with_tz(
    year: i32,
    month: u8,
    day: u8,
    tz_offset_minutes: i16,
) -> XsdDate {
    let epoch_days = ymd_to_epoch_days(year, month, day);
    XsdDate::new_with_tz(epoch_days, tz_offset_minutes)
}

/// Create a date from epoch days (UTC)
pub fn date_from_epoch_days(days: i32) -> XsdDate {
    XsdDate::new(days)
}

/// Create a date from epoch days with timezone
pub fn date_from_epoch_days_with_tz(days: i32, tz_offset_minutes: i16) -> XsdDate {
    XsdDate::new_with_tz(days, tz_offset_minutes)
}

// ============================================================================
// Date Component Extraction
// ============================================================================

/// Extract year from date
pub fn year_from_date(date: XsdDate) -> i32 {
    let (year, _, _) = epoch_days_to_ymd(date.epoch_days);
    year
}

/// Extract month from date (1-12)
pub fn month_from_date(date: XsdDate) -> u8 {
    let (_, month, _) = epoch_days_to_ymd(date.epoch_days);
    month
}

/// Extract day from date (1-31)
pub fn day_from_date(date: XsdDate) -> u8 {
    let (_, _, day) = epoch_days_to_ymd(date.epoch_days);
    day
}

/// Extract timezone from date as a dayTimeDuration
pub fn timezone_from_date(date: XsdDate) -> XsdDayTimeDuration {
    let offset_micros: i64 = (date.tz_offset_minutes as i64) * 60_000_000; // 60 * 1_000_000
    XsdDayTimeDuration::from_signed_micros(offset_micros)
}

// ============================================================================
// Date Comparison Functions
// ============================================================================

/// Compare two dates for equality
///
/// This compares the epoch_days directly, which means dates are equal if they have
/// the same calendar date representation (same year/month/day), regardless of timezone.
/// This follows XPath date semantics where dates represent calendar dates, not instants.
pub fn date_equal(a: XsdDate, b: XsdDate) -> bool {
    a.epoch_days == b.epoch_days
}

/// Check if first date is less than second
pub fn date_less_than(a: XsdDate, b: XsdDate) -> bool {
    a.epoch_days < b.epoch_days
}

/// Check if first date is greater than second
pub fn date_greater_than(a: XsdDate, b: XsdDate) -> bool {
    a.epoch_days > b.epoch_days
}

/// Check if first date is less than or equal to second
pub fn date_le(a: XsdDate, b: XsdDate) -> bool {
    a.epoch_days <= b.epoch_days
}

/// Check if first date is greater than or equal to second
pub fn date_ge(a: XsdDate, b: XsdDate) -> bool {
    a.epoch_days >= b.epoch_days
}

// ============================================================================
// Helper Functions for Date Conversion
// ============================================================================

/// Integer floor division for positive divisors.
/// Noir division truncates toward zero; for negative numerators we need floor.
fn floor_div_i64(a: i64, b: i64) -> i64 {
    assert(b > 0, "floor_div_i64 requires positive divisor");
    let q = a / b;
    let r = a % b;
    if (a < 0) & (r != 0) {
        q - 1
    } else {
        q
    }
}

/// Closed-form conversion: civil date -> days since Unix epoch (1970-01-01).
/// Proleptic Gregorian calendar, ISO-8601 compatible (allows year 0).
///
/// Based on Howard Hinnant's `days_from_civil` (public domain).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    // Shift March to be month 0.
    let y = year - if month <= 2 { 1 } else { 0 };
    let era = floor_div_i64(if y >= 0 { y } else { y - 399 }, 400);
    let yoe = y - era * 400; // [0, 399]
    let mp = month + if month > 2 { -3 } else { 9 }; // March=0,...,Feb=11
    let doy = floor_div_i64(153 * mp + 2, 5) + day - 1; // [0, 365]
    let doe = yoe * 365 + floor_div_i64(yoe, 4) - floor_div_i64(yoe, 100) + doy; // [0, 146096]
    era * 146097 + doe - 719468
}

/// Closed-form conversion: days since Unix epoch -> civil date (year, month, day).
/// Proleptic Gregorian calendar, ISO-8601 compatible (allows year 0).
///
/// Based on Howard Hinnant's `civil_from_days` (public domain).
fn civil_from_days(epoch_days: i64) -> (i64, i64, i64) {
    let z = epoch_days + 719468;
    let era = floor_div_i64(if z >= 0 { z } else { z - 146096 }, 146097);
    let doe = z - era * 146097; // [0, 146096]
    let yoe = floor_div_i64(
        doe - floor_div_i64(doe, 1460) + floor_div_i64(doe, 36524) - floor_div_i64(doe, 146096),
        365,
    ); // [0, 399]
    let mut y = yoe + era * 400;
    let doy = doe - (yoe * 365 + floor_div_i64(yoe, 4) - floor_div_i64(yoe, 100)); // [0, 365]
    let mp = floor_div_i64(5 * doy + 2, 153); // [0, 11]
    let d = doy - floor_div_i64(153 * mp + 2, 5) + 1; // [1, 31]
    let m = mp + if mp < 10 { 3 } else { -9 }; // [1, 12]
    if m <= 2 {
        y = y + 1;
    }
    (y, m, d)
}

/// Convert year, month, day to days since Unix epoch (1970-01-01)
fn ymd_to_epoch_days(year: i32, month: u8, day: u8) -> i32 {
    let y: i64 = year as i64;
    let m: i64 = month as i64;
    let d: i64 = day as i64;
    let days: i64 = days_from_civil(y, m, d);
    days as i32
}

/// Convert days since Unix epoch to year, month, day
fn epoch_days_to_ymd(epoch_days: i32) -> (i32, u8, u8) {
    let (y, m, d) = civil_from_days(epoch_days as i64);
    (y as i32, m as u8, d as u8)
}

/// Check if a year is a leap year
fn is_leap_year(year: i32) -> bool {
    // Leap year if divisible by 4, except centuries (unless divisible by 400)
    let div4 = year % 4 == 0;
    let div100 = year % 100 == 0;
    let div400 = year % 400 == 0;

    div4 & (!div100 | div400)
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_date_epoch_roundtrip_basic() {
    // Unix epoch
    let d0 = date_from_epoch_days(0);
    assert(year_from_date(d0) == 1970);
    assert(month_from_date(d0) == 1);
    assert(day_from_date(d0) == 1);

    // Day before epoch
    let d1 = date_from_epoch_days(-1);
    assert(year_from_date(d1) == 1969);
    assert(month_from_date(d1) == 12);
    assert(day_from_date(d1) == 31);
}

#[test]
fn test_date_roundtrip_leap_and_century_rules() {
    // 2000 is a leap year (divisible by 400)
    let d = date_from_components(2000, 2, 29);
    assert(date_from_epoch_days(d.epoch_days).epoch_days == d.epoch_days);
    assert(year_from_date(d) == 2000);
    assert(month_from_date(d) == 2);
    assert(day_from_date(d) == 29);

    // 1900 is NOT a leap year (century not divisible by 400)
    let d2 = date_from_components(1900, 3, 1);
    assert(year_from_date(d2) == 1900);
    assert(month_from_date(d2) == 3);
    assert(day_from_date(d2) == 1);

    // 2100 is NOT a leap year
    let d3 = date_from_components(2100, 3, 1);
    assert(year_from_date(d3) == 2100);
    assert(month_from_date(d3) == 3);
    assert(day_from_date(d3) == 1);
}

#[test]
fn test_date_roundtrip_various_samples() {
    let samples: [(i32, u8, u8); 6] =
        [(1970, 1, 1), (1999, 12, 31), (2000, 3, 1), (1960, 1, 15), (1600, 2, 29), (2400, 2, 29)];

    for i in 0..6 {
        let (y, m, d) = samples[i];
        let date = date_from_components(y, m, d);
        let (ry, rm, rd) = epoch_days_to_ymd(date.epoch_days);
        assert(ry == y);
        assert(rm == m);
        assert(rd == d);
    }
}

// ============================================================================
// Date + Duration Arithmetic
// ============================================================================

/// op:add-dayTimeDuration-to-date
/// Adds a dayTimeDuration to a date, returning a new date
pub fn date_add_duration(date: XsdDate, dur: XsdDayTimeDuration) -> XsdDate {
    let dur_micros = dur.to_signed_micros();
    // Convert microseconds to days (integer division)
    let dur_days: i32 = (dur_micros / 86_400_000_000) as i32;
    let new_days = date.epoch_days + dur_days;
    XsdDate::new_with_tz(new_days, date.tz_offset_minutes)
}

/// op:subtract-dayTimeDuration-from-date
/// Subtracts a dayTimeDuration from a date, returning a new date
pub fn date_subtract_duration(date: XsdDate, dur: XsdDayTimeDuration) -> XsdDate {
    let dur_micros = dur.to_signed_micros();
    // Convert microseconds to days (integer division)
    let dur_days: i32 = (dur_micros / 86_400_000_000) as i32;
    let new_days = date.epoch_days - dur_days;
    XsdDate::new_with_tz(new_days, date.tz_offset_minutes)
}

/// op:subtract-dates
/// Returns the dayTimeDuration between two dates (a - b)
pub fn subtract_dates(a: XsdDate, b: XsdDate) -> XsdDayTimeDuration {
    let day_diff = a.epoch_days - b.epoch_days;
    let micros: i64 = (day_diff as i64) * 86_400_000_000;
    XsdDayTimeDuration::from_signed_micros(micros)
}

// ============================================================================
// Timezone Adjustment Functions
// ============================================================================

/// fn:adjust-date-to-timezone - Adjust a date to a different timezone
/// Since dates represent calendar dates (not instants), changing the timezone
/// may change the date by one day if the timezone offset crosses midnight.
pub fn adjust_date_to_timezone(date: XsdDate, new_tz_offset_minutes: i16) -> XsdDate {
    // For simplicity, we just update the timezone without adjusting the calendar date
    // This is a simplified implementation - full XPath semantics involve more complex rules
    XsdDate::new_with_tz(date.epoch_days, new_tz_offset_minutes)
}

/// fn:adjust-date-to-timezone with no argument - removes timezone
pub fn adjust_date_to_timezone_none(date: XsdDate) -> XsdDate {
    XsdDate::new(date.epoch_days)
}

// ============================================================================
// DateTime Construction from Date and Time
// ============================================================================

/// fn:dateTime - Construct a dateTime from a date and a time
/// XPath: fn:dateTime($date as xs:date, $time as xs:time) as xs:dateTime
/// Both must have the same timezone (or both have no timezone)
pub fn fn_dateTime(date: XsdDate, time: crate::types::XsdTime) -> crate::types::XsdDateTime {
    // Convert date to epoch microseconds (at midnight)
    let date_micros: i64 = (date.epoch_days as i64) * 86_400_000_000;
    // Add time microseconds
    let total_micros: i64 = date_micros + time.microseconds_since_midnight as i64;
    // Convert to unsigned for Field
    let total_micros_u64: u64 = total_micros as u64;
    // Use the timezone from the date (or time if they match)
    crate::types::XsdDateTime::new_with_tz(total_micros_u64 as Field, date.tz_offset_minutes)
}
