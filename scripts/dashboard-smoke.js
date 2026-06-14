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

// ============================================================================================
// [OPUS-4.8] sq-xvow — featured well-known suites at the TOP (+ competitor seam for sq-t0c3).
// ============================================================================================
ok(typeof D.featuredSuiteOf === 'function', 'dashboard.js exports featuredSuiteOf');
ok(typeof D.buildFeatured === 'function', 'dashboard.js exports buildFeatured');
ok(typeof D.competitorsFor === 'function', 'dashboard.js exports competitorsFor');

// featuredSuiteOf: recognised public suites are featured; engine micro-suites are NOT.
eq(D.featuredSuiteOf('watdiv_S1_count_us') && D.featuredSuiteOf('watdiv_S1_count_us').key, 'WatDiv',
   'watdiv metric is featured under WatDiv');
eq(D.featuredSuiteOf('lubm_q06_count_us') && D.featuredSuiteOf('lubm_q06_count_us').key, 'LUBM',
   'lubm metric is featured under LUBM');
eq(D.featuredSuiteOf('sp2b_q1_count_us') && D.featuredSuiteOf('sp2b_q1_count_us').key, 'SP2Bench',
   'sp2b metric is featured under SP2Bench');
ok(D.featuredSuiteOf('op_q01_bgp_count_us') === null, 'operator micro-suite is NOT featured');
ok(D.featuredSuiteOf('load_s') === null, 'pipeline metric is NOT featured');
// Deep Taxonomy isn't in the label map yet (sq-1hgz) — name-token fallback must still recognise it.
eq(D.featuredSuiteOf('deeptax_d5_count_us') && D.featuredSuiteOf('deeptax_d5_count_us').key, 'Deep Taxonomy',
   'unlabelled deep-taxonomy metric is featured via name fallback');

// buildFeatured: groups by suite, latest value per metric, competitor SEAM present.
const featEntries = [{
  commit: { id: 'feedfacefeed', message: 'feat test', url: '#' }, date: Date.now(),
  benches: [
    { name: 'load_s', value: 1.2, unit: 's' },                  // NOT featured
    { name: 'op_q01_bgp_count_us', value: 5, unit: 'us' },      // NOT featured
    { name: 'watdiv_S1_count_us', value: 30, unit: 'µs' },
    { name: 'lubm_q06_count_us', value: 99, unit: 'µs' },
    { name: 'sp2b_q1_count_us', value: 50, unit: 'µs' }
  ]
}];
const feat = D.buildFeatured(featEntries, null);
const featSuites = feat.groups.map(function (g) { return g.suite; });
ok(featSuites.indexOf('WatDiv') !== -1 && featSuites.indexOf('LUBM') !== -1 &&
   featSuites.indexOf('SP2Bench') !== -1, 'buildFeatured surfaces WatDiv/LUBM/SP2Bench');
ok(featSuites.indexOf('Operators') === -1 && featSuites.indexOf('Pipeline') === -1,
   'buildFeatured excludes non-featured (engine) suites');
// the SEAM: no competitor file -> empty engine list, every row has a competitors:{} object.
ok(Array.isArray(feat.competitorEngines) && feat.competitorEngines.length === 0,
   'no competitor file -> competitorEngines is []');
let sawFeatRow = false;
feat.groups.forEach(function (g) { g.rows.forEach(function (r) {
  if (r.name === 'watdiv_S1_count_us') {
    sawFeatRow = true;
    eq(r.value, 30, 'featured row carries the latest value');
    ok(r.label === 'WatDiv S1 — star query, count', 'featured row uses the sq-ocuf readable label');
    ok(r.competitors && typeof r.competitors === 'object', 'featured row exposes a competitors seam');
  }
}); });
ok(sawFeatRow, 'watdiv row present in featured view');

