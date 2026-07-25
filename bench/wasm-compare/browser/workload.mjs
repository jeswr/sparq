// [FABLE-5] sq-3ul2n.1 — shared workload for the cross-engine browser wasm harness.
//
// Runs UNCHANGED in a browser page (page/page.mjs) and in plain Node
// (node-baseline.mjs): no `node:` imports, timing via `performance.now()`
// (present in both), plain `throw` instead of `node:assert`.
//
// The dataset generators are ports of the ones in
// `crates/sparq-wasm/test/bench.cjs` (buildTurtle) and
// `crates/sparq-wasm/test/mem.cjs` (genNT) — reused READ-ONLY per the bead
// spec (those files are CommonJS and cannot be imported from a browser page,
// so the generator bodies are reproduced here with their source cited).
// Both are deterministic: byte-identical output for a given size, so triple
// counts and query row counts are exact cross-engine oracles.
//
// Every `ms` this module produces is ADVISORY / NON-CANONICAL — measured on
// whatever host runs it. The deterministic gate is the row/triple-count
// oracle; the timings attribute cost to a LAYER, they are not headlines.

/** Workload tiers, in TRIPLES (the bead pins 25k/100k/300k). */
export const DEFAULT_SIZES = [25_000, 100_000, 300_000];

/**
 * Turtle generator — port of `buildTurtle` in crates/sparq-wasm/test/bench.cjs
 * (5 triples/entity: name, age, city, follows×2 — chains + cycles for WCOJ,
 * star attributes, a numeric column for FILTER). Sized by TRIPLE count;
 * `triples` must be a multiple of 5. All 5·n triples are distinct (the two
 * follows targets differ for n > 6), so `store.size === triples` exactly.
 */
export function genTurtle(triples) {
  if (triples % 5 !== 0) throw new Error(`genTurtle: ${triples} not a multiple of 5`);
  const n = triples / 5;
  const cities = ["Paris", "Berlin", "Tokyo", "Lima", "Cairo"];
  const lines = ["@prefix ex: <http://ex/> ."];
  for (let i = 0; i < n; i++) {
    const age = 18 + (i % 60);
    lines.push(
      `ex:p${i} ex:name "Person ${i}" ; ex:age ${age} ; ex:city "${cities[i % cities.length]}" ; ` +
        `ex:follows ex:p${(i + 1) % n} ; ex:follows ex:p${(i + 7) % n} .`,
    );
  }
  return { text: lines.join("\n"), entities: n, triples };
}

/**
 * N-Triples generator — port of `genNT` in crates/sparq-wasm/test/mem.cjs
 * (7 triples/entity: type, name, age(typed int), follows×4 — exercises the
 * custom byte-level N-Triples parser). Sized by TRIPLE count; `triples` must
 * be a multiple of 7. The 4 follows targets (i·7+k mod n, k=0..3) are
 * distinct for n > 4, so `store.size === triples` exactly.
 */
export function genNTriples(triples) {
  if (triples % 7 !== 0) throw new Error(`genNTriples: ${triples} not a multiple of 7`);
  const n = triples / 7;
  const T = "<http://www.w3.org/1999/02/22-rdf-syntax-ns#type>";
  const XSDINT = "<http://www.w3.org/2001/XMLSchema#integer>";
  const parts = [];
  for (let i = 0; i < n; i++) {
    const s = `<http://ex/n${i}>`;
    parts.push(`${s} ${T} <http://ex/Person> .\n`);
    parts.push(`${s} <http://ex/name> "name${i}" .\n`);
    parts.push(`${s} <http://ex/age> "${20 + (i % 80)}"^^${XSDINT} .\n`);
    for (let k = 0; k < 4; k++) parts.push(`${s} <http://ex/follows> <http://ex/n${(i * 7 + k) % n}> .\n`);
  }
  return { text: parts.join(""), entities: n, triples };
}

/** Nearest multiple of `m` at or above `t` — so "25k triples" is exact per format. */
export function roundTo(t, m) {
  return Math.ceil(t / m) * m;
}

