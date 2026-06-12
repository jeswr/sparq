//! YearMonthDuration functions for XPath/SPARQL
//!
//! Implements xsd:yearMonthDuration operations.
//! Duration values are stored as total months (signed integer).
//!
//! XPath operators implemented:
//! - op:add-yearMonthDurations, op:subtract-yearMonthDurations
//! - op:multiply-yearMonthDuration, op:divide-yearMonthDuration
//! - op:divide-yearMonthDuration-by-yearMonthDuration
//! - op:add-yearMonthDuration-to-dateTime, op:subtract-yearMonthDuration-from-dateTime
//! - op:add-yearMonthDuration-to-date, op:subtract-yearMonthDuration-from-date
//! - op:yearMonthDuration-equal, op:yearMonthDuration-less-than, op:yearMonthDuration-greater-than
//! XPath functions implemented:
//! - fn:years-from-duration, fn:months-from-duration (for yearMonthDuration)

use crate::date::{date_from_components_with_tz, day_from_date, month_from_date, year_from_date};
use crate::datetime::{
    datetime_from_components_with_tz, day_from_datetime, hours_from_datetime,
    microseconds_from_datetime, minutes_from_datetime, month_from_datetime, seconds_from_datetime,
    year_from_datetime,
};
use crate::types::{XsdDate, XsdDateTime};

// ============================================================================
// YearMonthDuration Type
// ============================================================================

/// XSD yearMonthDuration representation
/// Stores duration as total months (positive or negative)
pub struct XsdYearMonthDuration {
    /// Total months (can be negative)
    pub total_months: i32,
}

impl XsdYearMonthDuration {
    /// Create a new YearMonthDuration from total months
    pub fn new(total_months: i32) -> Self {
        XsdYearMonthDuration { total_months }
    }

    /// Create from years and months components
    pub fn from_components(negative: bool, years: u32, months: u32) -> Self {
        let total = (years as i32) * 12 + (months as i32);
        let signed_total = if negative { -total } else { total };
        XsdYearMonthDuration { total_months: signed_total }
    }

    /// Create a zero duration
    pub fn zero() -> Self {
        XsdYearMonthDuration { total_months: 0 }
    }

    /// Check if the duration is negative
    pub fn is_negative(self) -> bool {
        self.total_months < 0
    }

    /// Get absolute value of total months
    pub fn abs_months(self) -> u32 {
        if self.total_months < 0 {
            (-self.total_months) as u32
        } else {
            self.total_months as u32
        }
    }
}

impl Eq for XsdYearMonthDuration {
    fn eq(self, other: Self) -> bool {
        self.total_months == other.total_months
    }
}

// ============================================================================
// String Conversion Functions
// ============================================================================

/// fn:string for yearMonthDuration - Converts to lexical representation
/// XPath: fn:string($arg as xs:yearMonthDuration) as xs:string
/// Format: [-]P[nY][nM] (e.g., "P1Y2M", "-P6M", "P0M")
/// Returns (bytes, length) where bytes contains ASCII representation
pub fn fn_string_from_ym_duration<let N: u32>(dur: XsdYearMonthDuration) -> ([u8; N], u32) {
    let mut result: [u8; N] = [0; N];
    let mut pos: u32 = 0;

    let negative = dur.total_months < 0;
    let abs_months = dur.abs_months();
    let years = abs_months / 12;
    let months = abs_months % 12;

    // Add '-' if negative
    if negative & (pos < N) {
        result[pos] = 0x2D; // '-'
        pos += 1;
    }

    // Add 'P'
    if pos < N {
        result[pos] = 0x50; // 'P'
        pos += 1;
    }

    // Helper to write an unsigned integer
    // Returns the number of digits written
    let write_uint = |buf: &mut [u8; N], start: u32, val: u32| -> u32 {
        if val == 0 {
            if start < N {
                buf[start] = 0x30; // '0'
            }
            1
        } else {
            // Count digits
            let mut temp_val = val;
            let mut digit_count: u32 = 0;
            for _ in 0..10 {
                if temp_val > 0 {
                    digit_count += 1;
                    temp_val = temp_val / 10;
                }
            }

            // Write digits in reverse
            let mut write_pos = start + digit_count;
            let mut v = val;
            for _ in 0..10 {
                if v > 0 {
                    write_pos -= 1;
                    if write_pos < N {
                        buf[write_pos] = 0x30 + (v % 10) as u8; // '0' + digit
                    }
                    v = v / 10;
                }
            }
            digit_count
        }
    };

    // Add years component if non-zero
    if years > 0 {
        let digits = write_uint(&mut result, pos, years);
        pos += digits;
        if pos < N {
            result[pos] = 0x59; // 'Y'
            pos += 1;
        }
    }

    // Add months component (always include if years is 0, to ensure at least "P0M")
    if (months > 0) | (years == 0) {
        let digits = write_uint(&mut result, pos, months);
        pos += digits;
        if pos < N {
            result[pos] = 0x4D; // 'M'
            pos += 1;
        }
    }

    (result, pos)
}

