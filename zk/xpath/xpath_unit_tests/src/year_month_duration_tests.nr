//! Unit tests for YearMonthDuration functions

use dep::xpath::{
    months_from_duration, XsdYearMonthDuration, years_from_duration, ym_duration_add,
    ym_duration_divide, ym_duration_divide_by_duration, ym_duration_equal, ym_duration_ge,
    ym_duration_greater_than, ym_duration_le, ym_duration_less_than, ym_duration_multiply,
    ym_duration_negate, ym_duration_subtract,
};

// ============================================================================
// YearMonthDuration Construction Tests
// ============================================================================

#[test]
fn test_ym_duration_from_months() {
    let dur = XsdYearMonthDuration::new(14); // 1 year, 2 months
    assert(dur.total_months == 14);
}

#[test]
fn test_ym_duration_from_components() {
    let dur = XsdYearMonthDuration::from_components(false, 2, 6);
    assert(dur.total_months == 30); // 2*12 + 6
}

#[test]
fn test_ym_duration_from_components_negative() {
    let dur = XsdYearMonthDuration::from_components(true, 1, 3);
    assert(dur.total_months == -15); // -(1*12 + 3)
}

#[test]
fn test_ym_duration_zero() {
    let dur = XsdYearMonthDuration::zero();
    assert(dur.total_months == 0);
}

// ============================================================================
// Component Extraction Tests
// ============================================================================

#[test]
fn test_years_from_duration_positive() {
    let dur = XsdYearMonthDuration::new(30); // 2 years, 6 months
    assert(years_from_duration(dur) == 2);
}

#[test]
fn test_years_from_duration_negative() {
    let dur = XsdYearMonthDuration::new(-30); // -2 years, -6 months
    assert(years_from_duration(dur) == -2);
}

#[test]
fn test_months_from_duration_positive() {
    let dur = XsdYearMonthDuration::new(30); // 2 years, 6 months
    assert(months_from_duration(dur) == 6);
}

#[test]
fn test_months_from_duration_negative() {
    let dur = XsdYearMonthDuration::new(-30); // -2 years, -6 months
    assert(months_from_duration(dur) == -6);
}

// ============================================================================
// Arithmetic Tests
// ============================================================================

#[test]
fn test_ym_duration_add() {
    let d1 = XsdYearMonthDuration::new(12); // 1 year
    let d2 = XsdYearMonthDuration::new(6); // 6 months
    let result = ym_duration_add(d1, d2);
    assert(result.total_months == 18);
}

#[test]
fn test_ym_duration_subtract() {
    let d1 = XsdYearMonthDuration::new(24); // 2 years
    let d2 = XsdYearMonthDuration::new(6); // 6 months
    let result = ym_duration_subtract(d1, d2);
    assert(result.total_months == 18);
}

#[test]
fn test_ym_duration_multiply() {
    let dur = XsdYearMonthDuration::new(6); // 6 months
    let result = ym_duration_multiply(dur, 3);
    assert(result.total_months == 18);
}

#[test]
fn test_ym_duration_multiply_negative() {
    let dur = XsdYearMonthDuration::new(6);
    let result = ym_duration_multiply(dur, -2);
    assert(result.total_months == -12);
}

#[test]
fn test_ym_duration_divide() {
    let dur = XsdYearMonthDuration::new(24); // 2 years
    let result = ym_duration_divide(dur, 2);
    assert(result.total_months == 12);
}

#[test]
fn test_ym_duration_divide_by_duration() {
    let d1 = XsdYearMonthDuration::new(24); // 2 years
    let d2 = XsdYearMonthDuration::new(6); // 6 months
    let result = ym_duration_divide_by_duration(d1, d2);
    assert(result == 4);
}

#[test]
fn test_ym_duration_negate() {
    let dur = XsdYearMonthDuration::new(12);
    let result = ym_duration_negate(dur);
    assert(result.total_months == -12);
}

#[test]
fn test_ym_duration_negate_negative() {
    let dur = XsdYearMonthDuration::new(-12);
    let result = ym_duration_negate(dur);
    assert(result.total_months == 12);
}

// ============================================================================
// Comparison Tests
// ============================================================================

#[test]
fn test_ym_duration_equal() {
    let d1 = XsdYearMonthDuration::new(12);
    let d2 = XsdYearMonthDuration::new(12);
    let d3 = XsdYearMonthDuration::new(6);
    assert(ym_duration_equal(d1, d2));
    assert(!ym_duration_equal(d1, d3));
}

#[test]
fn test_ym_duration_less_than() {
    let d1 = XsdYearMonthDuration::new(6);
    let d2 = XsdYearMonthDuration::new(12);
    assert(ym_duration_less_than(d1, d2));
    assert(!ym_duration_less_than(d2, d1));
    assert(!ym_duration_less_than(d1, d1));
}

#[test]
fn test_ym_duration_greater_than() {
    let d1 = XsdYearMonthDuration::new(12);
    let d2 = XsdYearMonthDuration::new(6);
    assert(ym_duration_greater_than(d1, d2));
    assert(!ym_duration_greater_than(d2, d1));
    assert(!ym_duration_greater_than(d1, d1));
}

#[test]
fn test_ym_duration_le() {
    let d1 = XsdYearMonthDuration::new(6);
    let d2 = XsdYearMonthDuration::new(12);
    assert(ym_duration_le(d1, d2));
    assert(ym_duration_le(d1, d1));
    assert(!ym_duration_le(d2, d1));
}

#[test]
fn test_ym_duration_ge() {
    let d1 = XsdYearMonthDuration::new(12);
    let d2 = XsdYearMonthDuration::new(6);
    assert(ym_duration_ge(d1, d2));
    assert(ym_duration_ge(d1, d1));
    assert(!ym_duration_ge(d2, d1));
}
