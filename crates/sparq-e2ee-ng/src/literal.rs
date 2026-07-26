//! **Profile SE ("structure-exposed") encrypted-literal codec** — the one E2EE
//! shape in which an *untrusted server* can still evaluate the **structural**
//! fragment of SPARQL, because RDF structure stays cleartext and only literal
//! **values** are AEAD-encrypted.
//!
//! Normative scope: `research/e2ee-queryable-options.md` §3.c. The complementary
//! layer in this same crate — [`crate::capability`] / [`crate::envelope`] /
//! [`crate::epoch`] — is the block-oriented Profile BR primitive set; this module
//! is the *value*-oriented one, and the two are cryptographically
//! domain-separated (a block ciphertext can never open as a literal, and vice
//! versa).
//!
//! # The leakage headline — read before using this (§3.c)
//!
//! **Profile SE reveals the FULL GRAPH TOPOLOGY to the server.** Every subject,
//! every predicate, every IRI-valued object, named-graph membership, node
//! degree, co-occurrence, and the whole update dynamic stay in the clear. And
//! because predicates come from published vocabularies, **the predicate announces
//! the *kind* of every hidden value** (`foaf:name`, `dbo:diagnosis` say what the
//! ciphertext is even though the ciphertext is opaque). Structure alone is highly
//! identifying: de-anonymization from graph topology is classical, and RDF hands
//! the observer labelled, ontology-typed edges on top.
//!
//! So, plainly: **Profile SE protects the values, not the shape of the user's
//! life.** It does *not* hide structure, and it does *not* make SPARQL run over
//! ciphertext. What runs server-side is the *structural* fragment only — BGP
//! matching and joins on subjects/predicates/IRI objects, property paths,
//! `OPTIONAL`/`UNION`/`MINUS` over structure, counting over structure. Anything
//! that touches an encrypted value is **opaque**: no value `FILTER`, no
//! `ORDER BY`, no value join, no value aggregation. Such answers come back
//! carrying ciphertext literals that the **client** decrypts and then
//! post-filters locally.
//!
//! The one deliberate exception is the *separately* opt-in equality tag
//! ([`equality_tag`]), which buys server-side value equality at the price of an
//! additional, separately-disclosed leakage increment — read that function's docs
//! before emitting one.
//!
//! Also disclosed: ciphertext **length** is a value-size fingerprint, which is
//! why every sealed value is padded to a bucket ([`SE_PAD_CLASSES`]); the bucket
//! itself is still visible.
//!
//! # Why the server needs no new code
//!
//! An SE value is *just a typed literal*: the canonical lexical form of
//! [`EncryptedLiteral::to_lexical`] with datatype [`SE_ENC_DATATYPE`]. Any
//! ordinary triplestore stores it, indexes it, and joins on the surrounding
//! structure with no cipher, no key, and no engine change. This crate therefore
//! adds **no** server-side decryption hook: an engine-side "decrypt in FILTER"
//! UDF is explicitly out of scope, because it would move key material
//! server-side and destroy the end-to-end property (survey §5.6).
//! `sparq-core` / `sparq-engine` / `sparq-substrate` stay cipher-free and do not
//! depend on this crate.
//!
//! # Honesty & audit boundary
//!
//! Everything here is **research-grade** and **externally UNAUDITED**. Every
//! confidentiality and integrity property is **designed/intended, NOT proven**;
//! there has been no external cryptographic review, the v0 suite name is a
//! placeholder, and production use is gated by **`sq-qhy4`**. Nothing in this
//! module is claimed to be a settled security result, and the leakage profile
//! above is an intrinsic property of the profile rather than a defect to be
//! fixed later.
//!
//! # Example
//!
//! ```
//! use sparq_e2ee_ng::ids::Secret32;
//! use sparq_e2ee_ng::literal::{open_literal, seal_literal, EncryptedLiteral, ValueContext,
//!                              SE_ENC_DATATYPE};
//!
//! let dek = Secret32::random();                       // per-predicate DEK, client-held
//! let ctx = ValueContext {
//!     predicate: "http://xmlns.com/foaf/0.1/name",     // stays CLEARTEXT in the graph
//!     graph: None,                                     // default graph
//!     subject: Some("https://alice.example/#me"),      // position-pin the ciphertext
//! };
//!
//! let lit = seal_literal(&dek, &ctx, "Alice", "http://www.w3.org/2001/XMLSchema#string")?;
//! // Emit it as an ordinary typed literal: "se0.…"^^<urn:…#enc>
//! let lexical = lit.to_lexical();
//! assert_eq!(lit.datatype(), SE_ENC_DATATYPE);
//!
//! // …round-trip on the client, from whatever the server handed back.
//! let parsed = EncryptedLiteral::from_lexical(&lexical)?;
//! let (value, datatype) = open_literal(&dek, &ctx, &parsed)?;
//! assert_eq!((value.as_str(), datatype.as_str()),
//!            ("Alice", "http://www.w3.org/2001/XMLSchema#string"));
//! # Ok::<(), sparq_e2ee_ng::Error>(())
//! ```