// competitor seam wiring (sq-t0c3): a competitor file is mapped onto rows; matches by raw NAME
// AND by canonical stem (name minus _us). Absent numbers stay absent (-> "—" in the DOM).
const compFile = {
  engines: [{ id: 'qlever', label: 'QLever', version: '1.2.3', env: 'CI' }, { id: 'oxigraph' }],
  values: { 'watdiv_S1_count_us': { qlever: 12 }, 'lubm_q06_count': { oxigraph: 200 } }
};
const featC = D.buildFeatured(featEntries, compFile);
eq(featC.competitorEngines.length, 2, 'competitor file -> two engine columns');
let watdivComp = null, lubmComp = null;
featC.groups.forEach(function (g) { g.rows.forEach(function (r) {
  if (r.name === 'watdiv_S1_count_us') watdivComp = r.competitors;
  if (r.name === 'lubm_q06_count_us') lubmComp = r.competitors;
}); });
eq(watdivComp && watdivComp.qlever, 12, 'competitor matched by raw metric name');
eq(lubmComp && lubmComp.oxigraph, 200, 'competitor matched by canonical stem (name minus _us)');
ok(watdivComp && watdivComp.oxigraph === undefined, 'unmatched competitor cell stays absent (renders —)');

// ============================================================================================
// [OPUS-4.8] sq-viby — scaling comparison: size/depth axis derived from the metric NAME.
// ============================================================================================
ok(typeof D.sizeAxisOf === 'function', 'dashboard.js exports sizeAxisOf');
ok(typeof D.buildScalingFamilies === 'function', 'dashboard.js exports buildScalingFamilies');

// sizeAxisOf: recognise depth / scale-factor tokens; ignore incidental digits (S1, q06, star3).
const dt = D.sizeAxisOf('deeptax_d10_count_us');
ok(dt && dt.axisLabel === 'depth' && dt.axis === 10, 'sizeAxisOf reads deep-taxonomy depth (_d10 -> 10)');
const dt2 = D.sizeAxisOf('deep_taxonomy_depth20_count_us');
ok(dt2 && dt2.axisLabel === 'depth' && dt2.axis === 20, 'sizeAxisOf reads _depth20 -> 20');
const sf = D.sizeAxisOf('watdiv_sf100_C1_count_us');
ok(sf && sf.axisLabel === 'scale factor' && sf.axis === 100, 'sizeAxisOf reads WatDiv SF (_sf100 -> 100)');
const sfk = D.sizeAxisOf('watdiv_sf1k_C1_count_us');
ok(sfk && sfk.axis === 1000, 'sizeAxisOf honours k multiplier (_sf1k -> 1000)');
ok(D.sizeAxisOf('watdiv_S1_count_us') === null, 'sizeAxisOf ignores S1 (not a size token)');
ok(D.sizeAxisOf('lubm_q06_count_us') === null, 'sizeAxisOf ignores q06 (not a size token)');
ok(D.sizeAxisOf('op_q02_star3_count_us') === null, 'sizeAxisOf ignores star3 (not a size token)');
// two sizes of the same query collapse to ONE base -> one family.
eq(D.sizeAxisOf('deeptax_d1_count_us').base, D.sizeAxisOf('deeptax_d10_count_us').base,
   'different sizes of the same query share a family base');

// buildScalingFamilies: groups by base, points sorted ascending by axis; single-point family kept.
const scaleEntries = [{
  commit: { id: 'abc', message: 's', url: '#' }, date: Date.now(),
  benches: [
    { name: 'deeptax_d1_count_us', value: 10, unit: 'µs' },
    { name: 'deeptax_d10_count_us', value: 120, unit: 'µs' },
    { name: 'deeptax_d5_count_us', value: 40, unit: 'µs' },     // out of order on purpose
    { name: 'watdiv_sf100_C1_count_us', value: 800, unit: 'µs' }, // single-point family
    { name: 'lubm_q06_count_us', value: 99, unit: 'µs' }          // NOT size-parametrised
  ]
}];
const fams = D.buildScalingFamilies(scaleEntries);
ok(fams.length === 2, 'buildScalingFamilies found exactly the two size-parametrised families');
const dtFam = fams.filter(function (f) { return f.axisLabel === 'depth'; })[0];
ok(dtFam && dtFam.points.length === 3, 'deep-taxonomy family has 3 depth points');
ok(dtFam.points[0].axis === 1 && dtFam.points[2].axis === 10, 'scaling points sorted ascending by axis');
const sfFam = fams.filter(function (f) { return f.axisLabel === 'scale factor'; })[0];
ok(sfFam && sfFam.points.length === 1, 'single-size family is kept (renders a single marker + note)');
ok(fams.every(function (f) { return !/lubm_q06/.test(f.base); }), 'non-size-parametrised metric excluded');

