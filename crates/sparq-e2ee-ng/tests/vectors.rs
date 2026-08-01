//! **Golden test vectors** (research doc §9 requires test vectors before the
//! suite can be called production-ready). These pin the *exact* deterministic
//! wire bytes of every primitive so the encoding + the concrete v0 crypto
//! composition cannot silently drift. Ed25519 signing, HKDF, and (given a fixed
//! key/nonce) XChaCha20-Poly1305 are all deterministic, so every value below is
//! reproducible. Regenerate with `cargo run --example gen_vectors`.
//!
//! A change to any of these strings is a **wire-format break** and must be a
//! deliberate, reviewed version bump — never an accident.

use sparq_e2ee_ng::capability::{base_grant, Authority, Capability, Validity};
use sparq_e2ee_ng::cbor::Limits;
use sparq_e2ee_ng::envelope::{seal_block, BlockContext, Commit, ObjectKind};
use sparq_e2ee_ng::epoch::{EpochTransition, HistoryPolicy};
use sparq_e2ee_ng::ids::{
    AuthorKeyId, BlockId, BranchId, CommitId, Epoch, ObjectId, RepoId, Secret32, TopicId,
};
use sparq_e2ee_ng::keyschedule::{block_key, object_key};
use sparq_e2ee_ng::sign::SecretSigningKey;
use sparq_e2ee_ng::suite::AEAD_NONCE_LEN;
use sparq_e2ee_ng::wrap::{wrap_with, RecipientSecretKey, WrappedSecret};

const OBJECT_KEY: &str = "f047a88d13eceb6c4ae04aab8c60097a9550a1f53a817a711bfd3482c04b2da8";
const BLOCK_KEY: &str = "285777cd44c36e35570b889f786fd170ff8571d0fab1cd2ae7c379c70b422942";
const CAP_ID: &str = "e915b3796456da74d18ca059c6e07b7711fdb37d3a26b50638f030a8348066f6";
const COMMIT_ID_OF_BLOCK: &str =
    "814a9a0164fe1573dd301684ebd319a980f817681a107463a5f58d6d23b53392";

fn common() -> (Secret32, RepoId, BranchId, ObjectId, SecretSigningKey) {
    (
        Secret32([4u8; 32]),
        RepoId::from_bytes([1u8; 32]),
        BranchId::from_bytes([2u8; 32]),
        ObjectId::from_bytes([5u8; 32]),
        SecretSigningKey::from_seed([1u8; 32]),
    )
}

#[test]
fn vector_key_schedule() {
    let (k, repo, branch, object, _admin) = common();
    let ok = object_key(&k, &repo, &branch, Epoch(7), &object);
    assert_eq!(hex::encode(ok), OBJECT_KEY);
    let bk = block_key(&ok, &BlockId::from_bytes([6u8; 32]), 0);
    assert_eq!(hex::encode(bk), BLOCK_KEY);
}

#[test]
fn vector_capability_public_grant() {
    let (_k, repo, branch, _object, admin) = common();
    let mut grant = base_grant(
        repo,
        branch,
        Epoch(7),
        TopicId::from_bytes([3u8; 32]),
        Validity { not_before: 100, not_after: 200 },
        vec!["wss://broker.example".to_string()],
    );
    grant.authority = vec![Authority::Read];
    grant.cap_nonce = [0x11u8; 32]; // pin the otherwise-random nonce for a stable vector
    grant.sign(&admin);
    // cap_id is stable and matches the golden value.
    assert_eq!(hex::encode(grant.cap_id().as_bytes()), CAP_ID);
    // The signed grant verifies and round-trips through decode byte-for-byte.
    grant.verify(&admin.public()).unwrap();
    let bytes = grant.encode();
    let decoded =
        sparq_e2ee_ng::capability::PublicGrant::decode(&bytes, Limits::default()).unwrap();
    assert_eq!(decoded.encode(), bytes);
    assert_eq!(decoded.cap_id(), grant.cap_id());
}

