//! DateTime functions for XPath/SPARQL
//!
//! Implements component extraction from xsd:dateTime values and comparison.
//! DateTime values are stored as UTC microseconds since epoch plus timezone offset.
//! Component extraction returns local time values (adjusted by timezone).
//! Comparisons use UTC time for correct ordering.
//!
//! SIGNED EPOCH ENCODING (sq-3x7dl.7) [FABLE-5]:
//! The epoch value is a SIGNED i64 count of microseconds since 1970-01-01T00:00:00Z;
//! pre-1970 instants are negative. The `Field` slot on `XsdDateTime` stores the
//! two's-complement 64-bit pattern of that i64 (encode: `(e as u64) as Field`;
//! decode: `f as i64`). Every consumer here decodes to i64 first: ordering is a
//! signed comparison, and calendar extraction uses floor division / Euclidean
//! remainder (shared with `date.nr`) so that e.g. epoch -1 is
//! 1969-12-31T23:59:59.999999Z, not garbage. Range: about +/-292,000 years.
//!
//! XPath functions implemented:
//! - fn:year-from-dateTime, fn:month-from-dateTime, fn:day-from-dateTime
//! - fn:hours-from-dateTime, fn:minutes-from-dateTime, fn:seconds-from-dateTime
//! - op:dateTime-equal, op:dateTime-less-than, op:dateTime-greater-than

use crate::date::{civil_from_days, days_from_civil, floor_div_i64};
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

/// Microseconds per day as SIGNED i64 -- for floor division over the signed
/// epoch range (pre-1970 epochs are negative). [FABLE-5]
global MICROSECONDS_PER_DAY_SIGNED: i64 = 86_400_000_000;

// ============================================================================
// Construction Functions
// ============================================================================

/// Create a DateTime from epoch microseconds (assumes UTC, no timezone offset)
///
/// `micros` is the SIGNED i64 epoch encoded as its two's-complement 64-bit
/// pattern (see the module doc): for a pre-1970 instant pass
/// `(e as u64) as Field` where `e: i64` is negative.
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
    // All arithmetic is SIGNED: pre-1970 civil dates give negative day counts,
    // and Noir u64 arithmetic is overflow-checked, so the old
    // `(days as u64) * MICROSECONDS_PER_DAY` aborted for any date before 1970.
    // [FABLE-5]
    let days: i64 = civil_to_days(year, month, day);

    // Convert time components to microseconds (local time)
    let time_micros: i64 = (hour as i64) * (MICROSECONDS_PER_HOUR as i64)
        + (minute as i64) * (MICROSECONDS_PER_MINUTE as i64)
        + (second as i64) * (MICROSECONDS_PER_SECOND as i64)
        + microsecond as i64;

    // Total microseconds since epoch (in local time), signed
    let local_micros: i64 = days * MICROSECONDS_PER_DAY_SIGNED + time_micros;

    // Convert to UTC by subtracting the timezone offset
    let offset_micros: i64 = (tz_offset_minutes as i64) * 60_000_000;
    let utc_micros: i64 = local_micros - offset_micros;

    // Store the two's-complement bit pattern of the signed epoch (see module doc)
    XsdDateTime::new_with_tz((utc_micros as u64) as Field, tz_offset_minutes)
}

/// Extract epoch microseconds from a DateTime (UTC)
///
/// Returns the two's-complement pattern of the signed i64 epoch; decode a
/// possibly pre-1970 value with `as i64` (see the module doc).
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

/// Microseconds elapsed since local midnight, always in [0, MICROSECONDS_PER_DAY).
///
/// Euclidean remainder over the SIGNED local epoch value: for a negative local
/// epoch (pre-1970), a plain u64 `%` of the two's-complement bit pattern is
/// garbage (e.g. epoch -1 must give 86_399_999_999, i.e. 23:59:59.999999).
/// Computed as `local - floor(local / day) * day`, mirroring date.nr's
/// floor-division idiom. [FABLE-5]
fn time_of_day_micros(dt: XsdDateTime) -> u64 {
    let local: i64 = dt.local_microseconds() as i64;
    let days: i64 = floor_div_i64(local, MICROSECONDS_PER_DAY_SIGNED);
    (local - days * MICROSECONDS_PER_DAY_SIGNED) as u64
}

/// fn:hours-from-dateTime
/// Extracts the hours component (0-23) from a dateTime value (in local time).
pub fn hours_from_datetime(dt: XsdDateTime) -> u8 {
    let day_micros = time_of_day_micros(dt);
    (day_micros / MICROSECONDS_PER_HOUR) as u8
}