// ============================================================================================
// [OPUS-4.8] Copilot review fixes on PR #59 — unit-less labels (#2/#4), clean scaling base (#3),
// and the featured "Metric" column header (#1).
// ============================================================================================
ok(typeof D.labelForBare === 'function', 'dashboard.js exports labelForBare');

// #2/#4 — labelForBare strips ONLY a trailing ` (<unit>)` so the unit isn't doubled when the unit
// has its OWN cell/column. An UNLABELLED metric (humanize fallback) appends `(µs)`/`(s)` to its
// label; labelForBare must remove it. Labelled metrics (sq-ocuf) carry no trailing unit -> unchanged.
ok(/\(µs\)$/.test(D.labelFor('deeptax_d10_count_us')), 'labelFor (fallback) carries a trailing (µs)');
ok(!/\(µs\)$/.test(D.labelForBare('deeptax_d10_count_us')),
   'labelForBare strips the trailing (µs) from an unlabelled metric');
ok(!/\(s\)$/.test(D.labelForBare('totally_new_metric_s')),
   'labelForBare strips a trailing (s) too');
eq(D.labelForBare('watdiv_S1_count_us'), D.labelFor('watdiv_S1_count_us'),
   'labelForBare leaves a labelled (no-trailing-unit) label unchanged');
// mid-label parentheses must survive: only a TRAILING parenthesised token is a unit. lubm_q06's
// readable label mentions "Students" and carries no trailing unit, so it is returned untouched.
eq(D.labelForBare('lubm_q06_count_us'), D.labelFor('lubm_q06_count_us'),
   'labelForBare leaves a labelled metric (no trailing unit) untouched');

// #2 — buildFeatured rows use the unit-less label (Deep Taxonomy is unlabelled -> would double).
const dtFeatEntries = [{
  commit: { id: 'd', message: 'd', url: '#' }, date: Date.now(),
  benches: [{ name: 'deeptax_d10_count_us', value: 120, unit: 'µs' }]
}];
const dtFeat = D.buildFeatured(dtFeatEntries, null);
let dtFeatRow = null;
dtFeat.groups.forEach(function (g) { g.rows.forEach(function (r) {
  if (r.name === 'deeptax_d10_count_us') dtFeatRow = r;
}); });
ok(dtFeatRow, 'featured view surfaces the deep-taxonomy row');
ok(dtFeatRow && !/\(µs\)$/.test(dtFeatRow.label),
   'featured row label has NO trailing unit (unit lives in its own column) — fix #2');

// #4 — scaling family label is unit-less (renderScaling appends the unit to the card title).
const dtScaleFam = D.buildScalingFamilies(scaleEntries).filter(function (f) {
  return f.axisLabel === 'depth';
})[0];
ok(dtScaleFam && !/\(µs\)$/.test(dtScaleFam.label),
   'scaling family label has NO trailing unit (title appends it) — fix #4');

// #3 — sizeAxisOf base is normalized: no double/leading/trailing underscores, even though removing
// the token leaves a placeholder `_` adjacent to existing underscores.
eq(D.sizeAxisOf('deeptax_d10_count_us').base, 'deeptax_count_us',
   'sizeAxisOf base collapses the placeholder underscore (no `deeptax__count_us`) — fix #3');
ok(!/__/.test(D.sizeAxisOf('watdiv_sf100_C1_count_us').base), 'no double underscore in scaling base');
ok(!/^_|_$/.test(D.sizeAxisOf('deeptax_d1_count_us').base), 'no leading/trailing underscore in base');
// the base is still a SHARED key across sizes (fix #3 must not break family grouping).
eq(D.sizeAxisOf('deeptax_d1_count_us').base, D.sizeAxisOf('deeptax_d10_count_us').base,
   'normalized base is still shared across sizes (family grouping intact)');

