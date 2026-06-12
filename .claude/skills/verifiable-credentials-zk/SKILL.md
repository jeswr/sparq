---
name: verifiable-credentials-zk
description: Patterns for handling W3C Verifiable Credentials inside a zero-knowledge pipeline — signature scheme trade-offs (BBS+ vs SD-JWT-VC vs EdDSA), commitment schemes (Pedersen, Poseidon), encoding of credential graphs to circuit-friendly representations, domain-separation discipline, and the public-input contract between the credential layer and the Noir circuit. Use when choosing crypto primitives, drafting the assumptions section, or pinning the verifier interface.
---

# Verifiable Credentials in a ZK pipeline

Patterns for the cryptographic seam between W3C Verifiable Credentials
and the Noir circuits in this workspace.

## Signature scheme trade-offs

| Scheme | Native selective disclosure? | In-circuit verification cost | Ecosystem alignment | When to pick |
| --- | --- | --- | --- | --- |
| **BBS+** | Yes (per-message blinding) | High — pairing-friendly group ops are heavy in BN254 / Grumpkin | Strong (W3C VC, IRTF draft) | When selective disclosure is the dominant feature and the verifier outside the circuit can do the heavy lifting (verifying a BBS+ proof of knowledge, not the signature directly). |
| **SD-JWT-VC** | Yes (salted-hash disclosures) | Moderate — hashing only | Strong (IETF, OAuth ecosystem) | When you want JWT-shaped tokens, simple disclosures, no pairings. |
| **EdDSA** | No (sign whole document) | Low — Ed25519 verifications are cheap | Universal | When the whole credential is disclosed, or when the prover holds a separate "compiled" representation that the SPARQL evaluator commits to. |

For the paper's first version, **EdDSA + a separate prover-side
commitment** is the simplest defensible choice. BBS+ becomes
attractive once the paper has demonstrated the pipeline end-to-end.

## Commitment schemes

- **Pedersen** — additive homomorphism; classic for committing to
  field-element vectors.
- **Poseidon** — sponge-friendly, fast in Noir-style circuits.
  Default choice unless additivity is needed.
- **Merkle tree (Poseidon-leaved)** — when the circuit needs sparse
  random-access into a large committed set (e.g. selecting a few
  triples from a 10k-triple graph).
- **KZG / IPA polynomial commitment** — overkill for a paper
  prototype unless you need batched openings.

## Encoding RDF triples into field elements

A triple `(s, p, o)` becomes three field elements via a hash with
domain separation:

```
h_s = Poseidon(b"rdf-term-v1" ‖ s_bytes)
h_p = Poseidon(b"rdf-term-v1" ‖ p_bytes)
h_o = Poseidon(b"rdf-term-v1" ‖ o_bytes)
triple_hash = Poseidon(b"rdf-triple-v1" ‖ h_s ‖ h_p ‖ h_o)
```

`s_bytes` etc. come from the canonicalised serialisation chosen by
`sparql-semantics` (URDNA2015 / URDNA2024).

**Domain-separate every distinct hash use.** Maintain one table
listing every domain tag — the paper appendix.

## Public-input contract

The Noir circuit verifies that:

1. The graph commitment matches the credentials' canonicalised
   triple set.
2. The credentials' issuer signatures (or a proof-of-knowledge
   thereof) cover that triple set.
3. The query is the one the verifier asked for (committed publicly).
4. The result commitment is the multi-set the prover discloses.

Public inputs (typical):

- `issuer_pk` (or set of trusted issuer roots).
- `graph_commitment` — Poseidon root over committed triples.
- `query_commitment` — commitment to the SPARQL query string /
  parsed AST (whichever the protocol fixes).
- `result_commitment` — commitment to the disclosed result.

Private inputs include the credentials, signatures, witness paths,
and any randomness.

## Replay protection / freshness

The prover injects a **session nonce** committed publicly (e.g. a
verifier-supplied challenge) and binds it into the result commitment.
Without this, a proof is silently replayable.

## Assumptions to call out in the paper

- **Hash modelling.** Are Poseidon / Pedersen treated as random
  oracles? Algebraic group model?
- **Signature unforgeability.** Cite the EUF-CMA result for the
  chosen scheme; for BBS+, cite the specific variant
  (e.g. Au-Susilo-Mu vs more recent reductions).
- **Blank-node assumption.** State explicitly that blank-node
  semantics inside the circuit follow the canonicalisation contract,
  not the W3C "fresh blank node per scope" convention.

## Primary sources

- W3C Verifiable Credentials Data Model 2.0.
- IRTF "BBS Signatures" draft.
- IETF SD-JWT and SD-JWT-VC drafts.
- W3C RDF Dataset Canonicalisation (URDNA2015 / URDNA2024).
- Grassi et al. "Poseidon: A New Hash Function for Zero-Knowledge
  Proof Systems" (USENIX Security 2021).
