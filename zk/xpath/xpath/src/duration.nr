//! Duration functions for XPath/SPARQL
//!
//! Implements xsd:dayTimeDuration operations.
//! Duration values are stored as signed microseconds in a Field.
//!
//! XPath operators implemented:
//! - op:add-dayTimeDurations, op:subtract-dayTimeDurations
//! - op:multiply-dayTimeDuration, op:divide-dayTimeDuration
//! - op:add-dayTimeDuration-to-dateTime, op:subtract-dayTimeDuration-from-dateTime
//! - op:dayTimeDuration-equal, op:dayTimeDuration-less-than, op:dayTimeDuration-greater-than
//! XPath functions implemented (generic dispatch on the duration subtype):
//! - fn:years-from-duration (xs:dayTimeDuration -> 0 per F&O sec. 8.4.3,
//!   xs:yearMonthDuration -> real years)
//! - fn:months-from-duration (xs:dayTimeDuration -> 0 per F&O sec. 8.4.4,
//!   xs:yearMonthDuration -> real months)

use crate::types::{XsdDateTime, XsdDayTimeDuration};
use crate::year_month_duration::XsdYearMonthDuration;

// ============================================================================
// Time Constants (as i64 for signed arithmetic)
// ============================================================================

/// Microseconds per second
global MICROS_PER_SECOND: i64 = 1_000_000;

/// Microseconds per minute
global MICROS_PER_MINUTE: i64 = 60_000_000;

/// Microseconds per hour
global MICROS_PER_HOUR: i64 = 3_600_000_000;

/// Microseconds per day
global MICROS_PER_DAY: i64 = 86_400_000_000;

// ============================================================================
// Duration Construction Functions
// ============================================================================

/// Create a duration from total microseconds (can be negative)
pub fn duration_from_microseconds(micros: i64) -> XsdDayTimeDuration {
    XsdDayTimeDuration::from_signed_micros(micros)
}

/// Create a duration from components
/// days, hours, minutes, seconds can be any values; they will be normalized
pub fn duration_from_components(
    negative: bool,
    days: u32,
    hours: u32,
    minutes: u32,
    seconds: u32,
    microseconds: u32,
) -> XsdDayTimeDuration {
    let total: i64 = (days as i64) * MICROS_PER_DAY
        + (hours as i64) * MICROS_PER_HOUR
        + (minutes as i64) * MICROS_PER_MINUTE
        + (seconds as i64) * MICROS_PER_SECOND
        + microseconds as i64;

    let signed_total = if negative { -total } else { total };
    XsdDayTimeDuration::from_signed_micros(signed_total)
}

/// Create a zero duration
pub fn duration_zero() -> XsdDayTimeDuration {
    XsdDayTimeDuration::zero()
}

// ============================================================================
// Duration Component Extraction
// ============================================================================

/// Extract total microseconds from duration (signed)
pub fn duration_to_microseconds(dur: XsdDayTimeDuration) -> i64 {
    dur.to_signed_micros()
}

/// Check if duration is negative
pub fn duration_is_negative(dur: XsdDayTimeDuration) -> bool {
    dur.to_signed_micros() < 0
}

/// Extract days component (absolute value)
pub fn days_from_duration(dur: XsdDayTimeDuration) -> u32 {
    let micros = dur.to_signed_micros();
    let abs_micros: i64 = if micros < 0 { -micros } else { micros };
    (abs_micros / MICROS_PER_DAY) as u32
}

/// Extract hours component (0-23, absolute value)
pub fn hours_from_duration(dur: XsdDayTimeDuration) -> u8 {
    let micros = dur.to_signed_micros();
    let abs_micros: i64 = if micros < 0 { -micros } else { micros };
    let day_remainder = abs_micros % MICROS_PER_DAY;
    (day_remainder / MICROS_PER_HOUR) as u8
}

/// Extract minutes component (0-59, absolute value)
pub fn minutes_from_duration(dur: XsdDayTimeDuration) -> u8 {
    let micros = dur.to_signed_micros();
    let abs_micros: i64 = if micros < 0 { -micros } else { micros };
    let hour_remainder = abs_micros % MICROS_PER_HOUR;
    (hour_remainder / MICROS_PER_MINUTE) as u8
}

