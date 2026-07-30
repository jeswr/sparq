// [SONNET-4.6] (sq-rpdae) RSP4J/csparql2 replay driver — the full-SPARQL (Jena/Esper)
// arm of the bounded count-matched-replay RSP comparability protocol
// (research/comparative-benchmarking-everything.md sec 5.2).
//
// WHY THIS EXISTS: sq-hmd7l.20 could count-match only `srbench_join`, because YASPER's
// TPQueryFactory dialect has no GROUP BY. RSP4J's csparql2 module carries a full-SPARQL
// R2R operator (R2ROperatorSPARQL: Jena ARQ over an Esper-materialised SDS) that CAN
// express the aggregate scenarios, so this driver evaluates whether it widens the
// count-comparable surface. It does NOT — see the verdict below and
// research/gap-rsp-2026-07.md sec "csparql2 evaluation". The driver is kept as the
// REPRODUCTION artifact for that verdict, exactly like Rsp4jReplayRunner.java is for
// the YASPER leg; it is gather-time only and never compiled by CI.
//
// VERDICT (evaluated 2026-07-26 against streamreasoning/rsp4j @ c46e0f674, csparql2
// module version 2.0.0, Esper 7.1.0; non-canonical work box, counts are
// machine-independent):
//
//   1. BLOCKER — csparql2 SELF-DEADLOCKS under an externally driven clock.
//      EsperGGWindowOperator.EsperGGWindowAssigner.getContent() calls
//      statement.safeIterator() and never close()s it, so the Esper statement's
//      ReentrantReadWriteLock READ lock is leaked on the first window materialisation.
//      The next CurrentTimeEvent (clock advance) needs the WRITE lock on the same
//      thread and parks forever. Reproduced via jcmd Thread.print; removed by a
//      one-line `finally { iterator.close(); }` upstream. gather-csparql2.sh applies
//      that patch EXPLICITLY and records it in the envelope — without it this runner
//      hangs, it does not fail.
//
//   2. The MULTI-WINDOW scenarios (srbench_join, srbench_groupby_state) emit ZERO
//      rows in every flag combination tried. The two named windows report at
//      DIFFERENT event times (e.g. wm at 10000/20000/30001, wo at 12000/21000/35000),
//      so no synchronized two-window snapshot ever exists when the R2R operator runs,
//      and the join yields no solution. NOT-COUNT-COMPARABLE.
//
//   3. The SINGLE-WINDOW aggregate scenarios DO run once patched, and under
//      `--ts-offset 1 --non-empty-content false` the count-match gate scores
//      tumbling_avg 5/5 and sliding_sum 5/5. That agreement is NOT admissible as a
//      widened comparable surface, because it is WINDOW-ALIGNMENT-CONTINGENT:
//      csparql2's S2R is an Esper SLIDING `win:time` snapshotted at whatever
//      external-clock value the driver advanced to when it crossed the boundary —
//      `(T-omega, T]` — not the oracle's aligned `[k*step, k*step+range)`.
//      tumbling_groupby_join is the WITNESS: its w0 scores 1 against the oracle's 2
//      because the room triple at the window's left edge (ts 0) has already expired at
//      snapshot time. Landing the heartbeat exactly on the boundary instead makes
//      reporting worse (reports drift to element-arrival times). So the two "matches"
//      hold for this replay's event placement, not for the window semantics — a
//      left-edge-sensitive replay breaks them. gather-csparql2.sh therefore attaches a
//      machine-readable protocol caveat to every emitted row.
//
// Contract: emits `report\t<t_e>\t<distinct_rows>` per window evaluation on stdout,
// consumed by bench/rsp/rsp4j_compare.py exactly like the YASPER runner's output.
//
// Usage:
//   java Csparql2ReplayRunner --replay bench/rsp/replay/single_window.ts.tsv \
//        --scenario sliding_sum [--time-scale 1000] [--ts-offset 1] \
//        [--non-empty-content false] [--version <build-id>]

