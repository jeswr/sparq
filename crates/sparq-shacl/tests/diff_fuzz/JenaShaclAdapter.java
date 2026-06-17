// [OPUS-4.8] sq-evws: Apache Jena SHACL reference-engine adapter for the SHACL
// differential fuzzer (crates/sparq-shacl/tests/diff_fuzz.rs).
//
// A second, independent reference alongside the pySHACL adapter (sq-55c1). It is
// the SAME "report-cli" contract (bead sq-eifd): JSON request in, normalised JSON
// report out, so the Rust runner's comparison is engine-agnostic. A second engine
// catches bugs where sparq and pySHACL happen to agree but are both wrong.
//
// WHY a single .java file (no compile step): Java 11+ runs a source file directly
// via the single-file source launcher — `java -cp <jena-libs> JenaShaclAdapter.java`.
// So the differential ships ONLY source, compiled-and-run in one shot per case
// (the per-case-spawn isolation the pySHACL adapter uses), and no `.class`/jar is
// committed. The Jena jars come from an unpacked apache-jena tarball cached in CI.
//
// CONTRACT (stdin -> stdout) — identical to tests/diff_fuzz/pyshacl_adapter.py:
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
//            adapter/engine ERROR (so the runner distinguishes "engine says X" from
//            "engine could not run"), with a diagnostic on stderr.
//
// Reads ONE request and emits ONE report (one JVM per case, mirroring the pySHACL
// adapter's one-interpreter-per-case isolation).
import java.io.StringReader;
import java.nio.charset.StandardCharsets;

import org.apache.jena.graph.Graph;
import org.apache.jena.graph.Node;
import org.apache.jena.graph.Triple;
import org.apache.jena.riot.Lang;
import org.apache.jena.riot.RDFParser;
import org.apache.jena.shacl.ShaclValidator;
import org.apache.jena.shacl.Shapes;
import org.apache.jena.shacl.ValidationReport;
import org.apache.jena.sparql.graph.GraphFactory;
import org.apache.jena.vocabulary.RDF;

public final class JenaShaclAdapter {
    static final String SH = "http://www.w3.org/ns/shacl#";

    public static void main(String[] args) {
        // [OPUS-4.8] (sq-vsqr) `--selftest` exercises the sh:detail exclusion on a
        // synthetic report graph (no validator run): proves a nested sub-result that
        // is the object of an sh:detail is dropped from the top-level violation set,
        // independent of whether THIS Jena build emits sh:detail. Exit 0 on pass,
        // non-zero with a diagnostic on fail. See selfTest().
        if (args.length == 1 && "--selftest".equals(args[0])) {
            System.exit(selfTest() ? 0 : 3);
        }
        try {
            byte[] in = System.in.readAllBytes();
            String req = new String(in, StandardCharsets.UTF_8);
            String data = jsonField(req, "data");
            String shapes = jsonField(req, "shapes");

            Graph dataG = GraphFactory.createDefaultGraph();
            RDFParser.create().source(new StringReader(data)).lang(Lang.TURTLE).parse(dataG);
            Graph shapesG = GraphFactory.createDefaultGraph();
            RDFParser.create().source(new StringReader(shapes)).lang(Lang.TURTLE).parse(shapesG);

            Shapes sh = Shapes.parse(shapesG);
            ValidationReport report = ShaclValidator.get().validate(sh, dataG);
            System.out.print(normaliseGraph(report.getGraph(), report.conforms()));
            System.out.print("\n");
            System.exit(0);
        } catch (Throwable t) {
            System.err.println("jena_shacl_adapter: engine error: " + t);
            System.exit(1);
        }
    }

    // [OPUS-4.8] (sq-vsqr) Build a report graph with ONE top-level result and ONE
    // nested sh:detail sub-result, then assert normaliseGraph emits exactly the
    // top-level violation (the detail is excluded). A regression that dropped the
    // exclusion would surface the nested sub-result as a second violation here.
    static boolean selfTest() {
        Graph g = GraphFactory.createDefaultGraph();
        Node top = org.apache.jena.graph.NodeFactory.createBlankNode("top");
        Node inner = org.apache.jena.graph.NodeFactory.createBlankNode("inner");
        Node resultT = NodeFactoryUri(SH + "ValidationResult");
        Node focusOuter = NodeFactoryUri("http://ex/focus");
        Node focusInner = NodeFactoryUri("http://ex/value");
        Node compOuter = NodeFactoryUri(SH + "NodeConstraintComponent");
        Node compInner = NodeFactoryUri(SH + "MinLengthConstraintComponent");
        Node pathOuter = NodeFactoryUri("http://ex/p");
        // Outer (top-level) result, carrying a nested sh:detail to the inner result.
        g.add(Triple.create(top, RDF.type.asNode(), resultT));
        g.add(Triple.create(top, NodeFactoryUri(SH + "focusNode"), focusOuter));
        g.add(Triple.create(top, NodeFactoryUri(SH + "sourceConstraintComponent"), compOuter));
        g.add(Triple.create(top, NodeFactoryUri(SH + "resultPath"), pathOuter));
        g.add(Triple.create(top, NodeFactoryUri(SH + "detail"), inner));
        // Inner (nested) result — must be EXCLUDED from the top-level set.
        g.add(Triple.create(inner, RDF.type.asNode(), resultT));
        g.add(Triple.create(inner, NodeFactoryUri(SH + "focusNode"), focusInner));
        g.add(Triple.create(inner, NodeFactoryUri(SH + "sourceConstraintComponent"), compInner));

        String got = normaliseGraph(g, false);
        String want = "{\"conforms\":false,\"violations\":["
            + "{\"focus\":\"http://ex/focus\","
            + "\"component\":\"http://www.w3.org/ns/shacl#NodeConstraintComponent\","
            + "\"path\":\"http://ex/p\"}]}";
        if (!want.equals(got)) {
            System.err.println("jena_shacl_adapter selftest FAIL: sh:detail exclusion");
            System.err.println("  want: " + want);
            System.err.println("  got:  " + got);
            return false;
        }
        return true;
    }

