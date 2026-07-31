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
| **ASVS-G1** | ~~`sparq-server` sets **no security response headers**.~~ **Resolved** ([OPUS-4.8] sq-cmvh; residual hardening sq-2bhm) — a `map_response` layer in `harden()` stamps `X-Content-Type-Options: nosniff`, `Content-Security-Policy: default-src 'none'; frame-ancestors 'none'`, `X-Frame-Options: DENY` and `Referrer-Policy: no-referrer` onto **every** response (success / streamed / error / **auth-gated 401**), asserted by `tests/hardening.rs::security_headers_*` (incl. `security_headers_on_auth_gated_response`). HSTS is deliberately N/A (origin terminates plain HTTP; HSTS is the fronting TLS proxy's job); `X-XSS-Protection` omitted (deprecated, superseded by CSP). A blanket `Cache-Control: no-store` is **not** forced (results are uncached by default — nothing to tighten — and it would override `/health`,`/metrics`), BUT the sensitive auth-refusal (`401` from `unauthorized()`) **does** carry `Cache-Control: no-store` so a shared cache never retains it (sq-2bhm). | V14.4 | **Resolved** | — | done | `sq-cmvh`, `sq-2bhm` |
| **ASVS-G2** | **No first-party CORS allowlist option.** sparq-server emits no CORS headers, so cross-origin browser reads are denied by same-origin policy. | V14.5.3 | **Low** (documented decision) | This is a **safe default**, not a vulnerability — cross-origin access is the operator gateway's job (B3). *Optional* nice-to-have: an opt-in `--cors-allow-origin <allowlist>` that never defaults to `*`. Not required for L2 sign-off. | optional | `sq-o7o0` |
| **ASVS-G3** | ~~Engine **parse/exec error strings are echoed to the client** with no test asserting they never contain a filesystem path / internal host / resolved addr.~~ **Resolved & verified** ([OPUS-4.8] sq-kfel; builds on the sq-cz89/sq-j9zs info-leak guard). **Audit finding (honest):** the main echo paths (query-parse, update-parse, RDF-body parse, gzip-decode, query-execution) were *already* sanitized by the sq-cz89/sq-j9zs guard — each routes through `http.rs::sanitized_error`, which logs the full detail server-side under `target:"sparq_server"` and returns only a STABLE generic class message; covered by `tests/hardening.rs::no_echo_*`. sq-kfel closed the **residual raw-echo paths** the audit found that bypassed that discipline: the GSP-write **minted-update rejection** (`apply_gsp_update` previously `graph store write failed: {e}` verbatim), the **TPF term-parse** error (`{e}` echoed the caller's term), the descriptor-serialize 500s, the `--persist` **compaction** failure (echoed the absolute persist path), and the unreachable middleware-error fallback — all now route through `sanitized_error`/server-log. **Verify:** new regression guards assert, per failure class, that error bodies carry a useful category but NO absolute path (`/home/`,`/Users/`,`/tmp/`,…), no `Debug` of an internal type (`WriteError::`,`Os { code:`,…), no source-file path and no secret — `tests/hardening.rs` (`no_echo_gsp_write_rejection_stays_clean`, `unauthorized_error_is_clean_and_actionable`, `not_found_error_is_clean`, `main_failure_classes_are_clean`, `FORBIDDEN_INTERNALS`) + `tests/tpf.rs::malformed_term_does_not_echo_input`. | V7.1.1 | **Resolved** | — | done | `sq-kfel`, `sq-cz89`, `sq-j9zs` |
| **ASVS-G4** | ~~**No explicit SPARQL parse-depth bound** — deeply-nested-input DoS is bounded only by `--max-body-bytes`, not a depth constant.~~ **Resolved** ([OPUS-4.8] `sq-53s1`/`sq-v5dg`) — an explicit configurable nesting-depth cap now guards every native-recursion parse path. **SPARQL/SPARQL-Update** (`spargebra`): `MAX_RECURSION_DEPTH = 128` levels of nested group graph patterns, bracketed expressions/paths, blank-node property lists, RDF collections and reified/triple terms — exceeding it returns the clean `TooDeeplyNested` syntax error instead of overflowing the stack. **RDF** (`sparq-core::nt`): `MAX_TRIPLE_TERM_DEPTH = 128` bounds nested RDF 1.2 triple terms `<<( … )>>` on both the interning (`triple_term`) and N-Quads no-intern span (`span_triple_term`) paths, returning a normal parse `Err`. The Turtle/TriG/N-Quads path via `oxttl` is already a heap-`Vec` pushdown automaton (no native recursion → cannot stack-overflow). Errors are sanitized (no echo of offending input, per `sq-cz89`/#241). Verified by deep-nesting tests (10k levels → clean error, not crash); conformance ratchets unaffected (caps far exceed any real query/data). | V5.5.2 | **Resolved** | — | done | `sq-53s1` |

## Cross-cutting gaps that touch ASVS but are owned elsewhere

These were resolved or are tracked under other frameworks; ASVS **cites them as evidence**,
it does not re-own them:

| Ref | Item | ASVS relevance | Status / owner |
|---|---|---|---|
| **GX-14** | **SAST is not running — CodeQL disabled at the Actions level (`disabled_manually`) since 2026-07-18; no compensating SAST.** | V1.1 (SDLC gate stack), V10.2 (evidence struck), **V10.1.1 (L3) NOT met** | **OPEN (P1)** — cross-cutting anchor in [`../gap-register.md`](../gap-register.md) GX-14; `ASSURANCE.md` §11; posture decision issue **#4620**, alert triage **#4615** (35 open critical alerts triaged as false positives of one query-model defect — triaged ≠ covered). ASVS L1/L2 sign-off is **not** blocked (L2 mandates no SAST); the V1.1/V10.2 rows keep **PASS** on their other, live evidence, but CodeQL is struck from both. Not re-owned here. |
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
- **The ZK/MPC estate** — **EXCLUDED**, not a delivered control: <!-- [OPUS-4.8] reconciled with post-remediation re-audit (sq-gbp4); see ZK-verdict cross-ref sweep --> the ZK verifier is
  **remediated but NOT externally audited** (originally unsound; `sq-1s2` binding layer landed;
  internal re-audit "sound as landed for the assumed threat model"; external sign-off `sq-qhy4`
  PENDING; **no production guarantee**) and the MPC layer carries **no guarantee** (`SECURITY.md`,
  `research/zk-soundness-audit.md`, `research/zk-verifier-reaudit.md`). No ASVS claim depends on
  it. Any future claim that it provides a production security property before that external
  sign-off is, per the honesty contract, a high-severity finding. [OPUS-4.8]

## Residual external attestation (cannot be self-issued)

- A formal **ASVS L2 verification by an accredited assessor** and/or a **penetration test** of
  a *deployed* sparq-server (behind the operator's gateway) — the certificate is external by
  definition. The control set above is structured to make that assessment cheap (every
  control has a test + CI job). Labelled **AUDIT-READY**, not PASS, for that final step.
