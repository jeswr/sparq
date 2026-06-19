# Benchmark-coverage epic (sq-5o5): decomposition register

> 🤖 SPARQ agent — research/design-for-review record. Authored by Opus 4.8
> (`[OPUS-4.8]`; Fable unavailable — flag for re-review when Fable returns). No
> implementation; this catalogues the *remaining* atomic gaps under sq-5o5 and the
> beads created for them.

## Purpose

sq-5o5 ("Benchmarks: full coverage + prettier dashboard") asks for four things:
(1) run all reasonable-time benches per-commit; (2) well-known suites at CI scale;
(3) regression coverage of every crate + every SPARQL feature; (4) a prettier
dashboard with a latest-commit summary table. This record decomposes the genuinely
*remaining* gaps into small, independently-launchable beads, mirroring the sq-bif
test-coverage decomposition (one bead per crate/family, `area:<crate>` labelled).

## Method + correction to the brief's premise

The brief said `scripts/drift-scan.py` "already reports bench-missing + dashboard-row
gaps". Verified against the actual tree (origin/main `3d99ef29`):

- A first drift-scan invocation in a freshly-checked-out worktree returned 23 items
  (6 bench-missing, 16 dashboard-row, 1 conformance-split). This output did **not**
  reproduce. Two subsequent clean runs (`python3 scripts/drift-scan.py`, with and
  without `-B`) reproducibly returned **5 items, all `dashboard-row`** — the first
  run read a transiently-inconsistent tree mid-checkout.
