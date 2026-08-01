#!/usr/bin/env node
// [SONNET-4.6] sq-c6c2s — minified-browser-bundle byte comparison for the
// browser SHACL story: the esbuild-bundled `rdf-validate-shacl` peer vs the
// wasm-pack artifact of `sparq-shacl-wasm`. Machine-independent (no wall clock)
// but NOT reproducible across gathers — read REPRODUCIBILITY below before
// citing the peer bytes or the ratio.
//
//   node bundle.mjs --pkg [label=]<wasm-pack out dir> [--pkg ...]
//                   --modules-dir <gather-only node_modules parent>
//                   [--out envelope.json] [--wasm-opt-probe]
//
// WHY this script exists. research/gap-shacl-wasm-2026-07.md recorded the wasm
// artifact's bytes but deliberately made NO byte-ratio claim, because the only
// peer number available then was the peer's UNPACKED npm footprint — not a wire
// size (no tree-shaking, no minification, dev files and all). The comparable
// number is what a bundler actually emits for a browser app, so this script
// builds that bundle with esbuild and compares gzip-9 wire bytes.
//
// REPRODUCIBILITY (read this before citing the peer bytes). No wall clock is
// involved, so these numbers are machine-independent and a quiet box buys
// nothing — but they are NOT reproducible across gathers. run.sh installs the
// peer stack from BARE package names (esbuild included) with no committed
// lockfile, so a later gather can resolve different validator, RDF/JS,
// transitive, or bundler code and emit different bytes. The recorded
// `package_versions` and per-bundle sha256 are PROVENANCE for the run that
// produced them, NOT a recipe to re-derive them; the peer column and the ratio
// are therefore point-in-time and NOT canonical. Only the sparq side is pinned
// (repo commit + the recorded wasm-pack/rustc toolchain). Pinning the peer half
// would take a committed manifest + lockfile installed with `npm ci` — compare
// bench/wasm-compare/bundle.mjs, which pins its peer to npm:oxigraph@<PIN> and
// records the tarball sha256. gzip-9 is Node zlib and therefore
// zlib-version-dependent — informative, with the raw byte counts as the more
// stable metric.
//
// TWO PEER VARIANTS, and which one is like-for-like:
//   - `validator-only` — `rdf-validate-shacl` alone. A LOWER BOUND on the peer:
//     it cannot read a Turtle document, so it is not a usable browser SHACL app.
//   - `app-parity`     — the validator PLUS the RDF/JS factory + N3 parser stack
//     (exactly the imports bench/shacl-wasm/harness.mjs uses). This is the
//     honest comparable, because the sparq wasm artifact carries its OWN Turtle
//     parser: "parse two documents and validate" is the capability being sized.
// Both peer numbers are LOWER BOUNDS in a second way as well — Node core imports
// reached by the RDF/JS stack are stubbed to empty modules rather than polyfilled
// (the stubbed list is recorded); a real browser build ships polyfill bytes for
// them. Where the peer is understated, that is deliberate: never flatter sparq.
//
// CAPABILITY CAVEAT (do not read these bytes as a like-for-like feature matrix):
// `rdf-validate-shacl` is SHACL Core only, while the sparq artifact also carries
// SHACL-SPARQL (`sh:sparql`, W3C SHACL §5.2) and therefore a whole SPARQL engine.
// The byte gap prices a capability gap, not just an implementation-efficiency gap.

import { readFileSync, writeFileSync, statSync, existsSync, mkdtempSync, rmSync } from 'node:fs';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { gzipSync } from 'node:zlib';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { basename, join } from 'node:path';
import { tmpdir } from 'node:os';
import os from 'node:os';

// Node core modules the RDF/JS stack may reach for. Stubbed to empty so a
// browser-platform bundle can be emitted at all; every stub actually taken is
// recorded, and makes the peer number a lower bound.
const NODE_BUILTINS = [
  'stream', 'events', 'util', 'buffer', 'string_decoder', 'process', 'path',
  'fs', 'url', 'zlib', 'crypto', 'os', 'assert', 'querystring', 'http', 'https',
];

