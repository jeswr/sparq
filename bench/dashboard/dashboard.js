// [OPUS-4.8] sparq benchmark dashboard — vanilla JS, no build step.
// Consumes window.BENCHMARK_DATA (loaded by data.js BEFORE this file) as written by
// github-action-benchmark (tool=customSmallerIsBetter, so for EVERY metric smaller is better).
// Shape: { lastUpdate, repoUrl, entries: { "sparq engine": [ {commit,date,tool,benches:[{name,value,unit,range?}]}, ... ] } }
// The entries array is chronological; the LAST element is the latest commit.
//
// Renders: (1) a latest-commit SUMMARY TABLE grouped by section with a Δ-vs-best pill, and
// (2) per-group trend charts (one Chart.js line per metric). All pure functions live up top so
// they can be unit-smoke-tested under node (see scripts/dashboard-smoke.js).

(function () {
  'use strict';

  var SERIES = 'sparq engine';

  // ---- pure helpers (node-testable) ----------------------------------------

  // Classify a raw metric name into a {group, label} for the summary table + charts.
  // Query/operator latencies:  qNN_<query>_<mode>_us  and  op_<query>_<mode>_us.
  function classify(name) {
    var m = name.match(/^(q\d+|op)_(.+)_(count|materialize|json)_us$/);
    if (m) {
      var fam = m[1] === 'op' ? 'operator' : 'query';
      var query = m[2].replace(/_/g, ' ');
      return { group: fam[0].toUpperCase() + fam.slice(1) + ' latency',
               label: query + ' · ' + m[3] };
    }
    if (/^(store_|dict_)/.test(name) || /_bytes$/.test(name) ||
        name === 'parse_ns_per_byte' || name === 'wasm_bundle_bytes') {
      return { group: 'Memory / Size', label: humanize(name) };
    }
    if (name === 'load_s' || name === 'rdfs_infer_s') {
      return { group: 'Pipeline', label: humanize(name) };
    }
    return { group: 'Other', label: humanize(name) };
  }

  function humanize(name) {
    return name
      .replace(/_us$/, ' (µs)')
      .replace(/_s$/, ' (s)')
      .replace(/_/g, ' ')
      .replace(/\bns per byte\b/, 'ns/byte')
      .trim();
  }

  // Best (minimum, since smaller-is-better) value for a metric over ALL commits in the series.
  function bestOf(entries, name) {
    var best = Infinity;
    for (var i = 0; i < entries.length; i++) {
      var benches = entries[i].benches;
      for (var j = 0; j < benches.length; j++) {
        if (benches[j].name === name && benches[j].value < best) best = benches[j].value;
      }
    }
    return best === Infinity ? null : best;
  }

  // Δ-vs-best descriptor: best (latest is the all-time min) or +x.x% worse.
  function deltaVsBest(latestValue, best) {
    if (best == null || best === 0) {
      return { kind: latestValue === best ? 'best' : 'na', pct: 0 };
    }
    if (latestValue <= best) return { kind: 'best', pct: 0 };
    var pct = (latestValue - best) / best * 100;
    return { kind: 'worse', pct: pct };
  }

  function fmtNum(v) {
    if (v == null) return '—';
    if (v === 0) return '0';
    var abs = Math.abs(v);
    if (abs >= 1000) return v.toLocaleString('en-US', { maximumFractionDigits: 0 });
    if (abs >= 1) return (Math.round(v * 100) / 100).toLocaleString('en-US');
    return (Math.round(v * 10000) / 10000).toString();
  }

  // Build the ordered list of summary rows grouped by section. Stable group order.
  var GROUP_ORDER = ['Pipeline', 'Memory / Size', 'Query latency', 'Operator latency', 'Other'];

  function buildSummary(entries) {
    var latest = entries[entries.length - 1];
    var byGroup = {};
    latest.benches.forEach(function (b) {
      var c = classify(b.name);
      var best = bestOf(entries, b.name);
      var row = {
        name: b.name, label: c.label, value: b.value, unit: b.unit || '',
        best: best, delta: deltaVsBest(b.value, best)
      };
      (byGroup[c.group] = byGroup[c.group] || []).push(row);
    });
    var groups = [];
    GROUP_ORDER.forEach(function (g) { if (byGroup[g]) groups.push({ group: g, rows: sortRows(byGroup[g]) }); });
    // Any group not in GROUP_ORDER (defensive) appended alphabetically.
    Object.keys(byGroup).sort().forEach(function (g) {
      if (GROUP_ORDER.indexOf(g) === -1) groups.push({ group: g, rows: sortRows(byGroup[g]) });
    });
    return { commit: latest.commit, date: latest.date, groups: groups };
  }

  function sortRows(rows) { return rows.sort(function (a, b) { return a.label < b.label ? -1 : a.label > b.label ? 1 : 0; }); }

  // Series of {x:date, y:value} points for a metric across history.
  function trendPoints(entries, name) {
    var pts = [];
    for (var i = 0; i < entries.length; i++) {
      var b = entries[i].benches.find(function (x) { return x.name === name; });
      if (b) pts.push({ x: entries[i].date, y: b.value });
    }
    return pts;
  }

  // Export pure fns for node smoke-test (no effect in the browser).
  if (typeof module !== 'undefined' && module.exports) {
    module.exports = { classify: classify, bestOf: bestOf, deltaVsBest: deltaVsBest,
                       buildSummary: buildSummary, humanize: humanize, fmtNum: fmtNum,
                       trendPoints: trendPoints, GROUP_ORDER: GROUP_ORDER };
    return;
  }

  // ---- DOM rendering (browser only) ----------------------------------------

  function el(tag, attrs, kids) {
    var e = document.createElement(tag);
    if (attrs) Object.keys(attrs).forEach(function (k) {
      if (k === 'text') e.textContent = attrs[k];
      else if (k === 'html') e.innerHTML = attrs[k];
      else e.setAttribute(k, attrs[k]);
    });
    (kids || []).forEach(function (c) { e.appendChild(c); });
    return e;
  }

  function pill(delta) {
    if (delta.kind === 'best') return el('span', { 'class': 'pill pill-best', text: 'best' });
    if (delta.kind === 'na') return el('span', { 'class': 'pill pill-na', text: '—' });
    return el('span', { 'class': 'pill pill-worse',
      text: '+' + (Math.round(delta.pct * 10) / 10).toFixed(1) + '%',
      title: 'vs all-time best' });
  }

  function renderSummary(summary) {
    var host = document.getElementById('summary');
    host.innerHTML = '';
    var c = summary.commit;
    var shortId = c.id.slice(0, 7);
    var msg = (c.message || '').split('\n')[0];
    host.appendChild(el('div', { 'class': 'commit-meta' }, [
      el('a', { 'class': 'sha', href: c.url, target: '_blank', text: shortId }),
      el('span', { 'class': 'msg', text: msg, title: c.message || '' }),
      el('span', { 'class': 'when', text: new Date(summary.date).toLocaleString() })
    ]));

    var table = el('table', { 'class': 'summary-table' });
    var thead = el('thead', null, [el('tr', null, [
      el('th', { text: 'Metric' }), el('th', { 'class': 'num', text: 'Latest' }),
      el('th', { text: 'Unit' }), el('th', { 'class': 'num', text: 'Δ vs best' })
    ])]);
    table.appendChild(thead);
    var tbody = el('tbody');
    summary.groups.forEach(function (g) {
      tbody.appendChild(el('tr', { 'class': 'group-row' }, [
        el('th', { 'class': 'group', colspan: '4', text: g.group })
      ]));
      g.rows.forEach(function (r) {
        var tr = el('tr', null, [
          el('td', { 'class': 'metric', title: r.name, text: r.label }),
          el('td', { 'class': 'num', text: fmtNum(r.value) }),
          el('td', { 'class': 'unit', text: r.unit }),
          el('td', { 'class': 'num delta' })
        ]);
        tr.lastChild.appendChild(pill(r.delta));
        tbody.appendChild(tr);
      });
    });
    table.appendChild(tbody);
    host.appendChild(table);
  }

  var PALETTE = ['#2563eb', '#16a34a', '#dc2626', '#9333ea', '#ea580c', '#0891b2', '#ca8a04', '#db2777'];

  function renderCharts(entries, summary) {
    var host = document.getElementById('charts');
    host.innerHTML = '';
    var darkMode = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
    var grid = darkMode ? 'rgba(255,255,255,0.08)' : 'rgba(0,0,0,0.07)';
    var tick = darkMode ? '#9aa4b2' : '#586069';
    summary.groups.forEach(function (g) {
      var section = el('section', { 'class': 'chart-group' }, [el('h3', { text: g.group })]);
      var gridEl = el('div', { 'class': 'chart-grid' });
      g.rows.forEach(function (r, i) {
        var card = el('div', { 'class': 'chart-card' }, [
          el('div', { 'class': 'chart-title', title: r.name, text: r.label + ' (' + r.unit + ')' })
        ]);
        var canvas = el('canvas');
        card.appendChild(canvas);
        gridEl.appendChild(card);
        var color = PALETTE[i % PALETTE.length];
        var pts = trendPoints(entries, r.name);
        // eslint-disable-next-line no-new
        new Chart(canvas.getContext('2d'), {
          type: 'line',
          data: { datasets: [{
            label: r.label, data: pts, borderColor: color,
            backgroundColor: color + '22', borderWidth: 2, pointRadius: 0,
            pointHoverRadius: 4, fill: true, lineTension: 0.2
          }] },
          options: {
            responsive: true, maintainAspectRatio: false,
            legend: { display: false },
            tooltips: {
              mode: 'index', intersect: false,
              callbacks: {
                title: function (items) { return new Date(items[0].xLabel).toLocaleDateString(); },
                label: function (item) { return fmtNum(item.yLabel) + ' ' + r.unit; }
              }
            },
            scales: {
              xAxes: [{ type: 'time', time: { tooltipFormat: 'll' },
                        gridLines: { color: grid }, ticks: { fontColor: tick, maxTicksLimit: 4 } }],
              yAxes: [{ gridLines: { color: grid },
                        ticks: { fontColor: tick, maxTicksLimit: 4, callback: function (v) { return fmtNum(v); } } }]
            }
          }
        });
      });
      section.appendChild(gridEl);
      host.appendChild(section);
    });
  }

  function boot() {
    var data = window.BENCHMARK_DATA;
    if (!data || !data.entries || !data.entries[SERIES] || !data.entries[SERIES].length) {
      document.getElementById('summary').textContent =
        'No benchmark data yet — the chart populates after the first push to main.';
      return;
    }
    var entries = data.entries[SERIES];
    var summary = buildSummary(entries);
    document.getElementById('updated').textContent =
      'last updated ' + new Date(data.lastUpdate).toLocaleString() +
      ' · ' + entries.length + ' commits';
    var repo = el('a', { href: data.repoUrl, target: '_blank', text: data.repoUrl.replace(/^https?:\/\//, '') });
    document.getElementById('repo').appendChild(repo);
    renderSummary(summary);
    if (typeof Chart !== 'undefined') renderCharts(entries, summary);
  }

  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', boot);
  else boot();
})();
