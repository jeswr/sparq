"use client";

// [OPUS-4.8] sq-ixc3.9 — the operational engine context: the ONE live wasm store the whole
// workbench shares, plus warm status, the measured-latency query path, and the dataset
// summary the left-rail datasets tree renders.
//
// HONESTY: no performance number is baked in. The bottom status bar shows the latency of the
// query the user JUST ran, measured with `performance.now()` and labelled as such — never a
// benchmark claim. On the desktop Tauri target the design's end state is the DIRECT native
// engine link (gui/src-tauri/src/engine.rs); this foundation shell runs the same in-tab WASM
// engine in both targets (the honest, working-today path) and the IPC swap is a later phase.

import * as React from "react";
import {
  loadSparq,
  prewarmSparq,
  formatSparqlJson,
  isAskResult,
  askValue,
  streamQueryRows,
  COMMON_PREFIXES,
  type SparqlBinding,
  type SparqlResults,
  type SparqlTerm,
  type WasmStore,
  type WasmStoreCtor,
  type WorkspaceInferenceMode,
} from "@sparq/client";

import { basePath } from "@/lib/base-path";
import { SAMPLE_TURTLE, SAMPLE_FORMAT } from "@/data/sample-graph";
import { type ExportFormat } from "@/lib/rdf-format";
import {
  hasNativeLoader,
  nativeDiskUsage,
  nativeLoadPath,
  nativeLoadText,
  type LoadedDocument,
} from "@/lib/tauri-ipc";
// [OPUS-4.8] sq-tp1m (#757) — the tier-b W-reason wasm bundle loader: the REAL forward-chaining
// RDFS / OWL 2 RL reasoner (crates/sparq-reason via sparq-reason-wasm), lazy-loaded so the lean
// query engine pays nothing until a workspace turns inference on.
import { loadReasoner, modeToProfile } from "@/lib/reason-wasm";

/**
 * Internal sentinel thrown out of the streaming loop when the caller's {@link AbortSignal} fires,
 * so a Stop is distinguishable from a real engine error in the `catch` below. Named so the
 * instanceof check is robust to minification.
 */
class AbortError extends Error {
  constructor() {
    super("aborted");
    this.name = "AbortError";
  }
}

/** The engine warm lifecycle. `error` carries a load/parse failure message. */
export type EngineStatus =
  | { kind: "cold" }
  | { kind: "warming" }
  | { kind: "ready" }
  | { kind: "error"; message: string };

/** A per-named-graph row for the datasets tree (default graph + named graphs). */
export interface GraphSummary {
  /** The graph IRI, or null for the default graph. */
  graph: string | null;
  /** Triple/quad count in this graph. */
  count: number;
}

/**
 * The SELECT result the workbench renders. A streamed SELECT keeps at most {@link RunOptions.rowCap}
 * rows in JS (so a large result cannot blow the tab's memory); `truncated` records whether the
 * full result exceeded that cap. `rawJson` is a SPARQL-1.1-JSON document over the KEPT rows.
 */
export interface SelectOutcome {
  kind: "select";
  /** The SPARQL-JSON results doc over the kept rows (for the Table / Raw JSON views + export). */
  results: SparqlResults;
  rawJson: string;
  /** Rows kept in JS (≤ rowCap). */
  rowCount: number;
  /** Total rows the engine produced (may exceed `rowCount` when streamed + capped). */
  totalRows: number;
  /** True when the engine produced more rows than were kept in JS. */
  truncated: boolean;
}

/** What a run produced — discriminated by SPARQL form so the results panel can branch. */
export type QueryOutcome =
  | SelectOutcome
  | { kind: "ask"; value: boolean; rawJson: string }
  | { kind: "graph"; ntriples: string; tripleCount: number }
  | { kind: "update"; sizeAfter: number }
  | { kind: "explain"; mode: "explain" | "analyze"; plan: string }
  | { kind: "cancelled" }
  | { kind: "error"; message: string };

/** A completed run + its MEASURED latency (performance.now delta, ms). */
export interface RunResult {
  outcome: QueryOutcome;
  /** Wall-clock latency of THIS run, measured with performance.now() (ms). Labelled, not a benchmark. */
  latencyMs: number;
}

/** How to run a query — plain execution, EXPLAIN (plan only), or EXPLAIN ANALYZE (plan + run). */
export type RunMode = "run" | "explain" | "analyze";

/**
 * [OPUS-4.8] sq-tp1m (#757) — counts describing what the active inference regime ADDED over the
 * live store. `baseTriples` is the distinct asserted triples the reasoner saw (named graphs are
 * folded into the default graph by the forward-chainer); `closureTriples` the size of the
 * materialised closure (base + entailed); `entailed` the delta reasoning produced. Real measured
 * counts from the engine's reasoner + store, never fabricated.
 */
export interface ReasoningInfo {
  mode: Exclude<WorkspaceInferenceMode, "off">;
  baseTriples: number;
  closureTriples: number;
  entailed: number;
}

/**
 * [OPUS-4.8] sq-tp1m — the reasoning lifecycle for the active workspace's inference mode:
 *   * `off`     — no inference regime (queries run over the asserted store);
 *   * `loading` — the W-reason bundle is being fetched / the closure materialised;
 *   * `ready`   — the closure is materialised; queries run against it ({@link ReasoningInfo});
 *   * `error`   — the reasoning bundle failed to load/run (e.g. it was not synced into this
 *                 build) — surfaced honestly; queries then fail with a clear message until the
 *                 user turns inference off or the bundle is rebuilt.
 */
export type InferenceStatus =
  | { kind: "off" }
  | { kind: "loading" }
  | { kind: "ready"; info: ReasoningInfo }
  | { kind: "error"; message: string };

/** Optional per-run controls. */
export interface RunOptions {
  /** plain run / EXPLAIN / EXPLAIN ANALYZE. Defaults to `"run"`. */
  mode?: RunMode;
  /**
   * Max SELECT rows to keep in JS when streaming (the cap that bounds peak memory). Rows beyond
   * this are counted but dropped; the outcome is marked `truncated`. Defaults to {@link DEFAULT_ROW_CAP}.
   */
  rowCap?: number;
  /** A cooperative cancel signal — checked between streamed batches (the Stop button). */
  signal?: AbortSignal;
}

