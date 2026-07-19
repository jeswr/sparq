// [OPUS-4.8] sq-34ml: M4-v1 prerequisites — freshness/replay binding + the
// federated (multi-source) public-input layout. Scoping + OUT-OF-CIRCUIT binding
// ONLY; no signature gadget, no proof (those stay the honest stubs in `proof.rs`).
//! M4-v1 prerequisites (sq-34ml): the two BUILDABLE, non-research pieces split
//! out of the M4 collaborative-proof SPIKE so the *verifier-side-attestation
//! interim* (sq-f7bu — Artemis commit-and-prove anchor + Dutta authenticated
//! input) can progress while the genuine Q1 novelty (the distributed in-circuit
//! signature over a secret-shared witness) stays an audit-gated SPIKE (sq-bjl).
//!
//! Feasibility record: `research/mpc-m4-distributed-sig-feasibility.md` §2 / §4.
//!
//! Two prerequisites, both currently bundled inside the SPIKE but neither of
//! them research:
//!
//! 1. [`FreshnessBinding`] — the **explicit out-of-circuit freshness/replay
//!    binding** (RQ1 audit #4). In the single-prover *in-circuit* path the
//!    verifier's fresh challenge `N` is a circuit public input, so the proof is
//!    automatically bound to `N` and cannot be replayed. In the M4-v1
//!    verifier-side-attestation interim the issuer signature is checked
//!    **outside** the circuit (the Artemis commit-and-prove anchor: the
//!    signature is verified separately and only the commitment is linked to the
//!    proof), so `N` is **not** automatically part of what the signature covers.
//!    This type derives the domain-separated transcript the out-of-circuit
//!    signature check must cover, folding in `N` + the session id so an old
//!    `(statement, proof, signature)` triple cannot be replayed under a fresh
//!    verifier nonce or lifted onto a different federated statement.
//!
//! 2. [`reconstruct_federated_public_inputs`] — the **federated / multi-source
//!    public-input layout** spanning all holders' `commitments` / `rows` /
//!    `attribution`. Each source's segment is byte-for-byte the single-source
//!    layout produced by `sparq-zk-compose::verifier::reconstruct_public_inputs`
//!    (RQ1 audit #1, with the in-circuit per-row `attribution[k]` of audit #8),
//!    concatenated in a caller-chosen canonical holder order under the ONE
//!    shared verifier challenge `N`. "byte-compared unchanged": a source's slice
//!    of the federated vector equals that source's own single-source proof
//!    `public_inputs`, so the hardened audit-#1 byte binding is reused verbatim
//!    per source — the federation adds ordering + the shared nonce, nothing else.
//!
//! ## Honesty boundary (what this module is NOT)
//!
//! This is **scoping + out-of-circuit binding only**. It lays out and binds
//! public-input bytes and derives the message the out-of-circuit signature must
//! cover; it does **not** verify a signature, open a commitment, secret-share a
//! witness, or emit/verify a proof. The attestation and collaborative proof
//! themselves remain the honest [`crate::proof`] `NotYetImplemented` stubs,
//! gated on the ZK-foundation remediation (#3 issuer-signature / #4 replay /
//! #8/#9 attribution / …) and the Q1 spike. **No fake crypto:** every field this
//! module handles is a public input or a public binding transcript; nothing
//! secret is reconstructed and no cryptographic claim is asserted here.
//!
//! ## Cross-crate layout obligation
//!
//! The per-source segment MUST stay byte-identical to
//! `sparq-zk-compose::verifier::reconstruct_public_inputs`. `sparq-mpc` is
//! deliberately dependency-light and does NOT depend on `sparq-zk-compose` (which
//! pulls the heavy bb / arkworks estate), so the layout is *mirrored* here rather
//! than shared, and the `scan_segment_matches_single_source_layout` test pins the
//! exact bytes of the documented `scan_k{k}_n{n}_r{r}` layout. If the verifier's
//! reconstruction changes, that test is the tripwire; keep the two in lockstep.

use crate::partial::{HolderId, MpcError};
use sha2::{Digest, Sha512};

