// [OPUS-4.8] sq-17nw — a minimal ESM hook that transpiles the site's `.ts`
// modules on the fly with the already-installed `typescript` devDependency, so
// `node --test` can import and unit-test pure helpers WITHOUT pulling in a heavy
// test runner (vitest/jest) or a separate build step. Type-erasure only; no
// type-checking (that is the job of `next build`).
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import ts from 'typescript';

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
