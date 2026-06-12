//! DateTime functions for XPath/SPARQL
//!
//! Implements component extraction from xsd:dateTime values and comparison.
//! DateTime values are stored as UTC microseconds since epoch plus timezone offset.
//! Component extraction returns local time values (adjusted by timezone).
//! Comparisons use UTC time for correct ordering.
//!
//! XPath functions implemented:
//! - fn:year-from-dateTime, fn:month-from-dateTime, fn:day-from-dateTime
//! - fn:hours-from-dateTime, fn:minutes-from-dateTime, fn:seconds-from-dateTime
//! - op:dateTime-equal, op:dateTime-less-than, op:dateTime-greater-than

use crate::duration::duration_from_microseconds;
use crate::types::XsdDateTime;

// ============================================================================
// Time Constants (as u64 for arithmetic)
// ============================================================================

/// Microseconds per second
global MICROSECONDS_PER_SECOND: u64 = 1_000_000;

/// Microseconds per minute
global MICROSECONDS_PER_MINUTE: u64 = 60_000_000;

/// Microseconds per hour
global MICROSECONDS_PER_HOUR: u64 = 3_600_000_000;

/// Microseconds per day
global MICROSECONDS_PER_DAY: u64 = 86_400_000_000;

// ============================================================================
// Construction Functions
// ============================================================================

/// Create a DateTime from epoch microseconds (assumes UTC, no timezone offset)
pub fn datetime_from_epoch_microseconds(micros: Field) -> XsdDateTime {
    XsdDateTime::new(micros)
}

/// Create a DateTime from epoch microseconds with timezone offset
/// micros: UTC epoch microseconds
/// tz_offset_minutes: timezone offset in minutes (e.g., -300 for -05:00, +330 for +05:30)
pub fn datetime_from_epoch_microseconds_with_tz(
    micros: Field,
    tz_offset_minutes: i16,
) -> XsdDateTime {
    XsdDateTime::new_with_tz(micros, tz_offset_minutes)
}

/// Create a DateTime from individual components (UTC)
/// year: The year (e.g., 2024)
/// month: The month (1-12)
/// day: The day of month (1-31)
/// hour: The hour (0-23)
/// minute: The minute (0-59)
/// second: The second (0-59)
/// microsecond: Microseconds within the second (0-999999)
pub fn datetime_from_components(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    microsecond: u32,
) -> XsdDateTime {
    datetime_from_components_with_tz(year, month, day, hour, minute, second, microsecond, 0)
}

/// Create a DateTime from individual components with timezone offset
/// The components represent LOCAL time at the given timezone.
/// year: The year (e.g., 2024)
/// month: The month (1-12)
/// day: The day of month (1-31)
/// hour: The hour (0-23)
/// minute: The minute (0-59)
/// second: The second (0-59)
/// microsecond: Microseconds within the second (0-999999)
/// tz_offset_minutes: timezone offset in minutes (e.g., -300 for -05:00)
pub fn datetime_from_components_with_tz(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    microsecond: u32,
    tz_offset_minutes: i16,
) -> XsdDateTime {
    let days = civil_to_days(year, month, day);

    // Convert time components to microseconds (local time)
    let time_micros: u64 = (hour as u64) * MICROSECONDS_PER_HOUR
        + (minute as u64) * MICROSECONDS_PER_MINUTE
        + (second as u64) * MICROSECONDS_PER_SECOND
        + microsecond as u64;

    // Total microseconds since epoch (in local time)
    let local_micros: u64 = (days as u64) * MICROSECONDS_PER_DAY + time_micros;

    // Convert to UTC by subtracting the timezone offset
    let offset_micros: i64 = (tz_offset_minutes as i64) * 60_000_000;
    let utc_micros: i64 = (local_micros as i64) - offset_micros;
    let utc_u64: u64 = utc_micros as u64;

    XsdDateTime::new_with_tz(utc_u64 as Field, tz_offset_minutes)
}

/// Extract epoch microseconds from a DateTime (UTC)
pub fn datetime_to_epoch_microseconds(dt: XsdDateTime) -> Field {
    dt.epoch_microseconds
}

/// Extract timezone offset in minutes
pub fn datetime_timezone_offset(dt: XsdDateTime) -> i16 {
    dt.tz_offset_minutes
}

