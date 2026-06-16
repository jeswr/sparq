<!-- [OPUS-4.8] sq-toze — Privacy framework AUDIT findings (independent auditor).
     Adversarial review of cert-privacy (PR #236, head worktree-agent-af186064192837d69).
     Re-review when Fable returns. NON-CANONICAL timing (EC2 work box). -->

# Privacy (GDPR / ISO 27701 / SOC 2 Privacy) — independent audit findings

**Framework:** `privacy` (GDPR + ISO/IEC 27701 + SOC 2 Privacy/Confidentiality), epic **sq-toze**.
**Under review:** PR **#236** ("Certification readiness — Privacy"), head branch
`worktree-agent-af186064192837d69`, commit `cad9c95`.
**Auditor:** SPARQ compliance-auditor (independent). I did not edit any source, `compliance/privacy/`,
or the shared docs; I report, the engineer remediates.

> 🤖 SPARQ agent — independent privacy auditor.

## Verdict

**FINDINGS: 4** (1 high, 2 medium, 1 low). **NOT signed off.**

The framework is honest in its *framing* and correct on its headline gap, but it **overclaims one
control (P-12 error/log hygiene)**: it is labelled *implemented & verified — no query text / RDF
content echoed*, yet the code echoes caller input on the query-parse path and **echoes fragments of
the loaded RDF data** on the RDF-body-load path — neither of which the gap register acknowledges (it
asserts the UPDATE path is "the one exception"). Per the honesty contract, an "implemented & verified"
label that the cited code does not deliver is a **high-severity** finding (misrepresentation > a known
gap). The remaining three findings are the under-scoped severity reasoning behind that overclaim, a
missing regression test, and a bead-tracking mismatch.

What is **correct** and should NOT be re-litigated by the engineer (verified below): the
controller-vs-engine responsibility split is honest (no "sparq is GDPR compliant" claim — the only
occurrence is an explicit *negation*); the ZK/MPC estate is correctly and consistently **excluded**
from every privacy capability claim; **PR-G3 (the `--persist` WAL is not erasure-complete) is the
correct Medium headline privacy gap** and its evidence is sound.

---

## Findings

### F-1 (HIGH) — P-12 "error/log hygiene" is overclaimed as IMPL; the no-echo property is false on two code paths

- **Control:** P-12 — *Error/log hygiene — no personal data leakage in errors/logs (Art. 5(1)(f) /
  SOC 2 CC6.7)*, status **IMPL** ("HTTP error bodies are generic... **No query text / result rows /
  RDF content / stack traces / filesystem paths are echoed.**"). Gap register **PR-G1** narrows the
  single acknowledged exception to the UPDATE path and frames it as "*query text*, not loaded RDF
  data," and the control table states "Query responses already use generic bodies; UPDATE parse errors
  are the one exception."
- **What I checked** (read of `crates/sparq-server/src/http.rs` @ `cad9c95`):
  - `:1812` — `run_query_pinned`: `Err(PrepareError::Malformed(msg)) => return
    bad_request(&format!("malformed query: {msg}"))`. The **query** path echoes the `spargebra`
    parser diagnostic `{msg}` (caller query text). This directly contradicts the control-table claim
    that query responses "already use generic bodies."
  - `:2293` — `body_to_ntriples`: `Graph::load_str(text, format).map_err(|e|
    bad_request(&format!("malformed RDF body: {e}")))`. The GSP write-body path echoes the **RDF**
    parser error. Tracing `Graph::load_str` → `parse_to_triples` (`crates/sparq-core/src/lib.rs:632`,
    `:641` region) shows parser errors are surfaced via `e.to_string()` on the underlying RDF parser,
    whose diagnostics characteristically embed the offending token (the literal/IRI being parsed) —
    i.e. a fragment of the **loaded RDF data**, which is exactly the personal-data case.
  - `:2302` — same for `malformed RDF/XML body: {e}`; `:2738` — `malformed gzip body: {e}`.
- **Why it fails:** "Implemented & verified — no RDF content echoed" is not delivered by the cited
  code. There are at least three additional caller-input/loaded-data echo sites beyond the one
  PR-G1 admits. The honesty contract makes an overclaimed IMPL label worse than an honest gap.
- **Remediation:** (a) Downgrade P-12 from a flat **IMPL** to **IMPL with a documented residual**
  that enumerates *all* echo sites (`:1812`, `:2293`, `:2302`, `:2738`, plus the UPDATE `:2116`), not
  just the UPDATE path. (b) Either strip these diagnostics to generic bodies (parity with the query
  *result* path) or gate them behind `--verbose`. (c) Correct the control-table sentence "Query
  responses already use generic bodies; UPDATE parse errors are the one exception" — it is false.
  Tracked: bead **sq-cz89** (created by this audit, linked to epic sq-toze).

### F-2 (MEDIUM) — PR-G1 severity reasoning is unsound: the RDF-body path leaks loaded data, not just query text

- **Control:** gap-register **PR-G1**, severity **Low**, justified by "This is *query text*, not loaded
  RDF data."
- **What I checked:** the `:2293`/`:2302` echo sites (F-1) are on the **data-ingest** path
  (`POST`/`PUT` an RDF body to the graph store). The echoed parser error can contain a fragment of the
  RDF being loaded — i.e. the operator's actual (potentially personal) data — not merely the caller's
  query syntax.
- **Why it fails:** PR-G1's *entire* Low-severity argument rests on "not loaded data." That premise is
  false for the RDF-body path. A leak of loaded RDF content in a 400 body to an unauthenticated caller
  (boundary B3 — bare server has no auth) is a more material residual than "a fragment of malformed
  query syntax."
- **Remediation:** re-assess PR-G1 severity for the RDF-body sites to **Medium** (or split PR-G1 into
  G1a query-text/Low and G1b loaded-RDF/Medium). Keep the UPDATE/query *syntax* echo at Low if desired,
  but the loaded-data echo cannot be Low on the stated reasoning. Tracked in **sq-cz89**.

### F-3 (MEDIUM) — P-12 marked "verified" with no regression test pinning the no-echo property

- **Control:** P-12 status **IMPL** ("implemented & **verified**").
- **What I checked:** the `#[test]` block in `crates/sparq-server/src/http.rs` (`:3100+`) has tests for
  constant-time auth (`constant_time_eq_matches_plain_eq`), auth posture, and bearer parsing — all of
  which I confirmed exist — but **no test asserts that any error body is free of echoed query/RDF
  content**. The evidence pack §P-12 cites *line numbers of generic strings*, not a test that pins the
  property; meanwhile the echoing sites (F-1) have no negative test either.
- **Why it fails:** "verified" implies a re-runnable check. For a hygiene property whose violation is a
  privacy leak, an assertion-free citation of literal strings is not verification — especially when
  other branches of the same function violate it.
- **Remediation:** add a regression test that drives a malformed query, a malformed RDF GSP body, and a
  malformed UPDATE through the handlers and asserts the default-mode 400 body contains no fragment of
  the input; only then is P-12 honestly "verified." Tracked: bead **sq-zg0u** (created by this audit).

### F-4 (LOW) — remediation beads cited in the gap register do not match the gaps they are mapped to

- **Control:** engineer honesty contract ("record it... with a remediation plan + a `bd` bead"); the
  gap register maps **PR-G1** *and* **PR-G2** both to **sq-toze.32**.
- **What I checked:** `.beads/issues.jsonl` — `sq-toze.32` is titled *"privacy: document
  query-log/PII data-minimisation posture + operator hardening guide"* (a **documentation** task,
  status open, no description). It does not track (a) the PR-G1 **code** fix (strip/gate the
  diagnostic + regression test) nor (b) the PR-G2 structured audit sink. `sq-toze.33` (WAL runbook /
  PR-G3) and `sq-toze.34` (log redaction / PR-G4) do match their gaps; `sq-toze.35` (ZK gate) matches.
- **Why it fails:** PR-G1's and PR-G2's actual remediations have no bead that tracks them; a doc bead is
  cited in their place, so the "tracked" claim is not met for those two rows.
- **Remediation:** point PR-G1 at the new code/test beads (**sq-cz89** + **sq-zg0u**) and either create
  a distinct bead for the PR-G2 structured audit sink or relabel sq-toze.32's scope to cover it
  explicitly. (Beads are created from the main checkout; this auditor created sq-cz89/sq-zg0u and
  linked them to epic sq-toze.)

---

## Coverage note — what I assessed

**Independently verified against source at `cad9c95` (claim holds):**

| Control | Claim | Verification |
|---|---|---|
| P-2 | request logging OFF by default; metrics aggregate-only | `http.rs:276` `verbose:false`; `:1447` TraceLayer gated on `config.verbose`; `metrics.rs` `record_request(endpoint:&'static str,…)`, `endpoint_label` returns bounded `&'static str` (`:105`) — **no PII/raw-path label**. ✔ |
| P-3/P-4/P-6 | SPARQL DELETE/DROP/CLEAR/DELETE-INSERT erasure + atomic rectification | `sparq-engine/src/update.rs:429–495` (`DeleteData`/`Clear`/`Drop`/`DeleteInsert` route through durable graph ops); test `multi_op_one_unauthorized_denies_whole_body_no_partial_apply` exists at `sparq-solid/tests/update.rs:443`. ✔ |
| P-9 / PR-G3 | `--persist` WAL is append/replay → DELETE not erasure-complete | `main.rs:14–22` WAL semantics (fsync-before-ack, replay on restart); in-memory default. The erasure-incompleteness reasoning is **correct** and is rightly the **Medium headline** gap. ✔ |
| P-10 | constant-time Bearer; identical 401 | `http.rs:567` `constant_time_eq` (len-equality early-return + XOR fold, honestly documented as hand-rolled); `:629` `auth_check`; test `constant_time_eq_matches_plain_eq` exists. ✔ |
| P-11 | no self-grant; reserved `urn:sparq:` stripped/fail-closed | `sparq-solid/tests/hardening.rs` `sentinel_graph_cannot_be_smuggled` (:11), `reserved_session_values_fail_closed` (:63) exist. ✔ |
| P-13 | SERVICE deny-all default | `main.rs` SSRF allowlist (`:24–29` doc, `:194–201` parse) — default-deny confirmed. ✔ |
| P-14 | QueryBudget + body/concurrency caps | `sparq-engine/src/lib.rs:56+` `QueryBudget` explicit-error (no silent truncation); `http.rs` load-shed/body-limit layers. ✔ |
| P-15 | WASM keeps data client-side | `sparq-wasm/src/lib.rs:12` `#![forbid(unsafe_code)]`; in-browser build. ✔ |
| Honesty | no "sparq is GDPR compliant"; ZK/MPC excluded | Only occurrence of the phrase is an explicit **negation** (`compliance/README.md:14`); all ZK/MPC mentions are exclusion/carve-out; `SECURITY.md` v1-ZK-NOT-sound + MPC-deferred disclaimers intact and consistently referenced. ✔ |
| Beads | sq-toze.32/33/34/35 exist | All present in `.beads/issues.jsonl`. ✔ (mismatch is F-4, not absence) |

**Assessed and found honest:** the controller/processor split (P-1, P-16, P-17, P-18 correctly
**OPERATOR**/AUDIT-READY, not claimed as engine properties); the "NOT gaps — operator
responsibilities" list (at-rest encryption, TLS, lawful basis, etc. correctly excluded from the gap
register rather than papered as defects); the DPIA is a genuinely-scoped skeleton (engine-side half
filled with evidence, operator half left as prompts) and is not boilerplate; the data-flow doc maps
the real five touch-points.

