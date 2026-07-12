# SERVICE loopback differential

This self-contained harness implements bead `sq-139od` on the registered
`federation-fedshop` comparison axis. It evaluates committed SPARQL `SERVICE`
fixtures with sparq and Comunica over the same two real, in-process
`sparq-server` loopback endpoints.

The hard invariant is order-insensitive canonical solution-**multiset** equality.
The comparison preserves duplicate multiplicity, normalizes plain literals to
`xsd:string`, and rejects blank-node bindings rather than pretending labels from
independent engines are directly comparable. Every fixture also has a committed,
nonzero expected cardinality, preventing an empty/empty comparison from passing.

Run it from the repository root:

```sh
bash bench/service-fed-loopback/run.sh
```

The script reuses `bench/federation/comunica_runner.mjs` and installs Comunica
into that sibling directory at gather time when needed; `node_modules` remains
ignored. The Rust driver is a standalone Cargo workspace, so invoking this script
is the opt-in: ordinary workspace builds do not link the server or SERVICE client.

The result is a correctness-only JSON envelope under the ignored
`bench/competitor-results/` directory. Override the path with
`SERVICE_FED_ENVELOPE`. It records tool/host provenance and an honest table whose
verdict is `parity` or `behind`; a `behind` row includes the missing/extra
canonical bindings as its root cause and makes the script exit nonzero. It records
no performance measurements.

For the mutation-witnessed hermetic oracle test, run:

```sh
python3 bench/service-fed-loopback/compare.py --self-test
```

That test proves row order is ignored while duplicate removal and a
same-cardinality term substitution both turn equality red.
