//! End-to-end test of the `sparq-e2ee-ng-brokerd` transport: spawn the real
//! binary on an ephemeral port and drive a full negotiate -> open -> put ->
//! publish -> fetch exchange over length-prefixed deterministic-CBOR frames.
//!
//! This is the only place the framing (a 4-byte big-endian length, then the
//! frame) is exercised; the rest of the suite drives the state machine directly.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};

use sparq_e2ee_ng::broker_protocol::*;
use sparq_e2ee_ng::capability::Validity;
use sparq_e2ee_ng::envelope::{seal_block_random, BlockContext, ObjectKind};
use sparq_e2ee_ng::ids::{
    BranchId, Epoch, ObjectId, OverlayId, PeerId, RepoId, Secret32, TopicId,
};
use sparq_e2ee_ng::sign::SecretSigningKey;
use sparq_e2ee_ng::suite::SUITE_V0;

/// Kills the daemon when the test ends, pass or fail.
struct Daemon(Child);

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn write_frame(s: &mut TcpStream, bytes: &[u8]) {
    s.write_all(&(bytes.len() as u32).to_be_bytes()).expect("write len");
    s.write_all(bytes).expect("write frame");
    s.flush().expect("flush");
}

fn read_frame(s: &mut TcpStream) -> Vec<u8> {
    let mut len = [0u8; 4];
    s.read_exact(&mut len).expect("read len");
    let mut buf = vec![0u8; u32::from_be_bytes(len) as usize];
    s.read_exact(&mut buf).expect("read frame");
    buf
}

fn exchange(s: &mut TcpStream, req: Request, id: u64) -> Response {
    write_frame(s, &req.encode(id));
    let bytes = read_frame(s);
    let (echoed, resp) =
        Response::decode(&bytes, protocol_limits(1 << 20, 1024)).expect("decode response");
    assert_eq!(echoed, id, "the daemon must echo the request id");
    resp
}

#[test]
fn the_daemon_serves_a_full_publish_exchange_over_tcp() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sparq-e2ee-ng-brokerd"))
        .args(["--listen", "127.0.0.1:0", "--quiet"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn brokerd");
    let stdout = child.stdout.take().expect("piped stdout");
    let daemon = Daemon(child);
    let mut line = String::new();
    BufReader::new(stdout)
        .read_line(&mut line)
        .expect("daemon announces its bound address");
    let addr = line
        .trim()
        .strip_prefix("listening ")
        .expect("expected `listening <addr>`")
        .to_string();

    let mut sock = TcpStream::connect(&addr).expect("connect to brokerd");

    // 1. Negotiate.
    match exchange(&mut sock, Request::Hello(hello_v0(1 << 20)), 1) {
        Response::HelloAck(a) => {
            assert_eq!(a.version, PROTOCOL_V0);
            assert_eq!(a.suite, SUITE_V0);
            assert_eq!(a.header_mode, HeaderMode::Opaque);
        }
        other => panic!("expected HelloAck, got {:?}", other),
    }

    // 2. Open a routing context under an admin-signed admission grant.
    let admin = SecretSigningKey::generate();
    let publisher = SecretSigningKey::generate();
    let topic = TopicId::random();
    let mut grant = AdmissionGrant {
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
    grant.sign(&admin).expect("sign grant");
    assert!(matches!(
        exchange(
            &mut sock,
            Request::OpenRepo(OpenRepo {
                overlay: OverlayId::random(),
                topic,
                epoch: Epoch(0),
                peer: PeerId::random(),
                auth: Some(grant),
            }),
            2,
        ),
        Response::Ok
    ));

    // 3. Upload an opaque block the daemon cannot read.
    let k_read = Secret32::random();
    let ctx = BlockContext {
        repo: RepoId::random(),
        branch: BranchId::random(),
        epoch: Epoch(0),
        kind: ObjectKind::Commit,
    };
    let env = seal_block_random(&k_read, &ctx, &ObjectId::random(), 0, 1, b"opaque commit")
        .expect("seal");
    assert_eq!(
        exchange(
            &mut sock,
            Request::BlocksPut {
                envelopes: vec![env.clone()],
            },
            3,
        ),
        Response::Stored {
            stored: 1,
            duplicate: 0
        }
    );

    // 4. Announce it.
    let mut p = PublishEvent {
        topic,
        epoch: Epoch(0),
        commit_id: env.commit_id(),
        root_block_id: env.block_id,
        publisher_key_id: [0u8; 32],
        parents: vec![],
        signature: None,
    };
    p.sign(&publisher);
    match exchange(&mut sock, Request::PublishEvent(p.clone()), 4) {
        Response::Published { commit_id, cursor } => {
            assert_eq!(commit_id, p.commit_id);
            assert_eq!(cursor, 1);
        }
        other => panic!("expected Published, got {:?}", other),
    }

    // 5. Fetch it back byte-identically.
    match exchange(
        &mut sock,
        Request::CommitGet {
            commit_ids: vec![p.commit_id],
        },
        5,
    ) {
        Response::Commits { found, missing } => {
            assert!(missing.is_empty());
            assert_eq!(found.len(), 1);
            assert_eq!(found[0].encode(), env.encode());
            // What came back is still ciphertext: only a K_read holder can open it.
            assert_eq!(
                sparq_e2ee_ng::envelope::open_block(&k_read, &ctx, &found[0]).expect("open"),
                b"opaque commit"
            );
        }
        other => panic!("expected Commits, got {:?}", other),
    }

    drop(daemon);
}
