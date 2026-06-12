//! fn:parse-ietf-date - Parse IETF RFC 2822/RFC 5322 format dates
//!
//! Parses dates in formats like:
//! - "Wed, 20 Aug 2014 19:36:01 GMT"
//! - "Wed, 20 Aug 2014 19:36:01 +0000"
//! - "20 Aug 2014 19:36:01 GMT"
//! - "Wed 20 Aug 2014 19:36:01 GMT" (no comma)
//! - "Wednesday, 20 Aug 2014 19:36:01 GMT" (full day name)
//!
//! RFC 2822/5322 format:
//! [day-of-week ","] day month year hour ":" minute [":" second] [timezone]
//!
//! Supported timezone formats:
//! - Named: GMT, UT, UTC, EST, EDT, CST, CDT, MST, MDT, PST, PDT
//! - Numeric: +HHMM, -HHMM

use crate::datetime::datetime_from_components_with_tz;
use crate::types::XsdDateTime;

// ============================================================================
// Parse Result Type
// ============================================================================

/// Result of parsing an IETF date
/// success: true if parsing succeeded
/// datetime: the parsed DateTime (valid only if success is true)
pub struct IetfDateParseResult {
    pub success: bool,
    pub datetime: XsdDateTime,
}

impl IetfDateParseResult {
    pub fn failure() -> Self {
        IetfDateParseResult { success: false, datetime: XsdDateTime::new(0) }
    }

    pub fn ok(datetime: XsdDateTime) -> Self {
        IetfDateParseResult { success: true, datetime }
    }
}

// ============================================================================
// Character Classification Helpers
// ============================================================================

/// Check if a character is a digit (0-9)
fn is_digit(c: u8) -> bool {
    (c >= 48) & (c <= 57) // '0' = 48, '9' = 57
}

/// Check if a character is whitespace (space or tab)
fn is_whitespace(c: u8) -> bool {
    (c == 32) | (c == 9) // space = 32, tab = 9
}

/// Check if a character is a letter (a-z or A-Z)
fn is_letter(c: u8) -> bool {
    ((c >= 65) & (c <= 90)) | ((c >= 97) & (c <= 122))
}

/// Convert uppercase letter to lowercase
fn to_lower(c: u8) -> u8 {
    if (c >= 65) & (c <= 90) {
        c + 32
    } else {
        c
    }
}

/// Convert digit character to its numeric value
fn digit_value(c: u8) -> u8 {
    c - 48
}

// ============================================================================
// Parsing Helper Functions
// ============================================================================

/// Get character at position, or 0 if out of bounds
fn get_char<let N: u32>(input: [u8; N], pos: u32, len: u32) -> u8 {
    if pos >= len {
        0
    } else {
        input[pos]
    }
}

/// Skip whitespace and return new position
fn skip_ws<let N: u32>(input: [u8; N], start: u32, len: u32) -> u32 {
    let mut pos = start;
    // Bounded loop - skip at most 16 whitespace chars
    for _ in 0..16 {
        if (pos < len) & is_whitespace(get_char(input, pos, len)) {
            pos = pos + 1;
        }
    }
    pos
}

/// Parse a 1-2 digit number starting at pos
/// Returns (value, new_position, success)
fn parse_1_2_digits<let N: u32>(input: [u8; N], start: u32, len: u32) -> (u32, u32, bool) {
    let c1 = get_char(input, start, len);

    if !is_digit(c1) {
        (0, start, false)
    } else {
        let v1 = digit_value(c1) as u32;
        let c2 = get_char(input, start + 1, len);

        if is_digit(c2) {
            let v2 = digit_value(c2) as u32;
            (v1 * 10 + v2, start + 2, true)
        } else {
            (v1, start + 1, true)
        }
    }
}

/// Parse exactly 2 digits starting at pos
/// Returns (value, new_position, success)
fn parse_2_digits<let N: u32>(input: [u8; N], start: u32, len: u32) -> (u32, u32, bool) {
    let c1 = get_char(input, start, len);
    let c2 = get_char(input, start + 1, len);

    if is_digit(c1) & is_digit(c2) {
        let value = (digit_value(c1) as u32) * 10 + (digit_value(c2) as u32);
        (value, start + 2, true)
    } else {
        (0, start, false)
    }
}

