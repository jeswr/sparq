---
name: e2ee-ng
description: Build the NextGraph-style E2EE-queryable profile for sparq with the opt-in sparq-e2ee-ng crate (primitives + wire protocol) and the opt-in sparq-e2ee-ng-broker crate (the opaque broker + its `sparq-e2ee-ng-brokerd` daemon) — deterministic capability encoding with strict read/publish/admin separation, recipient-wrapped secrets (X25519 sealed-box), randomized padded block/commit envelopes (XChaCha20-Poly1305) under a domain-separated HKDF-SHA-256 key schedule, Ed25519 author/publisher/admin signatures, signed epoch transitions (the revocation mechanism), a fail-closed deterministic-CBOR codec with explicit parser limits, the versioned client/broker messages (negotiation, routing, pinning, topic subscription, have/want sync, block operations, publication, epoch advance, limits, retention), metadata-safe broker logging, and golden test vectors. Use when constructing/parsing capabilities, sealing/opening encrypted blocks, minting commit ids, authoring epoch transitions, speaking the broker protocol, or running an opaque broker. RESEARCH-GRADE, NOT externally audited — every confidentiality/integrity/authorization/revocation property is designed/intended, not proven; production use is gated by sq-qhy4. The client-side sync driver, CRDT, and materialization are NOT implemented here, and none of this makes SPARQL run over ciphertext (querying stays local over decrypted, materialized state).
---

# sparq-e2ee-ng — E2EE-NG profile primitives (capability / envelope / epoch)

`sparq-e2ee-ng` implements the low-level cryptographic **primitives** of the
NextGraph-style **E2EE-queryable profile** designed in
[`research/e2ee-nextgraph-variant-gpt56-2026-07.md`](../../research/e2ee-nextgraph-variant-gpt56-2026-07.md)
(program `sq-tag1q`). In that profile an RDF dataset is a local-first repository:
encrypted content-addressed blocks carry a causal DAG of encrypted commits; an
always-on broker stores and routes *opaque* blocks but never sees a SPARQL
request. This crate is the **capability / envelope / epoch** layer that everything
else is built on.

<!-- privacy-claims-allow: The whole "Maturity" block is a NEGATIVE/scoped honesty caveat
     (explicitly denies any proven soundness/privacy claim, states designed/intended only,
     and names the sq-qhy4 gate) — not an achieved-property claim. -->
> **Maturity (read first).** RESEARCH-GRADE and **externally UNAUDITED**. Every
> confidentiality, integrity, authorization, revocation, and convergence property
> is **designed/intended, NOT proven** — the construction has had no external
> cryptographic review. The v0 suite name is a placeholder; production suite
> selection + the soundness review are gated by **`sq-qhy4`**. Nothing here
> claims cryptographic soundness or zero-knowledge, and none of it turns SPARQL
> into ciphertext-side evaluation: the profile keeps querying *local* over
> decrypted, materialized state. The design's own disclosure ledger (§5) is
> explicit that a *conforming* broker still observes topic membership,
> subscription/publication patterns, timing, sizes, and storage volume — this
> **MUST NOT** be described as hiding access patterns, membership, volume, or
> timing, and a broker is not trusted for integrity or availability. The
> client-side **sync driver, CRDT, and materialization** (design §6, §8.5) are
> still **not** implemented. Encryption + key material live **only** behind
> `sparq-e2ee-ng`: `sparq-core` / `sparq-engine` / `sparq-substrate` never link a
> cipher, and the broker crate never links the query engine (design §7).

## Quickstart

Both crates are in-workspace, `publish = false`, so depend by path. A **client**
needs only the first; a **broker** deployment adds the second (and gets the
`sparq-e2ee-ng-brokerd` daemon with it):

```toml
[dependencies]
sparq-e2ee-ng = { path = "crates/sparq-e2ee-ng" }              # capabilities, envelopes, wire codec
sparq-e2ee-ng-broker = { path = "crates/sparq-e2ee-ng-broker" } # the opaque broker (server side only)
```

