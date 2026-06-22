// [OPUS-4.8] sq-he72 — GUI Phase 2 item 8: the server HEALTH / CAPABILITIES client.
//
// This is the framework-agnostic companion to the endpoint-mode query client
// (`./endpoint.ts`): where that one runs SPARQL against a connected `sparq-server`, this one
// reads the server's OPERATIONAL surface — its liveness, its Prometheus `/metrics`, and its
// opt-in VoID / SPARQL Service Description — and reshapes those wire documents into the
// readable structures a "server health / capabilities" view renders.
//
// It CONSUMES the existing `sparq-server` HTTP API (`crates/sparq-server/src/http.rs`,
// `metrics.rs`, `descriptors.rs`) verbatim — it changes nothing server-side and claims no
// capability the server does not advertise. It reuses the SAME `EndpointConfig` (URL +
// optional bearer token) the Connect panel owns, so one connection + bearer posture serves
// queries, subscriptions, AND this panel. The bearer token is sent ONLY in the
// `Authorization: Bearer` header (the channel the server's read gate `--auth-token-read`
// validates) — never logged, never echoed.
//
// HONESTY is load-bearing here. The `/metrics` and `/.well-known/void` surfaces are BOTH
// opt-in and OFF by default (the VoID/SD endpoints need the `federation-descriptors` feature
// AND a config flag). When the server has them disabled it answers `404`; this module reports
// that as a first-class `"not-exposed"` outcome so the UI can say so honestly rather than
// inventing data or surfacing a scary error. Likewise a metrics COUNTER/GAUGE value is
// whatever the server reports at scrape time — never relabelled, never a benchmark claim.
//
// No React, no Next.js, no DOM beyond the `fetch` / `URL` web-platform globals. No
// performance claim is made here (this repo's work box is non-canonical); a caller MAY time a
// single fetch with `performance.now()` and label it as a measured per-request latency.

import type { EndpointConfig } from "./endpoint.js";
import { parseEndpointUrl } from "./endpoint.js";
import { parseNTriples, type RdfStatement, type RdfTerm } from "./pretty-turtle.js";

// ---------------------------------------------------------------------------
// Endpoint URL derivation — the sibling operational endpoints off the /sparql URL.
// ---------------------------------------------------------------------------

/**
 * The configured endpoint URL points at the SPARQL query path (e.g.
 * `http://127.0.0.1:3030/sparql`). The operational endpoints — `/health`, `/metrics`,
 * `/.well-known/void` — live at the ROOT of the same origin, NOT under `/sparql`. Derive a
 * sibling URL by replacing the whole path of the configured URL with `path`, preserving the
 * origin (scheme + host + port) and dropping any query string. Returns `null` when the
 * configured URL is not a valid absolute http(s) URL (the same gate `parseEndpointUrl` uses),
 * so the caller can surface an honest "fix the endpoint URL" rather than build a bad request.
 *
 * `path` MUST begin with `/` (an absolute path on the origin). The mapping is deliberately
 * "replace the path", not "append": a server bound at `http://host:3030/sparql` exposes
 * `/metrics` at `http://host:3030/metrics`, never `http://host:3030/sparql/metrics`.
 */
export function deriveServerUrl(endpointUrl: string, path: string): string | null {
  const parsed = parseEndpointUrl(endpointUrl);
  if (!parsed) return null;
  // Rebuild from the origin so the configured path + query never leak into the sibling URL.
  return `${parsed.origin}${path}`;
}

/** The server-root paths this module reads (each off the configured endpoint's origin). */
export const HEALTH_PATH = "/health";
export const METRICS_PATH = "/metrics";
export const VOID_PATH = "/.well-known/void";

// ---------------------------------------------------------------------------
// Prometheus text-exposition parsing (mirrors `crates/sparq-server/src/metrics.rs`).
// ---------------------------------------------------------------------------
//
// The server hand-rolls the Prometheus [text exposition format] — `# HELP <name> <help>`
// and `# TYPE <name> <counter|gauge|histogram|summary|untyped>` comment lines, then one
// `name{label="v",…} <value>` sample line per series. We parse exactly that grammar into a
// list of metric FAMILIES (one per `name`), each carrying its help text, type, and samples.
// This is a focused parser for the well-formed output `metrics.rs` emits, not a general
// Prometheus parser — but it tolerates the variations the spec allows (missing HELP/TYPE,
// `+Inf`/`NaN` values, escaped label values) so it never throws on a conformant document.
//
// [text exposition format]: https://prometheus.io/docs/instrumenting/exposition_formats/