/**
 * The five query shapes from crates/sparq-wasm/test/bench.cjs QUERIES
 * (reused read-only), with an exact expected-row-count formula per shape as a
 * cross-engine correctness oracle. `n` = entity count of the turtle store the
 * queries run against.
 */
export const QUERIES = [
  {
    name: "scan-type",
    sparql: `PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:name ?n }`,
    expected: (n) => n,
  },
  {
    name: "star-3",
    sparql: `PREFIX ex: <http://ex/> SELECT ?s ?n ?a ?c WHERE { ?s ex:name ?n . ?s ex:age ?a . ?s ex:city ?c }`,
    expected: (n) => n,
  },
  {
    name: "chain-2",
    sparql: `PREFIX ex: <http://ex/> SELECT ?a ?b ?c WHERE { ?a ex:follows ?b . ?b ex:follows ?c }`,
    // out-degree 2 and in-degree 2 per node → Σ_b in(b)·out(b) = 4n.
    expected: (n) => 4 * n,
  },
  {
    name: "triangle-wcoj",
    sparql: `PREFIX ex: <http://ex/> SELECT ?a ?b ?c WHERE { ?a ex:follows ?b . ?b ex:follows ?c . ?c ex:follows ?a }`,
    // step sums from {1,7}³ are 3/9/15/21, none ≡ 0 mod n for the harness
    // sizes → 0 triangles (still exercises the WCOJ plan).
    expected: (n) => 0,
  },
  {
    name: "filter-age",
    sparql: `PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:age ?a . FILTER(?a > 50) }`,
    // ages are 18+(i%60) ∈ 18..77; >50 keeps residues 33..59. Counted exactly
    // (n need not divide 60), mirroring the generator.
    expected: (n) => {
      let c = 0;
      for (let i = 0; i < n; i++) if (18 + (i % 60) > 50) c++;
      return c;
    },
  },
];

/** CONSTRUCT for the serialization-out phase (N-Triples emitted from wasm). */
export const CONSTRUCT_QUERY = `PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:name ?n } WHERE { ?s ex:name ?n }`;

/**
 * Boundary-marshalling probe set (the #1885 phase 7): the SAME logical
 * pattern crossed through hand-offs of increasing JS-side materialisation —
 * ask (boolean, near-zero marshal floor) → count (usize) → query string out
 * (one JSON string copy, unparsed) → query + JSON.parse → queryChunks
 * (chunked string drain) → JS-wrapper queryBindings (JSON.parse + one
 * Bindings Map per row + one Term per cell). Adjacent deltas attribute the
 * boundary copy, the JSON.parse, and the RDF/JS materialisation separately.
 * NOTE the interpretation caveats: `ask` short-circuits at the first row and
 * `count` uses the engine's lazy-count paths, so both under-run the eval the
 * `query` variants pay — they are floors, not same-eval controls. The
 * apples-to-apples marshalling deltas are query vs queryChunks vs
 * wrapper-queryBindings (identical wasm-side eval + serialisation).
 */
export const MARSHAL_QUERIES = [
  {
    name: "scan-type",
    select: `PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:name ?n }`,
    ask: `PREFIX ex: <http://ex/> ASK { ?s ex:name ?n }`,
    expected: (n) => n,
  },
  {
    name: "star-3",
    select: `PREFIX ex: <http://ex/> SELECT ?s ?n ?a ?c WHERE { ?s ex:name ?n . ?s ex:age ?a . ?s ex:city ?c }`,
    ask: `PREFIX ex: <http://ex/> ASK { ?s ex:name ?n . ?s ex:age ?a . ?s ex:city ?c }`,
    expected: (n) => n,
  },
];

const now = () => performance.now();

/**
 * Times `fn`: one measured FIRST call (cold — carries any remaining JIT
 * tier-up / lazy init), then `warmup` unmeasured calls, then `iters` measured
 * calls taking the BEST. Fast calls are batched so coarse browser timers
 * (1ms-class under anti-fingerprinting clamps) still resolve them: if a call
 * runs under `minSampleMs`, each sample loops the call enough times to exceed
 * it and divides.
 */
