// [OPUS-4.8] sq-tp1m (#757) — the GUI-side loader for the tier-b "W-reason" wasm bundle
// (crates/sparq-reason-wasm), the SAME reasoner the marketing site's /surface/inference page
// runs (site/src/lib/reason-wasm.ts). It is a SEPARATE, lazy-loaded bundle from the lean
// sparq-wasm triplestore engine the workbench queries with: the forward-chaining RDFS / OWL 2
// RL closure, materialised on demand so a query can match an ENTAILED triple.
//
// Like the site loader (and the @sparq/client wasm loader), the wasm-pack glue is imported at
// RUNTIME with a webpackIgnore dynamic import from /public, so the ~2.5 MB wasm never enters
// the page bundle; the asset URLs are prefixed with the SAME NEXT_PUBLIC_BASE_PATH the rest of
// the GUI keys off (@/lib/base-path), so they resolve under both the Tauri root-relative export
// and the hosted "/app" sub-path. The bundle is OPTIONAL: a build that did not sync it
// (see gui/app/scripts/sync-wasm.mjs) surfaces an honest "reasoning unavailable" state at
// runtime rather than crashing — and, rather than silently answering un-reasoned, queries then
// hard-fail with a clear message while inference is on (until it is turned off or the bundle is
// rebuilt), as the UI says.

import { basePath } from "@/lib/base-path";
import type { WorkspaceInferenceMode } from "@sparq/client";

/**
 * The JS-facing `Reasoner` surface the W-reason bundle exports (mirrors crates/sparq-reason-wasm
 * and site/src/lib/reason-wasm.ts). Every method is a STATELESS one-shot: it parses the supplied
 * document, runs the closure, and returns a string — so the JS side never holds a long-lived
 * reasoner handle. `profile` is `"rdfs"` or `"owl-rl"`; document `format` is one of `"turtle"` |
 * `"ntriples"` | `"nquads"` | `"trig"` (named graphs are folded into the default graph).
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
   * [FABLE-5] sq-ixc3.20 — ONE derivation of the triple `(s, p, o)` from the asserted base
   * of `data` under `profile` (`"rdfs"` | `"owl-rl"`), as `sparq-reason`'s proof-tree JSON —
   * or the JSON literal `"null"` when the triple is not entailed. `s`/`p`/`o` are N-Triples
   * term strings. Present only in a bundle built with the `explain` feature (the published
   * GUI bundle is — `js`'s `build:reason-wasm`); callers feature-detect at runtime so an
   * older synced bundle degrades honestly instead of crashing.
   */
  why(data: string, format: string, profile: string, s: string, p: string, o: string): string;
  /**
   * [FABLE-5] sq-ixc3.20 — as {@link why}, for N3 RULES mode: `n3` is the combined
   * rules + base-facts document (exactly what {@link reasonN3} consumes). Internal nodes
   * name the fired rule as `n3-rule-<i>`. `explain`-feature bundles only; feature-detect.
   */
  whyN3(n3: string, s: string, p: string, o: string): string;
}

interface ReasonModule {
  default: (opts?: { module_or_path: string | URL }) => Promise<unknown>;
  Reasoner: WasmReasoner;
}

let modulePromise: Promise<ReasonModule> | null = null;

/**
 * [OPUS-4.8] sq-tp1m — load + initialise the W-reason bundle once; subsequent calls reuse it.
 * The fetch + compile + instantiate is the expensive cold start (paid the first time a workspace
 * turns inference on). A failed load resets the cache so a later attempt retries (e.g. after a
 * build that syncs the bundle). Rejects when the bundle is not present in this build.
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
      modulePromise = null; // allow retry on transient failure / a later synced build
    });
  }
  const mod = await modulePromise;
  return mod.Reasoner;
}

// ---------------------------------------------------------------------------
// Inference-mode metadata (the per-workspace toggle's UI vocabulary).
// ---------------------------------------------------------------------------

/**
 * The reasoner PROFILE string for RDFS / OWL 2 RL modes. Narrowed to `"rdfs" | "owl-rl"` —
 * N3 mode calls `reasoner.reasonN3(...)` directly and never maps through this function. The
 * mode strings equal the reasoner's own profile names, so this is the identity — but keeping
 * it explicit documents the contract and gives one place to change if the two ever diverge.
 */
export function modeToProfile(mode: "rdfs" | "owl-rl"): string {
  return mode;
}

/** Short label + one-line description for each inference mode (the selector + tool copy). */
export const INFERENCE_MODE_META: Record<
  WorkspaceInferenceMode,
  { label: string; short: string; blurb: string }
> = {
  off: {
    label: "No inference",
    short: "Off",
    blurb: "Plain SPARQL over the asserted data — no entailment.",
  },
  rdfs: {
    label: "RDFS",
    short: "RDFS",
    blurb:
      "RDFS entailment: subclass/subproperty, domain & range typing are forward-chained before the query.",
  },
  "owl-rl": {
    label: "OWL 2 RL",
    short: "OWL RL",
    blurb:
      "OWL 2 RL entailment (a superset of RDFS): also inverse/symmetric/transitive properties, sameAs, and more.",
  },
  n3: {
    label: "N3 Rules",
    short: "N3",
    blurb:
      "N3 rule reasoning: forward-chain custom { antecedent } => { consequent } rules over the live store. Attach rules in the Inference tool. Derived ground triples only.",
  },
};