// ============================================================================================
// [OPUS-4.8] BROWSER-DOM SIMULATION — confirm the featured section + scaling charts actually
// build DOM (sq-xvow #featured / sq-viby #scaling), like sq-ocuf's renderSummary smoke would. A
// tiny stub document/window/Chart lets the (browser-only) render fns run under node: we import
// dashboard.js a SECOND time with its module.exports branch suppressed so its DOM half executes.
// ============================================================================================
(function domSimulation() {
  // --- minimal DOM shim: just enough for el()/renderFeatured()/renderSummary()/render(). -------
  function makeNode(tag) {
    return {
      tagName: tag, children: [], attributes: {}, _text: '', _html: '',
      style: {},
      set textContent(v) { this._text = String(v); }, get textContent() { return this._text; },
      set innerHTML(v) { this._html = String(v); if (v === '') this.children = []; },
      get innerHTML() { return this._html; },
      get lastChild() { return this.children[this.children.length - 1]; },
      get firstChild() { return this.children[0]; },
      setAttribute: function (k, v) { this.attributes[k] = String(v); },
      appendChild: function (c) { this.children.push(c); return c; },
      getContext: function () { return {}; }
    };
  }
  const hosts = {};
  ['updated', 'repo', 'summary', 'charts', 'featured', 'scaling'].forEach(function (id) {
    hosts[id] = makeNode('div');
  });
  global.document = {
    readyState: 'complete',
    createElement: makeNode,
    getElementById: function (id) { return hosts[id] || makeNode('div'); },
    addEventListener: function () {}
  };
  global.window = {
    matchMedia: function () { return { matches: false }; },
    BENCHMARK_DATA: {
      lastUpdate: Date.now(), repoUrl: 'https://github.com/jeswr/sparq',
      entries: { 'sparq engine': [{
        commit: { id: 'deadbeefcafe', message: 'dom sim', url: '#' }, date: Date.now(),
        benches: [
          { name: 'load_s', value: 1.2, unit: 's' },
          { name: 'watdiv_S1_count_us', value: 30, unit: 'µs' },
          { name: 'lubm_q06_count_us', value: 99, unit: 'µs' },
          // size-parametrised family (sq-viby): two depths -> a real scaling curve.
          { name: 'deeptax_d1_count_us', value: 10, unit: 'µs' },
          { name: 'deeptax_d10_count_us', value: 120, unit: 'µs' }
        ]
      }] }
    },
    METRIC_LABELS: labelsFile
  };
  // a no-op Chart constructor so renderCharts doesn't blow up.
  global.Chart = function () { return {}; };

  // re-import dashboard.js with its module.exports branch suppressed so the DOM half runs. The
  // dashboard's IIFE takes the node-export early-return when `module.exports` is truthy; inside this
  // eval scope we shadow `module` to undefined so it falls through to the BROWSER DOM path instead.
  // boot() then runs synchronously: there is no window.fetch in the shim, so it calls render().
  const dashSrc = fs.readFileSync(DASH, 'utf8');
  // eslint-disable-next-line no-eval
  (function (module, exports, require) { eval(dashSrc); })(undefined, undefined, undefined);

  ok(hosts.featured.children.length > 0, 'DOM: #featured populated (sq-xvow rendered)');
  ok(hosts.scaling.children.length > 0, 'DOM: #scaling populated (sq-viby rendered)');
  ok(hosts.summary.children.length > 0, 'DOM: #summary still populated (existing path intact)');
  // featured host should contain at least one suite section with a table.
  const featuredSection = hosts.featured.children[0];
  ok(featuredSection && featuredSection.tagName === 'section', 'DOM: featured host holds suite sections');
  // #1 — the featured table's first column header is "Metric" (not "Query"): the cell may hold
  // non-query metrics. Walk section -> table -> thead -> tr -> first th.
  (function () {
    var table = featuredSection.children.filter(function (c) { return c.tagName === 'table'; })[0];
    var thead = table && table.children.filter(function (c) { return c.tagName === 'thead'; })[0];
    var headRow = thead && thead.children[0];
    var firstTh = headRow && headRow.children[0];
    ok(firstTh && firstTh.textContent === 'Metric',
       'DOM: featured table first column header is "Metric" (fix #1, got ' +
       JSON.stringify(firstTh && firstTh.textContent) + ')');
  })();
  // scaling host should contain a chart-grid with at least one scaling card.
  ok(hosts.scaling.children[0] && hosts.scaling.children[0].attributes['class'] === 'chart-grid',
     'DOM: scaling host holds a chart-grid');

  delete global.document; delete global.window; delete global.Chart;
})();

if (failures) {
  console.error('\nFAILED: ' + failures + ' assertion(s).');
  process.exit(1);
}
console.log('\nAll dashboard smoke checks passed.');
