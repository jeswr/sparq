//! [SONNET-4.6] (sq-l6zks, gh-3216) The **`ChangeSink` seam** — relay the durable
//! change-data-capture log ([`change_stream`](crate::change_stream)) onward to an
//! **external broker** (Kafka, NATS, Pulsar, an HTTP webhook, a downstream index).
//!
//! ## What this is
//!
//! [`ChangeLog`] is the durable, replayable stream ON DISK; a consumer polls it by
//! sequence number. This module is the **push side** of that: a
//! [`ChangeSink`] trait (one method — deliver one [`ChangeRecord`]) plus a
//! [`ChangeRelay`] pump that reads the log from a **durable cursor**, delivers records
//! **in order**, and only advances the cursor once the sink has confirmed them.
//!
//! ```text
//!   Writer ──commit hook──▶ ChangeLog (segments on disk)
//!                                │  poll(cursor+1)
//!                                ▼
//!                          ChangeRelay ──deliver()──▶ impl ChangeSink ──▶ Kafka / NATS / …
//!                                │
//!                                └─ durable cursor file  (resume after restart)
//!                                   └─▶ RetentionPolicy::acked_through_seq
//! ```
//!
//! ## What this is NOT (honest boundary — read before wiring a broker)
//!
//! **No broker client is vendored here.** This crate adds NO dependency for the sink
//! seam: there is no `rdkafka`, no `async-nats`, no TLS/SASL stack, and no async runtime
//! (library-first §6.1 — the same rule that keeps `sparq-serve` sync and
//! runtime-agnostic). What ships is the trait an embedder implements against THEIR
//! broker client, the ordered at-least-once pump, and one dependency-free bridge sink
//! ([`JsonLinesSink`]) that writes NDJSON to any [`std::io::Write`] — a file, a pipe into
//! a Kafka/NATS producer sidecar's stdin. A first-party Kafka/NATS *connector* (with
//! real client dependencies, retry/backoff policy and broker auth) would be a separate,
//! heavier opt-in crate; it is deliberately not this module.
//!
//! **Delivery is at-least-once, never exactly-once.** The cursor is persisted AFTER the
//! sink flushes, so a crash between the flush and the cursor write re-delivers the batch.
//! A consumer that needs idempotence keys on [`ChangeRecord::seq`] (unique and gapless).
//!
//! **No authenticity/encryption.** Exactly as for [`backup`](crate::backup) and the change
//! stream itself: records carry a non-cryptographic FNV-1a integrity digest on disk, and
//! nothing is signed. A sink that needs an authentic cross-service feed wraps its own
//! signing around [`record_to_json`].
//!
//! ## Ordering, fail-closed behaviour, and the retention hand-off
//!
//! - **In-order, gapless.** Records are delivered strictly by ascending
//!   [`seq`](ChangeRecord::seq); the pump stops at the first failure rather than skipping
//!   ahead, so a sink never sees a hole.
//! - **Re-base markers are delivered, never swallowed.** A [`ChangeRecord::rebase`] record
//!   is an HONEST GAP in the stream (see [`ChangeLog::rebase_to`]) — the relay hands it to
//!   the sink like any other record so the downstream consumer learns the span was not
//!   captured. Flattening it away would make an incomplete stream look complete.
//! - **A trimmed cursor fails closed.** If retention dropped the records the cursor still
//!   needs, [`ChangeLog::poll`] fails closed and so does [`ChangeRelay::pump`] — it never
//!   silently resumes at a later offset.
//! - **A permanent sink failure poisons the relay.** [`SinkError::Permanent`] stops the
//!   relay for good ([`RelayError::Poisoned`]); a [`SinkError::Retryable`] leaves it armed
//!   so the next `pump` re-delivers from the same record.
//! - **Retention hand-off.** [`ChangeRelay::retention_policy`] turns the durable cursor
//!   into the [`RetentionPolicy::acked_through_seq`] hard safety bound, so old segments are
//!   only ever dropped once this sink has durably taken them.
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use sparq_serve::change_sink::{ChangeRelay, JsonLinesSink};
//! use sparq_serve::ChangeLog;
//!
//! // A reader handle on the log the writer records into, and a sink bridging to stdout
//! // (in production: your broker client's `impl ChangeSink`).
//! let log = ChangeLog::open("/var/lib/sparq/cdc")?;
//! let sink = JsonLinesSink::new(std::io::stdout());
//! let mut relay = ChangeRelay::open("/var/lib/sparq/cdc.cursor", sink)?;
//!
//! let report = relay.pump(&log)?;           // deliver everything new, in order
//! println!("relayed {} records", report.delivered);
//! # Ok(()) }
//! ```

