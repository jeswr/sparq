//! Behavioural tests for the E2EE-NG **client/broker message** codec (§8.4).
//!
//! The load-bearing properties under test:
//!
//! 1. **Round-trip + determinism** — every message re-encodes to the same bytes
//!    and decodes back to an equal value (the wire format is a contract).
//! 2. **Fail-closed decoding** — non-canonical CBOR, trailing bytes, an unknown
//!    kind, an unknown mandatory field, a wrong version, and an over-limit list
//!    are all rejected, never best-effort recovered.
//! 3. **Authorization objects verify and tamper-fail** — admission grants,
//!    publish announcements, and epoch advances.
//! 4. **The disclosure ledger holds structurally** — a full session transcript
//!    contains no read secret and no query text.

use sparq_e2ee_ng::broker_protocol::*;
use sparq_e2ee_ng::capability::Validity;
use sparq_e2ee_ng::cbor::Limits;
use sparq_e2ee_ng::envelope::{seal_block_random, BlockContext, ObjectKind};
use sparq_e2ee_ng::ids::{
    BlockId, BranchId, CommitId, Epoch, ObjectId, OverlayId, PeerId, RepoId, Secret32, TopicId,
};
use sparq_e2ee_ng::sign::SecretSigningKey;
use sparq_e2ee_ng::suite::SUITE_V0;

fn limits() -> Limits {
    protocol_limits(1 << 20, 1024)
}

fn grant(
    admin: &SecretSigningKey,
    publisher: Option<&SecretSigningKey>,
    topic: TopicId,
) -> AdmissionGrant {
    let mut g = AdmissionGrant {
        topic,
        epoch: Epoch(3),
        suite: SUITE_V0.to_string(),
        admin_pub: admin.public().to_bytes(),
        publisher_pub: publisher.map(|p| p.public().to_bytes()),
        validity: Validity {
            not_before: 100,
            not_after: 900,
        },
        admin_sig: None,
    };
    g.sign(admin).expect("sign grant");
    g
}

fn sample_envelope() -> sparq_e2ee_ng::envelope::BlockEnvelope {
    let ctx = BlockContext {
        repo: RepoId::random(),
        branch: BranchId::random(),
        epoch: Epoch(3),
        kind: ObjectKind::Commit,
    };
    seal_block_random(&Secret32::random(), &ctx, &ObjectId::random(), 0, 1, b"payload")
        .expect("seal")
}

fn all_requests() -> Vec<Request> {
    let admin = SecretSigningKey::generate();
    let publisher = SecretSigningKey::generate();
    let topic = TopicId::random();
    let mut publish = PublishEvent {
        topic,
        epoch: Epoch(3),
        commit_id: CommitId::random(),
        root_block_id: BlockId::random(),
        publisher_key_id: [0u8; 32],
        parents: vec![CommitId::random()],
        signature: None,
    };
    publish.sign(&publisher);
    let mut advance = EpochAdvance {
        old_topic: topic,
        new_topic: TopicId::random(),
        old_epoch: Epoch(3),
        new_epoch: Epoch(4),
        transition_commit: CommitId::random(),
        new_publishers: vec![publisher.public().to_bytes()],
        admin_pub: admin.public().to_bytes(),
        admin_sig: None,
    };
    advance.sign(&admin).expect("sign advance");
    let mut bloom = BloomHint::new(64, 3);
    bloom.insert(&BlockId::random());

    vec![
        Request::Hello(hello_v0(1 << 20)),
        Request::OpenRepo(OpenRepo {
            overlay: OverlayId::random(),
            topic,
            epoch: Epoch(3),
            peer: PeerId::random(),
            auth: Some(grant(&admin, Some(&publisher), topic)),
        }),
        Request::OpenRepo(OpenRepo {
            overlay: OverlayId::random(),
            topic,
            epoch: Epoch(3),
            peer: PeerId::random(),
            auth: None,
        }),
        Request::PinRepo(PinRepo { topic, pin: true }),
        Request::RepoPinStatus { topic },
        Request::TopicSub(TopicSub {
            topic,
            epoch: Epoch(3),
            after_cursor: Some(17),
        }),
        Request::TopicUnsub { topic },
        Request::TopicSyncReq(TopicSyncReq {
            topic,
            epoch: Epoch(3),
            known_heads: vec![CommitId::random(), CommitId::random()],
            target_heads: Some(vec![CommitId::random()]),
            known_commits: Some(bloom),
            page_after: Some(BlockId::random()),
        }),
        Request::BlocksExist {
            ids: vec![BlockId::random()],
        },
        Request::BlocksGet {
            ids: vec![BlockId::random(), BlockId::random()],
        },
        Request::BlocksPut {
            envelopes: vec![sample_envelope(), sample_envelope()],
        },
        Request::CommitGet {
            commit_ids: vec![CommitId::random()],
        },
        Request::PublishEvent(publish),
        Request::EpochAdvance(advance),
    ]
}

