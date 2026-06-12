//! Gregorian partial date types and comparisons for XPath/SPARQL
//!
//! Implements comparison operations for xs:gYear, xs:gMonth, xs:gDay,
//! xs:gYearMonth, and xs:gMonthDay types.
//!
//! These are partial date types used for recurring events (e.g., birthdays).
//!
//! XPath operators implemented:
//! - op:gYear-equal, op:gMonth-equal, op:gDay-equal
//! - op:gYearMonth-equal, op:gMonthDay-equal

// ============================================================================
// Gregorian Types
// ============================================================================

/// xs:gYear - A Gregorian year
pub struct XsdGYear {
    pub year: i32,
    pub tz_offset_minutes: i16,
}

impl XsdGYear {
    pub fn new(year: i32) -> Self {
        Self { year, tz_offset_minutes: 0 }
    }

    pub fn new_with_tz(year: i32, tz_offset_minutes: i16) -> Self {
        Self { year, tz_offset_minutes }
    }
}

/// xs:gMonth - A Gregorian month (1-12)
pub struct XsdGMonth {
    pub month: u8,
    pub tz_offset_minutes: i16,
}

impl XsdGMonth {
    pub fn new(month: u8) -> Self {
        Self { month, tz_offset_minutes: 0 }
    }

    pub fn new_with_tz(month: u8, tz_offset_minutes: i16) -> Self {
        Self { month, tz_offset_minutes }
    }
}

/// xs:gDay - A Gregorian day (1-31)
pub struct XsdGDay {
    pub day: u8,
    pub tz_offset_minutes: i16,
}

impl XsdGDay {
    pub fn new(day: u8) -> Self {
        Self { day, tz_offset_minutes: 0 }
    }

    pub fn new_with_tz(day: u8, tz_offset_minutes: i16) -> Self {
        Self { day, tz_offset_minutes }
    }
}

/// xs:gYearMonth - A Gregorian year and month
pub struct XsdGYearMonth {
    pub year: i32,
    pub month: u8,
    pub tz_offset_minutes: i16,
}

impl XsdGYearMonth {
    pub fn new(year: i32, month: u8) -> Self {
        Self { year, month, tz_offset_minutes: 0 }
    }

    pub fn new_with_tz(year: i32, month: u8, tz_offset_minutes: i16) -> Self {
        Self { year, month, tz_offset_minutes }
    }
}

/// xs:gMonthDay - A Gregorian month and day (recurring annually)
pub struct XsdGMonthDay {
    pub month: u8,
    pub day: u8,
    pub tz_offset_minutes: i16,
}

impl XsdGMonthDay {
    pub fn new(month: u8, day: u8) -> Self {
        Self { month, day, tz_offset_minutes: 0 }
    }

    pub fn new_with_tz(month: u8, day: u8, tz_offset_minutes: i16) -> Self {
        Self { month, day, tz_offset_minutes }
    }
}

// ============================================================================
// Equality Comparison Functions
// ============================================================================

/// op:gYear-equal - Compare two gYear values for equality
/// Two gYear values are equal if they represent the same year
pub fn gyear_equal(a: XsdGYear, b: XsdGYear) -> bool {
    a.year == b.year
}

/// op:gMonth-equal - Compare two gMonth values for equality
/// Two gMonth values are equal if they represent the same month
pub fn gmonth_equal(a: XsdGMonth, b: XsdGMonth) -> bool {
    a.month == b.month
}

/// op:gDay-equal - Compare two gDay values for equality
/// Two gDay values are equal if they represent the same day
pub fn gday_equal(a: XsdGDay, b: XsdGDay) -> bool {
    a.day == b.day
}

/// op:gYearMonth-equal - Compare two gYearMonth values for equality
/// Two gYearMonth values are equal if they have the same year and month
pub fn gyearmonth_equal(a: XsdGYearMonth, b: XsdGYearMonth) -> bool {
    (a.year == b.year) & (a.month == b.month)
}

/// op:gMonthDay-equal - Compare two gMonthDay values for equality
/// Two gMonthDay values are equal if they have the same month and day
pub fn gmonthday_equal(a: XsdGMonthDay, b: XsdGMonthDay) -> bool {
    (a.month == b.month) & (a.day == b.day)
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_gyear_equal() {
    let y1 = XsdGYear::new(2024);
    let y2 = XsdGYear::new(2024);
    let y3 = XsdGYear::new(2025);

    assert(gyear_equal(y1, y2) == true);
    assert(gyear_equal(y1, y3) == false);
}

#[test]
fn test_gmonth_equal() {
    let m1 = XsdGMonth::new(6);
    let m2 = XsdGMonth::new(6);
    let m3 = XsdGMonth::new(7);

    assert(gmonth_equal(m1, m2) == true);
    assert(gmonth_equal(m1, m3) == false);
}

#[test]
fn test_gday_equal() {
    let d1 = XsdGDay::new(15);
    let d2 = XsdGDay::new(15);
    let d3 = XsdGDay::new(16);

    assert(gday_equal(d1, d2) == true);
    assert(gday_equal(d1, d3) == false);
}

#[test]
fn test_gyearmonth_equal() {
    let ym1 = XsdGYearMonth::new(2024, 6);
    let ym2 = XsdGYearMonth::new(2024, 6);
    let ym3 = XsdGYearMonth::new(2024, 7);
    let ym4 = XsdGYearMonth::new(2025, 6);

    assert(gyearmonth_equal(ym1, ym2) == true);
    assert(gyearmonth_equal(ym1, ym3) == false);
    assert(gyearmonth_equal(ym1, ym4) == false);
}

#[test]
fn test_gmonthday_equal() {
    let md1 = XsdGMonthDay::new(6, 15);
    let md2 = XsdGMonthDay::new(6, 15);
    let md3 = XsdGMonthDay::new(6, 16);
    let md4 = XsdGMonthDay::new(7, 15);

    assert(gmonthday_equal(md1, md2) == true);
    assert(gmonthday_equal(md1, md3) == false);
    assert(gmonthday_equal(md1, md4) == false);
}
