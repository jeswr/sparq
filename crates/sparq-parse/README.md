<!-- [OPUS-4.8] sq-4kr5: internal-stub README for a publish=false crate. -->
# sparq-parse

**Compressed serialization of query results** for [sparq](../../README.md):
chunk-at-a-time gzip / zstd encoding of serialized result chunks, built around
one format property — *multi-member gzip and multi-frame zstd are valid single
streams*.

Why it exists: each serialized chunk is compressed as an independent gzip member
/ zstd frame, and the members are concatenated in order. The result decodes as
**one stream with stock decoders** (browser `Content-Encoding: gzip`,
`MultiGzDecoder`, `gzip -d`, the `zstd` CLI), so chunks can be compressed in
parallel as they are produced — compression overlaps serialization instead of
following it, and the first bytes hit the wire sooner.

Hard constraint: this crate must **not** enter `sparq-wasm`'s dependency graph
(flate2 / zstd / rayon stay out of the browser bundle).

## API sketch

```rust
use sparq_parse::{Codec, CompressedSink, Mode, decode_gzip_concat};

let chunks: Vec<String> = vec!["{\"head\":...".into(), "...rest".into()];
let mut sink = CompressedSink::new(Codec::Gzip { level: 6 }, Mode::Parallel);
let mut wire = Vec::new();
for c in &chunks {
    sink.push(c.as_bytes());
    for member in sink.try_drain().unwrap() {
        wire.extend_from_slice(&member); // stream to the client immediately
    }
}
for member in sink.finish().unwrap() {
    wire.extend_from_slice(&member);
}
assert_eq!(decode_gzip_concat(&wire).unwrap(), chunks.concat().into_bytes());
```

> **Internal crate — not on crates.io** (`publish = false`). The design is gated
> on a measured baseline; it is consumed inside the workspace, not as a
> standalone public API.

Design + baseline:
[`research/custom-parsers-D4-compressed-serialization.md`](../../research/custom-parsers-D4-compressed-serialization.md).
Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