use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::backup::{body_digest, BackupError};
use crate::change_stream::{ChangeLog, ChangeOp, ChangeRecord, RetentionPolicy};

/// Magic line that opens a durable relay CURSOR file — distinct from the segment /
/// backup / delta magics, so pointing a relay at the wrong file fails closed at the
/// magic line rather than mis-reading an offset.
pub const CURSOR_MAGIC: &str = "SPARQ-CHANGESINK-CURSOR";

/// Cursor file format version. [`ChangeRelay::open`] rejects an unknown version
/// (fail-closed) rather than mis-reading a newer layout.
const CURSOR_FORMAT_VERSION: u32 = 1;

/// Default cap on how many records one [`ChangeRelay::pump`] delivers before returning
/// (see [`RelayConfig::max_records_per_pump`]). Bounds both the per-pump memory and the
/// window a crash can force the sink to re-consume.
pub const DEFAULT_MAX_RECORDS_PER_PUMP: usize = 1024;

/// Why a [`ChangeSink`] refused a record.
///
/// The distinction is load-bearing, not cosmetic: a [`Retryable`](Self::Retryable) failure
/// (broker unreachable, queue full, transient auth refresh) leaves the [`ChangeRelay`]
/// armed so the next [`pump`](ChangeRelay::pump) re-delivers from the same record, while a
/// [`Permanent`](Self::Permanent) failure (the broker rejected the payload itself, a
/// mis-configured topic) **poisons** the relay — every later `pump` fails closed rather
/// than skipping the record and silently gapping the downstream feed.
#[derive(Debug)]
pub enum SinkError {
    /// A transient failure. The relay stays armed; the record is re-delivered next pump.
    Retryable(String),
    /// A permanent failure. The relay fail-closes from here on ([`RelayError::Poisoned`]);
    /// an operator must fix the sink (the durable cursor still points at the failing
    /// record, so a restarted relay resumes exactly there — nothing is lost or skipped).
    Permanent(String),
}

impl SinkError {
    /// `true` for [`SinkError::Retryable`] — the relay stays armed and re-delivers.
    pub fn is_retryable(&self) -> bool {
        matches!(self, SinkError::Retryable(_))
    }

    /// The failure message, without the retryable/permanent prefix.
    pub fn message(&self) -> &str {
        match self {
            SinkError::Retryable(m) | SinkError::Permanent(m) => m,
        }
    }
}

impl fmt::Display for SinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // [SONNET-4.6] positional format args (CodeQL rust/unused-variable false-positive).
        match self {
            SinkError::Retryable(m) => write!(f, "change sink failed (retryable): {}", m),
            SinkError::Permanent(m) => write!(f, "change sink failed (permanent): {}", m),
        }
    }
}

impl std::error::Error for SinkError {}

impl From<io::Error> for SinkError {
    /// An I/O failure toward a broker/pipe is presumed **transient** (a closed pipe is
    /// re-openable, a full disk is drainable). A sink that knows a failure is terminal
    /// constructs [`SinkError::Permanent`] explicitly.
    fn from(e: io::Error) -> Self {
        SinkError::Retryable(e.to_string())
    }
}