function parseArgs(argv) {
  const a = { pkgs: [] };
  for (let i = 0; i < argv.length; i++) {
    const k = argv[i];
    if (k === '--pkg') a.pkgs.push(argv[++i]);
    else if (k === '--modules-dir') a.modulesDir = argv[++i];
    else if (k === '--out') a.out = argv[++i];
    else if (k === '--git-commit') a.gitCommit = argv[++i];
    else if (k === '--wasm-opt-probe') a.wasmOptProbe = true;
    else { process.stderr.write(`bundle: unknown arg ${k}\n`); process.exit(2); }
  }
  if (!a.pkgs.length || !a.modulesDir) {
    process.stderr.write(
      'usage: node bundle.mjs --pkg [label=]<wasm-pack out dir> [--pkg ...] ' +
      '--modules-dir <node_modules parent> [--out envelope.json] ' +
      '[--git-commit SHA] [--wasm-opt-probe]\n');
    process.exit(2);
  }
  return a;
}

const gz9 = (buf) => gzipSync(buf, { level: 9 }).length;
const sha256 = (buf) => createHash('sha256').update(buf).digest('hex');

function measure(file) {
  const buf = readFileSync(file);
  return { bytes: statSync(file).size, gzip9_bytes: gz9(buf), sha256: sha256(buf) };
}

// ---- sparq side: the wasm-pack artifact(s) ---------------------------------
// A pkg dir may be labelled `label=dir`; unlabelled dirs take their basename.
// run.sh writes a `.features` marker into each pkg it builds, so the feature set
// (and, for a trim variant, the cargo profile) travels with the artifact.
function measureSparqPkg(spec) {
  const eq = spec.indexOf('=');
  const label = eq === -1 ? basename(spec) : spec.slice(0, eq);
  const dir = eq === -1 ? spec : spec.slice(eq + 1);
  const wasm = join(dir, 'sparq_shacl_wasm_bg.wasm');
  const glue = join(dir, 'sparq_shacl_wasm.js');
  if (!existsSync(wasm)) {
    process.stderr.write(`bundle: no wasm artifact at ${wasm} — build it first (see run.sh)\n`);
    process.exit(2);
  }
  const featuresMarker = join(dir, '.features');
  const wasmRow = measure(wasm);
  const glueRow = existsSync(glue) ? measure(glue) : null;
  // wasm-pack writes the bindgen target into pkg/package.json (`main` for the
  // nodejs target, `module` for web); record it rather than assume.
  let target = 'unknown';
  try {
    const pj = JSON.parse(readFileSync(join(dir, 'package.json'), 'utf-8'));
    target = pj.module ? 'web/bundler' : pj.main ? 'nodejs' : 'unknown';
  } catch { /* pkg/package.json absent — leave 'unknown' */ }
  return {
    label,
    dir,
    features: existsSync(featuresMarker)
      ? (readFileSync(featuresMarker, 'utf-8').trim() || '(default)')
      : '(unrecorded)',
    bindgen_target: target,
    artifacts: { 'sparq_shacl_wasm_bg.wasm': wasmRow, ...(glueRow ? { 'sparq_shacl_wasm.js': glueRow } : {}) },
    total: {
      bytes: wasmRow.bytes + (glueRow ? glueRow.bytes : 0),
      gzip9_bytes: wasmRow.gzip9_bytes + (glueRow ? glueRow.gzip9_bytes : 0),
    },
  };
}

// ---- peer side: esbuild-minified, tree-shaken browser bundles ---------------
const PEER_ENTRIES = {
  // Lower bound: the validator alone. Cannot read a Turtle document, so it is
  // not a runnable browser SHACL app — recorded for attribution, not comparison.
  'validator-only': [
    "import SHACLValidator from 'rdf-validate-shacl';",
    'globalThis.__sparqBundleKeep = [SHACLValidator];',
  ].join('\n'),
  // The like-for-like column: everything harness.mjs imports to go from two
  // Turtle strings to a validation report — the capability the wasm artifact has.
  'app-parity': [
    "import SHACLValidator from 'rdf-validate-shacl';",
    "import Environment from '@rdfjs/environment';",
    "import DataFactory from '@rdfjs/data-model/Factory.js';",
    "import DatasetFactory from '@rdfjs/dataset/Factory.js';",
    "import TermMapFactory from '@rdfjs/term-map/Factory.js';",
    "import NamespaceFactory from '@rdfjs/namespace/Factory.js';",
    "import ClownfaceFactory from 'clownface/Factory.js';",
    "import ParserN3 from '@rdfjs/parser-n3';",
    'globalThis.__sparqBundleKeep = [SHACLValidator, Environment, DataFactory,',
    '  DatasetFactory, TermMapFactory, NamespaceFactory, ClownfaceFactory, ParserN3];',
  ].join('\n'),
};