use crate::cbor::{enc_map, enc_text, enc_uint, read_struct_map, Limits, Reader};
use crate::envelope::{pad_to_classes, unpad_from_classes};
use crate::error::{Error, Result};
use crate::ids::Secret32;
use crate::keyschedule::value_key;
use crate::suite::{aead_open, aead_seal, check_suite, AEAD_NONCE_LEN, SUITE_V0};
use hmac::{Hmac, Mac};
use rand_core::{OsRng, RngCore};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

// ===========================================================================
// Datatype IRIs
// ===========================================================================

/// Datatype IRI of an **encrypted literal** (Profile SE). The lexical form is
/// [`EncryptedLiteral::to_lexical`]'s output.
///
/// This is a **non-dereferenceable placeholder** IRI for the draft spec; the
/// sibling spec draft declares the same two IRIs, and a reviewed profile will
/// mint stable ones.
pub const SE_ENC_DATATYPE: &str = "urn:jeswr:w3id:e2ee-sparql:draft:2026-07#enc";

/// Datatype IRI of an **equality tag** (Profile SE, *separately* opt-in — see
/// [`equality_tag`] for the leakage it adds). The lexical form is
/// [`eqtag_to_lexical`]'s output. Also a non-dereferenceable draft placeholder.
pub const SE_EQTAG_DATATYPE: &str = "urn:jeswr:w3id:e2ee-sparql:draft:2026-07#eqtag";

// ===========================================================================
// Wire / domain-separation constants
// ===========================================================================

/// Padding length classes for SE **values** (§3.c: an unpadded ciphertext length
/// is a value-size fingerprint). A value's padded plaintext is grown to the
/// smallest class that fits `4 + plaintext_len`, reusing the exact padding
/// discipline of the block envelope — literally the same helper,
/// `envelope::pad_to_classes` / `envelope::unpad_from_classes` (4-byte
/// big-endian real length, then zero fill), with a value-sized class table.
///
/// The table extends [`crate::envelope::PAD_CLASSES`] *downward* with two small
/// classes (64, 128) because literals are orders of magnitude smaller than
/// blocks, and the block table's 256-byte floor would inflate every short value;
/// the tail is exactly the first six block classes. It stops at 256 KiB — a value
/// bigger than that belongs in a block envelope, not in a literal.
///
/// Honest residual: the *class* is still visible to the server, so a value's
/// coarse size bucket leaks. Fewer, larger classes leak less and cost more.
pub const SE_PAD_CLASSES: [usize; 8] = [64, 128, 256, 1024, 4096, 16384, 65536, 262144];

/// Length of an equality tag in bytes (a truncated HMAC-SHA-256).
pub const EQTAG_LEN: usize = 16;

/// Poly1305 authentication-tag length appended by the v0 AEAD.
const AEAD_TAG_LEN: usize = 16;

/// Version of the SE value-envelope AEAD associated data + plaintext encoding.
const SE_VALUE_VERSION: u64 = 0;

