// [FABLE-5] sq-ohnj1 (GH #1499, epic sq-xqchl) — @sparq-org/eyereasoner-compat.
//
// A drop-in for eye-js's `n3reasoner(data, query?, options?)`, backed by sparq's native N3
// forward-chaining reasoner compiled to WebAssembly (crates/sparq-reason-wasm). The wasm ships
// INSIDE this npm tarball; no SWI-Prolog / EYE image is used. The eye-js SWIPL-bound surface is
// re-exported as throwing migration stubs. See the package README for the honest boundary map,
// output-mode table, and builtins-coverage table.

import type { Quad } from '@rdfjs/types';
import { loadReasoner } from './loader.js';
import { parseNTriples, writeQuads } from './rdf.js';

export { configureWasm } from './loader.js';
export type { WasmInput } from './loader.js';
export { dataFactory, parseNTriples, writeQuads } from './rdf.js';

// ---- eye-js-compatible types (mirrors eye-js lib/transformers.ts) ----

/** The `output` mode for an implicit (query-less) reasoning call. */
export type ICoreQueryOptions = {
  output?: 'derivations' | 'deductive_closure' | 'deductive_closure_plus_rules'
    | 'grounded_deductive_closure_plus_rules' | 'none';
};

export type Options = ICoreQueryOptions & {
  outputType?: 'string' | 'quads';
  /** Accepted for source compatibility with eye-js; sparq uses its native engine (ignored, warns). */
  SWIPL?: unknown;
  /** Accepted for source compatibility with eye-js; not supported by the sparq backend (ignored, warns). */
  cb?: (res: string) => Promise<string>;
};

export type Data = Quad[] | string;
export type InputData = Data | [string, ...string[]];
export type Query = Data | undefined;

// ---- n3reasoner ----

/** Normalise the `data` overloads into the reasoner's single N3 document. */
function inputDataToStrings(data: InputData): string[] {
  if (typeof data === 'string') return [data];
  if (data.length > 0 && typeof data[0] === 'string') return data as string[];
  return data.length > 0 ? [writeQuads(data as Quad[])] : [];
}

function queryToString(query: Exclude<Query, undefined>): string {
  return typeof query === 'string' ? query : writeQuads(query);
}

/** Decide the result type exactly as eye-js does (default = the input's type). */
function wantsQuads(data: InputData, outputType: Options['outputType']): boolean {
  return outputType === 'quads'
    || (typeof data !== 'string' && typeof (data as [unknown])[0] !== 'string' && outputType !== 'string');
}

async function run(data: InputData, query: Query, options?: Options): Promise<Quad[] | string> {
  if (options?.SWIPL !== undefined) {
    // Positional-only warning; the sparq backend has no SWI-Prolog module to swap in.
    console.warn('[eyereasoner-compat] the `SWIPL` option is ignored: sparq uses its native N3 engine.');
  }
  if (options?.cb !== undefined) {
    console.warn('[eyereasoner-compat] the `cb` (question/answer) option is not supported and is ignored.');
  }

  const R = await loadReasoner();
  const dataDoc = inputDataToStrings(data).join('\n');

  let nt: string;
  if (query !== undefined) {
    // eye-js forbids combining an explicit `output` with an explicit query.
    if (options?.output !== undefined) {
      throw new Error('Cannot use explicit output with explicit query');
    }
    nt = R.reasonN3Query(dataDoc, queryToString(query));
  } else {
    const output = options?.output;
    switch (output) {
      case undefined:
      case 'derivations':
        nt = R.reasonN3New(dataDoc); // EYE --pass-only-new
        break;
      case 'deductive_closure':
        nt = R.reasonN3(dataDoc); // EYE --pass
        break;
      case 'none':
        nt = '';
        break;
      case 'deductive_closure_plus_rules':
      case 'grounded_deductive_closure_plus_rules':
        // EYE's --pass-all / --pass-all-ground echo the RULES into the output; sparq's forward
        // chainer CONSUMES rules (it emits only entailed ground triples), so there is no sound
        // equivalent yet. Fail loudly rather than silently return a different result set.
        throw new Error(
          `output mode '${output}' is not yet supported by @sparq-org/eyereasoner-compat `
          + "(sparq's forward chainer consumes rules and emits only ground triples). Use "
          + "'deductive_closure' for the ground closure. Tracked as a follow-up bead — see the README.",
        );
      default:
        throw new Error(`Unknown output option: ${String(output)}`);
    }
  }

  return wantsQuads(data, options?.outputType) ? parseNTriples(nt) : nt;
}

/* eslint-disable max-len */
export function n3reasoner(data: InputData, query: Query, options: { outputType: 'string' } & Options): Promise<string>;
export function n3reasoner(data: InputData, query: Query, options: { outputType: 'quads' } & Options): Promise<Quad[]>;
export function n3reasoner(data: Quad[], query?: Query, options?: { outputType?: undefined } & Options): Promise<Quad[]>;
export function n3reasoner(data: string | [string, ...string[]], query?: Query, options?: { outputType?: undefined } & Options): Promise<string>;
export function n3reasoner(data: InputData, query?: Query, options?: Options): Promise<Quad[] | string>;
export function n3reasoner(data: InputData, query?: Query, options?: Options): Promise<Quad[] | string> {
  return run(data, query, options);
}
/* eslint-enable max-len */

// ---- SWIPL/EYE-image-bound surface: throwing migration stubs ----
//
// These operate on a SWI-Prolog `SWIPLModule` / EYE `.pvm` image and are inherently EYE-bound;
// sparq's native reasoner cannot back them. They are re-exported so an `import` compiles, and
// throw a clear migration error if actually invoked. Migrate to `n3reasoner(...)`.

const MIGRATE = 'Migrate to n3reasoner(data, query?, options?). See the @sparq-org/eyereasoner-compat README.';
function swiplStub(name: string): never {
  throw new Error(
    `${name} is SWI-Prolog / EYE-image specific and is not backed by sparq's native reasoner. ${MIGRATE}`,
  );
}

export function SwiplEye(..._args: unknown[]): never { return swiplStub('SwiplEye'); }
export function loadEyeImage(..._args: unknown[]): never { return swiplStub('loadEyeImage'); }
export function loadImage(..._args: unknown[]): never { return swiplStub('loadImage'); }
export function runQuery(..._args: unknown[]): never { return swiplStub('runQuery'); }
export function buildQuery(..._args: unknown[]): never { return swiplStub('buildQuery'); }
export function qaQuery(..._args: unknown[]): never { return swiplStub('qaQuery'); }
export function query(..._args: unknown[]): never { return swiplStub('query'); }
export function queryOnce(..._args: unknown[]): never { return swiplStub('queryOnce'); }
export function executeBasicEyeQuery(..._args: unknown[]): never { return swiplStub('executeBasicEyeQuery'); }

/** The RDF-Lingua (`SEE`) variant is a different image/rule-language; not backed by sparq. */
export function linguareasoner(..._args: unknown[]): never {
  throw new Error(`linguareasoner (RDF Lingua / SEE image) is not supported by the sparq backend. ${MIGRATE}`);
}

/**
 * `EYE_PVM` is the compiled SWI-Prolog EYE image bytes in eye-js; there is no such image here.
 * Re-exported as a guard that throws on any property access so accidental use surfaces clearly.
 */
export const EYE_PVM: string = new Proxy({}, {
  get() { throw new Error(`EYE_PVM (the SWI-Prolog EYE image) is not available in the sparq backend. ${MIGRATE}`); },
}) as unknown as string;