/// Parse a 2-4 digit year starting at pos
/// Returns (year, new_position, success)
fn parse_year<let N: u32>(input: [u8; N], start: u32, len: u32) -> (i32, u32, bool) {
    // Parse up to 4 digits
    let c1 = get_char(input, start, len);
    let c2 = get_char(input, start + 1, len);
    let c3 = get_char(input, start + 2, len);
    let c4 = get_char(input, start + 3, len);

    let d1 = is_digit(c1);
    let d2 = is_digit(c2);
    let d3 = is_digit(c3);
    let d4 = is_digit(c4);

    // Need at least 2 digits
    if !d1 | !d2 {
        (0, start, false)
    } else {
        let v1 = digit_value(c1) as u32;
        let v2 = digit_value(c2) as u32;
        let v3 = if d3 { digit_value(c3) as u32 } else { 0 };
        let v4 = if d3 & d4 { digit_value(c4) as u32 } else { 0 };

        // Determine how many digits and calculate year
        let (value, count): (u32, u32) = if d3 & d4 {
            (v1 * 1000 + v2 * 100 + v3 * 10 + v4, 4)
        } else if d3 {
            (v1 * 100 + v2 * 10 + v3, 3)
        } else {
            (v1 * 10 + v2, 2)
        };

        // RFC 2822: 2-digit years 00-49 are 2000-2049, 50-99 are 1950-1999
        let year: i32 = if count == 2 {
            if value < 50 {
                2000 + value as i32
            } else {
                1900 + value as i32
            }
        } else {
            value as i32
        };

        (year, start + count, true)
    }
}

/// Parse fractional seconds (after decimal point)
/// Returns (microseconds, new_position)
fn parse_fractional<let N: u32>(input: [u8; N], start: u32, len: u32) -> (u32, u32) {
    // Check for decimal point
    if get_char(input, start, len) != 46 {
        // '.' = 46
        (0, start)
    } else {
        let mut pos = start + 1;
        let mut fraction: u64 = 0;
        let mut digits: u32 = 0;

        // Parse up to 6 digits (unrolled)
        let c1 = get_char(input, pos, len);
        if is_digit(c1) {
            fraction = digit_value(c1) as u64;
            digits = 1;
            pos = pos + 1;

            let c2 = get_char(input, pos, len);
            if is_digit(c2) {
                fraction = fraction * 10 + digit_value(c2) as u64;
                digits = 2;
                pos = pos + 1;

                let c3 = get_char(input, pos, len);
                if is_digit(c3) {
                    fraction = fraction * 10 + digit_value(c3) as u64;
                    digits = 3;
                    pos = pos + 1;

                    let c4 = get_char(input, pos, len);
                    if is_digit(c4) {
                        fraction = fraction * 10 + digit_value(c4) as u64;
                        digits = 4;
                        pos = pos + 1;

                        let c5 = get_char(input, pos, len);
                        if is_digit(c5) {
                            fraction = fraction * 10 + digit_value(c5) as u64;
                            digits = 5;
                            pos = pos + 1;

                            let c6 = get_char(input, pos, len);
                            if is_digit(c6) {
                                fraction = fraction * 10 + digit_value(c6) as u64;
                                digits = 6;
                                pos = pos + 1;
                            }
                        }
                    }
                }
            }
        }

        // Skip any remaining digits beyond 6
        for _ in 0..10 {
            if is_digit(get_char(input, pos, len)) {
                pos = pos + 1;
            }
        }

        // Convert to microseconds based on number of digits
        let microseconds: u64 = if digits == 0 {
            0
        } else if digits == 1 {
            fraction * 100000
        } else if digits == 2 {
            fraction * 10000
        } else if digits == 3 {
            fraction * 1000
        } else if digits == 4 {
            fraction * 100
        } else if digits == 5 {
            fraction * 10
        } else {
            fraction
        };

        (microseconds as u32, pos)
    }
}

