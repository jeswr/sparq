//! **Capabilities** (§4.2, §8.2): bearer secrets that encode *possession =
//! authority*. The profile keeps three authorities strictly separate, and this
//! module encodes that separation in the type + validation:
//!
//! * a **read** capability carries the branch read secret `K_read` (and permits
//!   intended decryption + topic subscription);
//! * a **publish** (write) capability additionally carries a publisher private
//!   key `sk_publish` (and registers `pk_publish` with the broker) — it does
//!   **not** carry admin authority;
//! * an **admin** capability carries a *distinct* `sk_admin` and never reuses
//!   `K_read` or `sk_publish`.
//!
//! A capability has a **canonical CBOR** representation (deterministic, so the
//! `cap_id` is stable) split into a **public grant** (safe to hand a broker for
//! admission, and the thing an admin signature authenticates) and **secret
//! fields** (`K_read`, private keys) that MUST be recipient-wrapped
//! ([`crate::wrap`]) or moved out of band — never placed in RDF, logs, URLs, or
//! topic messages (§4.2).

use crate::cbor::{
    enc_array, enc_bytes, enc_map, enc_text, enc_uint, read_struct_map, Limits, Reader,
};
use crate::error::{Error, Result};
use crate::ids::{BranchId, CapId, Epoch, RepoId, Secret32, TopicId};
use crate::sign::{PublicVerifyingKey, SecretSigningKey, PUBLIC_KEY_LEN, SIGNATURE_LEN};
use crate::suite::{check_suite, SUITE_V0};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// One of the three separated authorities (§4.2). The set carried by a
/// capability is canonicalized to this order (`read < publish < admin`) with no
/// duplicates, so its CBOR is deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Authority {
    /// Decrypt branch blocks and subscribe to its topic.
    Read,
    /// Publish signed commits to the branch topic.
    Publish,
    /// Create/rotate branches, change the publisher set, sign epoch transitions.
    Admin,
}

impl Authority {
    fn as_str(self) -> &'static str {
        match self {
            Authority::Read => "read",
            Authority::Publish => "publish",
            Authority::Admin => "admin",
        }
    }
    fn from_token(s: &str) -> Result<Self> {
        match s {
            "read" => Ok(Authority::Read),
            "publish" => Ok(Authority::Publish),
            "admin" => Ok(Authority::Admin),
            _ => Err(Error::Schema("unknown authority token")),
        }
    }
}

/// Canonicalize an authority set: sorted, deduplicated.
fn canon_authorities(mut v: Vec<Authority>) -> Vec<Authority> {
    v.sort();
    v.dedup();
    v
}

/// Validity window (§8.2 field 7): unix-second bounds. `not_before <= not_after`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Validity {
    /// Not valid before this unix second.
    pub not_before: u64,
    /// Not valid after this unix second.
    pub not_after: u64,
}

/// The **public grant** — everything a broker may see and the exact bytes an
/// admin signature authenticates. Contains no secret key material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicGrant {
    /// Repository identifier (§8.2 field 2).
    pub repo: RepoId,
    /// Branch identifier (§8.2 field 3).
    pub branch: BranchId,
    /// Epoch this grant is scoped to (§8.2 field 4).
    pub epoch: Epoch,
    /// Epoch-specific topic (§8.2 field 5).
    pub topic: TopicId,
    /// Authorities granted (§8.2 field 6), canonicalized.
    pub authority: Vec<Authority>,
    /// Validity window (§8.2 field 7).
    pub validity: Validity,
    /// Broker locators (§8.2 field 8).
    pub brokers: Vec<String>,
    /// Bound suite id (§8.2 field 9); MUST be the deployment's reviewed suite.
    pub suite: String,
    /// Random capability nonce (§8.2 field 13); makes the `cap_id` unlinkable to
    /// the grant's semantic fields.
    pub cap_nonce: [u8; 32],
    /// Publisher public key (profile field 15), present iff `publish` is granted.
    pub publisher_pub: Option<[u8; PUBLIC_KEY_LEN]>,
    /// Parent grant id for a delegated grant (profile field 16).
    pub parent_grant_id: Option<CapId>,
    /// Admin signature over the canonical public grant (§8.2 field 14).
    pub admin_sig: Option<[u8; SIGNATURE_LEN]>,
}

