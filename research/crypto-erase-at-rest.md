<!-- [OPUS-4.8] sq-du24 — Investigation/design record: optional crypto-erase / at-rest
     encryption primitives for sparq. DESIGN ONLY (no crates/ changes; a WAL-vacuum erasure
     PR #324 is in flight on sparq-core — this doc avoids touching that path). NON-CANONICAL
     timing; no measured numbers. Re-review when Fable returns. -->

# Crypto-erase / at-rest encryption — design investigation (sq-du24)

> Model: Opus 4.8 (Fable unavailable — flag for re-review/upgrade when Fable returns).
> Design record for bead **sq-du24**. This is **investigation/design only** — it proposes no
> code in `crates/`. It is the complement to the WAL-vacuum erasure work (bead **sq-x32t**,
> PR #324) and to the operator runbook (bead **sq-toze.33**,
> [`../compliance/privacy/retention-erasure-runbook.md`](../compliance/privacy/retention-erasure-runbook.md)).
> [OPUS-4.8]

## TL;DR

`vacuum()` (PR #324, bead sq-x32t) physically purges orphaned literal bytes + dead dictionary
entries from the **on-box** persisted store after a logical `DELETE`/`DROP`. It closes the
"plaintext residue in our own files" gap. It **cannot** reach bytes that have already left the
box — filesystem/block snapshots, COW reflinks, replicas, backups, RAID rebuilds — because the
engine has no handle on those copies.

**Crypto-erase** (cryptographic shredding) is the complement: encrypt data at rest under a key,
and "erase" by destroying the key, rendering *every* copy of the ciphertext — on-box and off-box
— permanently unrecoverable, without ever touching the off-box copies. It is the only erasure
primitive that scales to copies the engine cannot see.

**Recommendation: pursue Option A (per-graph / per-tenant key-wrapped encryption of the on-disk
segments + dictionary) as an opt-in `sparq-crypto-erase` crate / `at-rest-encryption` feature** —
but only with eyes open about the two hard limits below. Options B and C are rejected as the
*primary* mechanism (B's per-record overhead buys granularity sparq already gets from named
graphs; C gives no selective erasure). Option D (LUKS/FDE) is **complementary and recommended
regardless**, but is structurally incapable of *selective* subject-erasure on its own.

**The two honesty anchors, stated up front:**

1. **The key-copy problem.** Crypto-erase only erases data if the key was *never copied off-box
   either*. A key escrowed to a backup, replicated to a KMS that itself snapshots, or paged to
   an unencrypted swap is as recoverable as plaintext. Destroying *a* copy of the key is not
   destroying *the* key. This bounds the guarantee to "as good as the operator's key-custody
   discipline" — which the engine cannot enforce.
2. **It does not protect a live process.** Crypto-erase is a data-**at-rest** + **erasure-
   assurance** primitive. While sparq is running, plaintext (or the working key) is in process
   memory and is served to any caller who passes the B3 access boundary. It is orthogonal to
   confidentiality-in-use (that is the ZK/MPC estate's job) and to access control (`sparq-solid`).

## 1. Problem framing — what vacuum closes, and the gap it leaves

### What `vacuum()` (PR #324) achieves

After a logical `DELETE`/`DROP`/`CLEAR`, the persisted store still holds the superseded bytes
until compaction: the per-graph write-ahead log is append/replay (an earlier `INSERT` segment
survives a later `DELETE` delta), and dead dictionary entries / orphaned literal bytes linger in
the mmap'd segments. `vacuum()` is an erasure-grade compaction that physically rewrites the
on-disk store so those orphaned bytes and dead dictionary entries are *gone from sparq's own
files*. This is the right primitive for "make a logical delete physically complete on this disk".

The persisted on-disk surfaces it operates over (today, behind the `mmap` feature):

- the per-graph **WAL** (`crates/sparq-server/src/main.rs` — appended + fsync'd before ack,
  replayed on restart);
- the **mmap'd term dictionary** — `dict-meta.bin` header + the concatenated record/blob files
  (`crates/sparq-core/src/dict.rs`, `dictspill.rs`);
- the **permutation indexes** `perm{i}.bin` and `predstats.bin`
  (`crates/sparq-core/src/store.rs`).

### The gap vacuum structurally cannot close

`vacuum()` only rewrites the files sparq has a handle to. It is blind to every copy made *below*
or *beside* the engine:

- **filesystem / block-level snapshots** (LVM, ZFS, btrfs, EBS snapshots) taken before the
  delete still contain the old bytes;
- **COW reflinks** (`cp --reflink`, btrfs/XFS) that shared extents with the now-rewritten file;
- **backups and replicas** of the persist directory;
- **RAID/SSD wear-levelling remnants** and unmapped flash blocks (an overwrite is not guaranteed
  to land on the same physical cell);
- anything an operator copied off-box for ops/DR.

The retention-erasure runbook is explicit about this (§7b/§7c): "any backups, replicas, or
filesystem snapshots … will retain the subject's data until those backups expire / are rotated",
and "sparq has no built-in crypto-erase". That residue is precisely what crypto-erase addresses.

### Where crypto-erase complements vacuum (the regulatory framing)

- **GDPR Art. 17 (erasure) + Art. 32(1)(a–b) (security of processing).** A "complete" Art. 17
  erasure must reach backups (Art. 17(1)/(2)). With plaintext-at-rest the only honest answer is
  "live now; backups on next rotation, max <window>". With crypto-erase the answer becomes
  "destroy the key now; every ciphertext copy — including unrotated backups — is dead on key
  destruction", which is a *materially stronger and faster* erasure assurance. NIST SP 800-88
  recognises "Cryptographic Erase (CE)" as a sanitisation technique on exactly this basis.
- **EU CRA (Cyber Resilience Act), Annex I I.2(c)** — "protect the confidentiality of stored …
  data (e.g. encryption at rest)". sparq's CRA control table (`compliance/cra/controls.md`,
  I.2(c)) currently records "no data-at-rest encryption in the engine (operator disk-level
  concern)". An opt-in at-rest-encryption feature would let the *engine* offer a control here
  rather than deferring it entirely, without changing the default posture.
- **Privacy control P-9** (`compliance/privacy/controls.md`) — "Confidentiality of stored
  personal data (Art. 32)" is today **OPERATOR**-owned with "no engine-side at-rest encryption".
  This design does not change that default (P-9 stays operator-owned), but it gives the operator
  an *engine-native* option for the selective-erasure case that FDE cannot serve (see §2d).

The boundary is firm: **the deploying organisation remains the GDPR controller / CRA
manufacturer-operator.** This design proposes an engine *mechanism*, never a compliance claim.

## 2. Design options (with honest trade-offs)

All four assume the threat model of §4 (data-at-rest + erasure-assurance; **not** live-process
confidentiality). "Erase" throughout means "destroy the key such that ciphertext is
computationally unrecoverable", i.e. cryptographic shredding.

### Option A — per-graph / per-tenant key-wrapped segment + dictionary encryption (RECOMMENDED)

Encrypt the on-disk segments belonging to a graph/tenant under a **data encryption key (DEK)**;
wrap each DEK under a **key-encryption key (KEK)**; store wrapped DEKs in a small keystore
alongside the store. Erase a graph/tenant by destroying its DEK (delete the wrapped DEK and any
in-memory copy) — the ciphertext segments become unrecoverable even off-box.

- **Granularity.** Per named graph (or per tenant = a set of graphs). This aligns *exactly* with
  the runbook's recommended deployment pattern (§6: "partition per-data-subject data into a
  dedicated named graph"), so `DROP GRAPH <subject>` + DEK-destroy is a single, scoped,
  crypto-grade erasure. This is the sweet spot: selective erasure at the granularity sparq
  already encourages, without per-record bookkeeping.
- **What gets encrypted.** The per-graph WAL segments and the graph's slice of the persisted
  index. The **dictionary is the subtle part**: sparq's `Dict` is *cross-graph shared* (term ids
  are global). A literal `"alice@example.org"` interned for graph G may also be referenced by
  graph H. Encrypting "graph G's dictionary entries" is therefore ill-defined for shared terms.
  Two honest sub-options:
  - **A1 (clean, recommended): per-graph dictionary partitioning under crypto-erase.** Give each
    crypto-erase graph its *own* dictionary partition (no cross-graph term sharing for those
    graphs). The cost is dictionary-dedup is lost *across* erasable graphs (a term used in two
    tenants is stored twice) — acceptable, because crypto-erase tenants are exactly the case
    where you *want* isolation, and a shared dictionary entry is itself a residue leak (its mere
    presence reveals the tenant once held that literal).
  - **A2 (rejected): encrypt the shared dictionary under a store-wide key.** Then per-graph key
    destruction does *not* erase the literal bytes (they live under the store key), so selective
    crypto-erase is defeated. A2 collapses to Option C.
