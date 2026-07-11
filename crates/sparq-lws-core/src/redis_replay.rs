// AUTHORED-BY Claude Opus 4.8
//! Distributed (shared) DPoP `jti` replay store backed by Redis — the horizontal-scaling enabler.
//!
//! ## Why this exists
//! The verifier's default [`InMemoryReplayStore`](solid_oidc_verifier::replay::InMemoryReplayStore) is
//! **per-instance**: a `jti` consumed on instance A is invisible to instance B, so the moment the
//! server is scaled horizontally behind a load balancer, replay protection silently breaks (a captured
//! DPoP proof can be replayed against a DIFFERENT instance within its freshness window). It also fails
//! CLOSED once its bounded in-memory set reaches capacity — a single-instance safety bound, not a
//! scaling story. A **shared** Redis set fixes both: every instance marks `jti`s in the one set, so a
//! replay is caught no matter which instance the proof first hit.
//!
//! ## Data model — one atomic round-trip, the NX reply IS the New/Replay signal
//! `SET dpop:jti:<jti> 1 NX PX <ttl_ms>`:
//! - `NX` makes the write happen ONLY if the key is absent. Redis replies with the value on a write
//!   (the key was new) and `nil` when the key already existed (a replay). That single reply IS the
//!   atomic check-and-set: `Some(..)` ⇒ [`MarkResult::New`](solid_oidc_verifier::replay::MarkResult::New), `nil`/`None` ⇒ [`MarkResult::Replay`](solid_oidc_verifier::replay::MarkResult::Replay).
//!   There is NO `GET`-then-`SET` race — the decision is made server-side in one command.
//! - `PX <ttl_ms>` sets the key's expiry to EXACTLY the `ttl` the verifier passes to `mark()` (the
//!   proof-freshness window). Once the key expires the `jti` is re-markable, mirroring the in-memory
//!   store's lazy-expiry semantics (a genuinely stale proof is independently rejected by the proof's
//!   own `iat` freshness check).
//! - The key is the **FULL** `jti` string (namespaced, never hashed/truncated): a hash collision would
//!   be either a false replay (reject a legitimate proof) or — worse — a missed replay (accept a
//!   captured one). `jti` is short and high-entropy; the full string is the only safe key.
//!
//! ## Fail-closed (non-negotiable)
//! ANY Redis error — pool exhaustion, connect timeout, command timeout, a malformed reply — returns a
//! [`ReplayBackendError`](solid_oidc_verifier::replay::ReplayBackendError), which the verifier maps to its existing 503 (`replay_fail_closed` defaults
//! true). We NEVER fail open: a fail-open Redis outage would be a GLOBAL replay-protection bypass
//! across the whole fleet. A slow Redis becomes a fast 503, never a worker pile-up (see the timeout).
//!
//! ## Off the async runtime (no worker-blocking)
//! [`ReplayStore::mark`](solid_oidc_verifier::replay::ReplayStore::mark) is a SYNC trait method called directly from inside the async axum handler's
//! Tokio runtime. We must NOT block a Tokio worker on the Redis RTT, and we must NOT call a blocking
//! Redis client from inside the runtime (it would either block a worker or, for an async client,
//! trip "runtime within a runtime"). We mirror the verifier's `net.rs` discipline EXACTLY: a dedicated
//! background OS thread owns an **r2d2 pool of blocking Redis connections** and serves `mark` jobs over
//! a channel; `mark` ships the job and blocks on a plain `std::sync::mpsc` reply (NOT a runtime entry),
//! so it is safe to call from inside the caller's runtime and never parks a Tokio worker on socket I/O.
//! A TIGHT op/connect timeout (default 50 ms) turns a slow/unreachable Redis into a fast 503.

