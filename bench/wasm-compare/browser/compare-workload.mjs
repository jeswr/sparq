// [FABLE-5] sq-hmd7l.17 — cross-LIBRARY comparison workload, layered on the
// sq-3ul2n.1 harness (NOT a second harness): the dataset generators, query
// shapes and expected-row-count oracles are imported from ./workload.mjs
// unchanged, so every library answers the SAME deterministic workload the
// sparq-only harness measures.
//
// Runs UNCHANGED in a browser page (page/compare.mjs) and in plain Node
// (compare.mjs --child): no `node:` imports, timing via `performance.now()`.
//
// INVARIANT (bead sq-hmd7l.17): no latency row is emitted without row-count
// agreement — every query row is oracle-checked against the deterministic
// expected count BEFORE its timing is recorded, and the orchestrator
// re-checks counts ACROSS libraries at report time. A mismatch throws
// (run fails; nothing fabricated).
//
// Every `ms` is ADVISORY / NON-CANONICAL — measured on whatever host runs it.

import { genTurtle, genNTriples, roundTo, QUERIES, DEFAULT_SIZES } from "./workload.mjs";

/** Competitor npm pins (gather-only installs — never committed dependencies). */
export const COMPETITOR_PINS = {
  oxigraph: "0.5.9",
  n3: "2.1.1",
  quadstore: "15.4.1",
  "quadstore-comunica": "6.3.1",
  "memory-level": "3.1.0",
};

export const INSTALL_HINT =
  "npm install --no-save " +
  Object.entries(COMPETITOR_PINS)
    .map(([p, v]) => `${p}@${v}`)
    .join(" ");

const now = () => performance.now();

/**
 * Async analogue of workload.mjs `measure()`: one measured FIRST call (cold),
 * `warmup` unmeasured calls, then `iters` measured samples taking the BEST,
 * batching fast calls so coarse timers still resolve them. The per-call
 * `await` overhead is identical for every library (all adapters are async),
 * so cross-library ratios are unaffected.
 */
export async function measureAsync(fn, { warmup = 2, iters = 5, minSampleMs = 20 } = {}) {
  const t0 = now();
  let out = await fn();
  const first = now() - t0;
  for (let i = 0; i < warmup; i++) await fn();
  const c0 = now();
  await fn();
  const oneMs = now() - c0;
  const batch =
    oneMs >= minSampleMs ? 1 : Math.min(1000, Math.max(1, Math.ceil(minSampleMs / Math.max(oneMs, 0.0001))));
  let best = Infinity;
  for (let i = 0; i < iters; i++) {
    const s = now();
    for (let b = 0; b < batch; b++) out = await fn();
    const per = (now() - s) / batch;
    if (per < best) best = per;
  }
  return { first, warm: best, iters, batch, out };
}

function check(cond, msg) {
  if (!cond) throw new Error(`compare oracle failed: ${msg}`);
}

/**
 * Runs the reduced cross-library workload against ONE library adapter:
 * text→queryable-store load (N-Triples + Turtle, fresh store per iteration)
 * at the compare tier, then the five harness query shapes cold+warm on the
 * Turtle store, each oracle-checked before its timing row is emitted.
 *
 * The adapter surface (all methods may be sync or async):
 *   newStore() → store handle
 *   load(store, text, format)   format ∈ "ntriples" | "turtle"
 *   size(store) → number        (may scan; called OUTSIDE timed sections)
 *   queryCount(store, sparql) → number of SELECT rows
 *   free?(store)
 *
 * @returns {{ rows: object[], skipped: object[] }}
 */
export async function runCompareWorkload({ adapter, quick = false, tier, log = () => {} }) {
  const rows = [];
  const skipped = [];
  const queryTier = tier ?? (quick ? DEFAULT_SIZES[0] : 100_000);
  const loadIters = quick ? 1 : 2;
  const qOpts = quick ? { warmup: 1, iters: 3 } : { warmup: 2, iters: 7 };

  // ---- Phase: store load (text → queryable store), fresh store per iter. ----
  let queryStore;
  let queryEntities = 0;
  for (const [format, gen, mult] of [
    ["ntriples", genNTriples, 7],
    ["turtle", genTurtle, 5],
  ]) {
    const size = roundTo(queryTier, mult);
    const { text, entities, triples } = gen(size);
    log(`load ${format} ${triples} triples (${(text.length / 1e6).toFixed(1)} MB source)…`);
    let store;
    const m = await measureAsync(
      async () => {
        if (store) adapter.free?.(store); // superseded build — only the LAST store survives
        store = await adapter.newStore();
        await adapter.load(store, text, format);
        return store;
      },
      { warmup: 0, iters: loadIters, minSampleMs: 0 }, // loads are ms-heavy: no batching, rebuild per iter
    );
    const got = await adapter.size(store);
    check(got === triples, `${adapter.library} ${format}@${triples}: store size=${got}`);
    rows.push({ phase: "store_load", format, triples, entities, source_bytes: text.length, kind: "first", ms: m.first, iters: 1 });
    rows.push({ phase: "store_load", format, triples, entities, source_bytes: text.length, kind: "warm", ms: m.warm, iters: loadIters });
    if (format === "turtle") {
      queryStore = store;
      queryEntities = entities;
    } else {
      adapter.free?.(store);
    }
  }
  check(queryStore, "no query store captured");
  log(`query phases on turtle store (n=${queryEntities})`);

  // ---- Phase: query shapes — oracle-checked BEFORE any timing row. ----
  for (const q of QUERIES) {
    const expected = q.expected(queryEntities);
    let got = -1;
    const m = await measureAsync(async () => {
      got = await adapter.queryCount(queryStore, q.sparql);
      return got;
    }, qOpts);
    check(got === expected, `${adapter.library} ${q.name}: rows=${got}, expected ${expected}`);
    rows.push({ phase: "query", query: q.name, rows: got, kind: "first", ms: m.first, iters: 1 });
    rows.push({ phase: "query", query: q.name, rows: got, kind: "warm", ms: m.warm, iters: m.iters, batch: m.batch });
    log(`query ${q.name}: ${got} rows, first ${m.first.toFixed(2)}ms, warm ${m.warm.toFixed(3)}ms`);
  }

  adapter.free?.(queryStore);
  return { rows, skipped };
}