/**
 * The default cap on SELECT rows kept in JS for the table/JSON views (streaming bounds peak
 * memory at one batch + this many displayed rows; exports re-stream the WHOLE result). This is a
 * UI display bound, not a result bound — it is labelled in the results panel, not a benchmark.
 */
export const DEFAULT_ROW_CAP = 5_000;

/** The batch size {@link streamQueryRows} pulls per cursor step (one batch held at a time). */
const STREAM_BATCH_SIZE = 1_000;

/** Where an import came from — drives the source kind recorded in the workspace metadata. */
export type ImportKind = "file" | "url" | "paste";

/** Whether an import REPLACES the live store or ADDS (merges) into it. */
export type ImportMode = "replace" | "add";

/** A request to bring RDF into the live store via the Import drawer. */
export interface ImportRequest {
  kind: ImportKind;
  /** REPLACE the store, or merge into it. */
  mode: ImportMode;
  /** Preserve named graphs (route quad-bearing formats through the dataset loader). */
  preserveGraphs: boolean;
  /** A human label for the source (filename / URL tail / "pasted document"). */
  label: string;
  /** The RDF serialisation to parse the document as (e.g. `turtle` / `nquads` / `hdt`). */
  format: string;
  /**
   * For `kind: "file"` — the disk path (the native loader reads + decodes it, incl. compressed
   * + native-only HDT). Mutually exclusive with `text`.
   */
  path?: string;
  /**
   * For `kind: "paste"` / `"url"` — the document body. Mutually exclusive with `path`. (The URL
   * fetch happens in the drawer; the fetched body is passed here.)
   */
  text?: string;
  /** For `kind: "url"` — the source URL, recorded so the workspace source is re-fetchable. */
  url?: string;
}

/** The outcome of a successful import — what the drawer reports + records as workspace metadata. */
export interface ImportResult {
  /** Quads ADDED by this import (the parsed document's whole-dataset count). */
  added: number;
  /** Total quads in the live store after the import. */
  storeSize: number;
  /** Whether the native (Tauri) loader handled it, or the in-tab WASM fallback did. */
  loadedNatively: boolean;
  /** The format the document was actually parsed as. */
  format: string;
  /** UTF-8 byte length of the imported document body (best-effort; for an "≈N bytes" note). */
  bytes: number;
}

/**
 * [OPUS-4.8] sq-vw3ax (#820) — the live INGEST signal the status bar's unintrusive ingest meter
 * reads. While an import is decoding/merging, `active` is true and `label` names the source; the
 * elapsed time is MEASURED by the status bar from `startedAt` (performance.now). The native + WASM
 * loaders are SYNCHRONOUS (no byte-level progress callback exists), so this is honestly an
 * indeterminate "ingesting <file>…" with a real elapsed — never a fabricated percentage or ETA.
 * `null` when no import is in flight.
 */
export interface IngestState {
  /** A human label for the source being ingested (filename / URL tail / "pasted document"). */
  label: string;
  /** performance.now() at ingest start — the status bar derives the live elapsed from this. */
  startedAt: number;
}

/**
 * [OPUS-4.8] sq-vw3ax (#820) — the live on-device store FOOTPRINT the status bar's disk gauge
 * reads. This is the REAL byte length of the whole-dataset N-Quads snapshot the workspace persists
 * (the `dataSnapshot` save/open cache, sq-atb0) — i.e. the actual size of what is written to disk
 * on the desktop target (or to localStorage on the web). It is a real measured figure, not a
 * fabricated capacity: there is no fixed cap, so the gauge shows the snapshot bytes, recomputed
 * after every load/import/update. A precise filesystem probe of the app-data dir (vs this snapshot
 * estimate) is a follow-up that needs a new Tauri command + capability (beaded).
 */