use std::sync::mpsc::{Receiver, Sender, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use solid_oidc_verifier::replay::{
    InMemoryReplayStore, MarkResult, ReplayBackendError, ReplayStore,
};

/// The Redis key namespace for a DPoP `jti`. The full `jti` is appended verbatim.
const JTI_KEY_PREFIX: &str = "dpop:jti:";

/// Default per-operation SOCKET timeout (50 ms) — the HOT-PATH bound. Applied as the connection's
/// read/write timeout so a slow Redis fails an in-flight `SET` FAST (fail-closed → a quick 503), never
/// a worker pile-up. Tight by design.
pub const DEFAULT_OP_TIMEOUT: Duration = Duration::from_millis(50);

/// Floor for the pool's connection-ACQUISITION timeout (`pool.get()`), which covers TCP connect + the
/// redis handshake + r2d2's own scheduling overhead — a one-off cost paid only when a connection must
/// be established (the warm hot path leases an idle connection ~instantly). It is intentionally MORE
/// generous than [`DEFAULT_OP_TIMEOUT`] (the hot-path socket bound): a 50 ms hot-path bound is right for
/// an in-flight op, but the cold connection-establish + r2d2 scheduling legitimately needs more, so
/// the acquisition timeout is `max(op_timeout * 4, this floor)`. Still bounded, so an UNREACHABLE Redis
/// fails closed within it rather than blocking the worker indefinitely.
const POOL_ACQUIRE_TIMEOUT_FLOOR: Duration = Duration::from_millis(500);

/// The pool connection-acquisition timeout derived from the per-op timeout: comfortably above it (to
/// absorb establish + r2d2 overhead) but still bounded (fail-closed against an unreachable Redis).
fn pool_acquire_timeout(op_timeout: Duration) -> Duration {
    (op_timeout * 4).max(POOL_ACQUIRE_TIMEOUT_FLOOR)
}

/// The caller-side END-TO-END deadline `mark` waits for a reply (roborev Medium): even with the worker
/// pool, a burst larger than [`DEFAULT_WORKERS`] queues, so the calling (Tokio) thread must NOT block
/// indefinitely on the reply. This bounds the WHOLE round-trip — queue wait + pool acquisition + the
/// Redis op — so a saturated/slow backend fails CLOSED (`ReplayBackendError` → 503) promptly instead of
/// stalling the auth path. It must be at least the worst-case single-op cost (pool-acquire + op) plus a
/// margin for brief queueing; we use `pool_acquire_timeout + op_timeout`, floored, so it always exceeds
/// a single op's own bound (a healthy op never trips the caller deadline) yet stays bounded under load.
fn mark_deadline(op_timeout: Duration) -> Duration {
    (pool_acquire_timeout(op_timeout) + op_timeout).max(MARK_DEADLINE_FLOOR)
}

/// Floor for the caller-side `mark` deadline, so even a tiny op timeout leaves room for queue + op
/// before the caller gives up (fail-closed). Comfortably above the pool-acquire floor.
const MARK_DEADLINE_FLOOR: Duration = Duration::from_millis(750);

/// Number of dedicated Redis worker threads draining the shared job channel CONCURRENTLY (roborev
/// Medium: a single worker serialised all marks, so a slow Redis or bursty auth could queue requests
/// behind one worker far longer than the 50 ms socket timeout, stalling the auth path). With N workers,
/// up to N marks run their `SET NX PX` IN PARALLEL (each on its own pooled connection); the shared
/// `Receiver` mutex is held ONLY for the brief `recv()`, never during the Redis op, so the op
/// concurrency is genuinely N. Each op is still bounded by the tight socket timeout, so a slow Redis
/// degrades to fast 503s rather than a pile-up.
const DEFAULT_WORKERS: usize = 8;

/// r2d2 connection-pool size — one connection per worker so all workers can hold a connection at once
/// (a worker never waits on the pool for a peer's connection). Bounds total Redis connections.
const DEFAULT_POOL_SIZE: u32 = DEFAULT_WORKERS as u32;

/// BOUNDED job-queue capacity (roborev Medium: an unbounded channel let timed-out callers drop their
/// receivers while jobs kept accumulating → unbounded memory + stale marks executing long after the
/// request failed closed). The queue is a `sync_channel(BOUNDED)`; `mark` uses `try_send`, so once the
/// queue is full (the workers can't keep up) further marks fail CLOSED IMMEDIATELY (a fast 503,
/// backpressure) rather than enqueuing into an ever-growing backlog. Sized to absorb a normal burst
/// across the worker pool yet keep memory bounded (each queued job is a tiny `(String, Duration,
/// Sender)`); when full it is a genuine overload signal — exactly when failing closed fast is correct.
const DEFAULT_QUEUE_CAPACITY: usize = 1024;

/// A `mark` job sent to a Redis worker thread: the `jti`, its TTL, and a reply channel.
type MarkJob = (
    String,
    Duration,
    Sender<Result<MarkResult, ReplayBackendError>>,
);

/// A job sent to a Redis worker thread. Either a `mark` (atomic `SET NX PX` check-and-set) or a
/// READ-ONLY `contains` probe (`EXISTS`). Sharing the one worker pool + queue keeps the off-runtime
/// blocking-I/O discipline (no `mark` and no `contains` ever runs on the caller's async runtime) and a
/// single backpressure/fail-closed path for BOTH op kinds.
enum ReplayJob {
    /// Atomic check-and-set: `SET dpop:jti:<jti> 1 NX PX <ttl_ms>` → New/Replay. The authoritative op.
    Mark(MarkJob),
    /// READ-ONLY existence probe: `EXISTS dpop:jti:<jti>` → bool. NEVER mutates (no SET) — the
    /// optimization-hint pre-check seam (`ReplayStore::contains`). Reply carries `bool`.
    Contains(String, Sender<Result<bool, ReplayBackendError>>),
}

/// A distributed DPoP-`jti` replay store backed by a shared Redis (`SET NX PX`).
///
/// Construct with [`RedisReplayStore::connect`]. Cloning is cheap (the channel sender is cloneable);
/// every clone shares the one worker thread + pool. Implements [`ReplayStore`] so it drops into the
/// SAME `SharedReplay`/verifier/cache wiring the in-memory store uses (`main.rs` swap only).
pub struct RedisReplayStore {
    /// Ship a replay job (`mark` or `contains`) to a worker thread over a BOUNDED queue. Cloneable +
    /// `Send`/`Sync`, used from `&self`. `try_send` fails CLOSED on a full queue (backpressure; see
    /// [`DEFAULT_QUEUE_CAPACITY`]).
    tx: SyncSender<ReplayJob>,
    /// The caller-side end-to-end deadline `mark`/`contains` wait for a reply before failing closed
    /// (bounds the auth path even under queue saturation; see [`mark_deadline`]).
    mark_deadline: Duration,
}

impl RedisReplayStore {
    /// Connect to Redis at `url` (e.g. `redis://127.0.0.1:6379`) with the [`DEFAULT_OP_TIMEOUT`].
    ///
    /// Builds an r2d2 pool of BLOCKING connections (so the worker threads do ordinary blocking Redis I/O
    /// — never a Tokio runtime), spawns `DEFAULT_WORKERS` worker threads that share the pool + a single
    /// job channel (so up to N marks run their `SET NX PX` concurrently), and **eagerly validates one
    /// connection** so a misconfigured/unreachable Redis fails at boot (fail-closed) rather than only on
    /// the first authenticated request.
    pub fn connect(url: &str) -> Result<Self, ReplayBackendError> {
        Self::connect_with_timeout(url, DEFAULT_OP_TIMEOUT)
    }

    /// Connect with an explicit op/connect timeout (the test seam; production uses [`Self::connect`]).
    pub fn connect_with_timeout(
        url: &str,
        op_timeout: Duration,
    ) -> Result<Self, ReplayBackendError> {
        let client = redis::Client::open(url)
            .map_err(|e| ReplayBackendError(format!("redis client open failed: {e}")))?;

        // r2d2 pool of blocking connections. `connection_timeout` bounds how long `pool.get()` waits to
        // establish/lease a connection (TCP connect + redis handshake + r2d2 scheduling) — set to the
        // ACQUISITION timeout (more generous than the hot-path socket op bound, but still bounded so an
        // UNREACHABLE Redis fails CLOSED within it, never blocking the worker forever). The tight 50 ms
        // hot-path bound is applied separately as the per-connection read/write SOCKET timeout (see
        // `apply_timeouts`). `max_size` bounds Redis connections; `min_idle(0)` + `build_unchecked` means
        // connections are created LAZILY on demand (no eager establishment of all `max_size` at build,
        // which would block boot needlessly — we validate connectivity ourselves with one explicit PING
        // in `run_worker`, so boot still fails closed against a dead Redis).
        let pool = r2d2::Pool::builder()
            .max_size(DEFAULT_POOL_SIZE)
            .min_idle(Some(0))
            .connection_timeout(pool_acquire_timeout(op_timeout))
            .build_unchecked(client);

        // The shared BOUNDED job channel: `mark` sends here; the N worker threads drain it concurrently.
        // A std `sync_channel` (not Tokio): `mark` `try_send`s (fail-closed on a full queue) and then
        // waits on the REPLY channel, which is a plain timed recv (NOT a runtime entry), so calling it
        // from inside the caller's async runtime is safe. The bound caps memory under saturation (roborev
        // Medium). `Receiver` is single-consumer, so we share it across workers behind a `Mutex` — held
        // ONLY for the brief `recv()`, never during the Redis op, so the N workers' Redis ops run
        // genuinely in parallel.
        let (tx, rx) = std::sync::mpsc::sync_channel::<ReplayJob>(DEFAULT_QUEUE_CAPACITY);
        let shared_rx = Arc::new(Mutex::new(rx));

        // ONE eager init validation (connect + PING) on the boot thread BEFORE spawning workers, so a
        // misconfigured/unreachable Redis fails synchronously at boot (fail-closed) rather than silently
        // and only at first `mark`. Doing it here (not per-worker) keeps boot a single round-trip.
        validate_connection(&pool, op_timeout)?;

        // Spawn the worker pool. Each worker owns a clone of the pool handle (cheap `Arc`) + the shared
        // receiver, and loops serving marks until the channel closes (all `tx` senders dropped).
        for i in 0..DEFAULT_WORKERS {
            let pool = pool.clone();
            let rx = Arc::clone(&shared_rx);
            std::thread::Builder::new()
                .name(format!("solid-redis-replay-{i}"))
                .spawn(move || run_worker(pool, rx, op_timeout))
                .map_err(|e| {
                    ReplayBackendError(format!("redis replay worker spawn failed: {e}"))
                })?;
        }

        Ok(Self {
            tx,
            mark_deadline: mark_deadline(op_timeout),
        })
    }
}

impl Clone for RedisReplayStore {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            mark_deadline: self.mark_deadline,
        }
    }
}

