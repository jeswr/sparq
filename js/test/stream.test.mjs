import assert from 'node:assert/strict';
import { test } from 'node:test';
import { SparqlJsonRowsParser, SparqStore } from '../dist/index.js';

/** N-Triples with `n` subjects × 3 predicates — big enough for several 64 KiB chunks. */
function bigNT(n) {
  let out = '';
  for (let i = 0; i < n; i++) {
    out += `<http://ex/s${i}> <http://ex/name> "subject number ${i} with some padding text" .\n`;
    out += `<http://ex/s${i}> <http://ex/index> "${i}"^^<http://www.w3.org/2001/XMLSchema#integer> .\n`;
    if (i % 2 === 0) out += `<http://ex/s${i}> <http://ex/even> "true"^^<http://www.w3.org/2001/XMLSchema#boolean> .\n`;
  }
  return out;
}

const N = 4000;
const load = () => SparqStore.fromString(bigNT(N), 'ntriples');

function collectResultStream(stream) {
  return new Promise((resolve, reject) => {
    const rows = [];
    stream.on('data', (row) => rows.push(row));
    stream.on('error', reject);
    stream.on('end', () => resolve(rows));
  });
}

test('queryJsonChunks concatenates byte-identically to queryJson, in >1 chunk', async () => {
  const store = await load();
  const sparql = 'SELECT ?s ?p ?o WHERE { ?s ?p ?o }';
  const chunks = [...store.queryJsonChunks(sparql)];
  assert.ok(chunks.length > 1, `expected several chunks, got ${chunks.length}`);
  assert.ok(chunks.slice(0, -1).every(c => c.length >= 64 * 1024), 'non-final chunks flush at ~64 KiB');
  assert.equal(chunks.join(''), store.queryJson(sparql));
});

test('queryBindingsStream yields repeatable cursor results', async () => {
  const store = await load();
  const sparql = 'SELECT ?s ?name ?i WHERE { ?s <http://ex/name> ?name ; <http://ex/index> ?i } ORDER BY ?s';
  const materialised = [...store.queryBindingsStream(sparql)];
  assert.equal(materialised.length, N);

  let i = 0;
  for (const row of store.queryBindingsStream(sparql)) {
    assert.ok(row.equals(materialised[i]), `row ${i} differs`);
    i++;
  }
  assert.equal(i, N);
});

test('queryBindings returns an RDF/JS ResultStream with the same bindings as the cursor', async () => {
  const store = await load();
  const sparql =
    'SELECT ?s ?name ?i WHERE { ?s <http://ex/name> ?name ; <http://ex/index> ?i } ORDER BY ?s LIMIT 50';
  const cursorRows = [...store.queryBindingsStream(sparql)];

  const streamPromise = store.queryBindings(sparql);
  assert.ok(streamPromise instanceof Promise);
  const stream = await streamPromise;
  assert.equal(typeof stream.read, 'function');

  const eventRows = await collectResultStream(stream);
  assert.equal(eventRows.length, cursorRows.length);
  for (let i = 0; i < cursorRows.length; i++) {
    assert.ok(eventRows[i].equals(cursorRows[i]), `row ${i} differs`);
  }
  store.free();
});

test('queryBindings ResultStream pauses without over-pulling and ends exactly once', async () => {
  const store = await load();
  const stream = await store.queryBindings(
    'SELECT ?s WHERE { ?s <http://ex/index> ?i } ORDER BY ?s LIMIT 6',
  );
  const rows = [];
  let endCount = 0;

  await new Promise((resolve, reject) => {
    stream.on('error', reject);
    stream.on('end', () => {
      endCount++;
      resolve();
    });
    stream.on('data', (row) => {
      rows.push(row);
      if (rows.length !== 1) return;
      stream.pause();
      setTimeout(() => {
        try {
          assert.equal(rows.length, 1, 'cursor advanced while the ResultStream was paused');
          stream.resume();
        } catch (error) {
          reject(error);
        }
      }, 0);
    });
  });

  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(rows.length, 6);
  assert.equal(endCount, 1);
  store.free();
});

test('queryBindings ResultStream propagates cursor errors without emitting end', async () => {
  const store = await load();
  const stream = await store.queryBindings('SELECT ?s WHERE { broken');
  let ended = false;

  const error = await new Promise((resolve, reject) => {
    stream.on('end', () => {
      ended = true;
      reject(new Error('invalid query emitted end instead of error'));
    });
    stream.on('error', resolve);
    stream.on('data', () => reject(new Error('invalid query emitted data')));
  });

  await Promise.resolve();
  assert.ok(error instanceof Error);
  assert.match(error.message, /error|expected/i);
  assert.equal(ended, false);
  store.free();
});

test('queryBindings ResultStream follows EventEmitter error and duplicate-listener semantics', async () => {
  const store = await load();
  const stream = await store.queryBindings('SELECT ?s WHERE { ?s ?p ?o } LIMIT 1');
  const expected = new Error('unhandled');
  assert.throws(() => stream.emit('error', expected), (error) => error === expected);

  let calls = 0;
  const listener = () => calls++;
  stream.on('custom', listener).on('custom', listener);
  assert.equal(stream.listenerCount('custom'), 2);
  assert.equal(stream.emit('custom'), true);
  assert.equal(calls, 2);
  stream.removeListener('custom', listener);
  assert.equal(stream.listenerCount('custom'), 1);
  stream.removeAllListeners('custom');
  assert.deepEqual(stream.eventNames(), []);
  stream.destroy();
  store.free();
});

