//! The opaque broker state machine (§8.4).
//!
//! [`Broker`] is a pure, clock-free state machine: every entry point takes the
//! current unix second as an argument, so retention, validity, and rate limiting
//! are deterministic and testable, and the daemon is the only thing that reads a
//! real clock.
//!
//! ## What this broker enforces
//!
//! * **Negotiation** before anything else (`Hello`/`HelloAck`), with no silent
//!   algorithm substitution — an unknown suite is rejected, never downgraded.
//! * **Routing context**: block and topic operations are scoped to a topic the
//!   session has opened with `OpenRepo`.
//! * **Publisher admission**: a publish is accepted only if its key is admitted
//!   for the topic/epoch by an admin-signed [`AdmissionGrant`] *whose validity
//!   window still covers the publication time*, *and* the announcement's own
//!   signature verifies, *and* the announced root block is already stored.
//! * **Epoch advance**: only under the admin key pinned for the topic, only
//!   strictly forward, and only to a fresh topic.
//! * **Limits and rate**: every negotiated ceiling is checked before work is done.
//! * **Retention**: unpinned, aged, or over-budget blocks are collectable under
//!   the advertised policy.
//!
//! ## What this broker cannot do, by construction
//!
//! It holds no key material and never sees `K_read`, a private key, RDF, or
//! SPARQL — none of the messages it accepts have a field for any of those. It
//! cannot decrypt a block, cannot evaluate a query, and does not link the query
//! engine (enforced by `tests/boundary.rs`).
//!
//! ## Honest limits of the admission model
//!
//! The trust anchor for a topic is **trust-on-first-use**: the first valid
//! admission grant naming a topic pins that grant's admin key, and every later
//! grant and epoch advance for that topic must verify under the pinned key. Topic
//! ids are 256-bit CSPRNG values (§4.1), so squatting one means guessing it — but
//! TOFU is a bootstrap, not a proof, and a deployment that can pre-provision
//! should call [`Broker::pin_admin`] instead. Nothing here is claimed sound; the
//! whole profile is research-grade and externally unaudited (`sq-qhy4`).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use sparq_e2ee_ng::broker_protocol::{
    hello_v0, protocol_limits, AdmissionGrant, BrokerError, ErrorCode, Event, HeaderMode, Hello,
    HelloAck, OpenRepo, PinRepo, PinStatus, PublishEvent, Request, Response, RetentionPolicy,
    TopicSub, TopicSyncReq, TopicSyncResp, WireLimits, PROTOCOL_V0,
};
use sparq_e2ee_ng::envelope::BlockEnvelope;
use sparq_e2ee_ng::ids::{BlockId, CommitId, Epoch, PeerId, TopicId};
use sparq_e2ee_ng::sign::{PublicVerifyingKey, PUBLIC_KEY_LEN};
use sparq_e2ee_ng::suite::SUITE_V0;

use crate::log::{IdPrefix, LogRecord, MetadataLog, NullLog, Outcome};
use crate::store::{BlockStore, GcReport};

/// A broker-local session handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(pub u64);