impl RedisReplayStore {
    /// Enqueue a [`ReplayJob`] NON-BLOCKING onto the bounded queue, sharing the SAME backpressure /
    /// fail-closed posture for every op kind: `try_send` fails CLOSED immediately when the queue is FULL
    /// (the workers can't keep up — a genuine overload signal; fast error + backpressure rather than an
    /// unbounded backlog — roborev Medium) or when all workers are gone (Disconnected). The op never
    /// silently succeeds because the backend is unavailable/overloaded. `op` names the op for the error
    /// text.
    fn enqueue(&self, job: ReplayJob, op: &str) -> Result<(), ReplayBackendError> {
        self.tx.try_send(job).map_err(|e| match e {
            TrySendError::Full(_) => ReplayBackendError(format!(
                "redis replay queue is full (backend overloaded) — failing closed ({op})"
            )),
            TrySendError::Disconnected(_) => {
                ReplayBackendError(format!("redis replay workers are not available ({op})"))
            }
        })
    }

    /// Wait for a worker's reply with the BOUNDED end-to-end deadline (a plain channel `recv_timeout` —
    /// NOT a Tokio runtime entry, so safe inside the caller's async runtime). The Redis RTT happens on a
    /// worker thread, never on this one. The deadline bounds the WHOLE round-trip (queue wait + pool
    /// acquire + op): under a burst larger than the worker pool, or a slow backend, the caller fails
    /// CLOSED promptly instead of blocking the auth path indefinitely (roborev Medium). A healthy op
    /// replies well within the deadline, so this never fires on the happy path. A timeout or a dropped
    /// reply (worker died mid-op) both fail CLOSED. `op` names the op for the error text.
    fn await_reply<T>(
        &self,
        reply_rx: Receiver<Result<T, ReplayBackendError>>,
        op: &str,
    ) -> Result<T, ReplayBackendError> {
        match reply_rx.recv_timeout(self.mark_deadline) {
            Ok(result) => result,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(ReplayBackendError(format!(
                "redis replay {op} exceeded the {} ms deadline (backend saturated/slow) — failing closed",
                self.mark_deadline.as_millis()
            ))),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Err(ReplayBackendError(format!(
                "redis replay worker dropped the request ({op})"
            ))),
        }
    }
}

