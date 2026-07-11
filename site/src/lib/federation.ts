// [FABLE-5] sq-vw3ax.13 — pure, framework-free data for the federation capability row
// on /capabilities. The federation estate is three OPT-IN native crates the lean wasm
// bundle never carries: sparq-engine-service (the SPARQL 1.1 SERVICE transport behind
// the engine's non-default `service` feature, with a deny-by-default SSRF egress
// allowlist), sparq-fedclient (streaming multi-source SPARQL / brTPF / TPF consumer
// with FedX exclusive groups; native-only — its HTTP stack is compiled out on wasm),
// and sparq-fedplan (the pure, deterministic cost-based source-selection + join
// planner). The static GitHub-Pages site has no backend and the wasm bundles carry
// zero federation code, so this surface is the honest captured-output walkthrough tier.
//
// =====================================================================================
// HONESTY CONTRACT — read before editing (the sq-rnwc lesson, same discipline as
// src/lib/vector.ts). Every planner number below (selection cardinalities, pruning,
// join order, per-join algorithm, per-join cost, total cost) was produced by RUNNING
// THE REAL sparq-fedplan planner via the committed capture harness
//
//   cargo run -p sparq-fedplan --features fedplan --example capture_surface_federation
//
// over the small, FULLY-DECLARED fixture reproduced verbatim in `SOURCES` / `PATTERNS`
// below, and PASTED VERBATIM. The planner is pure and deterministic (no network, no
// randomness — descriptors in, plan out), so the same harness yields byte-identical
// output on any machine; site/test/federation.test.mjs pins the pasted values'
// internal consistency so silent hand-edits fail the unit gate. The three `SCENARIOS`
// differ ONLY in real public `PlanOptions` knobs (request_cost, stream_threshold) —
// exactly the cost-model flips the planner's rustdoc describes. Re-CAPTURE (do not
// hand-edit) if the fixture or the planner changes.
//
// These cardinalities and costs are the planner's ABSTRACT ESTIMATES over declared
// statistics — they are not benchmark results, latency figures, or throughput claims,
// and no such numbers belong on this page.
// =====================================================================================

/** The `vocab` prefixes used across the fixture (display only). */
export const PREFIXES = {
  foaf: "http://xmlns.com/foaf/0.1/",
  dcterms: "http://purl.org/dc/terms/",
  wgs84: "http://www.w3.org/2003/01/geo/wgs84_pos#",
} as const;

/** One served `void:propertyPartition` row of a source descriptor. */
export interface PredPartition {
  /** Prefixed predicate name (display form of the full IRI). */
  predicate: string;
  triples: number;
  distinctSubjects: number;
  distinctObjects: number;
}

/** One mined characteristic set (`scs:` vocab) — subjects sharing a predicate set. */
export interface CharSetRow {
  predicates: string[];
  subjects: number;
  avgMultiplicity: number[];
}

/** One federated endpoint as the planner sees it: a served VoID/scs descriptor. */
export interface FedSource {
  /** Endpoint IRI (the fixture's declared SourceId). */
  endpoint: string;
  /** Short display label. */
  label: string;
  totalTriples: number;
  partitions: PredPartition[];
  charSets: CharSetRow[];
}

/** The declared 3-source fixture — everything the planner knows is on this page. */
export const SOURCES: FedSource[] = [
  {
    endpoint: "https://social.example/sparql",
    label: "social",
    totalTriples: 50_000,
    partitions: [
      {
        predicate: "foaf:name",
        triples: 2_000,
        distinctSubjects: 2_000,
        distinctObjects: 1_970,
      },
      {
        predicate: "foaf:knows",
        triples: 8_000,
        distinctSubjects: 800,
        distinctObjects: 1_800,
      },
    ],
    charSets: [
      {
        predicates: ["foaf:name", "foaf:knows"],
        subjects: 800,
        avgMultiplicity: [1.0, 10.0],
      },
      { predicates: ["foaf:name"], subjects: 1_200, avgMultiplicity: [1.0] },
    ],
  },
  {
    endpoint: "https://pubs.example/sparql",
    label: "pubs",
    totalTriples: 90_000,
    partitions: [
      {
        predicate: "dcterms:creator",
        triples: 12_000,
        distinctSubjects: 9_000,
        distinctObjects: 3_000,
      },
      {
        predicate: "foaf:name",
        triples: 900,
        distinctSubjects: 900,
        distinctObjects: 890,
      },
    ],
    charSets: [],
  },
  {
    endpoint: "https://geo.example/sparql",
    label: "geo",
    totalTriples: 30_000,
    partitions: [
      {
        predicate: "wgs84:lat",
        triples: 15_000,
        distinctSubjects: 15_000,
        distinctObjects: 14_000,
      },
      {
        predicate: "wgs84:long",
        triples: 15_000,
        distinctSubjects: 15_000,
        distinctObjects: 14_000,
      },
    ],
    charSets: [],
  },
];

/** The 3-pattern BGP the planner plans (prefixed display form of the fixture BGP). */
export const PATTERNS = [
  "?paper dcterms:creator ?author",
  "?author foaf:name ?name",
  "?author foaf:knows ?friend",
] as const;

/** The BGP as one SPARQL block (display only — the planner consumes the parsed BGP). */
export const FED_QUERY = `PREFIX foaf:    <${PREFIXES.foaf}>
PREFIX dcterms: <${PREFIXES.dcterms}>

SELECT ?paper ?name ?friend WHERE {
  ?paper  dcterms:creator ?author .
  ?author foaf:name       ?name .
  ?author foaf:knows      ?friend .
}`;