/// One 32-byte **big-endian** field word — the atom of bb's `public_inputs`
/// blob (each public input is a fixed-width 32-byte BE field element, no length
/// prefix; the layout was determined empirically against bb 5.0.0-nightly, see
/// the `sparq-zk-compose` verifier). Held as raw bytes so this crate reproduces
/// the verifier byte layout WITHOUT depending on the heavy `sparq-zk-compose` /
/// arkworks `Fr` type — a commitment / encoding word is supplied already in its
/// canonical 32-byte BE form (the same word `field_to_be_bytes_32` emits).
pub type Word = [u8; 32];

/// A `u32` / `u64` / `bool` / op-code encoded as a **right-aligned** 32-byte BE
/// word — matching the verifier's `push_uint` (`bool -> {0,1}`, integer in the
/// low 8 bytes). Equal to `field_to_be_bytes_32` of the same small integer.
fn uint_word(v: u64) -> Word {
    let mut w = [0u8; 32];
    w[24..].copy_from_slice(&v.to_be_bytes());
    w
}

/// The three-slot pattern arity of a `scan` circuit (subject/predicate/object).
const PATTERN_SLOTS: usize = 3;

/// One source's (holder's) `scan` public-input segment — exactly the fields the
/// single-source `scan_k{k}_n{n}_r{r}` circuit declares AFTER the shared
/// challenge, in `main` declaration order:
///
/// `commitments[k]`, `pattern_is_const[3]`, `pattern_const_enc[3]`,
/// `rows[r][3]` (row-major, zero-padded to `r`), `row_count`, `attribution[k]`.
///
/// (`sparq-zk-compose::verifier::reconstruct_public_inputs`, audit #1 / #8.)
/// The `scan` member is the commitment-bearing one — it carries the
/// `commitments` / `rows` / `attribution` the federated attestation spans; the
/// commitment-free `filter_*` members are not part of the multi-source
/// commitment layout and are not modelled here.
#[derive(Debug, Clone)]
pub struct SourceSegment {
    /// The holder that produced this segment (its canonical position in the
    /// federated vector is the caller's ordering responsibility — see
    /// [`reconstruct_federated_public_inputs`]).
    pub holder: HolderId,
    /// The committed named-graph roots — exactly `k` words (`k` is the declared
    /// `CircuitId.k`). One issuer-signed commitment `C(G_i)` per committed graph.
    pub commitments: Vec<Word>,
    /// `pattern_is_const[3]`: whether each scan slot is a constant (`true`) or a
    /// variable (`false`). Encoded as `{0,1}` words.
    pub pattern_is_const: [bool; PATTERN_SLOTS],
    /// `pattern_const_enc[3]`: the encoded constant per slot; a variable slot
    /// carries the zero word (`[0u8; 32]`), exactly as the circuit declares.
    pub pattern_const_enc: [Word; PATTERN_SLOTS],
    /// The disclosed solution rows, row-major, `[subj, pred, obj]` encodings per
    /// row. At most [`declared_rows`](Self::declared_rows) actual rows; the layout
    /// zero-pads the remainder to `r`.
    pub rows: Vec<[Word; PATTERN_SLOTS]>,
    /// The declared `CircuitId.r` — the fixed row count the circuit pads to. The
    /// layout emits exactly `r` row triples (real rows then zero rows), so a
    /// wrong `r` yields a wrong-length vector that cannot byte-match the proof of
    /// the real member (this is the audit-#11 relabel guard, per source).
    pub declared_rows: usize,
    /// The disclosed row count (`u32`), bound as one word.
    pub row_count: u32,
    /// `attribution[k]` (audit #8): one `{0,1}` word per committed graph marking
    /// whether that graph contributed a row. Length MUST equal `commitments.len()`
    /// (`k`); a mismatch is a layout precondition violation, not a silent pad.
    pub attribution: Vec<bool>,
}

impl SourceSegment {
    /// The `k` (number of committed graphs) this segment declares.
    pub fn k(&self) -> usize {
        self.commitments.len()
    }

    /// The exact byte length this segment serialises to in the federated layout:
    /// `32 * (1 + k + 3 + 3 + 3r + 1 + k)` bytes
    /// (challenge + commitments + is_const + const_enc + padded rows + row_count
    /// + attribution). Useful for slicing a source out of the federated vector.
    pub fn encoded_len(&self) -> usize {
        32 * (1 + self.k() + PATTERN_SLOTS + PATTERN_SLOTS + PATTERN_SLOTS * self.declared_rows + 1 + self.k())
    }

