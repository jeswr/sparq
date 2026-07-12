// [FABLE-5] sq-ixc3.19 — the STRUCTURED query-plan client (the GUI plan explorer's wire
// layer).
//
// One schema, three sources: `sparq-engine`'s typed EXPLAIN tree
// (`explain_json::PlanNode`, sq-u4lgr/#902) reaches the GUI as identical camelCase JSON
// from (a) the in-tab wasm engine (`Store.explainPlanJson` / `explainPlanAnalyzeJson`),
// (b) the desktop-native Tauri command (`explain_native` — the only source with REAL
// per-operator wall nanos; wasm reads 0, no monotonic clock), and (c) a remote
// `sparq-server` (`Accept: application/x-sparq-explain+json` on `/sparql`). This module
// owns the shared TYPE (the sq-jbqh4 schema contract), the defensive parse, and the
// endpoint-mode fetch with its HONEST degradation ladder:
//
//   structured JSON (feature-on server) → plan TEXT (lean server answers 406 to the JSON
//   `Accept`, or a pre-feature server ignores it and answers the `explain` parameter with
//   `text/plain`) → a thrown `EndpointError` (real failures only).
//
// The text fallback is surfaced as `{ kind: "text" }` — never silently parsed as a tree —
// so the panel can render the honest "text plan only" state instead of a fabricated one.

import {
  type EndpointConfig,
  type PreparedRequest,
  EndpointError,
  buildSparqlRequest,
} from "./endpoint.js";

// ---------------------------------------------------------------------------
// The schema contract (sq-jbqh4).
// ---------------------------------------------------------------------------

/**
 * One operator of the engine's typed EXPLAIN tree — the exact camelCase JSON
 * `sparq_engine::explain_json::PlanNode::to_json()` emits (the sq-jbqh4 contract):
 *
 * - `estimated` — the planner's cardinality estimate (BGP/conjunctive leaves; `null`
 *   elsewhere, and on every node of a query form the planner does not estimate).
 * - `actual` — output rows observed by EXPLAIN ANALYZE (`null` in a plan-only dry run).
 * - `nanos` — wall time observed by ANALYZE (`null` plan-only; **reads 0 on wasm32** —
 *   no monotonic clock — so a 0 from the in-tab engine means "unmeasured", not "free").
 * - `qError` — max(est/actual, actual/est); `null` without both sides non-zero.
 */
export interface PlanNode {
  operator: string;
  estimated: number | null;
  actual: number | null;
  nanos: number | null;
  qError: number | null;
  children: PlanNode[];
}

/** The two explain forms (plan-only dry run vs execute-and-measure). */
export type PlanExplainMode = "plan" | "analyze";

/** The structured-plan media type the server negotiates on (`/sparql` Accept). */
export const EXPLAIN_JSON_CT = "application/x-sparq-explain+json";

/** The TEXT explain media type (T22) — the fallback the degradation ladder lands on. */
export const EXPLAIN_TEXT_CT = "text/x-sparq-explain";

// ---------------------------------------------------------------------------
// Defensive parse (the wire is trusted infrastructure, but never blindly).
// ---------------------------------------------------------------------------

function asNullableNumber(v: unknown, field: string): number | null {
  if (v === null || v === undefined) return null;
  if (typeof v === "number" && Number.isFinite(v)) return v;
  throw new Error(`plan node field "${field}" is not a number/null`);
}

/** Validate one parsed JSON value as a {@link PlanNode} tree (recursive; throws on shape drift). */
export function parsePlanNode(value: unknown): PlanNode {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("plan node is not an object");
  }
  const o = value as Record<string, unknown>;
  if (typeof o.operator !== "string" || o.operator === "") {
    throw new Error('plan node lacks an "operator" label');
  }
  const children = o.children === undefined ? [] : o.children;
  if (!Array.isArray(children)) throw new Error('plan node "children" is not an array');
  return {
    operator: o.operator,
    estimated: asNullableNumber(o.estimated, "estimated"),
    actual: asNullableNumber(o.actual, "actual"),
    nanos: asNullableNumber(o.nanos, "nanos"),
    qError: asNullableNumber(o.qError, "qError"),
    children: children.map(parsePlanNode),
  };
}