export function measure(fn, { warmup = 2, iters = 5, minSampleMs = 20 } = {}) {
  const t0 = now();
  let out = fn();
  const first = now() - t0;
  for (let i = 0; i < warmup; i++) fn();
  // Calibrate a batch size from one post-warmup call.
  const c0 = now();
  fn();
  const oneMs = now() - c0;
  const batch = oneMs >= minSampleMs ? 1 : Math.min(1000, Math.max(1, Math.ceil(minSampleMs / Math.max(oneMs, 0.0001))));
  let best = Infinity;
  for (let i = 0; i < iters; i++) {
    const s = now();
    for (let b = 0; b < batch; b++) out = fn();
    const per = (now() - s) / batch;
    if (per < best) best = per;
  }
  return { first, warm: best, iters, batch, out };
}

function check(cond, msg) {
  if (!cond) throw new Error(`workload oracle failed: ${msg}`);
}

/**
 * Runs the full per-phase workload against an ALREADY-INSTANTIATED wasm
 * module. Init phases (fetch/compile/instantiate) are environment-specific
 * and measured by the caller; everything from store-load onward is identical
 * across browser pages and Node.
 *
 * @param {object} opts
 * @param {any} opts.Store        raw wasm-bindgen `Store` class (js/wasm/sparq_wasm.js)
 * @param {any} [opts.SparqStore] JS-wrapper class (js/dist/index.js); wrapper
 *                                phases are skipped-with-notice if absent
 * @param {number[]} [opts.sizes] workload tiers in triples
 * @param {boolean} [opts.quick]  smoke mode: smallest tier, fewer iters
 * @param {(msg: string) => void} [opts.log]
 * @returns {{ rows: object[], skipped: object[] }}
 */