/// Domain-separation string bound into the AEAD associated data of every SE
/// value. It is what makes a *value* envelope structurally unopenable as a
/// *block* envelope and vice versa (the two also derive keys under different
/// HKDF labels, so this is belt *and* braces).
const SE_VALUE_DOMAIN: &str = "urn:jeswr:w3id:e2ee-sparql:draft:2026-07 se-value-envelope v0";

/// Domain-separation prefix of the equality-tag HMAC message.
const SE_EQTAG_DOMAIN: &[u8] = b"urn:jeswr:w3id:e2ee-sparql:draft:2026-07 se-equality-tag v0";

/// The `se<version>` tag every SE lexical form starts with.
const SE_LEXICAL_TAG: &str = "se";

// AEAD associated-data map keys (all positive: unknown ones are rejected).
const A_VERSION: u64 = 1;
const A_SUITE: u64 = 2;
const A_DOMAIN: u64 = 3;
const A_PREDICATE: u64 = 4;
const A_GRAPH: u64 = 5;
const A_SUBJECT: u64 = 6;
const A_PAD: u64 = 7;

// Value-plaintext map keys.
const P_VERSION: u64 = 1;
const P_LEXICAL: u64 = 2;
const P_DATATYPE: u64 = 3;

// ===========================================================================
// ValueContext
// ===========================================================================

/// The associated-data binding for **one literal position**. Every field is
/// authenticated (not encrypted — these are cleartext in the graph anyway) so a
/// ciphertext cannot be relocated to a position it was not sealed for.
///
/// The same context must be supplied to [`seal_literal`] and [`open_literal`];
/// any difference fails closed with [`Error::Decrypt`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueContext<'a> {
    /// Predicate IRI of the triple whose object this value is. Also a key-schedule
    /// input ([`crate::keyschedule::value_key`]), so a DEK leaked for one
    /// predicate cannot open another predicate's values.
    pub predicate: &'a str,
    /// Named-graph IRI, or `None` for the default graph. Also a key-schedule
    /// input, so a value cannot be replayed across graphs.
    pub graph: Option<&'a str>,
    /// Subject IRI, bound when the deployment wants **position-pinned**
    /// ciphertext.
    ///
    /// **Disclosed integrity limit:** with `None`, nothing in the ciphertext ties
    /// it to a subject, so an untrusted server can **relocate a ciphertext from
    /// one subject to another undetected** — move Bob's sealed salary onto
    /// Alice's node and the client will decrypt it happily as Alice's. That is a
    /// real weakening, not a nit. `None` exists only because pinning the subject
    /// also prevents a legitimate client from *moving* a value between subjects
    /// without re-sealing it (and blank-node subjects have no stable IRI to pin).
    /// Pin it unless the deployment has a concrete reason not to.
    pub subject: Option<&'a str>,
}

/// Build the AEAD associated data for one SE value position.
///
/// Optional fields are **omitted** when `None` rather than encoded as an empty
/// string, so `graph: None` and `graph: Some("")` produce different bytes and
/// therefore cannot be confused.
fn value_associated_data(ctx: &ValueContext<'_>, pad_class: u64) -> Vec<u8> {
    let mut entries = vec![
        (A_VERSION, enc_uint(SE_VALUE_VERSION)),
        (A_SUITE, enc_text(SUITE_V0)),
        (A_DOMAIN, enc_text(SE_VALUE_DOMAIN)),
        (A_PREDICATE, enc_text(ctx.predicate)),
        (A_PAD, enc_uint(pad_class)),
    ];
    if let Some(g) = ctx.graph {
        entries.push((A_GRAPH, enc_text(g)));
    }
    if let Some(s) = ctx.subject {
        entries.push((A_SUBJECT, enc_text(s)));
    }
    enc_map(entries)
}

/// Canonical plaintext of a value: its lexical form **and its real datatype
/// IRI**, so the datatype is hidden from the server too (all the server sees is
/// [`SE_ENC_DATATYPE`], which would otherwise leak `xsd:date` vs `xsd:string`).
fn value_plaintext(lexical: &str, datatype: &str) -> Vec<u8> {
    enc_map(vec![
        (P_VERSION, enc_uint(SE_VALUE_VERSION)),
        (P_LEXICAL, enc_text(lexical)),
        (P_DATATYPE, enc_text(datatype)),
    ])
}