impl ReplayStore for RedisReplayStore {
    fn mark(&self, jti: &str, ttl: Duration) -> Result<MarkResult, ReplayBackendError> {
        // A non-positive TTL means the proof is already past its freshness window: mirror the in-memory
        // store and treat it as fresh WITHOUT touching Redis (the proof's own `iat` check rejects it
        // independently; a `PX 0` would be a malformed Redis command anyway).
        if ttl <= Duration::ZERO {
            return Ok(MarkResult::New);
        }

        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        self.enqueue(ReplayJob::Mark((jti.to_string(), ttl, reply_tx)), "mark")?;
        self.await_reply(reply_rx, "mark")
    }

    /// READ-ONLY existence probe → a single Redis `EXISTS dpop:jti:<jti>` on a worker thread. NEVER a
    /// `SET`/`SET NX` (it must not mark) — the authoritative, mutating check-and-set is `mark`. Same
    /// fail-closed-on-error posture as `mark`: a queue-full / disconnect / timeout / Redis error all map
    /// to a `ReplayBackendError`, NEVER a false `Ok(false)` ("not seen") that could mask a replay. This
    /// is an OPTIMIZATION-hint seam (the held opt-4 jti-precheck) and is NOT wired into the auth path in
    /// this server today — but it matches `mark`'s error semantics so a future consumer can treat a probe
    /// error conservatively. Goes through the SAME worker pool/queue (off-runtime blocking I/O).
    fn contains(&self, jti: &str) -> Result<bool, ReplayBackendError> {
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        self.enqueue(ReplayJob::Contains(jti.to_string(), reply_tx), "contains")?;
        self.await_reply(reply_rx, "contains")
    }
}

