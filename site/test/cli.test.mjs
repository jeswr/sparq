// [GPT-5.6] sq-j4woz — pins the real sparq-cli transcripts used by the static walkthrough.
import { test } from "node:test";
import assert from "node:assert/strict";

import { CLI_CAPTURES, CLI_FIXTURE, captureById } from "../src/lib/cli.ts";

test("the declared fixture is the exact four-triple capture input", () => {
  assert.equal(CLI_FIXTURE.split("\n").filter(Boolean).length, 4);
  assert.match(CLI_FIXTURE, /rdf-schema#subClassOf/);
  assert.match(CLI_FIXTURE, /rdf-syntax-ns#type/);
});

test("the four exact cargo-run command shapes are pinned", () => {
  assert.deepEqual(
    CLI_CAPTURES.map(({ id, command }) => [id, command]),
    [
      [
        "query",
        'cargo run -q -p sparq-cli -- query site/cli-demo.nt ntriples "SELECT ?person WHERE { ?person a <http://example.org/Researcher> }" --format tsv',
      ],
      [
        "reason",
        "cargo run -q -p sparq-cli -- reason site/cli-demo.nt ntriples rdfs",
      ],
      [
        "build",
        "cargo run -q -p sparq-cli -- build site/cli-demo.nt ntriples /tmp/sparq-cli-demo-index 1",
      ],
      [
        "query-mmap",
        'cargo run -q -p sparq-cli -- query-mmap /tmp/sparq-cli-demo-index "SELECT ?person WHERE { ?person a <http://example.org/Researcher> }" --format tsv',
      ],
    ],
  );
  for (const capture of CLI_CAPTURES) {
    assert.match(capture.command, /^cargo run -q -p sparq-cli -- /);
    assert.ok(
      !capture.command.includes("\n"),
      `${capture.id}: command must be one shell line`,
    );
  }
});

test("stdout and stderr serialization is byte-for-byte pinned", () => {
  assert.equal(
    JSON.stringify(
      CLI_CAPTURES.map(({ id, stdout, stderr }) => ({ id, stdout, stderr })),
    ),
    '[{"id":"query","stdout":"?person\\n<http://example.org/alice>\\n","stderr":"loaded 4 triples in 0.016s (0.00 M/s) | store ~0.00 GB (271 B/triple), dict ~0.00 GB (8 terms, 92 B/term)\\n"},{"id":"reason","stdout":"5 triples after rdfs reasoning\\n","stderr":"reasoned [Rdfs]: 4 -> 5 triples (+1 entailed) in 0.000s\\n"},{"id":"build","stdout":"","stderr":"built on-disk indexes in /tmp/sparq-cli-demo-index in 0.0s (external-memory, 1M-triple runs)\\n"},{"id":"query-mmap","stdout":"?person\\n<http://example.org/alice>\\n","stderr":"opened 4 triples (indexes MEMORY-MAPPED) in 0.005s | store-heap ~0.00 GB (mmap\'d perms not counted), dict ~0.00 GB\\n"}]',
  );
});

test("captures preserve stream shape and mmap/query result parity", () => {
  for (const capture of CLI_CAPTURES) {
    assert.ok(capture.stdout.endsWith("\n") || capture.stdout === "");
    assert.ok(capture.stderr.endsWith("\n") || capture.stderr === "");
  }
  assert.equal(captureById("query").stdout, captureById("query-mmap").stdout);
  assert.equal(captureById("build").stdout, "");
});
