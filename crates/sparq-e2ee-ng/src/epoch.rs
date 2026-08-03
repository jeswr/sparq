//! **Epoch transitions** (§4.2, §8.3). Revocation cannot erase keys or plaintext
//! already obtained; instead an admin advances the epoch, minting a fresh topic,
//! read secret, and publisher set, and distributing new capabilities only to
//! remaining members. This module models the *signed transition record* that
//! authorizes that advance — the admin-authenticated statement a broker observes
//! before it swaps routing/admission state (§8.4 `EpochAdvance`).
//!
//! The transition binds the old/new epoch, old/new topics, the new
//! verification-key set, and the history policy under an admin signature (§8.3).
//! It does **not** carry the new read secret or new capabilities — those travel
//! recipient-wrapped, out of band (§4.2).

use crate::cbor::{
    enc_array, enc_bytes, enc_map, enc_text, enc_uint, read_struct_map, Limits, Reader,
};
use crate::error::{Error, Result};
use crate::ids::{BranchId, Epoch, RepoId, TopicId};
use crate::sign::{PublicVerifyingKey, SecretSigningKey, PUBLIC_KEY_LEN, SIGNATURE_LEN};

/// How historical (pre-transition) state is treated after the epoch advances
/// (§4.2). This is an **honest** disclosure, not a secrecy guarantee: neither
/// policy can retract plaintext a former member already decrypted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryPolicy {
    /// Old epochs remain readable to former members; only future writes are
    /// re-keyed.
    ForwardOnly,
    /// A fresh encrypted snapshot is produced under the new epoch and a
    /// garbage-collection policy applies to the old ciphertext.
    HistoryRekeyed,
}

impl HistoryPolicy {
    fn as_str(self) -> &'static str {
        match self {
            HistoryPolicy::ForwardOnly => "forward-only",
            HistoryPolicy::HistoryRekeyed => "history-rekeyed",
        }
    }
    fn from_token(s: &str) -> Result<Self> {
        match s {
            "forward-only" => Ok(HistoryPolicy::ForwardOnly),
            "history-rekeyed" => Ok(HistoryPolicy::HistoryRekeyed),
            _ => Err(Error::Schema("unknown history policy")),
        }
    }
}

/// A signed epoch-transition record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpochTransition {
    /// Repository id (bound to prevent cross-repo replay).
    pub repo: RepoId,
    /// Branch id (bound to prevent cross-branch replay).
    pub branch: BranchId,
    /// The epoch being left.
    pub old_epoch: Epoch,
    /// The epoch being entered (MUST be strictly greater).
    pub new_epoch: Epoch,
    /// The topic being retired.
    pub old_topic: TopicId,
    /// The topic for the new epoch.
    pub new_topic: TopicId,
    /// The new publisher verification-key set (raw Ed25519 public keys).
    pub new_publishers: Vec<[u8; PUBLIC_KEY_LEN]>,
    /// Declared treatment of historical state.
    pub history_policy: HistoryPolicy,
    /// Admin signature over fields 1..9 (set by [`EpochTransition::sign`]).
    pub admin_sig: Option<[u8; SIGNATURE_LEN]>,
}

// wire keys
const T_VERSION: u64 = 1;
const T_REPO: u64 = 2;
const T_BRANCH: u64 = 3;
const T_OLD_EPOCH: u64 = 4;
const T_NEW_EPOCH: u64 = 5;
const T_OLD_TOPIC: u64 = 6;
const T_NEW_TOPIC: u64 = 7;
const T_PUBLISHERS: u64 = 8;
const T_HISTORY: u64 = 9;
const T_SIG: u64 = 10;
const T_VERSION_VAL: u64 = 0;

impl EpochTransition {
    fn signing_entries(&self) -> Vec<(u64, Vec<u8>)> {
        vec![
            (T_VERSION, enc_uint(T_VERSION_VAL)),
            (T_REPO, enc_bytes(self.repo.as_bytes())),
            (T_BRANCH, enc_bytes(self.branch.as_bytes())),
            (T_OLD_EPOCH, enc_uint(self.old_epoch.0)),
            (T_NEW_EPOCH, enc_uint(self.new_epoch.0)),
            (T_OLD_TOPIC, enc_bytes(self.old_topic.as_bytes())),
            (T_NEW_TOPIC, enc_bytes(self.new_topic.as_bytes())),
            (
                T_PUBLISHERS,
                enc_array(
                    &self
                        .new_publishers
                        .iter()
                        .map(|p| enc_bytes(p))
                        .collect::<Vec<_>>(),
                ),
            ),
            (T_HISTORY, enc_text(self.history_policy.as_str())),
        ]
    }