- **Trade-offs.** Encryption/decryption on every read/write of an encrypted graph's segments (see
  §4 cost). Loses cross-tenant dictionary dedup (A1). Adds a keystore + KEK custody burden. The
  mmap fast path (`MappedDict`, B5) must decrypt-on-map or decrypt-page-on-fault — see §3
  integration. **This is the only option that delivers selective, crypto-grade subject-erasure
  that reaches off-box copies.**

### Option B — per-record (per-triple / per-literal) encryption (rejected as primary)

Encrypt each record (or each literal blob) under its own key or a key derived per-record.

- **Pro:** finest granularity; can crypto-erase a single triple without touching its graph.
- **Con (decisive):** the overhead is severe — a key/nonce per record, encryption on every term
  resolution (the dictionary `term(id)` path is the hottest path in the engine), and the
  permutation indexes become opaque to range scans if the *ids* are encrypted (you cannot do the
  sorted-column scan that `store.rs` relies on over ciphertext without an order-preserving or
  searchable scheme, which leaks order / is weaker). You would either encrypt only the *values*
  (leaving the graph structure — who-relates-to-whom — in plaintext, often the sensitive part) or
  encrypt structure too and lose the index. Granularity finer than a named graph is not something
  sparq's erasure surface needs: the runbook already reduces subject-erasure to `DROP GRAPH`.
  **Rejected**: it pays a large, hot-path, structural cost for granularity Option A already
  provides via named graphs.