/** A Prometheus metric type, as declared by a `# TYPE` line (defaults to `untyped`). */
export type MetricType =
  | "counter"
  | "gauge"
  | "histogram"
  | "summary"
  | "untyped";

/** One sample (time-series point): the metric name, its label set, and its numeric value. */
export interface MetricSample {
  /** The metric name (for a histogram, e.g. `sparq_query_duration_seconds_bucket`). */
  name: string;
  /** The label set, in declaration order (`{}` → empty). */
  labels: { name: string; value: string }[];
  /** The numeric value. `+Inf` / `-Inf` / `NaN` parse to the JS equivalents. */
  value: number;
}

/** A metric FAMILY: every sample sharing a base `name`, plus its declared help + type. */
export interface MetricFamily {
  /** The base metric name (the `# HELP` / `# TYPE` subject). */
  name: string;
  /** The `# HELP` text, or `null` when the document declared none. */
  help: string | null;
  /** The `# TYPE`, defaulting to `untyped` when the document declared none. */
  type: MetricType;
  /** Every sample of this family, in document order. */
  samples: MetricSample[];
}

/** The parsed `/metrics` document: the metric families, in first-seen order. */
export interface ParsedMetrics {
  families: MetricFamily[];
}

const METRIC_TYPES: ReadonlySet<string> = new Set([
  "counter",
  "gauge",
  "histogram",
  "summary",
  "untyped",
]);

/** Parse a Prometheus value token (`+Inf` / `-Inf` / `Nan` / a float) to a JS number. */
function parseMetricValue(token: string): number {
  const t = token.trim();
  if (t === "+Inf" || t === "Inf") return Number.POSITIVE_INFINITY;
  if (t === "-Inf") return Number.NEGATIVE_INFINITY;
  if (t === "NaN" || t === "Nan") return Number.NaN;
  const n = Number(t);
  return Number.isNaN(n) ? Number.NaN : n;
}

/**
 * Parse the label set inside a `{ … }` block: `name="value",name2="value2"`. Label values
 * are double-quoted and may carry the Prometheus escapes (`\\`, `\"`, `\n`); we unescape
 * those three. A bare `{}` yields `[]`. This is a forgiving scanner — a label it cannot read
 * is skipped rather than throwing, so a slightly-off line degrades to fewer labels, not a
 * crash.
 */
function parseLabels(block: string): { name: string; value: string }[] {
  const labels: { name: string; value: string }[] = [];
  let i = 0;
  const s = block;
  while (i < s.length) {
    // Skip whitespace / commas between labels.
    while (i < s.length && (s[i] === " " || s[i] === "," || s[i] === "\t")) i++;
    if (i >= s.length) break;
    // Read the label name (up to `=`).
    const eq = s.indexOf("=", i);
    if (eq === -1) break;
    const name = s.slice(i, eq).trim();
    i = eq + 1;
    if (s[i] !== '"') break; // value must be a quoted string
    i++; // past opening quote
    let value = "";
    while (i < s.length && s[i] !== '"') {
      if (s[i] === "\\" && i + 1 < s.length) {
        const next = s[i + 1];
        value += next === "n" ? "\n" : next; // \\ and \" fall through to the literal char
        i += 2;
      } else {
        value += s[i];
        i++;
      }
    }
    i++; // past closing quote
    if (name !== "") labels.push({ name, value });
  }
  return labels;
}

/**
 * [OPUS-4.8] sq-he72 — parse a Prometheus text-exposition document into metric families.
 *
 * Walks the document line by line: `# HELP <name> <text>` and `# TYPE <name> <type>` comments
 * attach to the named family (created on first reference); every other non-blank, non-comment
 * line is a sample `name{labels} value [timestamp]`. Samples whose name shares the family's
 * base name (or a histogram/summary suffix — `_bucket` / `_sum` / `_count` — of it) are
 * grouped under that family; a sample for an as-yet-undeclared name creates an `untyped`
 * family. Families are returned in first-seen order so the rendered view is stable.
 *
 * Never throws: a line it cannot classify is ignored. An empty / whitespace-only document
 * yields `{ families: [] }`.
 */
