<!-- [OPUS-4.8] sq-toze — Privacy evidence pack. Per-capability file/line/test the
     control table cites. Engineer↔auditor loop (epic sq-toze). Re-review when Fable returns. -->

# Privacy — evidence pack

Per-capability verification backing [`controls.md`](./controls.md). Each entry is a control ID,
the exact artifact, and what re-reading/re-running it proves. Paths are repo-relative. **Timing
is NON-CANONICAL** (EC2 work box) — no benchmark numbers are baked here.

---

## P-2 — Data minimisation by default (request logging OFF; aggregate-only metrics)

**Request logging is opt-in.** `crates/sparq-server/src/http.rs:1447`:

```rust
if config.verbose {
    routes.layer(TraceLayer::new_for_http())
} else {
    routes
}
```

`ServerConfig::verbose` defaults to `false` (`crates/sparq-server/src/http.rs:276`,
`verbose: false`). The CLI must pass `--verbose` (`crates/sparq-server/src/main.rs:162`) to
enable per-request `tower_http::trace::TraceLayer` logging. **Verified:** a grep for the
request-path logging macros (`tracing::{info,debug,…}` / `info!(`/…) across
`crates/sparq-server/src/` returns **zero** matches in the query/update handlers — the only
`eprintln!`s are **startup banner lines** in `main.rs` (loaded-triple count, persist/auth/bind
posture), none of which carry per-request query text or result data.

**Metrics carry no PII.** `crates/sparq-server/src/metrics.rs` exposes exactly:
`sparq_http_requests_total{endpoint,status}` (the `endpoint` label is a **static route name**,
e.g. `/sparql`, not a user value), `sparq_query_duration_seconds` (latency histogram),
`sparq_updates_total`, and two scrape-time gauges (`sparq_active_subscriptions`, graph triple
count). **No query text, no result rows, no client identifiers, no IP addresses.** The module
header states "nothing is sampled in the background." No analytics/telemetry egress exists.

> Honesty note for P-2: this is *minimisation by default*, not minimisation *guaranteed* — an
> operator who passes `--verbose` (or fronts the server with an access-logging gateway) **will**
> capture full request lines including SPARQL query text. That residual is tracked as bead
> **sq-toze.34** (a redaction option) and is the operator's choice to make.

## P-3 / P-4 — Erasure & retention mechanism (SPARQL DELETE / DROP / CLEAR)

`crates/sparq-engine/src/update.rs` implements the full SPARQL 1.1 Update erasure surface,
routing structural ops through the durable `Graph` methods:
- `DeleteData { data }` (`:429`) — delete specific ground triples.
- `Clear { graph }` (`:437`) — empty a graph, keep its entry.
- `Drop { graph }` (`:453-465`) — remove named-graph entries (`DROP GRAPH <g>`,
  `DROP NAMED`, `DROP ALL`); default-graph DROP empties it.
- `DeleteInsert { delete, insert, … }` (`:481`) — pattern-scoped erasure/rectification.

The crate's module header (`:3-17`) documents that v1.1 supports `DELETE DATA`, `DELETE … WHERE`
with graph templates (incl. variable graph names), `CLEAR`/`DROP`/`CREATE`, and that
`ADD`/`COPY`/`MOVE` desugar into `DROP + DELETE/INSERT`. SPARQL 1.1 Update is **W3C-conformance
gated** (the `sparq-conformance` crate), so the erasure semantics are spec-faithful, not ad hoc.

**Retention default:** the server is **in-memory only** unless `--persist DIR` is given
(`crates/sparq-server/src/main.rs:321` prints "persistence: OFF — in-memory only (updates are
LOST on restart …)"). So absent an explicit operator choice, data does **not** persist across a
restart — a conservative storage-limitation default.

## P-6 — Rectification atomicity (no partial apply)

`crates/sparq-engine/src/update.rs:481` (`DeleteInsert`) implements atomic
delete-then-insert. The all-or-nothing property of a multi-operation update body is regression-
tested by `multi_op_one_unauthorized_denies_whole_body_no_partial_apply`
(`crates/sparq-solid/tests/update.rs:443`): if any operation in a body is unauthorised, the
**whole** body is denied with no partial application.