### Option C — full-store at-rest encryption under one key (rejected for the erasure goal)

Encrypt the entire persisted store (all segments, the whole dictionary, all indexes) under one
store-wide DEK.

- **Pro:** simplest; one key; can encrypt-on-write / decrypt-on-read at the segment-IO layer with
  no schema awareness; covers the *confidentiality* angle (CRA I.2(c), P-9) cheaply.
- **Con (decisive for sq-du24):** **no selective erasure.** Destroying the one key erases *the
  entire store*, not a subject. That is "decommission the whole deployment", not "fulfil Alice's
  Art. 17 request". It is genuinely useful for whole-store decommissioning / device-retirement
  crypto-erase, but it does **not** answer the data-subject-erasure problem this bead exists for.
  **Rejected as the primary mechanism**, but note it is essentially a degenerate Option A with a
  single graph, and could be offered as a low-effort first phase (see §5).

### Option D — filesystem / OS-level encryption (LUKS / dm-crypt / fscrypt / SED) (complementary)

Rely on full-disk or per-directory OS encryption beneath sparq.

- **Pro:** zero engine code; mature, audited, often FIPS-validated; protects *all* on-box files
  (WAL, dict, indexes, logs, swap) transparently; the right answer for **device-loss / stolen-
  disk** confidentiality and for **whole-volume decommissioning crypto-erase** (destroy the LUKS
  master key → the volume is shredded). The runbook already recommends it (§7c).
- **Con (why it is insufficient for *selective* erasure):** the granularity is the *volume* (or,
  with fscrypt, the *directory tree*), not the *subject*. One LUKS key protects everything on the
  volume; you cannot destroy "Alice's key" because there isn't one — there is the volume key. To
  crypto-erase one subject under FDE you would have to re-encrypt the whole volume under a new key
  after removing Alice's plaintext, which is exactly the heavyweight re-seed the runbook describes
  (§7a) — not a key-destruction shortcut. fscrypt per-directory keys get you to per-tenant *if*
  the operator lays out one directory tree per tenant and manages the keys — but that is an
  operator deployment + key-management feat outside the engine, and it still cannot follow sparq's
  *named-graph* erasure granularity. **Verdict:** D is necessary-and-recommended for at-rest
  confidentiality and whole-volume shredding, and should be documented as the baseline; it is
  **not a substitute** for engine-aware per-graph selective crypto-erase (Option A).

### Key management (cross-cutting — the part that decides whether any of this works)

- **Where keys live.** Two-tier (DEK wrapped by KEK) is standard and correct. DEKs live wrapped
  in the keystore next to the store; the KEK lives **outside** the store — ideally in an operator
  KMS / HSM / OS keyring, *never* in the persist directory or its backups. The engine should
  accept a KEK from an injected provider (env-supplied key, a KMS-unwrap callback, an OS-keyring
  handle), and must **never persist the KEK** and **never write a DEK unwrapped**.