/// Parses xs:yearMonthDuration from lexical representation
/// XPath: xs:yearMonthDuration($arg as xs:string) as xs:yearMonthDuration
/// Format: [-]P[nY][nM] (e.g., "P1Y2M", "-P6M", "P0M")
/// Returns (duration, valid) where valid indicates successful parsing
pub fn ym_duration_from_string<let N: u32>(
    bytes: [u8; N],
    len: u32,
) -> (XsdYearMonthDuration, bool) {
    let mut pos: u32 = 0;
    let mut negative = false;
    let mut valid = len > 0;

    // Check for optional leading '-'
    if valid & (pos < len) & (pos < N) {
        if bytes[pos] == 0x2D {
            // '-'
            negative = true;
            pos += 1;
        }
    }

    // Expect 'P'
    if valid & (pos < len) & (pos < N) {
        if bytes[pos] != 0x50 {
            // 'P'
            valid = false;
        }
        pos += 1;
    } else if valid {
        valid = false;
    }

    let mut years: u32 = 0;
    let mut months: u32 = 0;
    let mut has_years = false;
    let mut has_months = false;

    // Parse number followed by 'Y' or 'M'
    // First, try to parse a number
    let mut num: u32 = 0;
    let mut has_num = false;

    for _ in 0..N {
        if (pos < len) & (pos < N) & valid {
            let c = bytes[pos];

            // Check if digit
            if (c >= 0x30) & (c <= 0x39) {
                num = num * 10 + (c - 0x30) as u32;
                has_num = true;
                pos += 1;
            } else if c == 0x59 {
                // 'Y'
                if has_num {
                    years = num;
                    has_years = true;
                    num = 0;
                    has_num = false;
                    pos += 1;
                } else {
                    valid = false;
                }
            } else if c == 0x4D {
                // 'M'
                if has_num {
                    months = num;
                    has_months = true;
                    num = 0;
                    has_num = false;
                    pos += 1;
                } else {
                    valid = false;
                }
            } else {
                // Invalid character
                valid = false;
            }
        }
    }

    // Must have consumed all input and have at least Y or M
    if pos != len {
        valid = false;
    }
    if !has_years & !has_months {
        valid = false;
    }

    // Build the result
    let total_months = (years * 12 + months) as i32;
    let signed_total = if negative {
        -total_months
    } else {
        total_months
    };

    (XsdYearMonthDuration::new(signed_total), valid)
}

// ============================================================================
// Component Extraction Functions
// ============================================================================

/// fn:years-from-duration (for yearMonthDuration)
/// Returns the years component (signed)
pub fn years_from_duration(dur: XsdYearMonthDuration) -> i32 {
    dur.total_months / 12
}

/// fn:months-from-duration (for yearMonthDuration)
/// Returns the months component (0-11, signed based on duration sign)
pub fn months_from_duration(dur: XsdYearMonthDuration) -> i32 {
    dur.total_months % 12
}