/// SPARQL TIMEZONE function - returns timezone as an xsd:dayTimeDuration
/// Maps to fn:timezone-from-dateTime in XPath
/// Returns the timezone offset as a duration (e.g., -PT5H for -05:00)
pub fn timezone_from_datetime(dt: XsdDateTime) -> crate::types::XsdDayTimeDuration {
    let offset_minutes = dt.tz_offset_minutes as i64;
    let offset_micros = offset_minutes * 60_000_000; // Convert minutes to microseconds
    duration_from_microseconds(offset_micros)
}

// ============================================================================
// Component Extraction Functions (return LOCAL time components)
// ============================================================================

/// fn:year-from-dateTime
/// Extracts the year component from a dateTime value (in local time).
/// Uses Howard Hinnant's civil date algorithms.
pub fn year_from_datetime(dt: XsdDateTime) -> i32 {
    let local_micros = dt.local_microseconds();
    let (year, _, _) = epoch_to_civil(local_micros);
    year
}

/// fn:month-from-dateTime
/// Extracts the month component (1-12) from a dateTime value (in local time).
pub fn month_from_datetime(dt: XsdDateTime) -> u8 {
    let local_micros = dt.local_microseconds();
    let (_, month, _) = epoch_to_civil(local_micros);
    month
}

/// fn:day-from-dateTime
/// Extracts the day component (1-31) from a dateTime value (in local time).
pub fn day_from_datetime(dt: XsdDateTime) -> u8 {
    let local_micros = dt.local_microseconds();
    let (_, _, day) = epoch_to_civil(local_micros);
    day
}

/// fn:hours-from-dateTime
/// Extracts the hours component (0-23) from a dateTime value (in local time).
pub fn hours_from_datetime(dt: XsdDateTime) -> u8 {
    let micros: u64 = dt.local_microseconds() as u64;
    let day_micros = micros % MICROSECONDS_PER_DAY;
    (day_micros / MICROSECONDS_PER_HOUR) as u8
}

/// fn:minutes-from-dateTime
/// Extracts the minutes component (0-59) from a dateTime value (in local time).
pub fn minutes_from_datetime(dt: XsdDateTime) -> u8 {
    let micros: u64 = dt.local_microseconds() as u64;
    let day_micros = micros % MICROSECONDS_PER_DAY;
    let hour_micros = day_micros % MICROSECONDS_PER_HOUR;
    (hour_micros / MICROSECONDS_PER_MINUTE) as u8
}

/// fn:seconds-from-dateTime (integer part only)
/// Extracts the seconds component (0-59) from a dateTime value (in local time).
/// Note: XPath returns a decimal including fractional seconds;
/// use microseconds_from_datetime for the fractional part.
pub fn seconds_from_datetime(dt: XsdDateTime) -> u8 {
    let micros: u64 = dt.local_microseconds() as u64;
    let day_micros = micros % MICROSECONDS_PER_DAY;
    let hour_micros = day_micros % MICROSECONDS_PER_HOUR;
    let minute_micros = hour_micros % MICROSECONDS_PER_MINUTE;
    (minute_micros / MICROSECONDS_PER_SECOND) as u8
}

/// Extract microseconds within the current second (0-999999)
pub fn microseconds_from_datetime(dt: XsdDateTime) -> u32 {
    let micros: u64 = dt.local_microseconds() as u64;
    (micros % MICROSECONDS_PER_SECOND) as u32
}

// ============================================================================
// Comparison Functions
// ============================================================================

/// op:dateTime-equal
/// Returns true if both dateTimes represent the same instant.
pub fn datetime_equal(a: XsdDateTime, b: XsdDateTime) -> bool {
    a.epoch_microseconds == b.epoch_microseconds
}

/// op:dateTime-less-than
/// Returns true if the first dateTime is earlier than the second.
pub fn datetime_less_than(a: XsdDateTime, b: XsdDateTime) -> bool {
    let a_micros: u64 = a.epoch_microseconds as u64;
    let b_micros: u64 = b.epoch_microseconds as u64;
    a_micros < b_micros
}

/// op:dateTime-greater-than
/// Returns true if the first dateTime is later than the second.
pub fn datetime_greater_than(a: XsdDateTime, b: XsdDateTime) -> bool {
    let a_micros: u64 = a.epoch_microseconds as u64;
    let b_micros: u64 = b.epoch_microseconds as u64;
    a_micros > b_micros
}

