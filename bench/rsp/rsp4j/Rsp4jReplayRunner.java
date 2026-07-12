// [FABLE-5] (sq-hmd7l.20) RSP4J/YASPER replay driver — the engine half of the bounded
// count-matched-replay RSP comparability protocol
// (research/comparative-benchmarking-everything.md sec 5.2).
//
// Drives RSP4J's YASPER reference implementation (operator API + TPQueryFactory
// RSP-QL dialect) with the IDENTICAL pinned timestamped replay used by sparq's
// clock-free oracle (bench/rsp/replay/*.ts.tsv), in RSP4J's event-time
// configuration: `DataStream.put(graph, ts)` with the replay's own timestamps —
// no wall clock, no sleeps. Emits one `report\t<t_e>\t<distinct_rows>` line per
// window evaluation on stdout; bench/rsp/rsp4j_compare.py maps reports to oracle
// window indices and runs the count-match gate BEFORE admitting any timing row.
//
// Protocol details this runner declares (all recorded in the envelope):
//   * BOUNDARY HEARTBEATS: YASPER's C-SPARQL S2R reports OnWindowClose STRICTLY
//     (w.getC() < t_e) and only on element arrival, so the driver pushes one dummy
//     heartbeat graph (predicate <http://ex/heartbeat>, matching no query pattern)
//     to EVERY input stream at each window boundary + 1. This closes each window
//     exactly once and keeps every stream's window sequence populated; the shared
//     TimeImpl suppresses duplicate same-ts evaluations across streams.
//   * TIME SCALE: the RSP-QL dialect only parses whole-second durations (PT10S =
//     10000 ms), so replay timestamps are multiplied by --time-scale (default
//     1000). Order and relative spacing are unchanged — it is the same replay.
//   * CONSUMER DEDUP: result rows are collected as a SET per report timestamp
//     (set semantics, matching both engines' per-window relation semantics).
//
// COUNT-COMPARABLE SURFACE (honest): YASPER's TP dialect has no GROUP BY /
// SUM / AVG, so only scenarios whose per-window ROW COUNT is expressible as a
// distinct-binding count are count-comparable. That is `srbench_join` (the
// SRBench-shaped multi-window observation-to-station-metadata join, SELECT of all
// three pattern variables). The aggregate scenarios are excluded and reported as
// NOT-COUNT-COMPARABLE-IN-DIALECT — see research/gap-rsp-2026-07.md.
//
// GATHER-TIME ONLY: compiled by bench/rsp/gather-rsp4j.sh against locally built
// RSP4J jars (pinned in that script); NOT compiled by CI. Verified compiling and
// count-matching against rsp4j master @ c46e0f674 (jar version 1.0.1) on
// 2026-07-11 (non-canonical box; counts are machine-independent).
//
// Usage:
//   java Rsp4jReplayRunner --replay bench/rsp/replay/srbench.ts.tsv \
//        --scenario srbench_join [--time-scale 1000] [--version <jar-version>]

import org.apache.commons.rdf.api.Graph;
import org.apache.commons.rdf.api.IRI;
import org.apache.commons.rdf.api.RDF;
import org.apache.commons.rdf.api.RDFTerm;
import org.streamreasoning.rsp4j.api.RDFUtils;
import org.streamreasoning.rsp4j.api.querying.ContinuousQuery;
import org.streamreasoning.rsp4j.operatorapi.ContinuousProgram;
import org.streamreasoning.rsp4j.operatorapi.QueryTaskOperatorAPIImpl;
import org.streamreasoning.rsp4j.operatorapi.TaskOperatorAPIImpl;
import org.streamreasoning.rsp4j.yasper.examples.RDFStream;
import org.streamreasoning.rsp4j.yasper.querying.operators.r2r.Binding;
import org.streamreasoning.rsp4j.yasper.querying.operators.r2r.joins.HashJoinAlgorithm;
import org.streamreasoning.rsp4j.yasper.querying.syntax.TPQueryFactory;

import java.io.BufferedReader;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;

public final class Rsp4jReplayRunner {

    private static final String HB_P = "http://ex/heartbeat";

    /** One replay event: (scaled ts, stream iri, s, p, o as N-Triples-ish strings). */
    private record Event(long ts, String stream, String s, String p, String o) {}

    /** Scenario config: RSP-QL text + window maths for heartbeat placement. */
    private record Scenario(String rspql, long range, long step, String[] streams) {}

    private static Scenario scenario(String name, long scale) {
        if (name.equals("srbench_join")) {
            // The count-comparable SRBench scenario: same window spec + patterns as
            // crates/sparq-rsp/examples/rsp_oracle.rs srbench_queries() "join"
            // (SELECT of ALL pattern variables => row count = distinct joined
            // bindings under both engines' set semantics).
            return new Scenario(
                "REGISTER RSTREAM <http://ex/out> AS "
                    + "SELECT ?st ?state ?v "
                    + "FROM NAMED WINDOW <http://ex/wo> ON <http://ex/obs> [RANGE PT10S STEP PT10S] "
                    + "FROM NAMED WINDOW <http://ex/wm> ON <http://ex/meta> [RANGE PT10S STEP PT10S] "
                    + "WHERE { "
                    + "  WINDOW <http://ex/wo> { ?st <http://ex/value> ?v . } "
                    + "  WINDOW <http://ex/wm> { ?st <http://ex/state> ?state . } "
                    + "}",
                10 * scale,
                10 * scale,
                new String[] {"http://ex/obs", "http://ex/meta"});
        }
        throw new IllegalArgumentException(
            "scenario "
                + name
                + " is not count-comparable in YASPER's TP dialect (no GROUP BY/SUM/AVG);"
                + " see research/gap-rsp-2026-07.md");
    }