/// The pluggable destination for change records — the seam an **external broker**
/// integration implements.
///
/// Heavy/external clients (a Kafka producer, a NATS connection, an HTTP webhook) stay OUT
/// of this crate — an embedder implements this trait over their own client, exactly as
/// `sparq-server`'s audit sink does for SIEM/OpenTelemetry targets. The one built-in
/// implementation is the dependency-free [`JsonLinesSink`] bridge.
///
/// `deliver` takes `&mut self` because the pump owns its sink exclusively; a sink that
/// must be shared can hold an `Arc<…>` internally.
pub trait ChangeSink {
    /// Hand one change record to the destination, in ascending
    /// [`seq`](ChangeRecord::seq) order.
    ///
    /// A record with [`rebase`](ChangeRecord::rebase) set is an honest GAP marker, not a
    /// commit diff — it carries no changes, and a downstream consumer that needs a
    /// complete view must re-bootstrap when it sees one.
    ///
    /// Returning `Ok(())` means the record is *accepted*, not necessarily durable at the
    /// broker; durability is confirmed by [`flush`](Self::flush), which the relay calls
    /// before it advances its cursor.
    fn deliver(&mut self, record: &ChangeRecord) -> Result<(), SinkError>;

    /// Confirm every record accepted since the last flush is durable at the destination
    /// (flush the producer, await broker acks). The relay calls this before persisting its
    /// cursor, so a failure here means the whole batch is re-delivered next pump.
    ///
    /// The default is a no-op — correct only for a sink whose `deliver` is already
    /// synchronous and durable.
    fn flush(&mut self) -> Result<(), SinkError> {
        Ok(())
    }
}

/// The built-in, **dependency-free** bridge sink: one NDJSON line per change record
/// written to any [`std::io::Write`].
///
/// This is the "no broker client vendored" escape hatch — point it at a file, or at the
/// stdin of a Kafka/NATS producer sidecar, and the durable CDC stream becomes a line feed
/// any broker tooling can consume. The payload is [`record_to_json`].
pub struct JsonLinesSink<W: Write> {
    out: W,
}

impl<W: Write> JsonLinesSink<W> {
    /// Wraps a writer. Nothing is written until the first
    /// [`deliver`](ChangeSink::deliver).
    pub fn new(out: W) -> Self {
        JsonLinesSink { out }
    }

    /// Borrows the underlying writer (e.g. to inspect a `Vec<u8>` in a test).
    pub fn get_ref(&self) -> &W {
        &self.out
    }

    /// Unwraps the sink, returning the underlying writer.
    pub fn into_inner(self) -> W {
        self.out
    }
}

impl<W: Write> ChangeSink for JsonLinesSink<W> {
    fn deliver(&mut self, record: &ChangeRecord) -> Result<(), SinkError> {
        let line = record_to_json(record);
        self.out.write_all(line.as_bytes())?;
        self.out.write_all(b"\n")?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), SinkError> {
        self.out.flush()?;
        Ok(())
    }
}

/// Renders one change record as a single-line JSON object — the wire payload
/// [`JsonLinesSink`] writes, and a ready-made body for a broker sink.
///
/// The field vocabulary matches `sparq-server`'s `GET /streams` endpoint (`ADD`/`REMOVE`,
/// `stmt`, `commitTimestampNanos`) with one deliberate difference: **one message per
/// COMMIT**, not one per quad, so a broker consumer observes the commit's atomicity and
/// can key on the unique `seq`.
///
/// ```text
/// {"seq":3,"generation":3,"commitTimestampNanos":17...,"rebase":false,
///  "changes":[{"op":"ADD","stmt":"<s> <p> <o> ."},{"op":"REMOVE","stmt":"…"}]}
/// ```
///
/// A [`rebase`](ChangeRecord::rebase) record renders with `"rebase":true` and an empty
/// `changes` array — the honest gap marker, never silently dropped.
pub fn record_to_json(record: &ChangeRecord) -> String {
    let mut s = String::with_capacity(96 + record.changes.len() * 64);
    // [SONNET-4.6] positional format args (CodeQL rust/unused-variable false-positive).
    s.push_str(&format!(
        "{{\"seq\":{},\"generation\":{},\"commitTimestampNanos\":{},\"rebase\":{},\"changes\":[",
        record.seq, record.generation, record.timestamp_unix_nanos, record.rebase
    ));
    for (i, change) in record.changes.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        let op = match change.op {
            ChangeOp::Insert => "ADD",
            ChangeOp::Delete => "REMOVE",
        };
        s.push_str(&format!("{{\"op\":\"{}\",\"stmt\":\"", op));
        escape_json_into(&change.quad, &mut s);
        s.push_str("\"}");
    }
    s.push_str("]}");
    s
}

