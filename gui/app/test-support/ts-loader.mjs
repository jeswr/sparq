// (sq-rcuvq) [SONNET-4.6] — A minimal ESM hook that transpiles the gui/app's `.ts` modules
// on the fly with the already-installed `typescript` devDependency, so `node --test` can import
// and unit-test pure helpers WITHOUT pulling in a heavy test runner (vitest/jest) or a separate
// build step. Type-erasure only; no type-checking (that is the job of `next build` / `typecheck`).
//
// Adapted from site/test-support/ts-loader.mjs with an added resolve hook for explicit `.ts`
// specifiers so `node --test src/lib/foo.test.ts` works (the site uses .mjs test files that
// import .ts sources; here the test file itself is .ts).
import { access, readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import ts from 'typescript';

export async function resolve(specifier, context, next) {
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
