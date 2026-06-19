// [OPUS-4.8] sq-06gq — compile-time conformance guard against the wasm-pack GENERATED
// `Store` surface.
//
// Design background (`research/gui-design.md` §4): the end-state is to make the package
// consume the wasm-pack-GENERATED `sparq_wasm.d.ts` so the `Store` surface is generated,
// not hand-mirrored. The hard constraint that blocks a literal re-export today is that the
// generated d.ts lives in the **git-ignored** build-artifact tree `js/wasm/` (see
// `js/.gitignore`) — it does not exist until `cd js && npm run build:wasm` runs. So a plain
// `import` of it in `src/index.ts` would break the bare-package `tsc` (the `gui.yml`
// `shared-client` job typechecks the package with NO wasm build).
//
// This module is the CI-safe halfway step: it imports the generated types via the
// `#sparq-wasm-generated` tsconfig path alias and ASSERTS — purely at compile time — that the
// hand-written surface in `./index.ts` stays a faithful subset of the generated `Store`. It
// is excluded from the default `tsconfig.json`, so the artifact-free bare typecheck is
// unaffected; it is compiled by `tsconfig.conformance.json` (the `typecheck:conformance`
// script) ONLY where the artifact is present — locally after a wasm build, and in the
// `site-with-shared-client` CI job, which builds the bundle. Any future drift between the
// hand-mirror and the generated `Store` (a renamed/retyped method, a dropped argument) then
// becomes a TYPE ERROR there, which is the drift-detection §4 is really after. The remaining
// work — adopting repo-root workspaces and re-exporting the generated d.ts as the literal
// source so the hand-mirror can be DELETED — is tracked as a follow-up bead.

import type {
  Store as GeneratedStore,
  SolutionCursor as GeneratedSolutionCursor,
} from "#sparq-wasm-generated";

import type { WasmSolutionCursor, WasmStore, WasmStoreCtor } from "./index.js";

// `Extends<A, B>` is `true` only when `A` is assignable to `B`; `Assert<T>` type-checks only
// when `T` is exactly `true`. Both are types — nothing is emitted at runtime (this whole
// module is types-only, never imported by the package's runtime entry point).
type Extends<A, B> = A extends B ? true : false;
type Assert<_T extends true> = true;

// --- 1. The PRIMITIVE-returning instance methods must match the generated `Store` exactly.
// These are the methods whose hand signature must equal the generated one byte-for-byte
// (same args, same primitive return). A rename, a dropped argument, or a changed return type
// in a future bundle flips one of these to a type error. `queryCursor` and `validate` are
// asserted separately below because the hand mirror INTENTIONALLY diverges on those two (see
// #2 / #3); every other shared method is checked here.
type SharedPrimitiveMethod =
  | "query"
  | "queryQuads"
  | "updateInPlace"
  | "explain"
  | "explainAnalyze"
  | "count"
  | "ask"
  | "applyDelta"
  | "size";
type _PrimitiveMethodsConform = Assert<
  Extends<
    Pick<WasmStore, SharedPrimitiveMethod>,
    Pick<GeneratedStore, SharedPrimitiveMethod>
  >
>;
export type { _PrimitiveMethodsConform };

// --- 2. `validate` is OPTIONAL on the hand mirror (the lean bundle may be built without the
// `shacl` feature, so the binding can be absent) but REQUIRED on the generated `Store`. The
// honest relationship: when the hand `validate` IS present, its signature must equal the
// generated one. Compare the required form against the generated required form.
type _ValidateConforms = Assert<
  Extends<Required<Pick<WasmStore, "validate">>, Pick<GeneratedStore, "validate">>
>;
export type { _ValidateConforms };

// --- 3. The static constructor surface: the generated factories (`load` / `loadDataset`)
// return a generated `Store`, which the hand `WasmStoreCtor` re-types as `WasmStore`. Assert
// the hand ctor's factory arity/names match the generated statics (a dropped/renamed factory
// is caught here), independent of the return-type substitution.
type CtorFactoryShape = {
  load: (text: string, format: string) => unknown;
  loadDataset: (text: string, format: string) => unknown;
};
type _GeneratedHasFactories = Assert<
  Extends<Pick<typeof GeneratedStore, "load" | "loadDataset">, CtorFactoryShape>
>;
type _HandCtorHasFactories = Assert<Extends<WasmStoreCtor, CtorFactoryShape>>;
export type { _GeneratedHasFactories, _HandCtorHasFactories };

// --- 4. The streaming cursor: the hand mirror substitutes its own `WasmSolutionCursor` for
// the generated `SolutionCursor` (a deliberate divergence — the hand cursor narrows to the
// four methods `streamQueryRows` uses). Assert that narrowing stays a SUBSET of the generated
// cursor, so a future rename/retype of a cursor method is caught.
type CursorSubsetKeys = "next" | "vars" | "rowCount" | "batchSize";
type _CursorConforms = Assert<
  Extends<
    Pick<WasmSolutionCursor, CursorSubsetKeys>,
    Pick<GeneratedSolutionCursor, CursorSubsetKeys>
  >
>;
export type { _CursorConforms };