    /// Validate the segment's shape against the single-source layout invariants
    /// BEFORE emitting any bytes (fail-closed): `attribution` is exactly `k`
    /// bits (audit #8 shape — the verifier's stage-1b `AttributionMalformed`
    /// gate, per source), and no more than `r` real rows are supplied.
    fn check_shape(&self) -> Result<(), MpcError> {
        if self.attribution.len() != self.k() {
            return Err(MpcError::Protocol(format!(
                "federated layout: holder {} attribution length {} != k {} (audit #8 shape)",
                self.holder,
                self.attribution.len(),
                self.k()
            )));
        }
        if self.rows.len() > self.declared_rows {
            return Err(MpcError::Protocol(format!(
                "federated layout: holder {} has {} rows > declared r {} (would overflow the padded layout)",
                self.holder,
                self.rows.len(),
                self.declared_rows
            )));
        }
        Ok(())
    }

    /// Serialise this segment's public inputs — **challenge FIRST** — EXACTLY as
    /// the single-source `reconstruct_public_inputs` would for a proof of this
    /// `scan` member under `challenge`. The returned bytes are byte-for-byte the
    /// source's slice of the federated layout AND of that source's own
    /// single-source proof `public_inputs` (the "byte-compared unchanged"
    /// invariant). Appends to `out`.
    fn encode(&self, challenge: &Word, out: &mut Vec<u8>) -> Result<(), MpcError> {
        self.check_shape()?;

        // Field 0 is always the verifier's shared fresh challenge N (every
        // member's first `pub`). Repeating it per segment is what makes each
        // segment byte-identical to the single-source proof's public inputs.
        out.extend_from_slice(challenge);
        // commitments[k].
        for c in &self.commitments {
            out.extend_from_slice(c);
        }
        // pattern_is_const[3] (bool -> {0,1}).
        for &b in &self.pattern_is_const {
            out.extend_from_slice(&uint_word(u64::from(b)));
        }
        // pattern_const_enc[3] (variable slots carry the zero word).
        for e in &self.pattern_const_enc {
            out.extend_from_slice(e);
        }
        // rows[r][3], row-major, zero-padded to the declared r.
        for j in 0..self.declared_rows {
            match self.rows.get(j) {
                Some(row) => {
                    for slot in row {
                        out.extend_from_slice(slot);
                    }
                }
                // Padding row: three zero words.
                None => out.extend_from_slice(&[0u8; 32 * PATTERN_SLOTS]),
            }
        }
        // row_count: u32.
        out.extend_from_slice(&uint_word(u64::from(self.row_count)));
        // attribution[k]: bool -> {0,1}, one word per committed graph (audit #8).
        // Length is exactly k (checked in `check_shape`), so this never pads.
        for b in &self.attribution {
            out.extend_from_slice(&uint_word(u64::from(*b)));
        }
        Ok(())
    }
}

/// Domain-separation tag for the out-of-circuit freshness/replay binding
/// transcript. Distinct from any in-circuit / commitment domain so the
/// transcript can never be confused with a signed commitment message
/// (domain-separation discipline — cf. the ZK estate's salt/domain separation).
pub const FRESHNESS_DOMAIN: &[u8] = b"sparq-mpc/m4v1/freshness-binding/v1";

/// The explicit **out-of-circuit freshness/replay binding** (RQ1 audit #4).
///
/// In-circuit, the verifier's fresh challenge `N` is a circuit public input, so
/// the proof is automatically bound to `N`. In the M4-v1 verifier-side
/// attestation interim the issuer signature is verified OUTSIDE the circuit
/// (Artemis commit-and-prove anchor), so `N` is not automatically covered by the
/// signature. This type makes the binding explicit: it derives the message the
/// out-of-circuit signature check must cover, folding in the verifier's fresh
/// nonce (`N`, also field 0 of every [`SourceSegment`]) and a session id.
///
/// Replay resistance is a *composition* property: the verifier reconstructs the
/// federated public inputs and this transcript with its OWN fresh `N`, then
/// requires the out-of-circuit signature to verify against
/// [`out_of_circuit_message`](Self::out_of_circuit_message). A proof replayed
/// from an earlier session carries an old `N`, so both the per-segment byte
/// compare (audit #1) and this transcript differ, and the signature made over
/// the old message will not verify under the fresh `N`. **This module provides
/// the binding; the signature *verification* it feeds is gated on ZK-remediation
/// #3 and stays a `NotYetImplemented` stub in [`crate::proof`].**
#[derive(Debug, Clone)]
pub struct FreshnessBinding {
    challenge: Word,
    session_id: Vec<u8>,
}

