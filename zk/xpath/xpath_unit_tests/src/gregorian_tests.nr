//! Unit tests for Gregorian partial date types

use dep::xpath::{
    gday_equal, gmonth_equal, gmonthday_equal, gyear_equal, gyearmonth_equal, XsdGDay, XsdGMonth,
    XsdGMonthDay, XsdGYear, XsdGYearMonth,
};

// ============================================================================
// GYear Tests
// ============================================================================

#[test]
fn test_gyear_equal_same() {
    let y1 = XsdGYear::new(2024);
    let y2 = XsdGYear::new(2024);
    assert(gyear_equal(y1, y2));
}

#[test]
fn test_gyear_equal_different() {
    let y1 = XsdGYear::new(2024);
    let y2 = XsdGYear::new(2023);
    assert(!gyear_equal(y1, y2));
}

#[test]
fn test_gyear_with_timezone() {
    let y1 = XsdGYear::new_with_tz(2024, 0);
    let y2 = XsdGYear::new_with_tz(2024, 60);
    // Same year, different timezone - equality is based on value
    assert(gyear_equal(y1, y2));
}

// ============================================================================
// GMonth Tests
// ============================================================================

#[test]
fn test_gmonth_equal_same() {
    let m1 = XsdGMonth::new(6);
    let m2 = XsdGMonth::new(6);
    assert(gmonth_equal(m1, m2));
}

#[test]
fn test_gmonth_equal_different() {
    let m1 = XsdGMonth::new(6);
    let m2 = XsdGMonth::new(7);
    assert(!gmonth_equal(m1, m2));
}

#[test]
fn test_gmonth_january() {
    let m1 = XsdGMonth::new(1);
    let m2 = XsdGMonth::new(1);
    assert(gmonth_equal(m1, m2));
}

#[test]
fn test_gmonth_december() {
    let m1 = XsdGMonth::new(12);
    let m2 = XsdGMonth::new(12);
    assert(gmonth_equal(m1, m2));
}

// ============================================================================
// GDay Tests
// ============================================================================

#[test]
fn test_gday_equal_same() {
    let d1 = XsdGDay::new(15);
    let d2 = XsdGDay::new(15);
    assert(gday_equal(d1, d2));
}

#[test]
fn test_gday_equal_different() {
    let d1 = XsdGDay::new(15);
    let d2 = XsdGDay::new(16);
    assert(!gday_equal(d1, d2));
}

#[test]
fn test_gday_first_day() {
    let d1 = XsdGDay::new(1);
    let d2 = XsdGDay::new(1);
    assert(gday_equal(d1, d2));
}

#[test]
fn test_gday_last_day() {
    let d1 = XsdGDay::new(31);
    let d2 = XsdGDay::new(31);
    assert(gday_equal(d1, d2));
}

// ============================================================================
// GYearMonth Tests
// ============================================================================

#[test]
fn test_gyearmonth_equal_same() {
    let ym1 = XsdGYearMonth::new(2024, 6);
    let ym2 = XsdGYearMonth::new(2024, 6);
    assert(gyearmonth_equal(ym1, ym2));
}

#[test]
fn test_gyearmonth_equal_different_year() {
    let ym1 = XsdGYearMonth::new(2024, 6);
    let ym2 = XsdGYearMonth::new(2023, 6);
    assert(!gyearmonth_equal(ym1, ym2));
}

#[test]
fn test_gyearmonth_equal_different_month() {
    let ym1 = XsdGYearMonth::new(2024, 6);
    let ym2 = XsdGYearMonth::new(2024, 7);
    assert(!gyearmonth_equal(ym1, ym2));
}

#[test]
fn test_gyearmonth_with_timezone() {
    let ym1 = XsdGYearMonth::new_with_tz(2024, 6, 0);
    let ym2 = XsdGYearMonth::new_with_tz(2024, 6, 120);
    assert(gyearmonth_equal(ym1, ym2));
}

// ============================================================================
// GMonthDay Tests
// ============================================================================

#[test]
fn test_gmonthday_equal_same() {
    let md1 = XsdGMonthDay::new(12, 25); // Christmas
    let md2 = XsdGMonthDay::new(12, 25);
    assert(gmonthday_equal(md1, md2));
}

#[test]
fn test_gmonthday_equal_different_month() {
    let md1 = XsdGMonthDay::new(12, 25);
    let md2 = XsdGMonthDay::new(11, 25);
    assert(!gmonthday_equal(md1, md2));
}

#[test]
fn test_gmonthday_equal_different_day() {
    let md1 = XsdGMonthDay::new(12, 25);
    let md2 = XsdGMonthDay::new(12, 26);
    assert(!gmonthday_equal(md1, md2));
}

#[test]
fn test_gmonthday_february_29() {
    // Leap day
    let md1 = XsdGMonthDay::new(2, 29);
    let md2 = XsdGMonthDay::new(2, 29);
    assert(gmonthday_equal(md1, md2));
}
