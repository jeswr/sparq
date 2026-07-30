//! Behavioural tests for the opaque broker (design §8.4).
//!
//! Each test pins one enforcement the design makes normative: negotiation before
//! anything else, no silent suite substitution, routing-context scoping,
//! publisher admission (admitted key + valid signature + uploaded root block),
//! opaque-header discipline, idempotent exact-byte storage, have/want paging,
//! epoch advance under the pinned admin key, limits, rate, retention, and a log
//! that cannot carry content.

use sparq_e2ee_ng::broker_protocol::*;
use sparq_e2ee_ng::capability::Validity;
use sparq_e2ee_ng::envelope::{seal_block_random, BlockContext, BlockEnvelope, ObjectKind};
use sparq_e2ee_ng::ids::{
    BlockId, BranchId, CommitId, Epoch, ObjectId, OverlayId, PeerId, RepoId, Secret32, TopicId,
};
use sparq_e2ee_ng::sign::SecretSigningKey;
use sparq_e2ee_ng::suite::SUITE_V0;
use sparq_e2ee_ng_broker::broker::{Broker, BrokerConfig, SessionId};
use sparq_e2ee_ng_broker::log::CaptureLog;

const NOW: u64 = 1_800_000_000;

struct Fixture {
    broker: Broker<CaptureLog>,
    session: SessionId,
    admin: SecretSigningKey,
    publisher: SecretSigningKey,
    topic: TopicId,
    k_read: Secret32,
    repo: RepoId,
    branch: BranchId,
}

/// Send one request on the fixture's session.
///
/// A macro rather than a function so the request expression (which usually reads
/// the fixture, e.g. `seal(&f, ..)`) is evaluated BEFORE the mutable borrow of the
/// broker, keeping the call sites readable.
macro_rules! call {
    ($f:ident, $req:expr $(,)?) => {{
        let req = $req;
        let bytes = req.encode(1).len() as u64;
        $f.broker.handle($f.session, NOW, req, bytes)
    }};
}

fn grant_for(
    admin: &SecretSigningKey,
    publisher: Option<&SecretSigningKey>,
    topic: TopicId,
    epoch: Epoch,
) -> AdmissionGrant {
    let mut g = AdmissionGrant {
        topic,
        epoch,
        suite: SUITE_V0.to_string(),
        admin_pub: admin.public().to_bytes(),
        publisher_pub: publisher.map(|p| p.public().to_bytes()),
        validity: Validity {
            not_before: 0,
            not_after: u64::MAX,
        },
        admin_sig: None,
    };
    g.sign(admin).expect("sign grant");
    g
}

/// A negotiated session with an open routing context on a fresh topic whose
/// admin key is pinned by the opening grant, and whose publisher is admitted.
fn open_fixture(cfg: BrokerConfig) -> Fixture {
    let mut broker = Broker::new(cfg, CaptureLog::default());
    let session = broker.open_session();
    let mut f = Fixture {
        broker,
        session,
        admin: SecretSigningKey::generate(),
        publisher: SecretSigningKey::generate(),
        topic: TopicId::random(),
        k_read: Secret32::random(),
        repo: RepoId::random(),
        branch: BranchId::random(),
    };
    assert!(matches!(
        call!(f, Request::Hello(hello_v0(1 << 20))),
        Response::HelloAck(_)
    ));
    let g = grant_for(&f.admin, Some(&f.publisher), f.topic, Epoch(0));
    let open = Request::OpenRepo(OpenRepo {
        overlay: OverlayId::random(),
        topic: f.topic,
        epoch: Epoch(0),
        peer: PeerId::random(),
        auth: Some(g),
    });
    assert!(matches!(call!(f, open), Response::Ok));
    f
}

fn seal(f: &Fixture, plaintext: &[u8]) -> BlockEnvelope {
    let ctx = BlockContext {
        repo: f.repo,
        branch: f.branch,
        epoch: Epoch(0),
        kind: ObjectKind::Commit,
    };
    seal_block_random(&f.k_read, &ctx, &ObjectId::random(), 0, 1, plaintext).expect("seal")
}

fn signed_publish(f: &Fixture, env: &BlockEnvelope) -> PublishEvent {
    let mut p = PublishEvent {
        topic: f.topic,
        epoch: Epoch(0),
        commit_id: env.commit_id(),
        root_block_id: env.block_id,
        publisher_key_id: [0u8; 32],
        parents: vec![],
        signature: None,
    };
    p.sign(&f.publisher);
    p
}

fn err_code(r: &Response) -> Option<ErrorCode> {
    match r {
        Response::Error(e) => Some(e.code),
        _ => None,
    }
}

// ===========================================================================
// Negotiation
// ===========================================================================