/// Parse 3-letter month name (case-insensitive)
/// Returns (month 1-12, new_position, success)
fn parse_month<let N: u32>(input: [u8; N], start: u32, len: u32) -> (u8, u32, bool) {
    if start + 3 > len {
        (0, start, false)
    } else {
        let c1 = to_lower(get_char(input, start, len));
        let c2 = to_lower(get_char(input, start + 1, len));
        let c3 = to_lower(get_char(input, start + 2, len));

        // Match month abbreviations using ASCII values
        // j=106, a=97, n=110, f=102, e=101, b=98, m=109, r=114, p=112, y=121
        // u=117, l=108, g=103, s=115, o=111, c=99, t=116, v=118, d=100
        let month: u8 = if (c1 == 106) & (c2 == 97) & (c3 == 110) {
            1 // jan
        } else if (c1 == 102) & (c2 == 101) & (c3 == 98) {
            2 // feb
        } else if (c1 == 109) & (c2 == 97) & (c3 == 114) {
            3 // mar
        } else if (c1 == 97) & (c2 == 112) & (c3 == 114) {
            4 // apr
        } else if (c1 == 109) & (c2 == 97) & (c3 == 121) {
            5 // may
        } else if (c1 == 106) & (c2 == 117) & (c3 == 110) {
            6 // jun
        } else if (c1 == 106) & (c2 == 117) & (c3 == 108) {
            7 // jul
        } else if (c1 == 97) & (c2 == 117) & (c3 == 103) {
            8 // aug
        } else if (c1 == 115) & (c2 == 101) & (c3 == 112) {
            9 // sep
        } else if (c1 == 111) & (c2 == 99) & (c3 == 116) {
            10 // oct
        } else if (c1 == 110) & (c2 == 111) & (c3 == 118) {
            11 // nov
        } else if (c1 == 100) & (c2 == 101) & (c3 == 99) {
            12 // dec
        } else {
            0
        };

        if month != 0 {
            (month, start + 3, true)
        } else {
            (0, start, false)
        }
    }
}

/// Parse timezone
/// Returns (offset_minutes, new_position, success)
fn parse_timezone<let N: u32>(input: [u8; N], start: u32, len: u32) -> (i16, u32) {
    let pos = skip_ws(input, start, len);

    if pos >= len {
        // No timezone - default to UTC
        (0, pos)
    } else {
        let c = get_char(input, pos, len);

        // Numeric timezone (+HHMM or -HHMM)
        if (c == 43) | (c == 45) {
            // '+' = 43, '-' = 45
            let is_neg = c == 45;
            let (hours, pos2, h_ok) = parse_2_digits(input, pos + 1, len);

            if !h_ok {
                (0, pos)
            } else {
                // Optional colon
                let pos3 = if get_char(input, pos2, len) == 58 {
                    pos2 + 1
                } else {
                    pos2
                };
                let (minutes, pos4, _m_ok) = parse_2_digits(input, pos3, len);
                let offset: i16 = (hours * 60 + minutes) as i16;
                let final_offset: i16 = if is_neg { -offset } else { offset };
                (final_offset, pos4)
            }
        } else if is_letter(c) {
            // Named timezone
            let c1 = to_lower(c);
            let c2 = to_lower(get_char(input, pos + 1, len));
            let c3 = to_lower(get_char(input, pos + 2, len));

            // g=103, m=109, t=116, u=117, e=101, s=115, d=100, c=99, p=112
            let (offset, skip): (i16, u32) = if (c1 == 103) & (c2 == 109) & (c3 == 116) {
                // GMT
                (0, 3)
            } else if (c1 == 117) & (c2 == 116) & (c3 == 99) {
                // UTC
                (0, 3)
            } else if (c1 == 117) & (c2 == 116) & !is_letter(c3) {
                // UT
                (0, 2)
            } else if (c1 == 101) & (c2 == 115) & (c3 == 116) {
                // EST -5:00
                (-300, 3)
            } else if (c1 == 101) & (c2 == 100) & (c3 == 116) {
                // EDT -4:00
                (-240, 3)
            } else if (c1 == 99) & (c2 == 115) & (c3 == 116) {
                // CST -6:00
                (-360, 3)
            } else if (c1 == 99) & (c2 == 100) & (c3 == 116) {
                // CDT -5:00
                (-300, 3)
            } else if (c1 == 109) & (c2 == 115) & (c3 == 116) {
                // MST -7:00
                (-420, 3)
            } else if (c1 == 109) & (c2 == 100) & (c3 == 116) {
                // MDT -6:00
                (-360, 3)
            } else if (c1 == 112) & (c2 == 115) & (c3 == 116) {
                // PST -8:00
                (-480, 3)
            } else if (c1 == 112) & (c2 == 100) & (c3 == 116) {
                // PDT -7:00
                (-420, 3)
            } else {
                // Unknown timezone name - skip letters and default to UTC
                let mut skip_count: u32 = 0;
                for i in 0..10 {
                    if is_letter(get_char(input, pos + i, len)) {
                        skip_count = i + 1;
                    }
                }
                (0, skip_count)
            };
            (offset, pos + skip)
        } else {
            (0, pos)
        }
    }
}