Seal a block and open it (the round-trip fails closed on any wrong scope):

```rust
use sparq_e2ee_ng::envelope::{seal_block_random, open_block, BlockContext, ObjectKind};
use sparq_e2ee_ng::ids::{RepoId, BranchId, Epoch, ObjectId, Secret32};

let k_read = Secret32::random();                 // branch read secret K_read
let ctx = BlockContext {                         // repo/branch/epoch/kind are AD-bound,
    repo: RepoId::random(),                       // NOT serialized into the envelope
    branch: BranchId::random(),
    epoch: Epoch(0),
    kind: ObjectKind::Operation,
};
let object = ObjectId::random();
let env = seal_block_random(&k_read, &ctx, &object, 0, 1, b"CRDT op bytes")?;
let pt = open_block(&k_read, &ctx, &env)?;        // wrong repo/branch/epoch/chunk => Err
assert_eq!(pt, b"CRDT op bytes");
# Ok::<(), sparq_e2ee_ng::Error>(())
```

Mint the three separated capabilities and sign the public grant with an admin key:

```rust
use sparq_e2ee_ng::capability::{base_grant, Capability, Validity};
use sparq_e2ee_ng::ids::{RepoId, BranchId, Epoch, TopicId, Secret32};
use sparq_e2ee_ng::sign::SecretSigningKey;

let admin = SecretSigningKey::generate();
let publisher = SecretSigningKey::generate();
let g = base_grant(RepoId::random(), BranchId::random(), Epoch(0), TopicId::random(),
                   Validity { not_before: 0, not_after: u64::MAX }, vec!["wss://b".into()]);

let read  = Capability::new_read(g.clone(), Secret32::random())?;          // {read}
let write = Capability::new_write(g.clone(), Secret32::random(), &publisher)?; // {read,publish}
let adm   = Capability::new_admin(g, &admin)?;                             // {admin}, distinct sk_admin
read.validate()?; write.validate()?; adm.validate()?;                     // separation invariants
# Ok::<(), sparq_e2ee_ng::Error>(())
```

Talk to a broker (§8.4). Note what a request *cannot* carry — there is no field
for a read secret, a private key, RDF, SPARQL, `RepoId`, or `BranchId`:

```rust
use sparq_e2ee_ng::broker_protocol::{hello_v0, protocol_limits, AdmissionGrant, OpenRepo, Request};
use sparq_e2ee_ng::capability::Validity;
use sparq_e2ee_ng::ids::{Epoch, OverlayId, PeerId, TopicId};
use sparq_e2ee_ng::sign::SecretSigningKey;
use sparq_e2ee_ng::suite::SUITE_V0;

let admin = SecretSigningKey::generate();
let publisher = SecretSigningKey::generate();
let topic = TopicId::random();

// The BROKER-facing grant: topic/epoch/suite/public keys/validity only.
let mut grant = AdmissionGrant {
    topic, epoch: Epoch(0), suite: SUITE_V0.to_string(),
    admin_pub: admin.public().to_bytes(),
    publisher_pub: Some(publisher.public().to_bytes()),
    validity: Validity { not_before: 0, not_after: u64::MAX },
    admin_sig: None,
};
grant.sign(&admin)?;

let hello = Request::Hello(hello_v0(1 << 20)).encode(1);          // framed CBOR, request_id 1
let open  = Request::OpenRepo(OpenRepo {
    overlay: OverlayId::random(), topic, epoch: Epoch(0),
    peer: PeerId::random(), auth: Some(grant),
}).encode(2);
let (id, _req) = Request::decode(&open, protocol_limits(1 << 20, 1024))?;
assert_eq!((id, hello.is_empty()), (2, false));
# Ok::<(), sparq_e2ee_ng::Error>(())
```

Run an opaque broker in-process (the daemon is the same state machine plus a
length-prefixed CBOR TCP transport):

