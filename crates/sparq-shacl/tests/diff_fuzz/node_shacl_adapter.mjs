// [OPUS-4.8] sq-vz2v: Node / RDF-JS reference-engine adapter for the SHACL
// differential fuzzer (crates/sparq-shacl/tests/diff_fuzz.rs).
//
// A third class of reference alongside pySHACL (sq-55c1) and Apache Jena
// (sq-evws): the JS / RDF-JS ecosystem SHACL engines, as a NODE "js-lib" adapter
// (bead sq-eifd's js-lib KIND). It is the SAME "report-cli" contract — JSON
// request in, normalised JSON report out — so the Rust runner's comparison stays
// engine-agnostic (resolve_pyshacl / resolve_jena / resolve_node all yield the
// same RefEngine). Two engines are wired, selected by SHACL_DIFF_NODE_ENGINE:
//   - Zazuko `rdf-validate-shacl`   (engine=rdf-validate-shacl)  [default]
//   - Zazuko `shacl-engine`         (engine=shacl-engine)
// Both are MIT and the de-facto JS SHACL validators; a third independent family
// catches bugs where sparq and the Python/Java references happen to agree but are
// jointly wrong.
//
// CONTRACT (stdin -> stdout) — identical to tests/diff_fuzz/pyshacl_adapter.py
// and JenaShaclAdapter.java:
//   stdin  : a JSON object {"data": "<turtle>", "shapes": "<turtle>"}
//   stdout : {"conforms": bool,
//             "violations": [{"focus": str|null, "component": str|null,
//                             "path": str|null}, ...]}
//            — `focus`/`component` are full IRIs (or "_:bnode" for blank nodes,
//            which have no cross-graph identity); `path` is the bare predicate IRI
//            for a simple path, the "_:path" sentinel for any complex/blank-rooted
//            path, or null when the result carries no path. Exactly the shape the
//            Rust runner's RefReport deserialises and `ref_keys` normalises.
//   exit   : 0 on a produced report (even non-conforming); non-zero only on an
//            adapter/engine ERROR (so the runner distinguishes "engine says X"
//            from "engine could not run"), with a diagnostic on stderr.
//
// We read the report from the report DATASET (the RDF/JS graph), NOT from the
// engine's structured `report.results` JS objects, so the normalisation is the
// IDENTICAL reduction the pySHACL and Jena adapters perform over the report graph
// (collect sh:ValidationResult subjects; read sh:focusNode / sh:resultPath /
// sh:sourceConstraintComponent; EXCLUDE any result that is the object of an
// sh:detail — sh:detail is non-normative, SHACL §3.6.2, and sparq keeps nested
// sub-results off the comparable top-level set).
//
// Reads ONE request and emits ONE report (one Node process per case, mirroring the
// pySHACL/Jena adapters' per-case isolation).
//
// The JS engines are NOT a committed dependency (AGENTS.md: reference engines stay
// out of git). The CI lane `npm i`s them into a scratch dir and points the runner
// at it via SHACL_DIFF_NODE_MODULES. When neither resolves, the Rust runner SKIPS
// the Node engine cleanly.

import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { Readable } from 'node:stream';

const SH = 'http://www.w3.org/ns/shacl#';
const RDF_TYPE = 'http://www.w3.org/1999/02/22-rdf-syntax-ns#type';

// Resolve the gather-only JS engines + RDF/JS factory pieces from a modules dir
// (SHACL_DIFF_NODE_MODULES) or cwd. Mirrors the bench adapter's importer so both
// find the same `npm i`-installed engines.
function makeImporter(modulesDir) {
  const base = modulesDir
    ? pathToFileURL(modulesDir.replace(/\/?$/, '/') + 'package.json')
    : pathToFileURL(process.cwd() + '/package.json');
  const req = createRequire(base);
  return { require: (id) => req(id) };
}

// Build the RDF/JS environment rdf-validate-shacl / shacl-engine consume
// (data-model + dataset + term-map + namespace + clownface), composed through
// @rdfjs/environment — exactly how @zazuko/env builds its env, without taking the
// heavier bundle as a dependency. `.default` unwraps transpiled-ESM CJS entries.
function makeFactory(imp) {
  const Environment = imp.require('@rdfjs/environment').default || imp.require('@rdfjs/environment');
  const def = (id, sub) => {
    const m = imp.require(sub ? `${id}/${sub}` : id);
    return m.default || m;
  };
  const DataFactory = def('@rdfjs/data-model', 'Factory.js');
  const DatasetFactory = def('@rdfjs/dataset', 'Factory.js');
  const TermMapFactory = def('@rdfjs/term-map', 'Factory.js');
  const NamespaceFactory = def('@rdfjs/namespace', 'Factory.js');
  const ClownfaceFactory = def('clownface', 'Factory.js');
  return new Environment([
    DataFactory, DatasetFactory, TermMapFactory, NamespaceFactory, ClownfaceFactory,
  ]);
}

