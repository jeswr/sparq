// [FABLE-5] sq-i858h — browser-SHACL same-runtime comparison harness:
// sparq-shacl-wasm (the wasm-pack'd `Validator`, crates/sparq-shacl-wasm) vs
// Zazuko rdf-validate-shacl (bench/competitors.json id `rdf-validate-shacl`,
// kind js-lib), BOTH in THIS one Node process — the natural runtime for
// sparq's browser SHACL story.
//
// REDUCTION CONTRACT (identical for both engines, mirroring the shared
// scripts/bench-adapters/js_shacl_adapter.mjs + shacl_report_count.py
// semantics): violations = report.results.length, conforms = report.conforms.
// Both engines expose exactly the W3C report model's sh:result set +
// sh:conforms — sparq via `JSON.parse(Validator.validate(...))`,
// rdf-validate-shacl via its ValidationReport object.
//
// GATE (the bead invariant): NO timing row for a workload unless that
// workload's #violations AND conforms AGREE across both engines AND match the
// hand-derived constants in expected.tsv (non-vacuity: both-engines-broken
// cannot pass as "agreement"). Workloads whose shapes use sh:sparql (SHACL
// §5.2) are sparq-only — rdf-validate-shacl implements SHACL Core only, so
// its column is ABSENT there (absent capability => absent column, never a
// fabricated 0); sparq is still self-asserted vs expected.tsv.
//
// TIMING (ADVISORY, canonical:false off a quiet box): one-shot END-TO-END,
// best-of-N — parse data + shapes + validate + reduce, per iteration, for
// BOTH engines (the sparq Validator is stateless, so parse cannot be hoisted
// out; the peer is charged the same work for apples-to-apples). The peer's
// validate-only time on pre-parsed datasets is recorded as an extra advisory
// column (`peer_validate_only_us`), since RDF/JS apps typically hold a parsed
// dataset. [FABLE-5] sq-01xlp: when the artifact is built with the opt-in
// `stateful` feature (run.sh FEATURES=stateful), the exported `ParsedGraph`
// handle yields the SYMMETRIC sparq column (`sparq_validate_only_us`) —
// validate-only on pre-parsed graphs, counts cross-checked against the
// one-shot per iteration. Absent feature => absent column, never a 0.
//
// stdout : `<workload>\t<engine>\t<violations>\t<e2e_best_us>` rows (gated).
// --out  : one canonical-competitor-results-shaped JSON envelope.
// exit   : non-zero on any gate failure or engine error.

import { readFileSync, writeFileSync, readdirSync, statSync } from 'node:fs';
import { performance } from 'node:perf_hooks';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { gzipSync } from 'node:zlib';
import { basename, join } from 'node:path';
import os from 'node:os';

function parseArgs(argv) {
  const a = { iters: 3, canonical: false, shapeDirs: [] };
  for (let i = 0; i < argv.length; i++) {
    const k = argv[i];
    if (k === '--pkg') a.pkg = argv[++i];
    else if (k === '--modules-dir') a.modulesDir = argv[++i];
    else if (k === '--data') a.data = argv[++i];
    else if (k === '--shapes-dir') a.shapeDirs.push(argv[++i]);
    else if (k === '--expected') a.expected = argv[++i];
    else if (k === '--iters') a.iters = parseInt(argv[++i], 10);
    else if (k === '--out') a.out = argv[++i];
    else if (k === '--canonical') a.canonical = argv[++i] === '1';
    else if (k === '--git-commit') a.gitCommit = argv[++i];
    else if (k === '--sparq-features') a.sparqFeatures = argv[++i];
    else if (k === '--peer-bundle') a.peerBundle = argv[++i];
    else if (k === '--allow-missing-peer') a.allowMissingPeer = true;
    else { process.stderr.write(`harness: unknown arg ${k}\n`); process.exit(2); }
  }
  if (!a.pkg || !a.data || !a.shapeDirs.length || !a.expected) {
    process.stderr.write(
      'usage: node harness.mjs --pkg <wasm-pack out dir> --data abox.ttl ' +
      '--shapes-dir DIR [--shapes-dir DIR2] --expected expected.tsv ' +
      '[--modules-dir DIR] [--iters N] [--out envelope.json] [--canonical 0|1] ' +
      '[--git-commit SHA] [--sparq-features CSV] [--peer-bundle bundle.json] ' +
      '[--allow-missing-peer]\n');
    process.exit(2);
  }
  return a;
}

