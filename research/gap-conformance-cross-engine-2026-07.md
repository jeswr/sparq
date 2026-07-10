<!-- [FABLE-5] sq-hmd7l.22 — cross-engine CONFORMANCE pass-rate record: sparq's
scoreboard floors (origin/main) vs the peers' PUBLISHED EARL / implementation-report
results. Peer cells only from pinned published sources with provenance; a peer
without a published number is an explicit NOT-PUBLISHED cell, never a sparq win. -->

# Gap record — cross-engine conformance pass-rates (2026-07)

**Axis:** #22 in `research/comparative-benchmarking-everything.md` §6 (epic `sq-hmd7l`, bead `sq-hmd7l.22`).
**Status:** DRAFT — peer cells pending research-agent citations.

## 1. Method and honesty rules

- **sparq's numbers are the ACTUAL CI-enforced scoreboard floors** on `origin/main`
  (commit `6df375096`): the central registry `crates/sparq-conformance/src/scoreboard.rs`
  (`SUITES`), the lib-side JSON-LD floor consts
  (`crates/sparq-conformance/src/floors/*.rs`), and the crate-local floor consts the
  guard test `tests/scoreboard_floors.rs` pins to the registry. Reproduce with
  `cargo run --release -p sparq-conformance --bin sparq-conformance-scoreboard`.
- **Peer numbers come ONLY from their published EARL / implementation reports** (or,
  where flagged, a published third-party benchmark) — cited with URL + retrieval
  date. Nothing is estimated, and **a peer without a published number is an explicit
  NOT-PUBLISHED cell, never a sparq win by omission.**
- **Suite-version mismatches are flagged per row.** A pass-rate is only strictly
  comparable at the same suite revision; most published EARL predates the current
  consolidated `w3c/rdf-tests` tree.
- **Verdicts use the fixed vocabulary** from
  `research/comparative-benchmarking-everything.md` §5.3: CLEARLY-AHEAD /
  AHEAD-BUT-NOT-OOM / PARITY / BEHIND / NOT-MEASURED / NOT-COMPARABLE.
- Conformance pass-rates are **correctness counts, not performance figures**; no
  latency/throughput numbers appear in this record.

## 2. sparq's scoreboard (verified on origin/main)

Sources: `scoreboard::SUITES` registry rows + the runner-local floor consts named
below; suite pins from `scripts/fetch-conformance.sh` (`w3c/rdf-tests`
`f25dbc092c65`), `scripts/fetch-jsonld-tests.sh` (`w3c/json-ld-api` `8654ac22b6cf`),
`scripts/fetch-jsonld-framing-tests.sh` (`w3c/json-ld-framing` `3bf782ba9a40`),
`crates/sparq-shacl/fetch-shacl-tests.sh` (`w3c/data-shapes` `b6e73695d619`),
`scripts/fetch-inference-suites.sh` (`w3c/N3` `23ccf3d56b25`).

| Suite | sparq floor (CI-enforced) | Denominator at the pin | Source const |
|---|---|---|---|
| W3C SPARQL 1.0/1.1/1.2 query+update+syntax | 1229 (1225 pass + 4 documented divergences, 0 fail, 0 skip) | 1229/1229 of the full harness scope | `ci.yml` `RATCHET=1229`; `FINDINGS.md` round 5 |
| Inference (rdf-mt / OWL 2 RL / N3 / sparql11-entailment / rdf-turtle) | 1967 (1950 pass + 17 documented divergences, 0 fail) | 100% of run scope (out-of-scope regimes excluded + reported) | `ci.yml` `RATCHET=1967` |
| W3C SHACL core | 98 pass | 98/98 `sht:Validate` cases at the pin | `w3c_core.rs` `BASELINE_PASS = 98` |
| W3C SHACL-SPARQL (node+property) | 5 pass | 5/5 in the node+property sub-suites | `w3c_sparql.rs` `SHACL_SPARQL_FLOOR = 5` |
| W3C JSON-LD 1.1 toRdf | 413 pass | of 467 manifest entries | `floors/to_rdf.rs` `FLOOR = 413` |
| W3C JSON-LD 1.1 fromRdf | 51 pass | of 53 | `floors/from_rdf.rs` `FLOOR = 51` |
| W3C JSON-LD 1.1 expand | 276 pass | of 385 | `floors/expand.rs` `FLOOR = 276` |
| W3C JSON-LD 1.1 compact | 186 pass | of 246 | `floors/compact.rs` `FLOOR = 186` |
| W3C JSON-LD 1.1 flatten | 53 pass | of 58 | `floors/flatten.rs` `FLOOR = 53` |
| W3C JSON-LD 1.1 frame | 61 pass | of 92 (json-ld-framing) | `floors/frame.rs` `FLOOR = 61` |
| OGC GeoSPARQL topology (hand-curated assertions) | 197 | sparq-authored battery, NOT the OGC suite | `ogc_compliance_ratchet.rs` `OGC_RATCHET_FLOOR = 197` |
| Solid WAC / ACP decision parity | 12 + 12 scenarios, differential oracles at 0 divergences | library-level, NOT wire-level CTH | `sparq-solid` floor consts |
| SolidLab ODRL Test Suite | 67 pass | of 68 (1 documented not-implemented) | `odrl_test_suite.rs` `ODRL_SUITE_FLOOR = 67` |
| W3C SPARQL 1.1 sparql11/service evaluation | 6 pass | 1 documented skip (variable `SERVICE ?ep`) | `service_eval_suite.rs` `SERVICE_EVAL_FLOOR = 6` |
| W3C SPARQL 1.1 Protocol (HTTP) | 21 pass | sparq-authored raw-HTTP battery over the Protocol contract | `http_protocol_suite.rs` `HTTP_PROTOCOL_FLOOR = 21` |
| SPARQL 1.1 Service Description + Graph Store Protocol | 39 pass | sparq-authored battery | `sd_gsp_suite.rs` `SD_GSP_FLOOR = 39` |
| W3C SPARQL 1.1 D-entailment | 1 pass | the single D-only sparql11/entailment case | `d_entail_suite.rs` |
| W3C RIF WG Core test suite | 3 pass | pinned `Core_v1.22` archive; most cases in the importer skip taxonomy | `rif_wg_core_suite.rs` `RIF_WG_CORE_FLOOR = 3` |

The registry's `family = "sparq extension"` rows (BM25 oracle, RSP/SRBench,
RIF-Core expressivity, OWL 2 QL/EL/DL arms, D value-space matrix) are **not**
standards conformance and are excluded here by construction — comparing them
against peers' W3C EARL would be a category error.

## 3. Cross-engine tables (peer cells: published sources only)

PENDING research-agent citations.

## 4. Verdicts

PENDING.

## 5. Follow-ups

PENDING.