async function parseTurtleToDataset(imp, factory, text, baseIRI) {
  const ParserN3mod = imp.require('@rdfjs/parser-n3');
  const ParserN3 = ParserN3mod.default || ParserN3mod;
  const parser = new ParserN3({ factory, baseIRI });
  const stream = parser.import(Readable.from([text]));
  const ds = factory.dataset();
  for await (const quad of stream) ds.add(quad);
  return ds;
}

// Engine adapters: each returns the report's RDF/JS dataset (the report GRAPH),
// which we then normalise the same way as the Python/Java references.
async function runRdfValidateShacl(imp, factory, dataDs, shapesDs) {
  const SHACLValidator = imp.require('rdf-validate-shacl').default || imp.require('rdf-validate-shacl');
  const validator = new SHACLValidator(shapesDs, { factory });
  const report = await validator.validate(dataDs);
  return { conforms: report.conforms, dataset: report.dataset };
}

async function runShaclEngine(imp, factory, dataDs, shapesDs) {
  const mod = imp.require('shacl-engine');
  const Validator = mod.Validator || mod.default;
  const validator = new Validator(shapesDs, { factory });
  const report = await validator.validate({ dataset: dataDs });
  return { conforms: report.conforms, dataset: report.dataset || null };
}

const ENGINES = {
  'rdf-validate-shacl': runRdfValidateShacl,
  'shacl-engine': runShaclEngine,
};

// A graph-independent term key: blank nodes collapse to '_:bnode' (no stable
// cross-graph identity — the same tolerance the Python/Java references and the
// Rust runner's norm_term use); named nodes render to the bare IRI.
function termKey(term) {
  if (!term) return null;
  if (term.termType === 'BlankNode') return '_:bnode';
  if (term.termType === 'NamedNode') return term.value;
  if (term.termType === 'Literal') return `"${term.value}"`;
  return String(term.value);
}

// An sh:resultPath rendered as a comparable string: a simple predicate path is a
// NamedNode and renders to the bare IRI (the same string sparq emits for
// Path::Predicate); any complex / blank-rooted path renders to the "_:path"
// sentinel — we do NOT byte-match the impl-specific Turtle serialisation, the
// (focus, component) pair + path-presence carry the signal. Null when absent.
function pathKey(term) {
  if (!term) return null;
  if (term.termType === 'NamedNode') return term.value;
  return '_:path';
}

// Identity key preserving blank-node distinctness (NOT the comparison-tolerant
// "_:bnode" collapse): used only to test result-node membership against the
// sh:detail object set, so distinct blank nodes don't all collide.
function nodeId(term) {
  return term.termType === 'BlankNode' ? `bnode:${term.value}` : `iri:${term.value}`;
}

// Reduce the report GRAPH to the normalised violation set. Reads the same way as
// pyshacl_adapter.run / JenaShaclAdapter.normaliseGraph:
//   - subjects of (?s rdf:type sh:ValidationResult) are the results;
//   - EXCLUDE any result that is the object of an sh:detail (nested sub-result of
//     an sh:node / logical component — non-normative, kept off sparq's top-level
//     set), so we never diff two engines' OPTIONAL detail policies;
//   - per top-level result read sh:focusNode / sh:sourceConstraintComponent /
//     sh:resultPath.
function normaliseReport(dataset, conforms) {
  // First pass: index by subject + collect nested (sh:detail object) node ids.
  const bySubject = new Map(); // nodeId -> { focus, comp, path, isResult }
  const nested = new Set();
  for (const q of dataset) {
    const sid = nodeId(q.subject);
    let rec = bySubject.get(sid);
    if (!rec) {
      rec = { focus: null, comp: null, path: null, isResult: false };
      bySubject.set(sid, rec);
    }
    const p = q.predicate.value;
    if (p === RDF_TYPE && q.object.value === SH + 'ValidationResult') rec.isResult = true;
    else if (p === SH + 'focusNode') rec.focus = q.object;
    else if (p === SH + 'sourceConstraintComponent') rec.comp = q.object;
    else if (p === SH + 'resultPath') rec.path = q.object;
    else if (p === SH + 'detail') nested.add(nodeId(q.object));
  }
  const violations = [];
  for (const [sid, rec] of bySubject) {
    if (!rec.isResult) continue;
    if (nested.has(sid)) continue; // nested sub-result — excluded.
    violations.push({
      focus: termKey(rec.focus),
      component: termKey(rec.comp),
      path: pathKey(rec.path),
    });
  }
  return { conforms: Boolean(conforms), violations };
}