- **Rotation.** Rotate the KEK by re-wrapping DEKs (cheap — no data re-encryption). Rotate a DEK
  by re-encrypting that graph's segments (expensive — full graph rewrite, naturally folded into a
  `vacuum`/compaction pass). KEK rotation should be routine; DEK rotation rare.
- **The off-box-key-copy problem (the load-bearing limit).** Crypto-erase reduces "erase the
  data" to "erase the key". That is only a win if **the key has a *smaller, more controllable*
  footprint than the data** and was never copied where the data wasn't. If the operator escrows
  KEKs into the same backup system that holds the ciphertext, key-destruction is theatre — the
  backup can be decrypted. The engine **cannot enforce** operator key custody; it can only (a)
  refuse to persist keys unwrapped, (b) zeroize in-memory key material on erase/drop
  (`zeroize`-style), and (c) **document loudly** that the guarantee is "no stronger than your KEK
  custody discipline". This must be the headline caveat in any SKILL.md/README we ship.

## 3. Opt-in architecture — feature-gated crate, core stays lean

Per the opt-in-feature constraint (AGENTS.md: "the core crates `sparq-core`, `sparq-engine` must
stay dependency-free of the opt-in capability crates, and the wasm build must not regress") and
the precedent set by `sparq-zk` / `sparq-mpc` (`publish = false`; nothing in the workspace
depends on them; the byte-identical wasm gate is untouched), at-rest encryption must be **opt-in
and isolated**, not a core dependency.

**Shape:** a new opt-in crate `sparq-crypto-erase` (mirrors `sparq-zk`'s manifest posture:
`publish = false`, no reverse dependency from core/engine/wasm), exposing an at-rest codec the
persistence path can call *through a seam* rather than a hard dependency. The integration seam is
the key design constraint, because core must not gain a crypto dependency.

Two seam designs, in preference order:

1. **A `SegmentCodec` trait seam in the persistence layer (recommended).** `sparq-core`'s
   on-disk IO (the `save`/`open` paths in `store.rs`, `dict.rs`, `dictspill.rs`, and the server
   WAL writer) gains a *narrow, crypto-free* trait — e.g. `trait SegmentCodec { fn seal(&self,
   plaintext) -> bytes; fn open(&self, bytes) -> plaintext; }` — defaulting to an identity codec
   (no behaviour change, no new dependency, default build byte-identical). `sparq-crypto-erase`
   provides an AEAD implementation (AES-256-GCM or XChaCha20-Poly1305 via a vetted crate; nonce
   per segment; AAD binds graph-id + segment-id + format-version to prevent swap/replay). The
   operator wires the codec in at construction. **This keeps the crypto out of core entirely; the
   seam is a pure-Rust trait with an identity default.** *(Note: the trait seam itself touches
   core's IO signatures — that is the one core change a future implementation phase needs, and it
   must be sequenced* after *PR #324's vacuum lands to avoid the in-flight conflict this design
   doc was told to avoid. This doc proposes it; it does not implement it.)*
2. **A wrapping `EncryptedStore` in the opt-in crate (lower-risk first step).** `sparq-crypto-
   erase` wraps the persist directory IO entirely — it owns reading/writing the segment files and
   hands `sparq-core` already-decrypted bytes / takes plaintext to seal. No core signature change
   at all, at the cost of the encrypted path not sharing core's mmap fast path (it must
   decrypt-to-buffer rather than mmap-and-go). Good for a Phase-1 full-store option (Option C);
   too coarse for the mmap'd per-graph fast path of Option A.

**Integration points with persistence + vacuum:**

- **Write path:** `Graph::save` / WAL append seal each segment before fsync. The DEK is resolved
  per graph from the keystore (KEK-unwrapped on open, held zeroizable in memory).
- **Read/open path:** `Store::open` / `Dict::open_mmap` either decrypt-on-map (small graphs) or
  decrypt-page-on-fault (large graphs — preserves the larger-than-RAM property; this is the
  hard engineering bit and interacts with the B5 untrusted-mmap loader: an AEAD tag check
  *strengthens* B5 because a tampered ciphertext fails authentication before it reaches the
  parser, but it removes the zero-copy mmap benefit for encrypted graphs).