import org.apache.jena.datatypes.TypeMapper;
import org.apache.jena.graph.Graph;
import org.apache.jena.graph.Node;
import org.apache.jena.graph.NodeFactory;
import org.apache.jena.graph.Triple;
import org.apache.jena.sparql.algebra.Table;
import org.apache.jena.sparql.core.Var;
import org.apache.jena.sparql.engine.binding.Binding;
import org.apache.jena.sparql.graph.GraphFactory;
import org.streamreasoning.rsp4j.api.engine.config.EngineConfiguration;
import org.streamreasoning.rsp4j.api.querying.ContinuousQuery;
import org.streamreasoning.rsp4j.api.sds.SDSConfiguration;
import org.streamreasoning.rsp4j.api.stream.data.DataStream;
import org.streamreasoning.rsp4j.csparql2.engine.CSPARQLEngine;
import org.streamreasoning.rsp4j.csparql2.engine.JenaContinuousQueryExecution;
import org.streamreasoning.rsp4j.io.DataStreamImpl;

import java.io.BufferedReader;
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;

public final class Csparql2ReplayRunner {

    private static final String HB_P = "http://ex/heartbeat";
    private static final String SINGLE_STREAM = "http://ex/stream";

    /** One replay event: (scaled ts, stream iri, s, p, o as N-Triples-ish strings). */
    private static final class Event {
        final long ts;
        final String stream;
        final String s;
        final String p;
        final String o;

        Event(long ts, String stream, String s, String p, String o) {
            this.ts = ts;
            this.stream = stream;
            this.s = s;
            this.p = p;
            this.o = o;
        }
    }

    /** Scenario config: RSP-QL text + window maths for heartbeat placement. */
    private static final class Scenario {
        final String rspql;
        final long range;
        final long step;
        final String[] streams;

        Scenario(String rspql, long range, long step, String[] streams) {
            this.rspql = rspql;
            this.range = range;
            this.step = step;
            this.streams = streams;
        }
    }

    /**
     * The five oracle scenarios, transcribed into csparql2's RSP-QL surface. The window
     * specs and graph patterns mirror crates/sparq-rsp/examples/rsp_oracle.rs; the
     * aggregate ones are the whole point of the csparql2 arm (YASPER's TP dialect has
     * no GROUP BY). The two SRBench scenarios are retained so the ZERO-output
     * multi-window finding stays reproducible.
     */
    private static Scenario scenario(String name) {
        switch (name) {
            case "srbench_join":
                return new Scenario(
                    "REGISTER RSTREAM <http://ex/out> AS "
                        + "SELECT ?st ?state ?v "
                        + "FROM NAMED WINDOW <http://ex/wo> ON <http://ex/obs> [RANGE PT10S STEP PT10S] "
                        + "FROM NAMED WINDOW <http://ex/wm> ON <http://ex/meta> [RANGE PT10S STEP PT10S] "
                        + "WHERE { "
                        + "  WINDOW <http://ex/wo> { ?st <http://ex/value> ?v . } "
                        + "  WINDOW <http://ex/wm> { ?st <http://ex/state> ?state . } "
                        + "}",
                    10, 10, new String[] {"http://ex/obs", "http://ex/meta"});
            case "srbench_groupby_state":
                return new Scenario(
                    "REGISTER RSTREAM <http://ex/out> AS "
                        + "SELECT ?state (COUNT(?v) AS ?n) "
                        + "FROM NAMED WINDOW <http://ex/wo> ON <http://ex/obs> [RANGE PT10S STEP PT10S] "
                        + "FROM NAMED WINDOW <http://ex/wm> ON <http://ex/meta> [RANGE PT10S STEP PT10S] "
                        + "WHERE { "
                        + "  WINDOW <http://ex/wo> { ?st <http://ex/value> ?v . } "
                        + "  WINDOW <http://ex/wm> { ?st <http://ex/state> ?state . } "
                        + "} GROUP BY ?state",
                    10, 10, new String[] {"http://ex/obs", "http://ex/meta"});
            case "tumbling_avg":
                return new Scenario(
                    "REGISTER RSTREAM <http://ex/out> AS "
                        + "SELECT (AVG(?v) AS ?avg) "
                        + "FROM NAMED WINDOW <http://ex/w> ON <" + SINGLE_STREAM + "> [RANGE PT10S STEP PT10S] "
                        + "WHERE { WINDOW <http://ex/w> { ?s <http://ex/value> ?v . } }",
                    10, 10, new String[] {SINGLE_STREAM});
            case "sliding_sum":
                return new Scenario(
                    "REGISTER RSTREAM <http://ex/out> AS "
                        + "SELECT ?s (SUM(?v) AS ?sum) "
                        + "FROM NAMED WINDOW <http://ex/w> ON <" + SINGLE_STREAM + "> [RANGE PT20S STEP PT10S] "
                        + "WHERE { WINDOW <http://ex/w> { ?s <http://ex/value> ?v . } } GROUP BY ?s",
                    20, 10, new String[] {SINGLE_STREAM});
            case "tumbling_groupby_join":
                return new Scenario(
                    "REGISTER RSTREAM <http://ex/out> AS "
                        + "SELECT ?room (AVG(?v) AS ?avg) (COUNT(?v) AS ?n) "
                        + "FROM NAMED WINDOW <http://ex/w> ON <" + SINGLE_STREAM + "> [RANGE PT20S STEP PT20S] "
                        + "WHERE { WINDOW <http://ex/w> { ?s <http://ex/in> ?room . ?s <http://ex/value> ?v . } } "
                        + "GROUP BY ?room",
                    20, 20, new String[] {SINGLE_STREAM});
            default:
                throw new IllegalArgumentException("unknown scenario " + name);
        }
    }