- **Correction: there are no real `bench-missing` gaps.** Every crate the stale run
  named (`sparq-fedclient`, `sparq-fedplan`, `sparq-prov`, `sparq-reason-wasm`,
  `sparq-zk`, `sparq-zk-compose`) carries `publish = false`. `scan_bench_missing`
  exempts `publish = false` stubs by design (mirrors gate G1's `crate_is_public`),
  so the scanner correctly does not flag them on a clean run. Every *public* crate
  has a registered bench `source` ref, except `sparq-core`/`sparq-engine`, which are
  in `BENCH_EXEMPT_CRATES` (their query logic is exercised via the CLI /
  operator-coverage suites). So `sq-layq`'s "9 bench-missing" premise is stale —
  those nine are not gaps and were **not** beaded here.
- **Correction: the `conformance-split` item is closed** — `sparq-solid`'s ratchets
  are consolidated into the central `sparq-conformance` scoreboard
  (`CONFORMANCE_CONSOLIDATED`). Not a gap; not beaded.
- SPARQL-feature coverage (sq-5o5 item 3) is effectively complete: the
  `operator-coverage` suite (`bench/operators/queries/`) carries 28 queries spanning
  BGP / star / chain / triangle / UNION / OPTIONAL (incl. not-bound) / MINUS /
  numeric+string+IN+EXISTS FILTER / BIND / VALUES / aggregate+GROUP+HAVING / DISTINCT
  / ORDER+LIMIT+OFFSET / all property-path forms (incl. negated property sets) /
  subquery / ASK / CONSTRUCT / DESCRIBE. No per-operator gap beaded.
- The well-known suites (LUBM, WatDiv, SP2Bench, BSBM, DBPSB, Deep Taxonomy, OWL
  sameAs) plus SHACL / Full-Text / Vector-ANN / GeoSPARQL / RSP-QL / HDT all run
  per-commit in `scripts/ci-bench.sh` and already have FEATURED_SUITES dashboard
  rows. Their EC2/full-scale heavy tiers are already beaded (see "already covered").
- Note: the epic gated its implementation wave on the perf-gate ratchet `sq-52e`,
  which is now **closed/done** — so the wave below is unblocked.

## Remaining real gaps → beads

The only machine-detected drift is **5 `dashboard-row` families** whose registered
bench has no `FEATURED_SUITES` row in `bench/dashboard/dashboard.js`. Each is a
single-family *disposition* decision (mirror sq-5o5.2): either add a forward-looking
capability row, or mark the bench `featured = false` in `bench/benchmarks.toml`.
Each touches one family only, so all five are parallel-safe. All are
**LOCALLY-DRAINABLE**: the work is a config/doc edit verified by re-running
`drift-scan.py` to exit 0; it needs no wall-clock, no quiet box, and no canonical
numbers. (Any head-to-head competitor numbers, if a family is promoted to a card,
are owned by separate EC2-blocked beads and are not required to clear the drift.)

| Family / crate | Registered bench | Current state | Bead | Run tier |
|---|---|---|---|---|
| sim (`sparq-sim`) | `sim-olympics-eval` | bench exists, no dashboard row | sq-5o5.5 | LOCAL |
| introspect (`sparq-introspect`) | `introspect-olympics` | bench exists, no dashboard row | sq-5o5.6 | LOCAL |
| nlq (`sparq-nlq`) | `nlq-offline-bench` | bench exists, no dashboard row | sq-5o5.7 | LOCAL |
| mpc (`sparq-mpc`) | `mpc-bench-matrix` (counting tier, deterministic) | bench exists, no dashboard row | sq-5o5.8 | LOCAL |
| wasm (`sparq-wasm`) | `wasm-bundle` (size gate, deterministic) | bench exists, no dashboard row | sq-5o5.9 | LOCAL |

Honesty caveats carried into the bead bodies:

- **mpc** (sq-5o5.8): `mpc-bench-matrix` is a *modelled* counting tier
  (`quiet_box_sensitive = false`), not a measured competitor card; `featured = false`
  is the likely-correct disposition. MPC is semi-honest-only and the ZK estate has
  **no external accredited-cryptographer sign-off** (sq-qhy4 pending) — any dashboard
  text stays caveated and the privacy-claims CI gate must pass.
- **wasm** (sq-5o5.9): `wasm-bundle` is a deterministic bundle-size byte gate; the
  query path is already measured via the engine suites. Distinct from the *proposed*
  wasm query micro-bench `wasm_query_us` (sq-5pnl).
- **nlq** (sq-5o5.7): the trend-only disposition makes no perf claim; live
  exec-accuracy numbers on a canonical host are owned by sq-qidj / sq-g0lw
  (EC2-blocked) and are not required for the dashboard-row drift.

## Already covered — not re-beaded

These would-be gaps already have open or closed beads; no new bead was created:

- Well-known-suite **EC2/heavy tiers**: sq-w0ax (WatDiv/BSBM/LUBM), sq-uubq
  (SP2Bench full-scale), sq-yc1q (BSBM full mixes), sq-xin1 (DBPSB heavy), sq-5o5.4
  (hnswlib/faiss gather on a SIFT/GloVe corpus) — all EC2-blocked, already tracked.
- Competitor numbers at CI scale: sq-xwnl; broaden the matrix beyond Oxigraph: sq-5gg.
- GenAI nightly perf gates (introspect/sim/ANN/nlq): sq-v4if (blocks sq-5o5).
- wasm query micro-bench: sq-5pnl; SHACL validate bench: sq-cvwl; serve/server
  micro-metric: sq-i8jn; FedBench (note-only, blocked on SERVICE): sq-zdv9.
- Per-crate **correctness** coverage is a different epic (sq-bif), not sq-5o5.
- Umbrella `sq-layq` overlaps this decomposition; its `bench-missing` (9 crates) and
  `conformance-split` premises are stale per the corrections above, so its remaining
  actionable content is exactly the 5 dashboard-row dispositions now atomised here.

## Phased plan (ordered future beads)

1. sq-5o5.5 — sim dashboard-row disposition (LOCAL).
2. sq-5o5.6 — introspect dashboard-row disposition (LOCAL).
3. sq-5o5.7 — nlq dashboard-row disposition (LOCAL).
4. sq-5o5.8 — mpc dashboard-row disposition, likely `featured = false` (LOCAL).
5. sq-5o5.9 — wasm dashboard-row disposition, likely `featured = false` (LOCAL).

All five are independent and can run in one parallel wave; each closes one
`drift-scan` `dashboard-row` item and is verified by `drift-scan` reaching exit 0
for that family. No EC2 beads were newly required — the heavy tiers are already
tracked above.

## Open questions for the maintainer

- For sim / introspect / nlq (sq-5o5.5–.7): **promote to a featured competitor card,
  or mark `featured = false`?** Promotion implies committing to gather competitor
  numbers (EC2), which is a larger commitment than clearing the drift; `featured =
  false` (trend-only, the sq-5o5.2 precedent) clears it with no perf claim. Default
  recommendation: `featured = false` unless you want these on the head-to-head card.
- Should `sq-layq` be closed/retitled now that its `bench-missing` and
  `conformance-split` premises are stale and its dashboard-row content is atomised
  into sq-5o5.5–.9?

## Uncertainties

- I shallow-checked the sim / introspect / nlq bench harnesses (confirmed each has a
  registered `[[benchmark]]` and that the drift is purely the missing dashboard row);
  I did not read each crate's `examples/`/`benches/` source line-by-line, so the
  promote-vs-`featured=false` recommendation is a disposition call, not a depth audit
  of each harness's metric quality.
- The 23-item first drift-scan run was an unreproducible transient (mid-checkout
  tree). I treated the reproducible 5-item run as ground truth; if a future scan
  surfaces `bench-missing` for a crate that has *lost* its `publish = false`, that is
  new work outside this decomposition.