- **Vacuum path (the natural home):** `vacuum()` already rewrites segments. DEK *rotation* and the
  "physically drop a crypto-erased graph's files" step fold cleanly into the same compaction pass:
  vacuum becomes the place where (a) a dropped graph's ciphertext files are unlinked and (b) a
  rotated DEK re-encrypts. **Crypto-erase and vacuum are complementary and share machinery** —
  which is the architectural reason to sequence this *after* #324, not against it.
- **Keystore:** a small `keystore.bin` (wrapped DEKs + KEK-id + algorithm/version metadata) in the
  persist dir. Wrapped DEKs only; the KEK never lands here.

**Server surface (later phase):** flags like `--at-rest-encryption` + a KEK source
(`--kek-from-env` / `--kek-from-kms <endpoint>` / OS keyring), and an admin
`crypto-erase-graph <iri>` that destroys the DEK + unlinks ciphertext (the crypto-grade analogue
of `DROP GRAPH` + `vacuum`). All opt-in; default server posture unchanged.

## 4. Honest assessment

### Performance cost

- **Read path:** AEAD decrypt on every segment read. For the **mmap fast path this is the real
  regression** — encrypted graphs lose zero-copy mmap (decrypt-on-fault buffers pages), so the
  "open is just mmap, nothing big resident" property (`dict.rs` `MappedDict`) is forfeited for
  encrypted graphs. Hot `term(id)` lookups pay per-page decrypt amortised by an OS/page cache.
- **Write path:** AEAD encrypt + fresh nonce per segment on every WAL append / compaction.
  Hardware AES-NI / vectorised ChaCha makes the raw cipher cheap relative to fsync, so the WAL
  ack-latency hit should be modest; the index/dictionary rewrite cost is the larger item and is
  naturally bounded by how often you compact.
- **No numbers here** (non-canonical environment; per repo policy). Any phase that ships must gate
  against the perf ratchet (`bench/perf-baseline.json`) on default (unencrypted) builds — the
  feature must be **zero-cost when off** (identity codec ⇒ byte-identical default build, the same
  bar `sparq-zk` meets for wasm).

### Threat model — what it does and does not cover