// ============================================================================
// YearMonthDuration Arithmetic
// ============================================================================

/// op:add-yearMonthDurations
/// Adds two yearMonthDurations
pub fn ym_duration_add(a: XsdYearMonthDuration, b: XsdYearMonthDuration) -> XsdYearMonthDuration {
    XsdYearMonthDuration::new(a.total_months + b.total_months)
}

/// op:subtract-yearMonthDurations
/// Subtracts second yearMonthDuration from first
pub fn ym_duration_subtract(
    a: XsdYearMonthDuration,
    b: XsdYearMonthDuration,
) -> XsdYearMonthDuration {
    XsdYearMonthDuration::new(a.total_months - b.total_months)
}

/// op:multiply-yearMonthDuration
/// Multiplies a yearMonthDuration by an integer
pub fn ym_duration_multiply(dur: XsdYearMonthDuration, factor: i32) -> XsdYearMonthDuration {
    XsdYearMonthDuration::new(dur.total_months * factor)
}

/// op:divide-yearMonthDuration
/// Divides a yearMonthDuration by an integer
pub fn ym_duration_divide(dur: XsdYearMonthDuration, divisor: i32) -> XsdYearMonthDuration {
    assert(divisor != 0, "Division by zero");
    XsdYearMonthDuration::new(dur.total_months / divisor)
}

/// op:divide-yearMonthDuration-by-yearMonthDuration
/// Divides two yearMonthDurations, returning a ratio
pub fn ym_duration_divide_by_duration(a: XsdYearMonthDuration, b: XsdYearMonthDuration) -> i32 {
    assert(b.total_months != 0, "Division by zero duration");
    a.total_months / b.total_months
}

/// Negates a yearMonthDuration
pub fn ym_duration_negate(dur: XsdYearMonthDuration) -> XsdYearMonthDuration {
    XsdYearMonthDuration::new(-dur.total_months)
}

// ============================================================================
// YearMonthDuration Comparison
// ============================================================================

/// op:yearMonthDuration-equal
pub fn ym_duration_equal(a: XsdYearMonthDuration, b: XsdYearMonthDuration) -> bool {
    a.total_months == b.total_months
}

/// op:yearMonthDuration-less-than
pub fn ym_duration_less_than(a: XsdYearMonthDuration, b: XsdYearMonthDuration) -> bool {
    a.total_months < b.total_months
}

/// op:yearMonthDuration-greater-than
pub fn ym_duration_greater_than(a: XsdYearMonthDuration, b: XsdYearMonthDuration) -> bool {
    a.total_months > b.total_months
}

/// op:yearMonthDuration-less-than-or-equal
pub fn ym_duration_le(a: XsdYearMonthDuration, b: XsdYearMonthDuration) -> bool {
    a.total_months <= b.total_months
}

/// op:yearMonthDuration-greater-than-or-equal
pub fn ym_duration_ge(a: XsdYearMonthDuration, b: XsdYearMonthDuration) -> bool {
    a.total_months >= b.total_months
}

// ============================================================================
// DateTime + YearMonthDuration Arithmetic
// ============================================================================

/// Helper: Get days in month for a given year and month (1-based)
fn days_in_month(year: i32, month: u8) -> u8 {
    // Determine if leap year
    let is_leap = ((year % 4 == 0) & (year % 100 != 0)) | (year % 400 == 0);

    if month == 2 {
        if is_leap {
            29
        } else {
            28
        }
    } else if (month == 4) | (month == 6) | (month == 9) | (month == 11) {
        30
    } else {
        31
    }
}

/// Helper: Clamp day to valid range for the month
fn clamp_day(year: i32, month: u8, day: u8) -> u8 {
    let max_day = days_in_month(year, month);
    if day > max_day {
        max_day
    } else {
        day
    }
}

