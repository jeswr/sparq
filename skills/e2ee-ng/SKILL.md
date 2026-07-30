---
name: e2ee-ng
description: Build the low-level primitives of the NextGraph-style E2EE-queryable profile for sparq with the opt-in sparq-e2ee-ng crate — deterministic capability encoding with strict read/publish/admin separation, recipient-wrapped secrets (X25519 sealed-box), randomized padded block/commit envelopes (XChaCha20-Poly1305) under a domain-separated HKDF-SHA-256 key schedule, Ed25519 author/publisher/admin signatures, signed epoch transitions (the revocation mechanism), a fail-closed deterministic-CBOR codec with explicit parser limits, and golden test vectors — plus the separately opt-in `se` feature, a Profile SE ("structure-exposed") encrypted-LITERAL codec that AEAD-seals literal values into ordinary typed literals so an untrusted server can still evaluate the structural SPARQL fragment. Use when constructing/parsing capabilities, sealing/opening encrypted blocks, minting commit ids, authoring epoch transitions, or sealing/opening encrypted literal values. RESEARCH-GRADE, NOT externally audited — every confidentiality/integrity/authorization/revocation property is designed/intended, not proven; production use is gated by sq-qhy4. This crate is the capability/envelope/epoch layer plus the SE value codec ONLY — sync, broker protocol, CRDT, and materialization are NOT implemented here, and it does NOT make SPARQL run over ciphertext (Profile BR keeps querying local over decrypted materialized state; Profile SE runs only the STRUCTURAL fragment server-side over structure that was never encrypted, and reveals the full graph topology to the server).
---

# sparq-e2ee-ng — E2EE profile primitives (capability / envelope / epoch, + opt-in Profile SE literals)

`sparq-e2ee-ng` implements the low-level cryptographic **primitives** of the
NextGraph-style **E2EE-queryable profile** designed in
[`research/e2ee-nextgraph-variant-gpt56-2026-07.md`](../../research/e2ee-nextgraph-variant-gpt56-2026-07.md)
(program `sq-tag1q`). In that profile an RDF dataset is a local-first repository:
encrypted content-addressed blocks carry a causal DAG of encrypted commits; an
always-on broker stores and routes *opaque* blocks but never sees a SPARQL
request. This crate is the **capability / envelope / epoch** layer that everything
else is built on.

It also carries one **separately opt-in** surface behind the `se` cargo feature
(OFF by default): the `literal` module, the **Profile SE** ("structure-exposed")
encrypted-literal codec from
[`research/e2ee-queryable-options.md`](../../research/e2ee-queryable-options.md)
§3.c. That is the *other* answer to "can a server query E2EE data" — literal
**values** are AEAD-sealed into ordinary typed literals, RDF **structure** stays
cleartext, and the untrusted server evaluates the *structural* SPARQL fragment
with **no new server-side code**. It is a different, weaker disclosure posture,
which is why it is a separate feature with its own leakage statement below.

<!-- privacy-claims-allow: The whole "Maturity" block is a NEGATIVE/scoped honesty caveat
     (explicitly denies any proven soundness/privacy claim, states designed/intended only,
     and names the sq-qhy4 gate) — not an achieved-property claim. -->
<!-- [OPUS-5] issue #2548 — the `scaffold-caveat` anchor is the SINGLE SOURCE for the docs
     guide's E2EE maturity caveat: book/src/getting-started/capabilities.md {{#include}}s
     exactly this region, alongside the ZK and MPC ones, so the hedges cannot drift from
     here. Keep the ANCHOR markers one-per-line (mdBook excludes only the exact marker
     line) and keep the two caveat paragraphs INSIDE it — the leakage statement is half the
     honesty, not an appendix to it. -->
<!-- ANCHOR: scaffold-caveat -->
> **Maturity (read first).** RESEARCH-GRADE and **externally UNAUDITED**. Every
> confidentiality, integrity, authorization, revocation, and convergence property
> is **designed/intended, NOT proven** — the construction has had no external
> cryptographic review. The v0 suite name is a placeholder; production suite
> selection + the soundness review are gated by **`sq-qhy4`**. This crate does
> **not** claim cryptographic soundness or zero-knowledge, and it does **not**
> turn SPARQL into ciphertext-side evaluation: the profile keeps querying *local*
> over decrypted, materialized state. It is the **primitives** layer only — the
> sync, broker-protocol, CRDT, and materialization layers (design §6, §8.4–8.5)
> are **not** in this crate. Encryption + key material live **only** behind this
> crate: `sparq-core` / `sparq-engine` / `sparq-substrate` never link a cipher.
> The `se` module changes none of that: nothing there evaluates SPARQL over
> ciphertext either, and it is equally research-grade and externally unaudited.

<!-- privacy-claims-allow: NEGATIVE/scoped — this block DISCLOSES leakage the profile does
     not prevent; it makes no achieved privacy/soundness claim (sq-qhy4 pending). -->
> **Profile SE leakage statement (read before enabling `se`).** SE reveals the
> **full graph topology** to the server — every subject, every predicate, every
> IRI-valued object, named-graph membership, node degree, co-occurrence and update
> dynamics — and because predicates come from published vocabularies, **the
> predicate announces the *kind* of every hidden value** (`foaf:name`,
> `dbo:diagnosis`). Structure alone is highly identifying. So, plainly: **Profile
> SE protects the values, not the shape of the user's life.** Do not describe it as
> hiding structure, and do not describe it as making SPARQL run over ciphertext:
> only the **structural** fragment (BGP matching + joins on
> subjects/predicates/IRI objects, property paths, OPTIONAL/UNION/MINUS and
> counting over structure) runs server-side. Anything touching an encrypted value
> — value `FILTER`, `ORDER BY`, value joins, value aggregation — is **opaque**, so
> answers come back carrying ciphertext literals the client decrypts and
> post-filters locally. Ciphertext length is padded to a bucket
> (`SE_PAD_CLASSES`), but the bucket itself is still visible. And **equality tags
> are a separate, separately-disclosed leakage increment** (`equality_tag`) — equal
> values under one predicate produce equal tags, disclosing that predicate's
> value-equality pattern and value frequency — which you do **not** get by simply
> using the profile.
<!-- ANCHOR_END: scaffold-caveat -->