/** Parse a plan-tree JSON *string* (what the wasm binding / Tauri command return). */
export function parsePlanJson(json: string): PlanNode {
  return parsePlanNode(JSON.parse(json));
}

// ---------------------------------------------------------------------------
// Endpoint mode (the SPARQL 1.1 Protocol `/sparql` explain surface).
// ---------------------------------------------------------------------------

/** An endpoint explain outcome: the structured tree, or the honest text-plan fallback. */
export type EndpointExplainResult =
  | { kind: "tree"; plan: PlanNode }
  | { kind: "text"; plan: string };

/**
 * Build the explain request: the same authenticated `application/sparql-query` POST as
 * a read (reusing {@link buildSparqlRequest}'s bearer posture), with the `explain`
 * parameter in the URL query string (the body is the query itself) and the `Accept`
 * negotiating the structured vs text plan.
 */
export function buildExplainRequest(
  config: EndpointConfig,
  sparql: string,
  mode: PlanExplainMode,
  accept: string = EXPLAIN_JSON_CT,
): PreparedRequest {
  const prepared = buildSparqlRequest(config, sparql, "select");
  const url = new URL(prepared.url);
  url.searchParams.set("explain", mode);
  (prepared.init.headers as Record<string, string>)["Accept"] = accept;
  return { url: url.toString(), init: prepared.init };
}

/**
 * Run EXPLAIN / EXPLAIN ANALYZE against the endpoint, preferring the structured tree
 * and degrading HONESTLY to the text plan (`{ kind: "text" }`) when the server cannot
 * produce one — a lean (`--no-default-features`) build answers the JSON `Accept` with
 * 406, and a pre-`explain-json` server answers the `explain` parameter with
 * `text/plain`. Throws {@link EndpointError} on real failures (auth, parse-400, 5xx,
 * transport/CORS).
 */
export async function runEndpointExplain(
  config: EndpointConfig,
  sparql: string,
  mode: PlanExplainMode,
  fetchImpl: typeof fetch = fetch,
): Promise<EndpointExplainResult> {
  const send = async (accept: string): Promise<Response> => {
    let prepared: PreparedRequest;
    try {
      prepared = buildExplainRequest(config, sparql, mode, accept);
    } catch (e) {
      throw new EndpointError(
        `Invalid endpoint URL (${e instanceof Error ? e.message : String(e)})`,
        null,
      );
    }
    try {
      return await fetchImpl(prepared.url, prepared.init);
    } catch (e) {
      const base = e instanceof Error ? e.message : String(e);
      throw new EndpointError(
        `Could not reach the endpoint (${base}). This is usually a CORS block, a refused connection, or a mixed-content block (HTTPS page → http endpoint).`,
        null,
      );
    }
  };

  let resp = await send(EXPLAIN_JSON_CT);
  if (resp.status === 406) {
    // A lean server refuses the structured plan up front; the TEXT explain still works.
    resp = await send(EXPLAIN_TEXT_CT);
  }
  if (!resp.ok) {
    const detail = (await resp.text().catch(() => "")).trim();
    throw new EndpointError(
      `explain failed (HTTP ${resp.status}${detail ? `: ${detail.slice(0, 300)}` : ""})`,
      resp.status,
    );
  }
  const ct = resp.headers.get("content-type") ?? "";
  const body = await resp.text();
  if (ct.startsWith(EXPLAIN_JSON_CT)) {
    try {
      return { kind: "tree", plan: parsePlanJson(body) };
    } catch (e) {
      throw new EndpointError(
        `structured plan parse failed (${e instanceof Error ? e.message : String(e)})`,
        null,
      );
    }
  }
  // A pre-`explain-json` server ignores the JSON Accept but honours the `explain`
  // parameter with the text plan — surface it as the honest text fallback.
  return { kind: "text", plan: body };
}