/// Extract seconds component (0-59, absolute value)
pub fn seconds_from_duration(dur: XsdDayTimeDuration) -> u8 {
    let micros = dur.to_signed_micros();
    let abs_micros: i64 = if micros < 0 { -micros } else { micros };
    let minute_remainder = abs_micros % MICROS_PER_MINUTE;
    (minute_remainder / MICROS_PER_SECOND) as u8
}

/// Dispatch trait for fn:years-from-duration / fn:months-from-duration across
/// the two XSD duration subtypes (F&O sec. 8.4.3 / sec. 8.4.4). The generated
/// qt3tests packages call the accessors under their canonical XPath names with
/// either subtype, so dispatch follows the codebase's generic trait-bound
/// pattern (see comparison.nr). [SONNET-4.6] (sq-u91ki)
pub trait YearMonthComponents {
    /// Years component of the duration (F&O sec. 8.4.3)
    fn years_component(self) -> i32;
    /// Months component of the duration (F&O sec. 8.4.4)
    fn months_component(self) -> i32;
}

/// F&O sec. 8.4.3 / sec. 8.4.4: an xs:dayTimeDuration carries no years or
/// months component, so both accessors return the xs:integer 0.
/// [SONNET-4.6] (sq-u91ki)
impl YearMonthComponents for XsdDayTimeDuration {
    fn years_component(self) -> i32 {
        let _ = self;
        0
    }
    fn months_component(self) -> i32 {
        let _ = self;
        0
    }
}

/// xs:yearMonthDuration keeps its real component values -- delegates to the
/// concrete accessors in year_month_duration.nr. [SONNET-4.6] (sq-u91ki)
impl YearMonthComponents for XsdYearMonthDuration {
    fn years_component(self) -> i32 {
        crate::year_month_duration::years_from_duration(self)
    }
    fn months_component(self) -> i32 {
        crate::year_month_duration::months_from_duration(self)
    }
}

/// fn:years-from-duration -- dispatches on the duration subtype:
/// xs:dayTimeDuration -> 0 (F&O sec. 8.4.3), xs:yearMonthDuration -> real years.
pub fn years_from_duration<T>(dur: T) -> i32
where
    T: YearMonthComponents,
{
    dur.years_component()
}

/// fn:months-from-duration -- dispatches on the duration subtype:
/// xs:dayTimeDuration -> 0 (F&O sec. 8.4.4), xs:yearMonthDuration -> real months.
pub fn months_from_duration<T>(dur: T) -> i32
where
    T: YearMonthComponents,
{
    dur.months_component()
}

// ============================================================================
// Duration Arithmetic
// ============================================================================

