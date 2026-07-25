// [SONNET-4.6] sq-250si: trap-recovery unit tests for the Node host's instance-recycle path.
//
// These tests exercise the real `attachTrapRecoveryHandler` and `isWasmTrap` logic in
// src/trap-recovery.js using a stubbed pod factory — no wasm binary is needed. The module is
// intentionally imported from trap-recovery.js (not index.js) to avoid the wasm binary import
// in index.js that would fail in CI before the wasm artifact is built.
//
// Non-vacuity: the "next request succeeds" assertion FAILS when the recycle assignment
// (`state.pod = makePod()`) is removed from attachTrapRecoveryHandler — verified manually by
// reverting the assignment and observing that both the trap request and the subsequent request
// return 503, not 503 then 200. Without recovery the server stays permanently broken.
import assert from 'node:assert/strict';
import test from 'node:test';

import { attachTrapRecoveryHandler, isWasmTrap } from '../src/trap-recovery.js';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function listenerUrl(server) {
  const address = server.address();
  assert.ok(address && typeof address !== 'string', 'server must be bound');
  return `http://127.0.0.1:${address.port}`;
}

async function startOn(server, port = 0) {
  await new Promise((resolve, reject) => {
    const onError = (error) => { server.off('listening', onListening); reject(error); };
    const onListening = () => { server.off('error', onError); resolve(); };
    server.once('error', onError);
    server.once('listening', onListening);
    server.listen(port, '127.0.0.1');
  });
}

async function stopServer(server) {
  await new Promise((resolve, reject) => {
    server.close((error) => (error ? reject(error) : resolve()));
  });
}

/**
 * Build a fake pod whose `handleRequest` delegates to the provided function.
 * A `free()` call is recorded for later assertion.
 */
function fakePod(handleFn) {
  return {
    freed: false,
    handleRequest: handleFn,
    free() { this.freed = true; },
  };
}

// ---------------------------------------------------------------------------
// isWasmTrap
// ---------------------------------------------------------------------------

test('isWasmTrap returns false for a generic Error', () => {
  assert.equal(isWasmTrap(new Error('boom')), false);
});

test('isWasmTrap returns false for null', () => {
  assert.equal(isWasmTrap(null), false);
});

test('isWasmTrap returns true for a WebAssembly.RuntimeError', () => {
  // [SONNET-4.6] sq-250si: WebAssembly.RuntimeError is available in Node >= 12.
  const trap = new WebAssembly.RuntimeError('unreachable');
  assert.equal(isWasmTrap(trap), true);
});

// ---------------------------------------------------------------------------
// Trap recovery: instance recycle
// ---------------------------------------------------------------------------

test(
  'a wasm trap on one request is recovered: the next request succeeds via a fresh instance',
  { timeout: 10_000 },
  async () => {
    // [SONNET-4.6] sq-250si: arrange — first pod throws a trap; the factory then supplies a fresh
    // pod that returns HTTP 200. This verifies the recycle path in attachTrapRecoveryHandler.
    let callCount = 0;
    const pods = [];

    // Pod 1: throws a wasm trap on its first (and only) call.
    const pod1 = fakePod(async () => {
      throw new WebAssembly.RuntimeError('unreachable');
    });
    pods.push(pod1);

    // Pod 2: the recycled instance — returns a minimal 200 OK.
    const pod2 = fakePod(async (_method, _path) => {
      return {
        status: 200,
        headers: ['content-type', 'text/plain; charset=utf-8'],
        body: new Uint8Array(Buffer.from('ok\n')),
        free() {},
      };
    });
    pods.push(pod2);

    let podIdx = 0;
    const makePod = () => {
      callCount++;
      return pods[podIdx++];
    };

    const { server } = attachTrapRecoveryHandler(makePod, {
      authenticate: async () => 'https://id.example/alice#me',
    });

    await startOn(server);
    const url = listenerUrl(server);
    try {
      // First request — triggers the trap path; must get 503 (not 500 from generic error).
      const trap = await fetch(`${url}/trigger-trap`, { method: 'GET' });
      assert.equal(trap.status, 503, 'a wasm trap responds with 503');

      // The old pod must have been freed during recycle.
      assert.equal(pod1.freed, true, 'the poisoned pod is freed on trap');

      // The factory must have been called twice: once at startup, once for the recycle.
      assert.equal(callCount, 2, 'makePod is called again to produce the replacement instance');

      // Second request — served by the fresh instance; must succeed (not 500/503-forever).
      const ok = await fetch(`${url}/any-path`, { method: 'GET' });
      assert.equal(ok.status, 200, 'next request after trap recycle succeeds');
    } finally {
      await stopServer(server);
    }
  },
);

test(
  'a non-trap error does NOT recycle the instance (plain 500 path)',
  { timeout: 10_000 },
  async () => {
    // [SONNET-4.6] sq-250si: a generic Error must not trigger the recycle path.
    let makeCount = 0;
    let handleCount = 0;

    const pod = fakePod(async () => {
      handleCount++;
      throw new Error('internal logic error');
    });

    const { server } = attachTrapRecoveryHandler(
      () => { makeCount++; return pod; },
      { authenticate: async () => undefined },
    );

    await startOn(server);
    const url = listenerUrl(server);
    try {
      const r = await fetch(`${url}/any`, { method: 'GET' });
      assert.equal(r.status, 500, 'a non-trap error returns 500');
      assert.equal(makeCount, 1, 'the pod factory is NOT called again on a non-trap error');
      assert.equal(pod.freed, false, 'the pod is NOT freed on a non-trap error');
      assert.equal(handleCount, 1, 'handleRequest was called exactly once');
    } finally {
      await stopServer(server);
    }
  },
);

test(
  'server close frees the current pod even after a recycle',
  { timeout: 10_000 },
  async () => {
    // [SONNET-4.6] sq-250si: after a trap, the replacement pod is freed on server.close().
    const pods = [];

    const pod1 = fakePod(async () => {
      throw new WebAssembly.RuntimeError('unreachable');
    });
    pods.push(pod1);

    const pod2 = fakePod(async () => ({
      status: 200,
      headers: [],
      body: new Uint8Array(0),
      free() {},
    }));
    pods.push(pod2);

    let podIdx = 0;
    const { server } = attachTrapRecoveryHandler(
      () => pods[podIdx++],
      { authenticate: async () => undefined },
    );

    await startOn(server);
    const url = listenerUrl(server);

    // Induce a trap so the server holds pod2 as the live instance.
    await fetch(`${url}/trap`, { method: 'GET' });
    assert.equal(pod1.freed, true, 'pod1 freed during recycle');
    assert.equal(pod2.freed, false, 'pod2 not yet freed');

    // Close the server — the freePod hook must free the current pod (pod2).
    await stopServer(server);
    assert.equal(pod2.freed, true, 'pod2 freed when server closes');
  },
);