/// Eagerly validate ONE connection (lease + bounded PING) so an unreachable/misconfigured Redis fails
/// CLOSED at boot rather than only on the first authenticated request. Called once on the boot thread
/// before the workers spawn.
fn validate_connection(
    pool: &r2d2::Pool<redis::Client>,
    op_timeout: Duration,
) -> Result<(), ReplayBackendError> {
    let mut conn = pool
        .get()
        .map_err(|e| ReplayBackendError(format!("redis pool connect failed at init: {e}")))?;
    apply_timeouts(&mut conn, op_timeout)
        .map_err(|e| ReplayBackendError(format!("redis connection timeout setup failed: {e}")))?;
    redis::cmd("PING")
        .query::<()>(&mut *conn)
        .map_err(|e| ReplayBackendError(format!("redis PING failed at init: {e}")))
}

/// A worker thread's loop: drain the SHARED job channel and serve each `mark` as one `SET NX PX`
/// round-trip on a pooled connection, until the channel closes (all senders dropped). The receiver
/// mutex is held ONLY for the brief `recv()` — NOT during the Redis op — so N workers' Redis ops run
/// genuinely in parallel (no head-of-line blocking behind a slow op). All blocking Redis I/O happens
/// HERE, off the Tokio runtime. A requester that has gone away just drops the result.
fn run_worker(
    pool: r2d2::Pool<redis::Client>,
    rx: Arc<Mutex<Receiver<ReplayJob>>>,
    op_timeout: Duration,
) {
    loop {
        // Lock ONLY to pull the next job, then release BEFORE the Redis op so peers can pull theirs and
        // run concurrently. A poisoned mutex (a peer worker panicked mid-`recv`) ends this worker (the
        // others, and boot-time validation, keep the fail-closed contract). `recv()` returns `Err` when
        // the channel is closed (the store dropped) → the worker exits cleanly.
        let job = match rx.lock() {
            Ok(guard) => guard.recv(),
            Err(_) => return,
        };
        match job {
            Ok(ReplayJob::Mark((jti, ttl, reply))) => {
                let _ = reply.send(mark_one(&pool, &jti, ttl, op_timeout));
            }
            Ok(ReplayJob::Contains(jti, reply)) => {
                let _ = reply.send(contains_one(&pool, &jti, op_timeout));
            }
            Err(_) => return, // channel closed: store dropped, no more work.
        }
    }
}