/// Skip day name (Mon, Tuesday, etc.) and optional comma
/// Returns new position
fn skip_day_name<let N: u32>(input: [u8; N], start: u32, len: u32) -> u32 {
    let pos = skip_ws(input, start, len);

    // Check if we start with a letter (day name)
    if !is_letter(get_char(input, pos, len)) {
        pos
    } else {
        // Count consecutive letters (up to 10 for "Wednesday")
        // We need to find where the letters stop, not just check each position
        let l0 = is_letter(get_char(input, pos, len));
        let l1 = is_letter(get_char(input, pos + 1, len));
        let l2 = is_letter(get_char(input, pos + 2, len));
        let l3 = is_letter(get_char(input, pos + 3, len));
        let l4 = is_letter(get_char(input, pos + 4, len));
        let l5 = is_letter(get_char(input, pos + 5, len));
        let l6 = is_letter(get_char(input, pos + 6, len));
        let l7 = is_letter(get_char(input, pos + 7, len));
        let l8 = is_letter(get_char(input, pos + 8, len));
        let l9 = is_letter(get_char(input, pos + 9, len));

        // Find first non-letter (or position 10)
        let end: u32 = if !l0 {
            pos
        } else if !l1 {
            pos + 1
        } else if !l2 {
            pos + 2
        } else if !l3 {
            pos + 3
        } else if !l4 {
            pos + 4
        } else if !l5 {
            pos + 5
        } else if !l6 {
            pos + 6
        } else if !l7 {
            pos + 7
        } else if !l8 {
            pos + 8
        } else if !l9 {
            pos + 9
        } else {
            pos + 10
        };

        // Skip whitespace and optional comma
        let pos2 = skip_ws(input, end, len);
        if get_char(input, pos2, len) == 44 {
            // ',' = 44
            skip_ws(input, pos2 + 1, len)
        } else {
            pos2
        }
    }
}

/// Skip date separator (whitespace and/or hyphen)
fn skip_date_sep<let N: u32>(input: [u8; N], start: u32, len: u32) -> u32 {
    let pos = skip_ws(input, start, len);
    if get_char(input, pos, len) == 45 {
        // '-' = 45
        skip_ws(input, pos + 1, len)
    } else {
        pos
    }
}

// ============================================================================
// Main Parsing Function
// ============================================================================

