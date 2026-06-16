<!-- [OPUS-4.8] sq-toze.10 — OWASP ASVS adversarial audit of PR #237 / branch cert-asvs.
     Independent auditor pass; engineer remediates. Re-review when Fable returns. -->

# OWASP ASVS — independent audit findings (PR #237, branch `cert-asvs`)

> 🤖 SPARQ agent — adversarial compliance auditor for epic `sq-toze` (`sq-toze.10`).

**Scope audited:** the four evidence docs `compliance/asvs/{README.md, controls/asvs.md,
evidence.md, gap-register.md}` on `origin/cert-asvs`, checked against the **actual repo** —
`crates/sparq-server/src/http.rs`, `crates/sparq-engine/src/service.rs`,
`crates/sparq-engine/src/lib.rs`, the regression suites under
`crates/sparq-server/tests/`, and the `.beads/issues.jsonl` tracker.

**Method:** every PASS row was checked against cited code (git-show against
`origin/cert-asvs`), the security-relevant server files were confirmed **byte-identical**
between `origin/cert-asvs` and the audit worktree HEAD (`git diff --quiet HEAD
origin/cert-asvs -- <file>` → SAME for `http.rs`, `hardening.rs`, `auth.rs`,
`service_allowlist.rs`), so the regression suite was **run** for runtime evidence. Timing is
NON-CANONICAL (EC2 work box).

---

## What was verified to actually hold (no finding)

These PASS claims were independently confirmed; they are recorded so the loop can see they
were checked, not waved through.

- **V11 DoS limits + V5 body/decompression caps + V7 panic isolation + V2.10/V4 auth gate —
  RUNTIME-VERIFIED.** Ran `cargo nextest run -p sparq-server --test auth --test hardening
  --test service_allowlist` → **22 tests, 22 passed, 0 failed.** Confirms at runtime:
  oversized body→413, slow query→503 (and UPDATE path→503), concurrency shed→429, row-cap→413
  (incl. the knob-naming on non-SELECT paths), zip-bomb gzip→413, decompress-disabled→413,
  handler panic→500 (connection not killed), structured-JSON errors, and the full auth matrix
  (write-only token gates writes/opens reads, read-gate flag, UPDATE-via-query-path gated,
  UPDATE-via-form gated, SELECT-in-update-form classified as write).
- **V7.1.2 401 parity.** `tests/auth.rs::update_without_header_is_401` and
  `::update_with_wrong_token_is_401` both assert `401` + identical `WWW-Authenticate: Bearer`
  and a verified no-mutation; the `unauthorized()` path is a single fixed response, so the
  body is identical by construction. Holds.
- **V2.10/V6.2 constant-time compare.** `constant_time_eq` (`http.rs:567`) is a length-check +
  XOR-accumulate routed through `core::hint::black_box(diff) == 0` (`:581`); test
  `constant_time_eq_matches_plain_eq` (`:3128`) present. Holds.
- **V12 SSRF default-deny + deny-before-dial.** `EgressFilterResolver` (`service.rs:469`)
  refuses any host not on the allowlist in `AllowlistOnly` mode **before** `to_socket_addrs()`
  (no DNS at all); `is_forbidden_ip` (`service.rs:237`) covers loopback / RFC1918 / link-local
  `169.254.0.0/16` (incl. metadata `169.254.169.254`) and IPv4-mapped-IPv6
  (`::ffff:169.254.169.254`, unit test `:771`) + unique-local v6. The empty-allowlist =
  deny-all default is the server posture; `tests/service_allowlist.rs` asserts **zero
  accept()** under deny. Holds. (See OBS-1 below for a precision caveat on the wording.)
- **V5.2 no query-injection.** The remote SERVICE query is rebuilt from the **typed**
  `GraphPattern` algebra via spargebra's `Display` round-trip (`service.rs` head §1), not by
  string-concatenating user input; SRJ responses are validated term-by-term. Holds.
- **V3 / V2 / V4 N/A-operator is HONEST (independently grepped).** On `origin/cert-asvs`,
  `git grep -ni` across `crates/`: `Set-Cookie` → **0**, `CorsLayer` → **0**,
  `tower_http::cors`/`use …cors` → **0**, `Access-Control-Allow*` (the real header) → **0**,
  cookie/session handling in `crates/sparq-server/*` → **0** (the only "session" hit is a
  docs heading in `SUBSCRIPTIONS.md`). There genuinely are no accounts/sessions/cookies — the
  B3 no-auth-by-default design — so V2/V3/V4 are legitimately N/A-operator, not a dodged
  control. The residual (token gate, op-classification, constant-time) is the only applicable
  slice and it is PASS.