#[test]
fn negotiation_is_required_before_any_other_request() {
    let mut broker = Broker::new(BrokerConfig::default(), CaptureLog::default());
    let s = broker.open_session();
    let r = broker.handle(
        s,
        NOW,
        Request::RepoPinStatus {
            topic: TopicId::random(),
        },
        0,
    );
    assert_eq!(err_code(&r), Some(ErrorCode::NotNegotiated));
}

#[test]
fn an_unknown_suite_is_rejected_not_silently_substituted() {
    let mut broker = Broker::new(BrokerConfig::default(), CaptureLog::default());
    let s = broker.open_session();
    let mut h = hello_v0(1 << 20);
    h.suites = vec!["urn:example:not-a-real-suite".to_string()];
    let r = broker.handle(s, NOW, Request::Hello(h), 0);
    assert_eq!(err_code(&r), Some(ErrorCode::Unsupported));

    // ...and the session stays un-negotiated, so nothing else can proceed.
    let r = broker.handle(
        s,
        NOW,
        Request::RepoPinStatus {
            topic: TopicId::random(),
        },
        0,
    );
    assert_eq!(err_code(&r), Some(ErrorCode::NotNegotiated));
}

#[test]
fn clear_header_mode_is_not_offered_by_default() {
    let mut broker = Broker::new(BrokerConfig::default(), CaptureLog::default());
    let s = broker.open_session();
    let mut h = hello_v0(1 << 20);
    h.header_modes = vec![HeaderMode::Clear];
    assert_eq!(
        err_code(&broker.handle(s, NOW, Request::Hello(h), 0)),
        Some(ErrorCode::Unsupported)
    );
}

// ===========================================================================
// Routing / block operations
// ===========================================================================

#[test]
fn block_operations_require_an_open_routing_context() {
    let mut broker = Broker::new(BrokerConfig::default(), CaptureLog::default());
    let s = broker.open_session();
    broker.handle(s, NOW, Request::Hello(hello_v0(1 << 20)), 0);
    let r = broker.handle(
        s,
        NOW,
        Request::BlocksExist {
            ids: vec![BlockId::random()],
        },
        0,
    );
    assert_eq!(err_code(&r), Some(ErrorCode::NoRouting));
}

#[test]
fn put_is_idempotent_and_get_returns_byte_identical_envelopes() {
    let mut f = open_fixture(BrokerConfig::default());
    let env = seal(&f, b"commit root plaintext");
    let put = Request::BlocksPut {
        envelopes: vec![env.clone()],
    };
    assert_eq!(
        call!(f, put.clone()),
        Response::Stored {
            stored: 1,
            duplicate: 0
        }
    );
    assert_eq!(
        call!(f, put),
        Response::Stored {
            stored: 0,
            duplicate: 1
        }
    );
    match call!(f,
        Request::BlocksGet {
            ids: vec![env.block_id],
        },
    ) {
        Response::Blocks { found, missing } => {
            assert!(missing.is_empty());
            assert_eq!(found.len(), 1);
            assert_eq!(found[0].encode(), env.encode(), "stored bytes changed");
        }
        other => panic!("expected Blocks, got {:?}", other),
    }
}

#[test]
fn blocks_exist_bit_vector_matches_the_stored_set() {
    let mut f = open_fixture(BrokerConfig::default());
    let held: Vec<BlockEnvelope> = (0..3).map(|_| seal(&f, b"held")).collect();
    call!(f,
        Request::BlocksPut {
            envelopes: held.clone(),
        },
    );
    let absent: Vec<BlockId> = (0..3).map(|_| BlockId::random()).collect();
    // Interleave held / absent so a constant answer cannot pass.
    let ids = vec![
        held[0].block_id,
        absent[0],
        held[1].block_id,
        absent[1],
        held[2].block_id,
        absent[2],
    ];
    match call!(f, Request::BlocksExist { ids: ids.clone() }) {
        Response::ExistBits { bits, count } => {
            assert_eq!(count, ids.len() as u64);
            for i in 0..ids.len() {
                let set = bits[i / 8] & (1u8 << (i % 8)) != 0;
                assert_eq!(set, i % 2 == 0, "wrong presence bit at index {}", i);
            }
        }
        other => panic!("expected ExistBits, got {:?}", other),
    }
}

#[test]
fn presence_is_scoped_to_the_sessions_routing_topic() {
    let mut f = open_fixture(BrokerConfig::default());
    let env = seal(&f, b"topic a block");
    call!(f,
        Request::BlocksPut {
            envelopes: vec![env.clone()],
        },
    );

    // A second topic, opened by the same peer, must not learn that the block
    // exists under the first.
    let other_topic = TopicId::random();
    let g = grant_for(&f.admin, Some(&f.publisher), other_topic, Epoch(0));
    assert!(matches!(
        call!(f,
            Request::OpenRepo(OpenRepo {
                overlay: OverlayId::random(),
                topic: other_topic,
                epoch: Epoch(0),
                peer: PeerId::random(),
                auth: Some(g),
            })
        ),
        Response::Ok
    ));
    match call!(f,
        Request::BlocksExist {
            ids: vec![env.block_id],
        },
    ) {
        Response::ExistBits { bits, .. } => assert_eq!(bits[0] & 1, 0, "cross-topic confirmation"),
        other => panic!("expected ExistBits, got {:?}", other),
    }
}

