//! **Versioned client/broker messages** (§8.4) — the wire protocol spoken between
//! an E2EE-NG client and an **opaque** broker.
//!
//! Every request carries a `request_id`; every response echoes it and returns
//! either a typed success body or a typed [`BrokerError`]. Both directions use the
//! crate's fail-closed deterministic-CBOR codec ([`crate::cbor`]) with explicit
//! [`Limits`], so a peer cannot smuggle non-canonical bytes past a decoder and a
//! declared length is checked *before* any proportional allocation.
//!
//! ## Disclosure ledger (§5) — what this protocol carries, by construction
//!
//! The messages here are the *routing* surface, and the type system is the first
//! line of the ledger: **no message in this module has a field for a read secret,
//! a private key, RDF terms, or SPARQL text** — a conforming client physically
//! cannot put one on the wire. (This constrains a *conforming* client, not a
//! malicious one: a peer determined to leak its own plaintext can always encode
//! it into a field it does control, and §5 already grants the broker message
//! sizes and timing. The property is that the protocol never *asks* for secrets
//! and never *requires* a client to reveal content.) Concretely:
//!
//! * broker-visible (present here): overlay/topic/peer identifiers, epoch,
//!   cursors, opaque block/commit identifiers, ciphertext envelope bytes, pin and
//!   retention state, publisher/admin **public** verification keys, signatures
//!   needed for admission, message sizes/timing;
//! * hidden by construction (absent here): `K_read`, publisher/admin private keys,
//!   capability secret fields, `RepoId` and stable `BranchId`, plaintext commits,
//!   CRDT operations, SPARQL query text/algebra, and — in the default
//!   [`HeaderMode::Opaque`] — the parent commit ids that would reveal the DAG.
//!
//! That is a *structural* statement about these types, **not** a privacy proof:
//! §5 is explicit that traffic correlation, membership, volume, and timing remain
//! observable, and nothing here is claimed sound or private (see the crate-level
//! honesty boundary and `sq-qhy4`).
//!
//! ## Deviations from, and additions to, the §8.4 sketch
//!
//! §8.4 is a *draft* message list, not a complete wire format. Implementing it
//! required these explicit, documented choices:
//!
//! 1. **[`AdmissionGrant`] is a distinct signed object**, not a projection of
//!    [`crate::capability::PublicGrant`]. A `PublicGrant` binds `RepoId` and
//!    `BranchId` into the admin-signed bytes, and §5 forbids a conforming broker
//!    from learning either — so the broker-facing grant must be separately signed
//!    over the broker-visible subset (topic, epoch, suite, admin/publisher public
//!    keys, validity). §8.2's "a broker receives `topic_id`, public
//!    publisher/admin verification keys, grant bounds necessary for admission, and
//!    signatures" is exactly this object.
//! 2. **[`EpochAdvance`] carries `new_publishers` + `admin_pub`.** §8.4 lists only
//!    `{old_topic, new_topic, transition_commit_id, admin_signature}`, but a
//!    broker cannot "replace routing/**admission** state" without the new
//!    publisher key set, and it cannot verify a [`crate::epoch::EpochTransition`]
//!    (which binds repo/branch) without learning fields §5 hides. So the wire
//!    message is a *separately signed routing statement* that names the
//!    transition commit it corresponds to; a client that also holds the
//!    capability verifies the real `EpochTransition` locally.
//! 3. **[`TopicSyncReq::page_after`]** makes `TopicSyncResp{..., more}` paging
//!    terminate deterministically (block ids are paged in ascending order)
//!    without depending on the client updating a non-authoritative Bloom hint
//!    between pages.
//! 4. **[`HeaderMode`]** is negotiated in `Hello`/`HelloAck` so the §5 choice
//!    between `opaque-header` (v0 default; parents inside ciphertext) and
//!    `clear routing headers` (parents visible, commit DAG revealed to the
//!    broker) is an explicit, logged protocol fact rather than an implicit one.
//! 5. **Block operations are scoped to the session's routing context**
//!    established by [`OpenRepo`]. §8.4's `BlocksPut{envelopes[]}` names no topic,
//!    but per-topic pinning/retention is meaningless unless stored blocks are
//!    attributed to a topic, so the broker attributes them to the open context.
//!
//! ## Framing
//!
//! Every message is one canonical CBOR map:
//!
//! ```cbor-diag
//! { 1: 0, 2: <request_id>, 3: <kind>, 4: <body map> }
//! ```
//!
//! Request kinds are `1..=13`, response kinds are `128..`. A [`Response`] whose
//! `request_id` is `0` is an **unsolicited push** (an [`Event`] fan-out or an
//! [`EpochAdvance`] notification), never a reply.

use crate::capability::Validity;
use crate::cbor::{
    enc_array, enc_bytes, enc_map, enc_text, enc_uint, read_struct_map, Limits, Reader,
};
use crate::envelope::BlockEnvelope;
use crate::error::{Error, Result};
use crate::ids::{BlockId, CommitId, Epoch, OverlayId, PeerId, TopicId};
use crate::sign::{
    PublicVerifyingKey, SecretSigningKey, PUBLIC_KEY_LEN, SIGNATURE_LEN,
};
use crate::suite::{check_suite, SUITE_V0};

/// The protocol version this module implements. A peer MUST reject a frame whose
/// version field is not a version it negotiated.
pub const PROTOCOL_V0: u64 = 0;

// ---- frame keys -----------------------------------------------------------
const F_VERSION: u64 = 1;
const F_REQUEST_ID: u64 = 2;
const F_KIND: u64 = 3;
const F_BODY: u64 = 4;

// ---- request kinds --------------------------------------------------------
const K_HELLO: u64 = 1;
const K_OPEN_REPO: u64 = 2;
const K_PIN_REPO: u64 = 3;
const K_PIN_STATUS_REQ: u64 = 4;
const K_TOPIC_SUB: u64 = 5;
const K_TOPIC_UNSUB: u64 = 6;
const K_TOPIC_SYNC_REQ: u64 = 7;
const K_BLOCKS_EXIST: u64 = 8;
const K_BLOCKS_GET: u64 = 9;
const K_BLOCKS_PUT: u64 = 10;
const K_COMMIT_GET: u64 = 11;
const K_PUBLISH_EVENT: u64 = 12;
const K_EPOCH_ADVANCE: u64 = 13;

// ---- response kinds -------------------------------------------------------
const K_HELLO_ACK: u64 = 128;
const K_OK: u64 = 129;
const K_PIN_STATUS: u64 = 130;
const K_SYNC_RESP: u64 = 131;
const K_EXIST_BITS: u64 = 132;
const K_BLOCKS: u64 = 133;
const K_COMMITS: u64 = 134;
const K_STORED: u64 = 135;
const K_PUBLISHED: u64 = 136;
const K_EVENT: u64 = 137;
const K_EPOCH_ADVANCED: u64 = 138;
const K_ERROR: u64 = 255;

/// Parser limits for one protocol frame. `max_block_bytes` bounds the largest
/// AEAD ciphertext field a peer will accept (§8.1 "deployment limits", advertised
/// by `Hello`), and `max_ids` bounds every identifier list.
pub fn protocol_limits(max_block_bytes: usize, max_ids: usize) -> Limits {
    Limits {
        // + headroom: a block envelope is nested as a byte string, so the outer
        // string limit must admit the ciphertext plus the envelope's own header.
        max_str_len: max_block_bytes.saturating_add(4096),
        max_array_len: max_ids,
        max_map_len: 32,
        max_depth: 8,
    }
}

// ===========================================================================
// Small codec helpers
// ===========================================================================

macro_rules! id_vec_codec {
    ($enc:ident, $read:ident, $ty:ty) => {
        fn $enc(v: &[$ty]) -> Vec<u8> {
            enc_array(&v.iter().map(|x| enc_bytes(x.as_bytes())).collect::<Vec<_>>())
        }
        fn $read(r: &mut Reader<'_>) -> Result<Vec<$ty>> {
            let n = r.array_header()?;
            let mut out = Vec::with_capacity(n);
            for _ in 0..n {
                out.push(<$ty>::from_bytes(r.bytes_fixed::<32>()?));
            }
            Ok(out)
        }
    };
}

id_vec_codec!(enc_commit_ids, read_commit_ids, CommitId);
id_vec_codec!(enc_block_ids, read_block_ids, BlockId);

fn enc_keys(v: &[[u8; PUBLIC_KEY_LEN]]) -> Vec<u8> {
    enc_array(&v.iter().map(|k| enc_bytes(k)).collect::<Vec<_>>())
}

fn read_keys(r: &mut Reader<'_>) -> Result<Vec<[u8; PUBLIC_KEY_LEN]>> {
    let n = r.array_header()?;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(r.bytes_fixed::<PUBLIC_KEY_LEN>()?);
    }
    Ok(out)
}

