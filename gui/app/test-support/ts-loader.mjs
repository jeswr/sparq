// (sq-rcuvq) [SONNET-4.6] — A minimal ESM hook that transpiles the gui/app's `.ts` modules
// on the fly with the already-installed `typescript` devDependency, so `node --test` can import
// and unit-test pure helpers WITHOUT pulling in a heavy test runner (vitest/jest) or a separate
// build step. Type-erasure only; no type-checking (that is the job of `next build` / `typecheck`).
//
// Adapted from site/test-support/ts-loader.mjs with an added resolve hook for explicit `.ts`
// specifiers so `node --test src/lib/foo.test.ts` works (the site uses .mjs test files that
// import .ts sources; here the test file itself is .ts).
//
// (sq-1y04h) [SONNET-4.6] — Added `@sparq/client` tsconfig-path alias resolution so unit tests
// that import helpers which re-export from `@sparq/client` can run under `node --test`. The alias
// maps to the same source the Next.js build sees (../../packages/sparq-client/src/index.ts).
import { access, readFile } from 'node:fs/promises';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { resolve as resolvePath, dirname } from 'node:path';
import ts from 'typescript';

// The tsconfig.json paths alias: `@sparq/client` → `../../packages/sparq-client/src/index.ts`
// resolved relative to the gui/app directory (import.meta.dirname of THIS loader file is
// gui/app/test-support/, so `..` takes us to gui/app/ and `../..` to the repo root).
const SPARQ_CLIENT_ENTRY = resolvePath(
  dirname(fileURLToPath(import.meta.url)),
  '../../../packages/sparq-client/src/index.ts',
);

// [SONNET-4.6] sq-ixc3.17 — the tsconfig `@/*` → `./src/*` alias, so a test can import an app
// module that uses the alias internally (zk-prover.ts imports `@/lib/base-path`). Only `.ts`
// candidates are tried: `load` below transpiles `.ts`, not `.tsx`, so pointing at a component
// would fail later with a confusing error instead of here with a clear one.
const SRC_ROOT = resolvePath(dirname(fileURLToPath(import.meta.url)), '../src');

export async function resolve(specifier, context, next) {
  // Resolve the @sparq/client tsconfig path alias to the package's real TypeScript entry.
  // This lets gui/app unit tests that import helpers which use @sparq/client work in Node.
  if (specifier === '@sparq/client') {
    return { shortCircuit: true, url: pathToFileURL(SPARQ_CLIENT_ENTRY).href };
  }

  if (specifier.startsWith('@/')) {
    const base = resolvePath(SRC_ROOT, specifier.slice(2));
    for (const candidate of [`${base}.ts`, `${base}/index.ts`]) {
      try {
        await access(candidate);
        return { shortCircuit: true, url: pathToFileURL(candidate).href };
      } catch {
        // try the next candidate
      }
    }
    throw new Error(`ts-loader: cannot resolve "${specifier}" to a .ts module under ${SRC_ROOT}`);
  }

  // Allow explicit `.ts` imports (relative paths or file: URLs) so the test runner can load
  // a `.ts` test file and tests can import `.ts` sources with the `.ts` extension directly.
  if (specifier.endsWith('.ts') && !specifier.endsWith('.d.ts')) {
    try {
      let href;
      if (specifier.startsWith('file://')) {
        href = specifier;
      } else {
        href = new URL(specifier, context.parentURL).href;
      }
      return { shortCircuit: true, url: href };
    } catch {
      // fall through to default resolution
    }
  }

  // Rewrite an explicit `.js` import specifier to the sibling `.ts` source when that `.ts`
  // exists. Needed for packages that use NodeNext `.js`-extension conventions internally.
  if (specifier.endsWith('.js') && (specifier.startsWith('./') || specifier.startsWith('../'))) {
    try {
      const tsUrl = new URL(specifier.slice(0, -'.js'.length) + '.ts', context.parentURL);
      await access(fileURLToPath(tsUrl));
      return next(tsUrl.href, context);
    } catch {
      // No sibling `.ts` — fall through to the default resolution of the real `.js`.
    }
  }
  return next(specifier, context);
}

export async function load(url, context, next) {
  if (url.endsWith('.ts') && !url.endsWith('.d.ts')) {
    const source = await readFile(fileURLToPath(url), 'utf8');
    const { outputText } = ts.transpileModule(source, {
      compilerOptions: {
        module: ts.ModuleKind.ESNext,
        target: ts.ScriptTarget.ES2022,
        isolatedModules: true,
        verbatimModuleSyntax: false,
      },
      fileName: fileURLToPath(url),
    });
    return { format: 'module', source: outputText, shortCircuit: true };
  }
  return next(url, context);
}
