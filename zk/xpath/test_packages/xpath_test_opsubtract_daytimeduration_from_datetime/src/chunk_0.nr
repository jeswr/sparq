//! Real test vectors for op:subtract-dayTimeDuration-from-dateTime (F&O sec. 8.3.18)
//! Signed epoch encoding: pre-1970 results are VALID negative i64 epochs.
//! Python oracle (tz-aware datetime):
//!   from datetime import datetime, timezone, timedelta
//!   def epoch_us(dt): return int((dt - datetime(1970,1,1,tzinfo=timezone.utc)).total_seconds() * 1_000_000)
//! [SONNET-4.6] (sq-rcpdz)

use dep::xpath::{datetime_subtract_duration, duration_from_components, XsdDateTime};

fn signed_epoch_us(e: i64) -> Field {
    (e as u64) as Field
}

#[test]
fn opsubtract_dt_dtd_epoch_minus_one_day_pre1970() {
    // Oracle: 1970-01-01T00:00:00Z - P1D = 1969-12-31T00:00:00Z
    // Python: epoch_us(datetime(1969,12,31,tzinfo=timezone.utc)) -> -86400000000
    let dt = XsdDateTime::new(0);
    let one_day = duration_from_components(false, 1, 0, 0, 0, 0);
    let result = datetime_subtract_duration(dt, one_day);
    assert((result.epoch_microseconds as i64) == -86_400_000_000);
}

#[test]
fn opsubtract_dt_dtd_day2_minus_one_day() {
    // Oracle: 1970-01-02T00:00:00Z - P1D = 1970-01-01T00:00:00Z
    // Python: epoch_us(datetime(1970,1,1,tzinfo=timezone.utc)) -> 0
    let dt = XsdDateTime::new(signed_epoch_us(86_400_000_000));
    let one_day = duration_from_components(false, 1, 0, 0, 0, 0);
    let result = datetime_subtract_duration(dt, one_day);
    assert((result.epoch_microseconds as i64) == 0);
}

#[test]
fn opsubtract_dt_dtd_year_2000_minus_one_day() {
    // Oracle: 2000-01-02T00:00:00Z - P1D = 2000-01-01T00:00:00Z
    // Python: epoch_us(datetime(2000,1,2,tzinfo=timezone.utc)) -> 946771200000000
    //         epoch_us(datetime(2000,1,1,tzinfo=timezone.utc)) -> 946684800000000
    let dt = XsdDateTime::new(signed_epoch_us(946_771_200_000_000));
    let one_day = duration_from_components(false, 1, 0, 0, 0, 0);
    let result = datetime_subtract_duration(dt, one_day);
    assert((result.epoch_microseconds as i64) == 946_684_800_000_000);
}

#[test]
fn opsubtract_dt_dtd_pre1970_minus_one_day_deeper() {
    // Oracle: 1969-12-31T00:00:00Z - P1D = 1969-12-30T00:00:00Z
    // Python: epoch_us(datetime(1969,12,30,tzinfo=timezone.utc)) -> -172800000000
    let dt = XsdDateTime::new(signed_epoch_us(-86_400_000_000));
    let one_day = duration_from_components(false, 1, 0, 0, 0, 0);
    let result = datetime_subtract_duration(dt, one_day);
    assert((result.epoch_microseconds as i64) == -172_800_000_000);
}

#[test]
fn opsubtract_dt_dtd_zero_duration() {
    // Oracle: 1970-01-01T00:00:00Z - PT0S = 1970-01-01T00:00:00Z
    // Python: epoch_us(datetime(1970,1,1,tzinfo=timezone.utc)) -> 0
    let dt = XsdDateTime::new(0);
    let zero = duration_from_components(false, 0, 0, 0, 0, 0);
    let result = datetime_subtract_duration(dt, zero);
    assert((result.epoch_microseconds as i64) == 0);
}

#[test]
fn opsubtract_dt_dtd_pre1970_minus_one_hour() {
    // Oracle: 1969-07-20T21:17:40Z - PT1H = 1969-07-20T20:17:40Z
    // Python: epoch_us(datetime(1969,7,20,21,17,40,tzinfo=timezone.utc)) -> -14179340000000
    //         epoch_us(datetime(1969,7,20,20,17,40,tzinfo=timezone.utc)) -> -14182940000000
    let base_us: i64 = -14_179_340_000_000;
    let dt = XsdDateTime::new(signed_epoch_us(base_us));
    let one_hour = duration_from_components(false, 0, 1, 0, 0, 0);
    let result = datetime_subtract_duration(dt, one_hour);
    assert((result.epoch_microseconds as i64) == -14_182_940_000_000);
}

#[test]
fn opsubtract_dt_dtd_crosses_1970_boundary() {
    // Oracle: 1970-01-01T12:00:00Z - P1D = 1969-12-31T12:00:00Z
    // Python: epoch_us(datetime(1970,1,1,12,tzinfo=timezone.utc)) -> 43200000000
    //         epoch_us(datetime(1969,12,31,12,tzinfo=timezone.utc)) -> -43200000000
    let dt = XsdDateTime::new(signed_epoch_us(43_200_000_000));
    let one_day = duration_from_components(false, 1, 0, 0, 0, 0);
    let result = datetime_subtract_duration(dt, one_day);
    assert((result.epoch_microseconds as i64) == -43_200_000_000);
}
