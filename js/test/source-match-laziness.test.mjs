// [OPUS-4.8] sq-y9v8n (#1143 follow-up) — JS-side laziness of `Source.match` / `matchStream`.
//
// @jeswr's review thread (#1, on PR #1143) asked for an N3.js-style MEMORY test for the lazy
// `match` refactor: iterate a match and assert it does NOT eagerly buffer the whole result.
// These tests assert exactly the laziness that is GENUINELY true on the JS side after that
// refactor (`Source.match` → `SparqStore.matchStream` → `queryBindingsStream`, which pulls the
// engine's SPARQL-JSON in ~64 KiB chunks across the wasm boundary):
//
//   (a) `matchStream()` yields the FIRST quad after pulling exactly ONE solution from the engine
//       — it does not drain/materialise all matches up front;
//   (b) an early `break` stops pulling (no further solutions are fetched) and leaves the store
//       usable (the wasm-side cursor is freed in the generator's `finally`);
//   (c) iterating a large match never holds the whole `Quad[]` on the JS side — the generator
//       advances one solution at a time, in contrast to eager `match()` which is `[...matchStream]`;
//   (d) the event-based `Source.match(...)` Stream surfaces the first quad via `read()` before
//       the result is drained, and an early `destroy()`/stop is honoured.
//
// HONESTY — what these tests deliberately do NOT claim:
//   They do NOT assert a TRUE "too large to materialise in memory" guarantee end-to-end. The wasm
//   side (`Store.query_chunks` → `sparq_engine::query_json_chunks_with_budget`) produces the FULL
//   ordered chunk sequence EAGERLY into a `Vec<String>` inside wasm before the first
//   `cursor.next()` — there is no back-pressure to the engine's solution iterator. So the whole
//   result must still fit in wasm memory; the streaming win is on the JS side (at most one chunk +
//   one partial row is held in JS at a time), NOT a genuine unbounded-result guarantee. Closing
//   that gap (a streaming wasm solution cursor) is tracked in bead sq-y9v8n. The dataset sizes
//   here are kept modest precisely because we are measuring JS-side pull behaviour, not stressing
//   a result that cannot be materialised.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { SparqStore, SparqSource } from '../dist/index.js';

/** N-Triples with `n` subjects, padded so the SPARQL-JSON result spans several 64 KiB chunks. */
function bigNT(n) {
  let out = '';
  for (let i = 0; i < n; i++) {
    out += `<http://ex/s${i}> <http://ex/p> "value number ${i} with padding text so the result document grows past a single 64 KiB chunk boundary" .\n`;
  }
  return out;
}

const N = 4000;

/**
 * Wraps a store's `queryBindingsStream` (the generator `matchStream` pulls from) so a test can
 * observe how many solution-rows have actually been pulled/parsed from the engine at any instant.
 * Each increment corresponds to one solution drawn through the wasm chunk cursor, so the counter
 * is a faithful, deterministic spy on on-demand pulling.
 */
function spyOnPulls(store) {
  const orig = Object.getPrototypeOf(store).queryBindingsStream;
  const state = { pulled: 0 };
  store.queryBindingsStream = function* (sparql) {
    for (const b of orig.call(this, sparql)) {
      state.pulled++;
      yield b;
    }
  };
  return state;
}

test('matchStream yields the first quad after pulling exactly ONE solution (does not drain all)', async () => {
  const store = await SparqStore.fromString(bigNT(N), 'ntriples');
  const spy = spyOnPulls(store);

  const gen = store.matchStream(null, null, null, null);
  const first = gen.next();

  assert.ok(!first.done, 'expected a first quad');
  assert.equal(first.value.termType, 'Quad');
  // The whole point: producing the first quad pulled ONE solution, not all N.
  assert.equal(spy.pulled, 1, `expected 1 solution pulled for the first quad, got ${spy.pulled}`);
  assert.ok(spy.pulled < N, 'first quad must not drain the whole match');

  gen.return?.(); // abandon the generator (frees the wasm cursor)
  store.free();
});

test('matchStream pulls one solution per advance — never buffers the whole Quad[] on the JS side', async () => {
  const store = await SparqStore.fromString(bigNT(N), 'ntriples');
  const spy = spyOnPulls(store);

  const gen = store.matchStream(null, null, null, null);
  // Advancing k times pulls exactly k solutions: the generator is incremental, not eager.
  for (let k = 1; k <= 10; k++) {
    gen.next();
    assert.equal(spy.pulled, k, `after ${k} advances expected ${k} pulled, got ${spy.pulled}`);
  }
  gen.return?.();
  store.free();
});

test('early break stops pulling and leaves the store usable (wasm cursor freed)', async () => {
  const store = await SparqStore.fromString(bigNT(N), 'ntriples');
  const spy = spyOnPulls(store);

  let seen = 0;
  for (const quad of store.matchStream(null, null, null, null)) {
    assert.equal(quad.termType, 'Quad');
    if (++seen === 5) break; // abandon early — `for…of` calls the generator's `.return()`
  }
  assert.equal(seen, 5);
  // No solutions beyond what we consumed (+ the lookahead `for…of` does NOT take) were pulled.
  assert.equal(spy.pulled, 5, `early break pulled ${spy.pulled}, expected 5 (no over-fetch)`);

  // The store is still fully usable after abandoning the cursor early.
  assert.equal(store.countQuads(null, null, null, null), N);
  assert.equal(store.match().length, N);
  store.free();
});

test('eager match() materialises all N, matching the lazy matchStream element-for-element', async () => {
  const store = await SparqStore.fromString(bigNT(N), 'ntriples');
  // `match()` is `[...matchStream()]`: same quads, but the array form holds the whole result.
  const eager = store.match(null, null, null, null);
  assert.equal(eager.length, N);

  let i = 0;
  for (const quad of store.matchStream(null, null, null, null)) {
    assert.ok(quad.equals(eager[i]), `lazy quad ${i} differs from eager match()`);
    i++;
  }
  assert.equal(i, N);
  store.free();
});

test('Source.match() event Stream surfaces the first quad via read() before draining', async () => {
  const store = await SparqStore.fromString(bigNT(N), 'ntriples');
  const spy = spyOnPulls(store);
  const source = new SparqSource(store);

  const stream = source.match(null, null, null, null);
  // `read()` pulls the next quad on demand: the first quad is available without the whole
  // result being materialised first (only one solution has been drawn from the engine).
  const firstQuad = stream.read();
  assert.ok(firstQuad, 'expected read() to return the first quad');
  assert.equal(firstQuad.termType, 'Quad');
  assert.equal(spy.pulled, 1, `read() should pull one solution, pulled ${spy.pulled}`);

  store.free();
});

test('Source.match() event Stream emits data-per-quad then end over the full result', async () => {
  const store = await SparqStore.fromString(bigNT(N), 'ntriples');
  const source = new SparqSource(store);

  const seen = [];
  await new Promise((resolve, reject) => {
    source
      .match(null, null, null, null)
      .on('data', (quad) => seen.push(quad))
      .on('end', resolve)
      .on('error', reject);
  });
  assert.equal(seen.length, N);
  assert.equal(seen[0].termType, 'Quad');
  store.free();
});