    /// The exact bytes the admin signs (fields 1..9).
    pub fn signing_bytes(&self) -> Vec<u8> {
        enc_map(self.signing_entries())
    }

    /// Check the monotonicity invariant: the new epoch strictly follows the old.
    pub fn check_monotonic(&self) -> Result<()> {
        if self.new_epoch.0 > self.old_epoch.0 {
            Ok(())
        } else {
            Err(Error::Schema("epoch transition is not strictly increasing"))
        }
    }

    /// Sign the transition with an admin key (after checking monotonicity).
    pub fn sign(&mut self, admin: &SecretSigningKey) -> Result<()> {
        self.check_monotonic()?;
        self.admin_sig = Some(admin.sign(&self.signing_bytes()));
        Ok(())
    }

    /// Verify the admin signature and the monotonicity invariant.
    pub fn verify(&self, admin_pub: &PublicVerifyingKey) -> Result<()> {
        self.check_monotonic()?;
        let sig = self
            .admin_sig
            .ok_or(Error::Schema("missing admin signature"))?;
        admin_pub.verify(&self.signing_bytes(), &sig)
    }

    /// Canonical CBOR including the admin signature when set.
    pub fn encode(&self) -> Vec<u8> {
        let mut e = self.signing_entries();
        if let Some(sig) = &self.admin_sig {
            e.push((T_SIG, enc_bytes(sig)));
        }
        enc_map(e)
    }

    /// Decode + validate a transition record (fail-closed).
    pub fn decode(bytes: &[u8], limits: Limits) -> Result<Self> {
        let mut r = Reader::new(bytes, limits);
        let mut version = None;
        let mut repo = None;
        let mut branch = None;
        let mut oe = None;
        let mut ne = None;
        let mut ot = None;
        let mut nt = None;
        let mut pubs: Option<Vec<[u8; PUBLIC_KEY_LEN]>> = None;
        let mut hist: Option<HistoryPolicy> = None;
        let mut sig = None;
        read_struct_map(&mut r, |r, key| match key {
            T_VERSION => {
                version = Some(r.uint()?);
                Ok(true)
            }
            T_REPO => {
                repo = Some(RepoId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            T_BRANCH => {
                branch = Some(BranchId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            T_OLD_EPOCH => {
                oe = Some(Epoch(r.uint()?));
                Ok(true)
            }
            T_NEW_EPOCH => {
                ne = Some(Epoch(r.uint()?));
                Ok(true)
            }
            T_OLD_TOPIC => {
                ot = Some(TopicId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            T_NEW_TOPIC => {
                nt = Some(TopicId::from_bytes(r.bytes_fixed::<32>()?));
                Ok(true)
            }
            T_PUBLISHERS => {
                let n = r.array_header()?;
                let mut v = Vec::with_capacity(n);
                for _ in 0..n {
                    v.push(r.bytes_fixed::<PUBLIC_KEY_LEN>()?);
                }
                pubs = Some(v);
                Ok(true)
            }
            T_HISTORY => {
                hist = Some(HistoryPolicy::from_token(r.text()?)?);
                Ok(true)
            }
            T_SIG => {
                sig = Some(r.bytes_fixed::<SIGNATURE_LEN>()?);
                Ok(true)
            }
            _ => Ok(false),
        })?;
        r.finish()?;
        if version != Some(T_VERSION_VAL) {
            return Err(Error::Schema("epoch transition version"));
        }
        let t = EpochTransition {
            repo: repo.ok_or(Error::Schema("missing repo"))?,
            branch: branch.ok_or(Error::Schema("missing branch"))?,
            old_epoch: oe.ok_or(Error::Schema("missing old_epoch"))?,
            new_epoch: ne.ok_or(Error::Schema("missing new_epoch"))?,
            old_topic: ot.ok_or(Error::Schema("missing old_topic"))?,
            new_topic: nt.ok_or(Error::Schema("missing new_topic"))?,
            new_publishers: pubs.ok_or(Error::Schema("missing publishers"))?,
            history_policy: hist.ok_or(Error::Schema("missing history policy"))?,
            admin_sig: sig,
        };
        t.check_monotonic()?;
        Ok(t)
    }
}
