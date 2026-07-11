// Coverage-guided fuzz target for the SERVICE federation result parsers.
// Bead sq-3dyje.5; threat-model T-PARSE-FUZZ (remote-endpoint response bodies). [SONNET-4.6]
//
// SURFACES:
//   * SPARQL-Results-JSON (SRJ) — `parse_srj_into` / `parse_srj_into_read`
//   * SPARQL-Results-XML  (SRX) — `parse_srx_into` / `parse_srx_into_read`
//
// Both are reached through `eval_remote_into` / `eval_remote_into_read` (the public
// `sparq_engine_service::service` entry points) via a fake `Transport` that returns the
// fuzz bytes as the response body.  The content-sniff in `parse_results_into` dispatches
// on the first non-whitespace byte (`{` → SRJ, `<` → SRX) so both decoders are reachable
// from a single corpus.  The first byte of the fuzz input forces the sniff byte:
//
//   data[0] & 1 == 0  → prepend `{` → SRJ path
//   data[0] & 1 == 1  → prepend `<` → SRX path
//
// so libFuzzer covers both branches quickly.  The remaining bytes are the document body
// handed to the real decoder AS-IS (we do NOT convert from utf8_lossy: the parsers take
// `&str` so we use `String::from_utf8_lossy` to bridge, identical to what the existing
// parse_rdf_str + validate_shacl targets do).
//
// INVARIANT: hostile bytes must produce Ok(rows) or a clean Err — NEVER a panic,
// integer-overflow abort (overflow-checks on in this profile), allocator OOM-abort, or UB.
#![no_main]

use libfuzzer_sys::fuzz_target;
use sparq_engine_service::service::{eval_remote_into, Transport, TransportAsReader};

/// Fake transport that returns the supplied body verbatim.
struct FakeTransport<'a>(&'a str);

impl<'a> Transport for FakeTransport<'a> {
    fn fetch(&self, _endpoint: &str, _query: &str) -> Result<String, String> {
        Ok(self.0.to_string())
    }
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // Use the low bit of data[0] to choose the results format the content-sniff
    // dispatcher sees: SRJ (leading `{`) or SRX (leading `<`).
    let sniff_byte = if data[0] & 1 == 0 { b'{' } else { b'<' };
    let body_bytes = &data[1..];

    // Build the body string: sniff byte prepended to the fuzz payload interpreted as
    // (potentially lossy) UTF-8 — the same approach every existing sparq fuzz target uses
    // for byte → &str conversion.
    let mut body = String::with_capacity(1 + body_bytes.len());
    body.push(sniff_byte as char);
    body.push_str(&String::from_utf8_lossy(body_bytes));

    let transport = FakeTransport(&body);

    // String-body path (parse_results_into → parse_srj_into or parse_srx_into).
    let _ = eval_remote_into(
        &transport,
        "http://fuzz.invalid/sparql",
        "SELECT * WHERE { ?s ?p ?o }",
        // Row sink: accept every row, ignore the terms (we just want to exercise the
        // parse path without asserting anything about the terms themselves).
        &mut |_row| Ok(()),
    );

    // Reader-body path (parse_results_into_read → parse_srj_into_read or parse_srx_into_read).
    // Uses TransportAsReader<'_, FakeTransport> — the blanket adapter that wraps a Transport
    // impl as a ReaderTransport by returning an io::Cursor over the String body.
    let reader_transport = TransportAsReader(&transport);
    let _ = sparq_engine_service::service::eval_remote_into_read(
        &reader_transport,
        "http://fuzz.invalid/sparql",
        "SELECT * WHERE { ?s ?p ?o }",
        &mut |_row| Ok(()),
    );
});
