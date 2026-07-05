// [OPUS-4.8] sq-ro6if (PSS #1415/#1478) — the in-site data layer for the SERVED-SURFACE
// SPARQL 1.1 Protocol conformance dashboard. Consumes the committed snapshot
// (src/data/served-conformance.generated.json), which is emitted by the dev-only
// `served-conformance-report` binary re-running the two EXISTING served-surface CI lanes
// (sq-jaj38 http-protocol + sq-1uuxz Service-Description/Graph-Store-Protocol) in-process
// against the sparq-server loopback endpoint. Regenerate the snapshot with:
//
//   cargo run -p sparq-conformance --features http-protocol,federation-descriptors \
//     --bin served-conformance-report -- --out site/src/data/served-conformance.generated.json
//
// HONESTY (load-bearing, per the bead + the empirical-honesty rule):
//   * These are CONFORMANCE COUNTS (pass / divergence / fail assertion tallies), NOT
//     performance numbers — no timings or throughput appear here.
//   * `floor` is each suite's RATCHET FLOOR (may only rise); a pass count equal to the floor
//     is the honest current state, not a target.
//   * DOCUMENTED DIVERGENCES are reported per-assertion and are NEVER summed into `pass`, so
//     a documented gap can never inflate the conformance number.
//   * This page renders a SNAPSHOT of the current main checkout. The AUTHORITATIVE, versioned
//     per-release reports are attached to each GitHub Release (and emitted as a build artifact
//     by the service-federation-conformance CI job); this snapshot may lag a release.
//   * The served surface is the sparq-server `/sparql` HTTP endpoint a consumer (e.g. PSS)
//     talks to; engine-level conformance can diverge from it, which is why it is published
//     separately.

import data from "./served-conformance.generated.json";

export type ConformanceOutcome = "pass" | "divergence" | "fail";

export interface ConformanceTest {
  label: string;
  outcome: ConformanceOutcome;
  detail?: string;
}

export interface ConformanceCounts {
  pass: number;
  fail: number;
  divergence: number;
  total: number;
}

export interface ConformanceSuite {
  id: string;
  label: string;
  family: string;
  feature: string;
  ci_job: string;
  floor: number | null;
  floor_basis: string;
  note: string;
  counts: ConformanceCounts;
  tests: ConformanceTest[];
}

export interface ConformanceProvenance {
  commit: string;
  version: string;
  generated_at: string;
  generated_at_unix: number;
  run_source: string;
  runner: string;
  note: string;
}

export interface ServedConformanceReport {
  schema: string;
  title: string;
  provenance: ConformanceProvenance;
  totals: ConformanceCounts;
  suites: ConformanceSuite[];
}

export const servedConformance = data as ServedConformanceReport;

/** Short (7-char) commit for display; the full SHA stays in the report + provenance. */
export function shortCommit(sha: string): string {
  return /^[0-9a-f]{7,}$/i.test(sha) ? sha.slice(0, 7) : sha;
}
