// [OPUS-5] sq-xqchl.3 — the classic-`<script>` IIFE bundle (`dist/eyereasoner-compat.iife.js`).
//
// The load-bearing invariants, exercised against the REAL artifact and the REAL engine (no
// mock reasoner): the bundle parses as a NON-MODULE script, assigns the `eyereasoner` global,
// resolves its `.wasm` relative to `document.currentScript.src` (a classic script has no
// `import.meta.url`), keeps the engine OUT of the bundle, and actually reasons.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const pkgDir = join(dirname(fileURLToPath(import.meta.url)), '..');
const ARTIFACT = join(pkgDir, 'dist', 'eyereasoner-compat.iife.js');
const ENGINE = join(pkgDir, 'wasm', 'sparq_reason_wasm_bg.wasm');

const CDN = 'https://cdn.example.invalid/npm/@sparq-org/eyereasoner-compat';
const SCRIPT_SRC = `${CDN}/dist/eyereasoner-compat.iife.js`;
const EXPECTED_WASM_URL = `${CDN}/wasm/sparq_reason_wasm_bg.wasm`;

const S = 'http://example.org/socrates#';
const RULE_DATA = `@prefix : <${S}>.
:Socrates a :Human.
{ ?x a :Human } => { ?x a :Mortal }.`;

/**
 * Evaluate the artifact the way a browser executes a classic `<script src=…>`.
 *
 * `new Function` parses its body in the SCRIPT goal, not the module goal, so any surviving
 * top-level `import`/`export` statement is a SyntaxError right here — that is the non-module
 * assertion. `document` and `fetch` are declared as parameters so they SHADOW the ambient Node
 * globals inside the bundle without the test mutating global state.
 */
async function loadIife(fetchImpl) {
  const source = await readFile(ARTIFACT, 'utf8');
  const factory = new Function('document', 'fetch', `${source}\n;return eyereasoner;`);
  return factory({ currentScript: { src: SCRIPT_SRC } }, fetchImpl);
}

test('classic <script>: resolves the engine relative to document.currentScript and reasons', async () => {
  const engine = await readFile(ENGINE);
  const requested = [];
  const eyereasoner = await loadIife(async (input) => {
    requested.push(String(input));
    return new Response(engine, { headers: { 'content-type': 'application/wasm' } });
  });

  assert.equal(typeof eyereasoner.n3reasoner, 'function');
  // Nothing is fetched until the first reasoning call — loading the script stays cheap.
  assert.deepEqual(requested, []);

  const result = await eyereasoner.n3reasoner(RULE_DATA);
  // The engine URL was derived from the script's own src, not from a hard-coded origin.
  assert.deepEqual(requested, [EXPECTED_WASM_URL]);
  assert.ok(result.includes(`${S}Mortal`), `expected the derived Mortal typing in:\n${result}`);
});

test('classic <script>: configureWasm overrides the currentScript-derived engine URL', async () => {
  const engine = await readFile(ENGINE);
  const eyereasoner = await loadIife(async () => {
    throw new Error('fetch must not be used once configureWasm() supplies the engine bytes');
  });

  eyereasoner.configureWasm(engine);
  const result = await eyereasoner.n3reasoner(RULE_DATA);
  assert.ok(result.includes(`${S}Mortal`), `expected the derived Mortal typing in:\n${result}`);
});

test('the IIFE global exposes the same named surface as the ESM entry', async () => {
  const esm = await import('../dist/index.js');
  const eyereasoner = await loadIife(async () => {
    throw new Error('this test never reaches the engine');
  });
  assert.deepEqual(Object.keys(eyereasoner).sort(), Object.keys(esm).sort());
});

test('the engine binary is NOT inlined into the script bundle', async () => {
  const [bundle, engine] = await Promise.all([readFile(ARTIFACT), readFile(ENGINE)]);
  // A base64/binary inline would make the JS strictly larger than the wasm it carries; keeping
  // it out is what preserves the lazy fetch + streaming compile the ESM entry already gets.
  assert.ok(
    bundle.length < engine.length,
    `bundle (${bundle.length} B) should be smaller than the engine (${engine.length} B)`,
  );
  assert.ok(!bundle.includes('AGFzbQ'), 'bundle contains base64-encoded wasm (the \\0asm magic)');
});
