<!-- [OPUS-5] sq-tag1q.9 + issue #2548: this was a minimal stub README for a publish=false
     crate; it is now the FULL crate template, because the template's 30-line stub budget
     cannot carry the mandatory Profile-SE leakage disclosure. The full public-API surface +
     disclosure ledger still live in skills/e2ee-ng/SKILL.md. -->
# sparq-e2ee-ng

Opt-in **E2EE profile primitives** for sparq. Two disclosure postures, both
client-side, both opt-in:

- **Profile BR (default surface)** — the *capability / envelope / epoch* layer of
  the NextGraph-style E2EE-queryable design.
- **Profile SE (`se` feature, OFF by default)** — the `literal` module: an
  encrypted-**literal** codec whose values are AEAD-sealed into ordinary typed
  literals, so an untrusted server can still evaluate the *structural* SPARQL
  fragment with **no new server-side code**.

> **Internal crate — not published** to crates.io (`publish = false`).
> **No security guarantee — research-grade and externally unaudited (`sq-qhy4`).**
> Every confidentiality / integrity / authorization / revocation property is
> **designed/intended, not proven**; the v0 suite name is a placeholder pending
> external review; sync / broker / CRDT / materialize are NOT in this crate.
> <!-- privacy-claims-allow: NEGATIVE/scoped — explicitly denies any proven soundness/privacy claim; sq-qhy4 pending -->
> Encryption + key material live ONLY behind this crate: `sparq-core` /
> `sparq-engine` / `sparq-substrate` do not depend on it, and the default + wasm
> builds are byte-identical with or without it.

<!-- -->

> **Profile SE leakage — read before enabling `se`.** SE reveals the **full graph
> topology** to the server: every subject, every predicate, every IRI-valued
> object, named-graph membership, degree, co-occurrence and update dynamics — and
> predicates announce the *kind* of every hidden value. **It protects the values,
> not the shape of the user's life.** It does **not** hide structure and it does
> **not** make SPARQL run over ciphertext: only the *structural* fragment runs
> server-side, and anything touching an encrypted value is opaque and must be
> post-filtered client-side after decryption. Ciphertext length is padded to a
> bucket, but the bucket is still visible. **Equality tags are a separate,
> separately-disclosed leakage increment** (equal values ⇒ equal tags ⇒
> per-predicate value-frequency leakage) that you do **not** get by simply using
> the profile.

## 🚀 Quickstart

In-workspace and `publish = false`, so depend by path; `se` is opt-in:

```toml
sparq-e2ee-ng = { path = "crates/sparq-e2ee-ng", features = ["se"] }
```

```rust
use sparq_e2ee_ng::ids::Secret32;
use sparq_e2ee_ng::literal::{open_literal, seal_literal, EncryptedLiteral, ValueContext};

let dek = Secret32::random();                            // per-predicate DEK, client-held
let ctx = ValueContext {
    predicate: "http://xmlns.com/foaf/0.1/name",          // stays CLEARTEXT in the graph
    graph: None,                                          // default graph
    subject: Some("https://alice.example/#me"),           // position-pin the ciphertext
};
let lit = seal_literal(&dek, &ctx, "Alice", "http://www.w3.org/2001/XMLSchema#string")?;
let lexical = lit.to_lexical();                           // "se0.…"^^<urn:…#enc>
let (value, datatype) = open_literal(&dek, &ctx, &EncryptedLiteral::from_lexical(&lexical)?)?;
# Ok::<(), sparq_e2ee_ng::Error>(())
```

## ✨ Features

- Default surface: `ids`, `cbor` (fail-closed deterministic CBOR with parser
  limits), `suite`, `keyschedule`, `sign`, `wrap`, `capability`, `envelope`,
  `epoch`, golden test vectors.
- **`se`** (opt-in, OFF by default): the `literal` module — `SE_ENC_DATATYPE` /
  `SE_EQTAG_DATATYPE`, `ValueContext`, `seal_literal` / `open_literal`,
  `EncryptedLiteral` (`to_lexical` / `from_lexical` / `pad_class` / `datatype`),
  `SE_PAD_CLASSES`, and the *separately* opt-in `equality_tag` / `tags_equal` /
  `eqtag_to_lexical` / `eqtag_from_lexical` — plus `keyschedule::value_key`. Adds
  no third-party dependency, and adds **no** server-side decrypt hook (that would
  move keys server-side and end the end-to-end property).

## 📚 Learn more

- Public-API surface, quickstarts, gotchas, disclosure ledger:
  [`skills/e2ee-ng/SKILL.md`](../../skills/e2ee-ng/SKILL.md).
- Designs: [`research/e2ee-nextgraph-variant-gpt56-2026-07.md`](../../research/e2ee-nextgraph-variant-gpt56-2026-07.md)
  (BR) and [`research/e2ee-queryable-options.md`](../../research/e2ee-queryable-options.md) §3.c (SE).
- Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
