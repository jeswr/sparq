<!-- [OPUS-4.8] sq-toze.32 (PR-G2) — Query-log / PII data-minimisation posture + operator
     guidance. OPERATOR-FACING, NOT a certification claim. The deploying organisation is the
     GDPR controller; sparq is the technical means. Accurate to the REAL logging behaviour
     (default vs --verbose vs --audit-log), verified in-code. Re-review when Fable returns. -->

# Query-log / PII data-minimisation posture — operator guidance

> **What this is.** A factual description of **what sparq actually logs** (by default, with
> `--verbose`, and with the opt-in `--audit-log`), where personal data can surface in those
> logs, the **data-minimisation posture the engine ships by default**, and **operator guidance**
> to keep logs PII-clean.
>
> **What this is NOT.** This is **not** a certification or compliance claim, and not a statement
> that "sparq is GDPR/27701/SOC 2 compliant". **sparq is a data *engine*; the deploying
> organisation is the GDPR controller** (the ISO 27701 PII controller / SOC 2 entity) and owns
> the log-retention *policy*, log access control, and the attestation. This document describes
> the engine **mechanism** and the levers the operator drives. Everything labelled `<FILL-IN>`
> is a deployment/policy value this guidance deliberately does not assume.
>
> See [`README.md`](./README.md) for the operator-vs-engine split, [`controls.md`](./controls.md)
> P-2 (data minimisation) / P-8 (auditability) / P-12 (error-body hygiene), the
> [`retention-erasure-runbook.md`](./retention-erasure-runbook.md) for purging logs as part of an
> Art. 17 erasure, and [`../data-flow.md`](../data-flow.md) for every place the binary can touch
> data.

## 0. The PII problem in a SPARQL log (why this matters)

A SPARQL query is not neutral metadata — **it can embed personal identifiers directly in its
text**. For example:

```sparql
SELECT ?record WHERE { ?record :patient <http://hospital.example/patient/12345> }
ASK { <http://example.org/person/alice> :diagnosis ?d }
SELECT ?x WHERE { ?x :email ?e . FILTER(?e = "alice@example.org") }
```

The IRI `…/patient/12345`, the WebID `…/person/alice`, and the literal `"alice@example.org"`
are personal data **inside the query string**. So is a fragment of the **loaded RDF data**
echoed back in a parse error. Therefore "what does the server log" is a direct GDPR Art. 5(1)(c)
(minimisation) / Art. 5(1)(f) (integrity & confidentiality) question, and the answer below is
**accurate to the code**, not aspirational.

## 1. What sparq logs — by mode (verified in-code)

sparq installs **no logging subscriber at all** unless the operator opts in. A `tracing`
subscriber is installed **only** when `--verbose` is set, or (when the `audit-log` cargo feature
is compiled in) when `--audit-log` is set — otherwise `want_subscriber` is false and emitted
records go nowhere (`crates/sparq-server/src/main.rs` — `want_subscriber = config.verbose [||
config.audit_log]`, then `if want_subscriber { tracing_subscriber::fmt()…init() }`).

| Mode | Flag (default) | Request log? | What is recorded | PII exposure |
|---|---|---|---|---|
| **Default** | none (`verbose:false`) | **No** | Nothing per-request. No subscriber installed. Only aggregate Prometheus metrics if `/metrics` is scraped (counts/histograms, no content). | **None per-request.** Minimisation-clean by default. |
| **Verbose** (redaction ON — default) | `--verbose` | **Yes** (`TraceLayer`) | Per-request/response `tracing` span at `debug` (`tower_http=debug,sparq_server=debug`) — HTTP **method, status, latency**, and a **redacted URI**: path verbatim, query string → `<redacted len=N fp=…>`. | **Minimised.** The query text is replaced by a length + non-reversible fingerprint (`sq-toze.34`). Residual = metadata (method/endpoint/status/size/timing), not content. |
| **Verbose + full requests** | `--verbose --log-full-requests` | **Yes** (`TraceLayer`) | As above but the **raw URI** verbatim. | **Yes** — see §2. The URI of a **GET** `/sparql?query=…` carries the **full query text**. Deliberate operator opt-out of redaction (`SPARQ_LOG_FULL_REQUESTS=1`). |
| **Audit** (opt-in build) | `--audit-log` (feature `audit-log`) | Structured access record per query/update | `op` class, **non-reversible query fingerprint**, **requester fingerprint** (token hash or `anonymous`), decision, status, duration — **never** the raw query text or token. | **Minimised by design** — fingerprints, not content (§3). |
| **Metrics** | `/metrics` endpoint | n/a | Aggregate counters/histograms/gauges only (per-endpoint/status counts, latency buckets, triple/subscription gauges). | **None** — no query text, no result rows, no client identifiers (`crates/sparq-server/src/metrics.rs`). |