impl FreshnessBinding {
    /// `challenge` is the verifier's FRESH per-session nonce `N` (also field 0 of
    /// every source segment); `session_id` scopes the attestation to one
    /// verification session / transaction (an empty id is permitted — the
    /// binding still folds in `N` and the full federated statement).
    pub fn new(challenge: Word, session_id: impl Into<Vec<u8>>) -> Self {
        FreshnessBinding {
            challenge,
            session_id: session_id.into(),
        }
    }

    /// The shared verifier challenge `N` bound as field 0 of every segment.
    pub fn challenge(&self) -> &Word {
        &self.challenge
    }

    /// The session / transaction id the attestation is scoped to.
    pub fn session_id(&self) -> &[u8] {
        &self.session_id
    }

    /// The message the OUT-OF-CIRCUIT issuer-signature check MUST cover so a
    /// `(federated statement, proof, signature)` triple cannot be (a) replayed
    /// under a different verifier nonce, or (b) lifted onto a different federated
    /// statement: a domain-separated SHA-512 digest (truncated to 32 bytes) over
    ///
    /// `FRESHNESS_DOMAIN ‖ len(session_id) ‖ session_id ‖ challenge ‖ federated_public_inputs`.
    ///
    /// Everything before the variable-length `federated_public_inputs` is either
    /// fixed-width (the domain tag is a fixed constant, `challenge` is 32 bytes)
    /// or explicitly length-prefixed (`session_id`), so the encoding is
    /// unambiguous. `federated_public_inputs` is the output of
    /// [`reconstruct_federated_public_inputs`] for this same binding.
    pub fn out_of_circuit_message(&self, federated_public_inputs: &[u8]) -> [u8; 32] {
        let mut h = Sha512::new();
        h.update(FRESHNESS_DOMAIN);
        h.update((self.session_id.len() as u64).to_be_bytes());
        h.update(&self.session_id);
        h.update(self.challenge);
        h.update(federated_public_inputs);
        let digest = h.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest[..32]);
        out
    }
}

