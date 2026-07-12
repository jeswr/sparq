# JSON-LD round-trip example

<!-- [GPT-5.6] sq-ne6wo: executable documentation for the native JSON-LD pipeline. -->

This standalone crate demonstrates the native `sparq-jsonld` document pipeline with
an inline document and no network access:

1. expand;
2. flatten;
3. compact against an inline context;
4. frame against an inline frame.

Each intermediate document is printed. The program also asserts that the final framed
document equals its inline expected result, so a divergence exits with a non-zero status.
`NoopLoader` ensures that an unexpected remote-document request fails closed.

Run it from the repository root:

```sh
cargo run --manifest-path examples/Cargo.toml
```

The crate has its own workspace boundary, so it remains opt-in and does not add anything
to the main workspace or the lean engine and WebAssembly dependency graphs.