/// fn:parse-ietf-date - Parse an IETF RFC 2822/5322 format date string
///
/// Supports formats like:
/// - "Wed, 20 Aug 2014 19:36:01 GMT"
/// - "Wed, 20 Aug 2014 19:36:01 +0000"
/// - "20 Aug 2014 19:36:01 GMT"
/// - "Aug 20 19:36 2014" (alternative ordering)
///
/// Returns an IetfDateParseResult indicating success and the parsed datetime.
pub fn parse_ietf_date<let N: u32>(input: [u8; N], len: u32) -> IetfDateParseResult {
    let actual_len = if len > N { N } else { len };

    // Skip leading whitespace and optional day name
    let pos0 = skip_ws(input, 0, actual_len);
    let pos1 = skip_day_name(input, pos0, actual_len);
    let pos2 = skip_ws(input, pos1, actual_len);

    // Try to parse as day first (e.g., "20 Aug 2014")
    let (day1, pos3, day1_ok) = parse_1_2_digits(input, pos2, actual_len);

    // Initialize components
    let mut day: u8 = 0;
    let mut month: u8 = 0;
    let mut year: i32 = 0;
    let mut pos: u32 = pos3;
    let mut format_ok = true;

    if day1_ok {
        // Format 1: day month year (most common RFC 2822)
        day = day1 as u8;
        let pos4 = skip_date_sep(input, pos3, actual_len);
        let (m, pos5, m_ok) = parse_month(input, pos4, actual_len);
        if m_ok {
            month = m;
            let pos6 = skip_date_sep(input, pos5, actual_len);
            let (y, pos7, y_ok) = parse_year(input, pos6, actual_len);
            if y_ok {
                year = y;
                pos = pos7;
            } else {
                format_ok = false;
            }
        } else {
            format_ok = false;
        }
    } else {
        // Format 2: month-day time year (e.g., "Aug-20 19:36 2014")
        let (m, pos4, m_ok) = parse_month(input, pos2, actual_len);
        if m_ok {
            month = m;
            let pos5 = skip_date_sep(input, pos4, actual_len);
            let (d, pos6, d_ok) = parse_1_2_digits(input, pos5, actual_len);
            if d_ok {
                day = d as u8;
                pos = pos6;
                // Year will be parsed after time in this format
            } else {
                format_ok = false;
            }
        } else {
            format_ok = false;
        }
    }

    if !format_ok {
        IetfDateParseResult::failure()
    } else {
        // Parse time: HH:MM[:SS[.fraction]]
        let pos_time = skip_ws(input, pos, actual_len);
        let (hour, pos_h, h_ok) = parse_2_digits(input, pos_time, actual_len);

        // Expect colon
        let colon1 = get_char(input, pos_h, actual_len) == 58; // ':' = 58

        if !h_ok | !colon1 {
            IetfDateParseResult::failure()
        } else {
            let (minute, pos_m, min_ok) = parse_2_digits(input, pos_h + 1, actual_len);

            if !min_ok {
                IetfDateParseResult::failure()
            } else {
                // Optional seconds
                let has_seconds = get_char(input, pos_m, actual_len) == 58;
                let (second, pos_s) = if has_seconds {
                    let (s, ps, s_ok) = parse_2_digits(input, pos_m + 1, actual_len);
                    if s_ok {
                        (s, ps)
                    } else {
                        (0, pos_m + 1)
                    }
                } else {
                    (0, pos_m)
                };

                // Optional fractional seconds
                let (microseconds, pos_frac) = parse_fractional(input, pos_s, actual_len);

                // If year not yet parsed (format 2), parse it now
                let (final_year, pos_year) = if year == 0 {
                    let pos_pre_year = skip_ws(input, pos_frac, actual_len);
                    let (y, py, y_ok) = parse_year(input, pos_pre_year, actual_len);
                    if y_ok {
                        (y, py)
                    } else {
                        (0, pos_pre_year)
                    }
                } else {
                    (year, pos_frac)
                };

                // Parse timezone
                let (tz_offset, _pos_tz) = parse_timezone(input, pos_year, actual_len);

                // Validate ranges
                let valid = (month >= 1)
                    & (month <= 12)
                    & (day >= 1)
                    & (day <= 31)
                    & (hour <= 23)
                    & (minute <= 59)
                    & (second <= 59)
                    & (final_year != 0);

                if !valid {
                    IetfDateParseResult::failure()
                } else {
                    // Construct the datetime
                    let datetime = datetime_from_components_with_tz(
                        final_year,
                        month,
                        day,
                        hour as u8,
                        minute as u8,
                        second as u8,
                        microseconds,
                        tz_offset,
                    );

                    IetfDateParseResult::ok(datetime)
                }
            }
        }
    }
}

