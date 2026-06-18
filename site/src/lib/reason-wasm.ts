// [OPUS-4.8] sq-0po6 — live loader for the tier-b "W-reason" wasm bundle that drives
// /surface/inference. This is a SEPARATE, lazy-loaded bundle from the lean sparq-wasm
// triplestore bundle (crates/sparq-reason-wasm → public/wasm/reason): the reasoner
// (RDFS / OWL 2 RL / N3 forward-chaining closure) plus the `why()` derivation proof
// tree, built with `--features explain`. The lean landing page never pays for it; the
// inference page loads it on first interaction, exactly as the SHACL playground loads
// the SHACL-enabled engine bundle.
//
// As with src/lib/sparq-wasm.ts the wasm-pack glue is imported at RUNTIME with a
// webpackIgnore dynamic import, so the ~2.5 MB wasm never enters the page bundle and we
// pass the wasm bytes' URL explicitly (prefixed with the Pages basePath) rather than let
// the glue's `new URL('…_bg.wasm', import.meta.url)` resolution run.

/**
 * The JS-facing `Reasoner` surface the W-reason bundle exports (mirrors
 * crates/sparq-reason-wasm). Every method is a STATELESS one-shot: it parses the supplied
 * document, runs the closure, and returns a string — so the JS side never holds a
 * long-lived reasoner handle. `profile` is `"rdfs"` or `"owl-rl"`; document `format` is one
 * of `"turtle"` | `"ntriples"` | `"nquads"` | `"trig"`.
 */
export interface WasmReasoner {
  /** Full deductive closure (asserted base + every entailed triple) as N-Triples. */
  materialize(data: string, format: string, profile: string): string;
  /** Only the NEWLY-entailed triples (closure minus the asserted base) as N-Triples. */
  entailed(data: string, format: string, profile: string): string;
  /** `{"profile","baseTriples","closureTriples","entailed"}` JSON — counts only. */
  materializeStats(data: string, format: string, profile: string): string;
  /** N3 rule reasoning (`{ … } => { … }` + facts) -> entailed ground triples as N-Triples. */
  reasonN3(n3: string): string;
  /**
   * One derivation of the triple `(s, p, o)` under `profile` as a proof-tree JSON document,
   * or the JSON literal `null` if the triple is not entailed. `s`/`p`/`o` are N-Triples term
   * strings. Present only when the bundle is built with the `explain` feature (the site's
   * default — see scripts/sync-wasm.mjs).
   */
  why?: (
    data: string,
    format: string,
    profile: string,
    s: string,
    p: string,
    o: string,
  ) => string;
}

interface ReasonModule {
  default: (opts?: { module_or_path: string | URL }) => Promise<unknown>;
  Reasoner: WasmReasoner;
}

let modulePromise: Promise<ReasonModule> | null = null;

function basePath(): string {
  // next.config.ts sets basePath '/sparq'; mirror it for the runtime asset URLs.
  return process.env.NEXT_PUBLIC_BASE_PATH ?? "/sparq";
}

/**
 * [OPUS-4.8] sq-0po6 — loads + initialises the W-reason bundle once; subsequent calls reuse
 * it. The fetch + compile + instantiate is the expensive cold start. {@link prewarmReason}
 * kicks this off eagerly on route mount so the first "Reason" pays no cold start. A failed
 * load resets the cache so a later call retries.
 */
export async function loadReasoner(): Promise<WasmReasoner> {
  if (!modulePromise) {
    modulePromise = (async () => {
      const base = basePath();
      const gluePath = `${base}/wasm/reason/sparq_reason_wasm.js`;
      const wasmPath = `${base}/wasm/reason/sparq_reason_wasm_bg.wasm`;
      const mod = (await import(/* webpackIgnore: true */ gluePath)) as ReasonModule;
      await mod.default({ module_or_path: new URL(wasmPath, window.location.origin) });
      return mod;
    })();
    modulePromise.catch(() => {
      modulePromise = null; // allow retry on transient failure
    });
  }
  const mod = await modulePromise;
  return mod.Reasoner;
}

/**
 * [OPUS-4.8] sq-0po6 — eagerly pre-warm the W-reason bundle (fetch + compile + instantiate)
 * without blocking render. Safe to call on route mount and repeatedly; it shares the single
 * {@link loadReasoner} promise, so the cold start happens at most once.
 */
export function prewarmReason(): Promise<unknown> {
  return loadReasoner();
}

// ---- reasoning profiles ----

export type ReasoningProfile = "rdfs" | "owl-rl";

/** Human label for each profile, for the profile selector. */
export const PROFILE_LABEL: Record<ReasoningProfile, string> = {
  rdfs: "RDFS",
  "owl-rl": "OWL 2 RL",
};

// ---- materialize stats (mirrors Reasoner::materializeStats JSON) ----

/**
 * [OPUS-4.8] sq-0po6 — the base-vs-entailed triple-count summary the inference page shows:
 * `baseTriples` distinct asserted triples, `closureTriples` after forward-chaining, and
 * `entailed` = the number reasoning ADDED (`closureTriples - baseTriples`).
 */
export interface MaterializeStats {
  profile: string;
  baseTriples: number;
  closureTriples: number;
  entailed: number;
}

// ---- proof tree (mirrors sparq-reason's ProofTree::to_json) ----

/**
 * [OPUS-4.8] sq-0po6 — one node of a `why()` derivation proof tree. `rule === "asserted"`
 * marks a LEAF: the conclusion is an explicitly asserted base triple (`premises` empty);
 * `axiom-*` rules are premise-free tautologies; every other node is an application of the
 * named inference rule to the `premises` nodes (each an index strictly less than this
 * node's `id`). `conclusion` is the `[s, p, o]` triple as N-Triples term strings.
 */
export interface ProofNode {
  id: number;
  conclusion: [string, string, string];
  rule: string;
  premises: number[];
}

/**
 * [OPUS-4.8] sq-0po6 — a `why()` derivation: a flat, premises-before-conclusion DAG whose
 * `root` node (always the last) is the explained triple. The JSON shape `Reasoner.why`
 * returns (or `null` when the triple is not entailed).
 */
export interface ProofTree {
  root: number;
  nodes: ProofNode[];
}

/** A leaf node is an asserted base fact (the proof bottoms out here). */
export function isAsserted(node: ProofNode): boolean {
  return node.rule === "asserted";
}