/** One retained (pattern, source) candidate with its CostFed cardinality estimate. */
export interface Candidate {
  /** Index into SOURCES. */
  source: number;
  estimatedCardinality: number;
}

/** Per-pattern source selection — CAPTURED VERBATIM from `select_sources`. */
export interface PatternSelection {
  /** Index into PATTERNS. */
  pattern: number;
  retained: Candidate[];
  /** Source indices the descriptors PROVE cannot contribute (HiBISCuS prune). */
  pruned: number[];
  /** Union cardinality across retained sources (the planner's leaf input size). */
  total: number;
}

/**
 * Capture: `SELECTION` block of the harness run. Recall-safe — a source is pruned for
 * a pattern only when its complete predicate-capability set (its declared property
 * partitions) provably lacks the pattern's bound predicate.
 */
export const SELECTION: PatternSelection[] = [
  {
    pattern: 0,
    retained: [{ source: 1, estimatedCardinality: 12000 }],
    pruned: [0, 2],
    total: 12000,
  },
  {
    pattern: 1,
    retained: [
      { source: 0, estimatedCardinality: 2000 },
      { source: 1, estimatedCardinality: 900 },
    ],
    pruned: [2],
    total: 2900,
  },
  {
    pattern: 2,
    retained: [{ source: 0, estimatedCardinality: 8000 }],
    pruned: [1, 2],
    total: 8000,
  },
];

export type JoinAlgo = "Bind" | "Hash" | "Streaming";

/** One binary join step of the left-deep plan (innermost first). */
export interface JoinStep {
  /** Pattern index (into PATTERNS) joined on the right of this step. */
  right: number;
  algo: JoinAlgo;
  /** Estimated output rows of this join. */
  cardinality: number;
  /** Estimated cost of this join, in the planner's abstract cost units. */
  cost: number;
}

/** One captured plan: the same fixture under one real `PlanOptions` configuration. */
export interface PlanScenario {
  id: string;
  /** Chip label. */
  label: string;
  /** What the knob means, one sentence. */
  caption: string;
  /** The real PlanOptions fields that differ from default (display form). */
  options: string;
  joinOrder: number[];
  steps: JoinStep[];
  totalCost: number;
  rootCardinality: number;
}

/**
 * Capture: the three `PLAN` blocks of the harness run. Same selection, same greedy
 * join order — only the public cost knobs differ, flipping the per-join algorithm
 * exactly as the planner's cost model documents.
 */
export const SCENARIOS: PlanScenario[] = [
  {
    id: "default",
    label: "Default cost model",
    caption:
      "Round-trips are cheap (request_cost 1.0) — per-row bound requests (SERVICE/VALUES push-down) win both joins.",
    options: "PlanOptions::default()",
    joinOrder: [1, 0, 2],
    steps: [
      { right: 0, algo: "Bind", cardinality: 2900, cost: 5800 },
      { right: 2, algo: "Bind", cardinality: 8000, cost: 10900 },
    ],
    totalCost: 16700,
    rootCardinality: 8000,
  },
  {
    id: "pricier-round-trips",
    label: "Pricier round-trips",
    caption:
      "Doubling the per-request penalty (request_cost 2.0) flips the wide second join to a local hash join — scan both sides once instead of one bound request per row.",
    options: "PlanOptions { request_cost: 2.0, ..default }",
    joinOrder: [1, 0, 2],
    steps: [
      { right: 0, algo: "Bind", cardinality: 2900, cost: 8700 },
      { right: 2, algo: "Hash", cardinality: 8000, cost: 10900 },
    ],
    totalCost: 19600,
    rootCardinality: 8000,
  },
  {
    id: "bounded-memory",
    label: "Bounded memory",
    caption:
      "With a lower streaming threshold (stream_threshold 10 000), the large hash-class join is marked for the non-blocking StreamJoin operator (emit-as-you-go + spill) — same cost, different execution discipline.",
    options: "PlanOptions { request_cost: 2.0, stream_threshold: 10_000.0, ..default }",
    joinOrder: [1, 0, 2],
    steps: [
      { right: 0, algo: "Bind", cardinality: 2900, cost: 8700 },
      { right: 2, algo: "Streaming", cardinality: 8000, cost: 10900 },
    ],
    totalCost: 19600,
    rootCardinality: 8000,
  },
];

/** Look up a scenario by id (used by the walkthrough chips + tests). */
export function scenarioById(id: string): PlanScenario | undefined {
  return SCENARIOS.find((s) => s.id === id);
}

/**
 * Why a source was pruned for a pattern — derived from the SAME declared fixture the
 * planner consumed: the bound-predicate prune fires when the source's complete
 * property-partition set lacks the pattern's predicate.
 */
export function pruneReason(patternIdx: number, sourceIdx: number): string {
  const pred = PATTERNS[patternIdx].split(/\s+/)[1];
  const src = SOURCES[sourceIdx];
  return `${src.label} declares no ${pred} partition — provably no matching triple`;
}

/** The predicate (prefixed) of a PATTERNS row — used by the UI and the tests. */
export function patternPredicate(patternIdx: number): string {
  return PATTERNS[patternIdx].split(/\s+/)[1];
}