fn enc_uints(v: &[u64]) -> Vec<u8> {
    enc_array(&v.iter().map(|x| enc_uint(*x)).collect::<Vec<_>>())
}

fn read_uints(r: &mut Reader<'_>) -> Result<Vec<u64>> {
    let n = r.array_header()?;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(r.uint()?);
    }
    Ok(out)
}

fn enc_texts(v: &[String]) -> Vec<u8> {
    enc_array(&v.iter().map(|s| enc_text(s)).collect::<Vec<_>>())
}

fn read_texts(r: &mut Reader<'_>) -> Result<Vec<String>> {
    let n = r.array_header()?;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(r.text()?.to_owned());
    }
    Ok(out)
}

fn enc_bool(b: bool) -> Vec<u8> {
    enc_uint(u64::from(b))
}

fn read_bool(r: &mut Reader<'_>) -> Result<bool> {
    match r.uint()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Error::Schema("boolean must encode as 0 or 1")),
    }
}

fn need<T>(v: Option<T>, what: &'static str) -> Result<T> {
    v.ok_or(Error::Schema(what))
}

// ===========================================================================
// Negotiation (§8.4 Hello / HelloAck)
// ===========================================================================

/// Which routing header mode is in force (§5). This is a **metadata trade**, not
/// a confidentiality setting: both modes encrypt the same payload, but
/// [`HeaderMode::Clear`] additionally reveals the commit DAG shape to the broker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderMode {
    /// v0 default: parent commit ids live **inside** the ciphertext; only event
    /// order and opaque ids are visible to the broker.
    Opaque,
    /// Clear routing headers: parent commit ids travel in the clear, so the
    /// broker can reconstruct the branch history shape (§5).
    Clear,
}

impl HeaderMode {
    /// The wire token.
    pub fn as_str(self) -> &'static str {
        match self {
            HeaderMode::Opaque => "opaque-header",
            HeaderMode::Clear => "clear-header",
        }
    }
    /// Parse a wire token (fail-closed).
    pub fn from_token(s: &str) -> Result<Self> {
        match s {
            "opaque-header" => Ok(HeaderMode::Opaque),
            "clear-header" => Ok(HeaderMode::Clear),
            _ => Err(Error::Schema("unknown header mode")),
        }
    }
}

/// Deployment limits a broker advertises and both peers enforce (§8.1, §8.4).
/// These are never assumed from another implementation's defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireLimits {
    /// Largest accepted encoded frame, in bytes.
    pub max_message_bytes: u64,
    /// Largest accepted block ciphertext field, in bytes.
    pub max_block_bytes: u64,
    /// Largest identifier list in one request.
    pub max_ids_per_request: u64,
    /// Largest `BlocksPut` batch.
    pub max_blocks_per_put: u64,
    /// Largest number of concurrent topic subscriptions per session.
    pub max_subscriptions: u64,
    /// Requests permitted per [`Self::rate_window_secs`] window.
    pub max_requests_per_window: u64,
    /// Rate-limit window length in seconds.
    pub rate_window_secs: u64,
}

impl Default for WireLimits {
    fn default() -> Self {
        WireLimits {
            max_message_bytes: 8 << 20,
            max_block_bytes: 4 << 20,
            max_ids_per_request: 1024,
            max_blocks_per_put: 256,
            max_subscriptions: 64,
            max_requests_per_window: 4096,
            rate_window_secs: 60,
        }
    }
}

const W_MSG: u64 = 1;
const W_BLOCK: u64 = 2;
const W_IDS: u64 = 3;
const W_PUT: u64 = 4;
const W_SUBS: u64 = 5;
const W_RATE: u64 = 6;
const W_WINDOW: u64 = 7;

impl WireLimits {
    fn encode(&self) -> Vec<u8> {
        enc_map(vec![
            (W_MSG, enc_uint(self.max_message_bytes)),
            (W_BLOCK, enc_uint(self.max_block_bytes)),
            (W_IDS, enc_uint(self.max_ids_per_request)),
            (W_PUT, enc_uint(self.max_blocks_per_put)),
            (W_SUBS, enc_uint(self.max_subscriptions)),
            (W_RATE, enc_uint(self.max_requests_per_window)),
            (W_WINDOW, enc_uint(self.rate_window_secs)),
        ])
    }

    fn read(r: &mut Reader<'_>) -> Result<Self> {
        let (mut m, mut b, mut i, mut p, mut s, mut rt, mut w) =
            (None, None, None, None, None, None, None);
        read_struct_map(r, |r, k| match k {
            W_MSG => {
                m = Some(r.uint()?);
                Ok(true)
            }
            W_BLOCK => {
                b = Some(r.uint()?);
                Ok(true)
            }
            W_IDS => {
                i = Some(r.uint()?);
                Ok(true)
            }
            W_PUT => {
                p = Some(r.uint()?);
                Ok(true)
            }
            W_SUBS => {
                s = Some(r.uint()?);
                Ok(true)
            }
            W_RATE => {
                rt = Some(r.uint()?);
                Ok(true)
            }
            W_WINDOW => {
                w = Some(r.uint()?);
                Ok(true)
            }
            _ => Ok(false),
        })?;
        Ok(WireLimits {
            max_message_bytes: need(m, "limits.max_message_bytes")?,
            max_block_bytes: need(b, "limits.max_block_bytes")?,
            max_ids_per_request: need(i, "limits.max_ids_per_request")?,
            max_blocks_per_put: need(p, "limits.max_blocks_per_put")?,
            max_subscriptions: need(s, "limits.max_subscriptions")?,
            max_requests_per_window: need(rt, "limits.max_requests_per_window")?,
            rate_window_secs: need(w, "limits.rate_window_secs")?,
        })
    }
}

/// The retention policy a broker advertises (§8.4: it MAY garbage-collect
/// unpinned/unreachable opaque blocks under an *advertised* policy).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionPolicy {
    /// Seconds an **unpinned** block survives after its last touch before it may
    /// be collected. `u64::MAX` means "never collected by age".
    pub unpinned_ttl_secs: u64,
    /// Byte ceiling stored per topic; oldest-untouched blocks are evicted first.
    /// `u64::MAX` means unbounded.
    pub max_topic_bytes: u64,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        RetentionPolicy {
            unpinned_ttl_secs: 7 * 24 * 3600,
            max_topic_bytes: 1 << 30,
        }
    }
}

const R_TTL: u64 = 1;
const R_MAX: u64 = 2;

impl RetentionPolicy {
    fn encode(&self) -> Vec<u8> {
        enc_map(vec![
            (R_TTL, enc_uint(self.unpinned_ttl_secs)),
            (R_MAX, enc_uint(self.max_topic_bytes)),
        ])
    }
    fn read(r: &mut Reader<'_>) -> Result<Self> {
        let (mut t, mut m) = (None, None);
        read_struct_map(r, |r, k| match k {
            R_TTL => {
                t = Some(r.uint()?);
                Ok(true)
            }
            R_MAX => {
                m = Some(r.uint()?);
                Ok(true)
            }
            _ => Ok(false),
        })?;
        Ok(RetentionPolicy {
            unpinned_ttl_secs: need(t, "retention.unpinned_ttl_secs")?,
            max_topic_bytes: need(m, "retention.max_topic_bytes")?,
        })
    }
}

/// `Hello{versions, suites, max_block_size, padding_classes}` (§8.4) — the
/// client's offer. Sizes/classes are the client's *own* ceilings; the broker's
/// advertised limits in [`HelloAck`] are authoritative for the session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hello {
    /// Protocol versions the client supports, ascending.
    pub versions: Vec<u64>,
    /// Suite identifiers the client supports.
    pub suites: Vec<String>,
    /// Largest block ciphertext the client will send.
    pub max_block_size: u64,
    /// Padding length classes the client uses (§8.3).
    pub padding_classes: Vec<u64>,
    /// Routing header modes the client accepts, most-preferred first.
    pub header_modes: Vec<HeaderMode>,
}

/// `HelloAck{chosen,...}` (§8.4) — the broker's binding choice plus the limits and
/// retention policy the session is held to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloAck {
    /// Chosen protocol version.
    pub version: u64,
    /// Chosen suite identifier (MUST be one both peers offered; never substituted).
    pub suite: String,
    /// Chosen routing header mode.
    pub header_mode: HeaderMode,
    /// Authoritative session limits.
    pub limits: WireLimits,
    /// Padding classes in force.
    pub padding_classes: Vec<u64>,
    /// Advertised retention policy.
    pub retention: RetentionPolicy,
}

// ===========================================================================
// Admission grant (broker-visible authorization object)
// ===========================================================================