// ---- expected.tsv: workload \t conforms(0|1) \t violations \t engines ------
function readExpected(path) {
  const rows = new Map();
  for (const line of readFileSync(path, 'utf-8').split('\n')) {
    if (!line || line.startsWith('#')) continue;
    const [name, conforms, violations, engines] = line.split('\t');
    rows.set(name, {
      conforms: conforms === '1',
      violations: parseInt(violations, 10),
      engines, // 'both' | 'sparq-only'
    });
  }
  return rows;
}

// ---- peer stack (adapted from scripts/bench-adapters/js_shacl_adapter.mjs —
// that adapter is a standalone CLI, so the small factory/parser builders are
// re-derived here with the SAME package set; the REDUCTION is byte-identical).
function makeImporter(modulesDir) {
  const base = modulesDir
    ? pathToFileURL(modulesDir.replace(/\/?$/, '/') + 'package.json')
    : pathToFileURL(process.cwd() + '/package.json');
  return createRequire(base);
}

function makeFactory(req) {
  const Environment = req('@rdfjs/environment').default || req('@rdfjs/environment');
  const def = (id, sub) => {
    const m = req(sub ? `${id}/${sub}` : id);
    return m.default || m;
  };
  return new Environment([
    def('@rdfjs/data-model', 'Factory.js'),
    def('@rdfjs/dataset', 'Factory.js'),
    def('@rdfjs/term-map', 'Factory.js'),
    def('@rdfjs/namespace', 'Factory.js'),
    def('clownface', 'Factory.js'),
  ]);
}

async function parseTurtleToDataset(req, factory, text, baseIRI) {
  const ParserN3 = req('@rdfjs/parser-n3').default || req('@rdfjs/parser-n3');
  const parser = new ParserN3({ factory, baseIRI });
  const { Readable } = await import('node:stream');
  const stream = parser.import(Readable.from([text]));
  const ds = factory.dataset();
  for await (const quad of stream) ds.add(quad);
  return ds;
}