    public static void main(String[] args) throws IOException {
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
        Scenario sc = scenario(scenarioName, scale);

        // ---- parse the pinned replay (scaling timestamps) ---------------------
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
                    throw new IOException("bad replay line (want 5 cols): " + line);
                }
                long ts = Long.parseLong(c[0]) * scale;
                maxTs = Math.max(maxTs, ts);
                events.add(new Event(ts, trim(c[1]), c[2], c[3], c[4]));
            }
        }

        // ---- boundary heartbeats (declared protocol, see header) --------------
        // one per window-close boundary C = k*step + range for every oracle window
        // (k*step <= max replay ts), at C + 1, on every input stream.
        List<Event> heartbeats = new ArrayList<>();
        for (long k = 0; k * sc.step() <= maxTs; k++) {
            long c = k * sc.step() + sc.range();
            for (String stream : sc.streams()) {
                heartbeats.add(new Event(c + 1, stream, "<http://ex/hb>", "<" + HB_P + ">", "<http://ex/hb>"));
            }
        }
        List<Event> all = new ArrayList<>(events);
        all.addAll(heartbeats);
        all.sort((a, b) -> Long.compare(a.ts(), b.ts())); // stable: same-ts keeps replay order

        // ---- engine wiring -----------------------------------------------------
        RDF rdf = RDFUtils.getInstance();
        Map<String, RDFStream> streams = new HashMap<>();
        for (String s : sc.streams()) {
            streams.put(s, new RDFStream(s));
        }
        ContinuousQuery<Graph, Graph, Binding, Binding> query = TPQueryFactory.parse(sc.rspql());
        TaskOperatorAPIImpl<Graph, Graph, Binding, Binding> task =
            new QueryTaskOperatorAPIImpl.QueryTaskBuilder().fromQuery(query).build();
        ContinuousProgram.ContinuousProgramBuilder<Graph, Graph, Binding, Binding> builder =
            new ContinuousProgram.ContinuousProgramBuilder<Graph, Graph, Binding, Binding>()
                .addTask(task)
                .out(query.getOutputStream())
                .addJoinAlgorithm(new HashJoinAlgorithm());
        for (RDFStream s : streams.values()) {
            builder.in(s);
        }
        builder.build();

        // set-per-report-ts collection (set semantics; a duplicate evaluation of the
        // same window at the same t_e collapses instead of double-counting)
        Map<Long, Set<String>> reports = new TreeMap<>();
        query.getOutputStream().addConsumer(
            (el, ts) -> reports.computeIfAbsent(ts, x -> new HashSet<>()).add(String.valueOf(el)));

        // ---- the timed replay push loop ---------------------------------------
        long t0 = System.nanoTime();
        for (Event e : all) {
            Graph g = rdf.createGraph();
            g.add(rdf.createTriple((IRI) term(rdf, e.s()), (IRI) term(rdf, e.p()), term(rdf, e.o())));
            streams.get(e.stream()).put(g, e.ts());
        }
        long wallNs = System.nanoTime() - t0;

        // ---- emit the comparator contract --------------------------------------
        StringBuilder out = new StringBuilder();
        out.append("meta\tengine\trsp4j-yasper\n");
        out.append("meta\tversion\t").append(opt.getOrDefault("--version", "unpinned-local-build")).append('\n');
        out.append("meta\tjava\t").append(System.getProperty("java.version")).append('\n');
        out.append("meta\ttime_scale\t").append(scale).append('\n');
        out.append("meta\theartbeats\t").append(heartbeats.size())
            .append(" dummy <" + HB_P + "> graphs at each window boundary + 1, per stream\n");
        out.append("meta\tconsumer_dedup\tset-per-report-ts\n");
        out.append("meta\treplay_events\t").append(events.size()).append('\n');
        for (Map.Entry<Long, Set<String>> e : reports.entrySet()) {
            out.append("report\t").append(e.getKey()).append('\t').append(e.getValue().size()).append('\n');
        }
        double secs = wallNs / 1e9;
        long tps = secs > 0 ? Math.round(events.size() / secs) : 0;
        out.append("timing\trsp_replay_push_wall_us\t").append(wallNs / 1000).append("\tus\n");
        out.append("timing\trsp_replay_push_triples_per_s\t").append(tps).append("\ttriples_per_s\n");
        System.out.print(out);
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

    /** Minimal N-Triples term reader: IRI or typed literal ("lex"^^<dt>). */
    private static RDFTerm term(RDF rdf, String t) {
        if (t.startsWith("<")) {
            return rdf.createIRI(trim(t));
        }
        int close = t.lastIndexOf("\"^^<");
        if (t.startsWith("\"") && close > 0 && t.endsWith(">")) {
            String lex = t.substring(1, close);
            String dt = t.substring(close + 4, t.length() - 1);
            return rdf.createLiteral(lex, rdf.createIRI(dt));
        }
        throw new IllegalArgumentException("unsupported term syntax: " + t);
    }

    private Rsp4jReplayRunner() {}
}