/// Parse a decrypted value plaintext (fail-closed, canonical-only).
fn parse_value_plaintext(bytes: &[u8]) -> Result<(String, String)> {
    let mut r = Reader::new(bytes, Limits::default());
    let mut version = None;
    let mut lexical: Option<String> = None;
    let mut datatype: Option<String> = None;
    read_struct_map(&mut r, |r, key| match key {
        P_VERSION => {
            version = Some(r.uint()?);
            Ok(true)
        }
        P_LEXICAL => {
            lexical = Some(r.text()?.to_owned());
            Ok(true)
        }
        P_DATATYPE => {
            datatype = Some(r.text()?.to_owned());
            Ok(true)
        }
        _ => Ok(false),
    })?;
    r.finish()?;
    if version != Some(SE_VALUE_VERSION) {
        return Err(Error::Schema("SE value version"));
    }
    Ok((
        lexical.ok_or(Error::Schema("missing SE lexical form"))?,
        datatype.ok_or(Error::Schema("missing SE datatype"))?,
    ))
}

// ===========================================================================
// EncryptedLiteral
// ===========================================================================

/// One AEAD-sealed literal value, plus the random nonce it was sealed under.
///
/// Its canonical RDF surface form is [`to_lexical`](EncryptedLiteral::to_lexical)
/// with datatype [`SE_ENC_DATATYPE`]. Nothing about the *position* is carried on
/// the wire — the predicate/graph/subject binding lives in the AEAD associated
/// data and is reconstructed by the client from the triple it is looking at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedLiteral {
    /// The fresh random AEAD nonce this value was sealed under. Randomized, never
    /// derived from the plaintext: SE deliberately has **no** deterministic /
    /// convergent mode, because equal ciphertexts for equal values are exactly
    /// the leakage the profile refuses by default (see [`equality_tag`] for the
    /// separately opt-in alternative).
    pub nonce: [u8; AEAD_NONCE_LEN],
    /// AEAD ciphertext-and-tag over the padded plaintext.
    pub ciphertext: Vec<u8>,
}