**Could not fully assess (out of agent scope / external):** the 27701 / SOC 2 Privacy **certificate**
itself (PR-X2 — needs the operator's PIMS + accredited external auditor; correctly labelled
AUDIT-READY); the cryptographic soundness of the ZK/MPC estate (PR-X1 — external cryptographer; the
privacy framework correctly draws **no** assurance from it). I did not execute the live server to
trigger the echo responses; the finding rests on a static read of the handler code, which is
unambiguous (`format!("malformed query: {msg}")` etc.) — an engineer may confirm with the
F-3 regression test.

## Headline-gap and exclusion judgement (explicitly requested)

- **Controller-vs-engine split honest?** **Yes.** No claim states "sparq is GDPR compliant"; the one
  textual occurrence negates it. Each obligation is either a cited engine capability or a labelled
  OPERATOR responsibility.
- **WAL-erasure (PR-G3) correctly the headline privacy gap?** **Yes.** It is the only Medium engine-side
  gap, its mechanism is verified, and it is the right thing to foreground for a data engine offering
  Art. 17 erasure.
- **ZK/MPC excluded?** **Yes.** Consistently carved out across controls.md, evidence.md, data-flow.md,
  and dpia.md (B10); no privacy capability rests on it; gated on sq-toze.35.

## Loop-back to the engineer

Address F-1 (de-overclaim P-12 + enumerate all echo sites), F-2 (re-rate the loaded-RDF echo),
F-3 (ship the hygiene regression test), F-4 (fix the bead mapping). Beads **sq-cz89** (code) and
**sq-zg0u** (test) are filed under epic sq-toze for the code/test work. I will sign off when these are
resolved — with the standing caveat that the 27701/SOC 2 certificate and ZK/MPC soundness remain
**external** items outside this framework's closure.

---

*FINDINGS: 4*
