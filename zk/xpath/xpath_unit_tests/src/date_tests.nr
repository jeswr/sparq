//! Unit tests for date functions

use dep::xpath::{
    date_equal, date_from_components, date_from_epoch_days, date_greater_than, date_less_than,
    day_from_date, month_from_date, year_from_date,
};

#[test]
fn test_date_from_components_epoch() {
    // 1970-01-01 should be epoch day 0
    let date = date_from_components(1970, 1, 1);
    assert(date.epoch_days == 0);
}

#[test]
fn test_date_component_extraction() {
    // Test 2000-05-15
    let date = date_from_components(2000, 5, 15);
    assert(year_from_date(date) == 2000);
    assert(month_from_date(date) == 5);
    assert(day_from_date(date) == 15);
}

#[test]
fn test_date_equality() {
    let date1 = date_from_components(2000, 1, 1);
    let date2 = date_from_components(2000, 1, 1);
    assert(date_equal(date1, date2));
}

#[test]
fn test_date_less_than() {
    let date1 = date_from_components(2000, 1, 1);
    let date2 = date_from_components(2000, 1, 2);
    assert(date_less_than(date1, date2));
}

#[test]
fn test_date_greater_than() {
    let date1 = date_from_components(2000, 1, 2);
    let date2 = date_from_components(2000, 1, 1);
    assert(date_greater_than(date1, date2));
}

#[test]
fn test_date_roundtrip() {
    // Test that converting to epoch days and back works
    let date1 = date_from_components(1995, 6, 15);
    let days = date1.epoch_days;
    let date2 = date_from_epoch_days(days);

    assert(year_from_date(date2) == 1995);
    assert(month_from_date(date2) == 6);
    assert(day_from_date(date2) == 15);
}

#[test]
fn test_date_year_2000() {
    // Leap year test
    let date = date_from_components(2000, 2, 29);
    assert(year_from_date(date) == 2000);
    assert(month_from_date(date) == 2);
    assert(day_from_date(date) == 29);
}