/// The **broker-facing** admission grant: the admin-signed statement that a topic
/// exists and which publisher key (if any) may publish to it at an epoch.
///
/// It deliberately carries **no** `RepoId`/`BranchId` (§5 hides both from a
/// conforming broker) and **no** secret field. It is a distinct signed object
/// rather than a projection of [`crate::capability::PublicGrant`], because that
/// grant's signature binds the repo/branch bytes a broker must not learn (see the
/// module-level "Deviations" note).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionGrant {
    /// Epoch-specific routing topic.
    pub topic: TopicId,
    /// Epoch this admission is scoped to.
    pub epoch: Epoch,
    /// Bound suite identifier.
    pub suite: String,
    /// The repository admin's **public** verification key (the trust anchor a
    /// broker pins per topic).
    pub admin_pub: [u8; PUBLIC_KEY_LEN],
    /// The admitted publisher's **public** key, when this grant admits a publisher.
    pub publisher_pub: Option<[u8; PUBLIC_KEY_LEN]>,
    /// Validity window (unix seconds).
    pub validity: Validity,
    /// Admin signature over fields 1..6.
    pub admin_sig: Option<[u8; SIGNATURE_LEN]>,
}

const A_VERSION: u64 = 1;
const A_TOPIC: u64 = 2;
const A_EPOCH: u64 = 3;
const A_SUITE: u64 = 4;
const A_ADMIN_PK: u64 = 5;
const A_VALIDITY: u64 = 6;
const A_PUBLISHER_PK: u64 = 7;
const A_SIG: u64 = 8;
const A_NB: u64 = 1;
const A_NA: u64 = 2;

impl AdmissionGrant {
    fn signing_entries(&self) -> Vec<(u64, Vec<u8>)> {
        let mut e = vec![
            (A_VERSION, enc_uint(PROTOCOL_V0)),
            (A_TOPIC, enc_bytes(self.topic.as_bytes())),
            (A_EPOCH, enc_uint(self.epoch.0)),
            (A_SUITE, enc_text(&self.suite)),
            (A_ADMIN_PK, enc_bytes(&self.admin_pub)),
            (
                A_VALIDITY,
                enc_map(vec![
                    (A_NB, enc_uint(self.validity.not_before)),
                    (A_NA, enc_uint(self.validity.not_after)),
                ]),
            ),
        ];
        if let Some(pk) = &self.publisher_pub {
            e.push((A_PUBLISHER_PK, enc_bytes(pk)));
        }
        e
    }

    /// The exact bytes the admin signs.
    pub fn signing_bytes(&self) -> Vec<u8> {
        enc_map(self.signing_entries())
    }

    /// Sign this grant with the repository admin key, which MUST be the key whose
    /// public half is in [`Self::admin_pub`] (checked here, fail-closed).
    pub fn sign(&mut self, admin: &SecretSigningKey) -> Result<()> {
        if admin.public().to_bytes() != self.admin_pub {
            return Err(Error::Separation(
                "admission grant signed by a key other than its declared admin key",
            ));
        }
        self.admin_sig = Some(admin.sign(&self.signing_bytes()));
        Ok(())
    }

    /// Verify the grant against its own declared admin key. A broker additionally
    /// checks that `admin_pub` equals the key it pinned for this topic — a
    /// self-consistent grant proves authorship, not authority.
    pub fn verify_self(&self) -> Result<()> {
        let sig = need(self.admin_sig, "missing admission-grant signature")?;
        check_suite(&self.suite)?;
        let pk = PublicVerifyingKey::from_bytes(&self.admin_pub)?;
        pk.verify(&self.signing_bytes(), &sig)
    }

    /// Is this grant inside its validity window at `now` (unix seconds)?
    pub fn is_valid_at(&self, now: u64) -> bool {
        now >= self.validity.not_before && now <= self.validity.not_after
    }

    /// Canonical CBOR including the signature when set.
    pub fn encode(&self) -> Vec<u8> {
        let mut e = self.signing_entries();
        if let Some(sig) = &self.admin_sig {
            e.push((A_SIG, enc_bytes(sig)));
        }
        enc_map(e)
    }

    fn read(r: &mut Reader<'_>) -> Result<Self> {
        let (mut v, mut topic, mut epoch, mut suite, mut admin, mut val, mut pubk, mut sig) =
            (None, None, None, None, None, None, None, None);
        read_struct_map(r, |r, k| match k {
            A_VERSION => {
                v = Some(r.uint()?);
                Ok(true)
            }
            A_TOPIC => {
                topic = Some(TopicId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            A_EPOCH => {
                epoch = Some(Epoch(r.uint()?));
                Ok(true)
            }
            A_SUITE => {
                suite = Some(r.text()?.to_owned());
                Ok(true)
            }
            A_ADMIN_PK => {
                admin = Some(r.bytes_fixed::<PUBLIC_KEY_LEN>()?);
                Ok(true)
            }
            A_VALIDITY => {
                let (mut nb, mut na) = (None, None);
                read_struct_map(r, |r, k| match k {
                    A_NB => {
                        nb = Some(r.uint()?);
                        Ok(true)
                    }
                    A_NA => {
                        na = Some(r.uint()?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                val = Some(Validity {
                    not_before: need(nb, "grant.not_before")?,
                    not_after: need(na, "grant.not_after")?,
                });
                Ok(true)
            }
            A_PUBLISHER_PK => {
                pubk = Some(r.bytes_fixed::<PUBLIC_KEY_LEN>()?);
                Ok(true)
            }
            A_SIG => {
                sig = Some(r.bytes_fixed::<SIGNATURE_LEN>()?);
                Ok(true)
            }
            _ => Ok(false),
        })?;
        if v != Some(PROTOCOL_V0) {
            return Err(Error::Schema("admission grant version"));
        }
        let suite = need(suite, "grant.suite")?;
        check_suite(&suite)?;
        let validity: Validity = need(val, "grant.validity")?;
        if validity.not_before > validity.not_after {
            return Err(Error::Schema("grant validity window is inverted"));
        }
        Ok(AdmissionGrant {
            topic: need(topic, "grant.topic")?,
            epoch: need(epoch, "grant.epoch")?,
            suite,
            admin_pub: need(admin, "grant.admin_pub")?,
            publisher_pub: pubk,
            validity,
            admin_sig: sig,
        })
    }
}

// ===========================================================================
// Request bodies
// ===========================================================================

/// `OpenRepo{overlay_id, topic_id, epoch, peer_id, auth}` (§8.4) — establishes the
/// session's **routing context**. Despite its name it MUST NOT send stable
/// repo/branch identifiers, and this type has no field that could.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenRepo {
    /// Overlay this topic lives in.
    pub overlay: OverlayId,
    /// Epoch-specific routing topic.
    pub topic: TopicId,
    /// Epoch.
    pub epoch: Epoch,
    /// Pseudonymous peer identifier.
    pub peer: PeerId,
    /// Optional admission grant. A read-only subscriber may open without one;
    /// publication always requires an admitted publisher key.
    pub auth: Option<AdmissionGrant>,
}

/// `PinRepo{topic_id, retention}` (§8.4) — opaque retention control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinRepo {
    /// Topic to pin/unpin.
    pub topic: TopicId,
    /// `true` pins (exempts from age/size collection), `false` unpins.
    pub pin: bool,
}

/// `RepoPinStatus{topic_id}` reply (§8.4). Counts are broker-visible storage
/// facts (§5), never content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinStatus {
    /// Topic queried.
    pub topic: TopicId,
    /// Whether the topic is pinned.
    pub pinned: bool,
    /// Blocks currently stored for the topic.
    pub blocks: u64,
    /// Bytes currently stored for the topic.
    pub bytes: u64,
    /// Retention policy in force.
    pub retention: RetentionPolicy,
}

/// `TopicSub{topic_id, epoch, after_cursor?}` (§8.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopicSub {
    /// Topic to subscribe to.
    pub topic: TopicId,
    /// Epoch the subscription is scoped to.
    pub epoch: Epoch,
    /// Replay events strictly after this cursor when present.
    pub after_cursor: Option<u64>,
}

/// A **non-authoritative** Bloom-filter bandwidth hint (§6 step 2: "Bloom filters
/// are permitted only as a bandwidth hint; false positives are repaired by
/// parent-closure fetching").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BloomHint {
    /// Filter bits, LSB-first within each byte.
    pub bits: Vec<u8>,
    /// Number of hash functions the client used.
    pub hashes: u64,
}

impl BloomHint {
    /// An empty filter of `bytes` bytes using `hashes` probes.
    pub fn new(bytes: usize, hashes: u64) -> Self {
        BloomHint {
            bits: vec![0u8; bytes],
            hashes,
        }
    }

