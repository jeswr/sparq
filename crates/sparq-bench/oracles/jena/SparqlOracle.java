// [SONNET-4.6] sq-qcnn.8 — the Apache Jena subprocess oracle for the sparq differential fuzzer.
//
// Node F of research/differential-testing-value-level.md: a SECOND independent SPARQL engine, so
// the harness is no longer structurally blind to a bug sparq and Oxigraph SHARE. Jena is the most
// mature independent SPARQL 1.1 implementation and shares no lineage with either Rust engine —
// which is the entire point of choosing it.
//
// Wire protocol (see crates/sparq-bench/src/oracle.rs, `SubprocessOracle`):
//
//   argv[0] = path to a Turtle data file
//   argv[1] = path to a SPARQL query file
//
//   exit 0 — stdout is a SPARQL-Results-JSON document (SELECT bindings or an ASK boolean)
//   exit 3 — "I cannot evaluate this query": a parse error, an unimplemented feature, or a
//            CONSTRUCT/DESCRIBE (SPARQL Results JSON has no graph form). A SKIP for this oracle,
//            never a divergence.
//   exit 2 — bad invocation. Any other non-zero exit is a backend failure.
//
// Nothing but the JSON document may go to stdout — every diagnostic goes to stderr, because the
// adapter parses stdout in full and treats unparseable output from a zero exit as a BROKEN
// oracle (deliberately, so a silently failing oracle cannot read as an innocuous skip).

import java.io.FileInputStream;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Paths;

import org.apache.jena.query.Query;
import org.apache.jena.query.QueryException;
import org.apache.jena.query.QueryExecution;
import org.apache.jena.query.QueryExecutionFactory;
import org.apache.jena.query.QueryFactory;
import org.apache.jena.query.ResultSetFormatter;
import org.apache.jena.rdf.model.Model;
import org.apache.jena.rdf.model.ModelFactory;
import org.apache.jena.riot.Lang;
import org.apache.jena.riot.RDFDataMgr;

public final class SparqlOracle {

    /** The adapter's "I cannot evaluate this" exit code. Must match `UNSUPPORTED_EXIT_CODE`. */
    private static final int UNSUPPORTED = 3;

    private SparqlOracle() {
    }

    public static void main(String[] args) {
        if (args.length != 2) {
            System.err.println("usage: SparqlOracle <data.ttl> <query.rq>");
            System.exit(2);
        }

        Model model = ModelFactory.createDefaultModel();
        try (InputStream in = new FileInputStream(args[0])) {
            // Read the SAME serialisation both Rust engines were handed. A failure here is a
            // backend fault (exit 1), not a decline: all three engines get identical bytes, so
            // one of them rejecting them is itself worth surfacing.
            RDFDataMgr.read(model, in, Lang.TURTLE);
        } catch (Exception e) {
            System.err.println("data: " + e);
            System.exit(1);
        }

        String queryText;
        try {
            queryText = new String(Files.readAllBytes(Paths.get(args[1])), StandardCharsets.UTF_8);
        } catch (Exception e) {
            System.err.println("reading query: " + e);
            System.exit(1);
            return; // unreachable; keeps the compiler's definite-assignment analysis happy
        }

        try {
            Query query = QueryFactory.create(queryText);
            if (!query.isSelectType() && !query.isAskType()) {
                // CONSTRUCT / DESCRIBE: not representable in SPARQL Results JSON. Declining is
                // correct — inventing a graph encoding here would put the two sides of the
                // comparison on different wire forms.
                System.err.println("unsupported result form: " + query.getQueryType());
                System.exit(UNSUPPORTED);
            }
            try (QueryExecution qe = QueryExecutionFactory.create(query, model)) {
                if (query.isAskType()) {
                    ResultSetFormatter.outputAsJSON(System.out, qe.execAsk());
                } else {
                    ResultSetFormatter.outputAsJSON(System.out, qe.execSelect());
                }
            }
            System.out.flush();
        } catch (QueryException e) {
            // QueryParseException extends QueryException, so this covers both "will not parse"
            // and "cannot execute" — the decline bucket.
            System.err.println("unsupported: " + e.getMessage());
            System.exit(UNSUPPORTED);
        } catch (Exception e) {
            System.err.println("backend: " + e);
            System.exit(1);
        }
    }
}
