# Arrow result interop panel

This non-featured panel compares `sparq-arrow`'s real `to_record_batch` export path
with pyoxigraph solution serialization over two pinned SELECT workloads. Exact RDF
solution **multisets are checked before the stopwatch is created**. A mismatch emits
no timing envelope. If pyoxigraph is not installed, its column is absent.

```sh
python3 bench/arrow-interop/test_arrow_interop.py
cargo build -p sparq-arrow --features arrow --example arrow_interop --release
python3 bench/arrow-interop/arrow_interop.py \
  --sparq-bin target/release/examples/arrow_interop --json-out /tmp/arrow-interop.json
```

The envelope labels the sparq subprocess and pyoxigraph in-process scopes explicitly;
these are a loose interoperability read, not a claim of matched engine throughput.