export function parsePrometheusMetrics(text: string): ParsedMetrics {
  const families: MetricFamily[] = [];
  const byName = new Map<string, MetricFamily>();

  const family = (name: string): MetricFamily => {
    let f = byName.get(name);
    if (!f) {
      f = { name, help: null, type: "untyped", samples: [] };
      byName.set(name, f);
      families.push(f);
    }
    return f;
  };

  // Find the family a sample belongs to: an exact name match, else strip a histogram/summary
  // suffix (`_bucket` / `_sum` / `_count`) and match the base. Falls back to a fresh family.
  const familyForSample = (sampleName: string): MetricFamily => {
    if (byName.has(sampleName)) return byName.get(sampleName)!;
    for (const suffix of ["_bucket", "_sum", "_count"]) {
      if (sampleName.endsWith(suffix)) {
        const base = sampleName.slice(0, -suffix.length);
        const f = byName.get(base);
        if (f) return f;
      }
    }
    return family(sampleName);
  };

  for (const rawLine of text.split("\n")) {
    const line = rawLine.trim();
    if (line === "") continue;
    if (line.startsWith("#")) {
      // `# HELP <name> <text>` or `# TYPE <name> <type>` — anything else is a comment.
      const m = /^#\s+(HELP|TYPE)\s+(\S+)\s*(.*)$/.exec(line);
      if (!m) continue;
      const [, kind, name, rest] = m;
      const f = family(name);
      if (kind === "HELP") {
        f.help = rest;
      } else {
        const t = rest.trim().toLowerCase();
        f.type = METRIC_TYPES.has(t) ? (t as MetricType) : "untyped";
      }
      continue;
    }
    // A sample line: `name{labels} value [timestamp]` or `name value [timestamp]`.
    const brace = line.indexOf("{");
    let name: string;
    let labels: { name: string; value: string }[] = [];
    let valuePart: string;
    if (brace !== -1) {
      const close = line.indexOf("}", brace);
      if (close === -1) continue; // malformed — skip
      name = line.slice(0, brace).trim();
      labels = parseLabels(line.slice(brace + 1, close));
      valuePart = line.slice(close + 1).trim();
    } else {
      const sp = line.search(/\s/);
      if (sp === -1) continue; // no value — skip
      name = line.slice(0, sp).trim();
      valuePart = line.slice(sp).trim();
    }
    if (name === "") continue;
    // The value is the first whitespace-separated token (a trailing timestamp is ignored).
    const valueToken = valuePart.split(/\s+/)[0] ?? "";
    const value = parseMetricValue(valueToken);
    familyForSample(name).samples.push({ name, labels, value });
  }

  return { families };
}

// ---------------------------------------------------------------------------
// VoID + SPARQL Service Description → a readable capabilities view.
// ---------------------------------------------------------------------------
//
// The server serialises BOTH descriptors as RDF. We request `application/n-triples` (the
// server serialises it; `descriptors.rs` `serialise`) and reshape the triples into the
// structured facts a capabilities panel renders. We do NOT re-implement an RDF store: a flat
// linear pass keyed on the well-known VoID / SD vocabulary IRIs is enough for the small,
// shaped documents the server emits (`crates/sparq-introspect` `to_void`,
// `crates/sparq-server/src/descriptors.rs` `sd_ntriples`).

const RDF_TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const VOID_NS = "http://rdfs.org/ns/void#";
const SD_NS = "http://www.w3.org/ns/sparql-service-description#";

/** The VoID dataset-level counts (`crates/sparq-introspect` `to_void`). */
export interface VoidSummary {
  /** The `void:Dataset` IRI the server names (`{base}/.well-known/void#dataset`), or `null`. */
  datasetIri: string | null;
  /** `void:triples` — total triples (exact). `null` when absent. */
  triples: number | null;
  /** `void:entities` — distinct subjects carrying an `rdf:type` (exact). `null` when absent. */
  entities: number | null;
  /** `void:distinctSubjects` — distinct subjects (exact). `null` when absent. */
  distinctSubjects: number | null;
  /** `void:classes` — distinct classes (`rdf:type` objects). `null` when absent. */
  classes: number | null;
  /** `void:properties` — distinct predicates. `null` when absent. */
  properties: number | null;
}