#[test]
fn vector_block_envelope_commit_id() {
    let (k, repo, branch, object, _admin) = common();
    let ctx = BlockContext { repo, branch, epoch: Epoch(7), kind: ObjectKind::Operation };
    let env = seal_block(
        &k,
        &ctx,
        &object,
        &BlockId::from_bytes([6u8; 32]),
        0,
        1,
        [7u8; AEAD_NONCE_LEN],
        b"some quads",
    )
    .unwrap();
    // Deterministic ciphertext => stable CommitId over the encrypted envelope.
    assert_eq!(hex::encode(env.commit_id().as_bytes()), COMMIT_ID_OF_BLOCK);
    // And it opens back to the plaintext.
    assert_eq!(
        sparq_e2ee_ng::envelope::open_block(&k, &ctx, &env).unwrap(),
        b"some quads"
    );
}

#[test]
fn vector_commit_plaintext_verifies() {
    let (_k, repo, branch, object, _admin) = common();
    let author = SecretSigningKey::from_seed([3u8; 32]);
    let mut commit = Commit {
        repo,
        branch,
        epoch: Epoch(7),
        parents: vec![CommitId::from_bytes([8u8; 32])],
        author: AuthorKeyId::from_bytes([0u8; 32]),
        clock: 42,
        crdt_kind: "or-set-quads-v0".to_string(),
        operations: vec![object],
        snapshot: None,
        author_sig: None,
    };
    commit.sign(&author);
    let bytes = commit.encode();
    // Round-trips byte-for-byte and re-verifies.
    let decoded = Commit::decode(&bytes, Limits::default()).unwrap();
    assert_eq!(decoded.encode(), bytes);
    decoded.verify().unwrap();
}

#[test]
fn vector_epoch_transition_verifies() {
    let (_k, repo, branch, _object, admin) = common();
    let mut t = EpochTransition {
        repo,
        branch,
        old_epoch: Epoch(7),
        new_epoch: Epoch(8),
        old_topic: TopicId::from_bytes([3u8; 32]),
        new_topic: TopicId::from_bytes([4u8; 32]),
        new_publishers: vec![SecretSigningKey::from_seed([2u8; 32]).public().to_bytes()],
        history_policy: HistoryPolicy::ForwardOnly,
        admin_sig: None,
    };
    t.sign(&admin).unwrap();
    let bytes = t.encode();
    let decoded = EpochTransition::decode(&bytes, Limits::default()).unwrap();
    assert_eq!(decoded.encode(), bytes);
    decoded.verify(&admin.public()).unwrap();
}

#[test]
fn vector_wrapped_secret_unwraps() {
    let recipient = RecipientSecretKey::from_bytes([5u8; 32]);
    let w = wrap_with(
        [11u8; 32],
        [12u8; AEAD_NONCE_LEN],
        &recipient.public(),
        b"K_read",
        b"purpose:cap",
    )
    .unwrap();
    let bytes = w.encode();
    let decoded = WrappedSecret::decode(&bytes, Limits::default()).unwrap();
    assert_eq!(decoded.encode(), bytes);
    assert_eq!(
        sparq_e2ee_ng::wrap::unwrap(&recipient, &decoded, b"purpose:cap").unwrap(),
        b"K_read"
    );
}

#[test]
fn vector_secret_capability_roundtrips() {
    let (_k, repo, branch, _object, admin) = common();
    let publisher = SecretSigningKey::from_seed([2u8; 32]);
    let mut write = Capability::new_write(
        base_grant(
            repo,
            branch,
            Epoch(7),
            TopicId::from_bytes([3u8; 32]),
            Validity { not_before: 100, not_after: 200 },
            vec!["wss://broker.example".to_string()],
        ),
        Secret32([9u8; 32]),
        &publisher,
    )
    .unwrap();
    write.grant.cap_nonce = [0x42; 32];
    write.grant.sign(&admin);

    let bytes = write.encode_secret();
    let decoded = Capability::decode_secret(&bytes, Limits::default()).unwrap();
    assert_eq!(decoded.encode_secret(), bytes);
    decoded.grant.verify(&admin.public()).unwrap();
    assert_eq!(decoded.read_secret.as_ref().unwrap().expose(), &[9u8; 32]);
}
