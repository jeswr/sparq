//! `xsd:dateTime` / `xsd:date` / `xsd:duration` value comparison, independent of sparq. [OPUS-4.8]
//! sq-qcnn.4
//!
//! Everything here is built from a proleptic-Gregorian day count (Howard Hinnant's `days_from_civil`)
//! plus arbitrary-precision arithmetic ([`bigdecimal`]) — no `chrono`/`time` and, crucially, no sparq
//! temporal code, so a bug in the engine's own temporal handling cannot cancel out of the differential.
//!
//! The comparisons are **three-valued** ([`TemporalOrder`]) because XSD temporal order is *partial*:
//!
//! * A timezone-less `dateTime` compared with a timezoned one falls in the XPath **±14h indeterminate
//!   window** — an implicit timezone could place it either side, so the result may be `Indeterminate`
//!   rather than a definite order. This is a legitimate cross-engine divergence source (engines pick
//!   different implicit timezones), so the harness must triage it, not auto-fail.
//! * `xsd:duration` is only *partially* ordered — `P1M` (one month) and `P30D` (30 days) are
//!   **incomparable** — so [`duration_compare`] returns `Indeterminate` whenever the month and second
//!   components move in opposite directions (the conservative XSD monotone order).

use std::cmp::Ordering;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;

use crate::term::XSD;

/// A three-valued temporal order: XSD temporal comparison is partial, so a definite `<`/`=`/`>` is not
/// always available (the ±14h dateTime window; the duration month-vs-second partial order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalOrder {
    Less,
    Equal,
    Greater,
    /// No definite order (the ±14h window, or an incomparable duration pair). Route to triage.
    Indeterminate,
}

/// An `xsd:dateTime`/`xsd:date` value: seconds from the Unix epoch, plus whether the lexical carried a
/// timezone. A timezoned value's `seconds` is normalised to **UTC**; a timezone-less value's `seconds`
/// is its **local** wall-clock reading (compared under the ±14h rule).
#[derive(Debug, Clone)]
pub struct DateTimeValue {
    seconds: BigDecimal,
    has_tz: bool,
}

impl DateTimeValue {
    /// A stable multiset-keying string. Timezoned values key on their UTC instant (so two lexical
    /// spellings of the same instant collide); timezone-less values key on their local reading under a
    /// distinct tag (so a timezone-less value never collides with a timezoned one — the ±14h
    /// indeterminacy is preserved as "not equal" for keying, the conservative choice).
    pub(crate) fn canonical_key(&self) -> String {
        let tag = if self.has_tz { 'Z' } else { 'L' };
        format!("{}{}", tag, self.seconds.normalized())
    }
}

/// An `xsd:duration` value decomposed into its two independent axes: `months` (year+month) and
/// `seconds` (day+time). They are compared component-wise (the XSD monotone partial order).
#[derive(Debug, Clone)]
pub struct Duration {
    months: BigInt,
    seconds: BigDecimal,
}

impl Duration {
    /// A stable multiset-keying string: two durations with equal month and second components key the
    /// same (so `P1Y` and `P12M` collide by value), an incomparable pair keys differently.
    pub(crate) fn canonical_key(&self) -> String {
        format!("{}\u{1f}{}", self.months, self.seconds.normalized())
    }
}

/// Days from 1970-01-01 for a proleptic-Gregorian date, using astronomical year numbering (year 0
/// exists). Howard Hinnant's `days_from_civil`; valid across the whole integer year range.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = if m > 2 { m - 3 } else { m + 9 }; // [0, 11]
    let doy = (153 * mp + 2) / 5 + d - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146097 + doe - 719468
}

/// Parse an `xsd:dateTime` or `xsd:date` `(lexical, datatype-IRI)` into a comparable value.
pub fn parse_datetime(lexical: &str, datatype: &str) -> Option<DateTimeValue> {
    let local = datatype.strip_prefix(XSD)?;
    let has_time = match local {
        "dateTime" => true,
        "date" => false,
        _ => return None,
    };
    let s = lexical.trim();
    // Leading '-' is a negative (BCE) year sign; strip it before the internal date separators.
    let (neg_year, rest) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };

    let (date_str, time_str, tz) = if has_time {
        let (dpart, tpart) = rest.split_once('T')?;
        let (tstr, tz) = split_timezone(tpart)?;
        (dpart, Some(tstr), tz)
    } else {
        let (dstr, tz) = split_date_timezone(rest)?;
        (dstr, None, tz)
    };

    let mut dparts = date_str.split('-');
    let year: i64 = dparts.next()?.parse().ok()?;
    let month: i64 = dparts.next()?.parse().ok()?;
    let day: i64 = dparts.next()?.parse().ok()?;
    if dparts.next().is_some() {
        return None;
    }
    let signed_year = if neg_year { -year } else { year };
    let days = days_from_civil(signed_year, month, day);

    let (hh, mm, frac) = match time_str {
        Some(t) => parse_time(t)?,
        None => (0i64, 0i64, BigDecimal::from(0)),
    };

    let base = days * 86_400 + hh * 3_600 + mm * 60;
    let mut seconds = BigDecimal::from(base) + frac;
    if let Some(offset) = tz {
        // local = UTC + offset  =>  UTC = local - offset.
        seconds -= BigDecimal::from(offset);
    }
    Some(DateTimeValue {
        seconds,
        has_tz: tz.is_some(),
    })
}