- **ZK/MPC EXCLUDED — honesty tripwire CLEAR.** No ASVS control rests on the crypto estate;
  every reference in the four docs treats it as **EXCLUDED / NOT sound** and points at
  `SECURITY.md` + `research/zk-soundness-audit.md`. `SECURITY.md` §"research scaffolds with NO
  security guarantee" / "the v1 ZK verifier is NOT sound" is intact. No "verified crypto
  control" claim anywhere. ✅
- **`sparq-serve/tests/tokens.rs` is NOT cited as auth evidence.** Confirmed it appears only in
  the evidence.md honesty note disclaiming it (read-your-writes generation tokens, not auth).
- **Gaps G1–G4 are the real set, correctly rated, not laundered.** G1 (no `nosniff`/CSP/
  `X-Frame-Options`/HSTS — grep → 0 in `crates/`) Medium; G2 (no first-party CORS — labelled a
  *safe default*, Low/documented-decision, not a vuln) honest; G3 (engine error strings echoed
  — `execution_error()`/`bad_request()` do `format!("…: {e}")` with the raw engine `e`; no test
  asserts they are path/host-free) Medium and honest; G4 (no explicit SPARQL parse-depth bound;
  body-cap only) Low. SLSA/V10.3 is labelled **AUDIT-READY** / "honestly ≈L2", not overclaimed.

---

## FINDINGS