fn all_responses() -> Vec<Response> {
    let publisher = SecretSigningKey::generate();
    let admin = SecretSigningKey::generate();
    let topic = TopicId::random();
    let mut publish = PublishEvent {
        topic,
        epoch: Epoch(1),
        commit_id: CommitId::random(),
        root_block_id: BlockId::random(),
        publisher_key_id: [0u8; 32],
        parents: vec![],
        signature: None,
    };
    publish.sign(&publisher);
    let mut advance = EpochAdvance {
        old_topic: topic,
        new_topic: TopicId::random(),
        old_epoch: Epoch(1),
        new_epoch: Epoch(2),
        transition_commit: CommitId::random(),
        new_publishers: vec![],
        admin_pub: admin.public().to_bytes(),
        admin_sig: None,
    };
    advance.sign(&admin).expect("sign advance");

    vec![
        Response::HelloAck(HelloAck {
            version: PROTOCOL_V0,
            suite: SUITE_V0.to_string(),
            header_mode: HeaderMode::Opaque,
            limits: WireLimits::default(),
            padding_classes: vec![256, 1024],
            retention: RetentionPolicy::default(),
        }),
        Response::Ok,
        Response::PinStatus(PinStatus {
            topic,
            pinned: true,
            blocks: 7,
            bytes: 4096,
            retention: RetentionPolicy::default(),
        }),
        Response::SyncResp(TopicSyncResp {
            advertised_heads: vec![CommitId::random()],
            missing_block_ids: vec![BlockId::random(), BlockId::random()],
            cursor: 42,
            more: true,
        }),
        Response::ExistBits {
            bits: vec![0b1010_1010],
            count: 8,
        },
        Response::Blocks {
            found: vec![sample_envelope()],
            missing: vec![BlockId::random()],
        },
        Response::Commits {
            found: vec![sample_envelope()],
            missing: vec![CommitId::random()],
        },
        Response::Stored {
            stored: 3,
            duplicate: 1,
        },
        Response::Published {
            commit_id: CommitId::random(),
            cursor: 9,
        },
        Response::Event(Event {
            announcement: publish,
            cursor: 9,
        }),
        Response::EpochAdvanced(advance),
        Response::Error(BrokerError::new(ErrorCode::NotAdmitted, "not admitted")),
    ]
}

#[test]
fn every_request_round_trips_and_encodes_deterministically() {
    for (i, req) in all_requests().into_iter().enumerate() {
        let bytes = req.encode(i as u64 + 1);
        assert_eq!(bytes, req.encode(i as u64 + 1), "encoding is not deterministic");
        let (id, back) = Request::decode(&bytes, limits()).expect("decode request");
        assert_eq!(id, i as u64 + 1);
        assert_eq!(back, req, "round-trip changed the message");
        assert_eq!(back.encode(id), bytes, "re-encode is not byte-identical");
    }
}

#[test]
fn every_response_round_trips_and_encodes_deterministically() {
    for (i, resp) in all_responses().into_iter().enumerate() {
        let bytes = resp.encode(i as u64);
        assert_eq!(bytes, resp.encode(i as u64));
        let (id, back) = Response::decode(&bytes, limits()).expect("decode response");
        assert_eq!(id, i as u64);
        assert_eq!(back, resp);
        assert_eq!(back.encode(id), bytes);
    }
}

#[test]
fn decoder_rejects_trailing_bytes() {
    let mut bytes = Request::Hello(hello_v0(1024)).encode(1);
    bytes.push(0);
    assert!(Request::decode(&bytes, limits()).is_err());
}

#[test]
fn decoder_rejects_unknown_kind() {
    // Kind 99 is not allocated in either direction.
    let bytes = Request::RepoPinStatus {
        topic: TopicId::random(),
    }
    .encode(1);
    let mut tampered = bytes.clone();
    let kind_pos = tampered
        .windows(2)
        .position(|w| w == [0x03, 0x04])
        .expect("frame carries key 3 -> kind 4");
    tampered[kind_pos + 1] = 0x0f; // kind 15: unallocated
    assert!(Request::decode(&tampered, limits()).is_err());
}