/// Split a `"hh:mm:ss[.frac]"` (dateTime time part) trailing timezone off; returns `(time, offset)`
/// where `offset` is `Some(seconds)` for a present timezone (`Z` -> 0) and `None` otherwise.
fn split_timezone(tpart: &str) -> Option<(&str, Option<i64>)> {
    if let Some(t) = tpart.strip_suffix('Z') {
        return Some((t, Some(0)));
    }
    // The time "hh:mm:ss.fff" contains no '+' or '-', so the first one marks the timezone.
    if let Some(pos) = tpart.find(['+', '-']) {
        let (t, tzs) = tpart.split_at(pos);
        return Some((t, Some(parse_tz_offset(tzs)?)));
    }
    Some((tpart, None))
}

/// Split a trailing timezone off an `xsd:date` `"yyyy-mm-dd[tz]"` body (the year sign already removed,
/// so the date contains exactly two '-' separators; a third marks a negative timezone).
fn split_date_timezone(rest: &str) -> Option<(&str, Option<i64>)> {
    if let Some(d) = rest.strip_suffix('Z') {
        return Some((d, Some(0)));
    }
    if let Some(pos) = rest.find('+') {
        return Some((&rest[..pos], Some(parse_tz_offset(&rest[pos..])?)));
    }
    // Count '-' separators: 2 => no tz, 3 => the last group is a negative tz.
    let dash_count = rest.matches('-').count();
    if dash_count <= 2 {
        return Some((rest, None));
    }
    let pos = rest.rmatch_indices('-').next()?.0;
    Some((&rest[..pos], Some(parse_tz_offset(&rest[pos..])?)))
}

/// Parse a `"±hh:mm"` timezone into signed seconds.
fn parse_tz_offset(tz: &str) -> Option<i64> {
    let (sign, body) = match tz.strip_prefix('+') {
        Some(b) => (1, b),
        None => (-1, tz.strip_prefix('-')?),
    };
    let (h, m) = body.split_once(':')?;
    let h: i64 = h.parse().ok()?;
    let m: i64 = m.parse().ok()?;
    Some(sign * (h * 3_600 + m * 60))
}

/// Parse a `"hh:mm:ss[.frac]"` time into `(hours, minutes, seconds-with-fraction)`.
fn parse_time(t: &str) -> Option<(i64, i64, BigDecimal)> {
    let mut parts = t.split(':');
    let hh: i64 = parts.next()?.parse().ok()?;
    let mm: i64 = parts.next()?.parse().ok()?;
    let ss = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let frac = BigDecimal::from_str(ss).ok()?;
    Some((hh, mm, frac))
}

/// Compare two `xsd:dateTime`/`date` values. Two timezoned (or two timezone-less) values compare
/// definitely; a timezoned-vs-timezone-less pair uses the XPath ±14h rule and may be `Indeterminate`.
pub fn dt_compare(a: &DateTimeValue, b: &DateTimeValue) -> TemporalOrder {
    match (a.has_tz, b.has_tz) {
        (true, true) | (false, false) => cmp_to_order(a.seconds.cmp(&b.seconds)),
        (true, false) => tz_vs_local(&a.seconds, &b.seconds),
        (false, true) => flip(tz_vs_local(&b.seconds, &a.seconds)),
    }
}

/// The ±14h rule: `u` is a timezoned value's UTC instant, `l` a timezone-less local reading whose true
/// instant could be anywhere in `[l - 14h, l + 14h]`. Definite only outside that window.
fn tz_vs_local(u: &BigDecimal, l: &BigDecimal) -> TemporalOrder {
    let fourteen_h = BigDecimal::from(14 * 3_600);
    if *u < l - &fourteen_h {
        TemporalOrder::Less
    } else if *u > l + &fourteen_h {
        TemporalOrder::Greater
    } else {
        TemporalOrder::Indeterminate
    }
}

fn flip(o: TemporalOrder) -> TemporalOrder {
    match o {
        TemporalOrder::Less => TemporalOrder::Greater,
        TemporalOrder::Greater => TemporalOrder::Less,
        other => other,
    }
}

