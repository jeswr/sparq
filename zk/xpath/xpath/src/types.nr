//! Core type definitions for XPath datatypes
//!
//! Provides Noir representations of XML Schema datatypes used in SPARQL.

/// DateTime representation with timezone support.
/// Stores microseconds since Unix epoch (1970-01-01T00:00:00Z) as UTC,
/// plus the original timezone offset for component extraction.
///
/// Using two Fields: one for UTC instant, one for timezone offset.
/// This allows comparison using UTC while extracting local components.
pub struct XsdDateTime {
    /// Microseconds since Unix epoch (UTC), as a SIGNED i64 stored in its
    /// two's-complement 64-bit pattern (encode `(e as u64) as Field`, decode
    /// `f as i64`). Pre-1970 instants are negative i64 values; consumers must
    /// decode before dividing or comparing (sq-3x7dl.7). [FABLE-5]
    pub epoch_microseconds: Field,
    /// Timezone offset in minutes from UTC (e.g., -300 for EST, 0 for UTC)
    /// Stored as signed value: positive = east of UTC, negative = west of UTC
    /// Range: -840 to +840 minutes (-14:00 to +14:00)
    pub tz_offset_minutes: i16,
}

impl XsdDateTime {
    /// Create a new DateTime from epoch microseconds (assumes UTC)
    pub fn new(epoch_microseconds: Field) -> Self {
        XsdDateTime { epoch_microseconds, tz_offset_minutes: 0 }
    }

    /// Create a new DateTime with timezone offset
    /// epoch_microseconds: UTC instant
    /// tz_offset_minutes: timezone offset in minutes (e.g., -300 for -05:00)
    pub fn new_with_tz(epoch_microseconds: Field, tz_offset_minutes: i16) -> Self {
        XsdDateTime { epoch_microseconds, tz_offset_minutes }
    }

    /// Get the local microseconds (adjusted by timezone offset)
    pub fn local_microseconds(self) -> Field {
        let offset_micros: i64 = (self.tz_offset_minutes as i64) * 60_000_000;
        let utc_micros: i64 = self.epoch_microseconds as i64;
        let local: i64 = utc_micros + offset_micros;
        // Re-encode the signed result as its two's-complement pattern; a
        // pre-1970 local time is a valid NEGATIVE value that callers recover
        // with `as i64` (sq-3x7dl.7). [FABLE-5]
        let local_u64: u64 = local as u64;
        local_u64 as Field
    }
}

impl Eq for XsdDateTime {
    fn eq(self, other: Self) -> bool {
        self.epoch_microseconds == other.epoch_microseconds
    }
}

/// Duration representation for timezone offsets and intervals.
/// Uses microseconds with a sign flag for negative durations.
pub struct XsdDayTimeDuration {
    /// Total microseconds (absolute value)
    pub microseconds: Field,
    /// Whether duration is negative
    pub negative: bool,
}

impl XsdDayTimeDuration {
    /// Create a new duration
    pub fn new(microseconds: Field, negative: bool) -> Self {
        XsdDayTimeDuration { microseconds, negative }
    }

    /// Create a zero duration
    pub fn zero() -> Self {
        XsdDayTimeDuration { microseconds: 0, negative: false }
    }

    /// Create from signed microseconds (i64)
    pub fn from_signed_micros(micros: i64) -> Self {
        if micros < 0 {
            let abs_val: u64 = (-micros) as u64;
            XsdDayTimeDuration { microseconds: abs_val as Field, negative: true }
        } else {
            let abs_val: u64 = micros as u64;
            XsdDayTimeDuration { microseconds: abs_val as Field, negative: false }
        }
    }

    /// Convert to signed microseconds (i64)
    pub fn to_signed_micros(self) -> i64 {
        let abs_micros: i64 = self.microseconds as i64;
        if self.negative {
            -abs_micros
        } else {
            abs_micros
        }
    }
}

impl Eq for XsdDayTimeDuration {
    fn eq(self, other: Self) -> bool {
        (self.microseconds == other.microseconds) & (self.negative == other.negative)
    }
}