impl EncryptedLiteral {
    /// The datatype IRI to publish this value with, i.e. [`SE_ENC_DATATYPE`].
    pub fn datatype(&self) -> &'static str {
        SE_ENC_DATATYPE
    }

    /// The padded-plaintext length class implied by the ciphertext length, after
    /// validating that the ciphertext really is `class + tag` bytes for some
    /// class in [`SE_PAD_CLASSES`]. Fail-closed: a length that is not a valid
    /// class is [`Error::Malformed`].
    ///
    /// The class is an AEAD-bound field, so this value is *checked*, not merely
    /// derived — a truncated or extended ciphertext cannot silently shift class.
    pub fn pad_class(&self) -> Result<u64> {
        let padded = self
            .ciphertext
            .len()
            .checked_sub(AEAD_TAG_LEN)
            .ok_or(Error::Malformed("SE ciphertext shorter than the AEAD tag"))?;
        if !SE_PAD_CLASSES.contains(&padded) {
            return Err(Error::Malformed(
                "SE ciphertext length is not a pad class plus tag",
            ));
        }
        Ok(padded as u64)
    }

    /// The **canonical, deterministic** RDF lexical form:
    /// `se0.<nonce-hex>.<ciphertext-hex>` with lowercase hex. Use it with
    /// [`SE_ENC_DATATYPE`] and the value is an ordinary typed literal to any
    /// triplestore.
    ///
    /// The leading `se0` is a **combined version-and-suite tag**: `0` denotes
    /// exactly {format version 0, [`SUITE_V0`]}, and [`from_lexical`] maps it back
    /// through [`check_suite`] so an unrecognized tag fails closed rather than
    /// being guessed at. It is a compact tag rather than the spelled-out suite URN
    /// because that URN is longer than most encrypted values; the *full* suite id
    /// string is still bound in the AEAD associated data, so a suite substitution
    /// cannot open a value. The pad class is likewise implied by (and validated
    /// against) the ciphertext length rather than restated on the wire.
    ///
    /// [`from_lexical`]: EncryptedLiteral::from_lexical
    pub fn to_lexical(&self) -> String {
        let mut s = String::with_capacity(
            SE_LEXICAL_TAG.len() + 3 + 2 * (AEAD_NONCE_LEN + self.ciphertext.len()),
        );
        s.push_str(SE_LEXICAL_TAG);
        s.push('0'); // suite/version tag: "0" == SUITE_V0
        s.push('.');
        hex_encode_into(&self.nonce, &mut s);
        s.push('.');
        hex_encode_into(&self.ciphertext, &mut s);
        s
    }

    /// Parse a canonical lexical form. **Fail-closed**: rejects a wrong or absent
    /// `se<version>` tag ([`Error::UnknownSuite`] for a version this build does
    /// not implement), uppercase or otherwise non-canonical hex
    /// ([`Error::NonCanonical`]), odd-length or non-hex digits, the wrong nonce
    /// length, a ciphertext whose length is not a [`SE_PAD_CLASSES`] class plus
    /// the AEAD tag, and any extra field or trailing byte
    /// ([`Error::Malformed`]).
    ///
    /// Like the CBOR reader's [`Limits`], the length ceiling is checked **before**
    /// any allocation proportional to the declared size, so an adversarially long
    /// lexical form is rejected ([`Error::LimitExceeded`]) rather than decoded.
    ///
    /// Round-trip is exact: `from_lexical(&x.to_lexical()) == Ok(x)`, and
    /// `to_lexical` is the only encoding this accepts.
    pub fn from_lexical(s: &str) -> Result<Self> {
        let rest = s
            .strip_prefix(SE_LEXICAL_TAG)
            .ok_or(Error::Malformed("SE literal must start with the se tag"))?;
        let (version, rest) = rest
            .split_once('.')
            .ok_or(Error::Malformed("SE literal missing the version separator"))?;
        // Algorithm agility: the version tag selects the suite; anything this
        // build does not implement fails closed rather than being guessed at.
        check_suite(suite_for_version(version)?)?;
        let (nonce_hex, ct_hex) = rest.split_once('.').ok_or(Error::Malformed(
            "SE literal missing the ciphertext separator",
        ))?;
        if ct_hex.contains('.') {
            return Err(Error::Malformed("SE literal has a trailing field"));
        }
        // Ceiling check BEFORE decoding: the largest a valid SE ciphertext can be
        // is the largest pad class plus the AEAD tag, hex-expanded.
        let max_ct_hex = 2 * (SE_PAD_CLASSES[SE_PAD_CLASSES.len() - 1] + AEAD_TAG_LEN);
        if nonce_hex.len() > 2 * AEAD_NONCE_LEN || ct_hex.len() > max_ct_hex {
            return Err(Error::LimitExceeded("SE literal longer than any pad class"));
        }
        let mut nonce = [0u8; AEAD_NONCE_LEN];
        let nonce_bytes = hex_decode(nonce_hex)?;
        if nonce_bytes.len() != AEAD_NONCE_LEN {
            return Err(Error::Malformed("SE literal nonce length"));
        }
        nonce.copy_from_slice(&nonce_bytes);
        let lit = EncryptedLiteral {
            nonce,
            ciphertext: hex_decode(ct_hex)?,
        };
        lit.pad_class()?;
        Ok(lit)
    }
}

/// Map an SE lexical version tag to the suite it pins. `"0"` is the v0 suite;
/// every other tag (including `"00"`, `""`, or a future `"1"`) is an unknown
/// suite, never a silent substitution.
fn suite_for_version(tag: &str) -> Result<&'static str> {
    match tag {
        "0" => Ok(SUITE_V0),
        _ => Err(Error::UnknownSuite),
    }
}

// ===========================================================================
// seal / open
// ===========================================================================