// Read a dependency's pinned version WITHOUT assuming its `exports` map exposes
// "./package.json" — resolve the entry point instead and walk up to the nearest
// package.json whose `name` matches. Returns 'unknown' rather than throwing:
// an unreadable pin must not cost us the byte measurement.
function readPkgVersion(req, name) {
  try {
    return JSON.parse(readFileSync(req.resolve(`${name}/package.json`), 'utf-8')).version;
  } catch { /* exports map hides package.json — walk up from the entry point */ }
  try {
    let dir = join(req.resolve(name), '..');
    for (let i = 0; i < 12; i++) {
      const candidate = join(dir, 'package.json');
      if (existsSync(candidate)) {
        const pj = JSON.parse(readFileSync(candidate, 'utf-8'));
        if (pj.name === name && pj.version) return pj.version;
      }
      const parent = join(dir, '..');
      if (parent === dir) break;
      dir = parent;
    }
  } catch { /* fall through to 'unknown' */ }
  return 'unknown';
}

async function buildPeerBundles(modulesDir) {
  const req = createRequire(pathToFileURL(modulesDir.replace(/\/?$/, '/') + 'package.json'));
  let esbuild, esbuildVersion;
  try {
    esbuild = req('esbuild');
    esbuildVersion = readPkgVersion(req, 'esbuild');
  } catch (e) {
    return { status: `absent (esbuild not installed in ${modulesDir}: ${e})` };
  }

  const versions = {};
  for (const p of ['rdf-validate-shacl', '@rdfjs/data-model', '@rdfjs/dataset',
    '@rdfjs/term-map', '@rdfjs/namespace', '@rdfjs/parser-n3', '@rdfjs/environment',
    'clownface']) {
    try {
      req.resolve(p); // presence is required — an absent peer means no comparison
    } catch (e) {
      return { status: `absent (peer package ${p} not installed in ${modulesDir}: ${e})` };
    }
    // The VERSION is best-effort: a package whose `exports` map omits
    // "./package.json" (increasingly common) still bundles fine, so a
    // pin we cannot read is recorded as 'unknown' rather than failing the
    // measurement — but it is never silently omitted from the envelope.
    versions[p] = readPkgVersion(req, p);
  }

  // One plugin instance (and one stub set) PER build, so each variant reports
  // the builtins IT reached — a shared set would credit the first variant with
  // every stub and leave the later ones looking cleaner than they are.
  const stubPluginFor = (stubbed) => ({
    name: 'node-builtin-stub',
    setup(build) {
      const filter = new RegExp(`^(node:)?(${NODE_BUILTINS.join('|')})$`);
      build.onResolve({ filter }, (a) => {
        stubbed.add(a.path.replace(/^node:/, ''));
        return { path: a.path, namespace: 'node-builtin-stub' };
      });
      // CommonJS on purpose: an ESM stub would have to enumerate every named
      // export the peer imports (`{ Readable } from 'node:stream'` and friends)
      // and esbuild would hard-error on each one it cannot find. A CJS stub has
      // statically unknown exports, so any named import binds without error —
      // which is what we want, since these bytes are never executed.
      build.onLoad({ filter: /.*/, namespace: 'node-builtin-stub' }, () => ({
        contents: 'module.exports = {};', loader: 'js',
      }));
    },
  });

  const variants = {};
  for (const [name, contents] of Object.entries(PEER_ENTRIES)) {
    const stubbed = new Set();
    let out;
    try {
      out = await esbuild.build({
        stdin: { contents, resolveDir: modulesDir, sourcefile: `${name}.mjs`, loader: 'js' },
        bundle: true,
        minify: true,
        treeShaking: true,
        format: 'esm',
        platform: 'browser',
        target: ['es2022'],
        legalComments: 'none',
        write: false,
        plugins: [stubPluginFor(stubbed)],
      });
    } catch (e) {
      variants[name] = { status: `build FAILED: ${e && e.message ? e.message : e}` };
      continue;
    }
    const buf = Buffer.from(out.outputFiles[0].contents);
    variants[name] = {
      bytes: buf.length,
      gzip9_bytes: gz9(buf),
      sha256: sha256(buf),
      stubbed_node_builtins: [...stubbed].sort(),
    };
  }

  return {
    bundler: `esbuild ${esbuildVersion}`,
    options: 'bundle, minify, treeShaking, format=esm, platform=browser, target=es2022, legalComments=none',
    package_versions: versions,
    install_pinning:
      'FLOATING: run.sh npm-installs these packages (and esbuild) by bare name with no ' +
      'committed lockfile, so top-level AND transitive versions are whatever the registry ' +
      'resolved at gather time. These versions are recorded as PROVENANCE for this run — they ' +
      'do not make the bytes reproducible by a later gather.',
    lower_bound_note:
      'Peer bytes are a LOWER BOUND twice over: (1) Node core imports reached by the ' +
      'RDF/JS stack are stubbed to EMPTY modules rather than polyfilled, so a real ' +
      'browser build ships more; (2) `validator-only` cannot parse Turtle at all. ' +
      'Understating the peer is deliberate — the comparison must never flatter sparq.',
    variants,
  };
}