```rust
use sparq_e2ee_ng::broker_protocol::{hello_v0, Request, Response};
use sparq_e2ee_ng_broker::{Broker, BrokerConfig};
use sparq_e2ee_ng_broker::log::NullLog;

let mut broker = Broker::new(BrokerConfig::default(), NullLog);   // clock-free: `now` is an argument
let session = broker.open_session();
let ack = broker.handle(session, 1_800_000_000, Request::Hello(hello_v0(1 << 20)), 0);
assert!(matches!(ack, Response::HelloAck(_)));
```

```console
$ sparq-e2ee-ng-brokerd --listen 127.0.0.1:9425 --unpinned-ttl-secs 604800
listening 127.0.0.1:9425
```

## Features / public API surface

- **`ids`** — 32-byte public identifiers (`RepoId`/`BranchId`/`TopicId`/`BlockId`/
  `ObjectId`/`CommitId`/`CapId`/`AuthorKeyId`), `Epoch`, and the zeroizing
  `Secret32`. Every "random" id is drawn from a CSPRNG and is **not** derived from
  plaintext/ciphertext (no convergent-encryption leakage).
- **`cbor`** — a minimal **deterministic** CBOR codec (RFC 8949 core deterministic:
  shortest ints, definite lengths, strictly-ascending map keys) with a fail-closed
  `Reader` enforcing explicit `Limits` (max string/array/map/depth), rejecting
  non-canonical encodings + trailing bytes, and skipping negative-int extension keys
  while rejecting unknown mandatory ones.
- **`suite`** — algorithm agility with one bound v0 suite (`SUITE_V0`) and
  `check_suite`; `aead_seal`/`aead_open` (XChaCha20-Poly1305).
- **`keyschedule`** — domain-separated HKDF-SHA-256: `object_key` (binds K_read to
  repo/branch/epoch/object), `block_key`, `wrap_key`.
- **`sign`** — `SecretSigningKey` / `PublicVerifyingKey` (Ed25519) for the three
  separated authorities.
- **`wrap`** — recipient-wrapped secrets: `wrap`/`unwrap` + `WrappedSecret`
  (X25519 ECDH → wrap key → AEAD; fresh ephemeral key + nonce per wrap). This is
  ephemeral-*static* ECDH: **not forward-secret** — compromise of the recipient's
  long-term private key exposes previously recorded wraps.
- **`capability`** — `PublicGrant` (deterministic encode/decode, `cap_id`,
  admin `sign`/`verify`), `Capability` (`new_read`/`new_write`/`new_admin`,
  `validate`, `encode_secret`/`decode_secret`), and `delegate` (narrow-only).
- **`envelope`** — `BlockEnvelope` (`seal_block`/`seal_block_random`/`open_block`,
  padded to `PAD_CLASSES`, `commit_id`) and `Commit` (author `sign`/`verify`,
  `encode`/`decode`).
- **`epoch`** — `EpochTransition` (`sign`/`verify`, monotonic-epoch check,
  `HistoryPolicy::{ForwardOnly, HistoryRekeyed}`) — the revocation mechanism.
- **`broker_protocol`** — the versioned client/broker messages (§8.4): `Request` /
  `Response` framed as `{version, request_id, kind, body}` with
  `encode`/`decode` + `protocol_limits`; `Hello`/`HelloAck` negotiation
  (`HeaderMode::{Opaque, Clear}`, `WireLimits`, `RetentionPolicy`); `OpenRepo`,
  `PinRepo`/`PinStatus`, `TopicSub`/`TopicUnsub`, `TopicSyncReq`/`TopicSyncResp`
  (+ the non-authoritative `BloomHint`), `BlocksExist`/`BlocksGet`/`BlocksPut`,
  `CommitGet`, `PublishEvent`/`Event`, `EpochAdvance`; the admin-signed
  `AdmissionGrant`; and typed `BrokerError`/`ErrorCode`.

And in **`sparq-e2ee-ng-broker`** (server side; never links the query engine):