export interface EngineContextValue {
  status: EngineStatus;
  /** Total quads in the live store (default + all named graphs). */
  storeSize: number;
  /**
   * [OPUS-4.8] sq-vw3ax (#820) — the on-device store footprint ESTIMATE in bytes (the UTF-8 length
   * of the persisted whole-dataset N-Quads snapshot). 0 before the store warms. Honest measured
   * value, but an estimate of the on-disk size — it omits the workspace index JSON + encoding
   * overhead. On the desktop shell {@link diskBytes} reports the OS's exact figure instead.
   */
  storeBytes: number;
  /**
   * [OPUS-4.8] sq-cno90 (#820 follow-up) — the OS-REPORTED on-disk byte total of the
   * `$APPLOCALDATA/workspaces` tree (a recursive native `stat()` sum via the `disk_usage` command),
   * or `null` on the web target / before the first probe / if the probe failed. When present this
   * is the precise on-disk footprint the status bar PREFERS over the {@link storeBytes} estimate;
   * when `null` the gauge falls back to the estimate, labelled as such. Never fabricated.
   */
  diskBytes: number | null;
  /**
   * [OPUS-4.8] sq-cno90 (#820 follow-up) — re-run the OS-reported `disk_usage` probe (e.g. after a
   * workspace SAVE, which an import triggers, so {@link diskBytes} reflects the just-written file).
   * A no-op on the web target. Best-effort: a failure leaves the last value, never a fabrication.
   */
  refreshDiskUsage: () => void;
  /** [OPUS-4.8] sq-vw3ax (#820) — the in-flight import, or null. Drives the status bar ingest meter. */
  ingest: IngestState | null;
  /** Per-graph counts for the datasets tree. */
  graphs: GraphSummary[];
  /** The latency (ms) of the most recent run, or null before any run. */
  lastLatencyMs: number | null;
  /** The row count of the most recent SELECT run, or null. */
  lastRowCount: number | null;
  /**
   * Run a query/update against the live store; resolves with the outcome + measured latency.
   * [OPUS-4.8] sq-ixc3.10/.12 — this is the SINGLE EXPLAIN entry point: pass
   * {@link RunOptions.mode} `"explain"` / `"analyze"` to render the planner's plan (the canonical
   * EXPLAIN path the Cmd-K spine AND the workbench EXPLAIN/ANALYZE buttons both drive — there is
   * no separate `explain()` method). Pass {@link RunOptions.signal} to make the run cancellable
   * (Stop), and {@link RunOptions.rowCap} to bound the kept SELECT rows.
   */
  run: (query: string, opts?: RunOptions) => Promise<RunResult>;
  /**
   * [OPUS-4.8] sq-ixc3.11 — serialise the LIVE store to TriG so an operational tool (e.g. the
   * SHACL validator) can run over the actual imported store rather than a fixture. TriG (not
   * N-Triples — the serialise binding does not emit N-Triples) preserves every named graph as
   * well as the default graph. Returns `null` before the engine is ready or if the loaded
   * bundle lacks the serialise binding. The serialise-rdf binding is in the GUI's wasm bundle.
   */
  serializeStore: () => string | null;
  /**
   * [OPUS-4.8] sq-xvj9 — export the WHOLE live store to a pretty, human-readable RDF document
   * for download (the DatasetViewer's "Export data" action). Unlike {@link serializeStore} (an
   * internal, unabbreviated TriG dump the SHACL tool consumes), this is the USER-facing export:
   * indented + sorted, with IRIs abbreviated against the site's `COMMON_PREFIXES` (sq-l5kr's
   * caller-supplied prefix map), so `ex:`/`foaf:`/`schema:`/… compact exactly like the rest of
   * the workbench. `"turtle"` emits the DEFAULT GRAPH (Turtle has no named-graph syntax);
   * `"trig"` and `"jsonld"` emit the WHOLE dataset (default + every named graph). Returns `null`
   * before the engine is ready or if the loaded bundle lacks the serialise binding.
   */
  exportStore: (format: ExportFormat) => string | null;
  /**
   * [OPUS-4.8] sq-ixc3.13 — bring RDF into the live store via the Import drawer. A `file` import
   * (a disk path) goes through the NATIVE loader (`load_path`: threads, no wasm-tab ceiling,
   * compressed streams, native-only HDT) when running inside the Tauri desktop shell; `paste` /
   * `url` documents go through the native `load_text` there too, so named graphs are preserved by
   * the SAME engine path. On the hosted web target (no native loader) `paste` / `url` parse in
   * the in-tab WASM engine (no disk / compressed-file / HDT path — the drawer says so honestly).
   * `mode: "add"` MERGES into the current store (named-graph-preserving) instead of replacing it.
   * Resolves with the import outcome; rejects with the loader's error message on a parse failure.
   */
  importRdf: (req: ImportRequest) => Promise<ImportResult>;
  /** True when the native loader IPC is available (inside the Tauri desktop shell). */
  nativeLoaderAvailable: boolean;
  /**
   * [OPUS-4.8] sq-tp1m (#757) — the active per-workspace INFERENCE regime (query-time entailment).
   * `"off"` runs plain SPARQL; `"rdfs"` / `"owl-rl"` forward-chain the deductive closure (via the
   * real `sparq-reason` W-reason bundle) so a query matches entailed triples too. This is the
   * ENGINE's view of the mode; PERSISTING the choice per workspace is the workspace context's job
   * (the two are kept in lockstep by the inference-mode bridge).
   */
  inferenceMode: WorkspaceInferenceMode;
  /** [OPUS-4.8] sq-tp1m — the reasoning bundle/closure lifecycle for {@link inferenceMode}. */
  inferenceStatus: InferenceStatus;
  /**
   * [OPUS-4.8] sq-tp1m — set the active inference regime. `"off"` tears down the closure and runs
   * plain SPARQL; `"rdfs"` / `"owl-rl"` lazily load the W-reason bundle and materialise the
   * forward-chaining closure over the live store so subsequent queries see entailed triples. The
   * closure is a QUERY-TIME view — it never mutates the persisted store — and is rebuilt
   * automatically when the store changes (import / update).
   */
  setInferenceMode: (mode: WorkspaceInferenceMode) => void;
  /**
   * [OPUS-4.8] sq-ixc3.13 — the whole-dataset N-QUADS snapshot of the live store (default graph +
   * every named graph), the save/open cache a workspace persists as its `dataSnapshot` (sq-atb0).
   * N-Quads (not the TriG `serializeStore` emits) because that is the workspace snapshot format,
   * and it agrees byte-for-byte with the "add to current" merge path. `null` before warm.
   */
  snapshotStore: () => string | null;
}

const EngineContext = React.createContext<EngineContextValue | null>(null);

/** Heuristic SPARQL form classifier (the WASM Store has separate verbs per form). */
function classifyQuery(q: string): "select" | "ask" | "construct" | "describe" | "update" {
  // Strip comments + leading PREFIX/BASE declarations to find the first significant keyword.
  const body = q
    .replace(/(^|\s)#[^\n]*/g, " ")
    .replace(/\b(PREFIX\s+\S+\s+<[^>]*>|BASE\s+<[^>]*>)/gi, " ")
    .trim();
  const m = body.match(/\b(SELECT|ASK|CONSTRUCT|DESCRIBE|INSERT|DELETE|LOAD|CLEAR|CREATE|DROP|COPY|MOVE|ADD)\b/i);
  const kw = m ? m[1].toUpperCase() : "SELECT";
  if (kw === "ASK") return "ask";
  if (kw === "CONSTRUCT") return "construct";
  if (kw === "DESCRIBE") return "describe";
  if (["INSERT", "DELETE", "LOAD", "CLEAR", "CREATE", "DROP", "COPY", "MOVE", "ADD"].includes(kw))
    return "update";
  return "select";
}

// [OPUS-4.8] sq-ixc3.13 — the all-quads N-Quads (de)serialisation the Import drawer's MERGE
// path uses. The wasm `Store.serialize` binding does NOT emit N-Triples/N-Quads (only
// turtle/trig/jsonld — see `serializeStore` below), and the framework-agnostic `@sparq/client`
// deliberately leaves the dataset-format opinion (which formats carry named graphs) to the
// host. So this small, well-understood serialiser lives here: SELECT every quad over the live
// store and emit one N-Quads line per solution, exactly the shape the site's `storeToNQuads`
// (site/src/lib/repl-dataset.ts) produces. The merge then concatenates this with the incoming
// document's N-Quads and re-loads with `loadDataset(_, "nquads")`, so BOTH stores' named graphs
// survive the round-trip.

/** SELECT every quad (default graph as the unbound `?g`, plus every named graph). */
const ALL_QUADS_QUERY =
  "SELECT ?s ?p ?o ?g WHERE { { ?s ?p ?o } UNION { GRAPH ?g { ?s ?p ?o } } }";

const XSD_STRING = "http://www.w3.org/2001/XMLSchema#string";

/** Escape a literal lexical form for an N-Triples/N-Quads double-quoted string. */
function escapeNTLiteral(value: string): string {
  return value
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "\\r")
    .replace(/\t/g, "\\t");
}