/// Convenience function: parse IETF date from a bounded string
/// Returns Option-like result: success indicates if parsing succeeded
pub fn fn_parse_ietf_date<let N: u32>(input: [u8; N], len: u32) -> IetfDateParseResult {
    parse_ietf_date(input, len)
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_parse_ietf_date_basic() {
    // "Wed, 20 Aug 2014 19:36:01 GMT"
    let input: [u8; 30] = [
        87, 101, 100, 44, 32, 50, 48, 32, 65, 117, 103, 32, 50, 48, 49, 52, 32, 49, 57, 58, 51, 54,
        58, 48, 49, 32, 71, 77, 84, 0,
    ];
    let result = parse_ietf_date(input, 29);
    assert(result.success);

    // Verify components using datetime extraction functions
    assert(crate::datetime::year_from_datetime(result.datetime) == 2014);
    assert(crate::datetime::month_from_datetime(result.datetime) == 8);
    assert(crate::datetime::day_from_datetime(result.datetime) == 20);
    assert(crate::datetime::hours_from_datetime(result.datetime) == 19);
    assert(crate::datetime::minutes_from_datetime(result.datetime) == 36);
    assert(crate::datetime::seconds_from_datetime(result.datetime) == 1);
}

#[test]
fn test_parse_ietf_date_no_day_name() {
    // " 20 Aug 2014 19:36:01 GMT" (note leading space)
    let input: [u8; 26] = [
        32, 50, 48, 32, 65, 117, 103, 32, 50, 48, 49, 52, 32, 49, 57, 58, 51, 54, 58, 48, 49, 32,
        71, 77, 84, 0,
    ];
    let result = parse_ietf_date(input, 25);
    assert(result.success);

    assert(crate::datetime::year_from_datetime(result.datetime) == 2014);
    assert(crate::datetime::month_from_datetime(result.datetime) == 8);
    assert(crate::datetime::day_from_datetime(result.datetime) == 20);
}

#[test]
fn test_parse_ietf_date_est_timezone() {
    // "Wed, 20 Aug 2014 14:36:01 EST"
    let input: [u8; 30] = [
        87, 101, 100, 44, 32, 50, 48, 32, 65, 117, 103, 32, 50, 48, 49, 52, 32, 49, 52, 58, 51, 54,
        58, 48, 49, 32, 69, 83, 84, 0,
    ];
    let result = parse_ietf_date(input, 29);
    assert(result.success);

    // EST is -5 hours from UTC
    assert(result.datetime.tz_offset_minutes == -300);
}

#[test]
fn test_parse_ietf_date_numeric_timezone() {
    // "Wed, 20 Aug 2014 19:36:01 +0530"
    let input: [u8; 32] = [
        87, 101, 100, 44, 32, 50, 48, 32, 65, 117, 103, 32, 50, 48, 49, 52, 32, 49, 57, 58, 51, 54,
        58, 48, 49, 32, 43, 48, 53, 51, 48, 0,
    ];
    let result = parse_ietf_date(input, 31);
    assert(result.success);

    // +05:30 = 330 minutes
    assert(result.datetime.tz_offset_minutes == 330);
}

#[test]
fn test_parse_ietf_date_no_seconds() {
    // "Wed, 20 Aug 2014 19:36 GMT"
    let input: [u8; 27] = [
        87, 101, 100, 44, 32, 50, 48, 32, 65, 117, 103, 32, 50, 48, 49, 52, 32, 49, 57, 58, 51, 54,
        32, 71, 77, 84, 0,
    ];
    let result = parse_ietf_date(input, 26);
    assert(result.success);

    assert(crate::datetime::seconds_from_datetime(result.datetime) == 0);
}

#[test]
fn test_parse_ietf_date_fractional_seconds() {
    // "Wed, 20 Aug 2014 19:36:01.25 GMT"
    let input: [u8; 33] = [
        87, 101, 100, 44, 32, 50, 48, 32, 65, 117, 103, 32, 50, 48, 49, 52, 32, 49, 57, 58, 51, 54,
        58, 48, 49, 46, 50, 53, 32, 71, 77, 84, 0,
    ];
    let result = parse_ietf_date(input, 32);
    assert(result.success);

    // 0.25 seconds = 250000 microseconds
    assert(crate::datetime::microseconds_from_datetime(result.datetime) == 250000);
}

#[test]
fn test_parse_ietf_date_case_insensitive() {
    // "wed, 20 aug 2014 19:36:01 gmt" (all lowercase)
    let input: [u8; 30] = [
        119, 101, 100, 44, 32, 50, 48, 32, 97, 117, 103, 32, 50, 48, 49, 52, 32, 49, 57, 58, 51, 54,
        58, 48, 49, 32, 103, 109, 116, 0,
    ];
    let result = parse_ietf_date(input, 29);
    assert(result.success);
}

#[test]
fn test_parse_ietf_date_negative_timezone() {
    // "Wed, 20 Aug 2014 19:36:01 -0600"
    let input: [u8; 32] = [
        87, 101, 100, 44, 32, 50, 48, 32, 65, 117, 103, 32, 50, 48, 49, 52, 32, 49, 57, 58, 51, 54,
        58, 48, 49, 32, 45, 48, 54, 48, 48, 0,
    ];
    let result = parse_ietf_date(input, 31);
    assert(result.success);

    // -06:00 = -360 minutes
    assert(result.datetime.tz_offset_minutes == -360);
}

#[test]
fn test_parse_ietf_date_pdt_timezone() {
    // "Wed, 20 Aug 2014 12:36:01 PDT"
    let input: [u8; 30] = [
        87, 101, 100, 44, 32, 50, 48, 32, 65, 117, 103, 32, 50, 48, 49, 52, 32, 49, 50, 58, 51, 54,
        58, 48, 49, 32, 80, 68, 84, 0,
    ];
    let result = parse_ietf_date(input, 29);
    assert(result.success);

    // PDT is -7 hours from UTC
    assert(result.datetime.tz_offset_minutes == -420);
}
