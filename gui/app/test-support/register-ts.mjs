// (sq-rcuvq) [SONNET-4.6] — Registers the on-the-fly TypeScript ESM loader (ts-loader.mjs) so
// `node --test` can import the gui/app's `.ts` helper modules directly. Used by the `test:unit`
// npm script; keeps the unit-test path build-free (no vitest/jest, no separate tsc emit) — type-
// checking remains the job of `npm run typecheck`.
import { register } from 'node:module';
import { pathToFileURL } from 'node:url';

register('./ts-loader.mjs', pathToFileURL(`${import.meta.dirname}/`));