/// AEAD-seal one literal value under a **fresh random nonce**.
///
/// The plaintext is `(lexical, datatype)` in deterministic CBOR — the real
/// datatype travels *inside* the ciphertext, so the server does not learn
/// `xsd:date` vs `xsd:string` — padded to a [`SE_PAD_CLASSES`] bucket via the
/// block envelope's own `pad_to_classes` helper before sealing, because raw
/// ciphertext length is a value-size fingerprint.
///
/// The associated data binds a version byte, the suite id, the SE value-envelope
/// domain string, and every [`ValueContext`] field, so a ciphertext is opaque
/// *and* non-relocatable across predicate/graph/(optionally) subject, and can
/// never be opened as a [`crate::envelope::BlockEnvelope`].
///
/// There is deliberately **no deterministic mode** and this never emits an
/// equality tag: see [`equality_tag`] if the deployment consciously wants that
/// leakage.
///
/// Rejects an attempt to seal a value that already claims an SE datatype
/// ([`SE_ENC_DATATYPE`] / [`SE_EQTAG_DATATYPE`]) with [`Error::Schema`], so a
/// double-seal cannot be mistaken for a single one.
pub fn seal_literal(
    dek: &Secret32,
    ctx: &ValueContext<'_>,
    lexical: &str,
    datatype: &str,
) -> Result<EncryptedLiteral> {
    if datatype == SE_ENC_DATATYPE || datatype == SE_EQTAG_DATATYPE {
        return Err(Error::Schema(
            "cannot seal an SE datatype as a plaintext value",
        ));
    }
    let padded = pad_to_classes(&value_plaintext(lexical, datatype), &SE_PAD_CLASSES)?;
    let pad_class = padded.len() as u64;
    let key = value_key(dek, ctx.predicate, ctx.graph);
    let mut nonce = [0u8; AEAD_NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    let aad = value_associated_data(ctx, pad_class);
    Ok(EncryptedLiteral {
        nonce,
        ciphertext: aead_seal(&key, &nonce, &aad, &padded),
    })
}

/// Open one sealed literal, returning `(lexical, datatype)`.
///
/// Fails closed with [`Error::Decrypt`] on a wrong DEK, a tampered ciphertext, or
/// **any** wrong [`ValueContext`] field (predicate, graph, subject), and on a
/// ciphertext that was sealed as something other than an SE value (a block
/// envelope, say).
pub fn open_literal(
    dek: &Secret32,
    ctx: &ValueContext<'_>,
    lit: &EncryptedLiteral,
) -> Result<(String, String)> {
    let pad_class = lit.pad_class()?;
    let key = value_key(dek, ctx.predicate, ctx.graph);
    let aad = value_associated_data(ctx, pad_class);
    let padded = aead_open(&key, &lit.nonce, &aad, &lit.ciphertext)?;
    parse_value_plaintext(&unpad_from_classes(&padded, &SE_PAD_CLASSES)?)
}

// ===========================================================================
// Equality tags — SEPARATELY opt-in, SEPARATELY disclosed leakage
// ===========================================================================

/// Derive the **equality tag** of one value — a truncated, domain-separated
/// HMAC-SHA-256 under the same per-(predicate, graph) key as the value envelope.
///
/// # This is a separate, separately-disclosed leakage increment
///
/// Emitting a tag is **NOT** something you get by simply using Profile SE.
/// [`seal_literal`] never produces one; a deployment must decide to publish tags,
/// per predicate, as its own disclosed step — and it steps onto the
/// **deterministic-encryption leakage ladder** by doing so:
///
/// * equal `(lexical, datatype)` values under the same predicate/graph produce
///   **equal tags**, so the server learns the full **value-equality pattern**;
/// * that is per-predicate **value-frequency leakage** — the server can count how
///   often each hidden value occurs, and frequency analysis against auxiliary
///   data (a census list, a code table, a public vocabulary) is the classical way
///   deterministic columns get recovered outright, especially for the skewed,
///   low-entropy literals a personal dataset is full of;
/// * the leak is *permanent* for anything already published, and it stacks with
///   the topology the profile already exposes.
///
/// What it buys, in exchange: server-side value-equality **joins** and
/// `FILTER(?x = <const>)`, because the client can derive the tag of a constant
/// and hand it to the server as an ordinary term (see [`SE_EQTAG_DATATYPE`] /
/// [`eqtag_to_lexical`]). Nothing else — no ordering, no ranges, no
/// substring matching.
///
/// # Scope
///
/// The tag binds the predicate and graph (via
/// [`crate::keyschedule::value_key`]) but **deliberately NOT the subject**: it
/// has to be comparable across subjects or it could not serve a join. So
/// [`ValueContext::subject`] is **ignored** here even when it is `Some`. Two
/// different predicates, or two different graphs, yield unrelated tags, which is
/// what keeps the frequency leak scoped to one predicate at a time.
///
/// Truncation to [`EQTAG_LEN`] bytes bounds tag size; a collision makes the
/// server report a spurious equality, so a client that cares must re-check
/// equality after decrypting. Compare tags with [`tags_equal`].
pub fn equality_tag(
    dek: &Secret32,
    ctx: &ValueContext<'_>,
    lexical: &str,
    datatype: &str,
) -> [u8; EQTAG_LEN] {
    let key = value_key(dek, ctx.predicate, ctx.graph);
    let mut mac = <HmacSha256 as Mac>::new_from_slice(&key).expect("hmac 32-byte key");
    // Domain separation from the AEAD use of the same key, then a
    // length-delimited message so ("ab","c") and ("a","bc") cannot collide.
    mac.update(SE_EQTAG_DOMAIN);
    for part in [lexical.as_bytes(), datatype.as_bytes()] {
        mac.update(&(part.len() as u64).to_be_bytes());
        mac.update(part);
    }
    let full = mac.finalize().into_bytes();
    let mut tag = [0u8; EQTAG_LEN];
    tag.copy_from_slice(&full[..EQTAG_LEN]);
    tag
}

/// Constant-time equality-tag comparison (the tag is derived from secret-keyed
/// material, so a data-dependent early exit is avoided).
pub fn tags_equal(a: &[u8; EQTAG_LEN], b: &[u8; EQTAG_LEN]) -> bool {
    a.ct_eq(b).into()
}

/// Canonical lexical form of an equality tag: lowercase hex, to publish with
/// [`SE_EQTAG_DATATYPE`].
pub fn eqtag_to_lexical(tag: &[u8; EQTAG_LEN]) -> String {
    let mut s = String::with_capacity(2 * EQTAG_LEN);
    hex_encode_into(tag, &mut s);
    s
}

/// Parse an equality tag from its canonical lexical form. Fail-closed on
/// non-lowercase / non-hex digits and on any length other than [`EQTAG_LEN`]
/// bytes.
pub fn eqtag_from_lexical(s: &str) -> Result<[u8; EQTAG_LEN]> {
    let bytes = hex_decode(s)?;
    if bytes.len() != EQTAG_LEN {
        return Err(Error::Malformed("SE equality tag length"));
    }
    let mut tag = [0u8; EQTAG_LEN];
    tag.copy_from_slice(&bytes);
    Ok(tag)
}

// ===========================================================================
// Local hex codec (hex is a dev-dependency only; no runtime dep is added)
// ===========================================================================

/// Append lowercase hex of `bytes` to `out`.
fn hex_encode_into(bytes: &[u8], out: &mut String) {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    for b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 0x0f) as usize] as char);
    }
}

/// Decode **canonical** (lowercase, even-length, non-empty) hex. Uppercase is a
/// non-canonical encoding of the same bytes and is rejected rather than
/// normalized, so a lexical form has exactly one spelling.
fn hex_decode(s: &str) -> Result<Vec<u8>> {
    if s.is_empty() {
        return Err(Error::Malformed("empty hex field"));
    }
    if !s.len().is_multiple_of(2) {
        return Err(Error::Malformed("odd-length hex field"));
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(s.len() / 2);
    for pair in bytes.chunks_exact(2) {
        out.push((hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?);
    }
    Ok(out)
}

/// One lowercase hex digit to its nibble.
fn hex_nibble(c: u8) -> Result<u8> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Err(Error::NonCanonical("hex must be lowercase")),
        _ => Err(Error::Malformed("hex digit expected")),
    }
}