/// Appends `raw` to `out` with the JSON string escapes RFC 8259 requires. N-Quads lines
/// carry `"` and `\` verbatim, so escaping is load-bearing for a parseable line.
fn escape_json_into(raw: &str, out: &mut String) {
    for c in raw.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // [SONNET-4.6] positional format args (CodeQL rust/unused-variable FP).
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
}

/// Tuning for a [`ChangeRelay`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelayConfig {
    /// Deliver at most this many records per [`pump`](ChangeRelay::pump) call
    /// ([`DEFAULT_MAX_RECORDS_PER_PUMP`]). Bounds per-pump memory AND the at-least-once
    /// re-delivery window a crash can open. `0` is treated as `1` (a pump always makes
    /// progress).
    pub max_records_per_pump: usize,
    /// `fsync` the cursor file before [`pump`](ChangeRelay::pump) returns. ON by default:
    /// without it a crash can lose the advance and re-deliver an already-taken batch (the
    /// at-least-once contract still holds — this only bounds how much is repeated).
    pub fsync: bool,
}

impl Default for RelayConfig {
    fn default() -> Self {
        RelayConfig {
            max_records_per_pump: DEFAULT_MAX_RECORDS_PER_PUMP,
            fsync: true,
        }
    }
}

/// What one [`ChangeRelay::pump`] actually delivered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelayReport {
    /// Number of records handed to the sink and confirmed by its flush in this pass.
    pub delivered: usize,
    /// The highest seq the sink has now durably taken (`None` for a relay that has never
    /// delivered anything) — the same watermark [`ChangeRelay::retention_policy`] feeds to
    /// retention.
    pub acked_through_seq: Option<u64>,
    /// `true` if the log still holds records beyond this pass's cap — call
    /// [`pump`](ChangeRelay::pump) again to continue.
    pub has_more: bool,
}

/// Why a [`ChangeRelay::pump`] failed. Every variant is fail-closed: the durable cursor
/// only ever names records the sink CONFIRMED, so a failed pump is always safe to retry
/// (at-least-once) and never silently skips a record.
#[derive(Debug)]
pub enum RelayError {
    /// Reading the durable log failed — including the fail-closed case where retention
    /// dropped the records this cursor still needs (re-bootstrap, then reset the cursor).
    Log(BackupError),
    /// The durable cursor file could not be read or persisted, or is corrupt / a version
    /// this build does not understand.
    Cursor(BackupError),
    /// The sink refused the record at `seq`. Retryable unless
    /// [`SinkError::is_retryable`] says otherwise (in which case the relay is now
    /// poisoned).
    Sink {
        /// The sequence number of the record the sink refused.
        seq: u64,
        /// The sink's own error.
        source: SinkError,
    },
    /// The relay fail-closed earlier on a [`SinkError::Permanent`] and refuses to deliver
    /// anything further. The cursor still points at the failing record, so a relay
    /// re-opened after the sink is fixed resumes exactly there.
    Poisoned(String),
}

impl fmt::Display for RelayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // [SONNET-4.6] positional format args (CodeQL rust/unused-variable false-positive).
        match self {
            RelayError::Log(e) => write!(f, "change relay could not read the log: {}", e),
            RelayError::Cursor(e) => write!(f, "change relay cursor error: {}", e),
            RelayError::Sink { seq, source } => {
                write!(f, "change relay stopped at seq {}: {}", seq, source)
            }
            RelayError::Poisoned(m) => write!(f, "change relay is fail-closed: {}", m),
        }
    }
}

impl std::error::Error for RelayError {}