fn cmp_to_order(o: Ordering) -> TemporalOrder {
    match o {
        Ordering::Less => TemporalOrder::Less,
        Ordering::Equal => TemporalOrder::Equal,
        Ordering::Greater => TemporalOrder::Greater,
    }
}

/// Parse an `xsd:duration` / `xsd:yearMonthDuration` / `xsd:dayTimeDuration` into a comparable value.
pub fn parse_duration(lexical: &str, datatype: &str) -> Option<Duration> {
    let local = datatype.strip_prefix(XSD)?;
    if !matches!(
        local,
        "duration" | "yearMonthDuration" | "dayTimeDuration"
    ) {
        return None;
    }
    let s = lexical.trim();
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let s = s.strip_prefix('P')?;
    let (dpart, tpart) = match s.split_once('T') {
        Some((d, t)) => (d, Some(t)),
        None => (s, None),
    };

    let mut months = BigInt::from(0);
    let mut seconds = BigDecimal::from(0);
    let mut any = false;
    read_components(dpart, false, &mut months, &mut seconds, &mut any)?;
    if let Some(t) = tpart {
        // A 'T' with nothing after it is invalid.
        if t.is_empty() {
            return None;
        }
        read_components(t, true, &mut months, &mut seconds, &mut any)?;
    }
    if !any {
        return None;
    }
    if neg {
        months = -months;
        seconds = -seconds;
    }
    Some(Duration { months, seconds })
}

/// Read a run of `<number><unit>` components into the month/second accumulators. `is_time` selects the
/// time-side unit set (`H`/`M`/`S`) versus the date-side set (`Y`/`M`/`D`).
fn read_components(
    part: &str,
    is_time: bool,
    months: &mut BigInt,
    seconds: &mut BigDecimal,
    any: &mut bool,
) -> Option<()> {
    let mut cur = part;
    while !cur.is_empty() {
        let numend = cur.find(|c: char| !c.is_ascii_digit() && c != '.')?;
        let (num, rest) = cur.split_at(numend);
        let unit = rest.chars().next()?;
        cur = &rest[unit.len_utf8()..];
        if num.is_empty() {
            return None;
        }
        match (is_time, unit) {
            (false, 'Y') => *months += BigInt::from_str(num).ok()? * BigInt::from(12),
            (false, 'M') => *months += BigInt::from_str(num).ok()?,
            (false, 'D') => {
                *seconds = seconds.clone() + BigDecimal::from_str(num).ok()? * BigDecimal::from(86_400)
            }
            (true, 'H') => {
                *seconds = seconds.clone() + BigDecimal::from_str(num).ok()? * BigDecimal::from(3_600)
            }
            (true, 'M') => {
                *seconds = seconds.clone() + BigDecimal::from_str(num).ok()? * BigDecimal::from(60)
            }
            (true, 'S') => *seconds = seconds.clone() + BigDecimal::from_str(num).ok()?,
            _ => return None,
        }
        *any = true;
    }
    Some(())
}