// ===========================================================================
// Publication + admission
// ===========================================================================

#[test]
fn publish_requires_an_admitted_key_a_valid_signature_and_an_uploaded_root() {
    let mut f = open_fixture(BrokerConfig::default());
    let env = seal(&f, b"root");
    let p = signed_publish(&f, &env);

    // Root block not uploaded yet.
    assert_eq!(
        err_code(&call!(f, Request::PublishEvent(p.clone()))),
        Some(ErrorCode::NotFound)
    );
    call!(f,
        Request::BlocksPut {
            envelopes: vec![env.clone()],
        },
    );

    // An un-admitted publisher, even with a perfectly valid self-signature.
    let stranger = SecretSigningKey::generate();
    let mut rogue = p.clone();
    rogue.sign(&stranger);
    rogue.verify().expect("the rogue announcement is self-consistent");
    assert_eq!(
        err_code(&call!(f, Request::PublishEvent(rogue))),
        Some(ErrorCode::NotAdmitted)
    );

    // An admitted key with a tampered field.
    let mut tampered = p.clone();
    tampered.commit_id = CommitId::random();
    assert_eq!(
        err_code(&call!(f, Request::PublishEvent(tampered))),
        Some(ErrorCode::BadSignature)
    );

    // The real thing.
    match call!(f, Request::PublishEvent(p.clone())) {
        Response::Published { commit_id, cursor } => {
            assert_eq!(commit_id, p.commit_id);
            assert_eq!(cursor, 1);
        }
        other => panic!("expected Published, got {:?}", other),
    }
}

#[test]
fn a_publisher_admission_expires_with_the_grant_that_carried_it() {
    let mut f = open_fixture(BrokerConfig::default());

    // A second publisher, admitted by a grant valid only for [NOW, NOW + 10].
    // (The fixture's own publisher holds an unbounded grant, so it cannot show
    // expiry.) The topic's admin key is already pinned, so this grant is folded
    // into the existing topic rather than pinning a fresh one.
    let temporary = SecretSigningKey::generate();
    let mut g = grant_for(&f.admin, Some(&temporary), f.topic, Epoch(0));
    g.validity = Validity {
        not_before: NOW,
        not_after: NOW + 10,
    };
    g.sign(&f.admin).expect("re-sign the narrowed grant");
    assert!(matches!(
        call!(f,
            Request::OpenRepo(OpenRepo {
                overlay: OverlayId::random(),
                topic: f.topic,
                epoch: Epoch(0),
                peer: PeerId::random(),
                auth: Some(g),
            })
        ),
        Response::Ok
    ));

    // Inside the window: the real path accepts.
    let env = seal(&f, b"in-window root");
    call!(f,
        Request::BlocksPut {
            envelopes: vec![env.clone()],
        },
    );
    let mut inside = signed_publish(&f, &env);
    inside.sign(&temporary);
    assert!(matches!(
        f.broker
            .handle(f.session, NOW + 5, Request::PublishEvent(inside), 0),
        Response::Published { .. }
    ));

    // Past not_after: the same publisher, a correctly signed announcement, and an
    // uploaded root — and it is no longer admitted.
    let late_env = seal(&f, b"post-expiry root");
    f.broker.handle(
        f.session,
        NOW + 5,
        Request::BlocksPut {
            envelopes: vec![late_env.clone()],
        },
        0,
    );
    let mut late = signed_publish(&f, &late_env);
    late.sign(&temporary);
    late.verify().expect("the late announcement is self-consistent");
    assert_eq!(
        err_code(&f.broker.handle(f.session, NOW + 11, Request::PublishEvent(late), 0)),
        Some(ErrorCode::NotAdmitted)
    );
}

#[test]
fn parent_commit_ids_are_refused_in_opaque_header_mode() {
    let mut f = open_fixture(BrokerConfig::default());
    let env = seal(&f, b"root");
    call!(f,
        Request::BlocksPut {
            envelopes: vec![env.clone()],
        },
    );
    let mut p = PublishEvent {
        topic: f.topic,
        epoch: Epoch(0),
        commit_id: env.commit_id(),
        root_block_id: env.block_id,
        publisher_key_id: [0u8; 32],
        parents: vec![CommitId::random()],
        signature: None,
    };
    p.sign(&f.publisher);
    // Signed, admitted, root uploaded — and still refused, because accepting it
    // would silently widen the §5 disclosure ledger.
    assert_eq!(
        err_code(&call!(f, Request::PublishEvent(p))),
        Some(ErrorCode::Protocol)
    );
}