/// Perform ONE atomic `SET dpop:jti:<jti> 1 NX PX <ttl_ms>` on a pooled blocking connection.
///
/// The `NX` reply is the New/Replay signal in a single round-trip: a non-nil reply ⇒ the key was set
/// (NEW), a `nil` reply ⇒ the key already existed (REPLAY). Any pool/connection/command error returns
/// a [`ReplayBackendError`] (fail-closed). The full `jti` is the key (namespaced, never hashed).
fn mark_one(
    pool: &r2d2::Pool<redis::Client>,
    jti: &str,
    ttl: Duration,
    op_timeout: Duration,
) -> Result<MarkResult, ReplayBackendError> {
    // Get a pooled connection. Pool exhaustion / connect failure within `connection_timeout` ⇒ error
    // ⇒ fail-closed.
    let mut conn = pool
        .get()
        .map_err(|e| ReplayBackendError(format!("redis pool get failed: {e}")))?;

    // Apply the read/write socket timeouts so a hung Redis can't wedge the worker — a slow op errors
    // out within the timeout and fails closed.
    apply_timeouts(&mut conn, op_timeout)
        .map_err(|e| ReplayBackendError(format!("redis connection timeout setup failed: {e}")))?;

    let key = format!("{JTI_KEY_PREFIX}{jti}");
    // PX millisecond expiry = EXACTLY the ttl the verifier passed. Round UP to >=1 ms so a sub-ms but
    // positive ttl never collapses to `PX 0` (which Redis rejects) — it always sets a real expiry.
    let ttl_ms = ttl_millis_at_least_one(ttl);

    // `SET key 1 NX PX <ms>` — the value `1` is a placeholder (presence is all that matters). Typed as
    // `Option<String>`: `Some(_)` ⇒ the SET happened (key was absent) ⇒ NEW; `None` (nil) ⇒ the key
    // already existed ⇒ REPLAY. This is the whole atomic check-and-set, server-side, race-free.
    let set: redis::RedisResult<Option<String>> = redis::cmd("SET")
        .arg(&key)
        .arg(1)
        .arg("NX")
        .arg("PX")
        .arg(ttl_ms)
        .query(&mut *conn);

    match set {
        Ok(Some(_)) => Ok(MarkResult::New),
        Ok(None) => Ok(MarkResult::Replay),
        Err(e) => Err(ReplayBackendError(format!("redis SET NX failed: {e}"))),
    }
}