/// Reconstruct the **federated** public-input byte vector spanning ALL holders'
/// [`SourceSegment`]s, in the given canonical `sources` order, each segment
/// byte-identical to the single-source layout under the ONE shared verifier
/// challenge `N` carried by `binding`.
///
/// **Ordering is the caller's responsibility and is part of the bound
/// statement.** The verifier and provers must agree a canonical holder order up
/// front (e.g. sorted by [`HolderId`]); a different order yields different bytes
/// and fails the byte compare. This function binds the order it is given — it
/// does NOT sort — so the choice is explicit and auditable rather than an
/// implicit surprise. It also rejects an empty federation (there is nothing to
/// attest) and any segment whose shape violates the single-source invariants.
pub fn reconstruct_federated_public_inputs(
    binding: &FreshnessBinding,
    sources: &[SourceSegment],
) -> Result<Vec<u8>, MpcError> {
    if sources.is_empty() {
        return Err(MpcError::Protocol(
            "federated public-input layout needs >= 1 source (empty federation attests nothing)"
                .into(),
        ));
    }
    let mut out = Vec::with_capacity(sources.iter().map(SourceSegment::encoded_len).sum());
    for s in sources {
        s.encode(binding.challenge(), &mut out)?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    //! sq-34ml prerequisite tests: the federated layout is byte-for-byte the
    //! single-source layout per source (audit #1/#8), ordering is bound, shapes
    //! fail closed, and the freshness transcript actually binds N + the statement
    //! (so a replay under a fresh nonce cannot reuse an old signature).
    use super::*;

    fn word(b: u8) -> Word {
        [b; 32]
    }

    fn seg(holder: &str, commit: u8, attribution: Vec<bool>, r: usize) -> SourceSegment {
        SourceSegment {
            holder: HolderId::new(holder),
            commitments: vec![word(commit)],
            pattern_is_const: [true, false, true],
            pattern_const_enc: [word(0x01), [0u8; 32], word(0x02)],
            rows: vec![[word(0x03), word(0x04), word(0x05)]],
            declared_rows: r,
            row_count: 1,
            attribution,
        }
    }

    /// The exact documented `scan_k{k}_n{n}_r{r}` layout, byte-for-byte, for a
    /// k=1, r=2 segment with one real row + one pad row — pinning that this
    /// crate's mirror equals `reconstruct_public_inputs`'s emitted bytes (the
    /// "byte-compared unchanged" invariant; drift tripwire).
    #[test]
    fn scan_segment_matches_single_source_layout() {
        let binding = FreshnessBinding::new(word(0x11), b"session-1".to_vec());
        let s = seg("alice", 0xAA, vec![true], 2);
        let bytes = reconstruct_federated_public_inputs(&binding, std::slice::from_ref(&s)).unwrap();

        // Build the expected vector independently from the documented order:
        // challenge, commitments[1], is_const[3], const_enc[3], rows[2][3]
        // (1 real + 1 pad), row_count, attribution[1].
        let mut expect: Vec<u8> = Vec::new();
        expect.extend_from_slice(&word(0x11)); // challenge N
        expect.extend_from_slice(&word(0xAA)); // commitments[0]
        expect.extend_from_slice(&uint_word(1)); // is_const[0] = true
        expect.extend_from_slice(&uint_word(0)); // is_const[1] = false
        expect.extend_from_slice(&uint_word(1)); // is_const[2] = true
        expect.extend_from_slice(&word(0x01)); // const_enc[0]
        expect.extend_from_slice(&[0u8; 32]); // const_enc[1] (variable -> zero)
        expect.extend_from_slice(&word(0x02)); // const_enc[2]
        expect.extend_from_slice(&word(0x03)); // row0.subj
        expect.extend_from_slice(&word(0x04)); // row0.pred
        expect.extend_from_slice(&word(0x05)); // row0.obj
        expect.extend_from_slice(&[0u8; 96]); // row1 pad (three zero words)
        expect.extend_from_slice(&uint_word(1)); // row_count
        expect.extend_from_slice(&uint_word(1)); // attribution[0] = true

        assert_eq!(bytes, expect, "federated segment must match documented layout");
        assert_eq!(bytes.len(), s.encoded_len());
        // 32 * (1 + 1 + 3 + 3 + 3*2 + 1 + 1) = 32 * 16 = 512 bytes.
        assert_eq!(bytes.len(), 512);
    }

    /// A source's slice of a multi-source vector equals that source's OWN
    /// single-source reconstruction — the federation adds ordering + the shared
    /// nonce and nothing else (byte-compared unchanged, per source).
    #[test]
    fn federated_segments_are_the_concatenation_of_single_source_segments() {
        let binding = FreshnessBinding::new(word(0x11), b"s".to_vec());
        let a = seg("alice", 0xAA, vec![true], 2);
        let b = seg("bob", 0xBB, vec![false], 2);

        let fed = reconstruct_federated_public_inputs(&binding, &[a.clone(), b.clone()]).unwrap();
        let only_a = reconstruct_federated_public_inputs(&binding, std::slice::from_ref(&a)).unwrap();
        let only_b = reconstruct_federated_public_inputs(&binding, std::slice::from_ref(&b)).unwrap();

        assert_eq!(fed.len(), a.encoded_len() + b.encoded_len());
        assert_eq!(&fed[..a.encoded_len()], &only_a[..], "alice slice unchanged");
        assert_eq!(&fed[a.encoded_len()..], &only_b[..], "bob slice unchanged");
    }

    /// Holder order is BOUND: reordering the sources changes the bytes (so a
    /// canonical order must be agreed up front — the function never silently
    /// sorts).
    #[test]
    fn holder_order_is_bound() {
        let binding = FreshnessBinding::new(word(0x11), b"s".to_vec());
        let a = seg("alice", 0xAA, vec![true], 1);
        let b = seg("bob", 0xBB, vec![false], 1);
        let ab = reconstruct_federated_public_inputs(&binding, &[a.clone(), b.clone()]).unwrap();
        let ba = reconstruct_federated_public_inputs(&binding, &[b, a]).unwrap();
        assert_ne!(ab, ba, "a different holder order must produce different bytes");
    }

    #[test]
    fn empty_federation_is_refused() {
        let binding = FreshnessBinding::new(word(0x11), vec![]);
        let err = reconstruct_federated_public_inputs(&binding, &[]).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(_)));
    }

    /// audit-#8 shape: attribution length must equal k — a mismatch fails closed
    /// rather than silently padding/truncating.
    #[test]
    fn attribution_length_mismatch_fails_closed() {
        let binding = FreshnessBinding::new(word(0x11), vec![]);
        // k = 1 commitment but 2 attribution bits.
        let bad = seg("alice", 0xAA, vec![true, false], 1);
        let err = reconstruct_federated_public_inputs(&binding, &[bad]).unwrap_err();
        match err {
            MpcError::Protocol(m) => assert!(m.contains("attribution"), "got: {m}"),
            other => panic!("expected Protocol, got {other:?}"),
        }
    }

    /// Too many real rows for the declared r overflows the padded layout — refused.
    #[test]
    fn rows_exceeding_declared_r_fails_closed() {
        let binding = FreshnessBinding::new(word(0x11), vec![]);
        let mut bad = seg("alice", 0xAA, vec![true], 1);
        bad.rows.push([word(0x06), word(0x07), word(0x08)]); // 2 rows, r = 1
        let err = reconstruct_federated_public_inputs(&binding, &[bad]).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(_)));
    }

    /// The freshness transcript binds the verifier's fresh nonce N: a proof
    /// replayed under a DIFFERENT nonce yields a different out-of-circuit message,
    /// so an issuer signature made over the old message will not verify.
    #[test]
    fn freshness_binding_binds_the_nonce() {
        let seg0 = seg("alice", 0xAA, vec![true], 1);
        let b_old = FreshnessBinding::new(word(0x11), b"sess".to_vec());
        let b_new = FreshnessBinding::new(word(0x22), b"sess".to_vec());

        let pi_old = reconstruct_federated_public_inputs(&b_old, std::slice::from_ref(&seg0)).unwrap();
        let pi_new = reconstruct_federated_public_inputs(&b_new, std::slice::from_ref(&seg0)).unwrap();

        // The public inputs already differ (field 0 = N), and so does the bound
        // out-of-circuit message the signature must cover.
        assert_ne!(pi_old, pi_new);
        assert_ne!(
            b_old.out_of_circuit_message(&pi_old),
            b_new.out_of_circuit_message(&pi_new),
            "a fresh nonce must change the out-of-circuit signed message (anti-replay)"
        );
    }

    /// The transcript also binds the federated statement + the session id, so a
    /// signature cannot be lifted onto a different statement or session.
    #[test]
    fn freshness_binding_binds_statement_and_session() {
        let a = seg("alice", 0xAA, vec![true], 1);
        let b = seg("bob", 0xBB, vec![false], 1);
        let binding = FreshnessBinding::new(word(0x11), b"sess-A".to_vec());
        let other_session = FreshnessBinding::new(word(0x11), b"sess-B".to_vec());

        let pi_a = reconstruct_federated_public_inputs(&binding, std::slice::from_ref(&a)).unwrap();
        let pi_b = reconstruct_federated_public_inputs(&binding, std::slice::from_ref(&b)).unwrap();

        // Different statement, same binding -> different message.
        assert_ne!(
            binding.out_of_circuit_message(&pi_a),
            binding.out_of_circuit_message(&pi_b),
            "a different federated statement must change the bound message"
        );
        // Same statement, different session id -> different message.
        assert_ne!(
            binding.out_of_circuit_message(&pi_a),
            other_session.out_of_circuit_message(&pi_a),
            "a different session id must change the bound message"
        );
    }

    /// `uint_word` matches the verifier's right-aligned BE `push_uint`.
    #[test]
    fn uint_word_is_right_aligned_big_endian() {
        assert_eq!(uint_word(0), [0u8; 32]);
        let one = uint_word(1);
        assert_eq!(one[31], 1);
        assert!(one[..31].iter().all(|&b| b == 0));
        let big = uint_word(0x0102_0304_0506_0708);
        assert_eq!(&big[24..], &[1, 2, 3, 4, 5, 6, 7, 8]);
    }
}
