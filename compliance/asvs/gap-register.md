<!-- [OPUS-4.8] sq-toze.10 — ASVS gap register. Honesty contract: real gaps only,
     each with a bead; documented decisions are labelled as such, not laundered. -->

# OWASP ASVS — gap register

Open gaps against the **applicable** ASVS L1/L2 requirements (see
[`controls/asvs.md`](./controls/asvs.md)). Each has a severity, a remediation plan, a target,
and the `bd` bead tracking the fix (under epic `sq-toze`). Cross-cutting gaps already tracked
by other frameworks (GX-*) are noted but **not** re-owned here.

Severity: **High** = breaks an L2 requirement on the untrusted-input path / DoS / SSRF;
**Medium** = an L2 requirement partially met or unverified; **Low** = hardening / defense-in-
depth on a surface with low residual risk for a JSON API.

| ID | Gap | ASVS | Sev | Remediation | Target | Bead |
|---|---|---|---|---|---|---|
| **ASVS-G1** | ~~`sparq-server` sets **no security response headers**.~~ **Resolved** ([OPUS-4.8] sq-cmvh) — a `map_response` layer in `harden()` stamps `X-Content-Type-Options: nosniff`, `Content-Security-Policy: default-src 'none'; frame-ancestors 'none'`, `X-Frame-Options: DENY` and `Referrer-Policy: no-referrer` onto **every** response (success / streamed / error), asserted by `tests/hardening.rs::security_headers_*`. HSTS is deliberately N/A (origin terminates plain HTTP; HSTS is the fronting TLS proxy's job); `X-XSS-Protection` omitted (deprecated, superseded by CSP); a blanket `Cache-Control: no-store` was not forced (results are uncached by default — nothing to tighten — and it would override `/health`,`/metrics`). | V14.4 | **Resolved** | — | done | `sq-cmvh` |
| **ASVS-G2** | **No first-party CORS allowlist option.** sparq-server emits no CORS headers, so cross-origin browser reads are denied by same-origin policy. | V14.5.3 | **Low** (documented decision) | This is a **safe default**, not a vulnerability — cross-origin access is the operator gateway's job (B3). *Optional* nice-to-have: an opt-in `--cors-allow-origin <allowlist>` that never defaults to `*`. Not required for L2 sign-off. | optional | `sq-o7o0` |
| **ASVS-G3** | Engine **parse/exec error strings are echoed to the client** (query-author UX) with no test asserting they never contain a filesystem path / internal host / resolved addr. | V7.1.1 | **Medium** | Audit `engine_error_response` / `execution_error` message sources; add a regression test that error bodies for a crafted query/file contain no absolute-path prefix or resolved `host:port`. (The structured `json_error` envelope + panic isolation already pass V7.1.2/V7.4.) Same root cause as the wider error-body info-leak bead **`sq-cz89`** (P1) — cross-referenced. | next server PR | `sq-j9zs` |
| **ASVS-G4** | **No explicit SPARQL parse-depth bound** — deeply-nested-input DoS is bounded only by `--max-body-bytes`, not a depth constant (SHACL has explicit guards; the SPARQL path relies on `spargebra` recursive descent). | V5.5.2 | **Low** | Add a deeply-nested-SPARQL corpus entry to the existing `parse_sparql` fuzz target asserting no stack overflow at the 1 MiB body cap, **or** add an explicit nesting-depth guard. The body cap already bounds total input; this closes the *recursion-depth* sub-case. | fuzz corpus update | `sq-53s1` |

## Cross-cutting gaps that touch ASVS but are owned elsewhere

These were resolved or are tracked under other frameworks; ASVS **cites them as evidence**,
it does not re-own them:

| Ref | Item | ASVS relevance | Status / owner |
|---|---|---|---|
| GX-1 | cargo-deny advisories PR-gate | V10.2 (dependency integrity) | **Resolved** — `supply-chain.yml` job `cargo-deny check (advisories) — GATING` (no `continue-on-error`). Owned by `sbom`/`ssdf`. |
| GX-3 | `.well-known/security.txt` (RFC 9116) | V14 (machine-discoverable disclosure) | **Resolved** — `.well-known/security.txt` present (bead `sq-toze.4` ✓). Cited as PASS. |
| GX-6 | CONTRIBUTING secure-coding section | V1.1 (documented secure-coding standard) | **Resolved** — `CONTRIBUTING.md#secure-coding` (bead `sq-toze.7` ✓). Cited as PASS for V1. |
| GX-5 | unsafe-justification register + geiger ratchet | V10.3.2 (no unverified code) | **Resolved** — `compliance/memsafety/unsafe-register.md` (bead `sq-toze.6` ✓). ASVS V10 cites memsafety, doesn't re-litigate. |

## Items that are N/A by architecture (not gaps)

Recorded so the auditor sees they were *considered and assigned*, not missed:

- **V2/V3/V4 authentication-session-access-control lifecycle** — **operator gateway** per
  threat-model **B3**. sparq has no user accounts, sessions, or cookies. The thin applicable
  residual (optional token gate, op-classification, constant-time compare) is **PASS** in the
  control table.
- **V9 TLS** — terminated at the operator's reverse proxy by design; sparq **refuses** a
  non-loopback bind unless the operator opts in or fully authenticates (`bind_posture`).
  **AUDIT-READY** (operator-owned config).
- **V6 application crypto / V8 data classification** — the dataset's encryption +
  classification is the operator's (GDPR-controller) responsibility (`compliance/data-flow.md`).
- **The ZK/MPC estate** — **EXCLUDED**, not a delivered control: the v1 ZK verifier is **NOT
  sound** and the MPC layer carries **no guarantee** (`SECURITY.md`,
  `research/zk-soundness-audit.md`). No ASVS claim depends on it. Any future claim that it
  provides a security property is, per the honesty contract, a high-severity finding.

## Residual external attestation (cannot be self-issued)

- A formal **ASVS L2 verification by an accredited assessor** and/or a **penetration test** of
  a *deployed* sparq-server (behind the operator's gateway) — the certificate is external by
  definition. The control set above is structured to make that assessment cheap (every
  control has a test + CI job). Labelled **AUDIT-READY**, not PASS, for that final step.