// ---- trim probe: is there headroom left beyond the profile`s -Oz? ----------
// The crate already sets `wasm-opt = ["-Oz"]` for the wasm-pack release profile,
// so this re-runs binaryen with the size-max settings (-Oz to convergence plus
// the strip passes) over the SHIPPED artifact. A delta near zero is the useful
// answer: it says the packaged -Oz already captured the binaryen-side headroom
// and any further trim has to come from the Rust side (features/deps/profile).
function wasmOptProbe(pkgRow) {
  if (!spawnSync('wasm-opt', ['--version'], { encoding: 'utf8' }).stdout) {
    return { status: 'skipped (wasm-opt not on PATH)' };
  }
  const version = spawnSync('wasm-opt', ['--version'], { encoding: 'utf8' }).stdout.trim();
  const dir = mkdtempSync(join(tmpdir(), 'shacl-wasm-trim-'));
  try {
    const input = join(pkgRow.dir, 'sparq_shacl_wasm_bg.wasm');
    const output = join(dir, 'trimmed.wasm');
    const flags = ['-all', '-Oz', '--converge', '--strip-debug', '--strip-producers', '--vacuum'];
    const r = spawnSync('wasm-opt', [...flags, input, '-o', output], { encoding: 'utf8' });
    if (r.status !== 0) {
      return { status: `failed: ${(r.stderr || '').trim().slice(0, 400)}`, binaryen: version };
    }
    const before = pkgRow.artifacts['sparq_shacl_wasm_bg.wasm'];
    const after = measure(output);
    return {
      binaryen: version,
      flags: flags.join(' '),
      applied_to: pkgRow.label,
      before: { bytes: before.bytes, gzip9_bytes: before.gzip9_bytes },
      after: { bytes: after.bytes, gzip9_bytes: after.gzip9_bytes },
      delta_bytes: after.bytes - before.bytes,
      delta_gzip9_bytes: after.gzip9_bytes - before.gzip9_bytes,
      note:
        'A delta at or near zero means the packaged `wasm-opt = ["-Oz"]` already ' +
        'captured the binaryen-side headroom; further trim must come from the Rust ' +
        'side (cargo profile, feature set, dependency graph). This probe is a ' +
        'MEASUREMENT, not a shipped build step.',
    };
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const sparqRows = args.pkgs.map(measureSparqPkg);
  const peer = await buildPeerBundles(args.modulesDir);
  const trim = args.wasmOptProbe ? wasmOptProbe(sparqRows[0]) : { status: 'not requested (--wasm-opt-probe)' };

  // The headline ratio is gzip-9 wire bytes, sparq's FIRST (default) artifact
  // against the peer's app-parity bundle — the only pair with matching
  // capability (parse a Turtle document, then validate).
  let ratio = null;
  const appParity = peer.variants && peer.variants['app-parity'];
  if (appParity && typeof appParity.gzip9_bytes === 'number') {
    ratio = {
      metric: 'gzip-9 wire bytes, sparq (wasm + glue) / peer app-parity bundle',
      sparq_label: sparqRows[0].label,
      sparq_gzip9_bytes: sparqRows[0].total.gzip9_bytes,
      peer_gzip9_bytes: appParity.gzip9_bytes,
      sparq_over_peer: +(sparqRows[0].total.gzip9_bytes / appParity.gzip9_bytes).toFixed(3),
      direction: sparqRows[0].total.gzip9_bytes > appParity.gzip9_bytes
        ? 'sparq BEHIND on wire bytes (expected: a wasm engine vs a JS library)'
        : 'sparq AHEAD on wire bytes',
      capability_caveat:
        'Not a like-for-like feature matrix: the peer is SHACL Core only, while the ' +
        'sparq artifact also carries SHACL-SPARQL (W3C SHACL §5.2) and the SPARQL ' +
        'engine it routes constraint queries through. The gap prices a capability ' +
        'difference as well as an implementation one.',
    };
  }

  const envelope = {
    gather: 'shacl-wasm-minified-bundle-bytes',
    bead: 'sq-c6c2s',
    machine_independent: true,
    reproducible: false,
    reproducibility_note:
      'Bytes only — no wall clock, so no quiet box is required and the numbers do not depend ' +
      'on machine load. They are NOT reproducible across gathers, and NOT canonical: the peer ' +
      'stack (esbuild included) is installed from bare package names with no committed ' +
      'lockfile, so a later gather can resolve different peer, transitive, or bundler code and ' +
      'emit different bytes. `peer_minified_bundle.package_versions` and the per-bundle sha256 ' +
      'are PROVENANCE for this run, not a recipe to re-derive it. Only the sparq rows are ' +
      'pinned (git_commit + the recorded wasm-pack/rustc toolchain). Raw `bytes` are the more ' +
      'stable metric; `gzip9_bytes` is Node zlib level 9 and therefore zlib-version-dependent.',
    git_commit: args.gitCommit || 'unknown',
    sparq: sparqRows,
    peer_minified_bundle: peer,
    ratio,
    wasm_opt_trim_probe: trim,
    env: {
      node: process.version,
      host: os.hostname(),
      os: `${os.type()} ${os.release()}`,
      wasm_pack: process.env.WASM_PACK_VERSION || 'unknown',
    },
  };

  if (args.out) {
    writeFileSync(args.out, JSON.stringify(envelope, null, 2) + '\n');
    process.stderr.write(`bundle: envelope: ${args.out}\n`);
  }

  // ---- human-readable table (stdout) ----
  const row = (label, bytes, gzip, note) =>
    `${label.padEnd(44)} ${String(bytes).padStart(9)} B  gzip-9 ${String(gzip).padStart(9)} B  ${note}`;
  process.stdout.write('== minified browser-bundle bytes (sq-c6c2s) ==\n');
  for (const s of sparqRows) {
    for (const [name, m] of Object.entries(s.artifacts)) {
      process.stdout.write(row(`sparq[${s.label}] ${name}`, m.bytes, m.gzip9_bytes,
        `features ${s.features}, target ${s.bindgen_target}`) + '\n');
    }
    process.stdout.write(row(`sparq[${s.label}] TOTAL`, s.total.bytes, s.total.gzip9_bytes, '') + '\n');
  }
  if (peer.variants) {
    for (const [name, v] of Object.entries(peer.variants)) {
      if (typeof v.bytes !== 'number') {
        process.stdout.write(`peer[${name}]`.padEnd(44) + ` ${v.status}\n`);
        continue;
      }
      process.stdout.write(row(`peer[${name}]`, v.bytes, v.gzip9_bytes,
        v.stubbed_node_builtins.length
          ? `stubbed: ${v.stubbed_node_builtins.join(',')}`
          : 'no node builtins stubbed') + '\n');
    }
  } else {
    process.stdout.write(`peer bundle ABSENT — ${peer.status}\n`);
  }
  if (ratio) {
    process.stdout.write(
      `\nwire ratio (gzip-9): sparq/peer(app-parity) = ${ratio.sparq_over_peer} — ${ratio.direction}\n` +
      `capability caveat: peer is SHACL Core only; sparq also carries SHACL-SPARQL + the SPARQL engine.\n`);
  }
  if (trim.delta_bytes !== undefined) {
    process.stdout.write(
      `wasm-opt trim probe (${trim.flags}): ${trim.delta_bytes} B raw, ` +
      `${trim.delta_gzip9_bytes} B gzip-9 vs the shipped artifact\n`);
  } else {
    process.stdout.write(`wasm-opt trim probe: ${trim.status}\n`);
  }
}

main().catch((e) => {
  process.stderr.write(`bundle: fatal: ${e && e.stack ? e.stack : e}\n`);
  process.exit(1);
});
