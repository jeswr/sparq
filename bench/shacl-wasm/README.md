# shacl-wasm — browser SHACL same-runtime comparison

<!-- [FABLE-5] sq-i858h (epic sq-hmd7l) — the browser half of the SHACL
     competitor story. The NATIVE same-box harness is
     scripts/bench/shacl-same-box.sh (sparq vs pySHACL vs Jena); this suite
     compares the WASM bundle against its registered JS peer instead. -->

Compares **`sparq-shacl-wasm`** (the wasm-pack'd `Validator`,
[`crates/sparq-shacl-wasm`](../../crates/sparq-shacl-wasm/README.md)) against
**Zazuko `rdf-validate-shacl`** (`bench/competitors.json` id
`rdf-validate-shacl`, kind `js-lib`) — **both in the same Node process**, the
natural runtime for sparq's browser SHACL story.

```sh
bash run.sh --smoke   # acceptance: exit 0 iff the agreement gate is green
bash run.sh           # best-of-$ITERS (default 3) + results/ envelope
```

## Workloads — the SAME (data × shapes) pairs as `bench/shacl`

The **committed** shape workloads are reused verbatim (zero overlap — this
suite adds no shapes): [`bench/shacl/shapes/*.ttl`](../shacl/shapes) +
[`bench/shacl/shapes-sparql/sparql_heavy.ttl`](../shacl/shapes-sparql). The
data graph is a small **vendored micro-ABox** (`data/abox.ttl`, LUBM
vocabulary, hand-authored) so every violation constant is **hand-countable**
(derivations in `expected.tsv`'s header) and the harness needs no LUBM
generator / Java in a Node-only environment. Scale-tier corpora for the
browser story are tracked separately (sq-hmd7l.40).

## The gate (HARD) vs timing (ADVISORY)

**INVARIANT: no timing row without a green per-workload `#violations` +
`conforms` AGREEMENT gate** between the two engines, each ALSO matching the
hand-derived `expected.tsv` constant (so both-engines-broken can't pass as
"agreement"). Both engines are reduced identically — `report.results.length` /
`report.conforms`, the `scripts/bench-adapters/js_shacl_adapter.mjs` +
`shacl_report_count.py` contract.

The two `sh:sparql` workloads (`sparql_constraint`, `sparql_heavy`) are
**sparq-only**: `rdf-validate-shacl` implements SHACL Core only (no SHACL-SPARQL,
W3C SHACL §5.2), so its column is **absent** there — a capability gap recorded
honestly, never a fabricated `0`. sparq is still self-asserted vs
`expected.tsv` on them. The four core workloads are single-route, so sparq's
per-occurrence counting and a dedup engine agree (see the
[`bench/shacl` README caveat](../shacl/README.md#competitors)).

**Timing** is one-shot **end-to-end** (parse data + shapes + validate +
reduce) best-of-N for BOTH engines — the stateless wasm `Validator` cannot
hoist the parse, so the peer is charged the same work; the peer's
validate-only time on pre-parsed datasets is an extra advisory column. With
`FEATURES=stateful` (sq-01xlp) the artifact exports the opt-in pre-parsed
`ParsedGraph` handle and the harness records the SYMMETRIC sparq column
(`sparq_validate_only_us`, counts cross-checked against the one-shot every
iteration); without it the column is absent, never a fabricated 0. All
timings are NON-canonical on the work box (`canonical:false` in the envelope;
`CANONICAL=1` only on a dedicated quiet EC2 box).

**Bundle bytes** are the second, **deterministic** column: the
`wasm-pack --release` nodejs-target artifact (default features — no
`shacl-af`, no `stateful`) byte + gzip-9 sizes, recorded per toolchain in the
envelope. A `FEATURES=…` build flags its bytes as NON-canonical in the
envelope (the deterministic record is the default-features artifact). The
pre-bindgen ratchet (`scripts/ci-bench.sh` `wasm_bundle_bytes`) is
deliberately untouched.

## Outputs

One `bench/canonical-competitor-results`-shaped JSON envelope per run in the
git-ignored `results/` (stdout carries the gated
`<workload>\t<engine>\t<violations>\t<e2e_best_us>` rows). Peer npm packages
are **gather-only** (`/tmp/shacl-wasm-deps` scratch by default — never
committed; the exact pinned version is recorded in the envelope at gather
time, per `bench/competitors.json`).

First-read gap record: [`research/gap-shacl-wasm-2026-07.md`](../../research/gap-shacl-wasm-2026-07.md).