// wire keys (§8.2 + profile allocations 15/16)
const K_VERSION: u64 = 1;
const K_REPO: u64 = 2;
const K_BRANCH: u64 = 3;
const K_EPOCH: u64 = 4;
const K_TOPIC: u64 = 5;
const K_AUTHORITY: u64 = 6;
const K_VALIDITY: u64 = 7;
const K_BROKERS: u64 = 8;
const K_SUITE: u64 = 9;
const K_READ_SECRET: u64 = 10;
const K_PUBLISHER_SK: u64 = 11;
const K_ADMIN_SK: u64 = 12;
const K_CAP_NONCE: u64 = 13;
const K_ADMIN_SIG: u64 = 14;
const K_PUBLISHER_PK: u64 = 15;
const K_PARENT_GRANT: u64 = 16;
const VERSION: u64 = 0;

// validity sub-map keys
const K_NB: u64 = 1;
const K_NA: u64 = 2;

impl PublicGrant {
    /// The canonical public-grant fields **excluding** the admin signature
    /// (field 14). These are the exact bytes the admin signs and the `cap_id`
    /// hashes.
    fn signing_entries(&self) -> Vec<(u64, Vec<u8>)> {
        let mut e = vec![
            (K_VERSION, enc_uint(VERSION)),
            (K_REPO, enc_bytes(self.repo.as_bytes())),
            (K_BRANCH, enc_bytes(self.branch.as_bytes())),
            (K_EPOCH, enc_uint(self.epoch.0)),
            (K_TOPIC, enc_bytes(self.topic.as_bytes())),
            (
                K_AUTHORITY,
                enc_array(
                    &canon_authorities(self.authority.clone())
                        .iter()
                        .map(|a| enc_text(a.as_str()))
                        .collect::<Vec<_>>(),
                ),
            ),
            (
                K_VALIDITY,
                enc_map(vec![
                    (K_NB, enc_uint(self.validity.not_before)),
                    (K_NA, enc_uint(self.validity.not_after)),
                ]),
            ),
            (
                K_BROKERS,
                enc_array(&self.brokers.iter().map(|b| enc_text(b)).collect::<Vec<_>>()),
            ),
            (K_SUITE, enc_text(&self.suite)),
            (K_CAP_NONCE, enc_bytes(&self.cap_nonce)),
        ];
        if let Some(pk) = &self.publisher_pub {
            e.push((K_PUBLISHER_PK, enc_bytes(pk)));
        }
        if let Some(p) = &self.parent_grant_id {
            e.push((K_PARENT_GRANT, enc_bytes(p.as_bytes())));
        }
        e
    }

    /// The exact bytes the admin signs / the `cap_id` hashes.
    pub fn signing_bytes(&self) -> Vec<u8> {
        enc_map(self.signing_entries())
    }

    /// Local lookup handle `cap_id = SHA-256(canonical public grant)` (§4.2). The
    /// secret-bearing serialization is never used as an identifier.
    pub fn cap_id(&self) -> CapId {
        let mut h = Sha256::new();
        h.update(self.signing_bytes());
        CapId::from_bytes(h.finalize().into())
    }

    /// Canonical CBOR of the public grant, including the admin signature when set.
    pub fn encode(&self) -> Vec<u8> {
        let mut e = self.signing_entries();
        if let Some(sig) = &self.admin_sig {
            e.push((K_ADMIN_SIG, enc_bytes(sig)));
        }
        enc_map(e)
    }