/**
 * [sq-hmd7l.40] OPT-IN corpus mode: runs a well-known-suite corpus (SP2Bench /
 * WatDiv at the native suite's FIXED per-commit tier) against ONE library
 * adapter. Strictly additive — the default workload above is untouched.
 *
 * `corpus` is the descriptor from corpus.mjs `loadCorpusSpec` (node) or the
 * served `/corpus/<name>.json` (browser page): `{ name, format, text,
 * queries: [{ name, sparql, expected, ask }] }`, where every `expected` comes
 * verbatim from the suite's native expected-rows.tsv — the SAME file the
 * native ci-bench equality check gates on.
 *
 * INVARIANT (unchanged from the default workload): no latency row without
 * row-count agreement — every query row is checked against the native
 * expected count BEFORE its timing rows are emitted (ASK reports 1/0 in
 * count mode, per the tsv header). The corpus STORE SIZE has no per-library
 * absolute oracle (the native source pins query counts, not the deduplicated
 * triple count), so it is emitted in `rows` for the orchestrator's
 * cross-library agreement check instead.
 *
 * @returns {{ rows: object[], skipped: object[] }}
 */
export async function runCorpusWorkload({ adapter, corpus, quick = false, log = () => {} }) {
  const rows = [];
  const skipped = [];
  const loadIters = quick ? 1 : 2;
  const qOpts = quick ? { warmup: 0, iters: 2 } : { warmup: 1, iters: 5 };
  const label = `${corpus.name}/${corpus.format}`;

  // ---- Phase: corpus load (text → queryable store), fresh store per iter. ----
  log(`load corpus ${label} (${(corpus.text.length / 1e6).toFixed(1)} MB source)…`);
  let store;
  const m = await measureAsync(
    async () => {
      if (store) adapter.free?.(store); // superseded build — only the LAST store survives
      store = await adapter.newStore();
      await adapter.load(store, corpus.text, corpus.format);
      return store;
    },
    { warmup: 0, iters: loadIters, minSampleMs: 0 },
  );
  const size = await adapter.size(store);
  check(size > 0, `${adapter.library} ${label}: store is empty after load`);
  rows.push({ phase: "corpus_load", format: label, rows: size, source_bytes: corpus.text.length, kind: "first", ms: m.first, iters: 1 });
  rows.push({ phase: "corpus_load", format: label, rows: size, source_bytes: corpus.text.length, kind: "warm", ms: m.warm, iters: loadIters });
  log(`corpus store: ${size} triples`);

  // ---- Phase: the suite's queries — native-tsv-checked BEFORE any timing row. ----
  for (const q of corpus.queries) {
    if (q.ask && !adapter.queryAsk) {
      skipped.push({ phase: "corpus_query", query: `${corpus.name}/${q.name}`, reason: `${adapter.library}: no ASK support wired` });
      continue;
    }
    let got = -1;
    const mq = await measureAsync(async () => {
      got = q.ask ? ((await adapter.queryAsk(store, q.sparql)) ? 1 : 0) : await adapter.queryCount(store, q.sparql);
      return got;
    }, qOpts);
    check(got === q.expected, `${adapter.library} ${corpus.name}/${q.name}: rows=${got}, expected ${q.expected} (native expected-rows.tsv)`);
    rows.push({ phase: "corpus_query", query: `${corpus.name}/${q.name}`, rows: got, kind: "first", ms: mq.first, iters: 1 });
    rows.push({ phase: "corpus_query", query: `${corpus.name}/${q.name}`, rows: got, kind: "warm", ms: mq.warm, iters: mq.iters, batch: mq.batch });
    log(`query ${q.name}: ${got} rows, first ${mq.first.toFixed(2)}ms, warm ${mq.warm.toFixed(3)}ms`);
  }

  adapter.free?.(store);
  return { rows, skipped };
}