/** One named graph the Service Description advertises (`sd:namedGraph` → `sd:name`). */
export interface NamedGraphSummary {
  /** The graph's `sd:name` IRI (usable in `FROM NAMED` / `GRAPH <name>`). */
  name: string;
  /** The graph's `void:triples` count, or `null` when not advertised. */
  triples: number | null;
}

/** The SPARQL Service Description facts (`crates/sparq-server/src/descriptors.rs`). */
export interface ServiceDescriptionSummary {
  /** The `sd:Service` IRI, or `null` when no `sd:Service` was found. */
  serviceIri: string | null;
  /** The `sd:endpoint` IRI. `null` when absent. */
  endpoint: string | null;
  /** `sd:supportedLanguage` IRIs (e.g. `sd:SPARQL11Query`, `sd:SPARQL11Update`). */
  supportedLanguages: string[];
  /** `sd:feature` IRIs (e.g. `sd:BasicFederatedQuery` when the server feature is on). */
  features: string[];
  /** `sd:resultFormat` IRIs the server can RETURN. */
  resultFormats: string[];
  /** `sd:inputFormat` IRIs the server can PARSE. */
  inputFormats: string[];
  /** `sd:extensionFunction` IRIs the engine has registered (e.g. the `geof:` set). */
  extensionFunctions: string[];
  /** The named graphs the SD enumerates (`sd:namedGraph`), sorted as the server emits them. */
  namedGraphs: NamedGraphSummary[];
}

/** The combined capabilities view extracted from the VoID + SD documents. */
export interface CapabilitiesSummary {
  void: VoidSummary;
  service: ServiceDescriptionSummary;
}

/** An IRI term's value, or `null` for a non-IRI (blank node / literal) term. */
function iriValue(t: RdfTerm): string | null {
  return t.kind === "iri" ? t.value : null;
}

/** A literal's integer value, or `null` for a non-integer / non-literal term. */
function intValue(t: RdfTerm): number | null {
  if (t.kind !== "literal") return null;
  const n = Number(t.value);
  return Number.isInteger(n) ? n : null;
}

/** A stable key for a term, used to follow blank-node / IRI links in the SD graph. */
function termKey(t: RdfTerm): string {
  return t.kind === "bnode" ? `_:${t.label}` : t.nt;
}

/**
 * [OPUS-4.8] sq-he72 — extract the readable capabilities facts from the VoID document the
 * server serves at `/.well-known/void` (parsed N-Triples statements). Reads the dataset-level
 * `void:triples` / `void:entities` / `void:distinctSubjects` / `void:classes` /
 * `void:properties` off the `void:Dataset` subject. The per-class / per-predicate partitions
 * are intentionally NOT surfaced here (the panel shows the dataset summary, not every
 * partition); a caller wanting them can read the raw document.
 */
export function extractVoidSummary(statements: RdfStatement[]): VoidSummary {
  // Find the dataset subject (the one typed `void:Dataset`). The introspect emits exactly one.
  let datasetKey: string | null = null;
  let datasetIri: string | null = null;
  for (const st of statements) {
    if (
      iriValue(st.p) === RDF_TYPE &&
      iriValue(st.o) === `${VOID_NS}Dataset` &&
      st.s.kind === "iri"
    ) {
      datasetKey = termKey(st.s);
      datasetIri = st.s.value;
      break;
    }
  }
  const summary: VoidSummary = {
    datasetIri,
    triples: null,
    entities: null,
    distinctSubjects: null,
    classes: null,
    properties: null,
  };
  if (datasetKey === null) return summary;
  for (const st of statements) {
    if (termKey(st.s) !== datasetKey) continue;
    const p = iriValue(st.p);
    if (p === `${VOID_NS}triples`) summary.triples = intValue(st.o);
    else if (p === `${VOID_NS}entities`) summary.entities = intValue(st.o);
    else if (p === `${VOID_NS}distinctSubjects`) summary.distinctSubjects = intValue(st.o);
    else if (p === `${VOID_NS}classes`) summary.classes = intValue(st.o);
    else if (p === `${VOID_NS}properties`) summary.properties = intValue(st.o);
  }
  return summary;
}