/// Broker configuration: the limits, retention policy, and header mode it will
/// advertise and enforce.
#[derive(Debug, Clone)]
pub struct BrokerConfig {
    /// Session limits advertised in `HelloAck` and enforced on every request.
    pub limits: WireLimits,
    /// Advertised retention policy.
    pub retention: RetentionPolicy,
    /// Header modes this broker accepts, most-preferred first. The v0 default is
    /// `[Opaque]`: accepting [`HeaderMode::Clear`] means accepting that the
    /// broker learns the commit DAG shape (§5), so it is an explicit opt-in.
    pub header_modes: Vec<HeaderMode>,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        BrokerConfig {
            limits: WireLimits::default(),
            retention: RetentionPolicy::default(),
            header_modes: vec![HeaderMode::Opaque],
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Negotiated {
    header_mode: HeaderMode,
}

#[derive(Debug, Clone, Copy)]
struct Routing {
    topic: TopicId,
    epoch: Epoch,
    peer: PeerId,
}

#[derive(Debug, Default)]
struct Session {
    negotiated: Option<Negotiated>,
    routing: Option<Routing>,
    opened: BTreeSet<TopicId>,
    subs: BTreeSet<TopicId>,
    outbox: VecDeque<Response>,
    window_start: u64,
    requests_in_window: u64,
}

#[derive(Debug)]
struct TopicState {
    epoch: Epoch,
    admin_pub: [u8; PUBLIC_KEY_LEN],
    /// Admitted publisher keys, each mapped to the `(not_before, not_after)`
    /// windows that admitted it. A publication is admitted only while some
    /// window covers its time, so an admission stops working when the grant
    /// that carried it expires.
    ///
    /// Repeated or overlapping grants **union**: registering a grant adds its
    /// window and never narrows an existing one, so a replayed narrow grant
    /// cannot cut an admission short (revocation is an epoch advance, which
    /// clears the map). Because it is a union of windows and not their hull, a
    /// gap between two disjoint grants is *not* admitted. Windows that had
    /// already expired are dropped whenever a grant is registered, so the set
    /// does not accumulate dead grants.
    publishers: BTreeMap<[u8; PUBLIC_KEY_LEN], BTreeSet<(u64, u64)>>,
    pinned: bool,
    /// Set once an epoch advance retires this topic in favour of another.
    retired_to: Option<TopicId>,
    cursor: u64,
    events: Vec<Event>,
    commit_root: BTreeMap<CommitId, BlockId>,
    /// Commit ids that some announcement declared as a parent. Only ever
    /// populated in [`HeaderMode::Clear`] — in the opaque default the broker has
    /// no parent information at all.
    non_heads: BTreeSet<CommitId>,
}

/// The opaque broker.
pub struct Broker<L: MetadataLog = NullLog> {
    cfg: BrokerConfig,
    topics: BTreeMap<TopicId, TopicState>,
    store: BlockStore,
    sessions: BTreeMap<SessionId, Session>,
    next_session: u64,
    log: L,
}

impl Default for Broker<NullLog> {
    fn default() -> Self {
        Broker::new(BrokerConfig::default(), NullLog)
    }
}

type Handled = (Response, Outcome, Option<TopicId>, u64);

impl<L: MetadataLog> Broker<L> {
    /// New broker with `cfg` and a metadata-safe log sink.
    pub fn new(cfg: BrokerConfig, log: L) -> Self {
        Broker {
            cfg,
            topics: BTreeMap::new(),
            store: BlockStore::new(),
            sessions: BTreeMap::new(),
            next_session: 1,
            log,
        }
    }

    /// The configuration in force.
    pub fn config(&self) -> &BrokerConfig {
        &self.cfg
    }

    /// Borrow the log sink (tests assert on what it did *not* record).
    pub fn log(&self) -> &L {
        &self.log
    }

    /// Pre-provision a topic's admin trust anchor, bypassing trust-on-first-use.
    /// Returns `false` if the topic already has a different pin.
    pub fn pin_admin(
        &mut self,
        topic: TopicId,
        admin_pub: [u8; PUBLIC_KEY_LEN],
        epoch: Epoch,
    ) -> bool {
        match self.topics.get(&topic) {
            Some(t) => t.admin_pub == admin_pub,
            None => {
                self.topics.insert(topic, TopicState::new(epoch, admin_pub));
                true
            }
        }
    }

    /// Open a new session.
    pub fn open_session(&mut self) -> SessionId {
        let id = SessionId(self.next_session);
        self.next_session += 1;
        self.sessions.insert(id, Session::default());
        id
    }

    /// Close a session, dropping its subscriptions and undelivered fan-out.
    pub fn close_session(&mut self, id: SessionId) {
        self.sessions.remove(&id);
    }

    /// Take the fan-out messages queued for a session (each is a [`Response`]
    /// that the transport writes with `request_id = 0`).
    pub fn take_pushes(&mut self, id: SessionId) -> Vec<Response> {
        match self.sessions.get_mut(&id) {
            Some(s) => s.outbox.drain(..).collect(),
            None => Vec::new(),
        }
    }

    /// Decode one framed request and handle it, returning the framed response.
    ///
    /// A frame that exceeds the negotiated message ceiling, or that fails to
    /// decode, is answered with [`ErrorCode::Protocol`] echoing `request_id = 0`
    /// (the id is inside the frame that would not parse, so there is nothing
    /// honest to echo).
    pub fn handle_frame(&mut self, session: SessionId, now: u64, frame: &[u8]) -> Vec<u8> {
        let limits = protocol_limits(
            self.cfg.limits.max_block_bytes as usize,
            self.cfg.limits.max_ids_per_request as usize,
        );
        if frame.len() as u64 > self.cfg.limits.max_message_bytes {
            self.log.record(&LogRecord {
                session: session.0,
                kind: "frame",
                topic: None,
                peer: None,
                bytes: frame.len() as u64,
                count: 0,
                outcome: Outcome::Rejected("LimitExceeded"),
            });
            return Response::Error(BrokerError::new(
                ErrorCode::LimitExceeded,
                "frame exceeds negotiated max_message_bytes",
            ))
            .encode(0);
        }
        match Request::decode(frame, limits) {
            Ok((request_id, req)) => {
                let resp = self.handle(session, now, req, frame.len() as u64);
                resp.encode(request_id)
            }
            Err(_) => {
                self.log.record(&LogRecord {
                    session: session.0,
                    kind: "frame",
                    topic: None,
                    peer: None,
                    bytes: frame.len() as u64,
                    count: 0,
                    outcome: Outcome::Rejected("Protocol"),
                });
                Response::Error(BrokerError::new(
                    ErrorCode::Protocol,
                    "frame is not a canonical protocol request",
                ))
                .encode(0)
            }
        }
    }

    /// Handle an already-decoded request. `bytes` is the encoded request size,
    /// used only for the metadata log.
    pub fn handle(&mut self, session: SessionId, now: u64, req: Request, bytes: u64) -> Response {
        let kind = request_kind_label(&req);
        let peer = self
            .sessions
            .get(&session)
            .and_then(|s| s.routing.map(|r| IdPrefix::of(r.peer.as_bytes())));
        let (resp, outcome, topic, count) = self.dispatch(session, now, req);
        self.log.record(&LogRecord {
            session: session.0,
            kind,
            topic: topic.map(|t| IdPrefix::of(t.as_bytes())),
            peer,
            bytes,
            count,
            outcome,
        });
        resp
    }

    fn dispatch(&mut self, session: SessionId, now: u64, req: Request) -> Handled {
        if !self.sessions.contains_key(&session) {
            return (
                err(ErrorCode::NoRouting, "unknown session"),
                Outcome::Rejected("NoRouting"),
                None,
                0,
            );
        }
        if let Some(e) = self.charge_rate(session, now) {
            return (e, Outcome::Rejected("RateLimited"), None, 0);
        }
        // Negotiation must complete before anything else.
        let negotiated = self.sessions[&session].negotiated.is_some();
        if !negotiated && !matches!(req, Request::Hello(_)) {
            return (
                err(ErrorCode::NotNegotiated, "Hello must precede any request"),
                Outcome::Rejected("NotNegotiated"),
                None,
                0,
            );
        }
        match req {
            Request::Hello(h) => self.on_hello(session, h),
            Request::OpenRepo(o) => self.on_open(session, now, o),
            Request::PinRepo(p) => self.on_pin(session, p),
            Request::RepoPinStatus { topic } => self.on_pin_status(session, topic),
            Request::TopicSub(s) => self.on_sub(session, s),
            Request::TopicUnsub { topic } => self.on_unsub(session, topic),
            Request::TopicSyncReq(q) => self.on_sync(session, q),
            Request::BlocksExist { ids } => self.on_exist(session, ids),
            Request::BlocksGet { ids } => self.on_get(session, now, ids),
            Request::BlocksPut { envelopes } => self.on_put(session, now, envelopes),
            Request::CommitGet { commit_ids } => self.on_commit_get(session, now, commit_ids),
            Request::PublishEvent(p) => self.on_publish(session, now, p),
            Request::EpochAdvance(a) => self.on_epoch_advance(session, a),
            _ => (
                err(ErrorCode::Unsupported, "unsupported request kind"),
                Outcome::Rejected("Unsupported"),
                None,
                0,
            ),
        }
    }

    // ---- rate limiting ----------------------------------------------------

    fn charge_rate(&mut self, session: SessionId, now: u64) -> Option<Response> {
        let window = self.cfg.limits.rate_window_secs.max(1);
        let cap = self.cfg.limits.max_requests_per_window;
        let s = self.sessions.get_mut(&session)?;
        if now.saturating_sub(s.window_start) >= window {
            s.window_start = now;
            s.requests_in_window = 0;
        }
        s.requests_in_window += 1;
        if s.requests_in_window > cap {
            return Some(err(
                ErrorCode::RateLimited,
                "session exceeded its request allowance for the window",
            ));
        }
        None
    }

    // ---- negotiation ------------------------------------------------------

    fn on_hello(&mut self, session: SessionId, h: Hello) -> Handled {
        if !h.versions.contains(&PROTOCOL_V0) {
            return (
                err(ErrorCode::Unsupported, "no common protocol version"),
                Outcome::Rejected("Unsupported"),
                None,
                0,
            );
        }
        // Algorithm agility with NO silent substitution: the suite must be one the
        // client offered AND the one this build binds.
        if !h.suites.iter().any(|s| s == SUITE_V0) {
            return (
                err(ErrorCode::Unsupported, "no common crypto suite"),
                Outcome::Rejected("Unsupported"),
                None,
                0,
            );
        }
        let Some(mode) = h
            .header_modes
            .iter()
            .copied()
            .find(|m| self.cfg.header_modes.contains(m))
        else {
            return (
                err(ErrorCode::Unsupported, "no common routing header mode"),
                Outcome::Rejected("Unsupported"),
                None,
                0,
            );
        };
        if let Some(s) = self.sessions.get_mut(&session) {
            s.negotiated = Some(Negotiated { header_mode: mode });
        }
        (
            Response::HelloAck(HelloAck {
                version: PROTOCOL_V0,
                suite: SUITE_V0.to_string(),
                header_mode: mode,
                limits: self.cfg.limits,
                padding_classes: hello_v0(self.cfg.limits.max_block_bytes).padding_classes,
                retention: self.cfg.retention,
            }),
            Outcome::Ok,
            None,
            0,
        )
    }

    // ---- routing / admission ---------------------------------------------

    fn on_open(&mut self, session: SessionId, now: u64, o: OpenRepo) -> Handled {
        let topic = o.topic;
        if let Some(g) = &o.auth {
            if let Some(e) = self.register_grant(g, topic, o.epoch, now) {
                let label = code_label(&e);
                return (e, Outcome::Rejected(label), Some(topic), 0);
            }
        }
        let Some(t) = self.topics.get(&topic) else {
            return (
                err(ErrorCode::NotFound, "unknown topic"),
                Outcome::Rejected("NotFound"),
                Some(topic),
                0,
            );
        };
        if t.retired_to.is_some() {
            return (
                err(ErrorCode::Retired, "topic retired by an epoch advance"),
                Outcome::Rejected("Retired"),
                Some(topic),
                0,
            );
        }
        if t.epoch != o.epoch {
            return (
                err(ErrorCode::EpochMismatch, "topic is at a different epoch"),
                Outcome::Rejected("EpochMismatch"),
                Some(topic),
                0,
            );
        }
        if let Some(s) = self.sessions.get_mut(&session) {
            s.routing = Some(Routing {
                topic,
                epoch: o.epoch,
                peer: o.peer,
            });
            s.opened.insert(topic);
        }
        (Response::Ok, Outcome::Ok, Some(topic), 0)
    }

    /// Validate an admission grant and fold it into topic admission state.
    /// Returns `Some(error_response)` on rejection.
    fn register_grant(
        &mut self,
        g: &AdmissionGrant,
        topic: TopicId,
        epoch: Epoch,
        now: u64,
    ) -> Option<Response> {
        if g.topic != topic || g.epoch != epoch {
            return Some(err(
                ErrorCode::NotAdmitted,
                "admission grant does not match the opened topic/epoch",
            ));
        }
        if !g.is_valid_at(now) {
            return Some(err(
                ErrorCode::NotAdmitted,
                "admission grant is outside its validity window",
            ));
        }
        if g.verify_self().is_err() {
            return Some(err(
                ErrorCode::BadSignature,
                "admission grant signature did not verify",
            ));
        }
        match self.topics.get_mut(&topic) {
            Some(t) => {
                if t.admin_pub != g.admin_pub {
                    return Some(err(
                        ErrorCode::NotAdmitted,
                        "admission grant is not under the topic's pinned admin key",
                    ));
                }
                if t.retired_to.is_some() {
                    return Some(err(
                        ErrorCode::Retired,
                        "topic retired by an epoch advance",
                    ));
                }
                if t.epoch != epoch {
                    return Some(err(
                        ErrorCode::EpochMismatch,
                        "topic is at a different epoch",
                    ));
                }
                if let Some(p) = g.publisher_pub {
                    t.admit(p, (g.validity.not_before, g.validity.not_after), now);
                }
            }
            None => {
                // Trust-on-first-use: this grant pins the topic's admin key.
                let mut t = TopicState::new(epoch, g.admin_pub);
                if let Some(p) = g.publisher_pub {
                    t.admit(p, (g.validity.not_before, g.validity.not_after), now);
                }
                self.topics.insert(topic, t);
            }
        }
        None
    }

    // ---- pinning / retention ---------------------------------------------

    fn on_pin(&mut self, session: SessionId, p: PinRepo) -> Handled {
        if let Some(e) = self.require_opened(session, p.topic) {
            return (e, Outcome::Rejected("NoRouting"), Some(p.topic), 0);
        }
        match self.topics.get_mut(&p.topic) {
            Some(t) => {
                t.pinned = p.pin;
                (Response::Ok, Outcome::Ok, Some(p.topic), 0)
            }
            None => (
                err(ErrorCode::NotFound, "unknown topic"),
                Outcome::Rejected("NotFound"),
                Some(p.topic),
                0,
            ),
        }
    }

    fn on_pin_status(&mut self, session: SessionId, topic: TopicId) -> Handled {
        if let Some(e) = self.require_opened(session, topic) {
            return (e, Outcome::Rejected("NoRouting"), Some(topic), 0);
        }
        let Some(t) = self.topics.get(&topic) else {
            return (
                err(ErrorCode::NotFound, "unknown topic"),
                Outcome::Rejected("NotFound"),
                Some(topic),
                0,
            );
        };
        let (blocks, bytes) = self.store.usage(&topic);
        (
            Response::PinStatus(PinStatus {
                topic,
                pinned: t.pinned,
                blocks,
                bytes,
                retention: self.cfg.retention,
            }),
            Outcome::Ok,
            Some(topic),
            blocks,
        )
    }

    // ---- subscription -----------------------------------------------------

    fn on_sub(&mut self, session: SessionId, s: TopicSub) -> Handled {
        if let Some(e) = self.require_opened(session, s.topic) {
            return (e, Outcome::Rejected("NoRouting"), Some(s.topic), 0);
        }
        let Some(t) = self.topics.get(&s.topic) else {
            return (
                err(ErrorCode::NotFound, "unknown topic"),
                Outcome::Rejected("NotFound"),
                Some(s.topic),
                0,
            );
        };
        if t.epoch != s.epoch {
            return (
                err(ErrorCode::EpochMismatch, "topic is at a different epoch"),
                Outcome::Rejected("EpochMismatch"),
                Some(s.topic),
                0,
            );
        }
        let replay: Vec<Event> = match s.after_cursor {
            Some(c) => t
                .events
                .iter()
                .filter(|e| e.cursor > c)
                .take(self.cfg.limits.max_ids_per_request as usize)
                .cloned()
                .collect(),
            None => Vec::new(),
        };
        let max_subs = self.cfg.limits.max_subscriptions as usize;
        let Some(sess) = self.sessions.get_mut(&session) else {
            return (
                err(ErrorCode::NoRouting, "unknown session"),
                Outcome::Rejected("NoRouting"),
                Some(s.topic),
                0,
            );
        };
        if !sess.subs.contains(&s.topic) && sess.subs.len() >= max_subs {
            return (
                err(ErrorCode::LimitExceeded, "too many subscriptions"),
                Outcome::Rejected("LimitExceeded"),
                Some(s.topic),
                0,
            );
        }
        sess.subs.insert(s.topic);
        let n = replay.len() as u64;
        for e in replay {
            sess.outbox.push_back(Response::Event(e));
        }
        (Response::Ok, Outcome::Ok, Some(s.topic), n)
    }

    fn on_unsub(&mut self, session: SessionId, topic: TopicId) -> Handled {
        if let Some(s) = self.sessions.get_mut(&session) {
            s.subs.remove(&topic);
        }
        (Response::Ok, Outcome::Ok, Some(topic), 0)
    }

    // ---- have/want sync ---------------------------------------------------

    fn on_sync(&mut self, session: SessionId, q: TopicSyncReq) -> Handled {
        if let Some(e) = self.require_opened(session, q.topic) {
            return (e, Outcome::Rejected("NoRouting"), Some(q.topic), 0);
        }
        let Some(t) = self.topics.get(&q.topic) else {
            return (
                err(ErrorCode::NotFound, "unknown topic"),
                Outcome::Rejected("NotFound"),
                Some(q.topic),
                0,
            );
        };
        if t.epoch != q.epoch {
            return (
                err(ErrorCode::EpochMismatch, "topic is at a different epoch"),
                Outcome::Rejected("EpochMismatch"),
                Some(q.topic),
                0,
            );
        }
        let known: BTreeSet<CommitId> = q.known_heads.iter().copied().collect();
        let cap = self.cfg.limits.max_ids_per_request as usize;
        // In the opaque default the broker has no parent information, so
        // `non_heads` is empty and these are announcement-order ids, NOT a
        // verified causal frontier (§8.4: a broker MUST NOT claim completeness).
        let advertised_heads: Vec<CommitId> = t
            .events
            .iter()
            .map(|e| e.announcement.commit_id)
            .filter(|c| !known.contains(c) && !t.non_heads.contains(c))
            .take(cap)
            .collect();

        let mut missing = Vec::new();
        let mut more = false;
        for id in self.store.ids_in(&q.topic) {
            if let Some(after) = q.page_after {
                if id <= after {
                    continue;
                }
            }
            if let Some(b) = &q.known_commits {
                if b.probably_contains(&id) {
                    continue;
                }
            }
            if missing.len() == cap {
                more = true;
                break;
            }
            missing.push(id);
        }
        let cursor = t.cursor;
        let n = missing.len() as u64;
        (
            Response::SyncResp(TopicSyncResp {
                advertised_heads,
                missing_block_ids: missing,
                cursor,
                more,
            }),
            Outcome::Ok,
            Some(q.topic),
            n,
        )
    }

    // ---- block operations -------------------------------------------------

    fn on_exist(&mut self, session: SessionId, ids: Vec<BlockId>) -> Handled {
        let (topic, e) = self.routing_topic(session);
        if let Some(e) = e {
            let label = code_label(&e);
            return (e, Outcome::Rejected(label), topic, 0);
        }
        let topic = topic.expect("routing_topic returns a topic when it returns no error");
        if let Some(e) = self.check_id_count(ids.len()) {
            return (e, Outcome::Rejected("LimitExceeded"), Some(topic), 0);
        }
        let mut bits = vec![0u8; ids.len().div_ceil(8)];
        for (i, id) in ids.iter().enumerate() {
            if self.store.contains_in(id, &topic) {
                bits[i / 8] |= 1u8 << (i % 8);
            }
        }
        let count = ids.len() as u64;
        (
            Response::ExistBits { bits, count },
            Outcome::Ok,
            Some(topic),
            count,
        )
    }

    fn on_get(&mut self, session: SessionId, now: u64, ids: Vec<BlockId>) -> Handled {
        let (topic, e) = self.routing_topic(session);
        if let Some(e) = e {
            let label = code_label(&e);
            return (e, Outcome::Rejected(label), topic, 0);
        }
        let topic = topic.expect("routing_topic returns a topic when it returns no error");
        if let Some(e) = self.check_id_count(ids.len()) {
            return (e, Outcome::Rejected("LimitExceeded"), Some(topic), 0);
        }
        let limits = self.frame_limits();
        let mut found = Vec::new();
        let mut missing = Vec::new();
        for id in &ids {
            match self.store.get_in(id, &topic, now) {
                // The stored bytes were canonical-validated on the way in, so this
                // decode cannot fail; if it somehow does, treat the block as
                // missing rather than serving something unparseable.
                Some(bytes) => match BlockEnvelope::decode(bytes, limits) {
                    Ok(env) => found.push(env),
                    Err(_) => missing.push(*id),
                },
                None => missing.push(*id),
            }
        }
        let n = found.len() as u64;
        (
            Response::Blocks { found, missing },
            Outcome::Ok,
            Some(topic),
            n,
        )
    }

    fn on_put(&mut self, session: SessionId, now: u64, envelopes: Vec<BlockEnvelope>) -> Handled {
        let (topic, e) = self.routing_topic(session);
        if let Some(e) = e {
            let label = code_label(&e);
            return (e, Outcome::Rejected(label), topic, 0);
        }
        let topic = topic.expect("routing_topic returns a topic when it returns no error");
        if envelopes.len() as u64 > self.cfg.limits.max_blocks_per_put {
            return (
                err(ErrorCode::LimitExceeded, "too many blocks in one put"),
                Outcome::Rejected("LimitExceeded"),
                Some(topic),
                0,
            );
        }
        let mut stored = 0;
        let mut duplicate = 0;
        for env in envelopes {
            let bytes = env.encode();
            if bytes.len() as u64 > self.cfg.limits.max_block_bytes {
                return (
                    err(ErrorCode::LimitExceeded, "block exceeds max_block_bytes"),
                    Outcome::Rejected("LimitExceeded"),
                    Some(topic),
                    0,
                );
            }
            // Charge the budget only for a NEW attribution to this topic. A block
            // already attributed here costs zero further bytes, so re-putting it
            // must stay idempotent right up to the boundary rather than turning
            // into `LimitExceeded`. (The store keeps the first-seen bytes either
            // way, so the repeat never changes what is stored.)
            if !self.store.contains_in(&env.block_id, &topic) {
                let (_, used) = self.store.usage(&topic);
                if used.saturating_add(bytes.len() as u64) > self.cfg.retention.max_topic_bytes {
                    return (
                        err(ErrorCode::LimitExceeded, "topic storage budget exhausted"),
                        Outcome::Rejected("LimitExceeded"),
                        Some(topic),
                        0,
                    );
                }
            }
            if self.store.put(env.block_id, bytes, topic, now) {
                stored += 1;
            } else {
                duplicate += 1;
            }
        }
        (
            Response::Stored { stored, duplicate },
            Outcome::Ok,
            Some(topic),
            stored + duplicate,
        )
    }

    fn on_commit_get(&mut self, session: SessionId, now: u64, ids: Vec<CommitId>) -> Handled {
        let (topic, e) = self.routing_topic(session);
        if let Some(e) = e {
            let label = code_label(&e);
            return (e, Outcome::Rejected(label), topic, 0);
        }
        let topic = topic.expect("routing_topic returns a topic when it returns no error");
        if let Some(e) = self.check_id_count(ids.len()) {
            return (e, Outcome::Rejected("LimitExceeded"), Some(topic), 0);
        }
        let limits = self.frame_limits();
        let roots: Vec<(CommitId, Option<BlockId>)> = {
            let t = self.topics.get(&topic);
            ids.iter()
                .map(|c| (*c, t.and_then(|t| t.commit_root.get(c).copied())))
                .collect()
        };
        let mut found = Vec::new();
        let mut missing = Vec::new();
        for (commit, root) in roots {
            let env = root
                .and_then(|r| self.store.get_in(&r, &topic, now))
                .and_then(|bytes| BlockEnvelope::decode(bytes, limits).ok());
            match env {
                Some(e) => found.push(e),
                None => missing.push(commit),
            }
        }
        let n = found.len() as u64;
        (
            Response::Commits { found, missing },
            Outcome::Ok,
            Some(topic),
            n,
        )
    }

    // ---- publication ------------------------------------------------------

    fn on_publish(&mut self, session: SessionId, now: u64, p: PublishEvent) -> Handled {
        let (topic, e) = self.routing_topic(session);
        if let Some(e) = e {
            let label = code_label(&e);
            return (e, Outcome::Rejected(label), topic, 0);
        }
        let topic = topic.expect("routing_topic returns a topic when it returns no error");
        if p.topic != topic {
            return (
                err(
                    ErrorCode::NoRouting,
                    "publish topic differs from the session routing context",
                ),
                Outcome::Rejected("NoRouting"),
                Some(topic),
                0,
            );
        }
        let mode = self
            .sessions
            .get(&session)
            .and_then(|s| s.negotiated)
            .map(|n| n.header_mode)
            .unwrap_or(HeaderMode::Opaque);
        // The v0 default hides parents inside the ciphertext (§5). A client that
        // sends them anyway would silently widen the disclosure ledger, so it is
        // rejected rather than quietly accepted.
        if mode == HeaderMode::Opaque && !p.parents.is_empty() {
            return (
                err(
                    ErrorCode::Protocol,
                    "parent commit ids are not permitted in opaque-header mode",
                ),
                Outcome::Rejected("Protocol"),
                Some(topic),
                0,
            );
        }
        {
            let Some(t) = self.topics.get(&topic) else {
                return (
                    err(ErrorCode::NotFound, "unknown topic"),
                    Outcome::Rejected("NotFound"),
                    Some(topic),
                    0,
                );
            };
            if t.retired_to.is_some() {
                return (
                    err(ErrorCode::Retired, "topic retired by an epoch advance"),
                    Outcome::Rejected("Retired"),
                    Some(topic),
                    0,
                );
            }
            if t.epoch != p.epoch {
                return (
                    err(ErrorCode::EpochMismatch, "topic is at a different epoch"),
                    Outcome::Rejected("EpochMismatch"),
                    Some(topic),
                    0,
                );
            }
            // Admission is validity-bounded, not a one-way latch: a key admitted
            // by a grant that has since expired is no longer admitted, however
            // well the announcement is signed.
            if !t.is_admitted_at(&p.publisher_key_id, now) {
                return (
                    err(
                        ErrorCode::NotAdmitted,
                        "no unexpired admission grant covers this publisher for the topic/epoch",
                    ),
                    Outcome::Rejected("NotAdmitted"),
                    Some(topic),
                    0,
                );
            }
        }
        if p.verify().is_err() {
            return (
                err(
                    ErrorCode::BadSignature,
                    "publish announcement signature did not verify",
                ),
                Outcome::Rejected("BadSignature"),
                Some(topic),
                0,
            );
        }
        if !self.store.contains_in(&p.root_block_id, &topic) {
            return (
                err(
                    ErrorCode::NotFound,
                    "announced root block has not been uploaded",
                ),
                Outcome::Rejected("NotFound"),
                Some(topic),
                0,
            );
        }
        self.store.mark_commit_root(&p.root_block_id);
        let commit_id = p.commit_id;
        let event = {
            let t = self.topics.get_mut(&topic).expect("topic checked above");
            t.cursor += 1;
            let event = Event {
                announcement: p.clone(),
                cursor: t.cursor,
            };
            t.commit_root.insert(commit_id, p.root_block_id);
            for parent in &p.parents {
                t.non_heads.insert(*parent);
            }
            t.events.push(event.clone());
            event
        };
        let cursor = event.cursor;
        let fanned = self.fan_out(topic, Response::Event(event));
        (
            Response::Published { commit_id, cursor },
            Outcome::Ok,
            Some(topic),
            fanned,
        )
    }

    // ---- epoch advance ----------------------------------------------------

    fn on_epoch_advance(
        &mut self,
        session: SessionId,
        a: sparq_e2ee_ng::broker_protocol::EpochAdvance,
    ) -> Handled {
        if let Some(e) = self.require_opened(session, a.old_topic) {
            return (e, Outcome::Rejected("NoRouting"), Some(a.old_topic), 0);
        }
        let Some(old) = self.topics.get(&a.old_topic) else {
            return (
                err(ErrorCode::NotFound, "unknown topic"),
                Outcome::Rejected("NotFound"),
                Some(a.old_topic),
                0,
            );
        };
        if old.retired_to.is_some() {
            return (
                err(ErrorCode::Retired, "topic already retired"),
                Outcome::Rejected("Retired"),
                Some(a.old_topic),
                0,
            );
        }
        if old.epoch != a.old_epoch {
            return (
                err(ErrorCode::EpochMismatch, "topic is at a different epoch"),
                Outcome::Rejected("EpochMismatch"),
                Some(a.old_topic),
                0,
            );
        }
        let pinned = match PublicVerifyingKey::from_bytes(&old.admin_pub) {
            Ok(k) => k,
            Err(_) => {
                return (
                    err(ErrorCode::BadSignature, "pinned admin key is unusable"),
                    Outcome::Rejected("BadSignature"),
                    Some(a.old_topic),
                    0,
                )
            }
        };
        if a.verify(&pinned).is_err() {
            return (
                err(
                    ErrorCode::BadSignature,
                    "epoch advance is not signed by the topic's pinned admin key",
                ),
                Outcome::Rejected("BadSignature"),
                Some(a.old_topic),
                0,
            );
        }
        if self.topics.contains_key(&a.new_topic) {
            return (
                err(ErrorCode::Protocol, "new topic already exists"),
                Outcome::Rejected("Protocol"),
                Some(a.old_topic),
                0,
            );
        }
        let admin_pub = a.admin_pub;
        let mut fresh = TopicState::new(a.new_epoch, admin_pub);
        // An epoch advance carries no validity window of its own, so the keys it
        // names are admitted for the life of the new epoch: the next advance
        // retires the topic and clears them.
        fresh.publishers = a
            .new_publishers
            .iter()
            .map(|p| (*p, BTreeSet::from([(0u64, u64::MAX)])))
            .collect();
        self.topics.insert(a.new_topic, fresh);
        if let Some(old) = self.topics.get_mut(&a.old_topic) {
            old.retired_to = Some(a.new_topic);
            old.publishers.clear();
        }
        let fanned = self.fan_out(a.old_topic, Response::EpochAdvanced(a.clone()));
        (Response::Ok, Outcome::Ok, Some(a.old_topic), fanned)
    }

    // ---- retention --------------------------------------------------------

    /// Run one retention pass: age-collect unpinned blocks, then evict any
    /// unpinned topic that is over the advertised byte budget. Returns what was
    /// removed.
    ///
    /// This is a *policy*, not a guarantee of erasure: §8.4 permits collecting
    /// unpinned/unreachable blocks, and §4.2 is explicit that nothing a peer has
    /// already fetched can be retracted.
    pub fn collect_garbage(&mut self, now: u64) -> GcReport {
        let pinned: BTreeSet<TopicId> = self
            .topics
            .iter()
            .filter(|(_, t)| t.pinned)
            .map(|(id, _)| *id)
            .collect();
        let is_pinned = move |t: &TopicId| pinned.contains(t);
        let mut report = self
            .store
            .collect_aged(now, self.cfg.retention.unpinned_ttl_secs, &is_pinned);
        let budget = self.cfg.retention.max_topic_bytes;
        let unpinned: Vec<TopicId> = self
            .topics
            .iter()
            .filter(|(_, t)| !t.pinned)
            .map(|(id, _)| *id)
            .collect();
        for topic in unpinned {
            let r = self.store.evict_over_budget(&topic, budget);
            report.removed_blocks += r.removed_blocks;
            report.removed_bytes += r.removed_bytes;
        }
        report
    }

    /// Release all storage attributed to a retired topic.
    pub fn release_retired(&mut self, topic: TopicId) -> GcReport {
        if self
            .topics
            .get(&topic)
            .is_some_and(|t| t.retired_to.is_some())
        {
            self.store.drop_topic(&topic)
        } else {
            GcReport::default()
        }
    }

    /// Number of stored opaque blocks (a §5-visible storage fact).
    pub fn stored_blocks(&self) -> usize {
        self.store.len()
    }

    // ---- helpers ----------------------------------------------------------

    fn frame_limits(&self) -> sparq_e2ee_ng::cbor::Limits {
        protocol_limits(
            self.cfg.limits.max_block_bytes as usize,
            self.cfg.limits.max_ids_per_request as usize,
        )
    }

    fn check_id_count(&self, n: usize) -> Option<Response> {
        if n as u64 > self.cfg.limits.max_ids_per_request {
            Some(err(
                ErrorCode::LimitExceeded,
                "too many identifiers in one request",
            ))
        } else {
            None
        }
    }

    /// The session's open routing topic, re-validated against live topic state.
    ///
    /// A routing context is not a lease: if an epoch advance retired the topic (or
    /// moved it to another epoch) after `OpenRepo`, the stale context fails closed
    /// here instead of letting a block or publish land under the old epoch.
    fn routing_topic(&self, session: SessionId) -> (Option<TopicId>, Option<Response>) {
        let Some(r) = self.sessions.get(&session).and_then(|s| s.routing) else {
            return (
                None,
                Some(err(
                    ErrorCode::NoRouting,
                    "OpenRepo must establish a routing context first",
                )),
            );
        };
        match self.topics.get(&r.topic) {
            None => (
                Some(r.topic),
                Some(err(ErrorCode::NotFound, "unknown topic")),
            ),
            Some(t) if t.retired_to.is_some() => (
                Some(r.topic),
                Some(err(ErrorCode::Retired, "topic retired by an epoch advance")),
            ),
            Some(t) if t.epoch != r.epoch => (
                Some(r.topic),
                Some(err(
                    ErrorCode::EpochMismatch,
                    "routing context is stale; re-open at the current epoch",
                )),
            ),
            Some(_) => (Some(r.topic), None),
        }
    }

    fn require_opened(&self, session: SessionId, topic: TopicId) -> Option<Response> {
        let opened = self
            .sessions
            .get(&session)
            .is_some_and(|s| s.opened.contains(&topic));
        if opened {
            None
        } else {
            Some(err(
                ErrorCode::NoRouting,
                "topic has not been opened by this session",
            ))
        }
    }

    /// Queue `msg` for every session subscribed to `topic`; returns the fan-out
    /// count.
    fn fan_out(&mut self, topic: TopicId, msg: Response) -> u64 {
        let mut n = 0;
        for s in self.sessions.values_mut() {
            if s.subs.contains(&topic) {
                s.outbox.push_back(msg.clone());
                n += 1;
            }
        }
        n
    }
}

impl TopicState {
    fn new(epoch: Epoch, admin_pub: [u8; PUBLIC_KEY_LEN]) -> Self {
        TopicState {
            epoch,
            admin_pub,
            publishers: BTreeMap::new(),
            pinned: false,
            retired_to: None,
            cursor: 0,
            events: Vec::new(),
            commit_root: BTreeMap::new(),
            non_heads: BTreeSet::new(),
        }
    }

    /// Admit `publisher` for the window `(not_before, not_after)`, forgetting any
    /// of its windows that had already expired at `now`.
    fn admit(&mut self, publisher: [u8; PUBLIC_KEY_LEN], window: (u64, u64), now: u64) {
        let windows = self.publishers.entry(publisher).or_default();
        windows.retain(|(_, not_after)| *not_after >= now);
        windows.insert(window);
    }

    /// Does some admission window for `publisher` cover `now`?
    fn is_admitted_at(&self, publisher: &[u8; PUBLIC_KEY_LEN], now: u64) -> bool {
        self.publishers.get(publisher).is_some_and(|windows| {
            windows
                .iter()
                .any(|(not_before, not_after)| now >= *not_before && now <= *not_after)
        })
    }
}

fn err(code: ErrorCode, detail: &'static str) -> Response {
    Response::Error(BrokerError::new(code, detail))
}

fn code_label(r: &Response) -> &'static str {
    match r {
        Response::Error(e) => match e.code {
            ErrorCode::Protocol => "Protocol",
            ErrorCode::Unsupported => "Unsupported",
            ErrorCode::NotNegotiated => "NotNegotiated",
            ErrorCode::NoRouting => "NoRouting",
            ErrorCode::LimitExceeded => "LimitExceeded",
            ErrorCode::RateLimited => "RateLimited",
            ErrorCode::NotFound => "NotFound",
            ErrorCode::NotAdmitted => "NotAdmitted",
            ErrorCode::BadSignature => "BadSignature",
            ErrorCode::EpochMismatch => "EpochMismatch",
            ErrorCode::Retired => "Retired",
            _ => "Error",
        },
        _ => "Error",
    }
}

fn request_kind_label(r: &Request) -> &'static str {
    match r {
        Request::Hello(_) => "Hello",
        Request::OpenRepo(_) => "OpenRepo",
        Request::PinRepo(_) => "PinRepo",
        Request::RepoPinStatus { .. } => "RepoPinStatus",
        Request::TopicSub(_) => "TopicSub",
        Request::TopicUnsub { .. } => "TopicUnsub",
        Request::TopicSyncReq(_) => "TopicSyncReq",
        Request::BlocksExist { .. } => "BlocksExist",
        Request::BlocksGet { .. } => "BlocksGet",
        Request::BlocksPut { .. } => "BlocksPut",
        Request::CommitGet { .. } => "CommitGet",
        Request::PublishEvent(_) => "PublishEvent",
        Request::EpochAdvance(_) => "EpochAdvance",
        _ => "Unknown",
    }
}