#[test]
fn a_published_commit_fans_out_to_subscribers() {
    let mut f = open_fixture(BrokerConfig::default());
    let env = seal(&f, b"root");
    call!(f,
        Request::BlocksPut {
            envelopes: vec![env.clone()],
        },
    );

    // A second, read-only session subscribes to the same topic.
    let reader = f.broker.open_session();
    f.broker
        .handle(reader, NOW, Request::Hello(hello_v0(1 << 20)), 0);
    assert!(matches!(
        f.broker.handle(
            reader,
            NOW,
            Request::OpenRepo(OpenRepo {
                overlay: OverlayId::random(),
                topic: f.topic,
                epoch: Epoch(0),
                peer: PeerId::random(),
                auth: None,
            }),
            0,
        ),
        Response::Ok
    ));
    assert!(matches!(
        f.broker.handle(
            reader,
            NOW,
            Request::TopicSub(TopicSub {
                topic: f.topic,
                epoch: Epoch(0),
                after_cursor: None,
            }),
            0,
        ),
        Response::Ok
    ));

    let p = signed_publish(&f, &env);
    assert!(matches!(
        call!(f, Request::PublishEvent(p.clone())),
        Response::Published { .. }
    ));

    let pushes = f.broker.take_pushes(reader);
    assert_eq!(pushes.len(), 1);
    match &pushes[0] {
        Response::Event(e) => {
            assert_eq!(e.announcement, p);
            assert_eq!(e.cursor, 1);
        }
        other => panic!("expected an Event push, got {:?}", other),
    }
    // Drained once.
    assert!(f.broker.take_pushes(reader).is_empty());
}

#[test]
fn a_subscription_replays_events_after_a_cursor() {
    let mut f = open_fixture(BrokerConfig::default());
    for _ in 0..3 {
        let env = seal(&f, b"root");
        call!(f,
            Request::BlocksPut {
                envelopes: vec![env.clone()],
            },
        );
        assert!(matches!(
            call!(f, Request::PublishEvent(signed_publish(&f, &env))),
            Response::Published { .. }
        ));
    }
    assert!(matches!(
        call!(f,
            Request::TopicSub(TopicSub {
                topic: f.topic,
                epoch: Epoch(0),
                after_cursor: Some(1),
            })
        ),
        Response::Ok
    ));
    let pushes = f.broker.take_pushes(f.session);
    assert_eq!(pushes.len(), 2, "expected cursors 2 and 3 replayed");
}

// ===========================================================================
// Have/want sync
// ===========================================================================

#[test]
fn sync_pages_missing_blocks_and_a_bloom_hint_suppresses_held_ids() {
    let mut cfg = BrokerConfig::default();
    cfg.limits.max_ids_per_request = 2;
    let mut f = open_fixture(cfg);
    let envs: Vec<BlockEnvelope> = (0..5).map(|_| seal(&f, b"chunk")).collect();
    call!(f,
        Request::BlocksPut {
            envelopes: envs.clone(),
        },
    );

    let mut all: Vec<BlockId> = envs.iter().map(|e| e.block_id).collect();
    all.sort();

    // Page 1.
    let page1 = match call!(f,
        Request::TopicSyncReq(TopicSyncReq {
            topic: f.topic,
            epoch: Epoch(0),
            known_heads: vec![],
            target_heads: None,
            known_commits: None,
            page_after: None,
        }),
    ) {
        Response::SyncResp(s) => s,
        other => panic!("expected SyncResp, got {:?}", other),
    };
    assert_eq!(page1.missing_block_ids, all[0..2].to_vec());
    assert!(page1.more);

    // Page 2 resumes strictly after the last id, so paging terminates.
    let page2 = match call!(f,
        Request::TopicSyncReq(TopicSyncReq {
            topic: f.topic,
            epoch: Epoch(0),
            known_heads: vec![],
            target_heads: None,
            known_commits: None,
            page_after: Some(all[1]),
        }),
    ) {
        Response::SyncResp(s) => s,
        other => panic!("expected SyncResp, got {:?}", other),
    };
    assert_eq!(page2.missing_block_ids, all[2..4].to_vec());

    // A Bloom hint naming everything the client already holds suppresses them.
    let mut bloom = BloomHint::new(256, 4);
    for id in &all {
        bloom.insert(id);
    }
    let hinted = match call!(f,
        Request::TopicSyncReq(TopicSyncReq {
            topic: f.topic,
            epoch: Epoch(0),
            known_heads: vec![],
            target_heads: None,
            known_commits: Some(bloom),
            page_after: None,
        }),
    ) {
        Response::SyncResp(s) => s,
        other => panic!("expected SyncResp, got {:?}", other),
    };
    assert!(hinted.missing_block_ids.is_empty());
    assert!(!hinted.more);
}

