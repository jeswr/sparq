<!-- [SONNET-4.6] sq-qcnn.8 — external oracle harnesses for the differential fuzzer. -->
# Second-oracle harnesses

External SPARQL engines the differential fuzzer can consult **in addition to** its in-process
Oxigraph oracle. Node F of
[`research/differential-testing-value-level.md`](../../../research/differential-testing-value-level.md)
(bead `sq-qcnn.8`).

## Why

The fuzzer checks sparq against one reference implementation. That catches a sparq-only bug but is
structurally blind to a bug sparq and Oxigraph **share** — a spec clause both authors read the same
wrong way. An engine of unrelated lineage is the mitigation.

**A second oracle is not an oracle of truth.** Jena and rdflib are implementations, not the
specification. When two oracles disagree, that is evidence of a spec ambiguity or of one engine's
non-conformance; it is **not** attributable to sparq, and the harness never treats it as a sparq
defect — it counts and prints it. Turning such a case into a reviewed, checked-in allowlist entry
with a written reason is design record §5.2 and a separate bead.

## Enabling one

Off by default: absent `SPARQ_FUZZ_ORACLE2_CMD`, `sparq-bench fuzz` makes exactly the comparisons
it always made, spawns no process, and needs no JVM and no Python — it only adds one banner line
recording that no second oracle was consulted.

| Variable | Meaning |
| --- | --- |
| `SPARQ_FUZZ_ORACLE2_CMD` | Program plus fixed args, whitespace-separated. **Unset = no second oracle.** |
| `SPARQ_FUZZ_ORACLE2_NAME` | Display name (default: the program's basename). Cosmetic. |
| `SPARQ_FUZZ_ORACLE2_TIMEOUT_SECS` | Per-query budget (default 60s; must absorb JVM start-up). |

The adapter appends the two arguments below to whatever `…_CMD` names, so a harness is invoked as
`<cmd> <data.ttl> <query.rq>`.

```sh
# Apache Jena
javac -cp "$JENA_HOME/lib/*" -d build jena/SparqlOracle.java
SPARQ_FUZZ_ORACLE2_NAME=jena \
SPARQ_FUZZ_ORACLE2_CMD="java -cp build:$JENA_HOME/lib/* SparqlOracle" \
  cargo run -p sparq-bench -- fuzz 0 500 all

# rdflib (optional third; cheaper to provision, less conformant ⇒ more triage)
pip install rdflib
SPARQ_FUZZ_ORACLE2_NAME=rdflib \
SPARQ_FUZZ_ORACLE2_CMD="python3 rdflib/sparql_oracle.py" \
  cargo run -p sparq-bench -- fuzz 0 500 all
```

The run then prints an extra `oracle2[<name>] agree=… disagree=… skipped=…` line, plus the first
oracle-vs-oracle divergence for triage. That line is deliberately **separate** from the `fuzz[…]`
summary, which `scripts/ci-file-differential-failure.py` scrapes and which must stay
byte-compatible.

## Wire protocol

`<program> <fixed args…> <data.ttl> <query.rq>`, with:

* **exit 0** — stdout is a SPARQL-Results-JSON document (`SELECT` bindings or an `ASK` boolean).
  Nothing else may go to stdout; diagnostics go to stderr.
* **exit 3** — "I cannot evaluate this query" (parse error, unimplemented feature, or a
  `CONSTRUCT`/`DESCRIBE`, which SPARQL Results JSON cannot carry). A **skip** for this oracle,
  never a divergence.
* **exit 2** — bad invocation; **any other non-zero exit, a timeout, or unparseable stdout** — a
  backend fault. Exiting 0 and writing garbage is classified as a *broken oracle*, not a decline,
  so a silently failing harness cannot read as an innocuous skip.

`CONSTRUCT`/`DESCRIBE` are out of scope for this protocol on purpose: comparing graph results
across a subprocess needs an N-Triples channel plus RDFC-1.0 isomorphism, and half-building it
here would put the two sides of the comparison on different wire forms.

## Verification status — read before trusting these

The Rust side (`src/oracle.rs`) is unit-tested, including the subprocess adapter's argv passing,
stdout capture, timeout kill, and its decline/break/garbage classification — exercised against
stub child processes, so those tests need no JVM and no rdflib.

**The two harnesses in this directory are not.** Neither was compiled nor executed when it was
written (the development box has no JVM and no rdflib), and **no CI lane runs them**. They are
provided as the adapter's reference implementations and should be treated as unverified until
someone provisions the toolchain and runs them. Standing up that nightly lane — and only then
gating on the `oracle2` counters — is follow-on work, not part of this change.