/// Date representation with timezone support.
/// Stores days since Unix epoch (1970-01-01) as an integer,
/// plus the original timezone offset for correct component extraction.
pub struct XsdDate {
    /// Days since Unix epoch (1970-01-01), can be negative for dates before 1970
    pub epoch_days: i32,
    /// Timezone offset in minutes from UTC (e.g., -300 for EST, 0 for UTC)
    /// Range: -840 to +840 minutes (-14:00 to +14:00)
    pub tz_offset_minutes: i16,
}

impl XsdDate {
    /// Create a new Date from epoch days (assumes UTC)
    pub fn new(epoch_days: i32) -> Self {
        XsdDate { epoch_days, tz_offset_minutes: 0 }
    }

    /// Create a new Date with timezone offset
    pub fn new_with_tz(epoch_days: i32, tz_offset_minutes: i16) -> Self {
        XsdDate { epoch_days, tz_offset_minutes }
    }
}

impl Eq for XsdDate {
    fn eq(self, other: Self) -> bool {
        // Compare epoch_days directly, which compares calendar dates (not instants in time).
        // This follows XPath date semantics: dates with the same year/month/day are equal
        // regardless of timezone. For example, 2000-01-01+05:00 equals 2000-01-01-10:00
        // because they represent the same calendar date in different timezones.
        // Note: timezone offset is intentionally not compared here.
        self.epoch_days == other.epoch_days
    }
}

/// Time representation with timezone support.
/// Stores microseconds since midnight plus timezone offset.
pub struct XsdTime {
    /// Microseconds since midnight (0 to 86,399,999,999)
    pub microseconds_since_midnight: u64,
    /// Timezone offset in minutes from UTC
    /// Range: -840 to +840 minutes (-14:00 to +14:00)
    pub tz_offset_minutes: i16,
}

/// Microseconds in a day constant for time calculations
global TIME_MICROS_PER_DAY: u64 = 86_400_000_000;
global TIME_MICROS_PER_HOUR: u64 = 3_600_000_000;
global TIME_MICROS_PER_MINUTE: u64 = 60_000_000;
global TIME_MICROS_PER_SECOND: u64 = 1_000_000;

impl XsdTime {
    /// Create a new Time from microseconds since midnight (assumes UTC)
    pub fn new(microseconds_since_midnight: u64) -> Self {
        XsdTime { microseconds_since_midnight, tz_offset_minutes: 0 }
    }

    /// Create a new Time with timezone offset
    pub fn new_with_tz(microseconds_since_midnight: u64, tz_offset_minutes: i16) -> Self {
        XsdTime { microseconds_since_midnight, tz_offset_minutes }
    }

    /// Create a Time from components
    pub fn from_components(hours: u8, minutes: u8, seconds: u8, microseconds: u32) -> Self {
        let micros = (hours as u64) * TIME_MICROS_PER_HOUR
            + (minutes as u64) * TIME_MICROS_PER_MINUTE
            + (seconds as u64) * TIME_MICROS_PER_SECOND
            + microseconds as u64;
        XsdTime::new(micros)
    }

    /// Create a Time from components with timezone
    pub fn from_components_with_tz(
        hours: u8,
        minutes: u8,
        seconds: u8,
        microseconds: u32,
        tz_offset_minutes: i16,
    ) -> Self {
        let micros = (hours as u64) * TIME_MICROS_PER_HOUR
            + (minutes as u64) * TIME_MICROS_PER_MINUTE
            + (seconds as u64) * TIME_MICROS_PER_SECOND
            + microseconds as u64;
        XsdTime::new_with_tz(micros, tz_offset_minutes)
    }

    /// Get UTC microseconds (for comparison)
    /// Normalizes to UTC by subtracting the timezone offset
    pub fn utc_microseconds(self) -> i64 {
        let local_micros = self.microseconds_since_midnight as i64;
        let offset_micros = (self.tz_offset_minutes as i64) * 60_000_000;
        local_micros - offset_micros
    }
}

impl Eq for XsdTime {
    fn eq(self, other: Self) -> bool {
        // Compare in UTC for correct equality
        self.utc_microseconds() == other.utc_microseconds()
    }
}