/**
 * [OPUS-4.8] sq-he72 — extract the readable capabilities facts from the SPARQL Service
 * Description the server serves for a `GET /sparql` with no `query` (parsed N-Triples
 * statements). Reads the `sd:Service` subject's `sd:endpoint`, `sd:supportedLanguage`,
 * `sd:feature`, `sd:resultFormat`, `sd:inputFormat`, `sd:extensionFunction`, and follows the
 * `sd:defaultDataset` → `sd:namedGraph` → (`sd:name`, `sd:graph` → `void:triples`) blank-node
 * chain (`descriptors.rs` `sd_ntriples`) to enumerate the named graphs.
 */
export function extractServiceDescription(
  statements: RdfStatement[],
): ServiceDescriptionSummary {
  const summary: ServiceDescriptionSummary = {
    serviceIri: null,
    endpoint: null,
    supportedLanguages: [],
    features: [],
    resultFormats: [],
    inputFormats: [],
    extensionFunctions: [],
    namedGraphs: [],
  };

  // The sd:Service subject (the document's root).
  let serviceKey: string | null = null;
  for (const st of statements) {
    if (
      iriValue(st.p) === RDF_TYPE &&
      iriValue(st.o) === `${SD_NS}Service` &&
      st.s.kind === "iri"
    ) {
      serviceKey = termKey(st.s);
      summary.serviceIri = st.s.value;
      break;
    }
  }
  if (serviceKey === null) return summary;

  let defaultDatasetKey: string | null = null;
  for (const st of statements) {
    if (termKey(st.s) !== serviceKey) continue;
    const p = iriValue(st.p);
    const o = iriValue(st.o);
    if (p === `${SD_NS}endpoint` && o) summary.endpoint = o;
    else if (p === `${SD_NS}supportedLanguage` && o) summary.supportedLanguages.push(o);
    else if (p === `${SD_NS}feature` && o) summary.features.push(o);
    else if (p === `${SD_NS}resultFormat` && o) summary.resultFormats.push(o);
    else if (p === `${SD_NS}inputFormat` && o) summary.inputFormats.push(o);
    else if (p === `${SD_NS}extensionFunction` && o) summary.extensionFunctions.push(o);
    else if (p === `${SD_NS}defaultDataset`) defaultDatasetKey = termKey(st.o);
  }

  // Follow defaultDataset → sd:namedGraph → (_:ng) → sd:name / sd:graph → (_:ngG) → void:triples.
  if (defaultDatasetKey !== null) {
    const namedGraphKeys: string[] = [];
    for (const st of statements) {
      if (termKey(st.s) === defaultDatasetKey && iriValue(st.p) === `${SD_NS}namedGraph`) {
        namedGraphKeys.push(termKey(st.o));
      }
    }
    for (const ngKey of namedGraphKeys) {
      let name: string | null = null;
      let graphKey: string | null = null;
      for (const st of statements) {
        if (termKey(st.s) !== ngKey) continue;
        const p = iriValue(st.p);
        if (p === `${SD_NS}name`) name = iriValue(st.o);
        else if (p === `${SD_NS}graph`) graphKey = termKey(st.o);
      }
      if (name === null) continue; // an sd:name-less entry is not FROM NAMED-referenceable
      let triples: number | null = null;
      if (graphKey !== null) {
        for (const st of statements) {
          if (termKey(st.s) === graphKey && iriValue(st.p) === `${VOID_NS}triples`) {
            triples = intValue(st.o);
            break;
          }
        }
      }
      summary.namedGraphs.push({ name, triples });
    }
  }

  return summary;
}

/** Extract both descriptors' facts from a single parsed-statements list (used for SD which
 * also carries the default-graph `void:triples` count; VoID is a separate document). */
export function extractCapabilities(statements: RdfStatement[]): CapabilitiesSummary {
  return {
    void: extractVoidSummary(statements),
    service: extractServiceDescription(statements),
  };
}