export function runWorkload({ Store, SparqStore, sizes = DEFAULT_SIZES, quick = false, log = () => {} }) {
  const rows = [];
  const skipped = [];
  const tiers = quick ? [sizes[0]] : sizes;
  const loadIters = (t) => (quick ? 1 : t >= 300_000 ? 2 : 3);
  const qOpts = quick ? { warmup: 1, iters: 3 } : { warmup: 2, iters: 7 };

  // ---- Phase: store load/parse (N-Triples then Turtle, ascending sizes). ----
  // The very first load is the globally coldest call (parser tier-up); order
  // is fixed and documented so cold rows are comparable across engines.
  let queryStore; // turtle store the query phases run on (largest quick tier / 100k default)
  let queryEntities = 0;
  let queryTurtleText = "";
  const queryTier = quick ? tiers[0] : 100_000;
  for (const [format, gen, mult] of [
    ["ntriples", genNTriples, 7],
    ["turtle", genTurtle, 5],
  ]) {
    for (const t of tiers) {
      const size = roundTo(t, mult);
      const { text, entities, triples } = gen(size);
      log(`load ${format} ${triples} triples (${entities} entities, ${(text.length / 1e6).toFixed(1)} MB source)…`);
      let store;
      const iters = loadIters(t);
      const m = measure(
        () => {
          store = Store.load(text, format === "ntriples" ? "ntriples" : "turtle");
          return store;
        },
        { warmup: 0, iters, minSampleMs: 0 }, // loads are ms-heavy: no batching, rebuild per iter
      );
      check(store.size === triples, `${format}@${triples}: store.size=${store.size}`);
      const heap = store.heapBytes();
      rows.push({ phase: "store_load", format, triples, entities, source_bytes: text.length, kind: "first", ms: m.first, iters: 1, heap_bytes: heap });
      rows.push({ phase: "store_load", format, triples, entities, source_bytes: text.length, kind: "warm", ms: m.warm, iters, heap_bytes: heap });
      if (format === "turtle" && t === queryTier) {
        queryStore = store;
        queryEntities = entities;
        queryTurtleText = text;
      }
    }
  }
  check(queryStore, `no query store captured for tier ${queryTier}`);
  log(`query phases on turtle store: ${queryStore.size} triples, n=${queryEntities}`);

  // ---- Phase: query shapes (first = tier-up-inclusive cold, then warm best). ----
  for (const q of QUERIES) {
    const expected = q.expected(queryEntities);
    let got = -1;
    const m = measure(() => {
      const parsed = JSON.parse(queryStore.query(q.sparql));
      got = parsed.results.bindings.length;
      return got;
    }, qOpts);
    check(got === expected, `${q.name}: rows=${got}, expected ${expected}`);
    rows.push({ phase: "query", query: q.name, rows: got, kind: "first", ms: m.first, iters: 1 });
    rows.push({ phase: "query", query: q.name, rows: got, kind: "warm", ms: m.warm, iters: m.iters, batch: m.batch });
    log(`query ${q.name}: ${got} rows, first ${m.first.toFixed(2)}ms, warm ${m.warm.toFixed(3)}ms`);
  }

  // ---- Phase: serialization out (CONSTRUCT → N-Triples string from wasm). ----
  {
    let outLen = 0;
    const m = measure(() => {
      const nt = queryStore.queryQuads(CONSTRUCT_QUERY);
      outLen = nt.length;
      return outLen;
    }, qOpts);
    // One triple per entity; every line ends "\n".
    const lines = queryStore.queryQuads(CONSTRUCT_QUERY).split("\n").filter((l) => l.length > 0).length;
    check(lines === queryEntities, `serialize_construct: ${lines} triples out, expected ${queryEntities}`);
    rows.push({ phase: "serialize_construct", query: "construct-name", rows: lines, out_bytes: outLen, kind: "first", ms: m.first, iters: 1 });
    rows.push({ phase: "serialize_construct", query: "construct-name", rows: lines, out_bytes: outLen, kind: "warm", ms: m.warm, iters: m.iters, batch: m.batch });
  }

  // ---- Phase: boundary-marshalling share (ask / count / query / queryChunks / wrapper). ----
  let wrapperStore;
  if (SparqStore) {
    // fromStringSync: the wasm engine is already instantiated by the caller,
    // so the wrapper must NOT re-init (a second initWasm would re-instantiate
    // and orphan the raw stores above).
    wrapperStore = SparqStore.fromStringSync(queryTurtleText, "turtle");
    check(wrapperStore.size === queryStore.size, `wrapper store size ${wrapperStore.size} != ${queryStore.size}`);
  } else {
    skipped.push({ phase: "marshal_wrapper_bindings", reason: "js/dist wrapper not provided in this environment" });
  }
  for (const mq of MARSHAL_QUERIES) {
    const expected = mq.expected(queryEntities);
    const variants = [
      ["marshal_ask", () => queryStore.ask(mq.ask), (v) => v === true],
      ["marshal_count", () => queryStore.count(mq.select), (v) => v === expected],
      // raw boundary hand-off: wasm-side eval + JSON serialisation + ONE string copy out
      ["marshal_query_string", () => queryStore.query(mq.select).length, (v) => v > 0],
      // + the JS-side JSON.parse of that string (the wrapper's first layer)
      [
        "marshal_query_json_parse",
        () => JSON.parse(queryStore.query(mq.select)).results.bindings.length,
        (v) => v === expected,
      ],
      [
        "marshal_query_chunks",
        () => {
          const cur = queryStore.queryChunks(mq.select);
          let bytes = 0;
          try {
            for (;;) {
              const chunk = cur.next();
              if (chunk === undefined) break;
              bytes += chunk.length;
            }
          } finally {
            cur.free?.();
          }
          return bytes;
        },
        (v) => v > 0,
      ],
    ];
    if (wrapperStore) {
      variants.push(["marshal_wrapper_bindings", () => wrapperStore.queryBindings(mq.select).length, (v) => v === expected]);
    }
    for (const [phase, fn, ok] of variants) {
      const m = measure(fn, qOpts);
      check(ok(m.out), `${phase}/${mq.name}: unexpected result ${m.out}`);
      rows.push({ phase, query: mq.name, rows: expected, kind: "warm", ms: m.warm, iters: m.iters, batch: m.batch });
      log(`${phase} ${mq.name}: warm ${m.warm.toFixed(3)}ms`);
    }
  }

  return { rows, skipped };
}