- **`broker`** — `Broker` / `BrokerConfig` / `SessionId`: `open_session`,
  `handle` (decoded) and `handle_frame` (wire), `take_pushes` for fan-out,
  `pin_admin` to pre-provision a topic's trust anchor instead of trust-on-first-use,
  `collect_garbage` / `release_retired` for retention.
- **`store`** — `BlockStore` / `StoredBlock` / `GcReport`: exact-byte idempotent
  storage, topic-scoped presence, age- and budget-driven collection.
- **`log`** — `LogRecord` / `MetadataLog` / `NullLog` / `StderrLog` / `CaptureLog`
  and `IdPrefix`: a log line with **no field** for ciphertext, plaintext, or a
  secret, and identifiers truncated to a 4-byte prefix.

## Gotchas

- **Secrets never go on the wire in the clear.** `K_read` and private keys live in
  the secret-bearing `Capability::encode_secret` output (a bearer secret) — wrap it
  with `wrap` before it leaves protected local storage. `PublicGrant::encode` (what
  a broker sees) carries **no** secret field and `PublicGrant::decode` **rejects** a
  secret field if one is present.
- **The opaque header is AD-bound, not serialized.** `open_block` needs the same
  `BlockContext` (repo/branch/epoch/kind) used to seal; those fields are
  authenticated but never appear in `BlockEnvelope` (design §5). A wrong context
  fails closed as `Error::Decrypt`.
- **Revocation is not retroactive.** An `EpochTransition` mints a fresh topic/key
  set and re-keys *future* writes; it cannot erase plaintext or keys a former member
  already obtained. `HistoryPolicy` is an honest disclosure of that, not a secrecy
  guarantee.
- **Delegation only narrows.** `delegate` rejects any child grant that widens the
  authority set, validity window, or epoch bound relative to its parent. The epoch
  bound is the child's own signed `PublicGrant::max_epoch` ceiling — the child keeps
  the parent's exact `epoch` and its epoch-specific `topic`, so a narrowed grant
  never claims an epoch that disagrees with the topic it carries.
- **Determinism is load-bearing.** `cap_id` and `CommitId` are hashes of canonical
  bytes; the golden vectors in `tests/vectors.rs` pin the exact wire format. A
  change to those bytes is a wire-format break, not a refactor.
- **The broker grant is NOT the capability grant.** `AdmissionGrant` is a
  *separately signed* object because `PublicGrant`'s signature binds `RepoId` and
  `BranchId`, which §5 hides from a conforming broker. Never send a `PublicGrant`
  to a broker.
- **A routing context is not a lease.** `OpenRepo` scopes the session's block
  operations to one topic; if an epoch advance retires that topic, later
  `BlocksPut`/`PublishEvent` fail closed with `Retired`/`EpochMismatch` rather
  than landing under the old epoch. Re-open at the new topic.
- **Parents are refused in the default header mode.** `PublishEvent::parents` may
  only be non-empty under a negotiated `HeaderMode::Clear`; sending them in the v0
  default is a `Protocol` error, because accepting them would silently widen the
  §5 disclosure ledger to the whole commit DAG.
- **`advertised_heads` is not a frontier.** In opaque-header mode the broker
  cannot see parents at all, so its advertisement is announcement-order and
  possibly incomplete — §8.4 forbids a broker from claiming completeness, and the
  client detects missing causal closure itself.
- **Pinning ≠ durability, retention ≠ erasure.** `PinRepo` exempts a topic from
  age/budget collection while the broker chooses to honour it; collection cannot
  retract anything a peer already fetched.
- **The daemon has no transport auth and no TLS.** Run `sparq-e2ee-ng-brokerd`
  behind an authenticated, encrypted transport.

## Learn more

- Design record: [`research/e2ee-nextgraph-variant-gpt56-2026-07.md`](../../research/e2ee-nextgraph-variant-gpt56-2026-07.md)
- Complementary simpler baseline (Profile CS) + threat-model handoff: design §3, §9.
- The vetted-crypto, standards-interop sibling for *signing* RDF (not confidentiality):
  [`verifiable-credentials`](../verifiable-credentials/SKILL.md).