/// Pumps the durable [`ChangeLog`] into a [`ChangeSink`], in order, with a **durable
/// cursor** so a restarted process resumes exactly where the sink left off.
///
/// The relay does NOT own the log handle: pass a [`ChangeLog`] to
/// [`pump`](Self::pump) (the writer's own recording handle, or a separate reader handle
/// opened on the same directory). That keeps the single-writer append discipline of the
/// change stream intact — the relay only ever polls.
///
/// The relay is a passive pump: the host decides when to call it (after a commit, on a
/// timer, from a dedicated relay thread). It spawns nothing and needs no async runtime.
pub struct ChangeRelay<S: ChangeSink> {
    cursor_path: PathBuf,
    acked_through_seq: Option<u64>,
    config: RelayConfig,
    sink: S,
    poisoned: Option<String>,
}

impl<S: ChangeSink> ChangeRelay<S> {
    /// Opens the relay against the durable cursor at `cursor_path`, recovering an existing
    /// cursor (resume) or starting fresh (the file is created on the first successful
    /// pump). A fresh relay starts at the log's earliest RETAINED record, i.e. it replays
    /// everything the log still holds.
    ///
    /// **Fail-closed:** a cursor file with a bad magic, an unknown version, or a mismatched
    /// digest is a [`RelayError::Cursor`] — never a silent restart from zero (which would
    /// flood the broker with a full replay) and never a silent skip.
    pub fn open(cursor_path: impl AsRef<Path>, sink: S) -> Result<Self, RelayError> {
        Self::open_with_config(cursor_path, sink, RelayConfig::default())
    }

    /// [`open`](Self::open) with an explicit [`RelayConfig`].
    pub fn open_with_config(
        cursor_path: impl AsRef<Path>,
        sink: S,
        config: RelayConfig,
    ) -> Result<Self, RelayError> {
        let cursor_path = cursor_path.as_ref().to_path_buf();
        let acked_through_seq = read_cursor(&cursor_path).map_err(RelayError::Cursor)?;
        Ok(ChangeRelay {
            cursor_path,
            acked_through_seq,
            config,
            sink,
            poisoned: None,
        })
    }

    /// The highest seq the sink has durably taken, or `None` if it has taken nothing yet.
    pub fn acked_through_seq(&self) -> Option<u64> {
        self.acked_through_seq
    }

    /// `true` once a [`SinkError::Permanent`] has fail-closed this relay.
    pub fn is_poisoned(&self) -> bool {
        self.poisoned.is_some()
    }

    /// Borrows the sink (e.g. to read a bridge sink's counters).
    pub fn sink(&self) -> &S {
        &self.sink
    }

    /// A [`RetentionPolicy`] carrying this relay's ack watermark as the **hard safety
    /// bound** — pass it to [`ChangeLog::apply_retention`] and no segment holding a record
    /// this sink has not taken can ever be dropped. Compose the pressure criteria
    /// (`max_age` / `max_total_bytes`) on top:
    ///
    /// ```no_run
    /// # fn demo<S: sparq_serve::change_sink::ChangeSink>(
    /// #     relay: &sparq_serve::change_sink::ChangeRelay<S>,
    /// #     log: &mut sparq_serve::ChangeLog,
    /// # ) -> Result<(), Box<dyn std::error::Error>> {
    /// let policy = sparq_serve::RetentionPolicy {
    ///     max_total_bytes: Some(1 << 30),
    ///     ..relay.retention_policy()
    /// };
    /// log.apply_retention(&policy)?;
    /// # Ok(()) }
    /// ```
    pub fn retention_policy(&self) -> RetentionPolicy {
        RetentionPolicy {
            acked_through_seq: self.acked_through_seq,
            ..RetentionPolicy::default()
        }
    }