### Finding 1 — The four ASVS gap-tracking beads do NOT exist (fabricated/missing tracking artifact)
- **Severity:** **Medium**
- **Control / claim violated:** `gap-register.md` header ("each with a bead"), line 7–8 ("Each
  has a severity, a remediation plan, a target, **and the `bd` bead tracking the fix** (under
  epic `sq-toze`)"), the per-gap **Bead** column (`sq-2bhm`, `sq-vtkj`, `sq-kfel`, `sq-1ukn`),
  the `controls/asvs.md` rows for V14.4 ("Tracked **GAP ASVS-G1** + bead") and the Summary line
  166–167 ("All in `gap-register.md` **with beads**"). This is the project's hygiene contract
  (TODOs→beads) and the lead's loop-closure mechanism.
- **What I checked:**
  `grep -c "sq-2bhm|sq-vtkj|sq-kfel|sq-1ukn" .beads/issues.jsonl` in the worktree (current,
  Jun-16) → **0 each**; and on the committed trees:
  `git show origin/cert-asvs:.beads/issues.jsonl | grep -c sq-2bhm` → **0** (same for the other
  three), and identically **0** on `origin/main`. There are also **no** beads with the gap
  titles (`grep -iE "nosniff|cors.allow|parse-depth|security.response.header"` → none). The
  epic `sq-toze` and `sq-toze.10` (the ASVS cert task) **do** exist — only the four *gap* beads
  are missing.
- **Why it fails:** the gap-register asserts a tracking artifact that does not exist for **all
  four** open gaps. The remediation items (G1 security headers, G3 error-string leakage test,
  etc.) are therefore **untracked** — there is nothing for the engineer→auditor loop to close
  against, and a reader cross-referencing the cited bead IDs finds dangling references. The
  gaps themselves are correctly identified and described; it is the *"tracked"* claim that is
  false. (Per the auditor mindset, a documented-evidence-vs-reality mismatch is a finding even
  when the underlying gap analysis is sound.)
- **Remediation:** create the four beads under epic `sq-toze` in the **main checkout** with
  `bd` (never hand-edit `.beads/issues.jsonl`), titled to match the gaps, then re-export so
  the IDs in `gap-register.md` resolve. If the engineer prefers different IDs, update the
  doc's Bead column to the real IDs. Either way the doc and the tracker must agree.
  *(Auditor note: `bd` is not on PATH in this isolated worktree and I must not `cd` to the main
  checkout, so I could not create the bead myself — this remediation is the engineer's, in the
  main checkout. Flagging here so it is not lost.)*

### Finding 2 — `evidence.md` "returns nothing" grep is imprecise → a re-runner sees 18 false-positive hits
- **Severity:** **Low** (documentation accuracy; the substantive control claim still holds)
- **Control / claim:** `evidence.md` §V14 ("No CORS / no security headers") and Honesty-note 1
  both state the command
  `grep -rn "CorsLayer\|Access-Control\|nosniff\|Content-Security-Policy\|X-Frame-Options"
  crates/` **"returns nothing"** / **"grep to zero hits"** — this is the cited basis for the V3
  N/A, the V14.4 GAP, and the V14.5.3 documented-decision.
- **What I checked:** `git grep -ni "Access-Control" origin/cert-asvs -- 'crates/*'` → **18
  hits** — all the *English phrase* "access-control" in `sparq-solid` (WAC/ACP code, comments,
  `.n3` rule files, `solid-access-control-design.md` references), **none** the HTTP
  `Access-Control-Allow-Origin` response header. The narrow, correct grep
  (`"Access-Control-Allow"`) → **0**, and `CorsLayer`/`nosniff`/CSP/`X-Frame-Options` → **0**.
- **Why it fails:** the re-runnable anchor the evidence pack is built on does **not** return
  nothing for the `Access-Control` alternative; an auditor re-running the documented command
  verbatim gets 18 hits and a moment of false alarm. The control conclusion (no CORS header is
  emitted) is **correct** — only the cited command is over-broad.
- **Remediation:** tighten the documented grep to the header form
  (`grep -rn "CorsLayer\|Access-Control-Allow\|nosniff\|Content-Security-Policy\|X-Frame-Options"
  crates/`), or add a one-line note that bare `Access-Control` matches the Solid WAC English
  term and the header-specific grep is the load-bearing one. No code change required.

---

## Coverage note

**Assessed (with the result above):** V1 (secure-coding standard + threat model exist), V2.10
/V6.2 (constant-time, secrets-from-env), V3 (N/A grep-confirmed), V4 residual (op-class, least
priv), V5.1/V5.2/V5.3/V5.5 (parse seam, body cap, decompression cap, error escaping, fuzz
targets present), V7.1/V7.3/V7.4 (401 parity, body-free opt-in logging, panic isolation), V9
(operator-TLS labelling), V10 (gating cargo-deny / CodeQL / forbid-unsafe references), V11
(DoS — runtime-verified), V12 (SSRF — code + deny-before-dial test), V13 (405 routing, content
-type), V14 (secure defaults, no static-serve sink, security.txt, the G1/G2 gaps), plus the
ZK-exclusion honesty tripwire and the `tokens.rs` non-citation.

**Could not fully assess (out of this audit's reach / external):**
- **An accredited ASVS L2 verification / penetration test of a deployed instance** — external
  by definition; the docs correctly label it AUDIT-READY, not PASS. Not a finding.
- **`fuzz.yml` actually finding zero crashes on the cited corpus** — I confirmed the fuzz
  *targets* exist and are enumerated; I did not run a fuzz campaign (time/scope). The V5
  parser-robustness claim rests on "targets exist + run in CI", which is what the doc claims;
  the residual depth-bound case is honestly the open G4. Not escalated.
- **G3 leak reachability** — I confirmed `execution_error`/`bad_request` echo the raw engine
  string, which is exactly the unverified risk G3 names; I did not enumerate every engine error
  source to prove a path *can* leak. The Medium rating + tracking (once Finding 1 is fixed) is
  appropriate; not separately escalated.

## OBS-1 (informational, not a finding) — SSRF wording vs. allowlist trust model
`controls/asvs.md` V12.6.1 says the engine "**also** blocks loopback/private/link-local +
cloud-metadata … on the **resolved** IP." In `EgressFilterResolver` the IP filter is
`filter(|sa| allowed || !is_forbidden_ip(sa.ip()))` — i.e. an **allowlisted** host **bypasses**
`is_forbidden_ip`. This is a deliberate trust model (an explicit allowlist entry = the operator
vouches for that host, including a host that resolves to a private/metadata IP), and the
server's default (empty allowlist, `AllowlistOnly`) denies everything before any dial, so the
default SSRF posture is sound. The belt-and-suspenders resolved-IP filter therefore only adds
defense for **non-allowlisted** hosts in a **non-strict** embedder mode. Not a control failure
(default-deny holds), but a relying party could over-read the sentence as "metadata IP is
blocked even for allowlisted hosts," which is not the case. Optional: add half a sentence
clarifying the filter applies to the not-explicitly-allowlisted path.

---

## Verdict

The ASVS slice is, on substance, strong and unusually honest: the core untrusted-input
controls (V5/V7/V11/V12/V13/V14) are real, tested, and **runtime-verified** here; the
N/A-operator scoping for V2/V3/V4 is grep-confirmed (no sessions/cookies/CORS), not a dodge;
the SLSA/TLS items are labelled AUDIT-READY rather than overclaimed; and the **ZK/MPC estate is
cleanly excluded with the "NOT sound" disclaimer intact** — the honesty tripwire is clear. The
SSRF and DoS PASS claims **hold**.

Two findings block sign-off: a **Medium** integrity gap (the four gap-tracking beads the
register claims do not exist — the open remediation is untracked) and a **Low** doc-accuracy
gap (an over-broad "returns nothing" grep that actually returns 18 false positives). Neither is
a control failure on the untrusted-input path; both are evidence-vs-reality mismatches that the
auditor contract requires flagging.

**FINDINGS: 2**

*Standing caveat:* an accredited external ASVS L2 verification / penetration test of a deployed
instance remains external and cannot be self-issued; the control set is structured (test + CI
per control) to make that assessment cheap.