/// Compare two `xsd:duration` values under the XSD monotone partial order: definite only when the month
/// and second components agree in direction; `Indeterminate` when they diverge (`P1M` vs `P30D`).
pub fn duration_compare(a: &Duration, b: &Duration) -> TemporalOrder {
    let mc = a.months.cmp(&b.months);
    let sc = a.seconds.cmp(&b.seconds);
    use Ordering::{Equal, Greater, Less};
    match (mc, sc) {
        (Equal, Equal) => TemporalOrder::Equal,
        (Less, Less) | (Less, Equal) | (Equal, Less) => TemporalOrder::Less,
        (Greater, Greater) | (Greater, Equal) | (Equal, Greater) => TemporalOrder::Greater,
        _ => TemporalOrder::Indeterminate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: &str = "http://www.w3.org/2001/XMLSchema#dateTime";
    const DATE: &str = "http://www.w3.org/2001/XMLSchema#date";
    const DUR: &str = "http://www.w3.org/2001/XMLSchema#duration";
    const YM: &str = "http://www.w3.org/2001/XMLSchema#yearMonthDuration";
    const DAYT: &str = "http://www.w3.org/2001/XMLSchema#dayTimeDuration";

    fn dt(s: &str) -> DateTimeValue {
        parse_datetime(s, DT).unwrap()
    }

    #[test]
    fn parse_datetime_forms() {
        // Z, offset, timezone-less, fractional seconds, xsd:date, negative year.
        assert!(parse_datetime("2020-01-01T00:00:00Z", DT).is_some());
        assert!(parse_datetime("2020-01-01T00:00:00+01:00", DT).is_some());
        assert!(parse_datetime("2020-01-01T00:00:00", DT).is_some());
        assert!(parse_datetime("2020-01-01T00:00:00.500Z", DT).is_some());
        assert!(parse_datetime("2020-01-01", DATE).is_some());
        assert!(parse_datetime("2020-01-01-05:00", DATE).is_some());
        assert!(parse_datetime("-0044-03-15T00:00:00Z", DT).is_some());
        // wrong datatype / malformed.
        assert!(parse_datetime("2020-01-01T00:00:00Z", DUR).is_none());
        assert!(parse_datetime("not-a-date", DT).is_none());
    }

    #[test]
    fn dt_compare_timezone_equivalence() {
        // Same instant, different lexical -> Equal.
        assert_eq!(
            dt_compare(&dt("2020-01-01T13:00:00Z"), &dt("2020-01-01T14:00:00+01:00")),
            TemporalOrder::Equal
        );
        // Different instant -> definite order.
        assert_eq!(
            dt_compare(&dt("2020-01-01T13:00:00Z"), &dt("2020-01-01T13:00:00-05:00")),
            TemporalOrder::Less
        );
        // Timezone-less vs timezoned, well outside ±14h -> definite.
        assert_eq!(
            dt_compare(
                &parse_datetime("2020-01-05T00:00:00", DT).unwrap(),
                &dt("2020-01-01T00:00:00Z")
            ),
            TemporalOrder::Greater
        );
        // Timezone-less vs timezoned, within ±14h -> Indeterminate.
        assert_eq!(
            dt_compare(
                &parse_datetime("2020-01-01T06:00:00", DT).unwrap(),
                &dt("2020-01-01T00:00:00Z")
            ),
            TemporalOrder::Indeterminate
        );
        // Two timezone-less values compare directly.
        assert_eq!(
            dt_compare(
                &parse_datetime("2020-01-01T00:00:00", DT).unwrap(),
                &parse_datetime("2020-01-01T00:00:01", DT).unwrap()
            ),
            TemporalOrder::Less
        );
    }

    #[test]
    fn parse_duration_forms() {
        assert!(parse_duration("P1Y2M3DT4H5M6S", DUR).is_some());
        assert!(parse_duration("-P1Y", YM).is_some());
        assert!(parse_duration("PT1.5S", DAYT).is_some());
        assert!(parse_duration("P30D", DAYT).is_some());
        // invalid: empty, no P, bad unit, fractional year.
        assert!(parse_duration("P", DUR).is_none());
        assert!(parse_duration("1Y", DUR).is_none());
        assert!(parse_duration("PT", DUR).is_none());
        assert!(parse_duration("P1.5Y", DUR).is_none());
        assert!(parse_duration("P1Y", DT).is_none());
    }

    #[test]
    fn duration_compare_partial_order() {
        let p1m = parse_duration("P1M", DUR).unwrap();
        let p30d = parse_duration("P30D", DUR).unwrap();
        // one-month vs thirty-days is the canonical incomparable pair.
        assert_eq!(duration_compare(&p1m, &p30d), TemporalOrder::Indeterminate);
        // equal value via different lexical (P1Y == P12M).
        assert_eq!(
            duration_compare(
                &parse_duration("P1Y", YM).unwrap(),
                &parse_duration("P12M", YM).unwrap()
            ),
            TemporalOrder::Equal
        );
        // monotone: adding days keeps the same month component -> definite.
        assert_eq!(
            duration_compare(
                &parse_duration("P1M15D", DUR).unwrap(),
                &parse_duration("P1M", DUR).unwrap()
            ),
            TemporalOrder::Greater
        );
        assert_eq!(
            duration_compare(&p30d, &parse_duration("P31D", DAYT).unwrap()),
            TemporalOrder::Less
        );
    }

    #[test]
    fn canonical_keys_are_value_stable() {
        // Same instant, different lexical -> same key.
        assert_eq!(
            dt("2020-01-01T13:00:00Z").canonical_key(),
            dt("2020-01-01T14:00:00+01:00").canonical_key()
        );
        // Timezone-less never collides with timezoned.
        assert_ne!(
            parse_datetime("2020-01-01T13:00:00", DT).unwrap().canonical_key(),
            dt("2020-01-01T13:00:00Z").canonical_key()
        );
        // P1Y and P12M key the same by value.
        assert_eq!(
            parse_duration("P1Y", YM).unwrap().canonical_key(),
            parse_duration("P12M", YM).unwrap().canonical_key()
        );
        assert_ne!(
            parse_duration("P1M", DUR).unwrap().canonical_key(),
            parse_duration("P30D", DUR).unwrap().canonical_key()
        );
    }
}