## P-9 — At-rest is operator-owned; the persist-WAL erasure caveat

`crates/sparq-server/src/main.rs:14-22` documents `--persist DIR` (env `SPARQ_PERSIST_DIR`,
QLever's `--persist-updates`): updates are made **WAL-durable to DIR before ack** and **replayed**
on restart. There is **no engine-side encryption** of that WAL — it is plaintext on the
operator's filesystem. **The erasure caveat (P-9 / sq-toze.33):** because the WAL is an
append/replay log, a triple removed by a later `DELETE` may still exist in an earlier WAL
segment until the operator compacts/rotates the persist directory. A complete Art. 17 erasure
must therefore also purge/rotate the WAL — captured in the retention/erasure operator runbook
bead **sq-toze.33**.

## P-10 — Access control (Bearer auth + sparq-solid graph-level authz)

**Transport auth.** `crates/sparq-server/src/http.rs`:
- `constant_time_eq(a, b)` (`:567`) — hand-rolled constant-time byte comparison (the header
  comment `:554-559` explains it avoids a timing oracle without pulling in `subtle`).
- `auth_check(config, op, presented)` (`:629`) — `:638` does
  `presented.is_some_and(|p| constant_time_eq(token.as_bytes(), p.as_bytes()))`.
- The doc comment at `:199-200` states a **missing vs a wrong token produce the *identical*
  401**, so there is no auth oracle.
- Posture: `--auth-token` gates **writes**; `--auth-token-read` (`:188`) also gates **reads**;
  an **empty token is rejected** (`:182`, "must not be empty"). A write-token alone leaves reads
  open by design and is **insufficient to allow a remote bind** (`main.rs:49-51`).

Unit tests at `crates/sparq-server/src/http.rs:3110+`:
`constant_time_eq_matches_plain_eq` (`:3128`, fuzzes ct-eq against `==`),
`auth_posture_from_config` (`:3186`), `auth_gate_no_token_never_gates` (`:3199`).

**Fine-grained authz (`sparq-solid`).** `crates/sparq-solid/README.md` documents graph-level
**WAC + ACP**, **fail-closed**: "Absence of a grant means a graph is invisible, and a
non-authorized graph is indistinguishable from an absent one." Every query is filtered per
`(WebID, client)` session (`∪ allow ∖ ∪ deny`). Tests:
`crates/sparq-solid/tests/{wac.rs, acp.rs, e2e.rs, update.rs, hardening.rs}` (the example
`query_as` returns different rows for `alice` vs the default session).

## P-11 — No self-grant / reserved-namespace hardening

`crates/sparq-solid/README.md` §security-posture: the reasoner is fed **only ACL/ACR +
structural facts, never pod content**; the reserved `urn:sparq:` namespace is rejected on input;
forged `<urn:sparq:auth>` graphs are stripped at load. Tests in
`crates/sparq-solid/tests/hardening.rs`: `:13-19` loads a triple in a forged `<urn:sparq:auth>`
graph and asserts only `install_auth_view` may create `urn:sparq:*` terms;
`reserved_session_values_fail_closed` (`:63`) rejects a spoofed `urn:sparq:pair?agent=…` session
value.

## P-12 — Error/log hygiene (PARTIAL — corrected per audit F-1/F-2)

> **Correction (audit F-1, HIGH).** An earlier draft of this section claimed "no query text /
> RDF content echoed" and called the UPDATE path "the one exception." **That was an overclaim.**
> Several parse/validation error bodies echo caller input — including, on the RDF-body load path,
> **fragments of the loaded RDF data** to an **unauthenticated** caller. This section now states
> exactly which bodies are hygienic and which echo caller input. The code fix is bead **sq-cz89**;
> the regression test (audit F-3) is **sq-zg0u**.

**Hygienic (verified) — these `crates/sparq-server/src/http.rs` bodies echo no caller input:**
- `:1429` `"internal server error (panic)"`; `:1776` `"update worker panicked"`;
  `:1963` `"query worker panicked"`; `:1374` `"not found"`;
  `:1435` `"server is at its concurrent-request limit, retry later"`.
- Budget errors name **limits**, not data: `engine_error_response` (`:2076`) maps a max-rows hit
  to `"result exceeds the server's working-set row limit (N rows, --max-query-rows …)"` — no
  query text, no rows.
- `json_error_bodies` middleware (`:2828`) normalises extractor rejections (e.g. the 413
  body-size reject) into the same structured shape.
- Metrics (`crates/sparq-server/src/metrics.rs`) carry no PII (P-2).

**NOT hygienic (the overclaim — these bodies echo caller input via the parser diagnostic):**
- `:1812` `bad_request(&format!("malformed query: {msg}"))` — echoes **SPARQL query text** on a
  query-parse failure.
- `:2293` `bad_request(&format!("malformed RDF body: {e}"))` and `:2302` `"malformed RDF/XML body:
  {e}"` — echo **fragments of the loaded RDF data** (the diagnostic comes from
  `Graph::load_str`→`e.to_string()`, `crates/sparq-core/src/lib.rs:632`). **Because the bare
  server has no auth (boundary B3), this returns loaded-data fragments to an unauthenticated
  caller** — the load-bearing reason F-2 raised PR-G1 from Low to Medium.
- `:2116` `bad_request(&format!("update failed: {e}"))` — echoes UPDATE parse/semantic text.
- `:2860` `"query execution error: {msg}"` — echoes the engine's execution diagnostic.

**Mechanism.** Each site wraps an underlying parser/engine error string verbatim into the 400/500
body. The fix (sq-cz89) is to return a **generic** body by default (parity with the panic/budget
path) and gate the verbose diagnostic behind `--verbose` (opt-in, like request logging), with a
regression test (sq-zg0u) asserting the default-mode body contains no echoed query/RDF fragment.
Tracked in [`gap-register.md`](./gap-register.md) **PR-G1** (Medium).

## P-13 — SSRF / exfiltration guard (default-deny SERVICE allowlist)

`crates/sparq-server/src/main.rs:24-29`: `SERVICE` federation is **deny-all by default** — a
`SERVICE <iri>` reaches nothing unless its host is allowlisted via `--service-allow` (repeatable),
`--service-allow-file`, or `SPARQ_SERVICE_ALLOW`. The header comment names this an SSRF guard:
"a `SERVICE` clause turns attacker-controlled query text into an outbound request from this host."
Maps to threat-model boundary **B4** (`research/threat-model.md`).

## P-14 — Availability limits (QueryBudget + body/concurrency caps)

`crates/sparq-engine/src/lib.rs:56-78`: `QueryBudget` is a cooperative timeout + max-rows budget;
the default `unlimited()` costs nothing on the hot path; a hit returns the explicit
`"query budget exceeded (timeout|max-rows)"` error — **never silent truncation**. Server-side:
`max_concurrent` load-shed (`crates/sparq-server/src/http.rs:1435` → 429),
`DefaultBodyLimit::max(config.max_body_bytes)` (`:1445` → 413), `--max-query-rows`/`--max-results`
mapped to 413 in `engine_error_response` (`:2076`).

## P-15 — Client-side processing keeps data local

`crates/sparq-wasm/src/lib.rs:12` is `#![forbid(unsafe_code)]` (sq-emay, zero `unsafe`); the
header (`:1-3`) describes "the sparq parser + triplestore + SPARQL engine compiled to
WebAssembly". The engine therefore runs **in the user's browser tab**; an embedding app can
query personal RDF entirely client-side with no server round-trip — a structural data-transfer
minimisation (the embedding app's own handling is its responsibility).

## ZK/MPC exclusion (carve-out evidence)

`SECURITY.md` §"`sparq-zk`/`sparq-zk-compose` — the v1 ZK verifier is NOT sound" and
§"`sparq-mpc` — cryptography deferred", plus `research/zk-soundness-audit.md` (12 confirmed
issues, 6 critical), are the design-of-record. **No control in this evidence pack cites the ZK
or MPC crates as a privacy guarantee.** Bead **sq-toze.35** gates any future privacy-by-
cryptography claim on the soundness fix.