test('queryBindings ResultStream destroy() closes an abandoned wasm cursor', async () => {
  const store = await load();
  const original = store.queryBindingsStream.bind(store);
  let cursorClosed = false;
  store.queryBindingsStream = function* (sparql) {
    try {
      yield* original(sparql);
    } finally {
      cursorClosed = true;
    }
  };

  const stream = await store.queryBindings('SELECT ?s ?p ?o WHERE { ?s ?p ?o }');
  let seen = 0;
  await new Promise((resolve, reject) => {
    stream.on('error', reject);
    stream.on('end', resolve);
    stream.on('data', () => {
      seen++;
      stream.pause();
      stream.destroy();
    });
  });

  assert.equal(seen, 1);
  assert.equal(cursorClosed, true);
  assert.equal(store.count('SELECT ?s WHERE { ?s <http://ex/index> ?o }'), N);
  store.free();
});

test('queryBindings accepts only the supported default RDF/JS query context', async () => {
  const store = await load();
  const stream = await store.queryBindings('SELECT ?s WHERE { ?s ?p ?o } LIMIT 1', {
    queryFormat: { language: 'sparql', version: '1.1' },
  });
  stream.destroy();

  await assert.rejects(
    store.queryBindings('SELECT ?s WHERE { ?s ?p ?o }', { baseIRI: 'http://ex/' }),
    /not supported/,
  );
  await assert.rejects(
    store.queryBindings('SELECT ?s WHERE { ?s ?p ?o }', {
      queryFormat: { language: 'sparql', version: '1.2' },
    }),
    /SPARQL 1\.1/,
  );
  store.free();
});

test('queryBindings ResultStream read() pulls rows and signals empty/end once', async () => {
  const store = await load();
  const sparql = 'SELECT ?s WHERE { ?s <http://ex/index> ?i } ORDER BY ?s LIMIT 3';
  const cursorRows = [...store.queryBindingsStream(sparql)];
  const stream = await store.queryBindings(sparql);
  let endCount = 0;
  const ended = new Promise((resolve, reject) => {
    stream.on('error', reject);
    stream.on('end', () => {
      endCount++;
      resolve();
    });
  });

  const pulled = [];
  for (let row = stream.read(); row !== null; row = stream.read()) pulled.push(row);
  await ended;

  assert.equal(pulled.length, cursorRows.length);
  assert.ok(pulled.every((row, i) => row.equals(cursorRows[i])));
  assert.equal(stream.read(), null);
  assert.equal(endCount, 1);

  const empty = await store.queryBindings('SELECT ?s WHERE { ?s <http://ex/missing> ?o }');
  let emptyData = 0;
  let emptyEnds = 0;
  await new Promise((resolve, reject) => {
    empty.on('error', reject);
    empty.on('end', () => {
      emptyEnds++;
      resolve();
    });
    empty.on('data', () => emptyData++);
  });
  assert.equal(emptyData, 0);
  assert.equal(emptyEnds, 1);
  store.free();
});

test('queryBindingsStream works with for await…of and OPTIONAL rows', async () => {
  const store = await load();
  const sparql =
    'SELECT ?s ?even WHERE { ?s <http://ex/name> ?n OPTIONAL { ?s <http://ex/even> ?even } }';
  let total = 0;
  let bound = 0;
  for await (const row of store.queryBindingsStream(sparql)) {
    total++;
    if (row.has('even')) bound++;
  }
  assert.equal(total, N);
  assert.equal(bound, N / 2);
});

test('queryBindingsStream: empty results, early break, ASK rejection', async () => {
  const store = await load();
  assert.deepEqual([...store.queryBindingsStream('SELECT ?s WHERE { ?s <http://ex/nope> ?o }')], []);

  // Early break abandons (and frees) the wasm cursor; the store stays usable.
  let seen = 0;
  for (const _ of store.queryBindingsStream('SELECT ?s ?p ?o WHERE { ?s ?p ?o }')) {
    if (++seen === 3) break;
  }
  assert.equal(seen, 3);
  assert.equal(store.count('SELECT ?s WHERE { ?s <http://ex/index> ?o }'), N);

  assert.throws(() => [...store.queryBindingsStream('ASK { ?s ?p ?o }')], /SELECT/);
});

test('SparqlJsonRowsParser is chunk-boundary-agnostic (split mid-row, mid-head)', () => {
  const doc =
    '{"head":{"vars":["s","o"]},"results":{"bindings":[' +
    '{"s":{"type":"uri","value":"http://ex/a"},"o":{"type":"literal","value":"has \\"quotes\\" and {braces}"}},' +
    '{"s":{"type":"bnode","value":"b0"},"o":{"type":"literal","value":"x","xml:lang":"en"}}]}}';
  for (const splitEvery of [1, 3, 7, 50, doc.length]) {
    const parser = new SparqlJsonRowsParser();
    const rows = [];
    for (let i = 0; i < doc.length; i += splitEvery) rows.push(...parser.push(doc.slice(i, i + splitEvery)));
    assert.equal(rows.length, 2, `split=${splitEvery}`);
    assert.equal(rows[0].o.value, 'has "quotes" and {braces}');
    assert.equal(rows[1].s.type, 'bnode');
    assert.equal(parser.boolean, undefined);
  }
  // ASK boolean form sets .boolean and yields no rows
  const ask = new SparqlJsonRowsParser();
  assert.deepEqual(ask.push('{"head":{},"bool'), []);
  assert.deepEqual(ask.push('ean":true}'), []);
  assert.equal(ask.boolean, true);
});
