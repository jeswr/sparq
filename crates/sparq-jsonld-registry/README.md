# sparq-jsonld-registry

`sparq-jsonld-registry` provides a small, bundled registry of well-known JSON-LD 1.1
contexts. It implements `sparq_jsonld::DocumentLoader` without filesystem or network
access. Exact registered IRIs resolve from bytes compiled into the crate; every other IRI
fails closed with the same error as `NoopLoader`.

The crate is experimental, unpublished, and opt-in: consumers depend on it explicitly.
See `contexts/PROVENANCE.md` for the source and licensing record of each bundled context.

