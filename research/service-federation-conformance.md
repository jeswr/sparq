# Design — SERVICE federation evaluation + in-process mock-server conformance harness

<!-- [OPUS-4.8] Design-for-review for epic sq-my8wd (under umbrella sq-6tykl). NO production
code in this PR. 🤖 SPARQ agent. -->

> 🤖 **SPARQ agent** — design record for @jeswr's review. DESIGN-FOR-REVIEW only.

**Status:** DESIGN / design-for-review. **Epic:** sq-my8wd (SERVICE federation + HTTP protocol
conformance), independent of the substrate — parallelises with sq-qonbz/pbz04.

**Recommendation in one line:** wire the W3C `sparql11/service` **evaluation** suite into the
conformance harness by standing up **real in-process `sparq-server` endpoints on `127.0.0.1:0`
loopback**, queried through the engine's existing HTTP `Transport`, rather than the current
`pub(crate)` canned-mock seam — this exercises the *whole* federated path (HTTP, content
negotiation, SRJ/SRX parsing, bind-join) end-to-end and doubles as the HTTP-protocol /
Service-Description / Graph-Store-Protocol conformance lane.

---

## 0. Premise check (honesty first)

Verified against the code:

- **SERVICE evaluation is real and feature-gated.** `crates/sparq-engine/src/service.rs` (1803
  LOC, `service` feature: `ureq` + `serde_json` + `quick-xml`), with `eval_service` (verbatim) and
  `try_bound_join_service` (VALUES-pushdown bind-join), SSRF egress policy, SRJ/SRX content-
  sniffing, RDF 1.2 / directional-literal reconstruction. The brief is accurate here.
- **Syntax conformance runs; evaluation does NOT.** `sparql11/syntax-fed` *is* run
  (`conformance/src/main.rs:95`); `sparql11/service` (evaluation) is explicitly documented as a
  gap "needs live remote endpoints" (`main.rs:520`). Service-Description and the CSV/TSV protocol
  tests are likewise listed as not-run. **Confirmed gap.**
- **The current mock is `pub(crate)` and engine-test-only.** The `Transport` trait
  (`service.rs:84`) is `pub(crate)`; `service_transport::install` (`exec.rs:2680`) is
  `pub(crate)`. The in-tree mocks (`Canned`, `Boom`, `RemoteGraph`) live in `exec.rs` tests. **A
  conformance harness in `sparq-conformance` cannot reach this seam today** — this is the central
  design constraint, and the brief's "no in-process server for SERVICE/protocol conformance" is
  exactly right.
- **`sparq-server` is embeddable in-process.** `sparq_server::{router, serve, AppState}` are
  **public** (`lib.rs:132`); the binary binds via `tokio::net::TcpListener::bind` in `main.rs:916`.
  So a test can build an `AppState` over a fixture graph, `serve` it on an ephemeral loopback port,
  and point a real `Transport` at it. **This is the key enabler the brief did not spell out.**

So the gap is real and the path is clear: the question is *which* transport the conformance
harness uses to reach the mock endpoint.

---

## 1. Design options for the mock-endpoint seam

### Option A — Real loopback `sparq-server` + real HTTP `Transport` (recommended)

The harness spins up one or more in-process `sparq_server::serve(...)` instances on `127.0.0.1:0`
(ephemeral ports), each loaded with the federated test's named-graph fixtures, then runs the
test's federated query through the **real** `ureq` HTTP `Transport` (with SSRF allowlist set to
the loopback host). **Pros:** exercises the *entire* stack — HTTP request/response, Accept
negotiation, SRJ *and* SRX parsing, bind-join VALUES blocks over the wire, SILENT error semantics
on a real connection-refused — so it is genuine protocol conformance, and it reuses public APIs
(no new `pub` surface in the engine). **Cons:** needs `tokio` + a port in the test; slightly
slower than an in-memory mock; loopback only (the SSRF allowlist must permit `127.0.0.1`).

### Option B — Promote the `Transport` seam to a public test-only API

Expose `service_transport::install` / the `Transport` trait behind a `test-transport` feature so
the conformance crate can inject a `RemoteGraph`-style in-memory mock. **Pros:** fast, no
network, no port. **Cons:** it does **not** test the HTTP layer, content negotiation, or SRJ/SRX
*parsing* — it tests the engine's join logic against a function that returns a relation. That is
already covered by the engine's own in-tree tests, so it adds little conformance value and widens
the public API for test-only reasons. **Not recommended as the primary path.**

### Option C — Both, layered (recommended overall)

Keep the existing in-engine canned-mock tests (fast, algebra-level, already proven multiset-equal
to verbatim). **Add** Option A as the conformance lane for the actual W3C `sparql11/service`
evaluation manifest + the protocol suites. The two layers test different things and both are
cheap to maintain. **This is the recommendation.**