/// op:dateTime-less-than-or-equal
/// Returns true if the first dateTime is earlier than or equal to the second.
pub fn datetime_le(a: XsdDateTime, b: XsdDateTime) -> bool {
    let a_micros: u64 = a.epoch_microseconds as u64;
    let b_micros: u64 = b.epoch_microseconds as u64;
    a_micros <= b_micros
}

/// op:dateTime-greater-than-or-equal
/// Returns true if the first dateTime is later than or equal to the second.
pub fn datetime_ge(a: XsdDateTime, b: XsdDateTime) -> bool {
    let a_micros: u64 = a.epoch_microseconds as u64;
    let b_micros: u64 = b.epoch_microseconds as u64;
    a_micros >= b_micros
}

// ============================================================================
// Internal Calendar Functions
// ============================================================================

/// Convert civil date to days since Unix epoch
/// Based on Howard Hinnant's algorithms: http://howardhinnant.github.io/date_algorithms.html
fn civil_to_days(year: i32, month: u8, day: u8) -> i64 {
    let y: i64 = if month <= 2 {
        year as i64 - 1
    } else {
        year as i64
    };
    let m: i64 = month as i64;
    let d: i64 = day as i64;

    // Calculate era (400-year period)
    let era: i64 = if y >= 0 { y / 400 } else { (y - 399) / 400 };

    // Year of era [0, 399]
    let yoe: i64 = y - era * 400;

    // Day of year calculation
    // For months Mar-Dec, use m-3; for Jan-Feb, use m+9
    let adj_m: i64 = if m > 2 { m - 3 } else { m + 9 };
    let doy: i64 = (153 * adj_m + 2) / 5 + d - 1;

    // Day of era
    let doe: i64 = yoe * 365 + yoe / 4 - yoe / 100 + doy;

    // Total days from epoch
    era * 146097 + doe - 719468
}

/// Convert epoch microseconds to civil date (year, month, day)
/// Based on Howard Hinnant's algorithms: http://howardhinnant.github.io/date_algorithms.html
fn epoch_to_civil(epoch_micros: Field) -> (i32, u8, u8) {
    // Convert to days since epoch
    let micros: u64 = epoch_micros as u64;
    let days: i64 = (micros / MICROSECONDS_PER_DAY) as i64;

    // Shift epoch from 1970-01-01 to 0000-03-01 for easier leap year handling
    // days from 1970-01-01 to 0000-03-01 is -719468
    let z: i64 = days + 719468;

    // Calculate era (400-year period)
    let era: i64 = if z >= 0 {
        z / 146097
    } else {
        (z - 146096) / 146097
    };

    // Day of era [0, 146096]
    let doe: i64 = z - era * 146097;

    // Year of era [0, 399]
    let yoe: i64 = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;

    // Year
    let y: i64 = yoe + era * 400;

    // Day of year [0, 365]
    let doy: i64 = doe - (365 * yoe + yoe / 4 - yoe / 100);

    // Month and day calculation
    let mp: i64 = (5 * doy + 2) / 153;
    let d: i64 = doy - (153 * mp + 2) / 5 + 1;
    let m: i64 = if mp < 10 { mp + 3 } else { mp - 9 };

    // Adjust year if month is Jan or Feb
    let year: i64 = if m <= 2 { y + 1 } else { y };

    (year as i32, m as u8, d as u8)
}

// ============================================================================
// Timezone Adjustment Functions
// ============================================================================

/// fn:adjust-dateTime-to-timezone - Adjust a dateTime to a different timezone
/// If new_tz_offset is None (represented as i16::MIN), removes timezone info
/// The instant in time remains the same, only the local representation changes
pub fn adjust_datetime_to_timezone(dt: XsdDateTime, new_tz_offset_minutes: i16) -> XsdDateTime {
    // The UTC instant stays the same, only the timezone changes
    XsdDateTime::new_with_tz(dt.epoch_microseconds, new_tz_offset_minutes)
}

/// fn:adjust-dateTime-to-timezone with no argument - removes timezone
pub fn adjust_datetime_to_timezone_none(dt: XsdDateTime) -> XsdDateTime {
    // Remove timezone but keep the same instant
    XsdDateTime::new(dt.epoch_microseconds)
}