/**
 * Emit a single canonical N-Triples/N-Quads TERM (IRI / blank node / literal) from a SPARQL-JSON
 * term. Unlike `formatTerm` (a DISPLAY helper that abbreviates the datatype to `xsd:…`, which is
 * NOT valid N-Triples), this writes the FULL datatype IRI and escapes the lexical form, so the
 * re-load of the merged N-Quads parses losslessly.
 */
function termToNT(t: SparqlTerm): string {
  if (t.type === "uri") return `<${t.value}>`;
  if (t.type === "bnode") return `_:${t.value}`;
  // Literal.
  const lex = `"${escapeNTLiteral(t.value)}"`;
  if (t["xml:lang"]) return `${lex}@${t["xml:lang"]}`;
  if (t.datatype && t.datatype !== XSD_STRING) return `${lex}^^<${t.datatype}>`;
  return lex;
}

/**
 * Serialise the WHOLE dataset of the live store (default graph + every named graph) to N-Quads,
 * one line per quad. Used as the LEFT side of the "add to current" merge. An empty store yields
 * `""`.
 */
function storeToNQuads(store: WasmStore): string {
  const json = store.query(ALL_QUADS_QUERY);
  const parsed = JSON.parse(json) as SparqlResults;
  const lines: string[] = [];
  for (const b of parsed.results?.bindings ?? []) {
    const s = b["s"];
    const p = b["p"];
    const o = b["o"];
    if (!s || !p || !o) continue;
    const g = b["g"];
    const tail = g ? ` ${termToNT(g)}` : "";
    lines.push(`${termToNT(s)} ${termToNT(p)} ${termToNT(o)}${tail} .`);
  }
  return lines.join("\n");
}

/** Summarise the live store into per-graph counts via a single grouped query. */
function summariseGraphs(store: WasmStore): { size: number; graphs: GraphSummary[] } {
  const graphs: GraphSummary[] = [];
  let size = 0;
  // Default graph.
  try {
    const def = store.count("SELECT * WHERE { ?s ?p ?o }");
    if (def > 0) {
      graphs.push({ graph: null, count: def });
      size += def;
    }
  } catch {
    /* count over an empty store can be zero; ignore */
  }
  // Named graphs (group by graph).
  try {
    const json = store.query(
      "SELECT ?g (COUNT(*) AS ?c) WHERE { GRAPH ?g { ?s ?p ?o } } GROUP BY ?g ORDER BY ?g",
    );
    const parsed = JSON.parse(json) as SparqlResults;
    for (const b of parsed.results?.bindings ?? []) {
      const g = b["g"]?.value ?? null;
      const c = Number.parseInt(b["c"]?.value ?? "0", 10) || 0;
      if (g) {
        graphs.push({ graph: g, count: c });
        size += c;
      }
    }
  } catch {
    /* a store with no named graphs yields no rows; ignore */
  }
  return { size, graphs };
}

/**
 * [OPUS-4.8] sq-vw3ax (#820) — the REAL on-device footprint of a store: the UTF-8 byte length of
 * its whole-dataset N-Quads snapshot (exactly what the workspace persists to disk / localStorage).
 * Used for the status bar disk gauge — a measured figure, never a fabricated capacity.
 */
function snapshotBytes(store: WasmStore): number {
  try {
    return new Blob([storeToNQuads(store)]).size;
  } catch {
    return 0;
  }
}

/**
 * [OPUS-4.8] sq-tp1m (#757) — measure what the active inference regime added: the distinct
 * asserted triples the reasoner saw (`base`, folded to the default graph exactly as the
 * forward-chainer folds named graphs), the closure size (`reasoned`, default-graph N-Triples),
 * and the entailed delta. All three are REAL counts queried from the two live stores — never a
 * fabricated figure. A count over an empty store can throw; treat that as zero.
 */
function computeReasoningInfo(
  base: WasmStore,
  reasoned: WasmStore,
  mode: Exclude<WorkspaceInferenceMode, "off">,
): ReasoningInfo {
  let baseTriples = 0;
  try {
    // Distinct s/p/o across the default graph AND every named graph — the reasoner folds named
    // graphs into the default graph, so this is the base-triple count it actually reasoned over.
    baseTriples = base.count(
      "SELECT DISTINCT ?s ?p ?o WHERE { { ?s ?p ?o } UNION { GRAPH ?g { ?s ?p ?o } } }",
    );
  } catch {
    /* empty store */
  }
  let closureTriples = 0;
  try {
    closureTriples = reasoned.count("SELECT * WHERE { ?s ?p ?o }");
  } catch {
    /* empty closure */
  }
  const entailed = Math.max(0, closureTriples - baseTriples);
  return { mode, baseTriples, closureTriples, entailed };
}

// [OPUS-4.8] sq-xvj9 — the caller-supplied prefix map (sq-l5kr) as the `[prefix, iri]` pairs the
// wasm `serialize` binding takes: the site's `COMMON_PREFIXES`, so an exported document abbreviates
// `ex:`/`foaf:`/`schema:`/… consistently with the rest of the workbench (and byte-parity with the
// site's serialiser opinion). Computed once — the registry is static.
const EXPORT_PREFIXES: [string, string][] = COMMON_PREFIXES.map((b) => [b.prefix, b.iri]);