/// Perform ONE READ-ONLY `EXISTS dpop:jti:<jti>` on a pooled blocking connection → `Ok(true)` iff the
/// key is currently present (a still-live, already-marked `jti`), `Ok(false)` otherwise (incl. an
/// expired key, which Redis has already evicted — consistent with `mark` treating an expired `jti` as
/// fresh). 🔒 NEVER a `SET`/`SET NX`: this MUST NOT mark, insert, refresh, or evict any replay state
/// (the [`ReplayStore::contains`] INV-4 read-only contract). Any pool/connection/command error returns
/// a [`ReplayBackendError`] (fail-closed, mirroring [`mark_one`]) — never a false `Ok(false)`. The full
/// `jti` is the key (namespaced, never hashed), matching `mark_one`.
fn contains_one(
    pool: &r2d2::Pool<redis::Client>,
    jti: &str,
    op_timeout: Duration,
) -> Result<bool, ReplayBackendError> {
    let mut conn = pool
        .get()
        .map_err(|e| ReplayBackendError(format!("redis pool get failed: {e}")))?;

    apply_timeouts(&mut conn, op_timeout)
        .map_err(|e| ReplayBackendError(format!("redis connection timeout setup failed: {e}")))?;

    let key = format!("{JTI_KEY_PREFIX}{jti}");
    // `EXISTS key` → 1 if present, 0 if absent (or expired — Redis evicts before reporting). A strictly
    // non-mutating read: it records nothing, so two requests racing a fresh jti can both observe false
    // here (only `mark`'s atomic `SET NX` resolves the race — exactly one New).
    let exists: redis::RedisResult<i64> = redis::cmd("EXISTS").arg(&key).query(&mut *conn);

    match exists {
        Ok(n) => Ok(n > 0),
        Err(e) => Err(ReplayBackendError(format!("redis EXISTS failed: {e}"))),
    }
}

/// Convert a positive `ttl` to whole milliseconds, clamped to AT LEAST 1 (so a sub-millisecond but
/// positive ttl never produces `PX 0`, which Redis rejects) and saturated to `u64::MAX` on overflow.
/// `mark` already returned early for a non-positive ttl, so this only ever sees `ttl > 0`.
fn ttl_millis_at_least_one(ttl: Duration) -> u64 {
    let ms = ttl.as_millis();
    if ms == 0 {
        1
    } else {
        u64::try_from(ms).unwrap_or(u64::MAX)
    }
}

/// Apply the op timeout as the connection's read + write socket timeouts, so a hung Redis fails the op
/// within the bound (fail-closed) instead of blocking the worker thread indefinitely.
fn apply_timeouts(conn: &mut redis::Connection, op_timeout: Duration) -> redis::RedisResult<()> {
    conn.set_read_timeout(Some(op_timeout))?;
    conn.set_write_timeout(Some(op_timeout))?;
    Ok(())
}

/// A single concrete [`ReplayStore`] type that dispatches to EITHER the verifier's per-instance
/// in-memory store OR the distributed [`RedisReplayStore`], decided at boot from config.
///
/// This is the seam that lets `main.rs` keep ONE monomorphised replay type (`SharedReplay<BackendReplay>`)
/// regardless of backend — the verifier, the token cache, the `AppState`, and `build_router` are all
/// generic over `R: ReplayStore`, so a single concrete `R` keeps the whole wiring unchanged. The
/// in-memory arm is byte-for-byte the existing behaviour (it forwards verbatim to `InMemoryReplayStore`),
/// so the DEFAULT (no Redis URL) path — and thus conformance — is unchanged; only the `mark` call gains
/// one cheap enum match. The Redis arm is selected ONLY when an operator sets the Redis URL.
pub enum BackendReplay {
    /// The default per-instance store (single-node v1). Unchanged behaviour; the default path.
    InMemory(InMemoryReplayStore),
    /// The shared, distributed Redis store (`SET NX PX`) — the horizontal-scaling backend.
    Redis(RedisReplayStore),
}

impl ReplayStore for BackendReplay {
    fn mark(&self, jti: &str, ttl: Duration) -> Result<MarkResult, ReplayBackendError> {
        match self {
            BackendReplay::InMemory(s) => s.mark(jti, ttl),
            BackendReplay::Redis(s) => s.mark(jti, ttl),
        }
    }

