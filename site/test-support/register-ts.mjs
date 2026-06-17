// [OPUS-4.8] sq-17nw — registers the on-the-fly TypeScript ESM loader (ts-loader.mjs)
// so `node --test` can import the site's `.ts` helper modules directly. Used by the
// `test:unit` npm script; keeps the unit-test path build-free (no vitest/jest, no
// separate tsc emit) — type-CHECKING remains the job of `next build`.
import { register } from "node:module";
import { pathToFileURL } from "node:url";

register("./ts-loader.mjs", pathToFileURL(`${import.meta.dirname}/`));