## Quickstart

The crate is in-workspace, `publish = false`, so depend by path:

```toml
[dependencies]
sparq-e2ee-ng = { path = "crates/sparq-e2ee-ng" }
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

Seal a **literal value** for Profile SE (needs `features = ["se"]`; read the
leakage statement above first):

```rust
use sparq_e2ee_ng::ids::Secret32;
use sparq_e2ee_ng::literal::{open_literal, seal_literal, EncryptedLiteral, ValueContext,
                             SE_ENC_DATATYPE};

let dek = Secret32::random();                          // per-predicate DEK, client-held
let ctx = ValueContext {
    predicate: "http://xmlns.com/foaf/0.1/name",        // stays CLEARTEXT in the graph
    graph: None,                                        // None = default graph
    subject: Some("https://alice.example/#me"),         // pin the position (see gotchas)
};
let lit = seal_literal(&dek, &ctx, "Alice", "http://www.w3.org/2001/XMLSchema#string")?;

// Publish it as an ordinary typed literal — the server needs no new code:
//   <https://alice.example/#me> foaf:name "se0.…"^^<urn:…#enc> .
let lexical = lit.to_lexical();
assert_eq!(lit.datatype(), SE_ENC_DATATYPE);

// Client-side, on whatever the server handed back (fail-closed on any wrong field):
let parsed = EncryptedLiteral::from_lexical(&lexical)?;
let (value, datatype) = open_literal(&dek, &ctx, &parsed)?;
assert_eq!(value, "Alice");
# assert_eq!(datatype, "http://www.w3.org/2001/XMLSchema#string");
# Ok::<(), sparq_e2ee_ng::Error>(())
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
  repo/branch/epoch/object), `block_key`, `wrap_key`; with `se`, `value_key` (binds
  a per-predicate DEK to predicate + graph, under a distinct `e2ee-sparql` label).
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
- **`literal`** (opt-in, feature **`se`**, OFF by default) — the Profile SE
  encrypted-literal codec: the datatype IRIs `SE_ENC_DATATYPE` /
  `SE_EQTAG_DATATYPE` (non-dereferenceable draft placeholders, shared with the
  `/specs` E2EE-SPARQL draft), `ValueContext` (predicate + optional graph +
  optional subject, all AEAD-bound), `seal_literal` / `open_literal`,
  `EncryptedLiteral` (`to_lexical` / `from_lexical` / `pad_class` / `datatype`,
  canonical `se0.<nonce-hex>.<ct-hex>`), `SE_PAD_CLASSES`, and — *separately*
  opt-in — `equality_tag` / `tags_equal` / `EQTAG_LEN` / `eqtag_to_lexical` /
  `eqtag_from_lexical`. Values are sealed under a **fresh random nonce** (no
  deterministic/convergent mode) and padded before sealing; the real datatype
  travels *inside* the ciphertext. Adds no third-party dependency and adds **no**
  server-side decrypt hook.

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
  change to those bytes is a wire-format break, not a refactor. Same for the SE
  derivation vectors in `tests/literal_se.rs`.
- **SE: an unpinned subject is relocatable.** `ValueContext { subject: None, .. }`
  binds no subject, so an untrusted server can **move a ciphertext from one subject
  to another undetected** under the same predicate — a plausible wrong answer, not
  a decryption failure. That is a real, disclosed integrity limit; pin the subject
  unless you have a concrete reason not to (e.g. a blank-node subject with no
  stable IRI), and disclose it if you do not.
- **SE: equality tags are opt-in twice over.** `seal_literal` never emits one.
  Publishing tags is its own configuration decision with its own disclosure (see
  the leakage statement above), and a tag deliberately does **not** bind the
  subject — it has to be comparable across subjects to serve a join, so
  `ValueContext::subject` is ignored by `equality_tag`.
- **SE: never move a key server-side.** Do not add a decrypt UDF, engine hook, or
  server-side key escrow to "make FILTER work" — that ends the end-to-end property
  (survey §5.6). Post-filter client-side after decryption instead.
- **SE parsing is fail-closed and canonical-only.** `from_lexical` rejects
  uppercase hex, odd-length hex, a wrong nonce length, a ciphertext length that is
  not a pad class plus the AEAD tag, an unknown `se<version>` tag
  (`Error::UnknownSuite`), and any trailing field. It never normalizes.

## Learn more

- Design record: [`research/e2ee-nextgraph-variant-gpt56-2026-07.md`](../../research/e2ee-nextgraph-variant-gpt56-2026-07.md)
- Profile SE (the `se` feature) is normatively scoped by
  [`research/e2ee-queryable-options.md`](../../research/e2ee-queryable-options.md) §3.c,
  including the leakage headline reproduced above.
- Complementary simpler baseline (Profile CS) + threat-model handoff: design §3, §9.
- The vetted-crypto, standards-interop sibling for *signing* RDF (not confidentiality):
  [`verifiable-credentials`](../verifiable-credentials/SKILL.md).
