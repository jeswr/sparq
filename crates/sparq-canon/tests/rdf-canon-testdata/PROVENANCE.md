# W3C RDF Dataset Canonicalization (RDFC-1.0) test suite — vendored snapshot

- Source: https://github.com/w3c/rdf-canon (`tests/` directory)
- Snapshot commit: `15619df2fda7a4ca88308733789b6774517f9638` (2026-02-24)
- Fetched: 2026-06-12
- Files: `manifest.ttl` + `rdfc10/` (inputs `testNNN-in.nq`, expected canonical
  outputs `testNNN-rdfc10.nq`, expected issued-identifier maps
  `testNNN-rdfc10map.json`)
- License: distributed under the W3C Test Suite License and the W3C 3-clause
  BSD License (see the header of `manifest.ttl`).
- Consumed by: `crates/sparq-canon/tests/rdf_canon_suite.rs` (manifest-driven;
  the manifest is parsed with sparq itself). Validates the `sparq-canon`
  public API (the single-sourced RDFC-1.0 surface; `sparq-zk` depends on it).

Do not edit these files; refresh by re-vendoring a newer upstream snapshot.
