// [GPT-5.6] sq-j4woz — framework-free captured CLI data for the static site.
//
// HONESTY CONTRACT: GitHub Pages has no sparq backend. Every stdout/stderr string below
// was captured verbatim by running its exact `command` from the repository root against
// CLI_FIXTURE. The two streams remain separate so the UI cannot blur diagnostics into
// query results. Re-run the commands and replace the captures; never invent output.

export const CLI_FIXTURE =
  `<http://example.org/Researcher> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://example.org/Person> .
<http://example.org/alice> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.org/Researcher> .
<http://example.org/alice> <http://example.org/knows> <http://example.org/bob> .
<http://example.org/bob> <http://example.org/knows> <http://example.org/carol> .
` as const;

export interface CliCapture {
  id: "query" | "reason" | "build" | "query-mmap";
  label: string;
  description: string;
  command: string;
  stdout: string;
  stderr: string;
}

export const CLI_CAPTURES: readonly CliCapture[] = [
  {
    id: "query",
    label: "query",
    description: "Load N-Triples in memory and emit W3C TSV bindings.",
    command:
      'cargo run -q -p sparq-cli -- query site/cli-demo.nt ntriples "SELECT ?person WHERE { ?person a <http://example.org/Researcher> }" --format tsv',
    stdout: `?person
<http://example.org/alice>
`,
    stderr:
      "loaded 4 triples in 0.016s (0.00 M/s) | store ~0.00 GB (271 B/triple), dict ~0.00 GB (8 terms, 92 B/term)\n",
  },
  {
    id: "reason",
    label: "reason",
    description:
      "Materialize the RDFS closure, including Alice's inferred Person type.",
    command:
      "cargo run -q -p sparq-cli -- reason site/cli-demo.nt ntriples rdfs",
    stdout: "5 triples after rdfs reasoning\n",
    stderr: "reasoned [Rdfs]: 4 -> 5 triples (+1 entailed) in 0.000s\n",
  },
  {
    id: "build",
    label: "build",
    description: "Build external-memory indexes for mmap-backed querying.",
    command:
      "cargo run -q -p sparq-cli -- build site/cli-demo.nt ntriples /tmp/sparq-cli-demo-index 1",
    stdout: "",
    stderr:
      "built on-disk indexes in /tmp/sparq-cli-demo-index in 0.0s (external-memory, 1M-triple runs)\n",
  },
  {
    id: "query-mmap",
    label: "query-mmap",
    description:
      "Open the persisted indexes with mmap and run the same SELECT.",
    command:
      'cargo run -q -p sparq-cli -- query-mmap /tmp/sparq-cli-demo-index "SELECT ?person WHERE { ?person a <http://example.org/Researcher> }" --format tsv',
    stdout: `?person
<http://example.org/alice>
`,
    stderr:
      "opened 4 triples (indexes MEMORY-MAPPED) in 0.005s | store-heap ~0.00 GB (mmap'd perms not counted), dict ~0.00 GB\n",
  },
] as const;

export function captureById(id: CliCapture["id"]): CliCapture | undefined {
  return CLI_CAPTURES.find((capture) => capture.id === id);
}
