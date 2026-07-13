// [GPT-5.6] sq-6xasp.9: mutation-sensitive tests for the Node/wasm HTTP seam.
import assert from 'node:assert/strict';
import test from 'node:test';

import { parseArgs } from '../src/cli.js';
import {
  copyWasmResponse,
  flattenRequestHeaders,
  writeNodeResponse,
} from '../src/http.js';

test('request headers remain flat, ordered, and repeated', () => {
  const raw = ['Host', 'pod.example', 'X-Probe', 'one', 'X-Probe', 'two'];
  const flat = flattenRequestHeaders(raw);
  assert.deepEqual(flat, raw);
  assert.notStrictEqual(flat, raw);
});

test('response reconstruction preserves status, repeated headers, and bytes', () => {
  let freed = false;
  const copied = copyWasmResponse({
    status: 207,
    headers: ['link', '<one>; rel="item"', 'link', '<two>; rel="item"'],
    body: new Uint8Array([0, 1, 2, 255]),
    free() {
      freed = true;
    },
  });
  assert.equal(freed, true);

  const appended = [];
  const nodeResponse = {
    statusCode: undefined,
    appendHeader(name, value) {
      appended.push([name, value]);
    },
    end(body) {
      this.body = body;
    },
  };
  writeNodeResponse(nodeResponse, copied);

  assert.equal(nodeResponse.statusCode, 207);
  assert.deepEqual(appended, [
    ['link', '<one>; rel="item"'],
    ['link', '<two>; rel="item"'],
  ]);
  assert.deepEqual([...nodeResponse.body], [0, 1, 2, 255]);
});

test('the bin parses the documented argv surface', () => {
  assert.deepEqual(
    parseArgs(
      ['--port', '4040', '--base-url', 'https://pod.example', '--owner-webid', 'https://id.example/alice#me'],
      {},
    ),
    {
      help: false,
      options: {
        port: 4040,
        baseUrl: 'https://pod.example',
        ownerWebid: 'https://id.example/alice#me',
      },
    },
  );
  assert.deepEqual(parseArgs(['--help'], {}), { help: true });
  assert.throws(() => parseArgs(['--unknown', 'value'], {}), /unknown option/);
});