    /// Delivers every record the sink has not yet taken, in ascending seq order, up to
    /// [`RelayConfig::max_records_per_pump`].
    ///
    /// The sequence per pass is: poll the log from `cursor + 1` (or the log's earliest
    /// retained record for a fresh cursor) → `deliver` each record in order → `flush` the
    /// sink → persist the cursor. The cursor therefore only ever names records the sink
    /// CONFIRMED, which is what makes delivery **at-least-once**: a crash anywhere in the
    /// pass re-delivers, and nothing is ever skipped.
    ///
    /// A failing record stops the pass immediately (no reordering, no hole); records
    /// already delivered before it are still flushed and their progress persisted, so a
    /// long batch does not lose ground to one bad record. The failure surfaces as
    /// [`RelayError::Sink`], and a [`SinkError::Permanent`] additionally poisons the relay.
    pub fn pump(&mut self, log: &ChangeLog) -> Result<RelayReport, RelayError> {
        if let Some(why) = &self.poisoned {
            return Err(RelayError::Poisoned(why.clone()));
        }

        let from = match self.acked_through_seq {
            Some(seq) => seq + 1,
            // A fresh cursor replays from the earliest RETAINED record (0 on a
            // never-trimmed log) — the TRIM_HORIZON shape, not a fail-closed poll.
            None => log.first_seq(),
        };
        let mut records = log.poll(from).map_err(RelayError::Log)?;

        let cap = self.config.max_records_per_pump.max(1);
        let has_more = records.len() > cap;
        records.truncate(cap);
        if records.is_empty() {
            return Ok(RelayReport {
                delivered: 0,
                acked_through_seq: self.acked_through_seq,
                has_more: false,
            });
        }

        let mut accepted_through = None;
        let mut delivered = 0usize;
        let mut failure = None;
        for record in &records {
            match self.sink.deliver(record) {
                Ok(()) => {
                    accepted_through = Some(record.seq);
                    delivered += 1;
                }
                Err(e) => {
                    failure = Some((record.seq, e));
                    break;
                }
            }
        }

        // The sink confirms durability BEFORE the cursor advances. A flush failure means
        // nothing in this pass is durable downstream, so the cursor stays put and the
        // whole batch is re-delivered next pump.
        if let Err(e) = self.sink.flush() {
            let first = records[0].seq;
            return Err(self.fail(first, e));
        }

        if let Some(seq) = accepted_through {
            self.acked_through_seq = Some(seq);
            write_cursor(&self.cursor_path, seq, self.config.fsync)
                .map_err(RelayError::Cursor)?;
        }

        if let Some((seq, e)) = failure {
            return Err(self.fail(seq, e));
        }

        Ok(RelayReport {
            delivered,
            acked_through_seq: self.acked_through_seq,
            has_more,
        })
    }

    /// Builds the [`RelayError::Sink`] for a refused record, poisoning the relay first if
    /// the sink called the failure permanent.
    fn fail(&mut self, seq: u64, source: SinkError) -> RelayError {
        if !source.is_retryable() {
            // [SONNET-4.6] positional format args (CodeQL rust/unused-variable FP).
            self.poisoned = Some(format!(
                "permanent sink failure at seq {}: {}",
                seq,
                source.message()
            ));
        }
        RelayError::Sink { seq, source }
    }
}

/// Reads the durable cursor at `path`, returning the acked watermark (`None` for an
/// absent file — a fresh relay). Fail-closed on a bad magic / unknown version / digest
/// mismatch.
fn read_cursor(path: &Path) -> Result<Option<u64>, BackupError> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(BackupError::Io(e)),
    };
    let mut text = String::new();
    file.read_to_string(&mut text)?;

    let mut lines = text.lines();
    let header = lines.next().unwrap_or("");
    let mut parts = header.splitn(2, ' ');
    if parts.next().unwrap_or("") != CURSOR_MAGIC {
        return Err(BackupError::Format(format!(
            "not a sparq change-sink cursor (expected magic {:?})",
            CURSOR_MAGIC
        )));
    }
    let version: u32 = parts
        .next()
        .and_then(|v| v.trim().parse().ok())
        .ok_or_else(|| BackupError::Format("missing/invalid cursor format version".into()))?;
    if version != CURSOR_FORMAT_VERSION {
        return Err(BackupError::Format(format!(
            "unsupported change-sink cursor version {} (this build reads/writes {})",
            version, CURSOR_FORMAT_VERSION
        )));
    }

    let body = lines
        .next()
        .ok_or_else(|| BackupError::Format("truncated change-sink cursor (no body)".into()))?;
    let digest_line = lines
        .next()
        .ok_or_else(|| BackupError::Format("truncated change-sink cursor (no digest)".into()))?;
    let digest: u64 = digest_line
        .strip_prefix("fnv1a64 ")
        .and_then(|hex| u64::from_str_radix(hex.trim(), 16).ok())
        .ok_or_else(|| BackupError::Format("invalid change-sink cursor digest line".into()))?;
    if digest != body_digest(body.as_bytes()) {
        return Err(BackupError::Corrupt(
            "change-sink cursor digest mismatch — the cursor file is corrupt".into(),
        ));
    }

    let seq = body
        .strip_prefix("acked ")
        .ok_or_else(|| BackupError::Format("missing `acked` line in change-sink cursor".into()))?;
    seq.trim()
        .parse::<u64>()
        .map(Some)
        .map_err(|_| BackupError::Format(format!("invalid acked seq {:?} in cursor", seq)))
}

