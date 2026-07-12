#!/usr/bin/env node
// [FABLE-5] sq-hmd7l.12 — Comunica column runner for the federation panel.
//
// Reads ONE SPARQL query on stdin, executes it with @comunica/query-sparql (the
// reference federated-SPARQL JS engine, gather-time `npm install` into
// bench/federation/node_modules — NEVER a committed dependency), and prints a
// single JSON document on stdout:
//
//   { ok, exec_ms, engine_version, bindings: [ { var: {type,value,datatype?,"xml:lang"?} } ] }
//
// bindings mirror the SPARQL 1.1 Query Results JSON term shape, so bench/
// federation/compare.py canonicalises sparq and Comunica rows through the SAME
// code path (the result-set-agreement oracle).
//
// Sources:
//   --source=<url>   repeatable; each becomes {type:'sparql', value:url} — the
//                    VIRTUAL federation regime (Comunica does source selection).
//                    The explicit type skips Comunica's format-probe requests, so
//                    the proxy request counts measure query execution, not
//                    detection.
//   (none)           an empty in-memory RDF/JS store is the sole source — the
//                    EXPLICIT regime, where the query names members via
//                    SERVICE <url> clauses and the outer join happens client-side.
//
// exec_ms measures queryBindings() start -> stream end (engine-internal
// execution; Node process startup is EXCLUDED — compare.py records this timing
// regime in the results JSON).
import { QueryEngine } from '@comunica/query-sparql';
import { Store } from 'n3';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);

function termToJson(term) {
  switch (term.termType) {
    case 'NamedNode':
      return { type: 'uri', value: term.value };
    case 'BlankNode':
      return { type: 'bnode', value: term.value };
    case 'Literal': {
      const out = { type: 'literal', value: term.value };
      if (term.language) out['xml:lang'] = term.language;
      else if (term.datatype && term.datatype.value !== 'http://www.w3.org/2001/XMLSchema#string')
        out.datatype = term.datatype.value;
      return out;
    }
    default:
      throw new Error(`unsupported term type in results: ${term.termType}`);
  }
}

async function main() {
  const sources = [];
  for (const arg of process.argv.slice(2)) {
    if (arg.startsWith('--source=')) sources.push({ type: 'sparql', value: arg.slice('--source='.length) });
    else throw new Error(`unknown arg: ${arg}`);
  }
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const query = Buffer.concat(chunks).toString('utf8');
  if (!query.trim()) throw new Error('empty query on stdin');

  const engine = new QueryEngine();
  const context = { sources: sources.length ? sources : [new Store()] };

  const t0 = process.hrtime.bigint();
  const stream = await engine.queryBindings(query, context);
  const bindings = [];
  for await (const b of stream) {
    const row = {};
    for (const [variable, term] of b) row[variable.value] = termToJson(term);
    bindings.push(row);
  }
  const execMs = Number(process.hrtime.bigint() - t0) / 1e6;

  let version = 'unknown';
  try {
    version = require('@comunica/query-sparql/package.json').version;
  } catch {
    /* version stays 'unknown' — compare.py records it as such */
  }
  process.stdout.write(
    JSON.stringify({ ok: true, exec_ms: execMs, engine_version: version, bindings }) + '\n',
  );
}

main().catch((e) => {
  process.stderr.write(`comunica_runner: ${e && e.stack ? e.stack : e}\n`);
  process.exit(1);
});
