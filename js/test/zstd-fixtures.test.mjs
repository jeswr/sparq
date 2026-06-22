import assert from 'node:assert/strict';
import { test } from 'node:test';
import { decompress as fzstdDecompress } from 'fzstd';
// Unit-tests the pure-JS zstd fixture synthesiser directly (no wasm/dist build
// needed): the fixtures themselves must survive payloads at and beyond the
// 128 KiB Raw_Block boundary. Regression for bead sq-3lh7, where a >=128 KiB
// chunk was spread as `push(...chunk)` args and overflowed the call stack
// (RangeError) before any frame was produced. [OPUS-4.8]
import { zstdRawFrame, zstdMultiFrame } from './helpers/zstd-fixtures.mjs';

const MAX_RAW_BLOCK = 128 * 1024;

function ramp(n) {
  const a = new Uint8Array(n);
  for (let i = 0; i < n; i++) a[i] = (i * 31 + 7) & 0xff;
  return a;
}

// Sizes straddling the single-block boundary: just under, exactly at, just
// over, several blocks, and a size whose final block is a remainder.
for (const n of [
  MAX_RAW_BLOCK - 1,
  MAX_RAW_BLOCK,
  MAX_RAW_BLOCK + 1,
  3 * MAX_RAW_BLOCK,
  3 * MAX_RAW_BLOCK + 123,
]) {
  test(`zstdRawFrame round-trips a ${n}-byte payload through fzstd`, () => {
    const payload = ramp(n);
    const frame = zstdRawFrame(payload);
    const out = fzstdDecompress(frame);
    assert.deepEqual(Uint8Array.from(out), payload);
  });
}

test('zstdRawFrame round-trips an empty payload (single empty Raw_Block)', () => {
  const out = fzstdDecompress(zstdRawFrame(new Uint8Array(0)));
  assert.equal(out.length, 0);
});

test('zstdRawFrame embeds a dictId across the block boundary', () => {
  const payload = ramp(MAX_RAW_BLOCK + 9);
  const out = fzstdDecompress(zstdRawFrame(payload, { dictId: 0x1234 }));
  assert.deepEqual(Uint8Array.from(out), payload);
});

test('zstdMultiFrame concatenates large independent frames', () => {
  const parts = [ramp(MAX_RAW_BLOCK + 5), ramp(10), ramp(2 * MAX_RAW_BLOCK)];
  const frame = zstdMultiFrame(parts);
  const expected = Uint8Array.from(parts.flatMap(p => [...p]));
  const out = fzstdDecompress(frame);
  assert.deepEqual(Uint8Array.from(out), expected);
});