    static String normaliseGraph(Graph g, boolean conforms) {
        Node pFocus = NodeFactoryUri(SH + "focusNode");
        Node pComp = NodeFactoryUri(SH + "sourceConstraintComponent");
        Node pPath = NodeFactoryUri(SH + "resultPath");
        Node pDetail = NodeFactoryUri(SH + "detail");
        Node resultT = NodeFactoryUri(SH + "ValidationResult");

        // [OPUS-4.8] (sq-vsqr) Collect only TOP-LEVEL results — mirror of the
        // pySHACL adapter's sq-0hj7 exclusion. For sh:node / logical components a
        // reference engine emits an outer result (against the value node, carrying
        // sh:resultPath) AND an OPTIONAL nested `sh:detail` sub-result for the inner
        // constraint that failed. `sh:detail` is non-normative (SHACL §3.6.2 "MAY"),
        // and sparq-shacl keeps inner results off the comparable top-level `results`
        // list (they live in `ValidationResult::details`). Diffing the nested
        // sub-results would compare two engines' OPTIONAL detail policies, not a real
        // disagreement (a latent over-report) — so we exclude any result that is the
        // object of an sh:detail.
        java.util.Set<Node> nested = new java.util.HashSet<>();
        var di = g.find(Node.ANY, pDetail, Node.ANY);
        while (di.hasNext()) {
            nested.add(di.next().getObject());
        }

        StringBuilder sb = new StringBuilder();
        sb.append("{\"conforms\":").append(conforms ? "true" : "false");
        sb.append(",\"violations\":[");
        boolean first = true;
        var it = g.find(Node.ANY, RDF.type.asNode(), resultT);
        while (it.hasNext()) {
            Triple t = it.next();
            Node res = t.getSubject();
            if (nested.contains(res)) {
                continue;
            }
            Node focus = objectOf(g, res, pFocus);
            Node comp = objectOf(g, res, pComp);
            Node path = objectOf(g, res, pPath);
            if (!first) sb.append(",");
            first = false;
            sb.append("{\"focus\":").append(termKey(focus));
            sb.append(",\"component\":").append(termKey(comp));
            sb.append(",\"path\":").append(pathKey(path));
            sb.append("}");
        }
        sb.append("]}");
        return sb.toString();
    }

    static Node NodeFactoryUri(String uri) {
        return org.apache.jena.graph.NodeFactory.createURI(uri);
    }

    static Node objectOf(Graph g, Node s, Node p) {
        var it = g.find(s, p, Node.ANY);
        if (it.hasNext()) return it.next().getObject();
        return null;
    }

    static String termKey(Node n) {
        if (n == null) return "null";
        if (n.isBlank()) return jsonStr("_:bnode");
        if (n.isURI()) return jsonStr(n.getURI());
        if (n.isLiteral()) return jsonStr(n.getLiteralLexicalForm());
        return jsonStr(n.toString());
    }

    static String pathKey(Node n) {
        if (n == null) return "null";
        if (n.isURI()) return jsonStr(n.getURI());
        // Complex / blank-rooted path: same sentinel the pySHACL adapter emits.
        return jsonStr("_:path");
    }

    // Minimal JSON string emit (escapes the characters that appear in IRIs/labels).
    static String jsonStr(String s) {
        StringBuilder b = new StringBuilder("\"");
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            switch (c) {
                case '"': b.append("\\\""); break;
                case '\\': b.append("\\\\"); break;
                case '\n': b.append("\\n"); break;
                case '\r': b.append("\\r"); break;
                case '\t': b.append("\\t"); break;
                default:
                    if (c < 0x20) b.append(String.format("\\u%04x", (int) c));
                    else b.append(c);
            }
        }
        b.append("\"");
        return b.toString();
    }

    // Tiny JSON-object field extractor for {"data": "...", "shapes": "..."}.
    // The request is produced by serde_json with simple string values, so a
    // dependency-free unescaper suffices (no nested objects/arrays).
    static String jsonField(String json, String field) {
        String needle = "\"" + field + "\"";
        int k = json.indexOf(needle);
        if (k < 0) throw new IllegalArgumentException("missing field: " + field);
        int i = json.indexOf('"', k + needle.length());
        if (i < 0) throw new IllegalArgumentException("no opening quote for: " + field);
        StringBuilder sb = new StringBuilder();
        for (int p = i + 1; p < json.length(); p++) {
            char c = json.charAt(p);
            if (c == '\\') {
                char n = json.charAt(++p);
                switch (n) {
                    case 'n': sb.append('\n'); break;
                    case 'r': sb.append('\r'); break;
                    case 't': sb.append('\t'); break;
                    case '"': sb.append('"'); break;
                    case '\\': sb.append('\\'); break;
                    case '/': sb.append('/'); break;
                    case 'u':
                        int cp = Integer.parseInt(json.substring(p + 1, p + 5), 16);
                        sb.append((char) cp);
                        p += 4;
                        break;
                    default: sb.append(n);
                }
            } else if (c == '"') {
                return sb.toString();
            } else {
                sb.append(c);
            }
        }
        throw new IllegalArgumentException("unterminated string for: " + field);
    }
}
