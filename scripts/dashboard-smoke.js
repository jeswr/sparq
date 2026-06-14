#!/usr/bin/env node
// [OPUS-4.8] Node smoke-test for the benchmark dashboard's PURE functions (bead sq-ocuf).
// dashboard.js exports its pure helpers under module.exports when require()d from node, so this
// runs them WITHOUT a browser/DOM. It also loads the committed bench/dashboard/metric-labels.json
// and asserts the readable-label plumbing (labelFor / suiteFor / titleFor / buildSummary).
//
// Usage:  node scripts/dashboard-smoke.js     (exit 0 = pass, 1 = fail)
// This is the script dashboard.js's header comment has long referenced; it was previously a
// dangling reference (no file). Now it exists and is the dashboard's CI smoke gate.
'use strict';

const fs = require('fs');
const path = require('path');

const ROOT = path.resolve(__dirname, '..');
const DASH = path.join(ROOT, 'bench', 'dashboard', 'dashboard.js');
const LABELS = path.join(ROOT, 'bench', 'dashboard', 'metric-labels.json');

let failures = 0;
function ok(cond, msg) {
  if (cond) { console.log('  ok   ' + msg); }
  else { console.error('  FAIL ' + msg); failures++; }
}
function eq(actual, expected, msg) {
  ok(actual === expected, msg + ' (got ' + JSON.stringify(actual) + ')');
}

// ---- load the label map into the global the module reads (it checks window/global) -----------
const labelsFile = JSON.parse(fs.readFileSync(LABELS, 'utf8'));
global.METRIC_LABELS = labelsFile;

// ---- require dashboard.js (module.exports branch returns the pure fns) -----------------------
const D = require(DASH);
ok(D && typeof D.labelFor === 'function', 'dashboard.js exports labelFor');
ok(typeof D.suiteFor === 'function', 'dashboard.js exports suiteFor');
ok(typeof D.titleFor === 'function', 'dashboard.js exports titleFor');
ok(typeof D.buildSummary === 'function', 'dashboard.js exports buildSummary');

// ---- metric-labels.json shape ----------------------------------------------------------------
ok(labelsFile && typeof labelsFile.labels === 'object', 'metric-labels.json has a labels object');
const labels = labelsFile.labels;
const stemCount = Object.keys(labels).length;
ok(stemCount > 100, 'labels map is populated (' + stemCount + ' stems)');
for (const k of Object.keys(labels)) {
  const r = labels[k];
  if (!r.label || !r.suite || !r.unit) {
    ok(false, 'label record ' + k + ' missing label/suite/unit');
    break;
  }
}

// ---- labelFor / suiteFor: the cryptic stems from the bead brief must become readable ----------
// These assert the label map is wired through (not the fallback) for the exact examples cited.
eq(D.labelFor('watdiv_S1_count_us'), 'WatDiv S1 — star query, count', 'watdiv_S1_count_us label');
eq(D.suiteFor('watdiv_S1_count_us'), 'WatDiv', 'watdiv suite');
ok(/Students/.test(D.labelFor('lubm_q06_count_us')), 'lubm_q06 mentions Students');
eq(D.suiteFor('lubm_q06_count_us'), 'LUBM (reasoning)', 'lubm suite');
ok(/star join/.test(D.labelFor('op_q02_star3_count_us')), 'op_q02_star3 mentions star join');
eq(D.suiteFor('op_q02_star3_count_us'), 'Operators', 'operator suite');
ok(/materialize/.test(D.labelFor('op_q03_chain_materialize_us')), 'mode appears in materialize label');
eq(D.suiteFor('load_s'), 'Pipeline', 'load_s -> Pipeline');
eq(D.suiteFor('wasm_bundle_bytes'), 'Memory / Size', 'wasm_bundle_bytes -> Memory / Size');

// ---- titleFor: raw stem first (transparency) + dataset/query lines ----------------------------
const t = D.titleFor('watdiv_S1_count_us');
ok(t.split('\n')[0] === 'watdiv_S1_count_us', 'titleFor leads with the raw stem');
ok(/dataset:/.test(t) && /query:/.test(t), 'titleFor includes dataset + query');

// ---- graceful fallback: an UNLABELLED metric must still get a label + a suite -----------------
const unknown = 'totally_new_metric_count_us';
ok(typeof D.labelFor(unknown) === 'string' && D.labelFor(unknown).length > 0,
   'unlabelled metric falls back to a humanized label');
ok(typeof D.suiteFor(unknown) === 'string' && D.suiteFor(unknown).length > 0,
   'unlabelled metric falls back to a structural suite');

// ---- buildSummary: groups by suite, in GROUP_ORDER, with readable row labels ------------------
const entries = [{
  commit: { id: 'deadbeefcafe', message: 'test', url: '#' },
  date: Date.now(),
  benches: [
    { name: 'load_s', value: 1.2, unit: 's' },
    { name: 'watdiv_S1_count_us', value: 30, unit: 'us' },
    { name: 'lubm_q06_count_us', value: 99, unit: 'us' },
    { name: 'op_q01_bgp_count_us', value: 5, unit: 'us' }
  ]
}];
const summary = D.buildSummary(entries);
ok(Array.isArray(summary.groups) && summary.groups.length >= 3, 'buildSummary produced groups');
const groupNames = summary.groups.map(function (g) { return g.group; });
ok(groupNames.indexOf('Pipeline') === 0, 'Pipeline is first group');
ok(groupNames.indexOf('WatDiv') !== -1 && groupNames.indexOf('LUBM (reasoning)') !== -1,
   'WatDiv + LUBM suites present as groups');
// each row carries the readable label + a title (raw stem) for the tooltip.
let sawWatdivRow = false;
summary.groups.forEach(function (g) {
  g.rows.forEach(function (r) {
    if (r.name === 'watdiv_S1_count_us') {
      sawWatdivRow = true;
      ok(r.label === 'WatDiv S1 — star query, count', 'summary row uses readable label');
      ok(r.title.indexOf('watdiv_S1_count_us') === 0, 'summary row title carries the raw stem');
    }
  });
});
ok(sawWatdivRow, 'watdiv row present in summary');

if (failures) {
  console.error('\nFAILED: ' + failures + ' assertion(s).');
  process.exit(1);
}
console.log('\nAll dashboard smoke checks passed.');
