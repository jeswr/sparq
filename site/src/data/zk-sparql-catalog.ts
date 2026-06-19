// [OPUS-4.8] sq-1s2.1.3 — typed accessor over the canonical SPARQL → ZK gate-cost
// catalog (src/data/zk-sparql-catalog.generated.json, a build-time copy of
// bench/zk-compose/sparql_feature_catalog.json synced by scripts/sync-zk-catalog.mjs).
//
// The catalog is the SINGLE SOURCE OF TRUTH for the /benchmarks/zk SPARQL-coverage
// table: which SPARQL-1.1 feature compiles to which circuit member(s), each `covered`
// member's `bb gates -s ultra_honk` circuit_size (JOINED from the regression-gated
// snapshot, never hand-typed), and the coverage status (covered / partial / gap).
// NOTHING here hard-codes a gate count — every number rendered is read from the JSON,
// so the page auto-reflects the canonical catalog.
//
// HONESTY (load-bearing, mirrors the catalog's own discipline):
//   - Gate counts are DETERMINISTIC circuit-SIZE metrics (`bb gates -s ultra_honk`),
//     NOT performance/throughput. The accessor never derives a timing or a ratio.
//   - A `gap` row carries circuit_size: null and is NEVER given a fabricated number.
//   - The blake3-bound numeric-FILTER family carries a PROJECTED `projected_after`
//     reduction target — surfaced verbatim and LABELLED as a projection, never as a
//     measured/achieved result (the audit + bb-gates re-measurement is pending,
//     CR-G8 / sq-qhy4).

import raw from "@/data/zk-sparql-catalog.generated.json";

/** Coverage status of a SPARQL feature against the ZK circuit estate. */
export type CatalogStatus = "covered" | "partial" | "gap";

/** High-gate / coverage flag exactly as the catalog emits it (or null). */
export type CatalogFlag =
  | "HIGH_GATE_blake3_binding"
  | "HIGH_GATE_lattice"
  | "VERIFIER_SIDE_OR_DESUGARED"
  | "NO_ZK_CIRCUIT_YET"
  | null;

/** One catalog query, as captured in the canonical JSON. */
interface RawQuery {
  feature: string;
  sparql: string;
  zk_members: string[];
  /** `bb gates -s ultra_honk` circuit_size; null for partial/gap (never fabricated). */
  circuit_size: number | null;
  status: string;
  flag: string | null;
  circuit_size_per_member?: Record<string, number>;
  /** [min, max] per-member size; typed loosely as the JSON infers it as number[]. */
  circuit_size_range?: number[];
  /** Measured raw-compare floor for the blake3-bound double FILTER (Q06 only). */
  floor_circuit_size?: number;
  floor_member?: string;
  /** PROJECTED post-reduction estimate — NOT a measurement. */
  projected_after?: string;
  reduction_target?: string;
  note: string;
}

interface RawCatalog {
  tool: string;
  bb_version: string;
  nargo_version: string;
  snapshot_source: string;
  high_gate_threshold: number;
  summary: {
    total_queries: number;
    covered: number;
    partial: number;
    gaps: number;
    high_gate_flagged: string[];
  };
  queries: Record<string, RawQuery>;
}

/** A normalised catalog row the table renders. */
export interface CatalogEntry {
  /** Stable id (e.g. "Q03_filter_integer_ge") — used as the React key. */
  id: string;
  feature: string;
  sparql: string;
  members: string[];
  circuitSize: number | null;
  status: CatalogStatus;
  flag: CatalogFlag;
  perMember?: Record<string, number>;
  /** [min, max] per-member circuit size, when the catalog records a range. */
  range?: readonly [number, number];
  floorMember?: string;
  floorCircuitSize?: number;
  /** Raw PROJECTED-after string from the catalog (already self-labelled an ESTIMATE). */
  projectedAfter?: string;
  reductionTarget?: string;
  note: string;
}

const data = raw as RawCatalog;