/// op:add-dayTimeDurations
/// Adds two durations
pub fn duration_add(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> XsdDayTimeDuration {
    let sum = a.to_signed_micros() + b.to_signed_micros();
    XsdDayTimeDuration::from_signed_micros(sum)
}

/// op:subtract-dayTimeDurations
/// Subtracts second duration from first
pub fn duration_subtract(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> XsdDayTimeDuration {
    let diff = a.to_signed_micros() - b.to_signed_micros();
    XsdDayTimeDuration::from_signed_micros(diff)
}

/// op:multiply-dayTimeDuration
/// Multiplies a duration by a scalar
pub fn duration_multiply(dur: XsdDayTimeDuration, factor: i64) -> XsdDayTimeDuration {
    let result = dur.to_signed_micros() * factor;
    XsdDayTimeDuration::from_signed_micros(result)
}

/// op:divide-dayTimeDuration
/// Divides a duration by a scalar
pub fn duration_divide(dur: XsdDayTimeDuration, divisor: i64) -> XsdDayTimeDuration {
    assert(divisor != 0, "Division by zero");
    let result = dur.to_signed_micros() / divisor;
    XsdDayTimeDuration::from_signed_micros(result)
}

/// op:divide-dayTimeDuration-by-dayTimeDuration
/// Divides two durations, returning a ratio as i64
pub fn duration_divide_by_duration(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> i64 {
    let b_micros = b.to_signed_micros();
    assert(b_micros != 0, "Division by zero duration");
    a.to_signed_micros() / b_micros
}

/// Negates a duration
pub fn duration_negate(dur: XsdDayTimeDuration) -> XsdDayTimeDuration {
    XsdDayTimeDuration::from_signed_micros(-dur.to_signed_micros())
}

// ============================================================================
// DateTime + Duration Arithmetic
// ============================================================================

/// op:add-dayTimeDuration-to-dateTime
/// Adds a duration to a dateTime
///
/// SIGNED EPOCH ENCODING (sq-3x7dl.7): the result may be a valid pre-1970
/// (negative) instant -- e.g. 1970-01-01T00:00:00Z + (-P1D) = 1969-12-31T00:00:00Z
/// is a perfectly valid xs:dateTime. We decode the operand epoch to i64, add the
/// signed duration, and store the two's-complement 64-bit pattern of the signed
/// result (`(e as u64) as Field`), exactly like datetime.nr's construction path.
/// The old `assert(result >= 0, ...)` aborted the circuit on every valid
/// pre-epoch op, so it is dropped. [OPUS-4.8] (sq-3x7dl.18)
pub fn datetime_add_duration(dt: XsdDateTime, dur: XsdDayTimeDuration) -> XsdDateTime {
    let dt_micros: i64 = dt.epoch_microseconds as i64;
    let dur_micros = dur.to_signed_micros();
    let result: i64 = dt_micros + dur_micros;
    XsdDateTime::new((result as u64) as Field)
}

/// op:subtract-dayTimeDuration-from-dateTime
/// Subtracts a duration from a dateTime
///
/// See `datetime_add_duration`: the difference may be a valid negative epoch, so
/// we encode the signed result via `(result as u64) as Field` and drop the old
/// non-negativity assert. [OPUS-4.8] (sq-3x7dl.18)
pub fn datetime_subtract_duration(dt: XsdDateTime, dur: XsdDayTimeDuration) -> XsdDateTime {
    let dt_micros: i64 = dt.epoch_microseconds as i64;
    let dur_micros = dur.to_signed_micros();
    let result: i64 = dt_micros - dur_micros;
    XsdDateTime::new((result as u64) as Field)
}

/// op:subtract-dateTimes
/// Returns the duration between two dateTimes (a - b)
pub fn datetime_difference(a: XsdDateTime, b: XsdDateTime) -> XsdDayTimeDuration {
    let a_micros: i64 = a.epoch_microseconds as i64;
    let b_micros: i64 = b.epoch_microseconds as i64;
    XsdDayTimeDuration::from_signed_micros(a_micros - b_micros)
}

// ============================================================================
// Duration Comparison
// ============================================================================

/// op:dayTimeDuration-equal
pub fn duration_equal(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> bool {
    a.to_signed_micros() == b.to_signed_micros()
}

/// op:dayTimeDuration-less-than
pub fn duration_less_than(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> bool {
    a.to_signed_micros() < b.to_signed_micros()
}

/// op:dayTimeDuration-greater-than
pub fn duration_greater_than(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> bool {
    a.to_signed_micros() > b.to_signed_micros()
}

/// op:dayTimeDuration-less-than-or-equal
pub fn duration_le(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> bool {
    a.to_signed_micros() <= b.to_signed_micros()
}

/// op:dayTimeDuration-greater-than-or-equal
pub fn duration_ge(a: XsdDayTimeDuration, b: XsdDayTimeDuration) -> bool {
    a.to_signed_micros() >= b.to_signed_micros()
}

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_duration_from_components() {
    // 1 day, 2 hours, 30 minutes, 45 seconds
    let dur = duration_from_components(false, 1, 2, 30, 45, 0);
    assert(days_from_duration(dur) == 1);
    assert(hours_from_duration(dur) == 2);
    assert(minutes_from_duration(dur) == 30);
    assert(seconds_from_duration(dur) == 45);
    assert(duration_is_negative(dur) == false);
}

#[test]
fn test_duration_negative() {
    let dur = duration_from_components(true, 1, 0, 0, 0, 0);
    assert(duration_is_negative(dur) == true);
    assert(days_from_duration(dur) == 1);
}

#[test]
fn test_duration_arithmetic() {
    let d1 = duration_from_components(false, 1, 0, 0, 0, 0); // 1 day
    let d2 = duration_from_components(false, 0, 12, 0, 0, 0); // 12 hours

    // Add: 1 day + 12 hours = 1.5 days
    let sum = duration_add(d1, d2);
    assert(days_from_duration(sum) == 1);
    assert(hours_from_duration(sum) == 12);

    // Subtract: 1 day - 12 hours = 12 hours
    let diff = duration_subtract(d1, d2);
    assert(days_from_duration(diff) == 0);
    assert(hours_from_duration(diff) == 12);

    // Multiply: 12 hours * 2 = 1 day
    let product = duration_multiply(d2, 2);
    assert(days_from_duration(product) == 1);
    assert(hours_from_duration(product) == 0);

    // Divide: 1 day / 2 = 12 hours
    let quotient = duration_divide(d1, 2);
    assert(days_from_duration(quotient) == 0);
    assert(hours_from_duration(quotient) == 12);
}

#[test]
fn test_datetime_duration_arithmetic() {
    // Start at Unix epoch
    let dt = XsdDateTime::new(0);
    let one_day = duration_from_components(false, 1, 0, 0, 0, 0);

    // Add one day
    let dt_plus = datetime_add_duration(dt, one_day);
    let expected_micros: Field = 86_400_000_000;
    assert(dt_plus.epoch_microseconds == expected_micros);

    // Subtract back
    let dt_back = datetime_subtract_duration(dt_plus, one_day);
    assert(dt_back.epoch_microseconds == 0);
}

#[test]
fn test_datetime_difference() {
    let dt1 = XsdDateTime::new(0);
    let dt2_micros: Field = 86_400_000_000; // 1 day in microseconds
    let dt2 = XsdDateTime::new(dt2_micros);

    let diff = datetime_difference(dt2, dt1);
    assert(days_from_duration(diff) == 1);
    assert(duration_is_negative(diff) == false);

    let diff_neg = datetime_difference(dt1, dt2);
    assert(days_from_duration(diff_neg) == 1);
    assert(duration_is_negative(diff_neg) == true);
}

#[test]
fn test_duration_comparison() {
    let d1 = duration_from_components(false, 1, 0, 0, 0, 0);
    let d2 = duration_from_components(false, 2, 0, 0, 0, 0);
    let d3 = duration_from_components(false, 1, 0, 0, 0, 0);

    assert(duration_equal(d1, d3) == true);
    assert(duration_equal(d1, d2) == false);

    assert(duration_less_than(d1, d2) == true);
    assert(duration_less_than(d2, d1) == false);

    assert(duration_greater_than(d2, d1) == true);
    assert(duration_greater_than(d1, d2) == false);
}

// ============================================================================
// Signed-epoch (pre-1970) DateTime +/- Duration tests (sq-3x7dl.18) [OPUS-4.8]
//
// The signed-i64 epoch encoding (sq-3x7dl.7) makes pre-1970 instants valid
// NEGATIVE epochs, so datetime_add_duration / datetime_subtract_duration must
// NOT assert `result >= 0`. Every expected epoch below is quoted from an
// INDEPENDENT timezone-aware Python oracle (datetime + timedelta):
//   (dt - datetime(1970,1,1, tzinfo=timezone.utc)) // timedelta(microseconds=1)
// ============================================================================

/// Encode a signed i64 epoch as the canonical Field bit pattern (test helper,
/// mirrors datetime.nr::signed_epoch).
fn signed_epoch_us(e: i64) -> Field {
    (e as u64) as Field
}

#[test]
fn test_datetime_subtract_duration_epoch_minus_one_day() {
    // (a) Oracle: 1970-01-01T00:00:00Z - P1D = 1969-12-31T00:00:00Z, epoch_us=-86400000000
    let epoch = XsdDateTime::new(0);
    let one_day = duration_from_components(false, 1, 0, 0, 0, 0);
    let result = datetime_subtract_duration(epoch, one_day);
    assert((result.epoch_microseconds as i64) == -86_400_000_000);

    // Adding a NEGATIVE duration lands on the same negative epoch.
    let neg_one_day = duration_from_components(true, 1, 0, 0, 0, 0);
    let via_add = datetime_add_duration(epoch, neg_one_day);
    assert((via_add.epoch_microseconds as i64) == -86_400_000_000);
}

#[test]
fn test_datetime_negative_epoch_stays_negative() {
    // (b) base 1969-07-20T20:17:40Z, epoch_us=-14182940000000
    let base = XsdDateTime::new(signed_epoch_us(-14_182_940_000_000));

    // minus P2D stays negative. Oracle: 1969-07-18T20:17:40Z, epoch_us=-14355740000000
    let two_days = duration_from_components(false, 2, 0, 0, 0, 0);
    let minus = datetime_subtract_duration(base, two_days);
    assert((minus.epoch_microseconds as i64) == -14_355_740_000_000);

    // plus PT1H stays negative. Oracle: 1969-07-20T21:17:40Z, epoch_us=-14179340000000
    let one_hour = duration_from_components(false, 0, 1, 0, 0, 0);
    let plus = datetime_add_duration(base, one_hour);
    assert((plus.epoch_microseconds as i64) == -14_179_340_000_000);
}

#[test]
fn test_datetime_negative_epoch_crosses_back_over_1970() {
    // (c) base 1969-12-31T12:00:00Z, epoch_us=-43200000000
    let base = XsdDateTime::new(signed_epoch_us(-43_200_000_000));
    // plus P1D crosses forward to 1970-01-01T12:00:00Z, epoch_us=43200000000 (positive)
    let one_day = duration_from_components(false, 1, 0, 0, 0, 0);
    let crossed = datetime_add_duration(base, one_day);
    assert((crossed.epoch_microseconds as i64) == 43_200_000_000);
    // The positive result's raw Field equals the plain unsigned value too.
    assert(crossed.epoch_microseconds == 43_200_000_000);
}

// ============================================================================
// DayTimeDuration overloads for fn:years-from-duration / fn:months-from-duration
// (sq-u91ki) [SONNET-4.6]
//
// F&O sec. 8.4.3 (fn:years-from-duration) and sec. 8.4.4 (fn:months-from-duration):
// when the argument is xs:dayTimeDuration, both accessors return the xs:integer 0
// because a dayTimeDuration carries no months or years component.
//
// Oracle: Python reference
//   import isodate; d = isodate.parse_duration("P1D")
//   # d is datetime.timedelta(days=1); no years/months attribute -> 0
// ============================================================================

#[test]
fn test_years_from_duration_dt_positive() {
    // Oracle: xs:dayTimeDuration("P1D") has no years component -> fn:years-from-duration = 0
    // (F&O sec. 8.4.3)
    let one_day = duration_from_components(false, 1, 0, 0, 0, 0);
    assert(years_from_duration(one_day) == 0);
}

#[test]
fn test_years_from_duration_dt_negative() {
    // Oracle: xs:dayTimeDuration("-P365D") has no years component -> fn:years-from-duration = 0
    // (F&O sec. 8.4.3)
    let neg_year = duration_from_components(true, 365, 0, 0, 0, 0);
    assert(years_from_duration(neg_year) == 0);
}

#[test]
fn test_months_from_duration_dt_positive() {
    // Oracle: xs:dayTimeDuration("P30D") has no months component -> fn:months-from-duration = 0
    // (F&O sec. 8.4.4)
    let thirty_days = duration_from_components(false, 30, 0, 0, 0, 0);
    assert(months_from_duration(thirty_days) == 0);
}

#[test]
fn test_months_from_duration_dt_negative() {
    // Oracle: xs:dayTimeDuration("-P30D") has no months component -> fn:months-from-duration = 0
    // (F&O sec. 8.4.4)
    let neg_thirty_days = duration_from_components(true, 30, 0, 0, 0, 0);
    assert(months_from_duration(neg_thirty_days) == 0);
}

#[test]
fn test_years_months_dt_zero_duration() {
    // Regression: zero dayTimeDuration -> both return 0 (degenerate case, F&O sec. 8.4.3/8.4.4)
    let zero = duration_zero();
    assert(years_from_duration(zero) == 0);
    assert(months_from_duration(zero) == 0);
}

#[test]
fn test_years_months_from_duration_ym_regression() {
    // Regression guard: the yearMonthDuration path through the SAME generic
    // accessors must still return real component values (not the dayTimeDuration 0).
    // Oracle: P2Y3M = 27 months -> years 2, months 3; -P1Y6M = -18 months ->
    // years -1, months -6 (i32 truncating division, matching
    // year_month_duration.nr::test_ym_duration_components).
    let ym = XsdYearMonthDuration::new(27);
    assert(years_from_duration(ym) == 2);
    assert(months_from_duration(ym) == 3);

    let ym_neg = XsdYearMonthDuration::new(-18);
    assert(years_from_duration(ym_neg) == -1);
    assert(months_from_duration(ym_neg) == -6);
}