// ---- the shared reduction, one per engine ----------------------------------
function reduceSparqReport(jsonText) {
  const report = JSON.parse(jsonText);
  return { conforms: report.conforms, violations: report.results.length };
}
function reducePeerReport(report) {
  return { conforms: report.conforms, violations: report.results.length };
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const iters = Math.max(1, args.iters);

  // sparq-shacl-wasm: the wasm-pack --target nodejs artifact (CJS entry).
  // `ParsedGraph` is exported only when the artifact was built with the opt-in
  // `stateful` feature — its presence switches on the sparq validate-only column.
  const req = createRequire(import.meta.url);
  const pkgEntry = join(args.pkg, 'sparq_shacl_wasm.js');
  const { Validator, ParsedGraph } = req(pkgEntry);

  // peer: rdf-validate-shacl from the gather-only modules dir.
  let peer = null;
  let peerVersion = null;
  try {
    const preq = makeImporter(args.modulesDir);
    const SHACLValidator = preq('rdf-validate-shacl').default || preq('rdf-validate-shacl');
    const factory = makeFactory(preq);
    peerVersion = JSON.parse(readFileSync(
      preq.resolve('rdf-validate-shacl/package.json'), 'utf-8')).version;
    peer = { preq, SHACLValidator, factory };
  } catch (e) {
    if (!args.allowMissingPeer) {
      process.stderr.write(
        `harness: rdf-validate-shacl stack unavailable (${e}).\n` +
        'harness: install the gather-only deps (see run.sh) or pass --allow-missing-peer\n' +
        'harness: to degrade to a sparq-only self-assert (peer column ABSENT).\n');
      process.exit(1);
    }
    process.stderr.write(`harness: NOTICE peer absent (${e}); peer column ABSENT.\n`);
  }

  const expected = readExpected(args.expected);
  const dataText = readFileSync(args.data, 'utf-8');

  // workloads = every *.ttl across the shape dirs; a shape using sh:sparql is
  // sparq-only (rdf-validate-shacl is SHACL Core only — capability-absent).
  const workloads = [];
  for (const dir of args.shapeDirs) {
    for (const f of readdirSync(dir).filter((f) => f.endsWith('.ttl')).sort()) {
      const text = readFileSync(join(dir, f), 'utf-8');
      workloads.push({
        name: basename(f, '.ttl'),
        text,
        sparqlConstraint: /\bsh:sparql\b/.test(text),
      });
    }
  }

  const failures = [];
  const rows = {}; // per-workload envelope rows

  // [FABLE-5] sq-01xlp: pre-parse the data graph ONCE for the stateful column
  // (outside every timed section — exactly how the peer's validate-only
  // pre-parsed datasets are treated).
  const dataParsed = ParsedGraph ? ParsedGraph.parse(dataText, 'turtle') : null;

  for (const wl of workloads) {
    const exp = expected.get(wl.name);
    if (!exp) { failures.push(`${wl.name}: no expected.tsv entry`); continue; }
    const peerComparable = exp.engines === 'both' && !wl.sparqlConstraint;
    if ((exp.engines === 'both') !== !wl.sparqlConstraint) {
      failures.push(`${wl.name}: expected.tsv engines='${exp.engines}' contradicts ` +
        `sh:sparql detection (${wl.sparqlConstraint})`);
      continue;
    }

    // -- sparq-shacl-wasm: one-shot end-to-end, best-of-N.
    let sparq = null, sparqBestUs = Infinity;
    try {
      for (let i = 0; i < iters; i++) {
        const t0 = performance.now();
        sparq = reduceSparqReport(Validator.validate(dataText, wl.text, 'turtle'));
        sparqBestUs = Math.min(sparqBestUs, (performance.now() - t0) * 1000);
      }
    } catch (e) {
      failures.push(`${wl.name}: sparq-shacl-wasm ERROR: ${e}`);
      continue;
    }

    // -- sparq stateful (opt-in `stateful`-feature artifact): validate-only on
    //    pre-parsed graphs, best-of-N, counts cross-checked per iteration.
    let sparqValidateOnlyUs = Infinity;
    if (dataParsed) {
      try {
        const shapesParsed = ParsedGraph.parse(wl.text, 'turtle');
        for (let i = 0; i < iters; i++) {
          const t0 = performance.now();
          const r = reduceSparqReport(dataParsed.validate(shapesParsed));
          sparqValidateOnlyUs = Math.min(sparqValidateOnlyUs, (performance.now() - t0) * 1000);
          if (r.violations !== sparq.violations || r.conforms !== sparq.conforms) {
            throw new Error('stateful validate-only drift vs one-shot');
          }
        }
        shapesParsed.free();
      } catch (e) {
        failures.push(`${wl.name}: sparq-shacl-wasm (stateful) ERROR: ${e}`);
        continue;
      }
    }

    // -- rdf-validate-shacl: same one-shot end-to-end, best-of-N (+ a
    //    validate-only advisory column on pre-parsed datasets).
    let peerRes = null, peerBestUs = Infinity, peerValidateOnlyUs = Infinity;
    if (peerComparable && peer) {
      try {
        for (let i = 0; i < iters; i++) {
          const t0 = performance.now();
          const dataDs = await parseTurtleToDataset(
            peer.preq, peer.factory, dataText, pathToFileURL(args.data).href);
          const shapesDs = await parseTurtleToDataset(
            peer.preq, peer.factory, wl.text, 'http://example.org/shapes');
          const validator = new peer.SHACLValidator(shapesDs, { factory: peer.factory });
          peerRes = reducePeerReport(await validator.validate(dataDs));
          peerBestUs = Math.min(peerBestUs, (performance.now() - t0) * 1000);
        }
        // validate-only advisory (datasets pre-parsed OUTSIDE the timed section).
        const dataDs = await parseTurtleToDataset(
          peer.preq, peer.factory, dataText, pathToFileURL(args.data).href);
        const shapesDs = await parseTurtleToDataset(
          peer.preq, peer.factory, wl.text, 'http://example.org/shapes');
        for (let i = 0; i < iters; i++) {
          const validator = new peer.SHACLValidator(shapesDs, { factory: peer.factory });
          const t0 = performance.now();
          const r = await validator.validate(dataDs);
          peerValidateOnlyUs = Math.min(peerValidateOnlyUs, (performance.now() - t0) * 1000);
          if (reducePeerReport(r).violations !== peerRes.violations) {
            throw new Error('validate-only rerun count drift');
          }
        }
      } catch (e) {
        failures.push(`${wl.name}: rdf-validate-shacl ERROR: ${e}`);
        continue;
      }
    }

    // -- the GATE: counts + conforms vs expected AND engine-vs-engine.
    const gateErrs = [];
    if (sparq.violations !== exp.violations || sparq.conforms !== exp.conforms) {
      gateErrs.push(`sparq-shacl-wasm ${sparq.violations}/${sparq.conforms} != ` +
        `expected ${exp.violations}/${exp.conforms}`);
    }
    if (peerComparable && peer) {
      if (peerRes.violations !== exp.violations || peerRes.conforms !== exp.conforms) {
        gateErrs.push(`rdf-validate-shacl ${peerRes.violations}/${peerRes.conforms} != ` +
          `expected ${exp.violations}/${exp.conforms}`);
      }
      if (peerRes.violations !== sparq.violations || peerRes.conforms !== sparq.conforms) {
        gateErrs.push('engine-vs-engine DISAGREEMENT');
      }
    }

    const row = {
      expected_violations: exp.violations,
      expected_conforms: exp.conforms,
      sparq_violations: sparq.violations,
      sparq_conforms: sparq.conforms,
      gate: gateErrs.length ? 'FAIL' : 'green',
    };
    if (peerComparable) {
      if (peer) {
        row.peer_violations = peerRes.violations;
        row.peer_conforms = peerRes.conforms;
      } else {
        row.peer = 'ABSENT (peer stack not installed; --allow-missing-peer)';
      }
    } else {
      row.peer = 'not-comparable (shape uses sh:sparql — rdf-validate-shacl is SHACL Core only; column absent, not 0)';
    }

    if (gateErrs.length) {
      failures.push(`${wl.name}: GATE: ${gateErrs.join('; ')}`);
      rows[wl.name] = row; // counts recorded, timings withheld
      continue;
    }

    // -- gate green: NOW (and only now) the timing rows.
    row.sparq_e2e_best_us = Math.round(sparqBestUs);
    if (dataParsed) row.sparq_validate_only_us = Math.round(sparqValidateOnlyUs);
    process.stdout.write(`${wl.name}\tsparq-shacl-wasm\t${sparq.violations}\t${Math.round(sparqBestUs)}\n`);
    if (peerComparable && peer) {
      row.peer_e2e_best_us = Math.round(peerBestUs);
      row.peer_validate_only_us = Math.round(peerValidateOnlyUs);
      process.stdout.write(`${wl.name}\trdf-validate-shacl\t${peerRes.violations}\t${Math.round(peerBestUs)}\n`);
    }
    rows[wl.name] = row;
  }

  // ---- deterministic column 2: wasm-pack --release artifact bytes ----------
  const bundle = {};
  for (const f of readdirSync(args.pkg)) {
    if (f.endsWith('_bg.wasm') || (f.endsWith('.js') && !f.endsWith('.d.js'))) {
      const bytes = readFileSync(join(args.pkg, f));
      bundle[f] = { bytes: statSync(join(args.pkg, f)).size,
                    gzip9_bytes: gzipSync(bytes, { level: 9 }).length };
    }
  }

  // [SONNET-4.6] sq-c6c2s: the PEER half of the byte column — bundle.mjs's
  // esbuild-minified, tree-shaken browser bundle (run.sh stage 2b), folded in
  // here so one envelope carries the whole comparison. Absent bundle => absent
  // sub-column with the reason recorded, never a fabricated 0.
  let peerBundle = { status: 'absent (run.sh stage 2b not run — see NO_BUNDLE / bundle.mjs)' };
  if (args.peerBundle) {
    try {
      const b = JSON.parse(readFileSync(args.peerBundle, 'utf-8'));
      peerBundle = {
        source: args.peerBundle,
        machine_independent: true,
        reproducible: false,
        reproducibility_note:
          'Bytes only (no wall clock), but NOT reproducible and NOT canonical: the peer stack ' +
          'and esbuild are installed unpinned (no committed lockfile), so a later gather can ' +
          'emit different bytes. `package_versions` is provenance for the run below. See ' +
          'bundle.mjs.',
        peer: b.peer_minified_bundle,
        sparq: b.sparq,
        ratio: b.ratio,
        wasm_opt_trim_probe: b.wasm_opt_trim_probe,
      };
    } catch (e) {
      peerBundle = { status: `absent (could not read ${args.peerBundle}: ${e})` };
    }
  }

  if (args.out) {
    const envelope = {
      gather: 'shacl-wasm-same-runtime-comparison',
      wave: 'browser-SHACL competitor baseline (sq-i858h, epic sq-hmd7l)',
      canonical: args.canonical,
      canonical_note: args.canonical
        ? 'CANONICAL: dedicated quiet box; both engines run one-shot in the same Node process, sequentially per workload.'
        : 'NON-canonical: shared work box. Timings are directional only — do NOT bake into docs/dashboards. Bundle bytes ARE deterministic (toolchain-pinned wasm-pack --release artifact).',
      git_commit: args.gitCommit || 'unknown',
      suite: 'shacl-wasm',
      scale: `vendored micro-ABox (${args.data}), hand-countable constants`,
      iters,
      runtime: `node ${process.version} (same process for both engines)`,
      engines: {
        'sparq-shacl-wasm': {
          version: args.gitCommit || 'unknown',
          features: args.sparqFeatures || '(default)',
          mode: 'wasm-pack --target nodejs --release; stateless Validator.validate one-shot end-to-end best-of-N (parse+validate+reduce in the timed section)'
            + (dataParsed
              ? '; PLUS stateful-feature ParsedGraph validate-only on pre-parsed graphs (sparq_validate_only_us, counts cross-checked vs one-shot)'
              : '; stateful column ABSENT (artifact built without the opt-in `stateful` feature)'),
        },
        'rdf-validate-shacl': peer ? {
          version: peerVersion,
          mode: 'in-process RDF/JS (same one-shot end-to-end best-of-N; validate-only on pre-parsed datasets as an extra advisory column)',
        } : { status: 'absent (gather-only npm deps not installed)' },
      },
      reduction: 'violations = report.results.length; conforms = report.conforms (the js_shacl_adapter.mjs / shacl_report_count.py contract)',
      gate: 'per-workload #violations + conforms must match expected.tsv AND agree engine-vs-engine BEFORE any timing row; sh:sparql workloads are sparq-only (peer capability-absent => column absent)',
      count_crosscheck: rows,
      bundle_bytes: {
        note: args.sparqFeatures
          ? `NOT the canonical bundle-bytes record: artifact built with non-default features (${args.sparqFeatures}). The deterministic record is the DEFAULT-features artifact.`
          : 'DETERMINISTIC per toolchain: the wasm-pack --release nodejs-target artifact (default features — no shacl-af). gzip-9 = the wire size a gzip-serving host ships.',
        wasm_pack: process.env.WASM_PACK_VERSION || 'unknown',
        artifacts: bundle,
        peer_minified_bundle: peerBundle,
      },
      env: {
        host: os.hostname(),
        machine: os.machine ? os.machine() : process.arch,
        os: `${os.type()} ${os.release()}`,
      },
      failures,
    };
    writeFileSync(args.out, JSON.stringify(envelope, null, 2) + '\n');
    process.stderr.write(`harness: envelope: ${args.out}\n`);
  }

  if (dataParsed) dataParsed.free();

  if (failures.length) {
    for (const f of failures) process.stderr.write(`harness: FAIL ${f}\n`);
    process.exit(1);
  }
  process.stderr.write('harness: agreement gate GREEN on every workload.\n');
}

main().catch((e) => {
  process.stderr.write(`harness: fatal: ${e && e.stack ? e.stack : e}\n`);
  process.exit(1);
});