#[test]
fn sync_advertises_announced_commits_the_client_does_not_know() {
    let mut f = open_fixture(BrokerConfig::default());
    let env = seal(&f, b"root");
    call!(f,
        Request::BlocksPut {
            envelopes: vec![env.clone()],
        },
    );
    let p = signed_publish(&f, &env);
    call!(f, Request::PublishEvent(p.clone()));

    let resp = match call!(f,
        Request::TopicSyncReq(TopicSyncReq {
            topic: f.topic,
            epoch: Epoch(0),
            known_heads: vec![],
            target_heads: None,
            known_commits: None,
            page_after: None,
        }),
    ) {
        Response::SyncResp(s) => s,
        other => panic!("expected SyncResp, got {:?}", other),
    };
    assert_eq!(resp.advertised_heads, vec![p.commit_id]);
    assert_eq!(resp.cursor, 1);

    // Declaring it known removes it from the advertisement.
    let resp = match call!(f,
        Request::TopicSyncReq(TopicSyncReq {
            topic: f.topic,
            epoch: Epoch(0),
            known_heads: vec![p.commit_id],
            target_heads: None,
            known_commits: None,
            page_after: None,
        }),
    ) {
        Response::SyncResp(s) => s,
        other => panic!("expected SyncResp, got {:?}", other),
    };
    assert!(resp.advertised_heads.is_empty());
}

#[test]
fn commit_get_resolves_announced_roots_and_reports_the_rest_missing() {
    let mut f = open_fixture(BrokerConfig::default());
    let env = seal(&f, b"root");
    call!(f,
        Request::BlocksPut {
            envelopes: vec![env.clone()],
        },
    );
    let p = signed_publish(&f, &env);
    call!(f, Request::PublishEvent(p.clone()));
    let unknown = CommitId::random();
    match call!(f,
        Request::CommitGet {
            commit_ids: vec![p.commit_id, unknown],
        },
    ) {
        Response::Commits { found, missing } => {
            assert_eq!(found.len(), 1);
            assert_eq!(found[0].encode(), env.encode());
            assert_eq!(missing, vec![unknown]);
        }
        other => panic!("expected Commits, got {:?}", other),
    }
}

// ===========================================================================
// Admission pinning + epoch advance
// ===========================================================================

#[test]
fn a_grant_under_a_different_admin_key_is_rejected_after_the_topic_is_pinned() {
    let mut f = open_fixture(BrokerConfig::default());
    let impostor = SecretSigningKey::generate();
    let forged = grant_for(&impostor, Some(&impostor), f.topic, Epoch(0));
    forged.verify_self().expect("the forged grant is self-consistent");
    let r = call!(f,
        Request::OpenRepo(OpenRepo {
            overlay: OverlayId::random(),
            topic: f.topic,
            epoch: Epoch(0),
            peer: PeerId::random(),
            auth: Some(forged),
        }),
    );
    assert_eq!(err_code(&r), Some(ErrorCode::NotAdmitted));
}