    public static void main(String[] args) throws Exception {
        // keep stdout clean for the comparator contract: RSP4J logs via log4j 1.x,
        // whose default appender writes DEBUG lines to System.out
        org.apache.log4j.Logger.getRootLogger().setLevel(org.apache.log4j.Level.OFF);
        org.apache.log4j.BasicConfigurator.configure(new org.apache.log4j.varia.NullAppender());

        Map<String, String> opt = new HashMap<>();
        for (int i = 0; i + 1 < args.length; i += 2) {
            opt.put(args[i], args[i + 1]);
        }
        String replayPath = require(opt, "--replay");
        String scenarioName = require(opt, "--scenario");
        long scale = Long.parseLong(opt.getOrDefault("--time-scale", "1000"));
        // ts-offset shifts every replay event by a whole scaled unit so the oracle's
        // half-open [k*step, k*step+range) lines up as closely as Esper's (T-omega, T]
        // sliding window allows. See the alignment caveat in the header.
        long tsOffset = Long.parseLong(opt.getOrDefault("--ts-offset", "0"));
        // NonEmptyContent suppresses the report for an empty window; the oracle still
        // emits one AVG row there, so the empty-window case needs this turned off.
        boolean nonEmpty = !"false".equals(opt.get("--non-empty-content"));
        Scenario sc = scenario(scenarioName);

        // ---- parse the pinned replay (scaling timestamps) ----------------------
        List<Event> events = new ArrayList<>();
        long maxTs = 0;
        try (BufferedReader r = Files.newBufferedReader(Paths.get(replayPath), StandardCharsets.UTF_8)) {
            String line;
            while ((line = r.readLine()) != null) {
                if (line.isEmpty() || line.startsWith("#")) {
                    continue;
                }
                String[] c = line.split("\t");
                if (c.length != 5) {
                    throw new IllegalArgumentException("bad replay line (want 5 cols): " + line);
                }
                long ts = Long.parseLong(c[0]) * scale;
                maxTs = Math.max(maxTs, ts);
                events.add(new Event(ts, trim(c[1]), c[2], c[3], c[4]));
            }
        }

        // ---- boundary heartbeats (same declared protocol as the YASPER runner) --
        // Esper only processes its schedule when an arriving element advances the
        // external clock, so every window boundary needs a dummy graph (predicate
        // matching no query pattern) pushed just past it, on every input stream.
        List<Event> all = new ArrayList<>(events);
        int heartbeats = 0;
        for (long k = 0; k * sc.step * scale <= maxTs + sc.step * scale; k++) {
            long c = k * sc.step * scale + sc.range * scale;
            for (String stream : sc.streams) {
                all.add(new Event(c + 1, stream, "<http://ex/hb>", "<" + HB_P + ">", "<http://ex/hb>"));
                heartbeats++;
            }
        }
        all.sort((a, b) -> Long.compare(a.ts, b.ts)); // stable: same-ts keeps replay order

        // ---- engine configuration ----------------------------------------------
        // Written to a temp properties file because EngineConfiguration extends
        // commons-configuration PropertiesConfiguration (file-backed, not programmatic).
        // Entailment is deliberately UNSET: `jasper.entailment` absent => Entailment.NONE,
        // so no RDFS materialisation can inflate the per-window counts.
        Path props = Files.createTempFile("csparql2-replay", ".properties");
        try (PrintWriter w = new PrintWriter(Files.newBufferedWriter(props, StandardCharsets.UTF_8))) {
            w.println("rsp_engine.time=EventTime");
            w.println("rsp_engine.t0=ZERO");
            w.println("rsp_engine.base_uri=http://ex/");
            w.println("rsp_engine.stream.item.class=org.streamreasoning.rsp4j.csparql2.stream.GraphStreamSchema");
            w.println("rsp_engine.response_format=JSON-LD");
            w.println("rsp_engine.on_window_close=true");
            w.println("rsp_engine.non_empty_content=" + nonEmpty);
            w.println("rsp_engine.periodic=false");
            w.println("rsp_engine.on_content_change=false");
            w.println("rsp_engine.tick=TIME_DRIVEN");
            w.println("rsp_engine.report_grain=SINGLE");
            w.println("rsp_engine.sds.mantainance=NAIVE");
            w.println("rsp_engine.partialwindow=false");
        }
        EngineConfiguration ec = new EngineConfiguration(props.toString());
        SDSConfiguration sdsConfig = new SDSConfiguration(props.toString());

        // ---- engine wiring ------------------------------------------------------
        CSPARQLEngine engine = new CSPARQLEngine(0, ec);
        Map<String, DataStream<Graph>> writable = new HashMap<>();
        for (String s : sc.streams) {
            writable.put(s, engine.register(new DataStreamImpl<Graph>(s)));
        }
        JenaContinuousQueryExecution cqe =
            (JenaContinuousQueryExecution) engine.register(sc.rspql, sdsConfig);
        ContinuousQuery query = cqe.query();
        List<String> resultVars = ((org.apache.jena.query.Query) query).getResultVars();

        // Set-per-report-ts collection (set semantics, matching the YASPER runner). The
        // dedup key is the projection onto the query's SELECT vars ONLY: csparql2 merges
        // a ?processingTime binding carrying System.currentTimeMillis() into every row,
        // which would defeat dedup, and it re-evaluates once per notifying window.
        Map<Long, Set<String>> reports = new TreeMap<>();
        DataStream out = cqe.outstream();
        if (out != null) {
            out.addConsumer((el, ts) ->
                reports.computeIfAbsent((Long) ts, x -> new HashSet<>())
                    .addAll(projectKeys(el, resultVars)));
        }

        // ---- the timed replay push loop -----------------------------------------
        long t0 = System.nanoTime();
        for (Event e : all) {
            Graph g = GraphFactory.createGraphMem();
            g.add(Triple.create(node(e.s), node(e.p), node(e.o)));
            writable.get(e.stream).put(g, e.ts + tsOffset);
        }
        long wallNs = System.nanoTime() - t0;

        // ---- emit the comparator contract ----------------------------------------
        StringBuilder sb = new StringBuilder();
        sb.append("meta\tengine\trsp4j-csparql2\n");
        sb.append("meta\tversion\t").append(opt.getOrDefault("--version", "unpinned-local-build")).append('\n');
        sb.append("meta\tjava\t").append(System.getProperty("java.version")).append('\n');
        sb.append("meta\tr2r\tR2ROperatorSPARQL (Jena ARQ full SPARQL over the Esper-materialised SDS)\n");
        sb.append("meta\ttime_scale\t").append(scale).append('\n');
        sb.append("meta\tts_offset\t").append(tsOffset).append('\n');
        sb.append("meta\tnon_empty_content\t").append(nonEmpty).append('\n');
        sb.append("meta\theartbeats\t").append(heartbeats)
            .append(" dummy <" + HB_P + "> graphs at each window boundary + 1, per stream\n");
        sb.append("meta\tconsumer_dedup\tset-per-report-ts, keyed on the SELECT vars only\n");
        sb.append("meta\twindow_alignment\tEsper sliding win:time (T-range, T] snapshotted at the "
            + "boundary-crossing clock advance; NOT the oracle's aligned [k*step, k*step+range)\n");
        sb.append("meta\treplay_events\t").append(events.size()).append('\n');
        for (Map.Entry<Long, Set<String>> e : reports.entrySet()) {
            sb.append("report\t").append(e.getKey()).append('\t').append(e.getValue().size()).append('\n');
        }
        double secs = wallNs / 1e9;
        long tps = secs > 0 ? Math.round(events.size() / secs) : 0;
        sb.append("timing\trsp_replay_push_wall_us\t").append(wallNs / 1000).append("\tus\n");
        sb.append("timing\trsp_replay_push_triples_per_s\t").append(tps).append("\ttriples_per_s\n");
        System.out.print(sb);
        // Esper's engine holds non-daemon threads; the replay is done, so leave now.
        System.exit(0);
    }