    /// READ-ONLY existence probe — delegates verbatim to the selected backend's `contains` (the
    /// in-memory `EXISTS`-equivalent map read, or the Redis `EXISTS`). Non-mutating on both arms.
    fn contains(&self, jti: &str) -> Result<bool, ReplayBackendError> {
        match self {
            BackendReplay::InMemory(s) => s.contains(jti),
            BackendReplay::Redis(s) => s.contains(jti),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ttl_millis_clamps_to_at_least_one() {
        // A sub-millisecond positive ttl must not collapse to PX 0.
        assert_eq!(ttl_millis_at_least_one(Duration::from_nanos(1)), 1);
        assert_eq!(ttl_millis_at_least_one(Duration::from_micros(500)), 1);
        // A whole-millisecond ttl passes through.
        assert_eq!(ttl_millis_at_least_one(Duration::from_millis(1)), 1);
        assert_eq!(ttl_millis_at_least_one(Duration::from_millis(250)), 250);
        assert_eq!(
            ttl_millis_at_least_one(Duration::from_secs(330)),
            330_000_u64
        );
    }

    #[test]
    fn key_uses_full_jti_namespaced() {
        // Document the keying contract: full jti, never hashed/truncated.
        let jti = "abc.def-GHI_123~unusual";
        assert_eq!(
            format!("{JTI_KEY_PREFIX}{jti}"),
            "dpop:jti:abc.def-GHI_123~unusual"
        );
    }

    #[test]
    fn mark_deadline_exceeds_a_single_op_bound() {
        // The caller-side end-to-end deadline must ALWAYS exceed a single op's own worst-case bound
        // (pool-acquire + op), so a healthy op never trips the caller deadline, yet it stays bounded
        // (fail-closed) under saturation.
        for op_ms in [10u64, 50, 100, 500] {
            let op = Duration::from_millis(op_ms);
            let deadline = mark_deadline(op);
            let single_op_worst = pool_acquire_timeout(op) + op;
            assert!(
                deadline >= single_op_worst,
                "mark deadline ({deadline:?}) must be >= a single op's worst case ({single_op_worst:?})"
            );
            assert!(
                deadline >= MARK_DEADLINE_FLOOR,
                "mark deadline must respect its floor"
            );
        }
    }

    #[test]
    fn pool_acquire_timeout_exceeds_op_timeout() {
        // Connection acquisition (cold connect + r2d2 overhead) is always given more headroom than the
        // tight hot-path socket op bound.
        for op_ms in [10u64, 50, 200, 1000] {
            let op = Duration::from_millis(op_ms);
            assert!(
                pool_acquire_timeout(op) >= op,
                "pool acquire timeout must be >= the op timeout"
            );
            assert!(pool_acquire_timeout(op) >= POOL_ACQUIRE_TIMEOUT_FLOOR);
        }
    }

    #[test]
    fn connect_to_unreachable_redis_fails_closed() {
        // An unreachable Redis must fail at CONNECT (fail-closed), not silently succeed. Port 1 is
        // reserved/unused; the tight timeout makes this fast. (No live Redis needed for this assertion.)
        let res = RedisReplayStore::connect_with_timeout(
            "redis://127.0.0.1:1",
            Duration::from_millis(50),
        );
        assert!(
            res.is_err(),
            "connecting to an unreachable Redis must fail closed, got Ok"
        );
    }

    #[test]
    fn backend_replay_inmemory_arm_forwards_verbatim() {
        // The `BackendReplay::InMemory` arm must behave EXACTLY like the underlying in-memory store:
        // first mark of a jti ⇒ New, a second within the window ⇒ Replay. This is what guarantees the
        // default (no-Redis) path — and thus conformance — is unchanged by introducing the enum.
        let store =
            BackendReplay::InMemory(InMemoryReplayStore::with_window(Duration::from_secs(60)));
        let ttl = Duration::from_secs(60);
        assert_eq!(store.mark("jti-A", ttl).unwrap(), MarkResult::New);
        assert_eq!(
            store.mark("jti-A", ttl).unwrap(),
            MarkResult::Replay,
            "a repeated jti within its window must be reported as a replay"
        );
        assert_eq!(
            store.mark("jti-B", ttl).unwrap(),
            MarkResult::New,
            "a distinct jti must be fresh"
        );
    }
}