// The engine `serialize` format string for each UI export format. JSON-LD uses the COMPACTED form
// so the prefix map drives a readable `@context` (the plain `"jsonld"` form is expanded and ignores
// `abbreviate`); Turtle/TriG abbreviate via a sorted `@prefix` header (the `abbreviate=true` flag).
const EXPORT_SERIALIZE_FORMAT: Record<ExportFormat, string> = {
  turtle: "turtle",
  trig: "trig",
  jsonld: "jsonld-compacted",
};

export function EngineProvider({ children }: { children: React.ReactNode }) {
  const [status, setStatus] = React.useState<EngineStatus>({ kind: "cold" });
  const [storeSize, setStoreSize] = React.useState(0);
  // [OPUS-4.8] sq-vw3ax (#820) — the live on-device footprint (snapshot bytes) + in-flight import.
  const [storeBytes, setStoreBytes] = React.useState(0);
  // [OPUS-4.8] sq-cno90 (#820 follow-up) — the OS-reported on-disk byte total of the workspaces
  // tree (desktop only); null on the web target, where the gauge uses the snapshot estimate.
  const [diskBytes, setDiskBytes] = React.useState<number | null>(null);
  const [ingest, setIngest] = React.useState<IngestState | null>(null);
  const [graphs, setGraphs] = React.useState<GraphSummary[]>([]);
  const [lastLatencyMs, setLastLatencyMs] = React.useState<number | null>(null);
  const [lastRowCount, setLastRowCount] = React.useState<number | null>(null);

  const storeRef = React.useRef<WasmStore | null>(null);
  const ctorRef = React.useRef<WasmStoreCtor | null>(null);

  // [OPUS-4.8] sq-tp1m (#757) — the per-workspace inference regime + its materialised closure.
  const [inferenceMode, setInferenceModeState] = React.useState<WorkspaceInferenceMode>("off");
  const [inferenceStatus, setInferenceStatus] = React.useState<InferenceStatus>({ kind: "off" });
  // The reasoned (closure) store read queries run against when a regime is active, plus the cache
  // key (`mode:epoch`) it was built for. `storeEpoch` bumps whenever the BASE store content
  // changes (warm / import / update) so a stale closure is rebuilt automatically.
  const reasonedStoreRef = React.useRef<WasmStore | null>(null);
  const reasonedKeyRef = React.useRef<string | null>(null);
  const storeEpochRef = React.useRef(0);
  const [storeEpoch, setStoreEpoch] = React.useState(0);
  // [OPUS-4.8] sq-tp1m — a generation counter bumped on every applyInferenceMode invocation. An
  // async materialisation that resolves AFTER a newer invocation (the user toggled modes quickly,
  // or the store epoch bumped mid-build) is stale and must NOT publish its status, or the UI could
  // land in the wrong state (e.g. mode Off but status Ready/Error). Only the latest generation writes.
  const inferenceGenRef = React.useRef(0);

  // Warm the engine once on mount: load wasm, seed the sample graph, compute the summary.
  React.useEffect(() => {
    let cancelled = false;
    const opts = { basePath: basePath() };
    setStatus({ kind: "warming" });
    prewarmSparq(opts)
      .then(() => loadSparq(opts))
      .then((Store) => {
        if (cancelled) return;
        ctorRef.current = Store;
        const store = Store.load(SAMPLE_TURTLE, SAMPLE_FORMAT);
        storeRef.current = store;
        // [OPUS-4.8] sq-tp1m — first content ready: bump the epoch so any active regime materialises.
        storeEpochRef.current += 1;
        setStoreEpoch(storeEpochRef.current);
        const { size, graphs: gs } = summariseGraphs(store);
        setStoreSize(size);
        setStoreBytes(snapshotBytes(store));
        setGraphs(gs);
        setStatus({ kind: "ready" });
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        setStatus({
          kind: "error",
          message: err instanceof Error ? err.message : String(err),
        });
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // [OPUS-4.8] sq-cno90 (#820 follow-up) — probe the OS-reported on-disk byte total of the
  // workspaces tree via the native `disk_usage` command. On the desktop shell this is the precise
  // figure the status bar prefers; on the web target (no native FS) it stays `null` and the gauge
  // shows the snapshot estimate instead. Best-effort: a probe failure leaves the last value (or
  // `null`) — the gauge never fabricates a number. The on-disk figure lags a freshly persisted
  // snapshot by one write, so we re-probe after the workspace save that an import/update triggers.
  const refreshDiskUsage = React.useCallback(() => {
    nativeDiskUsage()
      .then((du) => {
        if (du) setDiskBytes(du.bytes);
      })
      .catch(() => {
        /* a probe failure must never break the status bar — keep the last value / estimate */
      });
  }, []);

  // [OPUS-4.8] sq-cno90 — probe the OS-reported on-disk footprint once on mount (desktop only; a
  // no-op that leaves diskBytes null on the web target). A restored workspace already has bytes on
  // disk, so this surfaces the real figure on first paint of the desktop shell.
  React.useEffect(() => {
    refreshDiskUsage();
  }, [refreshDiskUsage]);

  const refreshSummary = React.useCallback(() => {
    const store = storeRef.current;
    if (!store) return;
    const { size, graphs: gs } = summariseGraphs(store);
    setStoreSize(size);
    setStoreBytes(snapshotBytes(store));
    setGraphs(gs);
    refreshDiskUsage();
  }, [refreshDiskUsage]);

  // [OPUS-4.8] sq-tp1m (#757) — build (or reuse) the reasoned CLOSURE store for `mode` over the
  // current base store: serialise the whole dataset to N-Quads, forward-chain the RDFS / OWL 2 RL
  // closure with the real W-reason bundle, and load the closure N-Triples into a fresh in-tab
  // store queries can run against. Cached by `mode:epoch`, so every query while the mode + store
  // are unchanged reuses the same closure (the materialisation cost is paid once per change).
  // Throws if the bundle is unavailable or the closure fails — the caller surfaces that honestly.
  const buildReasonedStore = React.useCallback(
    async (mode: Exclude<WorkspaceInferenceMode, "off">): Promise<WasmStore> => {
      const Store = ctorRef.current;
      const base = storeRef.current;
      if (!Store || !base) {
        throw new Error("The engine is not ready yet — wait for the store to warm.");
      }
      const key = `${mode}:${storeEpochRef.current}`;
      if (reasonedKeyRef.current === key && reasonedStoreRef.current) {
        return reasonedStoreRef.current;
      }
      const reasoner = await loadReasoner();
      // The whole-dataset N-Quads snapshot; the reasoner folds named graphs into the default graph.
      const snapshot = storeToNQuads(base);
      const closure = reasoner.materialize(snapshot, "nquads", modeToProfile(mode));
      const reasoned = Store.load(closure, "ntriples");
      reasonedStoreRef.current = reasoned;
      reasonedKeyRef.current = key;
      return reasoned;
    },
    [],
  );

  // [OPUS-4.8] sq-tp1m — reconcile the reasoning STATUS with the active mode + current store:
  // tear the closure down for "off", else (re)materialise it and publish the entailed-triple
  // counts. Idempotent; the single place reasoning is (re)applied (see the effect below).
  const applyInferenceMode = React.useCallback(
    async (mode: WorkspaceInferenceMode): Promise<void> => {
      // Claim this generation up-front; any invocation started later supersedes us.
      const gen = ++inferenceGenRef.current;
      if (mode === "off") {
        reasonedStoreRef.current = null;
        reasonedKeyRef.current = null;
        setInferenceStatus({ kind: "off" });
        return;
      }
      const cachedKey = `${mode}:${storeEpochRef.current}`;
      const alreadyBuilt =
        reasonedKeyRef.current === cachedKey && reasonedStoreRef.current !== null;
      if (!alreadyBuilt) setInferenceStatus({ kind: "loading" });
      try {
        const reasoned = await buildReasonedStore(mode);
        // A newer invocation ran while we awaited — drop this stale result so we don't clobber it.
        if (gen !== inferenceGenRef.current) return;
        const base = storeRef.current;
        const info: ReasoningInfo = base
          ? computeReasoningInfo(base, reasoned, mode)
          : { mode, baseTriples: 0, closureTriples: 0, entailed: 0 };
        setInferenceStatus({ kind: "ready", info });
      } catch (err) {
        // Superseded invocations must not publish a stale error over the newer state either.
        if (gen !== inferenceGenRef.current) return;
        reasonedStoreRef.current = null;
        reasonedKeyRef.current = null;
        setInferenceStatus({
          kind: "error",
          message: err instanceof Error ? err.message : String(err),
        });
      }
    },
    [buildReasonedStore],
  );

  // [OPUS-4.8] sq-tp1m — the PUBLIC mode setter is a plain state write; the effect below does the
  // (async) reasoning work. Keeping the two apart means import/update need only bump the store
  // epoch to trigger a closure rebuild, and the workspace-restore bridge just calls this setter.
  const setInferenceMode = React.useCallback((mode: WorkspaceInferenceMode) => {
    setInferenceModeState(mode);
  }, []);

  // [OPUS-4.8] sq-tp1m — whenever the mode OR the base store content changes, reconcile the
  // closure + status. The SINGLE place reasoning is applied.
  React.useEffect(() => {
    void applyInferenceMode(inferenceMode);
  }, [inferenceMode, storeEpoch, applyInferenceMode]);

  // [OPUS-4.8] sq-ixc3.13 — the Import drawer's ingest. A `file` import (a disk path) goes
  // through the NATIVE loader IPC (`load_path`: compressed + native-only HDT, no wasm-tab
  // ceiling) inside the Tauri shell; `paste`/`url` documents go through native `load_text` there
  // too. Either way the loader hands back the document as N-QUADS (named-graph-preserving), which
  // we MERGE (concatenate with the live store's N-Quads) or REPLACE, then re-load with
  // `loadDataset(_, "nquads")` so every named graph of both sides survives. On the hosted web
  // target (no native loader), `paste`/`url` parse directly in the in-tab WASM engine; a `file`
  // import is rejected there with a clear message (browsers cannot read an arbitrary disk path).
  // The actual ingest body (separated so `importRdf` can bracket it with the ingest signal).
  const runImport = React.useCallback(
    async (req: ImportRequest, Store: WasmStoreCtor): Promise<ImportResult> => {
      // 1. Decode the incoming document to N-Quads (native loader when available, else in-tab).
      let incomingNQuads: string;
      let added: number;
      let format = req.format;
      let loadedNatively = false;
      let bytes = req.text ? new Blob([req.text]).size : 0;

      const native = hasNativeLoader();
      if (req.kind === "file") {
        if (!native || !req.path) {
          throw new Error(
            "Importing a file from disk needs the desktop app (the native loader). On the web " +
              "target, paste the document or load it by URL instead.",
          );
        }
        const doc = await nativeLoadPath(req.path, req.format, req.preserveGraphs);
        if (!doc) throw new Error("The native loader is unavailable.");
        incomingNQuads = doc.nquads;
        added = doc.count;
        format = doc.format;
        loadedNatively = true;
        bytes = new Blob([incomingNQuads]).size;
      } else {
        // paste / url — a document body in `req.text`.
        const text = req.text ?? "";
        const doc: LoadedDocument | null = native
          ? await nativeLoadText(text, req.format, req.preserveGraphs)
          : null;
        if (doc) {
          incomingNQuads = doc.nquads;
          added = doc.count;
          format = doc.format;
          loadedNatively = true;
        } else {
          // In-tab WASM parse: build a store from the incoming doc, then serialise to N-Quads.
          const incomingStore = req.preserveGraphs
            ? Store.loadDataset(text, req.format)
            : Store.load(text, req.format);
          incomingNQuads = storeToNQuads(incomingStore);
          added = incomingNQuads === "" ? 0 : incomingNQuads.split("\n").length;
        }
      }

      // 2. Merge (add) or replace, then re-load the whole dataset as N-Quads.
      let combined = incomingNQuads;
      if (req.mode === "add" && storeRef.current) {
        const current = storeToNQuads(storeRef.current);
        combined = current === "" ? incomingNQuads : `${current}\n${incomingNQuads}`;
      }
      const merged = Store.loadDataset(combined, "nquads");
      storeRef.current = merged;
      // [OPUS-4.8] sq-tp1m — the base store changed: bump the epoch so an active regime rebuilds
      // its closure over the newly-imported data.
      storeEpochRef.current += 1;
      setStoreEpoch(storeEpochRef.current);

      // 3. Refresh the datasets tree + store size + on-device footprint from the new store.
      const { size, graphs: gs } = summariseGraphs(merged);
      setStoreSize(size);
      setStoreBytes(snapshotBytes(merged));
      setGraphs(gs);

      return { added, storeSize: size, loadedNatively, format, bytes };
    },
    [],
  );

  // [OPUS-4.8] sq-ixc3.13 — the Import drawer's ingest. A `file` import (a disk path) goes
  // through the NATIVE loader IPC (`load_path`: compressed + native-only HDT, no wasm-tab
  // ceiling) inside the Tauri shell; `paste`/`url` documents go through native `load_text` there
  // too. Either way the loader hands back the document as N-QUADS (named-graph-preserving), which
  // we MERGE (concatenate with the live store's N-Quads) or REPLACE, then re-load with
  // `loadDataset(_, "nquads")` so every named graph of both sides survives. On the hosted web
  // target (no native loader), `paste`/`url` parse directly in the in-tab WASM engine; a `file`
  // import is rejected there with a clear message (browsers cannot read an arbitrary disk path).
  const importRdf = React.useCallback(
    async (req: ImportRequest): Promise<ImportResult> => {
      const Store = ctorRef.current;
      if (!Store) {
        throw new Error("The engine is not ready yet — wait for the store to warm.");
      }
      // [OPUS-4.8] sq-vw3ax (#820) — mark the ingest in-flight so the status bar shows the
      // unintrusive ingest meter (the real file label + a live measured elapsed). Cleared in the
      // `finally` whether the import succeeds or fails. The loaders are synchronous, so this is an
      // honest "ingesting <file>…" indicator, not a fabricated progress percentage or ETA.
      setIngest({ label: req.label, startedAt: performance.now() });
      try {
        return await runImport(req, Store);
      } finally {
        setIngest(null);
      }
    },
    [runImport],
  );

  const run = React.useCallback(
    async (query: string, opts: RunOptions = {}): Promise<RunResult> => {
      const store = storeRef.current;
      if (!store) {
        const outcome: QueryOutcome = {
          kind: "error",
          message: "The engine is not ready yet — wait for the store to warm.",
        };
        return { outcome, latencyMs: 0 };
      }
      const mode = opts.mode ?? "run";
      const rowCap = opts.rowCap ?? DEFAULT_ROW_CAP;
      const signal = opts.signal;
      const form = classifyQuery(query);
      // [OPUS-4.8] sq-tp1m (#757) — pick the target store. When an inference regime is active,
      // READ queries run against the materialised forward-chaining CLOSURE so entailed triples
      // match; UPDATEs always mutate the ASSERTED base store (the closure is a derived, read-only
      // view — a successful UPDATE bumps the store epoch, invalidating the cached closure). A
      // missing / broken reasoner bundle is surfaced honestly, never silently queried un-reasoned.
      // The closure build is OUTSIDE the latency window below so `latencyMs` measures the query,
      // not the one-time materialisation (which is cached across queries).
      let target = store;
      if (inferenceMode !== "off" && form !== "update") {
        try {
          target = await buildReasonedStore(inferenceMode);
        } catch (err) {
          const message = err instanceof Error ? err.message : String(err);
          return {
            outcome: {
              kind: "error",
              message:
                `Inference is set to ${inferenceMode.toUpperCase()}, but the reasoning bundle ` +
                `could not run: ${message} Turn inference Off to query the asserted data, or ` +
                `rebuild the reasoner bundle (js: npm run build:reason-wasm).`,
            },
            latencyMs: 0,
          };
        }
      }
      const t0 = performance.now();
      let outcome: QueryOutcome;
      try {
        if (mode === "explain" || mode === "analyze") {
          // EXPLAIN renders the planner's chosen plan; EXPLAIN ANALYZE also EXECUTES it and
          // traces the per-operator work (the wasm `explain` / `explainAnalyze` bindings, which
          // mirror `sparq_engine::explain[_analyze]` and the server's `explain=plan|analyze`).
          const plan = mode === "analyze" ? target.explainAnalyze(query) : target.explain(query);
          outcome = { kind: "explain", mode, plan };
        } else if (form === "ask") {
          const json = target.query(query);
          const parsed = JSON.parse(json) as SparqlResults;
          outcome = isAskResult(parsed)
            ? { kind: "ask", value: askValue(parsed) ?? false, rawJson: formatSparqlJson(parsed) }
            : { kind: "error", message: "ASK query did not return a boolean result." };
        } else if (form === "construct" || form === "describe") {
          const ntriples = target.queryQuads(query);
          const tripleCount = ntriples
            .split("\n")
            .filter((l) => l.trim().length > 0).length;
          outcome = { kind: "graph", ntriples, tripleCount };
        } else if (form === "update") {
          store.updateInPlace(query);
          // [OPUS-4.8] sq-tp1m — the asserted store changed: bump the epoch so an active regime
          // rebuilds its closure over the post-update data.
          storeEpochRef.current += 1;
          setStoreEpoch(storeEpochRef.current);
          // `size` reports the default graph only; recompute the full per-graph total below.
          const { size } = summariseGraphs(store);
          outcome = { kind: "update", sizeAfter: size };
        } else {
          // SELECT — STREAM the rows one batch at a time through the wasm cursor so a large
          // result never materialises whole in JS. We keep at most `rowCap` rows for the views;
          // rows beyond the cap are counted but dropped (the outcome records `truncated`). The
          // cooperative `signal` is checked between batches so Stop actually halts the pull.
          const kept: SparqlBinding[] = [];
          let total = 0;
          let cancelled = false;
          const meta = streamQueryRows(target, query, STREAM_BATCH_SIZE, (batch) => {
            if (signal?.aborted) {
              cancelled = true;
              // Throw to break streamQueryRows' loop; the cursor is freed in its `finally`.
              throw new AbortError();
            }
            total += batch.rows.length;
            for (const row of batch.rows) {
              if (kept.length < rowCap) kept.push(row);
            }
          });
          if (cancelled) {
            outcome = { kind: "cancelled" };
          } else {
            const parsed: SparqlResults = {
              head: { vars: meta.vars },
              results: { bindings: kept },
            };
            // `meta.rowCount` is the cursor's own total; prefer it (it covers an empty result's
            // single empty batch correctly), falling back to the counted total.
            const totalRows = meta.rowCount || total;
            outcome = {
              kind: "select",
              results: parsed,
              rawJson: formatSparqlJson(parsed),
              rowCount: kept.length,
              totalRows,
              truncated: totalRows > kept.length,
            };
          }
        }
      } catch (err) {
        outcome =
          err instanceof AbortError || signal?.aborted
            ? { kind: "cancelled" }
            : {
                kind: "error",
                message: err instanceof Error ? err.message : String(err),
              };
      }
      const latencyMs = performance.now() - t0;
      setLastLatencyMs(latencyMs);
      setLastRowCount(
        outcome.kind === "select"
          ? outcome.totalRows
          : outcome.kind === "graph"
            ? outcome.tripleCount
            : null,
      );
      // An UPDATE mutated the store; refresh the datasets tree.
      if (outcome.kind === "update") refreshSummary();
      return { outcome, latencyMs };
    },
    [refreshSummary, inferenceMode, buildReasonedStore],
  );

  // [OPUS-4.8] sq-ixc3.10/.12 — EXPLAIN / EXPLAIN ANALYZE is NOT a separate context method: it is
  // run(query, { mode: "explain" | "analyze" }) above, which surfaces an { kind: "explain" }
  // outcome through the SAME RunResult + measured-latency pipeline every other run uses. The Cmd-K
  // "Run EXPLAIN" verb and the workbench EXPLAIN/ANALYZE buttons both drive that single path, so
  // there is one EXPLAIN contract (this consolidates the standalone explain() #1018 had shipped).

  // [OPUS-4.8] sq-ixc3.11 — TriG dump of the live store, the input an operational tool (SHACL
  // validate-the-active-store) consumes. TriG (the serialise binding does NOT accept
  // "ntriples" — only turtle/trig/jsonld) preserves the default graph AND every named graph,
  // and is unabbreviated (`abbreviate=false`) so no caller-supplied prefix map can disagree.
  // `null` until the store warms or if a lean bundle lacks the binding.
  const serializeStore = React.useCallback((): string | null => {
    const store = storeRef.current;
    if (!store) return null;
    // The GUI bundle is built with `serialize-rdf`, but the runtime-loaded bundle decides
    // whether the binding exists; keep a defensive view so a lean bundle yields a clear empty
    // result rather than a `serialize is not a function` crash.
    const serialize = (store as { serialize?: WasmStore["serialize"] }).serialize;
    if (typeof serialize !== "function") return null;
    return store.serialize("trig", false, null, false, null);
  }, []);

  // [OPUS-4.8] sq-xvj9 — the USER-facing dataset export the DatasetViewer downloads. Serialises the
  // whole live store PRETTY (indented + sorted) and ABBREVIATED against the site's COMMON_PREFIXES
  // (sq-l5kr's caller-supplied prefix map), through the SAME engine writer as `serializeStore`.
  // `turtle` carries the default graph only (Turtle has no named-graph syntax); `trig`/`jsonld`
  // carry the whole dataset. `null` if the store is cold or a lean bundle lacks the binding.
  const exportStore = React.useCallback((format: ExportFormat): string | null => {
    const store = storeRef.current;
    if (!store) return null;
    const serialize = (store as { serialize?: WasmStore["serialize"] }).serialize;
    if (typeof serialize !== "function") return null;
    return store.serialize(EXPORT_SERIALIZE_FORMAT[format], true, null, true, EXPORT_PREFIXES);
  }, []);

  // [OPUS-4.8] sq-ixc3.13 — the N-Quads whole-dataset snapshot the workspace persists (sq-atb0).
  const snapshotStore = React.useCallback((): string | null => {
    const store = storeRef.current;
    if (!store) return null;
    return storeToNQuads(store);
  }, []);

  // [OPUS-4.8] sq-ixc3.13 — whether the native loader IPC is reachable (inside the Tauri shell).
  // Computed once on mount (the runtime never changes mid-session) so the status bar / drawer can
  // label the loader honestly without re-detecting on every render.
  const [nativeLoaderAvailable, setNativeLoaderAvailable] = React.useState(false);
  React.useEffect(() => {
    setNativeLoaderAvailable(hasNativeLoader());
  }, []);

  const value = React.useMemo<EngineContextValue>(
    () => ({
      status,
      storeSize,
      storeBytes,
      diskBytes,
      refreshDiskUsage,
      ingest,
      graphs,
      lastLatencyMs,
      lastRowCount,
      run,
      serializeStore,
      exportStore,
      importRdf,
      nativeLoaderAvailable,
      snapshotStore,
      inferenceMode,
      inferenceStatus,
      setInferenceMode,
    }),
    [
      status,
      storeSize,
      storeBytes,
      diskBytes,
      refreshDiskUsage,
      ingest,
      graphs,
      lastLatencyMs,
      lastRowCount,
      run,
      serializeStore,
      exportStore,
      importRdf,
      nativeLoaderAvailable,
      snapshotStore,
      inferenceMode,
      inferenceStatus,
      setInferenceMode,
    ],
  );

  return <EngineContext.Provider value={value}>{children}</EngineContext.Provider>;
}

export function useEngine(): EngineContextValue {
  const ctx = React.useContext(EngineContext);
  if (!ctx) throw new Error("useEngine must be used within an <EngineProvider>");
  return ctx;
}
