# W3C Data-Integrity `rdfc` test vectors — vendored snapshot

Canonical N-Quads documents copied **verbatim** out of the two W3C
Recommendations whose `rdfc` cryptosuites `sparq-zk`'s `vc_bridge` verifies
off-circuit.

- Sources (both W3C Recommendation, 2025-05-15 publication):
  - *Data Integrity EdDSA Cryptosuites v1.0* —
    <https://www.w3.org/TR/vc-di-eddsa/>, § B.1 "Representation:
    eddsa-rdfc-2022"
  - *Data Integrity ECDSA Cryptosuites v1.0* —
    <https://www.w3.org/TR/vc-di-ecdsa/>, § A.1 "Representation:
    ecdsa-rdfc-2019, with curve P-256"
- Fetched: 2026-07-31 (documents `last-modified: 2025-05-05`).
- License: W3C Software and Document License
  (<https://www.w3.org/copyright/software-license-2023/>).

## Files

| File | Upstream example | Published SHA-256 |
| --- | --- | --- |
| `credential.nq` | vc-di-eddsa Example 9 (byte-identical to vc-di-ecdsa Example 7) | `517744132ae165a5349155bef0bb0cf2258fff99dfe1dbd914b938d775a36017` |
| `eddsa-rdfc-2022.proof-config.nq` | vc-di-eddsa Example 12 | `bea7b7acfbad0126b135104024a5f1733e705108f42d59668b05c0c50004c6b0` |
| `ecdsa-rdfc-2019.proof-config.nq` | vc-di-ecdsa Example 10 | `3a8a522f689025727fb9d1f0fa99a618da023e8494ac74f51015d009d35abc2e` |

The two specs publish the **same** unsigned credential (the Alumni Credential),
so its canonical form is vendored once and shared by both suites; the proof
configurations differ (each names its own `cryptosuite` and
`verificationMethod`).

Each file is the RDFC-1.0 canonical N-Quads document **including its trailing
newline** — that is the exact byte string the Data-Integrity hashing step
digests, so the SHA-256 of each file as stored equals the hash published
alongside it. `sha256sum *.nq` reproduces the third column above.

The remaining scalars of each vector (the concatenated `hashData`, the raw
signature, the issuer `publicKeyMultibase`, and the `proofValue`) are short, so
they are quoted as `const`s in the consuming test next to their example number
rather than vendored as files.

## Consumed by

`crates/sparq-zk/tests/vc_bridge_w3c_vectors.rs` (behind the default-OFF
`vc-bridge` cargo feature). The suite drives the bridge's public API over these
vectors, so it witnesses **byte-level W3C interop** — sparq's RDFC-1.0
canonicalization reproduces the published canonical N-Quads, the bridge derives
the published `hashData` from them, and a third-party-issued W3C proof verifies
under the published issuer key. The crate's in-module `vc_bridge` tests sign
self-computed `hashData`, so they witness fail-closed verification but not
interop; this suite covers that gap (sq-wklnb, Opus review follow-up on #1155).

This is an **off-circuit, ingest-time** host verification only. It asserts no
in-circuit or query-soundness property, and the ZK estate is not externally
audited (sq-qhy4).

Do not edit these files; refresh by re-vendoring from a newer upstream
publication.