**Client IP / source address.** The engine does **not** wire `ConnectInfo`/`SocketAddr` into
any log or audit record — there is no per-request client-IP logging in sparq itself. (A reverse
proxy / gateway in front of sparq may log client IPs in *its* access log; that is the operator's
component and the operator's retention concern.) The opt-in audit record's `requester` field is a
**token fingerprint** (FNV-1a hash of the presented Bearer token) or the literal `anonymous` —
never an IP, never the token itself (`crates/sparq-server/src/audit.rs` `requester_identity`).

**Request bodies (POST).** `TraceLayer::new_for_http()` (installed only under `--verbose`) logs
the **request span** built from method/URI/version + the response status — it does **not** log
the POST request body. So a query/update submitted via **POST** does not have its body written to
the `--verbose` request log. The exposure under `--verbose` is the **GET query string** (which
contains the query text) and any **error body** the handler returns (see §2 / §4 / P-12).

## 2. Where personal data can surface under `--verbose`

When the operator turns on `--verbose`, the request log can contain personal data in two places:

1. **The GET query string.** A `GET /sparql?query=<URL-encoded SPARQL>` request logs the **URI**
   including the query string, so the **full SPARQL query text** (and any identifier/literal it
   embeds, per §0) would land in the log line. **[OPUS-4.8] sq-toze.34 — now redacted by default:**
   with `--verbose`, the request log redacts the URI query string to a `<redacted len=N fp=…>`
   length + non-reversible fingerprint placeholder (gap **PR-G4** in [`gap-register.md`](./gap-register.md)
   is now CLOSED). An operator only sees the verbatim query text if they deliberately pass
   `--log-full-requests` (`SPARQ_LOG_FULL_REQUESTS=1`). **Honest boundary:** this is log-*content*
   redaction, not anonymity — the redacted line still records method, endpoint, status, a size
   signal and timing. (POST query bodies were never logged by `TraceLayer`; that remains true.)
2. **Error bodies (P-12).** Several parse/validation error bodies historically echoed caller
   input (SPARQL query text or fragments of the loaded RDF data) to the caller. This was the
   most material privacy item and is **now sanitized at the HTTP boundary** by **PR #241** (the
   error-body bodies are generic by default; verbose diagnostics are gated), closed under
   beads `sq-cz89` / `sq-zg0u`. See [`controls.md`](./controls.md) P-12 and the consolidated
   [`../gap-register.md`](../gap-register.md). The residual is the request-log query string above.

## 3. What is minimised by default (the posture)

The engine ships **data-minimisation-by-default** for logs:

- **No request log unless asked.** Default `verbose:false` → no subscriber → no per-request
  records (P-2). The operator must take a deliberate action (`--verbose`) to log requests.
- **[OPUS-4.8] sq-toze.34: request log redacts content by default.** Even with `--verbose`, the
  request log replaces the URI query string (the GET query-text vector) with a length +
  non-reversible FNV-1a fingerprint, so the query content is not written to the log. Logging the
  verbatim URI is a deliberate opt-out (`--log-full-requests` / `SPARQ_LOG_FULL_REQUESTS=1`). This
  is log-*content* redaction, not anonymity — method/endpoint/status/size/timing metadata remains
  (`crates/sparq-server/src/redact.rs`; `crates/sparq-server/tests/log_redaction.rs`).
- **Audit trail is fingerprints, not content.** The opt-in `--audit-log` records a **stable
  non-reversible FNV-1a fingerprint** of the normalised query text and of the requester token —
  enough to correlate repeated identical queries or attribute activity to a caller, **without
  persisting the query content or the secret** (`crates/sparq-server/src/audit.rs` module docs +
  `query_fingerprint` / `requester_identity`). The audit log is also **off by default** even when
  the feature is compiled in.
- **Metrics are aggregate-only.** `/metrics` exposes counts/histograms/gauges, never query text,
  result rows, or client identifiers (`crates/sparq-server/src/metrics.rs`).
- **Error bodies are generic by default.** Post-PR #241, default-mode error bodies do not echo
  query text or loaded-data fragments (P-12).
- **No telemetry / phone-home.** No dependency or code path exfiltrates query content or usage
  data to a third party (P-2).

This is a genuinely minimisation-friendly default: an out-of-the-box sparq server writes **no
per-request personal data anywhere**. With `--verbose` the request log now **redacts the GET
query string by default** (sq-toze.34 / PR-G4 closed); the only way query text reaches the log is
the deliberate `--log-full-requests` opt-out. The residual under a redacted `--verbose` log is
**metadata** (method/endpoint/status/size/timing), not content — log-content redaction is not
anonymity.

## 4. Operator guidance — keep logs PII-clean

The deploying operator (the controller) owns the log-handling policy. Recommended practice:

