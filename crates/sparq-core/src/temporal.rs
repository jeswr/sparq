//! XSD date/dateTime VALUES and the epoch side-cache cells.
//!
//! [`Timeline`] is the parsed value of an `xsd:date` / `xsd:dateTime` lexical —
//! seconds-from-epoch plus timezone presence — with XPath comparison semantics
//! (both-with-tz / both-without compare directly; MIXED presence is only
//! decidable outside the ±14h window). It lives in core (rather than the engine,
//! which consumes it for FILTER/ORDER BY/`=`) so the graph can precompute a
//! per-term [`Temporal`] cache at load time, exactly like the f64 `numerics`
//! cache: dateTime evaluation then never round-trips the dictionary per row.
//!
//! [`Temporal`] is the cache cell: the literal's datatype family plus the
//! PRECOMPUTED comparison key `Timeline::instant()` (an f64 — bit-identical to
//! what the per-row parse would feed `partial_cmp`, so cached comparisons match
//! the dict-based path exactly, including sub-second precision and the f64
//! collapse of far-apart instants) and the timezone-presence bit that decides
//! the mixed-presence indeterminate window. `xsd:time` is NOT cached: the
//! engine compares it lexically (OtherXsd), not on the timeline, and a cache
//! must not change that.

use std::cmp::Ordering;

/// An xsd:date / xsd:dateTime VALUE: seconds-from-epoch of the local time, fractional
/// seconds, and the timezone offset when present. Comparison follows XSD: both-with-tz
/// and both-without compare directly; MIXED presence is only decidable outside the
/// ±14h window (inside it the comparison is indeterminate — a SPARQL type error).
#[derive(Clone, Copy, Debug)]
pub struct Timeline {
    pub secs: i64,
    pub frac: f64,
    pub tz: Option<i64>,
}

impl Timeline {
    pub fn parse_datetime(s: &str) -> Option<Timeline> {
        let (date, rest) = s.split_once('T')?;
        let (time, tz) = match rest.find(['Z', '+', '-']) {
            Some(i) => (&rest[..i], Some(parse_tz(&rest[i..])?)),
            None => (rest, None),
        };
        let days = parse_civil_date(date)?;
        let mut t = time.split(':');
        let h: i64 = t.next()?.parse().ok()?;
        let mi: i64 = t.next()?.parse().ok()?;
        let sec_lex = t.next()?;
        if t.next().is_some() {
            return None;
        }
        let sec: f64 = sec_lex.parse().ok()?;
        Some(Timeline {
            secs: days * 86_400 + h * 3600 + mi * 60 + sec.trunc() as i64,
            frac: sec.fract(),
            tz,
        })
    }

    pub fn parse_date(s: &str) -> Option<Timeline> {
        // The timezone suffix starts after the day: "...-23Z" / "...-23+05:00". A bare
        // date's own hyphens must not be mistaken for an offset sign, so require the
        // ":" of "±hh:mm" at the right position.
        let (date, tz) = if let Some(d) = s.strip_suffix('Z') {
            (d, Some(0))
        } else if s.len() > 10 && matches!(s.as_bytes()[s.len() - 6], b'+' | b'-') && s.as_bytes()[s.len() - 3] == b':' {
            (&s[..s.len() - 6], Some(parse_tz(&s[s.len() - 6..])?))
        } else {
            (s, None)
        };
        Some(Timeline { secs: parse_civil_date(date)? * 86_400, frac: 0.0, tz })
    }

    /// The absolute instant (treating an absent timezone as UTC) in seconds.
    pub fn instant(&self) -> f64 {
        (self.secs - self.tz.unwrap_or(0)) as f64 + self.frac
    }

    pub fn cmp_tl(a: Timeline, b: Timeline) -> Option<Ordering> {
        cmp_instants(a.instant(), a.tz.is_some(), b.instant(), b.tz.is_some())
    }
}

/// The XPath dateTime/date comparison on precomputed instants: direct when both
/// or neither operand carries a timezone; with MIXED presence only decidable
/// outside the ±14h window (inside it: indeterminate -> `None`).
#[inline]
fn cmp_instants(ai: f64, a_tz: bool, bi: f64, b_tz: bool) -> Option<Ordering> {
    // Same (or no) timezone: a direct compare. With MIXED presence the order is
    // only decidable outside the ±14h window; inside it the result is indeterminate.
    if a_tz == b_tz || (ai - bi).abs() > 14.0 * 3600.0 {
        ai.partial_cmp(&bi)
    } else {
        None
    }
}

/// `"Z"` / `"±hh:mm"` -> offset seconds.
pub fn parse_tz(tz: &str) -> Option<i64> {
    if tz == "Z" {
        return Some(0);
    }
    let (sign, hm) = tz.split_at(1);
    let (h, m) = hm.split_once(':')?;
    let (h, m): (i64, i64) = (h.parse().ok()?, m.parse().ok()?);
    let off = h * 3600 + m * 60;
    Some(if sign == "-" { -off } else { off })
}

/// `[-]YYYY-MM-DD` -> days from the epoch (Howard Hinnant's days_from_civil).
pub fn parse_civil_date(date: &str) -> Option<i64> {
    let neg = date.starts_with('-');
    let mut p = date.strip_prefix('-').unwrap_or(date).split('-');
    let y: i64 = p.next()?.parse().ok()?;
    let y = if neg { -y } else { y };
    let m: i64 = p.next()?.parse().ok()?;
    let d: i64 = p.next()?.parse().ok()?;
    if p.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

/// The datatype family of a cached temporal value. `xsd:dateTime` and
/// `xsd:dateTimeStamp` share a family (they compare on the timeline); `xsd:date`
/// is a DISJOINT family (cross-family `=` is false, ordering a type error).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalKind {
    DateTime,
    Date,
}