- **Covers:** confidentiality of data **at rest** (stolen disk / leaked backup / snapshot can't be
  read without the key) and **erasure assurance** (key-destruction shreds *all* ciphertext copies,
  including off-box ones vacuum can't reach). This is the precise gap §1 identifies.
- **Does NOT cover (state these plainly):**
  - **A live running process.** While sparq runs, plaintext and the working key are in process
    memory and served to anyone past the **B3** access boundary (no per-user auth by default).
    Crypto-erase is orthogonal to access control (`sparq-solid`) and to confidentiality-in-use
    (the ZK/MPC estate). It is not a defence against a compromised live host with the key resident.
  - **The off-box key.** As in §2 — if the KEK was copied where the ciphertext was, the guarantee
    is void. The engine cannot enforce key custody.
  - **Metadata / traffic / side channels.** Ciphertext sizes, segment counts, access patterns, and
    (with cross-tenant dictionary sharing — why A1 over A2) the mere presence of a term can leak.
    `--verbose` request logs (runbook §7d) are *not* covered by store encryption and remain a
    separate operator concern.

### Residual / out of scope

- **Key escrow / HSM / KMS integration is an operator concern.** The engine should accept an
  injected KEK provider and refuse to persist keys unwrapped; it should *not* ship its own KMS.
  Cryptographic-module validation (FIPS 140-3) is explicitly out of scope — sparq makes no FIPS
  claim (`compliance/cryptoreview/fips-posture.md`); an AEAD here is Tier-B bespoke-integration,
  and an operator needing a validated module must supply one (e.g. OS/HSM-backed). This must be
  said in the docs, consistent with the existing crypto-review posture.
- **Algorithm choice** (AES-256-GCM vs XChaCha20-Poly1305), nonce/AAD discipline, and the vetted
  crate selection are deferred to the implementation phase and should pass a `cargo-deny` /
  supply-chain review; no bespoke cipher.

## 5. Recommendation + phased plan

**Pursue Option A (per-graph/per-tenant key-wrapped segment + dictionary encryption, sub-option
A1) as an opt-in `sparq-crypto-erase` crate**, sequenced strictly **after PR #324 (vacuum)
lands** so it builds on — rather than conflicts with — the compaction machinery it shares. Adopt
Option D (FDE/LUKS) as the documented baseline regardless. Reject B (per-record) and A2/C as the
*primary* selective-erasure mechanism, while noting C is a cheap first phase for the whole-store /
device-decommission case.

Phased plan (each phase → a future bead the orchestrator can create):

1. **Phase 1 — `SegmentCodec` seam in `sparq-core`/`sparq-server` persistence, with an identity
   default (no crypto).** Land the narrow trait seam through `store.rs`/`dict.rs`/`dictspill.rs`/
   WAL writer; default identity codec ⇒ byte-identical default build + green perf ratchet + green
   B5 mmap. Sequenced after #324. (This is the one core change the feature needs; isolate it.)
2. **Phase 2 — `sparq-crypto-erase` crate: full-store AEAD codec (Option C degenerate case).**
   `publish = false`, no reverse dep from core/wasm. AES-256-GCM / XChaCha20-Poly1305 via a vetted
   crate; KEK-from-env provider; `--at-rest-encryption` server flag. Delivers at-rest
   confidentiality (CRA I.2(c) / P-9 option) + whole-store decommission crypto-erase. Lowest risk.
3. **Phase 3 — per-graph DEKs + keystore (Option A1) for selective crypto-erase.** Per-graph DEK
   wrapped under KEK in `keystore.bin`; per-graph dictionary partitioning (A1); admin
   `crypto-erase-graph <iri>` = destroy DEK + unlink ciphertext + zeroize in-memory key. Wire DEK
   rotation + dropped-graph cleanup into the `vacuum()` pass. This is the bead-du24 payoff.
4. **Phase 4 — KEK-provider integrations (KMS / OS-keyring / HSM-unwrap callback) + key rotation
   ops + zeroization audit.** Injected providers only; engine never owns a KMS. Document the
   off-box-key-copy limit + the FIPS-out-of-scope posture as headline caveats.
5. **Phase 5 — docs + compliance wiring.** Update the retention-erasure runbook (§7c) from "no
   built-in crypto-erase / operator-only" to "opt-in engine crypto-erase available (with its
   limits)"; flip CRA I.2(c) and privacy P-9 to record the engine option; add a
   `skills/<surface>/SKILL.md` for the public surface; ensure the off-box-key-copy +
   live-process-not-covered caveats are unmissable.

Each phase is independently shippable and gated by the standard merge bar (clippy + tests +
conformance + perf ratchet, default build byte-identical). Phases 1–2 are low-risk and could be
done early; Phase 3 is the substantive selective-erasure deliverable and depends on Phases 1–2
*and* on #324 having landed.

## References

- [`../compliance/privacy/retention-erasure-runbook.md`](../compliance/privacy/retention-erasure-runbook.md)
  — §7a (WAL re-seed / vacuum), §7b (backups), §7c ("no engine-side crypto-erase"; sq-du24 named).
- [`../compliance/privacy/controls.md`](../compliance/privacy/controls.md) — P-3/P-4 (erasure),
  P-9 (at-rest confidentiality, OPERATOR-owned).
- [`../compliance/cra/controls.md`](../compliance/cra/controls.md) — I.2(c) (encryption at rest).
- [`../compliance/cryptoreview/fips-posture.md`](../compliance/cryptoreview/fips-posture.md) —
  no-FIPS-claim posture (bounds any cipher we ship to Tier-B / operator-supplied-module).
- [`../compliance/threat-model.md`](../compliance/threat-model.md) — B3 (no-auth access boundary;
  why crypto-erase doesn't protect a live process), B5 (untrusted mmap loader; AEAD strengthens it
  but removes zero-copy for encrypted graphs).
- `crates/sparq-core/src/{store.rs,dict.rs,dictspill.rs}` — the persisted on-disk surfaces
  (`perm{i}.bin`, `predstats.bin`, `dict-meta.bin` + blobs) an at-rest codec must wrap.
- `crates/sparq-server/src/main.rs` — the per-graph WAL (append + fsync + replay) persistence
  model.
- `crates/sparq-zk/Cargo.toml` — the opt-in-crate posture this design mirrors (`publish = false`;
  no reverse dependency from core/wasm).
- Bead **sq-x32t** / PR #324 — `Graph::vacuum()` (the on-box complement this design builds on).