function normStatus(s: string): CatalogStatus {
  if (s === "covered") return "covered";
  if (s.startsWith("partial")) return "partial";
  return "gap"; // "NO ZK CIRCUIT YET (gap)"
}

function normFlag(f: string | null): CatalogFlag {
  switch (f) {
    case "HIGH_GATE_blake3_binding":
    case "HIGH_GATE_lattice":
    case "VERIFIER_SIDE_OR_DESUGARED":
    case "NO_ZK_CIRCUIT_YET":
      return f;
    default:
      return null;
  }
}

/** Catalog ordering matches the canonical JSON's insertion order (Q01 … Q26). */
export const ZK_SPARQL_CATALOG: readonly CatalogEntry[] = Object.entries(
  data.queries,
).map(([id, q]) => ({
  id,
  feature: q.feature,
  sparql: q.sparql,
  members: q.zk_members,
  circuitSize: q.circuit_size,
  status: normStatus(q.status),
  flag: normFlag(q.flag),
  perMember: q.circuit_size_per_member,
  range:
    q.circuit_size_range && q.circuit_size_range.length === 2
      ? [q.circuit_size_range[0], q.circuit_size_range[1]]
      : undefined,
  floorMember: q.floor_member,
  floorCircuitSize: q.floor_circuit_size,
  projectedAfter: q.projected_after,
  reductionTarget: q.reduction_target,
  note: q.note,
}));

export const ZK_SPARQL_SUMMARY = data.summary;
export const ZK_SPARQL_GATE_TOOL = data.tool;
export const ZK_SPARQL_BB_VERSION = data.bb_version;
export const ZK_SPARQL_NARGO_VERSION = data.nargo_version;
export const ZK_SPARQL_SNAPSHOT_SOURCE = data.snapshot_source;
export const ZK_SPARQL_HIGH_GATE_THRESHOLD = data.high_gate_threshold;

/**
 * The single PROJECTION figure to surface as the value-hook reduction target, derived
 * (not hard-coded) from the blake3-bound numeric-FILTER rows that share the same
 * `projected_after` string. Returns the current measured ceiling (the shared filter gate
 * count), the verbatim projection string (already self-labelled an ESTIMATE in the
 * canonical JSON), and the measured raw-compare floor where one exists (Q06's
 * `floor_circuit_size`). The page MUST present `projected` as projected-pending-audit,
 * never as achieved.
 */
export function blake3ReductionProjection(): {
  measuredCeiling: number;
  /** The projected post-reduction figure, PARSED from the catalog's `projected_after`
   *  string (e.g. "~3200") — never hard-coded in the page. Already formatted with a
   *  thousands separator. null if the catalog string carries no parseable number. */
  projectedGates: string | null;
  /** The verbatim self-labelled ESTIMATE string from the canonical catalog. */
  projectedRaw: string;
  floorMember?: string;
  floorCircuitSize?: number;
} | null {
  const blake3 = ZK_SPARQL_CATALOG.filter(
    (e) => e.flag === "HIGH_GATE_blake3_binding" && e.circuitSize != null,
  );
  if (blake3.length === 0) return null;
  const withProjection = blake3.find((e) => e.projectedAfter);
  if (!withProjection || withProjection.circuitSize == null) return null;
  const withFloor = blake3.find((e) => e.floorCircuitSize != null);
  // Pull the first integer out of the canonical "ESTIMATE ~3200 — …" string so the page
  // shows the projection WITHOUT hard-coding it in JSX (the catalog stays the only source).
  const m = withProjection.projectedAfter!.match(/(\d[\d,]*)/);
  const projectedGates = m ? Number(m[1].replace(/,/g, "")).toLocaleString() : null;
  return {
    measuredCeiling: withProjection.circuitSize,
    projectedGates,
    projectedRaw: withProjection.projectedAfter!,
    floorMember: withFloor?.floorMember,
    floorCircuitSize: withFloor?.floorCircuitSize,
  };
}