/// A cached temporal VALUE: the precomputed comparison key of one dictionary term.
/// `instant` is exactly [`Timeline::instant`] of the parsed lexical, so comparisons
/// through the cache are bit-identical to re-parsing the term per row.
#[derive(Clone, Copy, Debug)]
pub struct Temporal {
    pub instant: f64,
    pub has_tz: bool,
    pub kind: TemporalKind,
}

/// XSD namespace datatype IRIs (kept as plain strs: this module must not depend
/// on oxrdf — the dict hands the cache builder borrowed datatype strings).
const XSD_DATE_TIME: &str = "http://www.w3.org/2001/XMLSchema#dateTime";
const XSD_DATE_TIME_STAMP: &str = "http://www.w3.org/2001/XMLSchema#dateTimeStamp";
const XSD_DATE: &str = "http://www.w3.org/2001/XMLSchema#date";

impl Temporal {
    /// The cache cell for a literal's (lexical, datatype) — `None` when the datatype
    /// is not dateTime/dateTimeStamp/date or the lexical is ill-formed (ill-formed
    /// temporals stay on the slow path, which yields the type-error semantics).
    pub fn of_lit(value: &str, datatype: &str) -> Option<Temporal> {
        let (kind, tl) = match datatype {
            XSD_DATE_TIME | XSD_DATE_TIME_STAMP => (TemporalKind::DateTime, Timeline::parse_datetime(value)?),
            XSD_DATE => (TemporalKind::Date, Timeline::parse_date(value)?),
            _ => return None,
        };
        Some(Temporal { instant: tl.instant(), has_tz: tl.tz.is_some(), kind })
    }

    /// XPath comparison of two cached temporals: `None` for cross-family operands
    /// (dateTime vs date) and for the mixed-timezone-presence indeterminate window —
    /// exactly the cases the engine's per-row parse path treats as type errors.
    #[inline]
    pub fn cmp_t(a: Temporal, b: Temporal) -> Option<Ordering> {
        if a.kind != b.kind {
            return None;
        }
        cmp_instants(a.instant, a.has_tz, b.instant, b.has_tz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering::*;

    fn dt(s: &str) -> Temporal {
        Temporal::of_lit(s, XSD_DATE_TIME).expect("valid dateTime")
    }

    #[test]
    fn equal_instants_across_timezones() {
        // XPath: equal instants compare equal even with different offsets.
        let a = dt("2024-03-15T13:00:00Z");
        let b = dt("2024-03-15T14:00:00+01:00");
        assert_eq!(Temporal::cmp_t(a, b), Some(Equal));
        let c = dt("2024-03-15T08:00:00-05:00");
        assert_eq!(Temporal::cmp_t(a, c), Some(Equal));
    }

    #[test]
    fn mixed_presence_window_is_indeterminate() {
        let zoned = dt("2024-03-15T13:00:00Z");
        let floating = dt("2024-03-15T13:00:00");
        // Same wall time, one zoned one floating: inside ±14h -> indeterminate.
        assert_eq!(Temporal::cmp_t(zoned, floating), None);
        // Outside the window the order is decidable.
        let far = dt("2024-03-17T13:00:00");
        assert_eq!(Temporal::cmp_t(zoned, far), Some(Less));
    }

    #[test]
    fn subsecond_precision_preserved() {
        let a = dt("2024-03-15T13:00:00.250Z");
        let b = dt("2024-03-15T13:00:00.500Z");
        assert_eq!(Temporal::cmp_t(a, b), Some(Less));
        assert_eq!(Temporal::cmp_t(b, a), Some(Greater));
        let c = dt("2024-03-15T13:00:00.250Z");
        assert_eq!(Temporal::cmp_t(a, c), Some(Equal));
    }

    #[test]
    fn date_and_datetime_are_disjoint_families() {
        let d = Temporal::of_lit("2024-03-15", XSD_DATE).unwrap();
        let t = dt("2024-03-15T00:00:00Z");
        assert_eq!(Temporal::cmp_t(d, t), None);
    }

    #[test]
    fn cache_cell_matches_per_row_parse() {
        // The cell must carry EXACTLY the parsed Timeline's instant/tz-presence.
        for lex in ["2024-03-15T13:45:30.123456Z", "2024-03-15T13:45:30-09:30", "-0044-03-15T12:00:00", "9999-12-31T23:59:59.999Z"] {
            let tl = Timeline::parse_datetime(lex).unwrap();
            let t = dt(lex);
            assert_eq!(t.instant.to_bits(), tl.instant().to_bits(), "instant differs for {lex}");
            assert_eq!(t.has_tz, tl.tz.is_some());
        }
    }

    #[test]
    fn ill_formed_and_foreign_datatypes_are_not_cached() {
        assert!(Temporal::of_lit("not-a-date", XSD_DATE_TIME).is_none());
        assert!(Temporal::of_lit("2024-13-99T99:99:99", XSD_DATE_TIME).is_none());
        // xsd:time compares lexically in the engine — must NOT enter the cache.
        assert!(Temporal::of_lit("13:00:00", "http://www.w3.org/2001/XMLSchema#time").is_none());
        assert!(Temporal::of_lit("42", "http://www.w3.org/2001/XMLSchema#integer").is_none());
    }
}