/// op:add-yearMonthDuration-to-dateTime
/// Adds a yearMonthDuration to a dateTime
pub fn datetime_add_ym_duration(dt: XsdDateTime, dur: XsdYearMonthDuration) -> XsdDateTime {
    // Extract components
    let year = year_from_datetime(dt);
    let month = month_from_datetime(dt);
    let day = day_from_datetime(dt);
    let hours = hours_from_datetime(dt);
    let minutes = minutes_from_datetime(dt);
    let seconds = seconds_from_datetime(dt);
    let microseconds = microseconds_from_datetime(dt);
    let tz = dt.tz_offset_minutes; // Use raw field for tz offset

    // Calculate new year and month
    let total_months = (year as i32) * 12 + ((month as i32) - 1) + dur.total_months;

    // Handle negative month calculations
    let new_year: i32 = if total_months >= 0 {
        total_months / 12
    } else {
        // For negative, we need floor division
        (total_months - 11) / 12
    };

    let month_rem = total_months - (new_year * 12);
    let new_month = (month_rem + 1) as u8;

    // Clamp day to valid range for new month
    let new_day = clamp_day(new_year, new_month, day);

    datetime_from_components_with_tz(
        new_year,
        new_month,
        new_day,
        hours,
        minutes,
        seconds,
        microseconds,
        tz,
    )
}

/// op:subtract-yearMonthDuration-from-dateTime
/// Subtracts a yearMonthDuration from a dateTime
pub fn datetime_subtract_ym_duration(dt: XsdDateTime, dur: XsdYearMonthDuration) -> XsdDateTime {
    datetime_add_ym_duration(dt, ym_duration_negate(dur))
}

/// op:add-yearMonthDuration-to-date
/// Adds a yearMonthDuration to a date
pub fn date_add_ym_duration(d: XsdDate, dur: XsdYearMonthDuration) -> XsdDate {
    // Extract components
    let year = year_from_date(d);
    let month = month_from_date(d);
    let day = day_from_date(d);
    let tz = d.tz_offset_minutes; // Use raw field for tz offset

    // Calculate new year and month
    let total_months = (year as i32) * 12 + ((month as i32) - 1) + dur.total_months;

    // Handle negative month calculations
    let new_year: i32 = if total_months >= 0 {
        total_months / 12
    } else {
        // For negative, we need floor division
        (total_months - 11) / 12
    };

    let month_rem = total_months - (new_year * 12);
    let new_month = (month_rem + 1) as u8;

    // Clamp day to valid range for new month
    let new_day = clamp_day(new_year, new_month, day);

    date_from_components_with_tz(new_year, new_month, new_day, tz)
}