#[test]
fn epoch_advance_requires_the_pinned_admin_key_and_retires_the_old_topic() {
    let mut f = open_fixture(BrokerConfig::default());
    assert!(matches!(
        call!(f,
            Request::TopicSub(TopicSub {
                topic: f.topic,
                epoch: Epoch(0),
                after_cursor: None,
            })
        ),
        Response::Ok
    ));
    let new_topic = TopicId::random();
    let new_publisher = SecretSigningKey::generate();

    // Signed by someone who is not the pinned admin.
    let impostor = SecretSigningKey::generate();
    let mut forged = EpochAdvance {
        old_topic: f.topic,
        new_topic,
        old_epoch: Epoch(0),
        new_epoch: Epoch(1),
        transition_commit: CommitId::random(),
        new_publishers: vec![new_publisher.public().to_bytes()],
        admin_pub: impostor.public().to_bytes(),
        admin_sig: None,
    };
    forged.sign(&impostor).expect("sign");
    assert_eq!(
        err_code(&call!(f, Request::EpochAdvance(forged))),
        Some(ErrorCode::BadSignature)
    );

    // The real advance.
    let mut advance = EpochAdvance {
        old_topic: f.topic,
        new_topic,
        old_epoch: Epoch(0),
        new_epoch: Epoch(1),
        transition_commit: CommitId::random(),
        new_publishers: vec![new_publisher.public().to_bytes()],
        admin_pub: f.admin.public().to_bytes(),
        admin_sig: None,
    };
    advance.sign(&f.admin).expect("sign");
    assert!(matches!(
        call!(f, Request::EpochAdvance(advance.clone())),
        Response::Ok
    ));

    // Subscribers are told, so they can re-open at the new epoch.
    let pushes = f.broker.take_pushes(f.session);
    assert!(matches!(pushes.as_slice(), [Response::EpochAdvanced(a)] if *a == advance));

    // The retired topic accepts nothing further, and the stale routing context
    // fails closed rather than silently writing under the old epoch.
    let env = seal(&f, b"late");
    assert_eq!(
        err_code(&call!(f,
            Request::BlocksPut {
                envelopes: vec![env.clone()]
            }
        )),
        Some(ErrorCode::Retired)
    );
    assert_eq!(
        err_code(&call!(f, Request::PublishEvent(signed_publish(&f, &env)))),
        Some(ErrorCode::Retired)
    );

    // The revoked publisher is no longer admitted on the new topic.
    assert!(matches!(
        call!(f,
            Request::OpenRepo(OpenRepo {
                overlay: OverlayId::random(),
                topic: new_topic,
                epoch: Epoch(1),
                peer: PeerId::random(),
                auth: None,
            })
        ),
        Response::Ok
    ));
    let env2 = seal(&f, b"post-rotation");
    call!(f,
        Request::BlocksPut {
            envelopes: vec![env2.clone()],
        },
    );
    let mut old_pub = PublishEvent {
        topic: new_topic,
        epoch: Epoch(1),
        commit_id: env2.commit_id(),
        root_block_id: env2.block_id,
        publisher_key_id: [0u8; 32],
        parents: vec![],
        signature: None,
    };
    old_pub.sign(&f.publisher);
    assert_eq!(
        err_code(&call!(f, Request::PublishEvent(old_pub))),
        Some(ErrorCode::NotAdmitted)
    );
}

// ===========================================================================
// Limits, rate, retention
// ===========================================================================

#[test]
fn identifier_list_and_put_batch_limits_are_enforced() {
    let mut cfg = BrokerConfig::default();
    cfg.limits.max_ids_per_request = 2;
    cfg.limits.max_blocks_per_put = 1;
    let mut f = open_fixture(cfg);
    assert_eq!(
        err_code(&call!(f,
            Request::BlocksExist {
                ids: vec![BlockId::random(), BlockId::random(), BlockId::random()],
            }
        )),
        Some(ErrorCode::LimitExceeded)
    );
    assert_eq!(
        err_code(&call!(f,
            Request::BlocksPut {
                envelopes: vec![seal(&f, b"a"), seal(&f, b"b")],
            }
        )),
        Some(ErrorCode::LimitExceeded)
    );
}

#[test]
fn an_oversized_frame_is_rejected_before_it_is_parsed() {
    let mut cfg = BrokerConfig::default();
    cfg.limits.max_message_bytes = 64;
    let mut broker = Broker::new(cfg, CaptureLog::default());
    let s = broker.open_session();
    let frame = Request::Hello(hello_v0(1 << 20)).encode(1);
    assert!(frame.len() > 64);
    let reply = broker.handle_frame(s, NOW, &frame);
    let (id, resp) = Response::decode(&reply, protocol_limits(1 << 20, 1024)).expect("decode");
    assert_eq!(id, 0);
    assert_eq!(err_code(&resp), Some(ErrorCode::LimitExceeded));
}

#[test]
fn a_malformed_frame_is_answered_with_a_typed_protocol_error() {
    let mut broker = Broker::new(BrokerConfig::default(), CaptureLog::default());
    let s = broker.open_session();
    let reply = broker.handle_frame(s, NOW, b"\xff\xff not cbor");
    let (_, resp) = Response::decode(&reply, protocol_limits(1 << 20, 1024)).expect("decode");
    assert_eq!(err_code(&resp), Some(ErrorCode::Protocol));
}

#[test]
fn a_well_formed_frame_round_trips_through_the_transport_entry_point() {
    let mut broker = Broker::new(BrokerConfig::default(), CaptureLog::default());
    let s = broker.open_session();
    let frame = Request::Hello(hello_v0(1 << 20)).encode(77);
    let reply = broker.handle_frame(s, NOW, &frame);
    let (id, resp) = Response::decode(&reply, protocol_limits(1 << 20, 1024)).expect("decode");
    assert_eq!(id, 77, "the response must echo the request id");
    match resp {
        Response::HelloAck(a) => {
            assert_eq!(a.version, PROTOCOL_V0);
            assert_eq!(a.suite, SUITE_V0);
            assert_eq!(a.header_mode, HeaderMode::Opaque);
        }
        other => panic!("expected HelloAck, got {:?}", other),
    }
}

