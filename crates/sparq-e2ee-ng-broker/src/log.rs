//! **Metadata-safe logging.** A broker's log is the single easiest place to
//! accidentally turn the design's disclosure ledger (§5) into a lie, so this
//! module makes the safe shape the *only* shape available: a [`LogRecord`] has no
//! field that can hold ciphertext, plaintext, a secret, or a client-supplied
//! string, and identifiers are recorded as a short [`IdPrefix`], never in full.
//!
//! What a record may carry is exactly what §5 says a conforming broker observes:
//! the session, the message kind, the topic/peer it concerns, sizes and counts,
//! and the outcome. What it can never carry — because there is no field for it —
//! is RDF, SPARQL, `K_read`, a private key, or an envelope's bytes.
//!
//! This is log hygiene, not anonymity: §5 is explicit that a broker still learns
//! membership, timing, volume, and co-access patterns, and truncating an id in a
//! log does not change what the broker itself knows.

use core::fmt;

/// A short, fixed-width hex prefix of a 32-byte identifier. Enough to correlate
/// lines within one log, deliberately not enough to make an exported log a
/// distribution channel for topic/peer identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdPrefix([u8; 4]);

impl IdPrefix {
    /// Take the first four bytes of an identifier.
    pub fn of(id: &[u8; 32]) -> Self {
        IdPrefix([id[0], id[1], id[2], id[3]])
    }
}

impl fmt::Display for IdPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

/// How a handled request ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The request succeeded.
    Ok,
    /// The request was rejected; the carried label is a fixed error-code name.
    Rejected(&'static str),
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::Ok => f.write_str("ok"),
            Outcome::Rejected(c) => write!(f, "rejected:{}", c),
        }
    }
}

/// One metadata-safe log line. Every field is a §5-visible fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogRecord {
    /// Broker-local session number.
    pub session: u64,
    /// Fixed message-kind label (a `&'static str` from a closed set — never a
    /// client-supplied string).
    pub kind: &'static str,
    /// Topic the request concerned, truncated.
    pub topic: Option<IdPrefix>,
    /// Peer the session belongs to, truncated.
    pub peer: Option<IdPrefix>,
    /// Bytes moved by the request (request or response payload, whichever the
    /// call site is reporting).
    pub bytes: u64,
    /// Item count (identifiers probed, envelopes stored, events fanned out, …).
    pub count: u64,
    /// Result.
    pub outcome: Outcome,
}

impl fmt::Display for LogRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "session={} kind={}", self.session, self.kind)?;
        if let Some(t) = self.topic {
            write!(f, " topic={}", t)?;
        }
        if let Some(p) = self.peer {
            write!(f, " peer={}", p)?;
        }
        write!(
            f,
            " bytes={} count={} outcome={}",
            self.bytes, self.count, self.outcome
        )
    }
}

/// Sink for [`LogRecord`]s. Implementations must not widen the record: the type
/// is the contract.
pub trait MetadataLog {
    /// Record one line.
    fn record(&mut self, r: &LogRecord);
}

/// Drops every record. The default, so a broker embedded in a test or a library
/// logs nothing unless the operator opts in.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullLog;

impl MetadataLog for NullLog {
    fn record(&mut self, _r: &LogRecord) {}
}

/// Writes records to stderr, one line each.
#[derive(Debug, Default, Clone, Copy)]
pub struct StderrLog;

impl MetadataLog for StderrLog {
    fn record(&mut self, r: &LogRecord) {
        eprintln!("e2ee-ng-broker {}", r);
    }
}

/// Collects records in memory. Used by tests to assert what a log *cannot*
/// contain.
#[derive(Debug, Default, Clone)]
pub struct CaptureLog {
    /// Rendered lines, in order.
    pub lines: Vec<String>,
}

impl MetadataLog for CaptureLog {
    fn record(&mut self, r: &LogRecord) {
        self.lines.push(r.to_string());
    }
}