    /// The bit positions probed for `id`. Two 64-bit halves of the (already
    /// CSPRNG-drawn) id combined Kirsch-Mitzenmacher style; a uniformly random id
    /// needs no extra hashing to spread.
    fn positions(&self, id: &BlockId) -> Vec<u64> {
        let nbits = (self.bits.len() * 8) as u64;
        let b = id.as_bytes();
        let h1 = u64::from_le_bytes(b[0..8].try_into().expect("8 bytes"));
        let h2 = u64::from_le_bytes(b[8..16].try_into().expect("8 bytes")) | 1;
        (0..self.hashes)
            .map(|i| h1.wrapping_add(h2.wrapping_mul(i)) % nbits)
            .collect()
    }

    /// Record that the client already holds `id`.
    pub fn insert(&mut self, id: &BlockId) {
        if self.bits.is_empty() || self.hashes == 0 {
            return;
        }
        for pos in self.positions(id) {
            self.bits[(pos / 8) as usize] |= 1u8 << (pos % 8);
        }
    }

    /// Test membership of `id`. A `true` answer may be a false positive — this is
    /// a hint, never an authority — but a `false` answer is definitive, so an
    /// inserted id is never reported absent.
    pub fn probably_contains(&self, id: &BlockId) -> bool {
        if self.bits.is_empty() || self.hashes == 0 {
            return false;
        }
        self.positions(id)
            .into_iter()
            .all(|pos| self.bits[(pos / 8) as usize] & (1u8 << (pos % 8)) != 0)
    }
}

/// `TopicSyncReq{topic_id, epoch, known_heads[], target_heads[]?, known_commits?}`
/// (§8.4), plus the [`Self::page_after`] paging cursor documented at module level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicSyncReq {
    /// Topic to reconcile.
    pub topic: TopicId,
    /// Epoch.
    pub epoch: Epoch,
    /// Commits the client already accepted (empty for an initial clone).
    pub known_heads: Vec<CommitId>,
    /// Optional specific targets the client wants to reach.
    pub target_heads: Option<Vec<CommitId>>,
    /// Optional non-authoritative Bloom hint of blocks the client already holds.
    pub known_commits: Option<BloomHint>,
    /// Resume paging after this block id (ascending block-id order).
    pub page_after: Option<BlockId>,
}

/// `TopicSyncResp{advertised_heads[], missing_block_ids[], cursor, more}` (§8.4).
///
/// `advertised_heads` are **announcement-order** heads, not a verified causal
/// frontier: in [`HeaderMode::Opaque`] the broker cannot see parents at all, and
/// §8.4 is explicit that a broker MUST NOT claim completeness — the client
/// detects missing causal closure itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicSyncResp {
    /// Commit ids the broker advertises for this topic.
    pub advertised_heads: Vec<CommitId>,
    /// Opaque block ids the client appears to be missing.
    pub missing_block_ids: Vec<BlockId>,
    /// Current event cursor for the topic.
    pub cursor: u64,
    /// `true` when more missing ids remain; resend with `page_after` set to the
    /// last returned id.
    pub more: bool,
}

/// `PublishEvent{topic_id, epoch, commit_id, root_block_id, publisher_key_id,
/// signature}` (§8.4) — announces an already-uploaded commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishEvent {
    /// Routing topic.
    pub topic: TopicId,
    /// Epoch.
    pub epoch: Epoch,
    /// `CommitId = SHA-256(canonical encrypted root envelope)`.
    pub commit_id: CommitId,
    /// The root block the commit is carried in.
    pub root_block_id: BlockId,
    /// The publishing key's public bytes (must be admitted for the topic/epoch).
    pub publisher_key_id: [u8; PUBLIC_KEY_LEN],
    /// Parent commit ids — **only** populated in [`HeaderMode::Clear`]. In the v0
    /// default this MUST be empty (parents live inside the ciphertext, §5).
    pub parents: Vec<CommitId>,
    /// Publisher signature over fields 1..7.
    pub signature: Option<[u8; SIGNATURE_LEN]>,
}

const P_VERSION: u64 = 1;
const P_TOPIC: u64 = 2;
const P_EPOCH: u64 = 3;
const P_COMMIT: u64 = 4;
const P_ROOT: u64 = 5;
const P_PUBKEY: u64 = 6;
const P_PARENTS: u64 = 7;
const P_SIG: u64 = 8;

impl PublishEvent {
    fn signing_entries(&self) -> Vec<(u64, Vec<u8>)> {
        vec![
            (P_VERSION, enc_uint(PROTOCOL_V0)),
            (P_TOPIC, enc_bytes(self.topic.as_bytes())),
            (P_EPOCH, enc_uint(self.epoch.0)),
            (P_COMMIT, enc_bytes(self.commit_id.as_bytes())),
            (P_ROOT, enc_bytes(self.root_block_id.as_bytes())),
            (P_PUBKEY, enc_bytes(&self.publisher_key_id)),
            (P_PARENTS, enc_commit_ids(&self.parents)),
        ]
    }

    /// The exact bytes the publisher signs.
    pub fn signing_bytes(&self) -> Vec<u8> {
        enc_map(self.signing_entries())
    }

    /// Sign the announcement; the publisher key's public half becomes
    /// [`Self::publisher_key_id`].
    pub fn sign(&mut self, publisher: &SecretSigningKey) {
        self.publisher_key_id = publisher.public().to_bytes();
        self.signature = Some(publisher.sign(&self.signing_bytes()));
    }

    /// Verify the announcement signature under its own declared publisher key.
    /// The broker separately checks that key is **admitted** for the topic/epoch;
    /// a valid self-signature proves authorship, not authority.
    pub fn verify(&self) -> Result<()> {
        let sig = need(self.signature, "missing publish signature")?;
        let pk = PublicVerifyingKey::from_bytes(&self.publisher_key_id)?;
        pk.verify(&self.signing_bytes(), &sig)
    }

    fn encode_body(&self) -> Vec<u8> {
        let mut e = self.signing_entries();
        if let Some(sig) = &self.signature {
            e.push((P_SIG, enc_bytes(sig)));
        }
        enc_map(e)
    }