#[test]
fn the_request_rate_allowance_is_enforced_per_window() {
    let mut cfg = BrokerConfig::default();
    cfg.limits.max_requests_per_window = 2;
    cfg.limits.rate_window_secs = 10;
    let mut broker = Broker::new(cfg, CaptureLog::default());
    let s = broker.open_session();
    assert!(matches!(
        broker.handle(s, NOW, Request::Hello(hello_v0(1 << 20)), 0),
        Response::HelloAck(_)
    ));
    let second = broker.handle(
        s,
        NOW,
        Request::RepoPinStatus {
            topic: TopicId::random(),
        },
        0,
    );
    assert_ne!(err_code(&second), Some(ErrorCode::RateLimited));
    let third = broker.handle(
        s,
        NOW,
        Request::RepoPinStatus {
            topic: TopicId::random(),
        },
        0,
    );
    assert_eq!(err_code(&third), Some(ErrorCode::RateLimited));
    // The next window resets the allowance.
    let later = broker.handle(s, NOW + 10, Request::Hello(hello_v0(1 << 20)), 0);
    assert!(matches!(later, Response::HelloAck(_)));
}

#[test]
fn unpinned_aged_blocks_are_collected_and_pinned_ones_are_not() {
    let mut cfg = BrokerConfig::default();
    cfg.retention.unpinned_ttl_secs = 100;
    let mut f = open_fixture(cfg);
    let env = seal(&f, b"transient");
    call!(f,
        Request::BlocksPut {
            envelopes: vec![env.clone()],
        },
    );
    assert_eq!(f.broker.stored_blocks(), 1);

    // Not yet aged.
    assert_eq!(f.broker.collect_garbage(NOW + 50).removed_blocks, 0);
    assert_eq!(f.broker.stored_blocks(), 1);

    // Pinned topics are exempt.
    assert!(matches!(
        call!(f,
            Request::PinRepo(PinRepo {
                topic: f.topic,
                pin: true
            })
        ),
        Response::Ok
    ));
    assert_eq!(f.broker.collect_garbage(NOW + 1000).removed_blocks, 0);
    assert_eq!(f.broker.stored_blocks(), 1);

    // Unpin, then the aged block is collectable.
    assert!(matches!(
        call!(f,
            Request::PinRepo(PinRepo {
                topic: f.topic,
                pin: false
            })
        ),
        Response::Ok
    ));
    let report = f.broker.collect_garbage(NOW + 1000);
    assert_eq!(report.removed_blocks, 1);
    assert!(report.removed_bytes > 0);
    assert_eq!(f.broker.stored_blocks(), 0);
}

#[test]
fn pin_status_reports_topic_storage_and_the_advertised_retention_policy() {
    let mut f = open_fixture(BrokerConfig::default());
    let env = seal(&f, b"x");
    call!(f,
        Request::BlocksPut {
            envelopes: vec![env.clone()],
        },
    );
    match call!(f,
        Request::RepoPinStatus {
            topic: f.topic,
        },
    ) {
        Response::PinStatus(s) => {
            assert_eq!(s.topic, f.topic);
            assert!(!s.pinned);
            assert_eq!(s.blocks, 1);
            assert_eq!(s.bytes, env.encode().len() as u64);
            assert_eq!(s.retention, f.broker.config().retention);
        }
        other => panic!("expected PinStatus, got {:?}", other),
    }
}

#[test]
fn a_topic_storage_budget_is_enforced_on_put() {
    let mut cfg = BrokerConfig::default();
    cfg.retention.max_topic_bytes = 600; // smaller than two padded 256-byte blocks
    let mut f = open_fixture(cfg);
    assert!(matches!(
        call!(f,
            Request::BlocksPut {
                envelopes: vec![seal(&f, b"one")]
            }
        ),
        Response::Stored { .. }
    ));
    assert_eq!(
        err_code(&call!(f,
            Request::BlocksPut {
                envelopes: vec![seal(&f, b"two")]
            }
        )),
        Some(ErrorCode::LimitExceeded)
    );
}

#[test]
fn repeating_a_put_at_the_exact_quota_boundary_stays_idempotent() {
    // Size the budget to admit exactly one envelope. Envelope length depends only
    // on the padded plaintext length, so a probe fixture measures it; the assert
    // below keeps that assumption honest.
    let probe = open_fixture(BrokerConfig::default());
    let one_envelope = seal(&probe, b"exactly one").encode().len() as u64;
    let mut cfg = BrokerConfig::default();
    cfg.retention.max_topic_bytes = one_envelope;
    let mut f = open_fixture(cfg);
    let env = seal(&f, b"exactly one");
    assert_eq!(env.encode().len() as u64, one_envelope);

    let put = Request::BlocksPut {
        envelopes: vec![env],
    };
    assert_eq!(
        call!(f, put.clone()),
        Response::Stored {
            stored: 1,
            duplicate: 0
        }
    );
    // The topic is now exactly at budget. Re-attributing a block it already holds
    // consumes no further bytes, so the repeat is a duplicate, not LimitExceeded.
    assert_eq!(
        call!(f, put),
        Response::Stored {
            stored: 0,
            duplicate: 1
        }
    );
    // ...and the budget really is exhausted for anything new.
    assert_eq!(
        err_code(&call!(f,
            Request::BlocksPut {
                envelopes: vec![seal(&f, b"another one")],
            }
        )),
        Some(ErrorCode::LimitExceeded)
    );
}