async function readStdin() {
  const chunks = [];
  for await (const c of process.stdin) chunks.push(c);
  return Buffer.concat(chunks).toString('utf-8');
}

async function main() {
  const engineName = process.env.SHACL_DIFF_NODE_ENGINE || 'rdf-validate-shacl';
  const runner = ENGINES[engineName];
  if (!runner) {
    process.stderr.write(
      `node_shacl_adapter: unknown engine '${engineName}' (have: ${Object.keys(ENGINES).join(', ')})\n`,
    );
    process.exit(2);
  }

  let req;
  try {
    req = JSON.parse(await readStdin());
  } catch (e) {
    process.stderr.write(`node_shacl_adapter: bad request JSON: ${e}\n`);
    process.exit(2);
  }

  const imp = makeImporter(process.env.SHACL_DIFF_NODE_MODULES);
  let factory;
  try {
    factory = makeFactory(imp);
  } catch (e) {
    process.stderr.write(
      `node_shacl_adapter: cannot load RDF/JS factory (npm i in SHACL_DIFF_NODE_MODULES?): ${e}\n`,
    );
    process.exit(1);
  }

  let out;
  try {
    const dataDs = await parseTurtleToDataset(imp, factory, req.data, 'http://ex/');
    const shapesDs = await parseTurtleToDataset(imp, factory, req.shapes, 'http://ex/');
    const { conforms, dataset } = await runner(imp, factory, dataDs, shapesDs);
    out = dataset
      ? normaliseReport(dataset, conforms)
      : { conforms: Boolean(conforms), violations: [] };
  } catch (e) {
    process.stderr.write(
      `node_shacl_adapter: engine error (${engineName}): ${e && e.stack ? e.stack : e}\n`,
    );
    process.exit(1);
  }

  process.stdout.write(JSON.stringify(out) + '\n');
}

// [OPUS-4.8] Self-test mode (`--selftest`): exercise the sh:detail exclusion on a
// synthetic report dataset (no engine / npm dep needed), mirroring the Jena
// adapter's --selftest. Proves a nested sub-result that is the object of an
// sh:detail is dropped from the top-level violation set, independent of whether an
// engine build emits sh:detail. The Rust runner drives this in the fast lane when
// `node` resolves. Exit 0 on pass, non-zero with a diagnostic on fail.
function selfTest() {
  // Minimal in-memory dataset over plain term objects so the self-test needs no
  // npm dependency. It is iterable (the normaliser only iterates and reads
  // predicate/object .value), with terms exposing { termType, value }.
  const nn = (v) => ({ termType: 'NamedNode', value: v });
  const bn = (v) => ({ termType: 'BlankNode', value: v });
  const top = bn('top');
  const inner = bn('inner');
  const quads = [
    [top, nn(RDF_TYPE), nn(SH + 'ValidationResult')],
    [top, nn(SH + 'focusNode'), nn('http://ex/focus')],
    [top, nn(SH + 'sourceConstraintComponent'), nn(SH + 'NodeConstraintComponent')],
    [top, nn(SH + 'resultPath'), nn('http://ex/p')],
    [top, nn(SH + 'detail'), inner],
    [inner, nn(RDF_TYPE), nn(SH + 'ValidationResult')],
    [inner, nn(SH + 'focusNode'), nn('http://ex/value')],
    [inner, nn(SH + 'sourceConstraintComponent'), nn(SH + 'MinLengthConstraintComponent')],
  ].map(([subject, predicate, object]) => ({ subject, predicate, object }));
  const dataset = quads; // an Array is iterable, which is all normaliseReport needs.
  const got = normaliseReport(dataset, false);
  const want = {
    conforms: false,
    violations: [
      {
        focus: 'http://ex/focus',
        component: SH + 'NodeConstraintComponent',
        path: 'http://ex/p',
      },
    ],
  };
  const gotJson = JSON.stringify(got);
  const wantJson = JSON.stringify(want);
  if (gotJson !== wantJson) {
    process.stderr.write('node_shacl_adapter selftest FAIL: sh:detail exclusion\n');
    process.stderr.write(`  want: ${wantJson}\n`);
    process.stderr.write(`  got:  ${gotJson}\n`);
    return false;
  }
  return true;
}

if (process.argv.includes('--selftest')) {
  process.exit(selfTest() ? 0 : 3);
} else {
  main().catch((e) => {
    process.stderr.write(`node_shacl_adapter: fatal: ${e && e.stack ? e.stack : e}\n`);
    process.exit(1);
  });
}
