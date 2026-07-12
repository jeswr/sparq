// AUTHORED-BY GPT-5.6
//! Regression tests for multipart byte-range evaluation and wire framing.

use sparq_lws_core::ldp::range::{
    encode_multipart, evaluate, ByteRange, RangeOutcome, MULTIPART_BOUNDARY,
};

#[test]
fn two_range_get_has_complete_multipart_wire_body() {
    let body = b"abcdefghij";
    let ranges = match evaluate(Some("bytes=0-1,7-9"), body.len() as u64) {
        RangeOutcome::Multipart(ranges) => ranges,
        other => panic!("expected multipart 206 decision, got {other:?}"),
    };
    let encoded = encode_multipart(body, "text/plain", &ranges);
    let wire = String::from_utf8(encoded).expect("ASCII multipart body");

    assert_eq!(MULTIPART_BOUNDARY, "sparq-lws-byte-boundary");
    assert!(wire.starts_with("--sparq-lws-byte-boundary\r\n"));
    assert!(wire.contains("Content-Type: text/plain\r\nContent-Range: bytes 0-1/10\r\n\r\nab\r\n"));
    assert!(wire.contains("Content-Type: text/plain\r\nContent-Range: bytes 7-9/10\r\n\r\nhij\r\n"));
    assert!(wire.ends_with("--sparq-lws-byte-boundary--\r\n"));
}

#[test]
fn overlapping_and_wholly_unsatisfiable_sets_fail_closed() {
    assert_eq!(
        evaluate(Some("bytes=0-5,5-8"), 10),
        RangeOutcome::Unsatisfiable
    );
    assert_eq!(
        evaluate(Some("bytes=20-30,40-50"), 10),
        RangeOutcome::Unsatisfiable
    );
    assert_eq!(
        evaluate(Some("bytes=0-1,broken"), 10),
        RangeOutcome::Unsatisfiable
    );
    assert_eq!(
        evaluate(Some("bytes=0-1,20-30"), 10),
        RangeOutcome::Satisfied { start: 0, end: 1 }
    );
}

#[test]
fn single_range_golden_is_unchanged() {
    assert_eq!(
        evaluate(Some("bytes=2-5"), 10),
        RangeOutcome::Satisfied { start: 2, end: 5 }
    );
    assert_eq!(evaluate(None, 10), RangeOutcome::Full);

    // Keep the public interval type exercised independently of enum construction.
    assert_eq!(ByteRange { start: 2, end: 5 }.end, 5);
}