// ===========================================================================
// Metadata-safe logging
// ===========================================================================

#[test]
fn the_log_carries_no_ciphertext_no_plaintext_and_no_full_identifier() {
    let mut f = open_fixture(BrokerConfig::default());
    let marker = b"UNIQUE-PLAINTEXT-MARKER-9d2f";
    let env = seal(&f, marker);
    call!(f,
        Request::BlocksPut {
            envelopes: vec![env.clone()],
        },
    );
    call!(f, Request::PublishEvent(signed_publish(&f, &env)));

    let log = f.broker.log().lines.join("\n");
    assert!(!log.is_empty(), "the capture log recorded nothing");
    assert!(
        !log.contains(std::str::from_utf8(marker).expect("ascii")),
        "plaintext leaked into the log"
    );
    assert!(!log.contains(&hex(&env.ciphertext)), "ciphertext leaked into the log");
    assert!(!log.contains(&hex(env.block_id.as_bytes())), "a full block id leaked");
    assert!(!log.contains(&hex(f.topic.as_bytes())), "a full topic id leaked");
    assert!(!log.contains(&hex(f.k_read.expose())), "the read secret leaked");
    // The truncated prefix IS present — the log is useful, just not a channel.
    assert!(log.contains(&hex(&f.topic.as_bytes()[..4])));
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}

// ===========================================================================
// Operator-side APIs
// ===========================================================================

#[test]
fn pre_pinning_an_admin_key_overrides_trust_on_first_use() {
    let mut broker = Broker::new(BrokerConfig::default(), CaptureLog::default());
    let admin = SecretSigningKey::generate();
    let impostor = SecretSigningKey::generate();
    let topic = TopicId::random();
    assert!(broker.pin_admin(topic, admin.public().to_bytes(), Epoch(0)));
    // Re-pinning the same key is a no-op success; a different key is refused.
    assert!(broker.pin_admin(topic, admin.public().to_bytes(), Epoch(0)));
    assert!(!broker.pin_admin(topic, impostor.public().to_bytes(), Epoch(0)));

    let s = broker.open_session();
    broker.handle(s, NOW, Request::Hello(hello_v0(1 << 20)), 0);
    // A grant from the impostor cannot claim the pre-pinned topic...
    let forged = grant_for(&impostor, Some(&impostor), topic, Epoch(0));
    let r = broker.handle(
        s,
        NOW,
        Request::OpenRepo(OpenRepo {
            overlay: OverlayId::random(),
            topic,
            epoch: Epoch(0),
            peer: PeerId::random(),
            auth: Some(forged),
        }),
        0,
    );
    assert_eq!(err_code(&r), Some(ErrorCode::NotAdmitted));
    // ...but the real admin's grant does.
    let real = grant_for(&admin, None, topic, Epoch(0));
    assert!(matches!(
        broker.handle(
            s,
            NOW,
            Request::OpenRepo(OpenRepo {
                overlay: OverlayId::random(),
                topic,
                epoch: Epoch(0),
                peer: PeerId::random(),
                auth: Some(real),
            }),
            0,
        ),
        Response::Ok
    ));
}

#[test]
fn releasing_a_retired_topic_reclaims_its_storage_and_a_live_one_is_untouched() {
    let mut f = open_fixture(BrokerConfig::default());
    let env = seal(&f, b"pre-rotation");
    call!(
        f,
        Request::BlocksPut {
            envelopes: vec![env],
        }
    );
    assert_eq!(f.broker.stored_blocks(), 1);

    // A live topic's storage is never released by this call.
    assert_eq!(f.broker.release_retired(f.topic), Default::default());
    assert_eq!(f.broker.stored_blocks(), 1);

    let mut advance = EpochAdvance {
        old_topic: f.topic,
        new_topic: TopicId::random(),
        old_epoch: Epoch(0),
        new_epoch: Epoch(1),
        transition_commit: CommitId::random(),
        new_publishers: vec![],
        admin_pub: f.admin.public().to_bytes(),
        admin_sig: None,
    };
    advance.sign(&f.admin).expect("sign");
    assert!(matches!(call!(f, Request::EpochAdvance(advance)), Response::Ok));

    let report = f.broker.release_retired(f.topic);
    assert_eq!(report.removed_blocks, 1);
    assert!(report.removed_bytes > 0);
    assert_eq!(f.broker.stored_blocks(), 0);
}