    /**
     * Project one emitted output element onto the query's SELECT variables, dropping the
     * ?eventTime / ?processingTime bindings csparql2 merges into every result row.
     */
    private static List<String> projectKeys(Object el, List<String> resultVars) {
        List<String> keys = new ArrayList<>();
        if (el instanceof Table) {
            Iterator<Binding> it = ((Table) el).rows();
            while (it.hasNext()) {
                Binding b = it.next();
                StringBuilder k = new StringBuilder();
                for (String v : resultVars) {
                    Node n = b.get(Var.alloc(v));
                    k.append(v).append('=').append(n == null ? "UNBOUND" : n.toString()).append('|');
                }
                keys.add(k.toString());
            }
        } else {
            keys.add(String.valueOf(el));
        }
        return keys;
    }

    private static String require(Map<String, String> opt, String key) {
        String v = opt.get(key);
        if (v == null) {
            throw new IllegalArgumentException("missing required arg " + key);
        }
        return v;
    }

    private static String trim(String t) {
        return t.startsWith("<") && t.endsWith(">") ? t.substring(1, t.length() - 1) : t;
    }

    /** Minimal N-Triples term reader: IRI or typed literal ("lex"^^&lt;dt&gt;). */
    private static Node node(String t) {
        if (t.startsWith("<")) {
            return NodeFactory.createURI(trim(t));
        }
        int close = t.lastIndexOf("\"^^<");
        if (t.startsWith("\"") && close > 0 && t.endsWith(">")) {
            String lex = t.substring(1, close);
            String dt = t.substring(close + 4, t.length() - 1);
            return NodeFactory.createLiteral(lex, TypeMapper.getInstance().getSafeTypeByName(dt));
        }
        throw new IllegalArgumentException("unsupported term syntax: " + t);
    }

    private Csparql2ReplayRunner() {}
}
