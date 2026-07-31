<!-- [SONNET-4.6] sq-lhcot.2 (issue #2789) — the sparq side of the external-key `.spqv`
interoperability profile being co-designed with Kern/PSS on GitHub #1746. This document plus the
byte-level corpus in `crates/sparq-vectors/tests/fixtures/external-key/` IS the deliverable: the
boundary that #1746 freezes. It is a PROPOSAL until that freeze — see §1. -->

# Profile — external-key `.spqv` interoperability (DRAFT, version 0)

**Bead:** `sq-lhcot.2` (issue #2789). **Parent:** `sq-lhcot` / issue #2581.
**Routed surface:** `crates/sparq-vectors`. **Coordination:** GitHub #1746 (Kern/PSS).
**Status:** **DRAFT — NOT FROZEN.** Profile version **0** means exactly that. The frozen profile
will carry a version `>= 1`, which this implementation *rejects* rather than mis-parsing.

**Artifacts this document is the specification for:**

| artifact | what it is |
|---|---|
| `crates/sparq-vectors/src/external_key.rs` | the reference codec + fail-closed parser (opt-in `external-key` feature, off by default) |
| `crates/sparq-vectors/tests/fixtures/external-key/*.bin` | the cross-repository byte-level corpus — 5 positive, 13 negative |
| `.../external-key/MANIFEST.tsv` | the machine-readable expectation table a non-Rust implementation reads |
| `.../external-key/generate.py` | an **independent** stdlib-Python encoder that wrote the corpus |
| `crates/sparq-vectors/tests/external_key_profile.rs` | the conformance runner over the corpus |

## 1. What is frozen, what is proposed, and who owns which half

The bead is explicit that sparq is a **consumer and format co-designer** and **must not invent a
competing concept-hash scheme**. That line is drawn here as an ownership split, and the
implementation is built so the split is structural rather than a promise:

| owned by Kern/PSS — sparq does **not** implement | owned by sparq — this bead |
|---|---|
| the key-derivation (concept-hash) algorithm | nothing: keys enter as opaque bytes and are never derived, recomputed, or validated for meaning |
| which multihash code(s) a deployment uses | storing the declared code and refusing a lookup under any other |
| the signature scheme over a published table | round-tripping an opaque signature area, verifying nothing |
| the authoritative production corpus | this corpus, plus running against theirs when it lands |
| the container's placement inside a wider ecosystem | the `.spqv`-side container layout, ordering, and parse rules proposed below |

Everything in §2–§10 is a **proposal to #1746**, not an agreed decision. Nothing produced under
version 0 carries a compatibility promise. The parser rejects version `>= 1` precisely so that a
file written against the eventual frozen profile fails loudly here instead of being read under
draft rules — the freeze is therefore a visible event, not a silent drift.

*Verification limit, stated honestly:* the live state of #1746 was **not** queried — this work runs
under an orchestration contract that forbids GitHub API calls — so this document cannot and does not
claim Kern has agreed to any of it. It claims only to be sparq's side, written down in enough detail
to be argued with.

## 2. The problem external keys solve (and the one they do not)

A `.spqv` is keyed by the build-time **dictionary id**. That ties a served store to the exact
persisted graph generation it was built against: a logically identical re-parse can permute the ids,
and the order-independent fingerprint (`crates/sparq-vectors/src/fingerprint.rs`, `sq-xhiv`) folds
the term *set*, so it **passes** a re-parse whose ids merely permuted. `check_graph` is therefore a
backstop, not a sufficient guard — the id-keyed staleness contract in `crates/sparq-vectors/src/store.rs`
(`sq-wlzi`) spells this out, and `tests/staleness_contract.rs` demonstrates the trap.

An external key names the *identity*, not the id, so it survives a re-parse. What it does **not**
survive — and must not be allowed to appear to survive — is a change of **embedding space**. Hence
§9: the table binds the embedding provenance.

## 3. Key type and length

- A key is a **multihash digest**: an opaque byte string produced by the key producer. sparq never
  computes one.
- A table declares **one** multihash code and **one** digest length in its header; every entry uses
  them. This is what makes entries fixed-width, which is what lets a future mmap reader binary-search
  the section with no offset table. A deployment needing two codes publishes two tables.
- Length bound: `1..=64` bytes (`MAX_EXTERNAL_KEY_LEN`). 64 admits the longest digests in practical
  use and bounds the per-entry width a corrupt header can claim. A declared length of 0 or above the
  cap is a parse error, checked before any allocation.
- Factoring `(code, length)` into the header does **not** weaken the identity: `lookup` compares the
  code and the length *before* the digest, so a record declaring a weaker hash cannot match a digest
  stored under a stronger one. This is the substitution guard, and it is a hard error rather than a
  miss — "absent" and "unverifiable" must be distinguishable answers.

**Open question K1.** Should a table be permitted to carry mixed codes (a per-entry varint prefix,
losing fixed-width entries), or is one-code-per-table acceptable? sparq proposes one per table.

## 4. Multibase / multihash normalization

- **Comparison is on bytes, never on text.** A key received in any multibase text form is decoded to
  its binary multihash before it is compared, so two spellings of one digest can never read as two
  keys. The binary block stores raw digests; no text form appears in it.
- `parse_multihash` decodes `<code varint><length varint><digest>` and is fail-closed on: a truncated
  varint, a **non-minimally encoded** varint (canonical encoding is required so one digest has
  exactly one binary form), a code that does not fit in `u32`, a declared length that disagrees with
  the bytes present, and trailing bytes after the digest.
- **Multibase *text* decoding is deliberately out of scope for this implementation.** sparq does not
  ship a base58/base32 decoder here; a producer hands over bytes. Recording this as a scope boundary
  rather than implementing it keeps the normalization rule (decode-then-compare) agreed without
  sparq shipping a second-guess of a published multiformats spec.

**Open question K2.** Which multihash codes may a conforming deployment use? sparq's parser is
agnostic — the code is an opaque `u32` — which is the right default for a co-designed format but
means the *policy* (e.g. "sha2-256 or stronger") has to live in the frozen profile, not the parser.

## 5. Sorted `(hash, slot)` layout

```text
offset 0   magic       b"SPQVXKEY"                        8 bytes
offset 8   version     u16 = 0 (DRAFT)                    2 bytes
offset 10  flags       u16, MUST be 0 (reserved)          2 bytes
offset 12  hash_code   u32 multicodec hash code (opaque)  4 bytes
offset 16  key_len     u32 digest length in bytes         4 bytes
offset 20  count       u64 entry count                    8 bytes
offset 28  prov_len    u32 provenance block length        4 bytes
offset 32  provenance  [prov_len] bytes (0 ⇒ absent)
           entries     count × (digest[key_len] || slot u32)   — ASCENDING by digest
           sig_len     u32
           signature   [sig_len] bytes — opaque, never verified
```

All fields are fixed-width little-endian, matching the existing `.spqv` header and the
`EmbeddingProvenance` block codec, so no new dependency enters the crate.

**Canonical order is a *format* rule, not an implementation detail.** Entries are sorted ascending by
digest bytes, so the file is a function of the logical table and not of the order a producer happened
to insert. Two implementations holding the same `(key, slot)` set, provenance and signature emit
**byte-identical** blocks — which is the property the corpus exists to pin, and which
`tests/external_key_profile.rs` asserts by re-encoding every accepted fixture and comparing bytes
against what the independent Python generator wrote.

## 6. Duplicate handling

**A duplicate key is rejected, not resolved.** One key must map to exactly one slot; accepting a
second binding would make resolution depend on scan order, which is precisely the
order-dependence external keys exist to remove. The parser enforces **strictly ascending** order,
which rejects an unsorted table and a duplicate key in the same comparison, and `insert` refuses a
duplicate at construction time so a table this build can write is always one it can re-read.

**Open question K3.** Is "reject the whole table" the right blast radius, or should a producer be
able to publish a table with a documented last-writer-wins rule? sparq proposes reject: a duplicate
in a content-addressed key space is evidence of a producer bug, and a silent winner hides it.

## 7. Lookup API

```rust
table.lookup(hash_code, digest)  -> Result<Option<u32>, String>  // slot, or a clean miss
table.lookup_multihash(bytes)    -> Result<Option<u32>, String>  // the same, over a binary multihash
```

Three outcomes, deliberately distinct: `Ok(Some(slot))` resolved; `Ok(None)` the key is genuinely
absent; `Err` the query could not be *evaluated* — a foreign multihash code, a wrong-length digest,
or a malformed multihash. Collapsing the third into the second is the failure mode this shape exists
to prevent, because a caller cannot tell a substitution attempt from a cache miss.

The reference implementation binary-searches an owned `Vec`. **No mmap index is built**, on purpose:
the bead sequences *"format test vectors and parsers BEFORE optimizing the mmap index"*, and the
fixed-width sorted layout of §5 is chosen so that optimization is later a pure read-path change with
no format consequence.

## 8. Generation semantics

The table is deliberately **not** bound to a graph fingerprint. Binding it would re-import the exact
constraint external keys remove (§2). The consequences, stated rather than hidden:

- A table remains valid across a re-parse that permutes dictionary ids.
- The `slot` values still index the **companion `.spqv`'s dense data section**, so a table is only
  meaningful beside the store it was published with. Nothing in version 0 cryptographically binds a
  table to a specific `.spqv`.

**Open question K4 — the most load-bearing one.** How is a table bound to its store? sparq's reading
is that some digest over the companion store's header + data section belongs in the frozen profile;
version 0 deliberately does not invent one, because guessing here would privilege sparq's choice
before Kern has a say. Until it is answered, a consumer must treat pairing a table with a store as an
**out-of-band** trust decision, and this implementation does not pretend otherwise.

## 9. Signatures and provenance binding

**Signatures.** The block carries a length-prefixed, **opaque** signature area with **no algorithm
defined**. This implementation round-trips the bytes and **performs no verification whatsoever** —
the accessor is named `unverified_signature()` so no caller can read presence as an integrity claim.
Choosing the scheme (and what exactly it covers) is Kern's, and is **open question K5**.

**Provenance binding.** The table optionally embeds an `EmbeddingProvenance` block — the same codec
the `.spqv` v3 header uses (`src/spqv_provenance.rs`), so no new format and no new dependency. The
reasoning is in §2: an external key is generation-independent but **not** embedding-space-independent.
A key that survives a re-parse must not also survive a change of model, model/content version, metric,
normalization, or verbalization regime, because a query embedded in a different space returns
arithmetically-defined but semantically wrong neighbours. Version 0 makes the block **optional and
records its absence honestly** (`provenance()` returns `None`) rather than defaulting to a pipeline
nobody declared.

**Open question K6.** Should provenance be **mandatory** in the frozen profile? sparq leans yes for
published tables, but making it mandatory in a draft would fail every fixture a partner produced
before reading this document.

## 10. Migration and coexistence with dictionary-ID mode

Dictionary-ID keying is **not** replaced, and this bead changes nothing about it:

- The `.spqv` container (v1/v2/v3/v4) is **byte-for-byte unchanged**. The external-key table is a
  standalone block with its own magic; it is not a new `.spqv` section, and
  `src/external_key.rs` is not wired into `VectorStore` at all.
- The `external-key` cargo feature is **off by default**, so the default build compiles none of it
  and gains no dependency.
- Both keying modes therefore coexist trivially today, because they do not touch: dictionary-ID
  resolution is the store's, external-key resolution is a table beside it. A store may be served
  id-keyed, key-keyed, or both.
- The migration path *out* of draft is the version field: a frozen table is version `>= 1` and is
  rejected by this build, so the two eras cannot be confused.

**Open question K7.** Once frozen, should the table become a `.spqv` section (one file, one open) or
stay a sidecar (independently publishable, re-usable across stores)? sparq proposes sidecar and has
implemented it that way, but this is reversible precisely because nothing is wired in yet.

## 11. The corpus, and why it is not circular

`generate.py` is a **second, independent implementation** of the version-0 writer, written from this
document rather than from `external_key.rs`. A corpus generated by the Rust encoder and checked by
the Rust parser would prove only self-consistency; it could not catch a spec ambiguity, because both
sides share one reading of it. The conformance runner therefore asserts the two encoders agree
byte-for-byte on every accepted fixture.

The corpus is 5 positive and 13 negative fixtures; `MANIFEST.tsv` records, per fixture, whether a
conforming parser must accept or reject it, the header values it must report, and — for a rejection —
a **required error substring**, so an implementation cannot pass by rejecting everything for the
wrong reason. The negative half covers: a truncated header, bad magic, a frozen-profile version, a
non-zero reserved flags field, a zero and an oversized key length, unsorted entries, a duplicate key,
a truncated entry section, an absurd entry count, trailing bytes, a corrupt provenance block, and an
oversized signature length.

Both load-bearing guards were checked by **mutation** rather than by inspection. Disabling the
strictly-ascending check makes `negative_fixtures_are_rejected_for_the_recorded_reason` fail on
`neg-unsorted-entries.bin` ("must be REJECTED but parsed"); disabling the multihash-code check in
`lookup` makes `fixture_keys_resolve_and_a_foreign_hash_code_is_refused` fail. Both were restored and
the suite re-run green.

## 12. What a Kern/PSS reviewer is being asked for

1. Rule on **K1–K7** above; K4 (store binding) and K5 (signatures) block the freeze.
2. Confirm or replace the layout of §5 and the ordering/duplicate rules of §5–§6.
3. Point at the authoritative corpus so it can be vendored alongside this one; positive fixtures
   should be reproducible by both encoders, and adversarial ones should be merged into `MANIFEST.tsv`.
4. Fix the frozen version number so the version-0 rejection path becomes the migration boundary.

Implementation beads — a store binding, a mmap-backed reader, any `VectorStore` wiring — are filed
**from** that freeze, not before it.

## 13. Related, distinct

`sq-lhcot.6` (`research/genai-urn-concept-verifier-design.md`) is the **other** seam on the same KERN
boundary: independently verifying a `urn:concept` record over `sparq-canon`. It is design-only and
blocked on the same freeze. `EmbeddingProvenance::reserved` remains opaque with no fields defined;
nothing here extends it.