/// Persists `seq` as the cursor watermark ATOMICALLY: write a temp file, (optionally)
/// fsync it, then rename it over the cursor path — so a crash mid-write leaves the
/// previous cursor intact rather than a torn one (which would fail closed on the next
/// open and stall the relay).
fn write_cursor(path: &Path, seq: u64, fsync: bool) -> Result<(), BackupError> {
    // [SONNET-4.6] positional format args (CodeQL rust/unused-variable false-positive).
    let body = format!("acked {}", seq);
    let text = format!(
        "{} {}\n{}\nfnv1a64 {:016x}\n",
        CURSOR_MAGIC,
        CURSOR_FORMAT_VERSION,
        body,
        body_digest(body.as_bytes())
    );

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let tmp = path.with_extension("cursor-tmp");
    {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp)?;
        f.write_all(text.as_bytes())?;
        if fsync {
            f.sync_all()?;
        }
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change_stream::Change;

    fn scratch(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "sparq-cdc-sink-{}-{}-{:?}",
            tag,
            std::process::id(),
            std::thread::current().id()
        ))
    }

    fn record(seq: u64, changes: Vec<Change>) -> ChangeRecord {
        ChangeRecord {
            seq,
            generation: seq + 1,
            timestamp_unix_nanos: 1_700_000_000_000_000_000,
            changes,
            rebase: false,
        }
    }

    fn insert(quad: &str) -> Change {
        Change {
            op: ChangeOp::Insert,
            quad: quad.to_string(),
        }
    }

    #[test]
    fn record_to_json_renders_ops_and_escapes_the_quad() {
        // A literal with an embedded quote + backslash + tab: raw N-Quads text that would
        // produce UNPARSEABLE JSON without escaping.
        let rec = record(
            7,
            vec![
                insert("<http://ex/s> <http://ex/p> \"a\\\"b\" ."),
                Change {
                    op: ChangeOp::Delete,
                    quad: "<http://ex/s> <http://ex/p> \"tab\there\" .".into(),
                },
            ],
        );
        let json = record_to_json(&rec);
        assert!(json.starts_with(
            "{\"seq\":7,\"generation\":8,\"commitTimestampNanos\":1700000000000000000,\"rebase\":false,\"changes\":["
        ));
        assert!(json.contains("\"op\":\"ADD\""), "insert renders ADD: {}", json);
        assert!(
            json.contains("\"op\":\"REMOVE\""),
            "delete renders REMOVE: {}",
            json
        );
        // The inner `"` and `\` are escaped, and the tab became \t — one line, valid JSON.
        assert!(json.contains(r#"\"a\\\"b\""#), "escaped quad: {}", json);
        assert!(json.contains("\\ttab") || json.contains("tab\\there"), "{}", json);
        assert!(!json.contains('\n'), "payload must be one line: {}", json);
        assert!(json.ends_with("]}"));
    }

    #[test]
    fn record_to_json_marks_a_rebase_gap_explicitly() {
        let mut rec = record(3, Vec::new());
        rec.rebase = true;
        let json = record_to_json(&rec);
        assert!(json.contains("\"rebase\":true"), "{}", json);
        assert!(json.contains("\"changes\":[]"), "{}", json);
    }

    #[test]
    fn escape_json_escapes_control_characters_as_unicode() {
        let mut out = String::new();
        escape_json_into("a\u{1}b\r\n", &mut out);
        assert_eq!(out, "a\\u0001b\\r\\n");
    }

    #[test]
    fn json_lines_sink_writes_one_line_per_record() {
        let mut sink = JsonLinesSink::new(Vec::<u8>::new());
        sink.deliver(&record(0, vec![insert("<http://ex/s> <http://ex/p> <http://ex/o> .")]))
            .expect("deliver 0");
        sink.deliver(&record(1, Vec::new())).expect("deliver 1");
        sink.flush().expect("flush");
        let text = String::from_utf8(sink.into_inner()).expect("utf-8");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2, "one line per record: {:?}", text);
        assert!(lines[0].contains("\"seq\":0"));
        assert!(lines[1].contains("\"seq\":1"));
    }

    #[test]
    fn sink_error_classifies_retryable_vs_permanent() {
        let transient: SinkError = io::Error::new(io::ErrorKind::BrokenPipe, "broker down").into();
        assert!(transient.is_retryable());
        assert!(transient.to_string().contains("retryable"));
        assert_eq!(transient.message(), "broker down");

        let fatal = SinkError::Permanent("topic does not exist".into());
        assert!(!fatal.is_retryable());
        assert!(fatal.to_string().contains("permanent"));
        assert_eq!(fatal.message(), "topic does not exist");
    }

    #[test]
    fn relay_error_display_names_the_offending_seq() {
        let e = RelayError::Sink {
            seq: 12,
            source: SinkError::Retryable("timeout".into()),
        };
        assert!(e.to_string().contains("seq 12"), "{}", e);
        assert!(RelayError::Poisoned("boom".into())
            .to_string()
            .contains("fail-closed"));
        assert!(RelayError::Log(BackupError::Format("x".into()))
            .to_string()
            .contains("could not read the log"));
        assert!(RelayError::Cursor(BackupError::Format("x".into()))
            .to_string()
            .contains("cursor error"));
    }

    #[test]
    fn cursor_round_trips_and_fails_closed_on_corruption() {
        let dir = scratch("cursor");
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("relay.cursor");

        // Absent file => a fresh relay (None), not an error.
        assert_eq!(read_cursor(&path).expect("absent cursor is fresh"), None);

        write_cursor(&path, 41, true).expect("write cursor");
        assert_eq!(read_cursor(&path).expect("read back"), Some(41));

        // Digest mismatch: flip the recorded seq without re-digesting.
        let poisoned = fs::read_to_string(&path)
            .expect("read")
            .replace("acked 41", "acked 99");
        fs::write(&path, poisoned).expect("write tampered");
        let err = read_cursor(&path).expect_err("tampered cursor must fail closed");
        assert!(
            matches!(err, BackupError::Corrupt(_)),
            "expected a corrupt-digest error, got {:?}",
            err
        );

        // Wrong magic: a cursor pointed at some other file fails closed too.
        fs::write(&path, "NOT-A-CURSOR 1\nacked 1\nfnv1a64 0\n").expect("write foreign");
        assert!(matches!(
            read_cursor(&path).expect_err("foreign file"),
            BackupError::Format(_)
        ));

        // Unknown version: never mis-read as v1.
        let body = "acked 1";
        fs::write(
            &path,
            format!(
                "{} 99\n{}\nfnv1a64 {:016x}\n",
                CURSOR_MAGIC,
                body,
                body_digest(body.as_bytes())
            ),
        )
        .expect("write v99");
        assert!(matches!(
            read_cursor(&path).expect_err("future version"),
            BackupError::Format(_)
        ));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn relay_config_default_is_bounded_and_fsyncing() {
        let cfg = RelayConfig::default();
        assert_eq!(cfg.max_records_per_pump, DEFAULT_MAX_RECORDS_PER_PUMP);
        assert!(cfg.fsync);
    }
}