/// op:subtract-yearMonthDuration-from-date
/// Subtracts a yearMonthDuration from a date
pub fn date_subtract_ym_duration(d: XsdDate, dur: XsdYearMonthDuration) -> XsdDate {
    date_add_ym_duration(d, ym_duration_negate(dur))
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_ym_duration_creation() {
    // 2 years, 3 months = 27 months
    let dur = XsdYearMonthDuration::from_components(false, 2, 3);
    assert(dur.total_months == 27);

    // Negative: -1 year, 6 months = -18 months
    let dur_neg = XsdYearMonthDuration::from_components(true, 1, 6);
    assert(dur_neg.total_months == -18);
    assert(dur_neg.is_negative() == true);
}

#[test]
fn test_ym_duration_components() {
    // 27 months = 2 years, 3 months
    let dur = XsdYearMonthDuration::new(27);
    assert(years_from_duration(dur) == 2);
    assert(months_from_duration(dur) == 3);

    // -18 months = -1 year, -6 months
    let dur_neg = XsdYearMonthDuration::new(-18);
    assert(years_from_duration(dur_neg) == -1);
    assert(months_from_duration(dur_neg) == -6);
}

#[test]
fn test_ym_duration_arithmetic() {
    let d1 = XsdYearMonthDuration::from_components(false, 1, 0); // 12 months
    let d2 = XsdYearMonthDuration::from_components(false, 0, 6); // 6 months

    // Add: 12 + 6 = 18 months
    let sum = ym_duration_add(d1, d2);
    assert(sum.total_months == 18);

    // Subtract: 12 - 6 = 6 months
    let diff = ym_duration_subtract(d1, d2);
    assert(diff.total_months == 6);

    // Multiply: 6 * 3 = 18 months
    let product = ym_duration_multiply(d2, 3);
    assert(product.total_months == 18);

    // Divide: 12 / 4 = 3 months
    let quotient = ym_duration_divide(d1, 4);
    assert(quotient.total_months == 3);
}

#[test]
fn test_ym_duration_comparison() {
    let d1 = XsdYearMonthDuration::new(12);
    let d2 = XsdYearMonthDuration::new(18);
    let d3 = XsdYearMonthDuration::new(12);

    assert(ym_duration_equal(d1, d3) == true);
    assert(ym_duration_equal(d1, d2) == false);

    assert(ym_duration_less_than(d1, d2) == true);
    assert(ym_duration_less_than(d2, d1) == false);

    assert(ym_duration_greater_than(d2, d1) == true);
    assert(ym_duration_greater_than(d1, d2) == false);

    assert(ym_duration_le(d1, d2) == true);
    assert(ym_duration_le(d1, d3) == true);

    assert(ym_duration_ge(d2, d1) == true);
    assert(ym_duration_ge(d1, d3) == true);
}

#[test]
fn test_date_ym_duration_arithmetic() {
    // January 31, 2000
    let d = date_from_components_with_tz(2000, 1, 31, 0);

    // Add 1 month - should clamp to Feb 29 (2000 is leap year)
    let d_plus_1m = date_add_ym_duration(d, XsdYearMonthDuration::new(1));
    assert(year_from_date(d_plus_1m) == 2000);
    assert(month_from_date(d_plus_1m) == 2);
    assert(day_from_date(d_plus_1m) == 29);

    // Add 1 year - should stay Jan 31, 2001
    let d_plus_1y = date_add_ym_duration(d, XsdYearMonthDuration::new(12));
    assert(year_from_date(d_plus_1y) == 2001);
    assert(month_from_date(d_plus_1y) == 1);
    assert(day_from_date(d_plus_1y) == 31);

    // Subtract 13 months from Jan 31, 2000 -> Dec 31, 1998
    let d_minus = date_subtract_ym_duration(d, XsdYearMonthDuration::new(13));
    assert(year_from_date(d_minus) == 1998);
    assert(month_from_date(d_minus) == 12);
    assert(day_from_date(d_minus) == 31);
}

#[test]
fn test_fn_string_from_ym_duration() {
    // P1Y2M (14 months)
    let dur1 = XsdYearMonthDuration::new(14);
    let (bytes1, len1): ([u8; 5], u32) = fn_string_from_ym_duration(dur1);
    let expected1 = "P1Y2M".as_bytes();
    assert(len1 == 5);
    for i in 0..5 {
        assert(bytes1[i] == expected1[i]);
    }

    // P0M (0 months)
    let dur2 = XsdYearMonthDuration::new(0);
    let (bytes2, len2): ([u8; 3], u32) = fn_string_from_ym_duration(dur2);
    let expected2 = "P0M".as_bytes();
    assert(len2 == 3);
    for i in 0..3 {
        assert(bytes2[i] == expected2[i]);
    }

    // P6M (6 months, no years)
    let dur3 = XsdYearMonthDuration::new(6);
    let (bytes3, len3): ([u8; 3], u32) = fn_string_from_ym_duration(dur3);
    let expected3 = "P6M".as_bytes();
    assert(len3 == 3);
    for i in 0..3 {
        assert(bytes3[i] == expected3[i]);
    }

    // P2Y (24 months, no extra months)
    let dur4 = XsdYearMonthDuration::new(24);
    let (bytes4, len4): ([u8; 3], u32) = fn_string_from_ym_duration(dur4);
    let expected4 = "P2Y".as_bytes();
    assert(len4 == 3);
    for i in 0..3 {
        assert(bytes4[i] == expected4[i]);
    }

    // -P1Y2M (negative 14 months)
    let dur5 = XsdYearMonthDuration::new(-14);
    let (bytes5, len5): ([u8; 6], u32) = fn_string_from_ym_duration(dur5);
    let expected5 = "-P1Y2M".as_bytes();
    assert(len5 == 6);
    for i in 0..6 {
        assert(bytes5[i] == expected5[i]);
    }

    // -P6M (negative 6 months)
    let dur6 = XsdYearMonthDuration::new(-6);
    let (bytes6, len6): ([u8; 4], u32) = fn_string_from_ym_duration(dur6);
    let expected6 = "-P6M".as_bytes();
    assert(len6 == 4);
    for i in 0..4 {
        assert(bytes6[i] == expected6[i]);
    }

    // P10Y (120 months)
    let dur7 = XsdYearMonthDuration::new(120);
    let (bytes7, len7): ([u8; 4], u32) = fn_string_from_ym_duration(dur7);
    let expected7 = "P10Y".as_bytes();
    assert(len7 == 4);
    for i in 0..4 {
        assert(bytes7[i] == expected7[i]);
    }

    // P10Y11M (131 months)
    let dur8 = XsdYearMonthDuration::new(131);
    let (bytes8, len8): ([u8; 7], u32) = fn_string_from_ym_duration(dur8);
    let expected8 = "P10Y11M".as_bytes();
    assert(len8 == 7);
    for i in 0..7 {
        assert(bytes8[i] == expected8[i]);
    }
}

#[test]
fn test_ym_duration_from_string() {
    // P1Y2M (14 months)
    let input1 = "P1Y2M".as_bytes();
    let (dur1, valid1) = ym_duration_from_string(input1, 5);
    assert(valid1);
    assert(dur1.total_months == 14);

    // P0M (0 months)
    let input2 = "P0M".as_bytes();
    let (dur2, valid2) = ym_duration_from_string(input2, 3);
    assert(valid2);
    assert(dur2.total_months == 0);

    // P6M (6 months, no years)
    let input3 = "P6M".as_bytes();
    let (dur3, valid3) = ym_duration_from_string(input3, 3);
    assert(valid3);
    assert(dur3.total_months == 6);

    // P2Y (24 months, no extra months)
    let input4 = "P2Y".as_bytes();
    let (dur4, valid4) = ym_duration_from_string(input4, 3);
    assert(valid4);
    assert(dur4.total_months == 24);

    // -P1Y2M (negative 14 months)
    let input5 = "-P1Y2M".as_bytes();
    let (dur5, valid5) = ym_duration_from_string(input5, 6);
    assert(valid5);
    assert(dur5.total_months == -14);

    // -P6M (negative 6 months)
    let input6 = "-P6M".as_bytes();
    let (dur6, valid6) = ym_duration_from_string(input6, 4);
    assert(valid6);
    assert(dur6.total_months == -6);

    // P10Y (120 months)
    let input7 = "P10Y".as_bytes();
    let (dur7, valid7) = ym_duration_from_string(input7, 4);
    assert(valid7);
    assert(dur7.total_months == 120);

    // P10Y11M (131 months)
    let input8 = "P10Y11M".as_bytes();
    let (dur8, valid8) = ym_duration_from_string(input8, 7);
    assert(valid8);
    assert(dur8.total_months == 131);

    // P23Y11M (287 months) - the example from the user
    let input9 = "P23Y11M".as_bytes();
    let (dur9, valid9) = ym_duration_from_string(input9, 7);
    assert(valid9);
    assert(dur9.total_months == 287);

    // Invalid: empty string
    let input_empty: [u8; 1] = [0];
    let (_dur_empty, valid_empty) = ym_duration_from_string(input_empty, 0);
    assert(!valid_empty);

    // Invalid: missing P
    let input_no_p = "1Y2M".as_bytes();
    let (_dur_no_p, valid_no_p) = ym_duration_from_string(input_no_p, 4);
    assert(!valid_no_p);
}