1. **Default to no request log in production.** Leave `--verbose` **off** unless you have a
   specific operational need. The minimisation-clean default is the recommended production
   posture.
2. **If you need request logging, avoid logging query text.**
   - Prefer **POST** for queries (the POST body is not written to the `--verbose` request log;
     only GET query strings are).
   - When the `--log-redact-queries` control lands (gap **PR-G4** / bead **sq-toze.34**), enable
     it so the request log carries method+path+status, not the SPARQL text. Until then, **scrub
     or hash query strings in your log pipeline** (e.g. drop the `query=` parameter from URIs
     before shipping logs to long-term storage).
3. **Prefer the opt-in audit log for an access trail.** If you need an Art. 5(2) accountability /
   SOC 2-monitoring access trail, use `--audit-log` (build with the `audit-log` feature) rather
   than `--verbose`: it records the **fingerprint + op class + decision**, which is an
   access trail that is PII-minimised by construction. (Note the audit log records HTTP status as
   the outcome and a query *fingerprint*, not the full query or row counts — correlate with the
   request log only if you have accepted the request-log PII trade-off above.) See
   [`gap-register.md`](./gap-register.md) PR-G2.
4. **Set the log level deliberately.** `RUST_LOG` overrides the default filter. Do not raise
   `sparq_server`/`tower_http` to `debug`/`trace` in production unless you have accepted the
   query-text exposure and applied redaction. Keep the level at the minimum needed.
5. **Control access to logs.** Logs that contain query strings inherit the sensitivity of the
   data they reference. Apply the operator's access-control + least-privilege policy to the log
   store (who can read the logs), and treat the log store as in-scope for the same
   confidentiality controls as the data store. `<FILL-IN: operator log-access-control policy>`.
6. **Set a log-retention period and enforce it.** sparq has **no built-in log retention** — logs
   go to stdout/stderr (or wherever the operator's `tracing` sink / container runtime routes
   them) and live for as long as the operator's log pipeline keeps them. Define a retention
   window and an aging-out / purge mechanism in the operator's policy. `<FILL-IN: operator
   log-retention period>`.
7. **Include logs in data-subject erasure.** If `--verbose` was on, request logs may contain a
   subject's identifier (in a GET query string). An Art. 17 erasure must therefore also **scrub
   the logs** — see the [`retention-erasure-runbook.md`](./retention-erasure-runbook.md) §7d and
   the §9 checklist (step 9). `<FILL-IN: operator log-scrub procedure>`.
8. **Front-of-server logs are the operator's.** A reverse proxy / API gateway / `sparq-solid`
   layer in front of sparq may log client IPs, WebIDs, and full request lines in its **own**
   access log. That log is the operator's component and the operator's minimisation + retention
   + access-control concern — sparq has no visibility into it (boundary **B3** — see
   [`README.md`](./README.md)).

## 5. Summary table — operator decisions

| Decision | Default | Recommendation | Reference |
|---|---|---|---|
| Request logging | OFF (`--verbose` not set) | Keep OFF in prod; if needed, prefer POST + redact GET query strings | §1, §4.1–4.2; PR-G4 |
| Access trail | none | Use `--audit-log` (fingerprints, not content) over `--verbose` | §3, §4.3; PR-G2 |
| Log level | request log at `debug` when verbose | Minimum level needed; no `trace` in prod | §4.4 |
| Log access control | (operator's log store) | Least-privilege; treat log store at data sensitivity | §4.5 `<FILL-IN>` |
| Log retention | none built-in | Define + enforce a retention window | §4.6 `<FILL-IN>` |
| Erasure scope | logical store only | Scrub logs too if `--verbose` was on | §4.7; runbook §7d |

## References

- `crates/sparq-server/src/main.rs` — subscriber installed only when `--verbose` /
  `--audit-log`; default `verbose:false`; the `RUST_LOG`-overridable default filter.
- `crates/sparq-server/src/http.rs` — `TraceLayer::new_for_http()` mounted only under
  `if config.verbose`; the GET query string lands in the logged URI.
- `crates/sparq-server/src/audit.rs` — the opt-in access audit log: non-reversible
  query/requester fingerprints, decision, status; never raw query text or token.
- `crates/sparq-server/src/metrics.rs` — aggregate-only Prometheus metrics (no content).
- [`controls.md`](./controls.md) — P-2 (minimisation), P-8 (auditability), P-12 (error/log
  hygiene).
- [`gap-register.md`](./gap-register.md) — PR-G2 (no built-in structured audit log), PR-G4 (no
  request-log redaction control, bead sq-toze.34).
- [`retention-erasure-runbook.md`](./retention-erasure-runbook.md) — §7d / §9 step 9: scrub logs
  as part of an Art. 17 erasure.
- [`../data-flow.md`](../data-flow.md) — the data-touch map (where the binary can touch data).
