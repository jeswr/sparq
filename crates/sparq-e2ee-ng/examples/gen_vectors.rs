//! Throwaway generator that prints the deterministic golden vectors baked into
//! `tests/vectors.rs`. Run: `cargo run -p sparq-e2ee-ng --example gen_vectors`.

use sparq_e2ee_ng::capability::{base_grant, Authority, Capability, Validity};
use sparq_e2ee_ng::envelope::{seal_block, BlockContext, Commit, ObjectKind};
use sparq_e2ee_ng::epoch::{EpochTransition, HistoryPolicy};
use sparq_e2ee_ng::ids::{
    AuthorKeyId, BlockId, BranchId, CommitId, Epoch, ObjectId, RepoId, Secret32, TopicId,
};
use sparq_e2ee_ng::keyschedule::{block_key, object_key};
use sparq_e2ee_ng::sign::SecretSigningKey;
use sparq_e2ee_ng::suite::AEAD_NONCE_LEN;
use sparq_e2ee_ng::wrap::{wrap_with, RecipientSecretKey};

fn main() {
    // key schedule
    let k = Secret32([4u8; 32]);
    let repo = RepoId::from_bytes([1u8; 32]);
    let branch = BranchId::from_bytes([2u8; 32]);
    let object = ObjectId::from_bytes([5u8; 32]);
    let ok = object_key(&k, &repo, &branch, Epoch(7), &object);
    let bk = block_key(&ok, &BlockId::from_bytes([6u8; 32]), 0);
    println!("OBJECT_KEY = {}", hex::encode(ok));
    println!("BLOCK_KEY  = {}", hex::encode(bk));

    // capability public grant (admin-signed; ed25519 is deterministic)
    let admin = SecretSigningKey::from_seed([1u8; 32]);
    let mut grant = base_grant(
        repo,
        branch,
        Epoch(7),
        TopicId::from_bytes([3u8; 32]),
        Validity {
            not_before: 100,
            not_after: 200,
        },
        vec!["wss://broker.example".to_string()],
    );
    grant.authority = vec![Authority::Read];
    grant.cap_nonce = [0x11u8; 32]; // pin the otherwise-random nonce for a stable vector
    grant.sign(&admin);
    println!("GRANT_PUBLIC = {}", hex::encode(grant.encode()));
    println!("CAP_ID = {}", hex::encode(grant.cap_id().as_bytes()));

    // block envelope (fixed nonce + block id => deterministic ciphertext)
    let ctx = BlockContext {
        repo,
        branch,
        epoch: Epoch(7),
        kind: ObjectKind::Operation,
    };
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
    println!("BLOCK_ENVELOPE = {}", hex::encode(env.encode()));
    println!(
        "COMMIT_ID_OF_BLOCK = {}",
        hex::encode(env.commit_id().as_bytes())
    );

    // commit plaintext (fixed author seed)
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
    println!("COMMIT_PLAINTEXT = {}", hex::encode(commit.encode()));

    // epoch transition (fixed admin seed)
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
    println!("EPOCH_TRANSITION = {}", hex::encode(t.encode()));

    // wrapped secret (fixed ephemeral seed + nonce)
    let recipient = RecipientSecretKey::from_bytes([5u8; 32]);
    let w = wrap_with(
        [11u8; 32],
        [12u8; AEAD_NONCE_LEN],
        &recipient.public(),
        b"K_read",
        b"purpose:cap",
    )
    .unwrap();
    println!("WRAPPED_SECRET = {}", hex::encode(w.encode()));

    // secret-bearing capability
    let publisher = SecretSigningKey::from_seed([2u8; 32]);
    let mut write = Capability::new_write(
        base_grant(
            repo,
            branch,
            Epoch(7),
            TopicId::from_bytes([3u8; 32]),
            Validity {
                not_before: 100,
                not_after: 200,
            },
            vec!["wss://broker.example".to_string()],
        ),
        Secret32([9u8; 32]),
        &publisher,
    )
    .unwrap();
    write.grant.cap_nonce = [0x42; 32];
    write.grant.sign(&admin);
    println!("CAP_SECRET = {}", hex::encode(write.encode_secret()));
}