---

## 2. Recommendation

Adopt **Option C**: real in-process loopback `sparq-server` endpoints (Option A) for the
`sparql11/service` evaluation lane and the HTTP-protocol / Service-Description / Graph-Store lanes,
layered over the existing engine-level canned-mock unit tests. Register the new lanes in the
conformance scoreboard with pinned floors, following the established `Runner` + floor-const +
`scoreboard_floors.rs`-guard pattern.

Scope, honestly bounded:

- **In scope:** `sparql11/service` evaluation (the manifest is present in rdf-tests, just not
  run); Service-Description GET `/sparql`; Graph-Store-Protocol PUT/POST/DELETE round-trips;
  CSV/TSV result-format protocol tests.
- **Caveats to keep honest:** the variable-endpoint (`SERVICE ?ep`) and high-cardinality cases
  have a **no per-request cap** risk (brief: a 10M-distinct-`?ep` query spawns 10M requests, only
  bounded cooperatively post-HTTP). The harness should not paper over this; if the suite exercises
  it, surface it (and consider a follow-up bead for a per-query remote-request cap). Remote-result
  materialisation is non-streaming (full relation into memory) — fine for conformance fixtures,
  noted as a scaling caveat, not a conformance failure. *(Both since addressed: the per-query
  remote-request cap landed as `sq-b93pv`, and remote-result consumption is streaming/bounded as
  of `sq-my8wd.4` — rows are interned to id-level bindings as parsed, never held as a
  whole-document DOM or term-level relation. [FABLE-5])*

---

## 3. Soundness / safety notes

- **SSRF policy must stay strict even for loopback tests.** The harness sets the allowlist to
  exactly the loopback host:port it spun up; it must NOT disable the egress filter globally (that
  would regress the DNS-rebinding invariant the engine guarantees by installing its own resolver).
  The test allowlists the *specific* ephemeral endpoint.
- **SILENT semantics are algebra, not optimisation.** The harness must test that a refused
  connection under `SERVICE SILENT` yields the join identity (bindings preserved) and a non-SILENT
  failure propagates — this is a correctness property, verifiable by pointing at a closed port.
- **No privacy/ZK/MPC claim here.** This is plain federated SPARQL over HTTP. The `sparq-fedplan`
  cost-based planner and the `sparq-fedplan-mpc` seam are **out of scope** for this epic — the MPC
  seam has deferred Phases 5–6 (`SeamError::Deferred`, gated on sq-qhy4) and is semi-honest only;
  nothing here integrates it or makes any privacy guarantee.

---

## 4. Phased plan (each phase = a future bead under sq-my8wd)

1. **In-process test harness helper** — a reusable fixture that loads a graph, `serve`s it on
   `127.0.0.1:0`, returns the bound URL, and tears it down. *Acceptance:* a smoke test runs a
   trivial federated query against it end-to-end; SSRF allowlist scoped to the ephemeral port.
2. **Wire `sparql11/service` evaluation into the conformance harness** — walk the manifest, stand
   up the per-test endpoints, run the federated query through the real `Transport`, compare with
   the result oracle. *Acceptance:* a pinned `SERVICE_EVAL_FLOOR` const + scoreboard row + guard
   entry; CI lane green. *Depends on phase 1.*
3. **HTTP-protocol conformance lane** — SPARQL Protocol GET/POST + CSV/TSV result-format tests
   against the in-process server. *Acceptance:* protocol floor pinned + scoreboard row. *Depends
   on phase 1.*
4. **Service-Description + Graph-Store-Protocol lane** — assert the `sd:` document advertises the
   right languages/formats; GSP PUT/POST/DELETE round-trip. *Acceptance:* SD/GSP floor pinned +
   scoreboard row. *Depends on phase 1.* (These are behind the existing `federation-descriptors`
   feature; keep them opt-in so the default wasm/byte ratchets are unaffected.)
5. **(Follow-up bead, honest gap)** — per-query remote-request cap for high-cardinality
   `SERVICE ?ep`. *Acceptance:* a runaway-endpoint query is bounded before dispatch, not just
   cooperatively after. *Independent.*

---

## 5. Open questions for the maintainer

1. **Loopback server (Option A/C) vs public test-transport (Option B)?** I recommend the real
   loopback server for genuine protocol coverage; confirm you are happy with `tokio` + an
   ephemeral port inside the conformance crate's test deps (it is already a workspace dependency).
2. **Per-request remote cap** (phase 5): in-scope for this program or a separate hardening bead?
3. **CSV/TSV + SD/GSP**: include all protocol suites now, or land `sparql11/service` evaluation
   first and add the protocol lanes as a fast follow?