    fn read(r: &mut Reader<'_>) -> Result<Self> {
        let (mut v, mut topic, mut epoch, mut commit, mut root, mut pk, mut parents, mut sig) =
            (None, None, None, None, None, None, None, None);
        read_struct_map(r, |r, k| match k {
            P_VERSION => {
                v = Some(r.uint()?);
                Ok(true)
            }
            P_TOPIC => {
                topic = Some(TopicId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            P_EPOCH => {
                epoch = Some(Epoch(r.uint()?));
                Ok(true)
            }
            P_COMMIT => {
                commit = Some(CommitId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            P_ROOT => {
                root = Some(BlockId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            P_PUBKEY => {
                pk = Some(r.bytes_fixed::<PUBLIC_KEY_LEN>()?);
                Ok(true)
            }
            P_PARENTS => {
                parents = Some(read_commit_ids(r)?);
                Ok(true)
            }
            P_SIG => {
                sig = Some(r.bytes_fixed::<SIGNATURE_LEN>()?);
                Ok(true)
            }
            _ => Ok(false),
        })?;
        if v != Some(PROTOCOL_V0) {
            return Err(Error::Schema("publish event version"));
        }
        Ok(PublishEvent {
            topic: need(topic, "publish.topic")?,
            epoch: need(epoch, "publish.epoch")?,
            commit_id: need(commit, "publish.commit_id")?,
            root_block_id: need(root, "publish.root_block_id")?,
            publisher_key_id: need(pk, "publish.publisher_key_id")?,
            parents: need(parents, "publish.parents")?,
            signature: sig,
        })
    }
}

/// `Event{..., cursor}` (§8.4) — the fan-out form of a [`PublishEvent`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// The announcement, verbatim as the publisher signed it.
    pub announcement: PublishEvent,
    /// Monotonic per-topic delivery cursor.
    pub cursor: u64,
}

/// `EpochAdvance{old_topic, new_topic, transition_commit_id, admin_signature}`
/// (§8.4), extended with the admission state a broker actually needs (see the
/// module-level "Deviations" note). It replaces routing/admission state; new
/// capabilities travel entirely outside this API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpochAdvance {
    /// Topic being retired.
    pub old_topic: TopicId,
    /// Topic for the new epoch.
    pub new_topic: TopicId,
    /// Epoch being left.
    pub old_epoch: Epoch,
    /// Epoch being entered (MUST be strictly greater).
    pub new_epoch: Epoch,
    /// The transition commit this routing statement corresponds to. The broker
    /// cannot decrypt or verify it; a capability holder verifies the real
    /// [`crate::epoch::EpochTransition`] locally.
    pub transition_commit: CommitId,
    /// Publisher public keys admitted at the new epoch.
    pub new_publishers: Vec<[u8; PUBLIC_KEY_LEN]>,
    /// Admin public key authorising the advance (must match the broker's pin).
    pub admin_pub: [u8; PUBLIC_KEY_LEN],
    /// Admin signature over fields 1..7.
    pub admin_sig: Option<[u8; SIGNATURE_LEN]>,
}

const E_VERSION: u64 = 1;
const E_OLD_TOPIC: u64 = 2;
const E_NEW_TOPIC: u64 = 3;
const E_OLD_EPOCH: u64 = 4;
const E_NEW_EPOCH: u64 = 5;
const E_TRANSITION: u64 = 6;
const E_PUBLISHERS: u64 = 7;
const E_ADMIN_PK: u64 = 8;
const E_SIG: u64 = 9;

impl EpochAdvance {
    fn signing_entries(&self) -> Vec<(u64, Vec<u8>)> {
        vec![
            (E_VERSION, enc_uint(PROTOCOL_V0)),
            (E_OLD_TOPIC, enc_bytes(self.old_topic.as_bytes())),
            (E_NEW_TOPIC, enc_bytes(self.new_topic.as_bytes())),
            (E_OLD_EPOCH, enc_uint(self.old_epoch.0)),
            (E_NEW_EPOCH, enc_uint(self.new_epoch.0)),
            (E_TRANSITION, enc_bytes(self.transition_commit.as_bytes())),
            (E_PUBLISHERS, enc_keys(&self.new_publishers)),
            (E_ADMIN_PK, enc_bytes(&self.admin_pub)),
        ]
    }

    /// The exact bytes the admin signs.
    pub fn signing_bytes(&self) -> Vec<u8> {
        enc_map(self.signing_entries())
    }

    /// The monotonicity invariant: the new epoch strictly follows the old, and the
    /// new topic is distinct from the retired one (§4.2 mints a *fresh* topic).
    pub fn check_monotonic(&self) -> Result<()> {
        if self.new_epoch.0 <= self.old_epoch.0 {
            return Err(Error::Schema("epoch advance is not strictly increasing"));
        }
        if self.new_topic == self.old_topic {
            return Err(Error::Schema("epoch advance must mint a fresh topic"));
        }
        Ok(())
    }

    /// Sign the advance with the admin key declared in [`Self::admin_pub`].
    pub fn sign(&mut self, admin: &SecretSigningKey) -> Result<()> {
        self.check_monotonic()?;
        if admin.public().to_bytes() != self.admin_pub {
            return Err(Error::Separation(
                "epoch advance signed by a key other than its declared admin key",
            ));
        }
        self.admin_sig = Some(admin.sign(&self.signing_bytes()));
        Ok(())
    }

    /// Verify the admin signature and the monotonicity invariant against a
    /// **trusted** admin key (the broker passes the key it pinned for the topic).
    pub fn verify(&self, trusted_admin: &PublicVerifyingKey) -> Result<()> {
        self.check_monotonic()?;
        if trusted_admin.to_bytes() != self.admin_pub {
            return Err(Error::BadSignature);
        }
        let sig = need(self.admin_sig, "missing epoch-advance signature")?;
        trusted_admin.verify(&self.signing_bytes(), &sig)
    }

    fn encode_body(&self) -> Vec<u8> {
        let mut e = self.signing_entries();
        if let Some(sig) = &self.admin_sig {
            e.push((E_SIG, enc_bytes(sig)));
        }
        enc_map(e)
    }

    fn read(r: &mut Reader<'_>) -> Result<Self> {
        let (mut v, mut ot, mut nt, mut oe, mut ne, mut tc, mut pubs, mut admin, mut sig) =
            (None, None, None, None, None, None, None, None, None);
        read_struct_map(r, |r, k| match k {
            E_VERSION => {
                v = Some(r.uint()?);
                Ok(true)
            }
            E_OLD_TOPIC => {
                ot = Some(TopicId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            E_NEW_TOPIC => {
                nt = Some(TopicId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            E_OLD_EPOCH => {
                oe = Some(Epoch(r.uint()?));
                Ok(true)
            }
            E_NEW_EPOCH => {
                ne = Some(Epoch(r.uint()?));
                Ok(true)
            }
            E_TRANSITION => {
                tc = Some(CommitId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            E_PUBLISHERS => {
                pubs = Some(read_keys(r)?);
                Ok(true)
            }
            E_ADMIN_PK => {
                admin = Some(r.bytes_fixed::<PUBLIC_KEY_LEN>()?);
                Ok(true)
            }
            E_SIG => {
                sig = Some(r.bytes_fixed::<SIGNATURE_LEN>()?);
                Ok(true)
            }
            _ => Ok(false),
        })?;
        if v != Some(PROTOCOL_V0) {
            return Err(Error::Schema("epoch advance version"));
        }
        let a = EpochAdvance {
            old_topic: need(ot, "advance.old_topic")?,
            new_topic: need(nt, "advance.new_topic")?,
            old_epoch: need(oe, "advance.old_epoch")?,
            new_epoch: need(ne, "advance.new_epoch")?,
            transition_commit: need(tc, "advance.transition_commit")?,
            new_publishers: need(pubs, "advance.new_publishers")?,
            admin_pub: need(admin, "advance.admin_pub")?,
            admin_sig: sig,
        };
        a.check_monotonic()?;
        Ok(a)
    }
}

// ===========================================================================
// Typed errors
// ===========================================================================

/// Typed broker error codes. A broker's error *detail* is a short, fixed,
/// metadata-safe string: it never echoes ciphertext, identifiers, or anything
/// derived from a client's secrets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorCode {
    /// The frame was malformed, non-canonical, or over a parser limit.
    Protocol = 1,
    /// No common protocol version / suite / header mode.
    Unsupported = 2,
    /// A request arrived before `Hello`/`HelloAck` negotiation completed.
    NotNegotiated = 3,
    /// A request needs a routing context that `OpenRepo` has not established.
    NoRouting = 4,
    /// A negotiated size/count limit was exceeded.
    LimitExceeded = 5,
    /// The session exceeded its request-rate allowance.
    RateLimited = 6,
    /// The addressed topic/block/commit is unknown to this broker.
    NotFound = 7,
    /// The presented key is not admitted for the topic/epoch.
    NotAdmitted = 8,
    /// A signature did not verify.
    BadSignature = 9,
    /// The declared epoch does not match the topic's current epoch.
    EpochMismatch = 10,
    /// The topic has been retired by an epoch advance.
    Retired = 11,
}

impl ErrorCode {
    fn from_code(v: u64) -> Result<Self> {
        Ok(match v {
            1 => ErrorCode::Protocol,
            2 => ErrorCode::Unsupported,
            3 => ErrorCode::NotNegotiated,
            4 => ErrorCode::NoRouting,
            5 => ErrorCode::LimitExceeded,
            6 => ErrorCode::RateLimited,
            7 => ErrorCode::NotFound,
            8 => ErrorCode::NotAdmitted,
            9 => ErrorCode::BadSignature,
            10 => ErrorCode::EpochMismatch,
            11 => ErrorCode::Retired,
            _ => return Err(Error::Schema("unknown broker error code")),
        })
    }
}

/// A typed error response (§8.4 "returns either `ok` or a typed error").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokerError {
    /// Machine-readable code.
    pub code: ErrorCode,
    /// Short, fixed, metadata-safe explanation. Brokers construct this from a
    /// closed set of `&'static str` literals; it never echoes client input.
    pub detail: String,
}

impl BrokerError {
    /// Construct an error response body from a fixed literal.
    pub fn new(code: ErrorCode, detail: &'static str) -> Self {
        BrokerError { code, detail: detail.to_owned() }
    }
}

// ===========================================================================
// Request / Response enums
// ===========================================================================

/// A client -> broker request (§8.4).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Request {
    /// Version/suite/limits negotiation.
    Hello(Hello),
    /// Establish the session routing context.
    OpenRepo(OpenRepo),
    /// Pin or unpin a topic's opaque storage.
    PinRepo(PinRepo),
    /// Query a topic's pin/retention/storage state.
    RepoPinStatus {
        /// Topic queried.
        topic: TopicId,
    },
    /// Subscribe to a topic's event fan-out.
    TopicSub(TopicSub),
    /// Stop a subscription.
    TopicUnsub {
        /// Topic to unsubscribe from.
        topic: TopicId,
    },
    /// Ask for have/want reconciliation.
    TopicSyncReq(TopicSyncReq),
    /// Ask which of these opaque blocks the broker holds.
    BlocksExist {
        /// Block ids probed.
        ids: Vec<BlockId>,
    },
    /// Fetch opaque block envelopes.
    BlocksGet {
        /// Block ids requested.
        ids: Vec<BlockId>,
    },
    /// Store opaque block envelopes idempotently (exact bytes).
    BlocksPut {
        /// Envelopes to store.
        envelopes: Vec<BlockEnvelope>,
    },
    /// Resolve commit ids to their root block envelopes.
    CommitGet {
        /// Commit ids requested.
        commit_ids: Vec<CommitId>,
    },
    /// Announce an uploaded commit.
    PublishEvent(PublishEvent),
    /// Replace routing/admission state for a new epoch.
    EpochAdvance(EpochAdvance),
}

/// A broker -> client response (§8.4). A response carrying `request_id == 0` is an
/// unsolicited push ([`Response::Event`] / [`Response::EpochAdvanced`]).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Response {
    /// Negotiation result.
    HelloAck(HelloAck),
    /// Generic success with no payload.
    Ok,
    /// Pin/retention/storage state.
    PinStatus(PinStatus),
    /// Have/want reconciliation reply.
    SyncResp(TopicSyncResp),
    /// Packed presence bit vector, LSB-first, in request order.
    ExistBits {
        /// Packed bits.
        bits: Vec<u8>,
        /// Number of meaningful bits (equals the request's id count).
        count: u64,
    },
    /// Fetched block envelopes plus the ids that were absent.
    Blocks {
        /// Envelopes found.
        found: Vec<BlockEnvelope>,
        /// Ids the broker does not hold.
        missing: Vec<BlockId>,
    },
    /// Resolved commit root envelopes plus the commit ids that were absent.
    Commits {
        /// Root envelopes found.
        found: Vec<BlockEnvelope>,
        /// Commit ids the broker cannot resolve.
        missing: Vec<CommitId>,
    },
    /// Result of an idempotent `BlocksPut`.
    Stored {
        /// Newly stored envelopes.
        stored: u64,
        /// Envelopes already present with identical bytes.
        duplicate: u64,
    },
    /// A commit announcement was accepted and assigned a cursor.
    Published {
        /// The announced commit.
        commit_id: CommitId,
        /// Assigned per-topic cursor.
        cursor: u64,
    },
    /// Fan-out of a published commit.
    Event(Event),
    /// Fan-out notification that a topic's epoch advanced.
    EpochAdvanced(EpochAdvance),
    /// Typed failure.
    Error(BrokerError),
}

// ---- body keys, shared across the small per-kind bodies --------------------
// Bodies are small integer-keyed maps; each kind assigns its own meaning to these
// slots and its decoder rejects any key it does not know (fail-closed).
const B1: u64 = 1;
const B2: u64 = 2;
const B3: u64 = 3;
const B4: u64 = 4;
const B5: u64 = 5;
const B6: u64 = 6;

fn frame(request_id: u64, kind: u64, body: Vec<u8>) -> Vec<u8> {
    enc_map(vec![
        (F_VERSION, enc_uint(PROTOCOL_V0)),
        (F_REQUEST_ID, enc_uint(request_id)),
        (F_KIND, enc_uint(kind)),
        (F_BODY, body),
    ])
}

fn enc_envelopes(v: &[BlockEnvelope]) -> Vec<u8> {
    enc_array(&v.iter().map(|e| enc_bytes(&e.encode())).collect::<Vec<_>>())
}

fn read_envelopes(r: &mut Reader<'_>, limits: Limits) -> Result<Vec<BlockEnvelope>> {
    let n = r.array_header()?;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(BlockEnvelope::decode(r.bytes()?, limits)?);
    }
    Ok(out)
}

impl Request {
    /// The wire kind code.
    pub fn kind(&self) -> u64 {
        match self {
            Request::Hello(_) => K_HELLO,
            Request::OpenRepo(_) => K_OPEN_REPO,
            Request::PinRepo(_) => K_PIN_REPO,
            Request::RepoPinStatus { .. } => K_PIN_STATUS_REQ,
            Request::TopicSub(_) => K_TOPIC_SUB,
            Request::TopicUnsub { .. } => K_TOPIC_UNSUB,
            Request::TopicSyncReq(_) => K_TOPIC_SYNC_REQ,
            Request::BlocksExist { .. } => K_BLOCKS_EXIST,
            Request::BlocksGet { .. } => K_BLOCKS_GET,
            Request::BlocksPut { .. } => K_BLOCKS_PUT,
            Request::CommitGet { .. } => K_COMMIT_GET,
            Request::PublishEvent(_) => K_PUBLISH_EVENT,
            Request::EpochAdvance(_) => K_EPOCH_ADVANCE,
        }
    }

    fn body(&self) -> Vec<u8> {
        match self {
            Request::Hello(h) => enc_map(vec![
                (B1, enc_uints(&h.versions)),
                (B2, enc_texts(&h.suites)),
                (B3, enc_uint(h.max_block_size)),
                (B4, enc_uints(&h.padding_classes)),
                (
                    B5,
                    enc_array(
                        &h.header_modes
                            .iter()
                            .map(|m| enc_text(m.as_str()))
                            .collect::<Vec<_>>(),
                    ),
                ),
            ]),
            Request::OpenRepo(o) => {
                let mut e = vec![
                    (B1, enc_bytes(o.topic.as_bytes())),
                    (B2, enc_uint(o.epoch.0)),
                    (B3, enc_bytes(o.overlay.as_bytes())),
                    (B4, enc_bytes(o.peer.as_bytes())),
                ];
                if let Some(g) = &o.auth {
                    e.push((B5, g.encode()));
                }
                enc_map(e)
            }
            Request::PinRepo(p) => enc_map(vec![
                (B1, enc_bytes(p.topic.as_bytes())),
                (B3, enc_bool(p.pin)),
            ]),
            Request::RepoPinStatus { topic } | Request::TopicUnsub { topic } => {
                enc_map(vec![(B1, enc_bytes(topic.as_bytes()))])
            }
            Request::TopicSub(s) => {
                let mut e = vec![
                    (B1, enc_bytes(s.topic.as_bytes())),
                    (B2, enc_uint(s.epoch.0)),
                ];
                if let Some(c) = s.after_cursor {
                    e.push((B3, enc_uint(c)));
                }
                enc_map(e)
            }
            Request::TopicSyncReq(q) => {
                let mut e = vec![
                    (B1, enc_bytes(q.topic.as_bytes())),
                    (B2, enc_uint(q.epoch.0)),
                    (B3, enc_commit_ids(&q.known_heads)),
                ];
                if let Some(t) = &q.target_heads {
                    e.push((B4, enc_commit_ids(t)));
                }
                if let Some(b) = &q.known_commits {
                    e.push((
                        B5,
                        enc_map(vec![(1, enc_bytes(&b.bits)), (2, enc_uint(b.hashes))]),
                    ));
                }
                if let Some(p) = &q.page_after {
                    e.push((B6, enc_bytes(p.as_bytes())));
                }
                enc_map(e)
            }
            Request::BlocksExist { ids } | Request::BlocksGet { ids } => {
                enc_map(vec![(B3, enc_block_ids(ids))])
            }
            Request::BlocksPut { envelopes } => enc_map(vec![(B3, enc_envelopes(envelopes))]),
            Request::CommitGet { commit_ids } => enc_map(vec![(B3, enc_commit_ids(commit_ids))]),
            Request::PublishEvent(p) => p.encode_body(),
            Request::EpochAdvance(a) => a.encode_body(),
        }
    }

    /// Encode this request as a framed message with `request_id`.
    pub fn encode(&self, request_id: u64) -> Vec<u8> {
        frame(request_id, self.kind(), self.body())
    }

    /// Decode a framed request, returning `(request_id, request)`. Fail-closed on
    /// non-canonical CBOR, an unknown kind, a wrong version, a missing mandatory
    /// field, or any limit breach.
    pub fn decode(bytes: &[u8], limits: Limits) -> Result<(u64, Request)> {
        let mut r = Reader::new(bytes, limits);
        let mut version = None;
        let mut request_id = None;
        let mut kind = None;
        let mut req = None;
        read_struct_map(&mut r, |r, k| match k {
            F_VERSION => {
                version = Some(r.uint()?);
                Ok(true)
            }
            F_REQUEST_ID => {
                request_id = Some(r.uint()?);
                Ok(true)
            }
            F_KIND => {
                kind = Some(r.uint()?);
                Ok(true)
            }
            F_BODY => {
                let kind = need(kind, "frame kind must precede body")?;
                req = Some(Request::read_body(r, kind, limits)?);
                Ok(true)
            }
            _ => Ok(false),
        })?;
        r.finish()?;
        if version != Some(PROTOCOL_V0) {
            return Err(Error::Schema("frame version"));
        }
        Ok((
            need(request_id, "frame request_id")?,
            need(req, "frame body")?,
        ))
    }

    fn read_body(r: &mut Reader<'_>, kind: u64, limits: Limits) -> Result<Request> {
        match kind {
            K_HELLO => {
                let (mut v, mut s, mut mb, mut pc, mut hm) = (None, None, None, None, None);
                read_struct_map(r, |r, k| match k {
                    B1 => {
                        v = Some(read_uints(r)?);
                        Ok(true)
                    }
                    B2 => {
                        s = Some(read_texts(r)?);
                        Ok(true)
                    }
                    B3 => {
                        mb = Some(r.uint()?);
                        Ok(true)
                    }
                    B4 => {
                        pc = Some(read_uints(r)?);
                        Ok(true)
                    }
                    B5 => {
                        let n = r.array_header()?;
                        let mut out = Vec::with_capacity(n);
                        for _ in 0..n {
                            out.push(HeaderMode::from_token(r.text()?)?);
                        }
                        hm = Some(out);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                Ok(Request::Hello(Hello {
                    versions: need(v, "hello.versions")?,
                    suites: need(s, "hello.suites")?,
                    max_block_size: need(mb, "hello.max_block_size")?,
                    padding_classes: need(pc, "hello.padding_classes")?,
                    header_modes: need(hm, "hello.header_modes")?,
                }))
            }
            K_OPEN_REPO => {
                let (mut t, mut e, mut o, mut p, mut a) = (None, None, None, None, None);
                read_struct_map(r, |r, k| match k {
                    B1 => {
                        t = Some(TopicId::from_bytes(r.bytes_fixed::<32>()?));
                        Ok(true)
                    }
                    B2 => {
                        e = Some(Epoch(r.uint()?));
                        Ok(true)
                    }
                    B3 => {
                        o = Some(OverlayId::from_bytes(r.bytes_fixed::<32>()?));
                        Ok(true)
                    }
                    B4 => {
                        p = Some(PeerId::from_bytes(r.bytes_fixed::<32>()?));
                        Ok(true)
                    }
                    B5 => {
                        a = Some(AdmissionGrant::read(r)?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                Ok(Request::OpenRepo(OpenRepo {
                    overlay: need(o, "open.overlay")?,
                    topic: need(t, "open.topic")?,
                    epoch: need(e, "open.epoch")?,
                    peer: need(p, "open.peer")?,
                    auth: a,
                }))
            }
            K_PIN_REPO => {
                let (mut t, mut p) = (None, None);
                read_struct_map(r, |r, k| match k {
                    B1 => {
                        t = Some(TopicId::from_bytes(r.bytes_fixed::<32>()?));
                        Ok(true)
                    }
                    B3 => {
                        p = Some(read_bool(r)?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                Ok(Request::PinRepo(PinRepo {
                    topic: need(t, "pin.topic")?,
                    pin: need(p, "pin.pin")?,
                }))
            }
            K_PIN_STATUS_REQ | K_TOPIC_UNSUB => {
                let mut t = None;
                read_struct_map(r, |r, k| match k {
                    B1 => {
                        t = Some(TopicId::from_bytes(r.bytes_fixed::<32>()?));
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                let topic = need(t, "topic")?;
                Ok(if kind == K_PIN_STATUS_REQ {
                    Request::RepoPinStatus { topic }
                } else {
                    Request::TopicUnsub { topic }
                })
            }
            K_TOPIC_SUB => {
                let (mut t, mut e, mut c) = (None, None, None);
                read_struct_map(r, |r, k| match k {
                    B1 => {
                        t = Some(TopicId::from_bytes(r.bytes_fixed::<32>()?));
                        Ok(true)
                    }
                    B2 => {
                        e = Some(Epoch(r.uint()?));
                        Ok(true)
                    }
                    B3 => {
                        c = Some(r.uint()?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                Ok(Request::TopicSub(TopicSub {
                    topic: need(t, "sub.topic")?,
                    epoch: need(e, "sub.epoch")?,
                    after_cursor: c,
                }))
            }
            K_TOPIC_SYNC_REQ => {
                let (mut t, mut e, mut kh, mut th, mut bloom, mut page) =
                    (None, None, None, None, None, None);
                read_struct_map(r, |r, k| match k {
                    B1 => {
                        t = Some(TopicId::from_bytes(r.bytes_fixed::<32>()?));
                        Ok(true)
                    }
                    B2 => {
                        e = Some(Epoch(r.uint()?));
                        Ok(true)
                    }
                    B3 => {
                        kh = Some(read_commit_ids(r)?);
                        Ok(true)
                    }
                    B4 => {
                        th = Some(read_commit_ids(r)?);
                        Ok(true)
                    }
                    B5 => {
                        let (mut bits, mut hashes) = (None, None);
                        read_struct_map(r, |r, k| match k {
                            1 => {
                                bits = Some(r.bytes()?.to_vec());
                                Ok(true)
                            }
                            2 => {
                                hashes = Some(r.uint()?);
                                Ok(true)
                            }
                            _ => Ok(false),
                        })?;
                        bloom = Some(BloomHint {
                            bits: need(bits, "bloom.bits")?,
                            hashes: need(hashes, "bloom.hashes")?,
                        });
                        Ok(true)
                    }
                    B6 => {
                        page = Some(BlockId::from_bytes(r.bytes_fixed::<32>()?));
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                Ok(Request::TopicSyncReq(TopicSyncReq {
                    topic: need(t, "sync.topic")?,
                    epoch: need(e, "sync.epoch")?,
                    known_heads: need(kh, "sync.known_heads")?,
                    target_heads: th,
                    known_commits: bloom,
                    page_after: page,
                }))
            }
            K_BLOCKS_EXIST | K_BLOCKS_GET => {
                let mut ids = None;
                read_struct_map(r, |r, k| match k {
                    B3 => {
                        ids = Some(read_block_ids(r)?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                let ids = need(ids, "blocks.ids")?;
                Ok(if kind == K_BLOCKS_EXIST {
                    Request::BlocksExist { ids }
                } else {
                    Request::BlocksGet { ids }
                })
            }
            K_BLOCKS_PUT => {
                let mut envs = None;
                read_struct_map(r, |r, k| match k {
                    B3 => {
                        envs = Some(read_envelopes(r, limits)?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                Ok(Request::BlocksPut {
                    envelopes: need(envs, "put.envelopes")?,
                })
            }
            K_COMMIT_GET => {
                let mut ids = None;
                read_struct_map(r, |r, k| match k {
                    B3 => {
                        ids = Some(read_commit_ids(r)?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                Ok(Request::CommitGet {
                    commit_ids: need(ids, "commitget.ids")?,
                })
            }
            K_PUBLISH_EVENT => Ok(Request::PublishEvent(PublishEvent::read(r)?)),
            K_EPOCH_ADVANCE => Ok(Request::EpochAdvance(EpochAdvance::read(r)?)),
            _ => Err(Error::Schema("unknown request kind")),
        }
    }
}

impl Response {
    /// The wire kind code.
    pub fn kind(&self) -> u64 {
        match self {
            Response::HelloAck(_) => K_HELLO_ACK,
            Response::Ok => K_OK,
            Response::PinStatus(_) => K_PIN_STATUS,
            Response::SyncResp(_) => K_SYNC_RESP,
            Response::ExistBits { .. } => K_EXIST_BITS,
            Response::Blocks { .. } => K_BLOCKS,
            Response::Commits { .. } => K_COMMITS,
            Response::Stored { .. } => K_STORED,
            Response::Published { .. } => K_PUBLISHED,
            Response::Event(_) => K_EVENT,
            Response::EpochAdvanced(_) => K_EPOCH_ADVANCED,
            Response::Error(_) => K_ERROR,
        }
    }

    fn body(&self) -> Vec<u8> {
        match self {
            Response::HelloAck(a) => enc_map(vec![
                (B1, enc_uint(a.version)),
                (B2, enc_text(&a.suite)),
                (B3, enc_text(a.header_mode.as_str())),
                (B4, a.limits.encode()),
                (B5, enc_uints(&a.padding_classes)),
                (B6, a.retention.encode()),
            ]),
            Response::Ok => enc_map(vec![]),
            Response::PinStatus(p) => enc_map(vec![
                (B1, enc_bytes(p.topic.as_bytes())),
                (B3, enc_bool(p.pinned)),
                (B4, enc_uint(p.blocks)),
                (B5, enc_uint(p.bytes)),
                (B6, p.retention.encode()),
            ]),
            Response::SyncResp(s) => enc_map(vec![
                (B3, enc_commit_ids(&s.advertised_heads)),
                (B4, enc_block_ids(&s.missing_block_ids)),
                (B5, enc_uint(s.cursor)),
                (B6, enc_bool(s.more)),
            ]),
            Response::ExistBits { bits, count } => {
                enc_map(vec![(B3, enc_bytes(bits)), (B4, enc_uint(*count))])
            }
            Response::Blocks { found, missing } => enc_map(vec![
                (B3, enc_envelopes(found)),
                (B4, enc_block_ids(missing)),
            ]),
            Response::Commits { found, missing } => enc_map(vec![
                (B3, enc_envelopes(found)),
                (B4, enc_commit_ids(missing)),
            ]),
            Response::Stored { stored, duplicate } => {
                enc_map(vec![(B3, enc_uint(*stored)), (B4, enc_uint(*duplicate))])
            }
            Response::Published { commit_id, cursor } => enc_map(vec![
                (B3, enc_bytes(commit_id.as_bytes())),
                (B4, enc_uint(*cursor)),
            ]),
            Response::Event(e) => enc_map(vec![
                (B3, e.announcement.encode_body()),
                (B4, enc_uint(e.cursor)),
            ]),
            Response::EpochAdvanced(a) => a.encode_body(),
            Response::Error(e) => enc_map(vec![
                (B3, enc_uint(e.code as u64)),
                (B4, enc_text(&e.detail)),
            ]),
        }
    }

    /// Encode this response as a framed message echoing `request_id` (or `0` for
    /// an unsolicited push).
    pub fn encode(&self, request_id: u64) -> Vec<u8> {
        frame(request_id, self.kind(), self.body())
    }

    /// Decode a framed response, returning `(request_id, response)`.
    pub fn decode(bytes: &[u8], limits: Limits) -> Result<(u64, Response)> {
        let mut r = Reader::new(bytes, limits);
        let mut version = None;
        let mut request_id = None;
        let mut kind = None;
        let mut resp = None;
        read_struct_map(&mut r, |r, k| match k {
            F_VERSION => {
                version = Some(r.uint()?);
                Ok(true)
            }
            F_REQUEST_ID => {
                request_id = Some(r.uint()?);
                Ok(true)
            }
            F_KIND => {
                kind = Some(r.uint()?);
                Ok(true)
            }
            F_BODY => {
                let kind = need(kind, "frame kind must precede body")?;
                resp = Some(Response::read_body(r, kind, limits)?);
                Ok(true)
            }
            _ => Ok(false),
        })?;
        r.finish()?;
        if version != Some(PROTOCOL_V0) {
            return Err(Error::Schema("frame version"));
        }
        Ok((
            need(request_id, "frame request_id")?,
            need(resp, "frame body")?,
        ))
    }

    fn read_body(r: &mut Reader<'_>, kind: u64, limits: Limits) -> Result<Response> {
        match kind {
            K_HELLO_ACK => {
                let (mut v, mut s, mut hm, mut lim, mut pc, mut ret) =
                    (None, None, None, None, None, None);
                read_struct_map(r, |r, k| match k {
                    B1 => {
                        v = Some(r.uint()?);
                        Ok(true)
                    }
                    B2 => {
                        s = Some(r.text()?.to_owned());
                        Ok(true)
                    }
                    B3 => {
                        hm = Some(HeaderMode::from_token(r.text()?)?);
                        Ok(true)
                    }
                    B4 => {
                        lim = Some(WireLimits::read(r)?);
                        Ok(true)
                    }
                    B5 => {
                        pc = Some(read_uints(r)?);
                        Ok(true)
                    }
                    B6 => {
                        ret = Some(RetentionPolicy::read(r)?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                let suite = need(s, "ack.suite")?;
                check_suite(&suite)?;
                Ok(Response::HelloAck(HelloAck {
                    version: need(v, "ack.version")?,
                    suite,
                    header_mode: need(hm, "ack.header_mode")?,
                    limits: need(lim, "ack.limits")?,
                    padding_classes: need(pc, "ack.padding_classes")?,
                    retention: need(ret, "ack.retention")?,
                }))
            }
            K_OK => {
                read_struct_map(r, |_, _| Ok(false))?;
                Ok(Response::Ok)
            }
            K_PIN_STATUS => {
                let (mut t, mut p, mut b, mut by, mut ret) = (None, None, None, None, None);
                read_struct_map(r, |r, k| match k {
                    B1 => {
                        t = Some(TopicId::from_bytes(r.bytes_fixed::<32>()?));
                        Ok(true)
                    }
                    B3 => {
                        p = Some(read_bool(r)?);
                        Ok(true)
                    }
                    B4 => {
                        b = Some(r.uint()?);
                        Ok(true)
                    }
                    B5 => {
                        by = Some(r.uint()?);
                        Ok(true)
                    }
                    B6 => {
                        ret = Some(RetentionPolicy::read(r)?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                Ok(Response::PinStatus(PinStatus {
                    topic: need(t, "status.topic")?,
                    pinned: need(p, "status.pinned")?,
                    blocks: need(b, "status.blocks")?,
                    bytes: need(by, "status.bytes")?,
                    retention: need(ret, "status.retention")?,
                }))
            }
            K_SYNC_RESP => {
                let (mut ah, mut mb, mut c, mut more) = (None, None, None, None);
                read_struct_map(r, |r, k| match k {
                    B3 => {
                        ah = Some(read_commit_ids(r)?);
                        Ok(true)
                    }
                    B4 => {
                        mb = Some(read_block_ids(r)?);
                        Ok(true)
                    }
                    B5 => {
                        c = Some(r.uint()?);
                        Ok(true)
                    }
                    B6 => {
                        more = Some(read_bool(r)?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                Ok(Response::SyncResp(TopicSyncResp {
                    advertised_heads: need(ah, "sync.advertised_heads")?,
                    missing_block_ids: need(mb, "sync.missing_block_ids")?,
                    cursor: need(c, "sync.cursor")?,
                    more: need(more, "sync.more")?,
                }))
            }
            K_EXIST_BITS => {
                let (mut bits, mut count) = (None, None);
                read_struct_map(r, |r, k| match k {
                    B3 => {
                        bits = Some(r.bytes()?.to_vec());
                        Ok(true)
                    }
                    B4 => {
                        count = Some(r.uint()?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                Ok(Response::ExistBits {
                    bits: need(bits, "exist.bits")?,
                    count: need(count, "exist.count")?,
                })
            }
            K_BLOCKS | K_COMMITS => {
                let mut found = None;
                let mut missing_b = None;
                let mut missing_c = None;
                read_struct_map(r, |r, k| match k {
                    B3 => {
                        found = Some(read_envelopes(r, limits)?);
                        Ok(true)
                    }
                    B4 => {
                        if kind == K_BLOCKS {
                            missing_b = Some(read_block_ids(r)?);
                        } else {
                            missing_c = Some(read_commit_ids(r)?);
                        }
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                let found = need(found, "blocks.found")?;
                Ok(if kind == K_BLOCKS {
                    Response::Blocks {
                        found,
                        missing: need(missing_b, "blocks.missing")?,
                    }
                } else {
                    Response::Commits {
                        found,
                        missing: need(missing_c, "commits.missing")?,
                    }
                })
            }
            K_STORED => {
                let (mut s, mut d) = (None, None);
                read_struct_map(r, |r, k| match k {
                    B3 => {
                        s = Some(r.uint()?);
                        Ok(true)
                    }
                    B4 => {
                        d = Some(r.uint()?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                Ok(Response::Stored {
                    stored: need(s, "stored.stored")?,
                    duplicate: need(d, "stored.duplicate")?,
                })
            }
            K_PUBLISHED => {
                let (mut c, mut cur) = (None, None);
                read_struct_map(r, |r, k| match k {
                    B3 => {
                        c = Some(CommitId::from_bytes(r.bytes_fixed::<32>()?));
                        Ok(true)
                    }
                    B4 => {
                        cur = Some(r.uint()?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                Ok(Response::Published {
                    commit_id: need(c, "published.commit_id")?,
                    cursor: need(cur, "published.cursor")?,
                })
            }
            K_EVENT => {
                let (mut ann, mut cur) = (None, None);
                read_struct_map(r, |r, k| match k {
                    B3 => {
                        ann = Some(PublishEvent::read(r)?);
                        Ok(true)
                    }
                    B4 => {
                        cur = Some(r.uint()?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                Ok(Response::Event(Event {
                    announcement: need(ann, "event.announcement")?,
                    cursor: need(cur, "event.cursor")?,
                }))
            }
            K_EPOCH_ADVANCED => Ok(Response::EpochAdvanced(EpochAdvance::read(r)?)),
            K_ERROR => {
                let (mut c, mut d) = (None, None);
                read_struct_map(r, |r, k| match k {
                    B3 => {
                        c = Some(ErrorCode::from_code(r.uint()?)?);
                        Ok(true)
                    }
                    B4 => {
                        d = Some(r.text()?.to_owned());
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                Ok(Response::Error(BrokerError {
                    code: need(c, "error.code")?,
                    detail: need(d, "error.detail")?,
                }))
            }
            _ => Err(Error::Schema("unknown response kind")),
        }
    }
}

/// Build a client `Hello` offering exactly the v0 profile this crate implements.
pub fn hello_v0(max_block_size: u64) -> Hello {
    Hello {
        versions: vec![PROTOCOL_V0],
        suites: vec![SUITE_V0.to_string()],
        max_block_size,
        padding_classes: crate::envelope::PAD_CLASSES
            .iter()
            .map(|c| *c as u64)
            .collect(),
        header_modes: vec![HeaderMode::Opaque],
    }
}