/// fn:minutes-from-dateTime
/// Extracts the minutes component (0-59) from a dateTime value (in local time).
pub fn minutes_from_datetime(dt: XsdDateTime) -> u8 {
    let day_micros = time_of_day_micros(dt);
    let hour_micros = day_micros % MICROSECONDS_PER_HOUR;
    (hour_micros / MICROSECONDS_PER_MINUTE) as u8
}

/// fn:seconds-from-dateTime (integer part only)
/// Extracts the seconds component (0-59) from a dateTime value (in local time).
/// Note: XPath returns a decimal including fractional seconds;
/// use microseconds_from_datetime for the fractional part.
pub fn seconds_from_datetime(dt: XsdDateTime) -> u8 {
    let day_micros = time_of_day_micros(dt);
    let hour_micros = day_micros % MICROSECONDS_PER_HOUR;
    let minute_micros = hour_micros % MICROSECONDS_PER_MINUTE;
    (minute_micros / MICROSECONDS_PER_SECOND) as u8
}

/// Extract microseconds within the current second (0-999999)
pub fn microseconds_from_datetime(dt: XsdDateTime) -> u32 {
    let day_micros = time_of_day_micros(dt);
    (day_micros % MICROSECONDS_PER_SECOND) as u32
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
/// SIGNED comparison: a pre-1970 (negative) epoch orders BEFORE any
/// post-1970 epoch; the old u64 comparison ordered it after. [FABLE-5]
pub fn datetime_less_than(a: XsdDateTime, b: XsdDateTime) -> bool {
    let a_micros: i64 = a.epoch_microseconds as i64;
    let b_micros: i64 = b.epoch_microseconds as i64;
    a_micros < b_micros
}

/// op:dateTime-greater-than
/// Returns true if the first dateTime is later than the second.
/// Signed comparison over the i64 epoch (see datetime_less_than). [FABLE-5]
pub fn datetime_greater_than(a: XsdDateTime, b: XsdDateTime) -> bool {
    let a_micros: i64 = a.epoch_microseconds as i64;
    let b_micros: i64 = b.epoch_microseconds as i64;
    a_micros > b_micros
}

/// op:dateTime-less-than-or-equal
/// Returns true if the first dateTime is earlier than or equal to the second.
/// Signed comparison over the i64 epoch (see datetime_less_than). [FABLE-5]
pub fn datetime_le(a: XsdDateTime, b: XsdDateTime) -> bool {
    let a_micros: i64 = a.epoch_microseconds as i64;
    let b_micros: i64 = b.epoch_microseconds as i64;
    a_micros <= b_micros
}

/// op:dateTime-greater-than-or-equal
/// Returns true if the first dateTime is later than or equal to the second.
/// Signed comparison over the i64 epoch (see datetime_less_than). [FABLE-5]
pub fn datetime_ge(a: XsdDateTime, b: XsdDateTime) -> bool {
    let a_micros: i64 = a.epoch_microseconds as i64;
    let b_micros: i64 = b.epoch_microseconds as i64;
    a_micros >= b_micros
}

// ============================================================================
// Internal Calendar Functions
// ============================================================================

/// Convert civil date to days since Unix epoch.
/// Delegates to the shared Howard Hinnant `days_from_civil` in date.nr --
/// one implementation for both xs:date and xs:dateTime (sq-3x7dl.7). [FABLE-5]
fn civil_to_days(year: i32, month: u8, day: u8) -> i64 {
    days_from_civil(year as i64, month as i64, day as i64)
}

/// Convert SIGNED epoch microseconds (two's-complement pattern in the Field,
/// see module doc) to a civil date (year, month, day).
///
/// FLOOR division to days: for a negative epoch, unsigned division of the bit
/// pattern is garbage and truncating i64 division rounds toward zero (epoch -1
/// must land on day -1 = 1969-12-31, not day 0). Delegates to the shared
/// Hinnant `civil_from_days` in date.nr, which is exact over the signed range
/// (its `date_from_epoch_days(-1) -> 1969-12-31` test is the precedent).
/// [FABLE-5]
fn epoch_to_civil(epoch_micros: Field) -> (i32, u8, u8) {
    let micros: i64 = epoch_micros as i64;
    let days: i64 = floor_div_i64(micros, MICROSECONDS_PER_DAY_SIGNED);
    let (y, m, d) = civil_from_days(days);
    (y as i32, m as u8, d as u8)
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
///
/// XPath F&O spec: "returns the xs:dateTime that is the value of $arg without any
/// timezone information."  The LOCAL date-time representation is kept unchanged;
/// only the timezone tag is stripped.
///
/// Representation invariant: epoch_microseconds is UTC, local = UTC + tz_offset.
/// After stripping the timezone we want local_microseconds() to equal the *original*
/// local time.  Since local_microseconds() = epoch + tz*60e6 on the new tz=0 result,
/// we set epoch = original_local = dt.epoch_microseconds + dt.tz_offset_minutes*60e6.
/// [SONNET-4.6]
pub fn adjust_datetime_to_timezone_none(dt: XsdDateTime) -> XsdDateTime {
    // Compute the original local representation (UTC epoch + timezone shift).
    let local_micros: i64 =
        dt.epoch_microseconds as i64 + (dt.tz_offset_minutes as i64) * 60_000_000;
    XsdDateTime::new(local_micros as u64 as Field)
}

// ============================================================================
// Tests -- signed (pre-1970) epoch semantics (sq-3x7dl.7) [FABLE-5]
//
// Every expected value below is quoted from an INDEPENDENT Python oracle
// (proleptic-Gregorian `datetime` + `timedelta` microsecond arithmetic):
//   (dt - datetime(1970,1,1, tzinfo=utc)) // timedelta(microseconds=1)
// ============================================================================

/// Encode a signed i64 epoch as the canonical Field bit pattern (test helper).
fn signed_epoch(e: i64) -> Field {
    (e as u64) as Field
}

#[test]
fn test_datetime_negative_epoch_civil_extraction() {
    // Oracle: epoch_us=-1 -> 1969-12-31T23:59:59.999999+00:00
    let dt = datetime_from_epoch_microseconds(signed_epoch(-1));
    assert(year_from_datetime(dt) == 1969);
    assert(month_from_datetime(dt) == 12);
    assert(day_from_datetime(dt) == 31);
    assert(hours_from_datetime(dt) == 23);
    assert(minutes_from_datetime(dt) == 59);
    assert(seconds_from_datetime(dt) == 59);
    assert(microseconds_from_datetime(dt) == 999999);

    // Oracle: epoch_us=-86400000000 -> 1969-12-31T00:00:00+00:00
    let dt2 = datetime_from_epoch_microseconds(signed_epoch(-86_400_000_000));
    assert(year_from_datetime(dt2) == 1969);
    assert(month_from_datetime(dt2) == 12);
    assert(day_from_datetime(dt2) == 31);
    assert(hours_from_datetime(dt2) == 0);
    assert(minutes_from_datetime(dt2) == 0);
    assert(seconds_from_datetime(dt2) == 0);

    // Oracle: 1965-04-09T03:30:15.250000+00:00 -> epoch_us=-149286584750000
    let dt3 = datetime_from_epoch_microseconds(signed_epoch(-149_286_584_750_000));
    assert(year_from_datetime(dt3) == 1965);
    assert(month_from_datetime(dt3) == 4);
    assert(day_from_datetime(dt3) == 9);
    assert(hours_from_datetime(dt3) == 3);
    assert(minutes_from_datetime(dt3) == 30);
    assert(seconds_from_datetime(dt3) == 15);
    assert(microseconds_from_datetime(dt3) == 250000);
}

#[test]
fn test_datetime_pre1970_ordering_signed() {
    // Oracle: 1969-07-20T20:17:40Z -> epoch_us=-14182940000000;
    //         2020-06-15T12:00:00Z -> epoch_us=1592222400000000
    let dt1969 = datetime_from_epoch_microseconds(signed_epoch(-14_182_940_000_000));
    let dt2020 = datetime_from_epoch_microseconds(signed_epoch(1_592_222_400_000_000));
    // The u64-compared bit pattern of a negative epoch is huge; a signed
    // comparison must still order 1969 BEFORE 2020.
    assert(datetime_less_than(dt1969, dt2020));
    assert(!datetime_less_than(dt2020, dt1969));
    assert(datetime_greater_than(dt2020, dt1969));
    assert(datetime_le(dt1969, dt2020));
    assert(datetime_ge(dt2020, dt1969));

    // -1us orders immediately before the epoch itself
    let just_before = datetime_from_epoch_microseconds(signed_epoch(-1));
    let epoch = datetime_from_epoch_microseconds(0);
    assert(datetime_less_than(just_before, epoch));
    assert(!datetime_less_than(epoch, just_before));

    // Two negative epochs order among themselves:
    // Oracle: 1968-02-29T12:00:00Z=-58017600000000 < 1969-07-20T20:17:40Z=-14182940000000
    let dt1968 = datetime_from_epoch_microseconds(signed_epoch(-58_017_600_000_000));
    assert(datetime_less_than(dt1968, dt1969));
    assert(datetime_ge(dt1969, dt1968));
    assert(datetime_equal(dt1968, dt1968));
    assert(!datetime_equal(dt1968, dt1969));
}

#[test]
fn test_datetime_pre1970_construction_roundtrip() {
    // Oracle: 1969-07-20T20:17:40Z -> epoch_us=-14182940000000
    let dt = datetime_from_components(1969, 7, 20, 20, 17, 40, 0);
    assert((dt.epoch_microseconds as i64) == -14_182_940_000_000);
    assert(year_from_datetime(dt) == 1969);
    assert(month_from_datetime(dt) == 7);
    assert(day_from_datetime(dt) == 20);
    assert(hours_from_datetime(dt) == 20);
    assert(minutes_from_datetime(dt) == 17);
    assert(seconds_from_datetime(dt) == 40);

    // Leap-year boundary. Oracle: 1968-02-29T12:00:00Z -> epoch_us=-58017600000000
    let leap = datetime_from_components(1968, 2, 29, 12, 0, 0, 0);
    assert((leap.epoch_microseconds as i64) == -58_017_600_000_000);
    assert(year_from_datetime(leap) == 1968);
    assert(month_from_datetime(leap) == 2);
    assert(day_from_datetime(leap) == 29);

    // Day after the leap day. Oracle: 1968-03-01T00:00:00Z -> epoch_us=-57974400000000
    let after_leap = datetime_from_components(1968, 3, 1, 0, 0, 0, 0);
    assert((after_leap.epoch_microseconds as i64) == -57_974_400_000_000);

    // Non-leap century. Oracle: 1900-01-01T00:00:00Z -> epoch_us=-2208988800000000
    let y1900 = datetime_from_components(1900, 1, 1, 0, 0, 0, 0);
    assert((y1900.epoch_microseconds as i64) == -2_208_988_800_000_000);
    assert(year_from_datetime(y1900) == 1900);
    assert(month_from_datetime(y1900) == 1);
    assert(day_from_datetime(y1900) == 1);
}

#[test]
fn test_datetime_pre1970_tz_construction() {
    // Oracle: 1969-12-31T19:00:00-05:00 == 1970-01-01T00:00:00Z -> epoch_us=0
    let dt = datetime_from_components_with_tz(1969, 12, 31, 19, 0, 0, 0, -300);
    assert(dt.epoch_microseconds == 0);
    // Local components are the pre-1970 civil values
    assert(year_from_datetime(dt) == 1969);
    assert(month_from_datetime(dt) == 12);
    assert(day_from_datetime(dt) == 31);
    assert(hours_from_datetime(dt) == 19);
}

#[test]
fn test_datetime_year_one_edge() {
    // Oracle: 0001-01-01T00:00:00Z -> epoch_us=-62135596800000000
    // (Python datetime.min, proleptic Gregorian)
    let dt = datetime_from_components(1, 1, 1, 0, 0, 0, 0);
    assert((dt.epoch_microseconds as i64) == -62_135_596_800_000_000);
    assert(year_from_datetime(dt) == 1);
    assert(month_from_datetime(dt) == 1);
    assert(day_from_datetime(dt) == 1);
    assert(hours_from_datetime(dt) == 0);

    let extracted = datetime_from_epoch_microseconds(signed_epoch(-62_135_596_800_000_000));
    assert(year_from_datetime(extracted) == 1);
    assert(month_from_datetime(extracted) == 1);
    assert(day_from_datetime(extracted) == 1);
}

#[test]
fn test_datetime_signed_epoch_civil_epoch_identity() {
    // epoch -> civil -> epoch identity over negative oracle vectors.
    // Oracle epochs: -1 (1969-12-31T23:59:59.999999), -31535999999999
    // (1969-01-01T00:00:00.000001), -58017600000000 (1968-02-29T12:00:00),
    // -2208988800000000 (1900-01-01T00:00:00), -14182940000000
    // (1969-07-20T20:17:40).
    let epochs: [i64; 5] = [
        -1,
        -31_535_999_999_999,
        -58_017_600_000_000,
        -2_208_988_800_000_000,
        -14_182_940_000_000,
    ];
    for i in 0..5 {
        let e = epochs[i];
        let dt = datetime_from_epoch_microseconds(signed_epoch(e));
        let rebuilt = datetime_from_components(
            year_from_datetime(dt),
            month_from_datetime(dt),
            day_from_datetime(dt),
            hours_from_datetime(dt),
            minutes_from_datetime(dt),
            seconds_from_datetime(dt),
            microseconds_from_datetime(dt),
        );
        assert((rebuilt.epoch_microseconds as i64) == e);
    }
}
