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
bash run.sh --smoke        # acceptance: exit 0 iff the agreement gate is green
bash run.sh                # best-of-$ITERS (default 3) + results/ envelope
bash run.sh --bundle-only  # ONLY the minified-bundle byte column
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

### The PEER half of the byte column — `bundle.mjs` (sq-c6c2s)

The first-read record made **no byte-ratio claim**, because the only peer
number available was the peer's *unpacked npm footprint* — not a wire size.
[`bundle.mjs`](./bundle.mjs) builds the comparable number: an **esbuild**
(minify + tree-shake, `platform=browser`, ESM) bundle of the peer stack,
compared on **gzip-9 wire bytes** against the wasm artifact. It runs as
`run.sh` stage 2b and is folded into the envelope at
`bundle_bytes.peer_minified_bundle`; a missing esbuild / peer install leaves
that sub-column **ABSENT with the reason recorded**, never a fabricated 0, and
never fails the run.

Two peer variants, because only one is comparable:

| variant | contents | why |
|---|---|---|
| `validator-only` | `rdf-validate-shacl` alone | lower bound — it cannot read a Turtle document, so it is not a runnable browser SHACL app |
| `app-parity` | validator + the RDF/JS factory + `@rdfjs/parser-n3` stack (exactly `harness.mjs`'s imports) | **the like-for-like column** — the wasm artifact carries its own Turtle parser, so "parse two documents, then validate" is the capability being sized |

Both peer numbers are **lower bounds** on purpose: Node core imports reached by
the RDF/JS stack are stubbed to empty modules rather than polyfilled (the
stubbed list is recorded per variant), so a real browser build ships more.
Understating the peer is deliberate — the comparison must never flatter sparq.
And the ratio prices a **capability gap** as well as an implementation one:
the peer is SHACL Core only, while the sparq artifact also carries
SHACL-SPARQL and the SPARQL engine behind it.

These bytes carry no wall clock, so unlike the timing rows they need no quiet
box — but the peer half is **not reproducible and not canonical**: `run.sh`
installs the peer stack *and esbuild* from bare package names with no committed
lockfile, so a later gather can resolve different peer, transitive, or bundler
code and emit different bytes. The recorded `package_versions` are **provenance
for that run, not a recipe to re-derive it**; pinning them properly means a
committed manifest + lockfile installed with `npm ci` — the posture
[`bench/wasm-compare/bundle.mjs`](../wasm-compare/bundle.mjs) already has for
its peer (exact version + tarball sha256). Raw `bytes` are the more stable
metric; `gzip9_bytes` is Node zlib and therefore zlib-version-dependent.

**Size-trim levers** (`TRIM_SWEEP=1`, `WASM_OPT_PROBE=1`) are described in
[`research/gap-shacl-wasm-2026-07.md`](../../research/gap-shacl-wasm-2026-07.md#size-trim-levers-sq-c6c2s).

## Outputs

One `bench/canonical-competitor-results`-shaped JSON envelope per run in the
git-ignored `results/` (stdout carries the gated
`<workload>\t<engine>\t<violations>\t<e2e_best_us>` rows). Peer npm packages
are **gather-only** (`/tmp/shacl-wasm-deps` scratch by default — never
committed; the exact pinned version is recorded in the envelope at gather
time, per `bench/competitors.json`).

First-read gap record: [`research/gap-shacl-wasm-2026-07.md`](../../research/gap-shacl-wasm-2026-07.md).