// ---------------------------------------------------------------------------
// The fetch orchestration — health + metrics + descriptors, with honest 404 handling.
// ---------------------------------------------------------------------------

/**
 * The outcome of reading ONE operational endpoint. `"ok"` carries the parsed payload;
 * `"not-exposed"` is the server's honest `404` (the opt-in feature is OFF — the panel says
 * so, it is not an error); `"unauthorized"` is a `401` (the read gate needs a token);
 * `"error"` is any other HTTP failure or a transport failure (CORS / refused / mixed-content)
 * with an honest message. A discriminated union so the UI renders each state distinctly.
 */
export type FetchOutcome<T> =
  | { status: "ok"; data: T }
  | { status: "not-exposed" }
  | { status: "unauthorized"; message: string }
  | { status: "error"; message: string };

/** The aggregate server-health snapshot: liveness + metrics + capabilities, each outcome-tagged. */
export interface ServerHealth {
  /** `GET /health` — plaintext `ok` when live. */
  health: FetchOutcome<{ body: string }>;
  /** `GET /metrics` — the parsed Prometheus exposition. `not-exposed` is not used (always on). */
  metrics: FetchOutcome<ParsedMetrics>;
  /** `GET /.well-known/void` — the VoID dataset summary (opt-in; `not-exposed` when the feature is off). */
  voidDescriptor: FetchOutcome<VoidSummary>;
  /** `GET /sparql` (no query) — the SPARQL Service Description (opt-in; `not-exposed` when off). */
  serviceDescription: FetchOutcome<ServiceDescriptionSummary>;
}

/** Add `Authorization: Bearer <token>` iff a non-empty token is configured. */
function authHeaders(
  base: Record<string, string>,
  token?: string,
): Record<string, string> {
  const t = (token ?? "").trim();
  if (t !== "") base["Authorization"] = `Bearer ${t}`;
  return base;
}

/** Map a transport failure (the opaque browser `TypeError`) to an honest error message. */
function transportError(e: unknown): string {
  const base = e instanceof Error ? e.message : String(e);
  return (
    `Could not reach the server (${base}). This is usually a CORS block (sparq-server emits ` +
    `no CORS headers by default — opt this origin in with --cors-allow-origin), a refused ` +
    `connection (is the server running and reachable?), or a mixed-content block (HTTPS page ` +
    `→ http endpoint).`
  );
}

/**
 * Fetch one operational endpoint and map its response to a {@link FetchOutcome} via `onOk`.
 * Classifies a status in `notExposedStatuses` → `not-exposed` (the opt-in feature is off),
 * `401` → `unauthorized`, any other non-2xx → `error`, and a transport failure → `error`
 * with the CORS/refused hint.
 *
 * `notExposedStatuses` defaults to `[404]` — the VoID endpoint and a disabled `/metrics`
 * answer `404`. The Service Description is the exception: it is served on `GET /sparql` with
 * NO `query`, and when the opt-in `federation-descriptors` feature is OFF the server falls
 * through to the historical `400 missing 'query'` (`crates/sparq-server/src/http.rs`), so the
 * SD read passes `[404, 400]` to treat that fall-through as "not exposed" too rather than a
 * scary error.
 */
async function fetchOutcome<T>(
  url: string,
  init: RequestInit,
  fetchImpl: typeof fetch,
  onOk: (resp: Response) => Promise<T>,
  notExposedStatuses: readonly number[] = [404],
): Promise<FetchOutcome<T>> {
  let resp: Response;
  try {
    resp = await fetchImpl(url, init);
  } catch (e) {
    return { status: "error", message: transportError(e) };
  }
  if (notExposedStatuses.includes(resp.status)) return { status: "not-exposed" };
  if (resp.status === 401) {
    return {
      status: "unauthorized",
      message:
        "The server gates reads behind a Bearer token (--auth-token-read). Enter a valid token in the Connect panel.",
    };
  }
  if (!resp.ok) {
    return { status: "error", message: `Request rejected (${resp.status}).` };
  }
  try {
    return { status: "ok", data: await onOk(resp) };
  } catch (e) {
    const base = e instanceof Error ? e.message : String(e);
    return { status: "error", message: `Could not parse the server response: ${base}` };
  }
}