#[test]
fn decoder_rejects_wrong_frame_version() {
    let bytes = Request::Hello(hello_v0(1024)).encode(1);
    // Frame key 1 is the version; its value is the immediately following byte.
    let mut tampered = bytes.clone();
    let pos = tampered
        .iter()
        .position(|b| *b == 0x01)
        .expect("frame carries key 1");
    tampered[pos + 1] = 0x01; // version 1 is not implemented
    assert!(Request::decode(&tampered, limits()).is_err());
}

#[test]
fn decoder_rejects_unknown_mandatory_field() {
    use sparq_e2ee_ng::cbor::{enc_bytes, enc_map, enc_uint};
    let topic = TopicId::random();
    let body = enc_map(vec![(1, enc_bytes(topic.as_bytes()))]);
    // Exactly the canonical frame the encoder produces...
    let good = enc_map(vec![
        (1, enc_uint(0)),
        (2, enc_uint(7)),
        (3, enc_uint(4)),
        (4, body.clone()),
    ]);
    assert_eq!(
        Request::decode(&good, limits()).expect("hand-built frame decodes"),
        (7, Request::RepoPinStatus { topic })
    );
    // ...plus one unallocated POSITIVE key. Positive keys are mandatory fields
    // (§8.1), so an unknown one fails closed rather than being ignored.
    let extended = enc_map(vec![
        (1, enc_uint(0)),
        (2, enc_uint(7)),
        (3, enc_uint(4)),
        (4, body),
        (9, enc_uint(0)),
    ]);
    assert!(Request::decode(&extended, limits()).is_err());
}

#[test]
fn decoder_rejects_over_limit_identifier_list() {
    let req = Request::BlocksGet {
        ids: (0..40).map(|_| BlockId::random()).collect(),
    };
    let bytes = req.encode(1);
    let tight = protocol_limits(1 << 20, 8);
    assert!(Request::decode(&bytes, tight).is_err());
    assert!(Request::decode(&bytes, limits()).is_ok());
}

#[test]
fn admission_grant_verifies_and_tamper_fails() {
    let admin = SecretSigningKey::generate();
    let publisher = SecretSigningKey::generate();
    let topic = TopicId::random();
    let g = grant(&admin, Some(&publisher), topic);
    g.verify_self().expect("valid grant verifies");
    assert!(g.is_valid_at(500));
    assert!(!g.is_valid_at(99));
    assert!(!g.is_valid_at(901));

    // Flipping any signed field invalidates the signature.
    let mut tampered = g.clone();
    tampered.epoch = Epoch(4);
    assert!(tampered.verify_self().is_err());

    let mut swapped = g.clone();
    swapped.publisher_pub = Some(SecretSigningKey::generate().public().to_bytes());
    assert!(swapped.verify_self().is_err());
}

#[test]
fn admission_grant_cannot_be_signed_by_a_key_it_does_not_declare() {
    let admin = SecretSigningKey::generate();
    let impostor = SecretSigningKey::generate();
    let mut g = AdmissionGrant {
        topic: TopicId::random(),
        epoch: Epoch(0),
        suite: SUITE_V0.to_string(),
        admin_pub: admin.public().to_bytes(),
        publisher_pub: None,
        validity: Validity {
            not_before: 0,
            not_after: u64::MAX,
        },
        admin_sig: None,
    };
    assert!(g.sign(&impostor).is_err());
}

#[test]
fn publish_announcement_verifies_and_tamper_fails() {
    let publisher = SecretSigningKey::generate();
    let mut p = PublishEvent {
        topic: TopicId::random(),
        epoch: Epoch(0),
        commit_id: CommitId::random(),
        root_block_id: BlockId::random(),
        publisher_key_id: [0u8; 32],
        parents: vec![],
        signature: None,
    };
    p.sign(&publisher);
    p.verify().expect("valid announcement verifies");
    assert_eq!(p.publisher_key_id, publisher.public().to_bytes());

    let mut moved = p.clone();
    moved.root_block_id = BlockId::random();
    assert!(moved.verify().is_err(), "root block swap must not verify");

    let mut replayed = p.clone();
    replayed.topic = TopicId::random();
    assert!(replayed.verify().is_err(), "cross-topic replay must not verify");
}

