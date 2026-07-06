// [OPUS-4.8] sq-vw3ax.11 — ambient declaration for GLOBAL CSS side-effect imports.
//
// Next.js declares `*.module.css` / `*.module.sass` / `*.module.scss` (CSS Modules) in its own
// `next/types/global.d.ts`, but NOT a bare `*.css`. A GLOBAL stylesheet imported for its side
// effect — `import "./globals.css"` in `app/layout.tsx` — has no bindings, so historically TS
// simply did not type-check it. Under TypeScript's `noUncheckedSideEffectImports` (which newer
// TS defaults ON), that same side-effect import errors TS2882 without an ambient module for it.
//
// This one-line declaration makes the global CSS import resolvable regardless of that flag —
// harmless on older TypeScript (the module is simply unused) and forward-compatible with newer
// TypeScript. It declares only bare `*.css`; the more specific `*.module.css` (with its typed
// class map) still comes from Next's own declaration.
declare module "*.css";
