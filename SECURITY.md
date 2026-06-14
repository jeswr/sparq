<!-- [OPUS-4.8] Governance: security policy (bead sq-3pq5). -->
# Security Policy

`sparq` is an experimental research RDF triplestore and SPARQL 1.1 engine. We take
security reports seriously and welcome responsible disclosure. This document explains
how to report a vulnerability, what to expect, and — importantly — which parts of the
project carry **no security guarantee** and must not be relied on in production.

## Reporting a vulnerability

**Please report security issues privately. Do not open a public GitHub issue, PR, or
discussion for a suspected vulnerability.**

Two private channels, either is fine:

1. **GitHub Security Advisories (preferred).** Open a private report via the repository's
   **Security → Advisories → "Report a vulnerability"** page
   (<https://github.com/jeswr/sparq/security/advisories/new>). This keeps the report,
   discussion, and fix coordination private until a coordinated disclosure.
2. **Email.** Write to **jesse@jeswr.org** with a clear subject line (e.g.
   `[sparq security] <short summary>`). If you want to encrypt, say so in a first
   contact email and we will arrange a key.

Please include enough to reproduce: affected crate/surface and version (or commit), a
description of the issue and its impact, and a minimal proof-of-concept or steps where
possible.

## What to expect (response expectation)

`sparq` is maintained on a best-effort basis by a small team. We aim to:

- **Acknowledge** your report within **5 business days**.
- Provide an **initial assessment** (severity, whether we can reproduce, likely
  remediation path) within **10 business days**.
- Keep you updated as we work on a fix and **coordinate a disclosure timeline** with you.

We will credit reporters in the advisory unless you ask us not to. These are targets,
not contractual SLAs — this is a volunteer research project.

## Supported versions

`sparq` is **pre-1.0 and experimental; the API is unstable** (see [`AGENTS.md`](./AGENTS.md)).
We do not maintain long-term support branches. Security fixes are made against the
**`main`** branch and shipped in the **next release** of the affected published artifact
(the `sparq-*` crates on crates.io, `@jeswr/sparq` on npm, and `sparq` on PyPI).

| Version line | Supported for security fixes |
|---|---|
| `main` (latest) | Yes — fixes land here first |
| Latest published release | Yes — via the next patch/minor release |
| Older releases | No — please upgrade |

If you depend on a specific release and cannot upgrade, say so in your report and we will
discuss options, but the default remediation is "upgrade to the fixed release."

## Scope and a critical caveat: research scaffolds with NO security guarantee

`sparq` includes cryptography-adjacent research crates. **These are research scaffolds.
They do not yet provide the security property they model, and a relying party MUST NOT
depend on them for any production security, privacy, or integrity guarantee.** This is a
deliberate, documented limitation, not an undisclosed weakness — so reports that simply
restate the gaps below are already known (but reports of *new* classes of failure, or of
a gap in a part we claim *is* sound, are very welcome).

### `sparq-zk` and `sparq-zk-compose` — the v1 ZK verifier is NOT sound

The zero-knowledge query-proof estate (`sparq-zk`, `sparq-zk-compose`, and the Noir
circuits under `zk/`) is a faithful research scaffold of the **in-circuit relations**
(the scan completeness/soundness relation and the integer/float filter-comparison
relations are correctly constrained in-circuit). **However, the verifier-side binding
layer that would make those relations mean anything to a third party does not yet exist,
and so the v1 verifier (`verify_manifest`) provides NO meaningful soundness guarantee to
a relying party.** A prover that controls its own side can make the verifier return
success over arbitrary, false query results.

This is the documented verdict of an adversarial soundness audit — see
[`research/zk-soundness-audit.md`](./research/zk-soundness-audit.md) for the full
analysis. In summary, the audit confirmed (among others) that:

- the cryptographically verified public inputs are never reconstructed from, or compared
  against, the declared manifest statement — so a prover can attach a genuine proof of a
  *different* statement while advertising a false result;
- the verification key handed to the proof backend is taken from the prover's own blob
  rather than recomputed from the canonical circuit;
- there is no issuer-signature / key-set membership check — commitments are unsigned,
  prover-chosen field elements, so credential provenance has no cryptographic backing;
- there is no replay/freshness binding — a captured manifest is infinitely replayable;
- FILTER operator/bound/verdict and the query text are not bound to the proof.

**Do not present the v1 ZK verifier as proving anything to a relying party.** Treat any
"verified" result from it as untrusted. The remediation work for these gaps is tracked in
the issue tracker (beads); the audit document is the design-of-record for the fixes.

### `sparq-mpc` — cryptography deferred

The multi-party-computation crate (`sparq-mpc`) is an early research scaffold. The MPC
cryptography (secret sharing / garbled circuits / authenticated-input malicious security)
is **deferred and not implemented** to a production standard. It provides **no
confidentiality, correctness, attestation, or malicious-security guarantee** today and
must not be used to protect real data across distrusting parties.

### Other capability crates

The opt-in capability crates (`sparq-geo`, `sparq-text`, `sparq-vectors`, `sparq-rsp`,
`sparq-shacl`, `sparq-reason`, `sparq-solid`, `sparq-hdt`, …) are functional features, not
security boundaries. As with the whole engine at this stage, run untrusted input through
`sparq` only inside your own sandboxing/resource limits.

## Hardened-input expectations for the core engine

The parser, query engine, and HTTP server (`sparq-core`, `sparq-engine`, `sparq-server`)
are the surfaces most likely to process untrusted input. We treat memory-safety issues,
panics reachable from untrusted input that escalate to denial of service, and
authentication/authorization gaps in the server as in-scope security bugs — please report
them through the private channels above.