#[test]
fn epoch_advance_is_monotonic_fresh_topic_and_admin_bound() {
    let admin = SecretSigningKey::generate();
    let old_topic = TopicId::random();
    let mut a = EpochAdvance {
        old_topic,
        new_topic: TopicId::random(),
        old_epoch: Epoch(2),
        new_epoch: Epoch(3),
        transition_commit: CommitId::random(),
        new_publishers: vec![],
        admin_pub: admin.public().to_bytes(),
        admin_sig: None,
    };
    a.sign(&admin).expect("sign advance");
    a.verify(&admin.public()).expect("verifies under its admin key");

    // A different admin key must not be accepted even with a valid self-signature.
    let other = SecretSigningKey::generate();
    assert!(a.verify(&other.public()).is_err());

    // Backwards / same epoch is rejected before any signature work.
    let mut backwards = a.clone();
    backwards.new_epoch = Epoch(2);
    assert!(backwards.check_monotonic().is_err());
    assert!(backwards.sign(&admin).is_err());

    // Reusing the retired topic id defeats the point of a fresh topic.
    let mut same_topic = a.clone();
    same_topic.new_topic = old_topic;
    assert!(same_topic.check_monotonic().is_err());
}

#[test]
fn bloom_hint_has_no_false_negatives() {
    let mut b = BloomHint::new(256, 4);
    let held: Vec<BlockId> = (0..50).map(|_| BlockId::random()).collect();
    for id in &held {
        b.insert(id);
    }
    for id in &held {
        assert!(b.probably_contains(id), "inserted id reported absent");
    }
    // An empty filter never claims membership (it is a hint, not an authority).
    let empty = BloomHint::new(0, 4);
    assert!(!empty.probably_contains(&held[0]));
}

/// The disclosure ledger (§5), asserted on real bytes: a complete client->broker
/// transcript must not contain the branch read secret, nor any SPARQL text, nor
/// the repo/branch identifiers, even though the blocks it carries were sealed
/// under exactly those.
#[test]
fn session_transcript_carries_no_secret_no_query_and_no_repo_or_branch_id() {
    let k_read = Secret32::random();
    let repo = RepoId::random();
    let branch = BranchId::random();
    let ctx = BlockContext {
        repo,
        branch,
        epoch: Epoch(0),
        kind: ObjectKind::Commit,
    };
    let query = b"SELECT ?s WHERE { ?s <http://example/p> ?o }";
    let env = seal_block_random(&k_read, &ctx, &ObjectId::random(), 0, 1, query).expect("seal");

    let admin = SecretSigningKey::generate();
    let publisher = SecretSigningKey::generate();
    let topic = TopicId::random();
    let mut publish = PublishEvent {
        topic,
        epoch: Epoch(0),
        commit_id: env.commit_id(),
        root_block_id: env.block_id,
        publisher_key_id: [0u8; 32],
        parents: vec![],
        signature: None,
    };
    publish.sign(&publisher);

    let mut transcript: Vec<u8> = Vec::new();
    let mut g = AdmissionGrant {
        topic,
        epoch: Epoch(0),
        suite: SUITE_V0.to_string(),
        admin_pub: admin.public().to_bytes(),
        publisher_pub: Some(publisher.public().to_bytes()),
        validity: Validity {
            not_before: 0,
            not_after: u64::MAX,
        },
        admin_sig: None,
    };
    g.sign(&admin).expect("sign grant");
    for (i, req) in [
        Request::Hello(hello_v0(1 << 20)),
        Request::OpenRepo(OpenRepo {
            overlay: OverlayId::random(),
            topic,
            epoch: Epoch(0),
            peer: PeerId::random(),
            auth: Some(g),
        }),
        Request::BlocksPut {
            envelopes: vec![env.clone()],
        },
        Request::PublishEvent(publish),
    ]
    .into_iter()
    .enumerate()
    {
        transcript.extend_from_slice(&req.encode(i as u64));
    }

    assert!(
        !contains(&transcript, k_read.expose()),
        "the read secret appeared on the wire"
    );
    assert!(
        !contains(&transcript, query),
        "query text appeared on the wire"
    );
    assert!(
        !contains(&transcript, repo.as_bytes()),
        "the stable RepoId appeared on the wire"
    );
    assert!(
        !contains(&transcript, branch.as_bytes()),
        "the stable BranchId appeared on the wire"
    );
    // Sanity: the transcript really does carry the ciphertext and the topic, so
    // the assertions above are not vacuously true of an empty buffer.
    assert!(contains(&transcript, &env.ciphertext));
    assert!(contains(&transcript, topic.as_bytes()));
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}