/** Parse an RDF descriptor body (N-Triples) into statements, then reshape via `extract`. */
async function parseDescriptor<T>(
  resp: Response,
  extract: (statements: RdfStatement[]) => T,
): Promise<T> {
  const text = await resp.text();
  const { statements } = parseNTriples(text);
  return extract(statements);
}

/**
 * [OPUS-4.8] sq-he72 — read the connected server's full operational surface: `/health`,
 * `/metrics`, the opt-in `/.well-known/void` and the opt-in Service Description (a
 * `GET /sparql` with no `query`). Each is fetched independently off the configured endpoint's
 * ORIGIN (not under `/sparql`) so one disabled feature never blocks the others, and each is
 * outcome-tagged so the UI can render "live / not exposed / unauthorized / error" honestly.
 *
 * The bearer token (if configured) is sent only in the `Authorization: Bearer` header — the
 * channel the server's read gate validates — and is never logged. Descriptors are requested
 * as `application/n-triples` (the server serialises it) so this client parses one wire shape.
 *
 * Throws only if the configured endpoint URL is not a valid absolute http(s) URL (a caller
 * should gate on `connectionSafetyWarnings` first); every reachable-but-failing case is folded
 * into a {@link FetchOutcome} rather than thrown.
 */
export async function fetchServerHealth(
  config: EndpointConfig,
  fetchImpl: typeof fetch = fetch,
): Promise<ServerHealth> {
  const healthUrl = deriveServerUrl(config.url, HEALTH_PATH);
  const metricsUrl = deriveServerUrl(config.url, METRICS_PATH);
  const voidUrl = deriveServerUrl(config.url, VOID_PATH);
  // The Service Description is served on the SPARQL endpoint itself (a GET with no `query`).
  const sdUrl = parseEndpointUrl(config.url)?.toString() ?? null;
  if (!healthUrl || !metricsUrl || !voidUrl || !sdUrl) {
    throw new Error(
      "Enter a valid absolute endpoint URL (http:// or https://) in the Connect panel before reading server health.",
    );
  }

  const auth = (extra: Record<string, string> = {}) =>
    authHeaders({ ...extra }, config.token);

  // Run the four reads concurrently — each is independent and outcome-tagged.
  const [health, metrics, voidDescriptor, serviceDescription] = await Promise.all([
    fetchOutcome(
      healthUrl,
      { method: "GET", headers: auth() },
      fetchImpl,
      async (r) => ({ body: (await r.text()).trim() }),
    ),
    fetchOutcome(
      metricsUrl,
      { method: "GET", headers: auth() },
      fetchImpl,
      async (r) => parsePrometheusMetrics(await r.text()),
    ),
    fetchOutcome(
      voidUrl,
      { method: "GET", headers: auth({ Accept: "application/n-triples" }) },
      fetchImpl,
      (r) => parseDescriptor(r, extractVoidSummary),
    ),
    fetchOutcome(
      sdUrl,
      { method: "GET", headers: auth({ Accept: "application/n-triples" }) },
      fetchImpl,
      (r) => parseDescriptor(r, extractServiceDescription),
      // SD-off falls through to `400 missing 'query'`, not a 404 — treat both as not-exposed.
      [404, 400],
    ),
  ]);

  return { health, metrics, voidDescriptor, serviceDescription };
}

// ---------------------------------------------------------------------------
// Display helpers (pure shaping — so the site + a Tauri webview render identically).
// ---------------------------------------------------------------------------

/** Abbreviate a well-known VoID / SD IRI to a short `prefix:Local` form for display. */
export function shortenIri(iri: string): string {
  if (iri.startsWith(SD_NS)) return `sd:${iri.slice(SD_NS.length)}`;
  if (iri.startsWith(VOID_NS)) return `void:${iri.slice(VOID_NS.length)}`;
  if (iri === RDF_TYPE) return "rdf:type";
  return iri;
}

/**
 * Format a metric sample's label set as a compact `{name="v",…}` string (empty `""` when the
 * sample has no labels), for inline display next to its value.
 */
export function formatMetricLabels(labels: MetricSample["labels"]): string {
  if (labels.length === 0) return "";
  return `{${labels.map((l) => `${l.name}="${l.value}"`).join(",")}}`;
}