    /// Sign this grant with an admin key, setting `admin_sig`.
    pub fn sign(&mut self, admin: &SecretSigningKey) {
        let sig = admin.sign(&self.signing_bytes());
        self.admin_sig = Some(sig);
    }

    /// Verify the admin signature against a trusted admin public key.
    pub fn verify(&self, admin_pub: &PublicVerifyingKey) -> Result<()> {
        let sig = self.admin_sig.ok_or(Error::Schema("missing admin signature"))?;
        admin_pub.verify(&self.signing_bytes(), &sig)
    }

    /// Decode + validate a public grant (fail-closed). Rejects an unknown suite,
    /// any secret field (10/11/12 are not permitted in a public grant), unknown
    /// mandatory fields, non-canonical CBOR, and an inconsistent authority set.
    pub fn decode(bytes: &[u8], limits: Limits) -> Result<Self> {
        let mut r = Reader::new(bytes, limits);
        let g = Self::decode_from(&mut r, false)?;
        r.finish()?;
        Ok(g)
    }

    /// Decode the public fields from an in-progress reader. `allow_secret`
    /// controls whether secret keys (10/11/12) are tolerated (they are consumed
    /// by [`Capability::decode`], never here).
    fn decode_from(r: &mut Reader<'_>, allow_secret: bool) -> Result<Self> {
        let mut version = None;
        let mut repo = None;
        let mut branch = None;
        let mut epoch = None;
        let mut topic = None;
        let mut authority: Option<Vec<Authority>> = None;
        let mut validity = None;
        let mut brokers: Option<Vec<String>> = None;
        let mut suite: Option<String> = None;
        let mut cap_nonce = None;
        let mut publisher_pub = None;
        let mut parent = None;
        let mut admin_sig = None;

        read_struct_map(r, |r, key| match key {
            K_VERSION => {
                version = Some(r.uint()?);
                Ok(true)
            }
            K_REPO => {
                repo = Some(RepoId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            K_BRANCH => {
                branch = Some(BranchId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            K_EPOCH => {
                epoch = Some(Epoch(r.uint()?));
                Ok(true)
            }
            K_TOPIC => {
                topic = Some(TopicId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            K_AUTHORITY => {
                let n = r.array_header()?;
                let mut v = Vec::with_capacity(n);
                for _ in 0..n {
                    v.push(Authority::from_token(r.text()?)?);
                }
                authority = Some(v);
                Ok(true)
            }
            K_VALIDITY => {
                let mut nb = None;
                let mut na = None;
                read_struct_map(r, |r, k| match k {
                    K_NB => {
                        nb = Some(r.uint()?);
                        Ok(true)
                    }
                    K_NA => {
                        na = Some(r.uint()?);
                        Ok(true)
                    }
                    _ => Ok(false),
                })?;
                validity = Some(Validity {
                    not_before: nb.ok_or(Error::Schema("missing not_before"))?,
                    not_after: na.ok_or(Error::Schema("missing not_after"))?,
                });
                Ok(true)
            }
            K_BROKERS => {
                let n = r.array_header()?;
                let mut v = Vec::with_capacity(n);
                for _ in 0..n {
                    v.push(r.text()?.to_owned());
                }
                brokers = Some(v);
                Ok(true)
            }
            K_SUITE => {
                suite = Some(r.text()?.to_owned());
                Ok(true)
            }
            K_CAP_NONCE => {
                cap_nonce = Some(r.bytes_fixed::<32>()?);
                Ok(true)
            }
            K_PUBLISHER_PK => {
                publisher_pub = Some(r.bytes_fixed::<PUBLIC_KEY_LEN>()?);
                Ok(true)
            }
            K_PARENT_GRANT => {
                parent = Some(CapId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            K_ADMIN_SIG => {
                admin_sig = Some(r.bytes_fixed::<SIGNATURE_LEN>()?);
                Ok(true)
            }
            // Secret keys: consumed by Capability::decode, never valid in a
            // standalone public grant.
            K_READ_SECRET | K_PUBLISHER_SK | K_ADMIN_SK if allow_secret => {
                // Let the caller re-read; signal "recognized but skip here".
                r.skip_value()?;
                Ok(true)
            }
            _ => Ok(false),
        })?;

        if version != Some(VERSION) {
            return Err(Error::Schema("capability version"));
        }
        let suite = suite.ok_or(Error::Schema("missing suite"))?;
        check_suite(&suite)?;
        // The wire authority array must ALREADY be canonical (sorted, deduplicated):
        // normalizing here would accept multiple byte encodings of one logical
        // grant, breaking the bytes-level contract behind cap_id / admin_sig.
        let authority = authority.ok_or(Error::Schema("missing authority"))?;
        if authority != canon_authorities(authority.clone()) {
            return Err(Error::NonCanonical("authority array not sorted/deduplicated"));
        }
        let validity = validity.ok_or(Error::Schema("missing validity"))?;
        if validity.not_before > validity.not_after {
            return Err(Error::Schema("validity not_before > not_after"));
        }
        // The publisher public key is present iff publish is granted: a grant
        // without publish authority must not carry (and have authenticated) an
        // unrelated publisher key.
        if authority.contains(&Authority::Publish) != publisher_pub.is_some() {
            return Err(Error::Separation("publisher key presence must match publish authority"));
        }
        Ok(PublicGrant {
            repo: repo.ok_or(Error::Schema("missing repo"))?,
            branch: branch.ok_or(Error::Schema("missing branch"))?,
            epoch: epoch.ok_or(Error::Schema("missing epoch"))?,
            topic: topic.ok_or(Error::Schema("missing topic"))?,
            authority,
            validity,
            brokers: brokers.ok_or(Error::Schema("missing brokers"))?,
            suite,
            cap_nonce: cap_nonce.ok_or(Error::Schema("missing cap_nonce"))?,
            publisher_pub,
            parent_grant_id: parent,
            admin_sig,
        })
    }
}

/// Constraints a delegated grant may narrow (never widen) relative to its parent
/// (§4.2). Branch stays fixed; the authority set, validity window, and an
/// optional maximum epoch may only shrink.
#[derive(Debug, Clone)]
pub struct Delegation {
    /// Authorities the child may exercise — MUST be a subset of the parent's.
    pub authority: Vec<Authority>,
    /// Child validity window — MUST be within the parent's.
    pub validity: Validity,
    /// Optional maximum epoch — MUST NOT exceed the parent's epoch semantics.
    pub max_epoch: Option<u64>,
}

/// A full capability, optionally bearing secrets. The secret fields never appear
/// in the public grant; serialize them only via [`Capability::encode_secret`]
/// for recipient-wrapped / out-of-band transfer.
#[derive(Debug)]
pub struct Capability {
    /// The public, signable grant.
    pub grant: PublicGrant,
    /// Branch read secret `K_read` (§8.2 field 10) — present for a read+ capability.
    pub read_secret: Option<Secret32>,
    /// Publisher private key seed (§8.2 field 11) — present for a write capability.
    pub publisher_sk: Option<[u8; 32]>,
    /// Admin private key seed (§8.2 field 12) — present for an admin capability.
    pub admin_sk: Option<[u8; 32]>,
}

impl Capability {
    /// Validate the read/publish/admin **separation** invariants (§4.2, §8.2):
    /// * a publisher private key and an admin private key are never combined;
    /// * `publish` authority ⟺ a publisher key pair is present, and the secret
    ///   key actually derives the grant's `publisher_pub` (so the key the
    ///   bearer signs with is the key the admin-signed grant / broker
    ///   registration binds);
    /// * `admin` authority ⟺ an admin private key is present;
    /// * `read` authority ⟺ a read secret is present.
    pub fn validate(&self) -> Result<()> {
        if self.publisher_sk.is_some() && self.admin_sk.is_some() {
            return Err(Error::Separation(
                "publisher and admin private keys must not be combined",
            ));
        }
        let auth = &self.grant.authority;
        if auth.contains(&Authority::Read) != self.read_secret.is_some() {
            return Err(Error::Separation("read authority <-> read secret mismatch"));
        }
        // Check the secret and public halves independently, so a non-publish
        // capability carrying only one of them (e.g. a stray publisher_pub) is
        // still rejected.
        let publish = auth.contains(&Authority::Publish);
        if publish != self.publisher_sk.is_some() {
            return Err(Error::Separation("publish authority <-> publisher secret mismatch"));
        }
        if publish != self.grant.publisher_pub.is_some() {
            return Err(Error::Separation("publish authority <-> publisher_pub mismatch"));
        }
        if let (Some(sk), Some(pk)) = (&self.publisher_sk, &self.grant.publisher_pub) {
            let derived = SecretSigningKey::from_seed(*sk).public().to_bytes();
            if !bool::from(derived.as_slice().ct_eq(pk.as_slice())) {
                return Err(Error::Separation(
                    "publisher secret does not derive the granted publisher_pub",
                ));
            }
        }
        if auth.contains(&Authority::Admin) != self.admin_sk.is_some() {
            return Err(Error::Separation("admin authority <-> admin key mismatch"));
        }
        Ok(())
    }

    /// Build a **read** capability (authority = {read}), carrying only `K_read`.
    pub fn new_read(grant_base: PublicGrant, read_secret: Secret32) -> Result<Self> {
        let mut grant = grant_base;
        grant.authority = vec![Authority::Read];
        grant.publisher_pub = None;
        let cap = Capability {
            grant,
            read_secret: Some(read_secret),
            publisher_sk: None,
            admin_sk: None,
        };
        cap.validate()?;
        Ok(cap)
    }

    /// Build a **write** (read+publish) capability, carrying `K_read` and a
    /// publisher key pair. Not admin.
    pub fn new_write(
        grant_base: PublicGrant,
        read_secret: Secret32,
        publisher: &SecretSigningKey,
    ) -> Result<Self> {
        let mut grant = grant_base;
        grant.authority = vec![Authority::Read, Authority::Publish];
        grant.publisher_pub = Some(publisher.public().to_bytes());
        let cap = Capability {
            grant,
            read_secret: Some(read_secret),
            publisher_sk: Some(publisher.to_seed()),
            admin_sk: None,
        };
        cap.validate()?;
        Ok(cap)
    }

    /// Build an **admin** capability, carrying a distinct `sk_admin`. It carries
    /// `admin` authority and MUST NOT reuse `K_read` or `sk_publish`.
    pub fn new_admin(grant_base: PublicGrant, admin: &SecretSigningKey) -> Result<Self> {
        let mut grant = grant_base;
        grant.authority = vec![Authority::Admin];
        grant.publisher_pub = None;
        let cap = Capability {
            grant,
            read_secret: None,
            publisher_sk: None,
            admin_sk: Some(admin.to_seed()),
        };
        cap.validate()?;
        Ok(cap)
    }

    /// Canonical CBOR including secret fields, for recipient-wrapped / out-of-band
    /// transfer ONLY. The result is a bearer secret; wrap it ([`crate::wrap`])
    /// before it leaves protected local storage.
    pub fn encode_secret(&self) -> Vec<u8> {
        let mut e = self.grant.signing_entries();
        if let Some(sig) = &self.grant.admin_sig {
            e.push((K_ADMIN_SIG, enc_bytes(sig)));
        }
        if let Some(rs) = &self.read_secret {
            e.push((K_READ_SECRET, enc_bytes(rs.expose())));
        }
        if let Some(sk) = &self.publisher_sk {
            e.push((K_PUBLISHER_SK, enc_bytes(sk)));
        }
        if let Some(sk) = &self.admin_sk {
            e.push((K_ADMIN_SK, enc_bytes(sk)));
        }
        enc_map(e)
    }

    /// Decode a full (possibly secret-bearing) capability and validate separation.
    pub fn decode_secret(bytes: &[u8], limits: Limits) -> Result<Self> {
        // We need the secret fields too, so decode the map twice: once for the
        // public grant (canonical validation), once to pull the secret fields.
        // A single pass keeps ordering checks intact.
        let mut r = Reader::new(bytes, limits);
        let mut read_secret: Option<[u8; 32]> = None;
        let mut publisher_sk: Option<[u8; 32]> = None;
        let mut admin_sk: Option<[u8; 32]> = None;

        // First pass over a cloned reader for the public grant.
        let grant = {
            let mut r2 = Reader::new(bytes, limits);
            let g = PublicGrant::decode_from(&mut r2, true)?;
            r2.finish()?;
            g
        };

        // Second pass to capture secrets (canonical order already validated).
        read_struct_map(&mut r, |r, key| match key {
            K_READ_SECRET => {
                read_secret = Some(r.bytes_fixed::<32>()?);
                Ok(true)
            }
            K_PUBLISHER_SK => {
                publisher_sk = Some(r.bytes_fixed::<32>()?);
                Ok(true)
            }
            K_ADMIN_SK => {
                admin_sk = Some(r.bytes_fixed::<32>()?);
                Ok(true)
            }
            // Skip every public field in this pass.
            K_VERSION | K_EPOCH => {
                r.uint()?;
                Ok(true)
            }
            K_REPO | K_BRANCH | K_TOPIC | K_CAP_NONCE | K_PUBLISHER_PK | K_PARENT_GRANT
            | K_ADMIN_SIG => {
                r.skip_value()?;
                Ok(true)
            }
            K_AUTHORITY | K_VALIDITY | K_BROKERS | K_SUITE => {
                r.skip_value()?;
                Ok(true)
            }
            _ => Ok(false),
        })?;
        r.finish()?;

        let cap = Capability {
            grant,
            read_secret: read_secret.map(Secret32),
            publisher_sk,
            admin_sk,
        };
        cap.validate()?;
        Ok(cap)
    }

    /// The corresponding publisher signing key, if this is a write capability.
    pub fn publisher_key(&self) -> Option<SecretSigningKey> {
        self.publisher_sk.map(SecretSigningKey::from_seed)
    }

    /// The corresponding admin signing key, if this is an admin capability.
    pub fn admin_key(&self) -> Option<SecretSigningKey> {
        self.admin_sk.map(SecretSigningKey::from_seed)
    }

    /// Overwrite the raw private-key seeds in place. `read_secret` is a
    /// [`Secret32`] and zeroizes via its own `Drop`; these two fields are bare
    /// `[u8; 32]` seeds, so [`Drop`] routes through here to give the whole
    /// capability the same secret-memory hygiene.
    fn zeroize_secrets(&mut self) {
        use zeroize::Zeroize;
        if let Some(sk) = self.publisher_sk.as_mut() {
            sk.zeroize();
        }
        if let Some(sk) = self.admin_sk.as_mut() {
            sk.zeroize();
        }
    }
}

/// Zero the raw private-key seed bytes when a capability is dropped, so a decoded
/// or constructed write/admin capability does not leave `sk_publish` / `sk_admin`
/// in freed memory (Cargo.toml secret-memory hygiene). `read_secret` is a
/// [`Secret32`] and already zeroizes itself.
impl Drop for Capability {
    fn drop(&mut self) {
        self.zeroize_secrets();
    }
}

/// Delegate a **new** public grant from `parent`, narrowing constraints, and
/// sign it with the admin key. The child's authority set MUST be a subset of the
/// parent's, its validity window within the parent's, and its `max_epoch` (if
/// any) no greater than the parent's epoch bound. Cryptographic key possession
/// is still required to act — a signed grant alone is not a decryption key.
pub fn delegate(
    parent: &PublicGrant,
    admin: &SecretSigningKey,
    d: Delegation,
) -> Result<PublicGrant> {
    let child_auth = canon_authorities(d.authority);
    let parent_auth = canon_authorities(parent.authority.clone());
    for a in &child_auth {
        if !parent_auth.contains(a) {
            return Err(Error::Delegation("authority not a subset of parent"));
        }
    }
    if d.validity.not_before < parent.validity.not_before
        || d.validity.not_after > parent.validity.not_after
        || d.validity.not_before > d.validity.not_after
    {
        return Err(Error::Delegation("validity window widens parent"));
    }
    let epoch = match d.max_epoch {
        Some(m) if m > parent.epoch.0 => {
            return Err(Error::Delegation("max_epoch exceeds parent epoch"))
        }
        Some(m) => Epoch(m),
        None => parent.epoch,
    };
    // A delegated grant does not itself carry the publisher key pair unless
    // publish is delegated AND the parent carried one to bind.
    let publisher_pub = if child_auth.contains(&Authority::Publish) {
        parent.publisher_pub
    } else {
        None
    };
    if child_auth.contains(&Authority::Publish) && publisher_pub.is_none() {
        return Err(Error::Delegation("cannot delegate publish without a publisher key"));
    }
    let mut child = PublicGrant {
        repo: parent.repo,
        branch: parent.branch,
        epoch,
        topic: parent.topic,
        authority: child_auth,
        validity: d.validity,
        brokers: parent.brokers.clone(),
        suite: parent.suite.clone(),
        cap_nonce: Secret32::random().0, // fresh nonce so the child cap_id differs
        publisher_pub,
        parent_grant_id: Some(parent.cap_id()),
        admin_sig: None,
    };
    child.sign(admin);
    Ok(child)
}

/// Convenience constructor for a base public grant with a fresh random cap nonce.
/// The suite is fixed to the bound v0 suite; authority/publisher fields are set
/// by the [`Capability`] constructors.
#[allow(clippy::too_many_arguments)]
pub fn base_grant(
    repo: RepoId,
    branch: BranchId,
    epoch: Epoch,
    topic: TopicId,
    validity: Validity,
    brokers: Vec<String>,
) -> PublicGrant {
    PublicGrant {
        repo,
        branch,
        epoch,
        topic,
        authority: vec![],
        validity,
        brokers,
        suite: SUITE_V0.to_string(),
        cap_nonce: Secret32::random().0,
        publisher_pub: None,
        parent_grant_id: None,
        admin_sig: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Encode a sample grant whose authority array carries exactly `tokens`,
    /// bypassing the canonicalization the honest encoder applies.
    fn grant_bytes_with_authority_tokens(tokens: &[&str]) -> Vec<u8> {
        let mut g = base_grant(
            RepoId::from_bytes([1u8; 32]),
            BranchId::from_bytes([2u8; 32]),
            Epoch(7),
            TopicId::from_bytes([3u8; 32]),
            Validity { not_before: 100, not_after: 200 },
            vec!["wss://broker.example".to_string()],
        );
        g.authority = vec![Authority::Read];
        let mut entries = g.signing_entries();
        let auth = enc_array(&tokens.iter().map(|t| enc_text(t)).collect::<Vec<_>>());
        for e in &mut entries {
            if e.0 == K_AUTHORITY {
                e.1 = auth.clone();
            }
        }
        enc_map(entries)
    }

    #[test]
    fn decode_rejects_out_of_order_authority_array() {
        let bytes = grant_bytes_with_authority_tokens(&["publish", "read"]);
        assert!(matches!(
            PublicGrant::decode(&bytes, Limits::default()),
            Err(Error::NonCanonical(_))
        ));
    }

    #[test]
    fn decode_rejects_duplicate_authority_array() {
        let bytes = grant_bytes_with_authority_tokens(&["read", "read"]);
        assert!(matches!(
            PublicGrant::decode(&bytes, Limits::default()),
            Err(Error::NonCanonical(_))
        ));
    }

    #[test]
    fn decode_accepts_canonical_authority_array() {
        let bytes = grant_bytes_with_authority_tokens(&["read"]);
        let g = PublicGrant::decode(&bytes, Limits::default()).unwrap();
        assert_eq!(g.authority, vec![Authority::Read]);
    }

    /// The `Drop` path must overwrite the raw private-key seeds. Observed via the
    /// same `zeroize_secrets` routine `Drop` calls, so a future field change that
    /// silently drops the guarantee fails here. `read_secret` is a `Secret32` and
    /// zeroizes through its own `Drop`.
    #[test]
    fn drop_zeroizes_private_key_seeds() {
        let base = base_grant(
            RepoId::from_bytes([1u8; 32]),
            BranchId::from_bytes([2u8; 32]),
            Epoch(7),
            TopicId::from_bytes([3u8; 32]),
            Validity { not_before: 100, not_after: 200 },
            vec!["wss://broker.example".to_string()],
        );

        let publisher = SecretSigningKey::from_seed([7u8; 32]);
        let mut write = Capability::new_write(base.clone(), Secret32([9u8; 32]), &publisher).unwrap();
        assert_eq!(write.publisher_sk, Some([7u8; 32]));
        write.zeroize_secrets();
        assert_eq!(write.publisher_sk, Some([0u8; 32]));

        let admin = SecretSigningKey::from_seed([5u8; 32]);
        let mut admin_cap = Capability::new_admin(base, &admin).unwrap();
        assert_eq!(admin_cap.admin_sk, Some([5u8; 32]));
        admin_cap.zeroize_secrets();
        assert_eq!(admin_cap.admin_sk, Some([0u8; 32]));
    }

    /// A read-only grant carrying a publisher_pub violates the
    /// `publisher_pub iff publish` invariant on public decode.
    #[test]
    fn public_decode_rejects_publisher_pub_without_publish() {
        let mut g = base_grant(
            RepoId::from_bytes([1u8; 32]),
            BranchId::from_bytes([2u8; 32]),
            Epoch(7),
            TopicId::from_bytes([3u8; 32]),
            Validity { not_before: 100, not_after: 200 },
            vec!["wss://broker.example".to_string()],
        );
        g.authority = vec![Authority::Read];
        g.publisher_pub = Some([4u8; PUBLIC_KEY_LEN]);
        assert!(matches!(
            PublicGrant::decode(&g.encode(), Limits::default()),
            Err(Error::Separation(_))
        ));
    }

    /// Same reverse mismatch through the secret-bearing decode path: a read
    /// capability whose grant smuggles a publisher_pub (no publisher_sk).
    #[test]
    fn secret_decode_rejects_publisher_pub_without_publish() {
        let mut g = base_grant(
            RepoId::from_bytes([1u8; 32]),
            BranchId::from_bytes([2u8; 32]),
            Epoch(7),
            TopicId::from_bytes([3u8; 32]),
            Validity { not_before: 100, not_after: 200 },
            vec!["wss://broker.example".to_string()],
        );
        g.authority = vec![Authority::Read];
        g.publisher_pub = Some([4u8; PUBLIC_KEY_LEN]);
        let cap = Capability {
            grant: g,
            read_secret: Some(Secret32([9u8; 32])),
            publisher_sk: None,
            admin_sk: None,
        };
        assert!(matches!(
            Capability::decode_secret(&cap.encode_secret(), Limits::default()),
            Err(Error::Separation(_))
        ));
        // validate() itself must also catch it, independent of decoding.
        assert!(matches!(cap.validate(), Err(Error::Separation(_))));
    }
}
